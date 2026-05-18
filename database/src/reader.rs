use crate::{CHUNK_SIZE, Cache, read_buf};
use bytes::{Bytes, BytesMut};
use ledger::{Block, slot};
use std::{
    io,
    ops::Range,
    sync::{Arc, RwLock},
};
use tinycbor::encoded::Lazy;

/// Reader for blocks from the database.
pub struct Reader<const N: usize> {
    /// The shared data.
    pub(crate) shared: Arc<RwLock<Cache<N>>>,
    /// Buffer used to read from files.
    pub(crate) buffer: BytesMut,
    /// The range of slots to read.
    pub range: Range<slot::Number>,
}

impl<const N: usize> Reader<N> {
    pub fn tip(&self) -> io::Result<Option<slot::Number>> {
        let cache = self.shared.read().expect("cache should not be poisoned");
        let mut buffer = BytesMut::new();

        for chunk_number in (0..=cache.chunk_number).rev() {
            let (blocks, _, _) = cache.chunk_data(chunk_number, &mut buffer)?;
            if let Some(last) = blocks.last() {
                return Ok(Some(last.slot));
            }
        }

        Ok(None)
    }

    pub fn read_chunk(&mut self, buffer: &mut Vec<Lazy<Bytes, Block<'static>>>) -> io::Result<()> {
        let chunk_number = (self.range.start / CHUNK_SIZE) as u32;
        let cache = self.shared.read().expect("cache should not be poisoned");
        if self.range.is_empty() || chunk_number > cache.chunk_number {
            return Ok(());
        }

        let (block_info, size, Ok(ref file) | Err(&ref file)) =
            cache.chunk_data(chunk_number, &mut self.buffer)?;

        let start = block_info.partition_point(|info| info.slot < self.range.start);
        let stop = block_info.partition_point(|info| info.slot < self.range.end);

        let read_start = block_info.get(start).map_or(size, |info| info.offset);
        let read_stop = block_info.get(stop).map_or(size, |info| info.offset);
        let read_size = (read_stop - read_start) as usize;
        self.buffer.reserve(read_size);
        read_buf(file, &mut self.buffer, read_start as u64, read_size)?;

        block_info[start..stop]
            .iter()
            .enumerate()
            .try_for_each(|(i, &info)| {
                let next = block_info
                    .get(start + i + 1)
                    .map_or(size, |info| info.offset);
                let block_size = (next - info.offset) as usize;

                if crc32fast::hash(&self.buffer[..block_size]) != info.crc {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("crc mismatch for slot {}", info.slot),
                    ));
                }

                buffer.push(Lazy::from(self.buffer.split_to(block_size).freeze()));
                Ok(())
            })?;

        self.range.start = ((chunk_number + 1) as slot::Number) * CHUNK_SIZE;
        Ok(())
    }
}
