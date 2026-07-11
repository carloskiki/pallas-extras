use crate::{CHUNK_SIZE, Cache, ChunkData, path_prefix, read_buf, secondary};
use bytes::{Bytes, BytesMut};
use ledger::slot;
use std::{
    fs::OpenOptions,
    io,
    ops::Range,
    sync::{Arc, RwLock},
};

pub struct Reader<const N: usize>(pub(super) Arc<RwLock<Cache<N>>>);

impl<const N: usize> Reader<N> {
    /// Obtain the slot of the last block in the database, if any.
    pub fn tip(&self) -> Option<slot::Number> {
        self.0
            .read()
            .expect("cache should not be poisoned")
            .current_chunk_data()
            .0
            .last()
            .map(|info| info.slot)
    }

    /// Reads a chunk of blocks from the database and appends them to the provided buffer, and
    /// updating the slot `range`.
    ///
    /// The chunk constitutes a contiguous prefix of blocks of from the slot `range`. In rare cases,
    /// the read may be empty, but it does not imply that there are no more blocks to read from the
    /// slot `range`. Only `range.is_empty()` implies that there are no more blocks to read.
    ///
    /// ## Errors
    ///
    /// Returns an error on file system errors, or if the database is corrupted (e.g. crc mismatch).
    pub fn read(
        &self,
        mut range: Range<slot::Number>,
    ) -> impl Iterator<Item = io::Result<impl Iterator<Item = Bytes>>> {
        // FIXME: When `try_blocks` is stable we won't need this...
        macro_rules! some_try {
            ($expr:expr) => {
                match $expr {
                    Ok(value) => value,
                    Err(error) => return Some(Err(error)),
                }
            };
        }

        std::iter::from_fn(move || {
            let chunk_number = range.start / CHUNK_SIZE;
            let mut buffer = BytesMut::new();
            let cache = self.0.read().expect("cache should not be poisoned");
            if range.is_empty() || chunk_number > cache.chunk_number {
                return None;
            }

            // Read the chunk data
            let mut read_chunk = || -> io::Result<ChunkData> {
                let path = path_prefix(&cache.directory, chunk_number);
                let mut options = OpenOptions::new();
                options.read(true).write(true).create(true);
                let chunk_file = options.open(path.with_extension("chunk"))?;
                let secondary_file = options.open(path.with_extension("secondary"))?;
                let size = chunk_file.metadata()?.len() as u32;
                let block_info = secondary::read(&mut buffer, &secondary_file)?;

                Ok(ChunkData {
                    block_info,
                    size,
                    file: chunk_file,
                })
            };
            let chunk;
            let (block_info, size, file) = if chunk_number == cache.chunk_number {
                cache.current_chunk_data()
            } else {
                let diff = (cache.chunk_number - chunk_number) as usize;
                let ChunkData {
                    block_info,
                    size,
                    file,
                } = if diff > N {
                    // The chunk is too old to be in cache.
                    chunk = some_try!(read_chunk());
                    &chunk
                } else {
                    let (mut index, underflow) = cache.pointer.overflowing_sub(diff);
                    if underflow {
                        index = index.wrapping_add(N);
                    }
                    some_try!(cache.completed[index].get_or_try_init(|| {
                        // Chunk not loaded yet.
                        read_chunk()
                    }))
                };
                (block_info.as_ref(), *size, file)
            };

            // TODO: if the first slot is == to chunk_number, then its an ebb, so mult by chunk
            // number.
            let start = block_info.partition_point(|info| info.slot < range.start);
            let stop = block_info.partition_point(|info| info.slot < range.end);

            let read_start = block_info.get(start).map_or(size, |info| info.offset);
            let read_stop = block_info.get(stop).map_or(size, |info| info.offset);
            let read_size = (read_stop - read_start) as usize;
            buffer.reserve(read_size);
            some_try!(read_buf(file, &mut buffer, read_start as u64, read_size));

            range.start = ((chunk_number + 1) as slot::Number) * CHUNK_SIZE;
            Some(Ok(block_info[start..stop].iter().enumerate().map(
                move |(i, &info)| {
                    let next = block_info
                        .get(start + i + 1)
                        .map_or(size, |info| info.offset);
                    let block_size = (next - info.offset) as usize;
                    // We don't do crc32 check because:
                    // 1. If corruption happens, then the block will be invalid.
                    // 2. If an attacker can modify the chunk file, they can also modify the crc32
                    //    in the secondary file.
                    // 3. Even if the check is cheap, there would also be potential
                    //    errors on each block in the inner iterator (annoying).
                    buffer.split_to(block_size).freeze()
                },
            )))
        })
    }
}
