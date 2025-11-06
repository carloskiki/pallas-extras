// Non obvious thing from the spec:
// - The msb (most significant bit) is read first in the bit stream. For exmaple, If I have 9 bits
// to encode (0b011011011), they are encoded as such in memory: `01101101` `10000000`.

use std::{
    ffi::c_ulong,
    io::{Read, Result, Write},
    num::NonZeroU8, ops::{Deref, DerefMut},
};

use rug::{Complete, Integer};

trait Encode {
    fn encode(&self, buffer: &mut Buffer);
}

trait Decode: Sized {
    fn decode<R: Read>(reader: &mut R) -> Result<Self>;
}

struct CleanBuffer<'a> {
    buf: &'a mut Buffer,
}

impl<'a> Deref for CleanBuffer<'a> {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        self.buf
    }
}

impl<'a> DerefMut for CleanBuffer<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf
    }
}

impl<'a> CleanBuffer<'a> {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.buf.buf.extend(bytes)
    }
}

struct Buffer {
    buf: Vec<u8>,
    partial: u64,
    remaining: NonZeroU8,
}

impl Buffer {
    fn with_pad(&mut self, f: impl FnOnce(CleanBuffer<'_>)) {
        let shift_amount = (u8::from(self.remaining) + 7) % 8 + 1;
        self.partial <<= shift_amount;
        self.partial |= 1;
        let skip: usize = u8::from(self.remaining) as usize / 8;
        self.buf.extend(&self.partial.to_be_bytes()[skip..]);
        f(CleanBuffer {
            buf: self,
        })
    }
    
    /// The contents provided are in the lower part of the `contents`. The msb is the
    /// first "element", and the least significant bit is the last.
    ///
    /// Panics if count > 64.
    fn write_bits(&mut self, contents: u64, count: u8) {
        let remaining: u8 = self.remaining.into();
        if count >= remaining {
            self.partial <<= remaining;
            self.partial |= contents >> (count - remaining);
            self.buf.extend(&self.partial.to_be_bytes());
            self.partial = contents & ((1 << (count - remaining)) - 1);
            self.remaining = NonZeroU8::new(u64::BITS as u8 + remaining - count)
                .expect("count should be less than 64.");
        } else {
            self.partial <<= count;
            self.partial |= contents;
            self.remaining =
                NonZeroU8::new(remaining - count).expect("count is less than remaining");
        }
    }

}

/// LEB128 encoding of the words. The provided words are in little endian order.
// Wierd Cases:
// - The last word may not finish on a multiple of 7
// - The last word may not have enough bytes to fulfill even one read from the previous to
//   last word.
// - The slice may be empty.
fn leb128<const DOUBLE: bool>(data: &[c_ulong], buffer: &mut Buffer) {
    let mut count = 0;
    let mut word = 0;
    let mut to_read = DOUBLE as u32; // If we want to double, read 1 `0` bit as lsb, then words.
    while count != data.len() || to_read != 0 {
        let mut byte = word & 0x7F;
        word >>= 7;

        if to_read <= 7 {
            word = data.get(count).copied().unwrap_or(0);
            count += 1;
            let mut shift = 7 - to_read;
            let old_to_read = to_read;
            to_read = if count == data.len() {
                let remaining = c_ulong::BITS - word.leading_zeros();
                if remaining <= shift {
                    shift = remaining;
                    0
                } else {
                    remaining - shift
                }
            } else {
                c_ulong::BITS - shift
            };

            byte |= (word & ((1 << shift) - 1)) << old_to_read;
            word >>= shift;
        } else {
            to_read -= 7;
        }

        let not_last_byte = (count != data.len() || to_read != 0) as u64;
        buffer.write_bits(byte | (not_last_byte) << 7, 8);
    }
}

impl Encode for [u8] {
    fn encode(&self, buffer: &mut Buffer) {
        buffer.with_pad(|mut buf| {
            self.chunks(255).for_each(|chunk| {
                let len = chunk.len() as u8;
                buf.write_bytes(&[len]);
                buf.write_bytes(chunk);
            });
            buf.write_bytes(&[0]);
        });
    }
}

impl Encode for str {
    fn encode(&self, buffer: &mut Buffer) {
        self.as_bytes().encode(buffer)
    }
}

impl Encode for Integer {
    fn encode(&self, buffer: &mut Buffer) {
        if self.is_negative() {
            let x: Integer = -(self * 2usize).complete();
            leb128::<false>((x - 1usize).as_limbs(), buffer)
        } else {
            leb128::<true>(self.as_limbs(), buffer)
        }
    }
}


impl Decode for Integer {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let mut integer = Integer::new();
        #[allow(clippy::unbuffered_bytes)]
        for byte in reader.bytes() {
            let byte = byte?;
            integer |= byte & 0x7F;
            if byte & 0x80 == 0 {
                break;
            }
            integer <<= 7;
        }

        Ok(integer)
    }
}

