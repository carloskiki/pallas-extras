use std::io::{Error, ErrorKind, Read, Result, Write};

use rug::Integer;

trait Encode {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()>;
}

trait Decode: Sized {
    fn decode<R: Read>(reader: &mut R) -> Result<Self>;
}

impl Encode for Integer {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        todo!()
    }
}

impl Decode for Integer {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        todo!()
    }
}

impl Encode for [u64] {
    /// LEB128 encoding of the words. The provided words are in little endian order.
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut words = self.iter().copied();
        let mut count = 0;
        let mut word = 0;
        let mut to_read: u32 = 0;
        loop {
            let mut byte = (word & 0x7F) as u8;
            word >>= 7;

            if to_read <= 7 {
                count += 1;
                let shift = 7 - to_read;
                match words.next() {
                    Some(0) | None => return writer.write_all(&[byte]),
                    Some(w) => word = w,
                }
                byte <<= shift;
                byte |= word as u8 & ((1 << shift) - 1);
                word >>= shift;
                to_read = if count != self.len() {
                    u64::BITS - shift
                } else {
                    u64::BITS - word.leading_zeros()
                };
            }

            writer.write_all(&[byte | 0x80])?;
        }
    }
}

#[test]
pub fn test_thing() {
    let mut buf: Vec<u8> = Vec::new();
    Encode::encode([0x7Fu64].as_slice(), &mut buf).unwrap();
    dbg!(&buf);
    Encode::encode([0x80u64].as_slice(), &mut buf).unwrap();
    dbg!(&buf);
}

impl Encode for [u8] {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.chunks(255).try_for_each(|chunk| -> Result<()> {
            let len = chunk.len() as u8;
            writer.write_all(&[len])?;
            writer.write_all(chunk)?;
            Ok(())
        })?;
        writer.write_all(&[0])
    }
}

impl Decode for Vec<u8> {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let mut result = Vec::new();
        loop {
            let mut len_buf = [0u8; 1];
            reader.read_exact(&mut len_buf)?;
            let len = len_buf[0] as usize;
            if len == 0 {
                break;
            }
            let mut chunk = vec![0u8; len];
            reader.read_exact(&mut chunk)?;
            result.extend_from_slice(&chunk);
        }
        Ok(result)
    }
}

impl Encode for str {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.as_bytes().encode(writer)
    }
}

impl Decode for String {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let bytes = Vec::<u8>::decode(reader)?;
        String::from_utf8(bytes).map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }
}
