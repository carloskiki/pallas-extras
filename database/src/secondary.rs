use crate::{BlockInfo, chunk, read_buf};
use bytes::BytesMut;
use std::{fs::File, io, path::Path};
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
    directory: &impl AsRef<Path>,
    chunk_number: u32,
) -> io::Result<chunk::Data> {
    // TOCTOU: Happens under adverserial (or careless) behavior on the file system while the
    // database is running. These conditions are not caused by concurrent database instances because
    // of file lock.
    // - File is deleted/missing bytes: error
    // - File has more bytes: We read the previous size, and ignore the extra bytes.
    let file = File::open(directory.as_ref().join(format!("{chunk_number:05}.chunk")))?;
    let size: u32 = file
        .metadata()?
        .len()
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "chunk file is too large"))?;
    let secondary_file = File::open(
        directory
            .as_ref()
            .join(format!("{chunk_number:05}.secondary")),
    )?;
    let secondary_size = secondary_file.metadata()?.len() as usize;
    read_buf(&secondary_file, buffer, 0, secondary_size)?;
    let block_info = <[Entry]>::ref_from_prefix(&buffer)
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

    Ok(chunk::Data {
        block_info,
        file,
        size,
    })
}
