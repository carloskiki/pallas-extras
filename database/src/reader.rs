use crate::{CHUNK_SIZE, Cache, ChunkData, read_buf, secondary};
use bytes::{Bytes, BytesMut};
use ledger::slot;
use std::{
    borrow::Cow,
    fs::File,
    io,
    ops::Range,
    sync::{Arc, RwLock, RwLockReadGuard, atomic},
};

use super::open_or_create;

/// A reader handle for the database.
///
/// This handle is cheap to clone and can be used concurrently.
#[derive(Clone)]
pub struct Reader<const N: usize>(pub(super) Arc<RwLock<Cache<N>>>);

impl<const N: usize> Reader<N> {
    /// Obtain the slot of the last block in the database, if any.
    pub fn tip(&self) -> Option<slot::Number> {
        let cache = self.0.read().expect("cache should not be poisoned");
        cache
            .current_chunk_data(atomic::Ordering::Acquire)
            .0
            .last()
            .map(|info| {
                if info.slot == cache.chunk_number {
                    info.slot * CHUNK_SIZE
                } else {
                    info.slot
                }
            })
    }

    /// Returns a streaming reader for chunks from the provided slot range.
    ///
    /// Keeping [`Read`] alive prevents new chunks from being written to the database. This can
    /// prevent the [`Writer`](crate::Writer) from making progress if it needs to write across a
    /// chunk boundary.
    pub fn read(&self, range: Range<slot::Number>) -> Read<'_, N> {
        Read {
            cache: self.0.read().expect("cache should not be poisoned"),
            buffer: BytesMut::new(),
            range,
        }
    }
}

/// Streaming reader.
///
/// Every call to [`Read::next`] reads the next chunk of blocks from the database.
pub struct Read<'a, const N: usize> {
    cache: RwLockReadGuard<'a, Cache<N>>,
    buffer: BytesMut,
    range: Range<slot::Number>,
}

impl<const N: usize> Read<'_, N> {
    /// Read the next chunk of blocks from the database.
    ///
    /// If this returns `None`, the end of the range has been reached.
    ///
    /// ### Errors
    ///
    /// `Some(Err(_))` is returned when a file system operation fails. This skips the chunk that was
    /// attempted to be read and continues.
    pub fn next<'a>(&'a mut self) -> Option<io::Result<Blocks<'a>>> {
        // FIXME: When `try_blocks` is stable we won't need this...
        macro_rules! some_try {
            ($expr:expr) => {
                match $expr {
                    Ok(value) => value,
                    Err(error) => return Some(Err(error)),
                }
            };
        }
        let mut read_chunk = |chunk_number| {
            let (chunk_file, secondary_file) = open_or_create(&self.cache.directory, chunk_number)?;
            let size = chunk_file.metadata()?.len() as u32;
            let block_info = secondary::read(&mut self.buffer, &secondary_file)?;
            self.buffer.clear();
            Ok(ChunkData {
                block_info,
                size,
                file: chunk_file,
            })
        };

        let chunk_number = self.range.start / CHUNK_SIZE;
        if self.range.is_empty() || chunk_number > self.cache.chunk_number {
            return None;
        }
        let old_start = self.range.start;
        self.range.start = ((chunk_number + 1) as slot::Number) * CHUNK_SIZE;

        let owned_file: File;
        let (block_info, size, file) = if chunk_number == self.cache.chunk_number {
            let (block_info, size, file) = self.cache.current_chunk_data(atomic::Ordering::Acquire);
            (Cow::Borrowed(block_info), size, file)
        } else {
            let diff = (self.cache.chunk_number - chunk_number) as usize;
            if diff > N {
                // The chunk is too old to be in cache.
                let ChunkData {
                    block_info,
                    size,
                    file,
                } = some_try!(read_chunk(
                    chunk_number,
                ));
                owned_file = file;
                (Cow::Owned(block_info.into_vec()), size, &owned_file)
            } else {
                let mut index = self.cache.pointer.wrapping_sub(diff);
                if index >= N {
                    index = index.wrapping_add(N);
                }
                let chunk = some_try!(self.cache.completed[index].get_or_try_init(|| {
                    // Chunk not loaded yet.
                    read_chunk(chunk_number)
                }));
                (
                    Cow::Borrowed(chunk.block_info.as_ref()),
                    chunk.size,
                    &chunk.file,
                )
            }
        };

        let start = if old_start.is_multiple_of(CHUNK_SIZE) {
            // We skip partitioning if the start of the range includes the full chunk, since EBBs
            // have `slot_or_ebb` which is less than the chunk's first slot. This ensures that we
            // include EBBS.
            0
        } else {
            block_info.partition_point(|info| info.slot < old_start)
        };
        let stop = block_info.partition_point(|info| info.slot < self.range.end);

        let read_start = block_info.get(start).map_or(size, |info| info.offset);
        let read_stop = block_info.get(stop).map_or(size, |info| info.offset);
        let read_size = (read_stop - read_start) as usize;
        some_try!(read_buf(
            file,
            &mut self.buffer,
            read_start as u64,
            read_size
        ));

        Some(Ok(Blocks {
            buffer: self.buffer.split_to(read_size),
            block_info,
            index: start,
            stop,
            size,
        }))
    }
}

/// Iterator over blocks in a chunk.
///
/// The iterator yields `Result<Bytes, Bytes>` where the `Ok` variant contains the block bytes that
/// were validated against a checksum, and the `Err` variant contains the block bytes that failed
/// checksum verification. If the checksum fails, the database is considered corrupted.
pub struct Blocks<'a> {
    buffer: BytesMut,
    block_info: Cow<'a, [crate::BlockInfo]>,
    size: u32,
    index: usize,
    stop: usize,
}

impl Iterator for Blocks<'_> {
    type Item = Result<Bytes, Bytes>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.stop {
            return None;
        }

        let block_size = {
            let info = self.block_info[self.index];
            let next = self
                .block_info
                .get(self.index + 1)
                .map_or(self.size, |info| info.offset);
            (next - info.offset) as usize
        };
        self.index += 1;

        let bytes = self.buffer.split_to(block_size).freeze();
        Some(
            if crc32fast::hash(&bytes) == self.block_info[self.index - 1].crc {
                Ok(bytes)
            } else {
                Err(bytes)
            },
        )
    }
}
