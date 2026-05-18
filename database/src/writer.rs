use crate::{BlockInfo, CHUNK_SIZE, Cache, ChunkData, open_or_create, secondary};
use blake2::{Blake2b256, Digest};
use ledger::Block;
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
use tinycbor::{CborLen, encoded::With};
use zerocopy::IntoBytes;

pub struct Writer<const N: usize> {
    /// Shared data.
    pub(crate) shared: Arc<RwLock<Cache<N>>>,
    /// Primary file relative slot.
    ///
    /// This is used to determine how much to backfill the primary file. The last block's slot
    /// cannot always be used for this because EBBs are represented with slot number `CHUNK_SIZE *
    /// chunk_number`. This is ambiguous with the first slot of the chunk, so there could be a
    /// regular block and an EBB with the same slot number. If the current chunk contains a single
    /// block with that slot, we can't know whether this is the EBB (offset index 0) or the regular
    /// block (offset index 1). So we store
    pub(crate) primary_count: usize,
}

impl<const N: usize> Writer<N> {
    /// Append a block to the database.
    ///
    /// If an error occurs the database is not corrupted and the write is not applied.
    pub fn append(&mut self, block: &With<'_, Block>) -> io::Result<()> {
        let mut hasher = Blake2b256::new();
        let bytes: &[u8] = block.as_ref();
        const HEADER_OFFSET: usize = 3;
        macro_rules! shelley_data {
            ($block:ident, $bytes:ident) => {
                (
                    $block.header.body.slot,
                    $block.header.cbor_len(),
                    Blake2b256::digest(
                        &$bytes[HEADER_OFFSET..$block.header.cbor_len() + HEADER_OFFSET],
                    )
                    .into(),
                )
            };
        }

        let (slot_or_ebb, header_size, hash) = match block.as_ref() {
            // EBBs are always the first block in a chunk. The database must line up the chunk size
            // to contain exactly one byron epoch of blocks for that to happen, hence `epoch ==
            // chunk_number`.
            Block::Boundary(b) => (b.header.consensus_data.epoch, b.header.cbor_len(), {
                hasher.update([0x82, 0x00]);
                hasher.update(&bytes[HEADER_OFFSET..b.header.cbor_len() + HEADER_OFFSET]);
                hasher.finalize().into()
            }),
            Block::Byron(b) => (
                b.header.consensus_data.slot.epoch * CHUNK_SIZE + b.header.consensus_data.slot.slot,
                b.header.cbor_len(),
                {
                    hasher.update([0x82, 0x01]);
                    hasher.update(&bytes[HEADER_OFFSET..b.header.cbor_len() + HEADER_OFFSET]);
                    hasher.finalize().into()
                },
            ),
            Block::Shelley(b) => shelley_data!(b, bytes),
            Block::Allegra(b) => shelley_data!(b, bytes),
            Block::Mary(b) => shelley_data!(b, bytes),
            Block::Alonzo(b) => shelley_data!(b, bytes),
            Block::Babbage(b) => shelley_data!(b, bytes),
            Block::Conway(b) => shelley_data!(b, bytes),
        };
        let (slot, relative_slot) = if matches!(block.as_ref(), Block::Boundary(_)) {
            (slot_or_ebb * CHUNK_SIZE, 0)
        } else {
            (slot_or_ebb, slot_or_ebb % CHUNK_SIZE + 1)
        };
        let chunk_number = (slot / CHUNK_SIZE) as u32;
        let mut cache = self.shared.read().expect("cache should not be poisoned");
        let primary_file_offset = (1 + self.primary_count * std::mem::size_of::<u32>()) as u64;
        // Only the writer updates `len_size`, so `Relaxed` is OK.
        let (mut len, mut size) = cache.len_size(atomic::Ordering::Relaxed);

        if chunk_number > cache.chunk_number {
            cache.primary_file.write_all_at(
                &size
                    .to_be_bytes()
                    .repeat((CHUNK_SIZE as usize + 2) - self.primary_count),
                primary_file_offset,
            )?;
            for i in (cache.chunk_number + 1)..chunk_number {
                cold_path();
                let (new_chunk_file, new_primary_file, _) = open_or_create(&cache.directory, i)?;
                new_primary_file.write_all_at(
                    &[0; (CHUNK_SIZE as usize + 2) * std::mem::size_of::<u32>()],
                    1,
                )?;
                if N > 0 {
                    cache.completed[(cache.pointer + (i - cache.chunk_number) as usize - 1) % N]
                        .set(ChunkData {
                            block_info: Box::new([]),
                            size: 0,
                            file: new_chunk_file,
                        })
                        .ok();
                }
            }
            let (new_chunk_file, new_primary_file, new_secondary_file) =
                open_or_create(&cache.directory, chunk_number)?;

            drop(cache);
            let mut cache_mut = self.shared.write().expect("cache should not be poisoned");
            let old_chunk_file = std::mem::replace(&mut cache_mut.chunk_file, new_chunk_file);
            cache_mut.secondary_file = new_secondary_file;
            cache_mut.primary_file = new_primary_file;
            self.primary_count = 0;
            if N > 0 {
                let chunk = ChunkData {
                    block_info: cache_mut
                        .entries
                        .get_mut()
                        .iter()
                        .take(len)
                        .map(|entry| {
                            // Safety: `len` is the number of initialized entries in `entries`. These are
                            // never modified after initialization, so we can safely read them.
                            unsafe { (*entry).assume_init() }
                        })
                        .collect(),
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
            drop(cache_mut);
            cache = self.shared.read().expect("cache should not be poisoned");
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
        cache.primary_file.write_all_at(
            &secondary_offset
                .to_be_bytes()
                .repeat(relative_slot as usize - self.primary_count + 1),
            primary_file_offset,
        )?;
        cache.chunk_file.write_all_at(bytes, size as u64)?;
        let crc = crc32fast::hash(bytes);
        let entry = secondary::Entry {
            offset: (size as u64).into(),
            header_offset: (HEADER_OFFSET as u16).into(),
            header_size: (header_size as u16).into(),
            crc: crc.into(),
            hash,
            slot: slot_or_ebb.into(),
        };
        cache
            .secondary_file
            .write_all_at(entry.as_bytes(), secondary_offset as u64)?;

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
        self.primary_count = relative_slot as usize + 1;
        // Syncrhonizes with the `Acquire` load of the reader.
        cache.len_size.store(
            (size as u64) << 32 | (len as u64),
            atomic::Ordering::Release,
        );
        Ok(())
    }
}
