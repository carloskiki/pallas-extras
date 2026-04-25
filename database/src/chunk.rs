use bytes::{Bytes, BytesMut};
use ledger::{Block, slot};
use std::ops::Range;
use std::{fs::File, io};
use tinycbor::encoded::Lazy;

use crate::{BlockInfo, read_buf};

pub struct Data {
    pub block_info: Box<[BlockInfo]>,
    pub size: u32,
    pub file: File,
}

pub fn read(
    block_info: &[BlockInfo],
    size: u32,
    file: &File,
    buffer: &mut BytesMut,
    range: Range<slot::Number>,
) -> io::Result<Vec<Lazy<Bytes, Block<'static>>>> {
    let start = block_info.partition_point(|info| info.slot < range.start);
    let stop = block_info.partition_point(|info| info.slot < range.end);

    let read_start = block_info.get(start).map_or(size, |info| info.offset);
    let read_stop = block_info.get(stop).map_or(size, |info| info.offset);
    let read_size = read_stop - read_start;
    read_buf(file, buffer, read_start as u64, read_size as usize)?;

    block_info[start..stop]
        .iter()
        .enumerate()
        .map(|(i, &info)| {
            let next = block_info
                .get(start + i + 1)
                .map_or(size, |info| info.offset);
            let block_size = (next - info.offset) as usize;

            if crc32fast::hash(&buffer[..block_size]) != info.crc {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("crc mismatch for slot {}", info.slot),
                ));
            }

            Ok(Lazy::from(buffer.split_to(block_size).freeze()))
        })
        .collect()
}
