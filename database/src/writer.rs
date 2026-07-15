use crate::{BlockInfo, CHUNK_SIZE, Cache, ChunkData, open_or_create, secondary};
use once_cell::sync::OnceCell;
use std::{
    fs::File,
    io,
    os::unix::fs::FileExt,
    range::Range,
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
        header: Range<usize>,
        id: [u8; 32],
        slot_or_ebb: u64,
    ) -> io::Result<()> {
        let crc = crc32fast::hash(bytes);
        let write_block = |len, size, chunk_file: &File, secondary_file: &File| -> io::Result<()> {
            let secondary_offset = (len * std::mem::size_of::<secondary::Entry>()) as u32;
            chunk_file.write_all_at(bytes, size as u64)?;
            let entry = secondary::Entry {
                offset: u64::from(size).into(),
                header_offset: (header.start as u16).into(),
                header_size: ((header.end - header.start) as u16).into(),
                crc: crc.into(),
                id,
                slot: slot_or_ebb.into(),
            };
            chunk_file.sync_data()?;
            secondary_file.write_all_at(entry.as_bytes(), u64::from(secondary_offset))
        };

        let chunk_number = slot_or_ebb / CHUNK_SIZE;
        let guard = self.0.read().expect("cache should not be poisoned");
        let mut cache: &Cache<_> = &guard;
        let mut cache_mut;
        // Only the writer updates `len_size`, so `Relaxed` is OK.
        let (block_info, mut size, _) = cache.current_chunk_data(atomic::Ordering::Relaxed);
        let mut len = block_info.len();
        let last_slot = block_info.last().map(|info| info.slot);

        if chunk_number > cache.chunk_number
            || (slot_or_ebb == chunk_number + 1 && last_slot.is_none_or(|slot| slot_or_ebb < slot))
        {
            let block_info = block_info.into();
            let (new_chunk_file, new_secondary_file) =
                open_or_create(&cache.directory, chunk_number)?;
            write_block(0, 0u32, &new_chunk_file, &new_secondary_file)?;

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
                cache_mut.pointer =
                    (pointer + (chunk_number - cache_mut.chunk_number) as usize) % N;
            }
            cache_mut.chunk_number = chunk_number;
            len = 0;
            size = 0;
            cache = &cache_mut;
        } else if chunk_number == cache.chunk_number
            && last_slot.is_none_or(|slot| slot_or_ebb > slot)
        {
            write_block(len, size, &cache.chunk_file, &cache.secondary_file)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "block slot {slot_or_ebb} is less than or equal to the tip of the database {last_slot:?}"
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
