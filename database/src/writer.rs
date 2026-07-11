use crate::{BlockInfo, CHUNK_SIZE, Cache, ChunkData, open_or_create, secondary};
use blake2::{Blake2b256, Digest};
use ledger::block;
use once_cell::sync::OnceCell;
use std::{
    hint::cold_path,
    io,
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
        block::Shell {
            bytes,
            header,
            id,
            slot,
            number,
        }: block::Shell<'_>,
    ) -> io::Result<()> {
        let chunk_number = slot / CHUNK_SIZE;
        let mut cache = self.0.read().expect("cache should not be poisoned");
        // Only the writer updates `len_size`, so `Relaxed` is OK.
        let (mut len, mut size) = cache.len_size(atomic::Ordering::Relaxed);

        if chunk_number > cache.chunk_number {
            let (new_chunk_file, new_primary_file, new_secondary_file) =
                open_or_create(&cache.directory, chunk_number)?;

            // Safety: `len` is the number of initialized entries in `entries`, so `entries[0..len]`
            // are all initialized.
            let block_info: Box<[_]> = unsafe {
                std::slice::from_raw_parts(cache.entries.get().cast::<BlockInfo>(), len).into()
            };

            drop(cache);
            let mut cache_mut = self.0.write().expect("cache should not be poisoned");
            // TODO: do all of this at the end?
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
            *cache_mut.len_size.get_mut() = 0;
            drop(cache_mut);
            cache = self.0.read().expect("cache should not be poisoned");
        } else if chunk_number != cache.chunk_number
            || (relative_slot as usize) < self.primary_count
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "cannot append block with slot {slot_or_ebb} because more recent blocks exist"
                ),
            ));
        }

        let secondary_offset = (len * std::mem::size_of::<secondary::Entry>()) as u32;
        cache.chunk_file.write_all_at(bytes, size as u64)?;
        let crc = crc32fast::hash(bytes);
        let entry = secondary::Entry {
            offset: u64::from(size).into(),
            header_offset: (header.start as u16).into(),
            header_size: ((header.end - header.start) as u16).into(),
            crc: crc.into(),
            id,
            slot: slot.into(),
        };
        cache
            .secondary_file
            .write_all_at(entry.as_bytes(), u64::from(secondary_offset))?;

        // All failable operations are done, we can update the cache.

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
                    slot,
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
