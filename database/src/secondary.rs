use crate::{BlockInfo, CHUNK_SIZE, read_buf};
use bytes::BytesMut;
use std::{fs::File, io};
use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned,
    big_endian::{U16, U32, U64},
};

#[repr(C)]
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
pub struct Entry {
    pub offset: U64,
    pub header_offset: U16,
    pub header_size: U16,
    pub crc: U32,
    pub hash: [u8; 32],
    pub slot: U64,
}

pub fn read(
    buffer: &mut BytesMut,
    secondary_file: &File,
    chunk_number: u32,
) -> io::Result<Box<[BlockInfo]>> {
    let secondary_size = secondary_file.metadata()?.len() as usize;
    read_buf(secondary_file, buffer, 0, secondary_size)?;
    let mut data: Box<[BlockInfo]> = <[Entry]>::ref_from_prefix(buffer)
        .expect("Entry is `Unaligned` and reading a slice can't error because of size")
        .0
        .iter()
        .map(|e| BlockInfo {
            slot: e.slot.get(),
            offset: e
                .offset
                .get()
                .try_into()
                .expect("chunk files should be smaller than 4 GiB"),
            crc: e.crc.get(),
        })
        .collect();
    if let Some(first) = data.first_mut() && first.slot == chunk_number as u64 {
        first.slot *= CHUNK_SIZE;
    }
    Ok(data)
}
