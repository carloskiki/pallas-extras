use crate::{BlockInfo, CHUNK_SIZE, Cache, ChunkData, open_or_create, secondary};
use once_cell::sync::OnceCell;
use std::{
    fs::File,
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
    /// Append a block to the database.
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
