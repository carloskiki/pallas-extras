use crate::{BlockInfo, CHUNK_SIZE, Cache, ChunkData, open_or_create, path_prefix, secondary};
use bytes::BytesMut;
use once_cell::sync::OnceCell;
use std::{
    fs::{self, File},
    io,
    ops::Range,
    os::unix::fs::FileExt,
    sync::{
        Arc, RwLock,
        atomic::{self},
    },
};
use zerocopy::IntoBytes;

/// Database writer.
pub struct Writer<const N: usize>(pub(super) Arc<RwLock<Cache<N>>>);

impl<const N: usize> Writer<N> {
    /// Truncate the database after `slot`.
    ///
    /// Blocks at `slot` are retained. If the database tip is before `slot`, this is a no-op.
    ///
    /// Keeping a [`Read`](crate::reader::Read) alive while calling this method prevents it from
    /// making progress until the read is dropped.
    ///
    /// ## Errors
    ///
    /// Returns an error on file system errors.
    pub fn truncate(&mut self, slot: u64) -> io::Result<()> {
        fn remove_file_if_exists(path: impl AsRef<std::path::Path>) -> io::Result<()> {
            match fs::remove_file(path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
            }
        }

        let target_chunk_number = slot / CHUNK_SIZE;
        let mut cache = self.0.write().expect("cache should not be poisoned");

        if target_chunk_number > cache.chunk_number {
            return Ok(());
        }

        if target_chunk_number == cache.chunk_number {
            // Only the writer updates `len_size`, so `Relaxed` is OK while holding the lock.
            let (len, _) = cache.len_size(atomic::Ordering::Relaxed);
            // Safety: the write lock prevents readers and `append` from accessing `entries`.
            let entries =
                unsafe { std::slice::from_raw_parts(cache.entries.get().cast::<BlockInfo>(), len) };
            let new_len = entries.partition_point(|info| info.slot <= slot);
            if new_len == len {
                return Ok(());
            }
            let new_size = entries[new_len].offset;

            cache
                .secondary_file
                .set_len((new_len * std::mem::size_of::<secondary::Entry>()) as u64)?;
            // Publish the smaller initialized prefix before allowing another append or read.
            cache.len_size.store(
                (u64::from(new_size) << 32) | new_len as u64,
                atomic::Ordering::Release,
            );
            // Extra chunk bytes are harmless if shrinking the file fails: the secondary index and
            // cache already describe the truncated database, and a subsequent append overwrites
            // them.
            cache.chunk_file.set_len(u64::from(new_size))?;
            return Ok(());
        }

        let (chunk_file, secondary_file) = open_or_create(&cache.directory, target_chunk_number)?;
        let size = chunk_file.metadata()?.len() as u32;
        let mut buffer = BytesMut::new();
        let mut entries = secondary::read(&mut buffer, &secondary_file)?.into_vec();
        let new_len = entries.partition_point(|info| info.slot <= slot);
        let new_size = entries.get(new_len).map_or(size, |info| info.offset);
        entries.truncate(new_len);
        let old_chunk_number = cache.chunk_number;

        // Remove secondary files first because `open` uses them to find the database tip.
        for chunk_number in (target_chunk_number + 1)..=old_chunk_number {
            remove_file_if_exists(
                path_prefix(&cache.directory, chunk_number).with_extension("secondary"),
            )?;
        }
        secondary_file.set_len((entries.len() * std::mem::size_of::<secondary::Entry>()) as u64)?;

        cache.chunk_file = chunk_file;
        cache.secondary_file = secondary_file;
        cache.chunk_number = target_chunk_number;
        let rollback = old_chunk_number - target_chunk_number;
        if rollback >= N as u64 {
            // The new current chunk predates every cached chunk.
            for completed in &mut cache.completed {
                *completed = OnceCell::new();
            }
            cache.pointer = 0;
        } else {
            let mut pointer = cache.pointer.wrapping_sub(rollback as usize);
            if pointer >= N {
                pointer = pointer.wrapping_add(N);
            }

            // Moving the pointer preserves the mapping for chunks older than the new current
            // chunk. Clear the slots which held the new current chunk and the newer chunks being
            // removed; otherwise they would be mistaken for older chunks after the pointer moves.
            let mut index = pointer;
            for _ in 0..rollback {
                cache.completed[index] = OnceCell::new();
                index += 1;
                if index == N {
                    index = 0;
                }
            }
            cache.pointer = pointer;
        }
        for (destination, entry) in cache.entries.get_mut().iter_mut().zip(entries) {
            destination.write(entry);
        }
        cache.len_size.store(
            (u64::from(new_size) << 32) | new_len as u64,
            atomic::Ordering::Release,
        );

        // As with truncating the current chunk, these files are no longer logically reachable if
        // cleanup fails.
        cache.chunk_file.set_len(u64::from(new_size))?;
        for chunk_number in (target_chunk_number + 1)..=old_chunk_number {
            remove_file_if_exists(
                path_prefix(&cache.directory, chunk_number).with_extension("chunk"),
            )?;
        }

        Ok(())
    }

    /// Append a block to the database.
    ///
    /// This does not validate any of the block's structure or contents. The caller is responsible
    /// for ensuring that the `bytes`, `header`, `id`, and `slot` are consistent with each other.
    ///
    /// ## Errors
    ///
    /// Returns an error on file system errors, or if the block's slot is less than the tip of the
    /// database.
    pub fn append(
        &mut self,
        bytes: &[u8],
        header: Range<u16>,
        id: [u8; 32],
        slot_or_ebb: u64,
    ) -> io::Result<()> {
        let crc = crc32fast::hash(bytes);
        let write_block =
            |chunk_len, chunk_size, chunk_file: &File, secondary_file: &File| -> io::Result<()> {
                chunk_file.write_all_at(bytes, u64::from(chunk_size))?;
                let secondary_offset = chunk_len * std::mem::size_of::<secondary::Entry>() as u64;
                let entry = secondary::Entry {
                    offset: u64::from(chunk_size).into(),
                    header_offset: header.start.into(),
                    header_size: (header.end - header.start).into(),
                    crc: crc.into(),
                    id,
                    slot: slot_or_ebb.into(),
                };
                secondary_file.write_all_at(entry.as_bytes(), secondary_offset)
            };

        let guard = self.0.read().expect("cache should not be poisoned");
        let mut cache: &Cache<_> = &guard;
        let mut cache_mut;
        // Only the writer updates `len_size`, so `Relaxed` is OK.
        let (block_info, mut size, _) = cache.current_chunk_data(atomic::Ordering::Relaxed);
        let mut len = block_info.len();
        let last_slot = block_info.last().and_then(|info| {
            if info.slot == cache.chunk_number && block_info.len() == 1 {
                (info.slot * CHUNK_SIZE).checked_sub(1)
            } else {
                Some(info.slot)
            }
        });

        // FIXME: We only accept appending to the current chunk or next chunk. We don't allow
        // appending to a chunk further ahead. This may cause issue if for example the blockchain
        // goes down for a long time and then resumes at a slot much later.
        let chunk_number = if slot_or_ebb == cache.chunk_number + 1
            && last_slot.is_none_or(|slot| slot_or_ebb <= slot)
        {
            slot_or_ebb
        } else {
            slot_or_ebb / CHUNK_SIZE
        };
        if chunk_number == cache.chunk_number + 1 {
            let (new_chunk_file, new_secondary_file) =
                open_or_create(&cache.directory, chunk_number)?;
            write_block(0u64, 0u32, &new_chunk_file, &new_secondary_file)?;

            let block_info = block_info.into();
            // Acquire a write lock to update the cache.
            drop(guard);
            cache_mut = self.0.write().expect("cache should not be poisoned");
            let old_chunk_file = std::mem::replace(&mut cache_mut.chunk_file, new_chunk_file);
            cache_mut.secondary_file = new_secondary_file;
            if N > 0 {
                let chunk = ChunkData {
                    block_info,
                    size,
                    file: old_chunk_file,
                };
                let pointer = cache_mut.pointer;
                cache_mut.completed[pointer] = OnceCell::from(chunk);
                cache_mut.pointer = (pointer + 1) % N;
            }
            cache_mut.chunk_number = chunk_number;
            len = 0;
            size = 0;
            cache = &cache_mut;
        } else if chunk_number == cache.chunk_number
            && last_slot.is_none_or(|slot| slot_or_ebb > slot)
        {
            write_block(len as u64, size, &cache.chunk_file, &cache.secondary_file)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "block slot {slot_or_ebb} does not follow the tip of the database {last_slot:?}",
                ),
            ));
        }

        // Safety:
        // - `append` ensures that at most `CHUNK_SIZE + 1` blocks are written to the current chunk
        //   by checking that `relative_slot >= primary_count`.
        // => `len < CHUNK_SIZE + 1` here, so `len` is in bounds.
        // - `append` is the only function that accesses `entries` at `len`.
        // - `Writer: !Clone` and `append(&mut self)` ensure that there are no concurrent calls
        //   to `append`.
        // => We have exclusive access to `entries[len]`.
        unsafe {
            cache
                .entries
                .get()
                .cast::<BlockInfo>()
                .add(len)
                .write(BlockInfo {
                    slot: slot_or_ebb,
                    offset: size,
                    crc,
                });
        };
        len += 1;
        size += bytes.len() as u32;
        // Syncrhonizes with the `Acquire` load of the reader.
        cache.len_size.store(
            (size as u64) << 32 | (len as u64),
            atomic::Ordering::Release,
        );
        Ok(())
    }
}
