//! Flat binary encoding and decoding for Plutus Core programs.

use std::{
    convert::Infallible,
    ffi::c_ulong,
    num::NonZeroU8,
    ops::{Deref, DerefMut},
};

use rug::Integer;
use tinycbor::CborLen;

use crate::{
    ConstantIndex, Data, DeBruijn, Instruction, Program, TermIndex, Version, builtin::Builtin, constant::{self, Array, Constant, List}
};

/// Trait for encoding into a [`Buffer`].
///
/// This is a failable oepration since not all constants can currently be encoded.
pub trait Encode {
    fn encode(&self, buffer: &mut Buffer) -> Option<()>;
}

/// Trait for decoding an object from a [`Reader`].
pub trait Decode<'a>: Sized {
    fn decode(reader: &mut Reader<'a>) -> Option<Self>;
}

/// A [`Buffer`] that is byte aligned.
///
/// Obtained by calling `Buffer::with_pad`.
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

/// A buffer of flat content.
#[derive(Debug)]
pub struct Buffer {
    buf: Vec<u8>,
    partial: u64,
    remaining: NonZeroU8,
}

impl Buffer {
    fn with_pad(&mut self, f: impl FnOnce(CleanBuffer<'_>)) {
        let shift_amount = (u8::from(self.remaining) + 7) % 8 + 1;
        self.partial <<= shift_amount;
        self.partial |= 1;
        let skip: usize = (u8::from(self.remaining) + 7) as usize / 8 - 1;
        self.buf.extend(&self.partial.to_be_bytes()[skip..]);
        self.partial = 0;
        self.remaining = NonZeroU8::new(64).unwrap();
        f(CleanBuffer { buf: self })
    }

    /// The contents provided are in the lower part of the `contents`. The msb is the
    /// first "element", and the least significant bit is the last.
    ///
    /// Panics at compile time if COUNT > 64.
    fn write_bits<const COUNT: u8>(&mut self, contents: u64) {
        const {
            assert!(
                COUNT <= 64,
                "write_bits: COUNT must be less than or equal to 64."
            );
        }
        let remaining: u8 = self.remaining.into();
        if COUNT >= remaining {
            self.partial <<= remaining;
            self.partial |= contents >> (COUNT - remaining);
            self.buf.extend(&self.partial.to_be_bytes());
            self.partial = contents & ((1 << (COUNT - remaining)) - 1);
            self.remaining = NonZeroU8::new(u64::BITS as u8 + remaining - COUNT)
                .expect("COUNT should be less than 64.");
        } else {
            self.partial <<= COUNT;
            self.partial |= contents;
            self.remaining =
                NonZeroU8::new(remaining - COUNT).expect("COUNT is less than remaining");
        }
    }

    fn encode_iter<'a, I: Encode + 'a, T: IntoIterator<Item = &'a I>>(&mut self, iter: T) {
        for item in iter {
            self.write_bits::<1>(1);
            item.encode(self);
        }
        self.write_bits::<1>(0);
    }

    pub fn finish(mut self) -> Vec<u8> {
        self.with_pad(|_| {});
        self.buf
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            buf: Default::default(),
            partial: Default::default(),
            remaining: NonZeroU8::new(64).unwrap(),
        }
    }
}

/// LEB128 encoding of the words. The provided words are in little endian order.
///
/// A non-exhaustive list of cases this needs to handle.
/// - Reads may not align on word boundaries.
/// - When reading from a word that is not the last word, we may still have a case where we are
///   reading the last byte of the whole word sequence.
/// - When reading from the last word, we do not read the full bit width of the word, but only up
///   to the highest set bit.
///
/// Schematic (we use 8 bits per word for simplicity, real words are 32 or 64 bits):
/// ```txt
///   00000001   22111111   33322222       3333
/// _____________________________________________
/// | ........ | ........ | ........ | 000000.. |
/// ---------------------------------------------
/// ```
///
/// When reading the block index `1`, we need to read some bits from word `0` and some
/// from word `1`. When reading block index `3`, we are not reading from the last word, yet this is
/// the last block we need to read. The last word has only two significant bits, so we only read
/// those two bits.
fn leb128<const DOUBLE: bool, const SUB1: bool>(data: &[c_ulong], buffer: &mut Buffer) {
    let mut count = 0;
    let mut word = (DOUBLE & SUB1) as c_ulong;
    let mut to_read = DOUBLE as u32; // If we want to double, read 1 `0` bit as lsb, then words.
    let mut byte;
    let mut sub_1 = SUB1;
    loop {
        byte = word & 0x7F;
        word >>= 7;

        if to_read <= 7 {
            let Some(mut new_word) = data.get(count).copied() else {
                break;
            };
            if sub_1 {
                let (subbed, overflowed) = new_word.overflowing_sub(1);
                sub_1 = overflowed;
                new_word = subbed;
            }
            word = new_word;
            count += 1;
            let shift = 7 - to_read;
            byte |= (word & ((1 << shift) - 1)) << to_read;
            word >>= shift;
            if count == data.len() {
                to_read = c_ulong::BITS - word.leading_zeros();
                if to_read == 0 {
                    break;
                }
            } else {
                to_read = c_ulong::BITS - shift;
            };
        } else {
            to_read -= 7;
        }
        let orred = byte | 0x80;
        buffer.write_bits::<8>(orred);
    }
    buffer.write_bits::<8>(byte);
}

impl Encode for [u8] {
    fn encode(&self, buffer: &mut Buffer) -> Option<()> {
        buffer.with_pad(|mut buf| {
            self.chunks(255).for_each(|chunk| {
                let len = chunk.len() as u8;
                buf.write_bytes(&[len]);
                buf.write_bytes(chunk);
            });
            buf.write_bytes(&[0]);
        });
        Some(())
    }
}

impl Encode for str {
    fn encode(&self, buffer: &mut Buffer) -> Option<()> {
        self.as_bytes().encode(buffer)
    }
}

impl Encode for Program<DeBruijn> {
    fn encode(&self, buffer: &mut Buffer) -> Option<()> {
        self.version.major.encode(buffer)?;
        self.version.minor.encode(buffer)?;
        self.version.patch.encode(buffer)?;
        // Ok(x): element is part of a list.
        // Err(x): element is not part of a list.
        // (_, true): variable is bound.
        let mut list_stack: Vec<Result<u16, (u16, bool)>> = vec![Err((1, false))];
        let mut var_count: u32 = 0;
        for instruction in &self.program {
            let top = match list_stack.last_mut().expect("stack is not empty") {
                Ok(x) => {
                    buffer.write_bits::<1>(1);
                    x
                }
                Err((x, _)) => x,
            };

            match instruction {
                Instruction::Variable(DeBruijn(var)) => {
                    buffer.write_bits::<4>(0);
                    let var = var_count.checked_sub(*var).expect("variable in scope");
                    var.encode(buffer)?;
                    decrement(&mut list_stack, buffer, &mut var_count);
                }
                Instruction::Delay => {
                    buffer.write_bits::<4>(1);
                }
                Instruction::Lambda(_) => {
                    buffer.write_bits::<4>(0b10);
                    *top -= 1;
                    list_stack.push(Err((1, true)));
                    var_count += 1;
                }
                Instruction::Application(_) => {
                    buffer.write_bits::<4>(0b11);
                    match list_stack.last_mut().expect("stack is not empty") {
                        Err((x, _)) => *x += 1,
                        _ => list_stack.push(Err((1, false))),
                    }
                }
                Instruction::Constant(constant_index) => {
                    buffer.write_bits::<4>(0b100);
                    let constant = &self.constants[constant_index.0 as usize];
                    constant.type_of().encode(buffer)?;
                    constant.encode(buffer)?;
                    decrement(&mut list_stack, buffer, &mut var_count);
                }
                Instruction::Force => {
                    buffer.write_bits::<4>(0b101);
                }
                Instruction::Error => {
                    buffer.write_bits::<4>(0b110);
                    decrement(&mut list_stack, buffer, &mut var_count);
                }
                Instruction::Builtin(builtin) => {
                    buffer.write_bits::<4>(0b111);
                    buffer.write_bits::<7>(*builtin as u64);
                    decrement(&mut list_stack, buffer, &mut var_count);
                }
                Instruction::Construct {
                    discriminant,
                    length,
                    large_discriminant,
                } => {
                    buffer.write_bits::<4>(0b1000);
                    if *large_discriminant {
                        let Constant::Integer(discriminant) =
                            &self.constants[*discriminant as usize]
                        else {
                            panic!("large_discriminant should point to an Integer constant");
                        };
                        discriminant
                            .to_u64()
                            .expect("discriminant should fit in u64")
                    } else {
                        *discriminant as u64
                    }
                    .encode(buffer)?;
                    if *length == 0 {
                        buffer.write_bits::<1>(0);
                        decrement(&mut list_stack, buffer, &mut var_count);
                    } else {
                        *top -= 1;
                        list_stack.push(Ok(*length));
                    }
                }
                Instruction::Case { count } => {
                    buffer.write_bits::<4>(0b1001);
                    *top -= 1;
                    list_stack.push(Ok(*count));
                    list_stack.push(Err((1, false)));
                }
            }
        }
        return Some(());

        fn decrement(
            stack: &mut Vec<Result<u16, (u16, bool)>>,
            buffer: &mut Buffer,
            var_count: &mut u32,
        ) {
            match stack.last_mut().expect("stack is not empty") {
                Ok(x) | Err((x, _)) => {
                    *x -= 1;
                }
            }

            while let Some(top) = stack.last_mut() {
                match top {
                    Ok(0) => {
                        buffer.write_bits::<1>(0);
                        stack.pop();
                    }
                    Err((0, bound_var)) => {
                        *var_count -= u32::from(*bound_var);
                        stack.pop();
                    }
                    _ => break,
                }
            }
        }
    }
}

impl Encode for Constant {
    fn encode(&self, buffer: &mut Buffer) -> Option<()> {
        match self {
            Constant::Integer(integer) => {
                integer.encode(buffer)?;
            }
            Constant::Bytes(bytes) => {
                bytes.encode(buffer)?;
            }
            Constant::String(str) => {
                str.encode(buffer)?;
            }
            Constant::Unit => {}
            Constant::Boolean(b) => {
                buffer.write_bits::<1>(u64::from(*b));
            }
            Constant::List(list) => {
                buffer.encode_iter(list.iter());
            }
            Constant::Array(array) => {
                buffer.encode_iter(array.iter());
            }
            Constant::Pair(pair) => {
                pair.0.encode(buffer)?;
                pair.1.encode(buffer)?;
            }
            Constant::Data(data) => {
                data.encode(buffer)?;
            }
            _ => {
                return None;
            }
        }
        Some(())
    }
}

impl Encode for constant::Type {
    fn encode(&self, buffer: &mut Buffer) -> Option<()> {
        let mut ty_stack = vec![self];
        while let Some(ty) = ty_stack.pop() {
            buffer.write_bits::<1>(1);
            match ty {
                constant::Type::Integer => {
                    buffer.write_bits::<4>(0);
                }
                constant::Type::Bytes => {
                    buffer.write_bits::<4>(1);
                }
                constant::Type::String => {
                    buffer.write_bits::<4>(2);
                }
                constant::Type::Unit => {
                    buffer.write_bits::<4>(3);
                }
                constant::Type::Boolean => {
                    buffer.write_bits::<4>(4);
                }
                constant::Type::List(elem_ty) => {
                    buffer.write_bits::<4>(7);
                    buffer.write_bits::<1>(1);
                    buffer.write_bits::<4>(5);
                    ty_stack.push(elem_ty);
                }
                constant::Type::Pair(pair_tys) => {
                    buffer.write_bits::<4>(7);
                    buffer.write_bits::<1>(1);
                    buffer.write_bits::<4>(7);
                    buffer.write_bits::<1>(1);
                    buffer.write_bits::<4>(6);
                    ty_stack.push(&pair_tys.1);
                    ty_stack.push(&pair_tys.0);
                }
                constant::Type::Data => {
                    buffer.write_bits::<4>(8);
                }
                constant::Type::Array(elem_ty) => {
                    buffer.write_bits::<4>(7);
                    buffer.write_bits::<1>(1);
                    buffer.write_bits::<4>(12);
                    ty_stack.push(elem_ty);
                }
                _ => {
                    return None;
                }
            }
        }
        buffer.write_bits::<1>(0);
        Some(())
    }
}

impl Encode for u64 {
    fn encode(&self, buffer: &mut Buffer) -> Option<()> {
        let mut x = *self;
        loop {
            let byte = (x & 0x7F) as u8;
            x >>= 7;
            let not_last_byte = (x != 0) as u8;
            buffer.write_bits::<8>(u64::from(byte | (not_last_byte << 7)));
            if x == 0 {
                break;
            }
        }
        Some(())
    }
}

impl Encode for u32 {
    fn encode(&self, buffer: &mut Buffer) -> Option<()> {
        (*self as u64).encode(buffer)
    }
}

impl Encode for Data {
    fn encode(&self, buffer: &mut Buffer) -> Option<()> {
        struct DataWriter<'a> {
            writer: CleanBuffer<'a>,
            len: usize,
            written: usize,
        }

        impl embedded_io::ErrorType for DataWriter<'_> {
            type Error = Infallible;
        }

        impl embedded_io::Write for DataWriter<'_> {
            fn write(&mut self, mut buf: &[u8]) -> Result<usize, Self::Error> {
                let len = buf.len();
                let written_in_slot = self.written % 255;
                if written_in_slot != 0 {
                    let to_write_in_slot = 255 - written_in_slot;
                    let write_now = to_write_in_slot.min(buf.len());
                    self.writer.write_bytes(&buf[..write_now]);
                    self.written += write_now;
                    buf = &buf[write_now..];
                }
                buf.chunks(255).for_each(|chunk| {
                    let len = (self.len - self.written).min(255);
                    debug_assert!(chunk.len() <= len, "Data length exceeded during encoding");

                    self.writer.write_bytes(&[len as u8]);
                    self.writer.write_bytes(chunk);
                    self.written += chunk.len();
                });

                Ok(len)
            }

            fn flush(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }
        }

        buffer.with_pad(|writer| {
            let mut encoder = tinycbor::Encoder(DataWriter {
                writer,
                len: self.cbor_len(),
                written: 0,
            });
            tinycbor::Encode::encode(self, &mut encoder);
            encoder.0.writer.write_bytes(&[0]);
        });
        Some(())
    }
}

impl Encode for Integer {
    fn encode(&self, buffer: &mut Buffer) -> Option<()> {
        if self.is_negative() {
            leb128::<true, true>(self.as_limbs(), buffer)
        } else {
            leb128::<true, false>(self.as_limbs(), buffer)
        }
        Some(())
    }
}

/// Reader of `flat` content.
pub struct Reader<'a> {
    buf: &'a [u8],
    /// bit position.
    position: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, position: 0 }
    }

    pub fn read_bits<const COUNT: usize>(&mut self) -> Option<u8> {
        const {
            if COUNT > 8 || COUNT == 0 {
                panic!("COUNT must be less than or equal to 8, and greater than 0.");
            }
        }

        let mask = ((1u16 << COUNT) - 1) as u8;
        let offset = COUNT - 1;
        let byte_index = self.position / 8;
        let bit_index = 7 - (self.position % 8);
        self.position += COUNT;
        self.buf.get(byte_index).and_then(|byte| {
            if bit_index >= offset {
                Some((byte >> (bit_index - offset)) & mask)
            } else {
                let mut value = (byte & ((1 << (bit_index + 1)) - 1)) << (offset - bit_index);
                let next_byte = self.buf.get(byte_index + 1)?;
                value |= next_byte >> (8 - (offset - bit_index));
                Some(value & mask)
            }
        })
    }

    pub fn read_bytes_padded<'b>(&'b mut self) -> Option<impl use<'b> + Iterator<Item = u8>> {
        struct BytesIter<'a> {
            bytes: &'a [u8],
            position: &'a mut usize,
        }

        impl Iterator for BytesIter<'_> {
            type Item = u8;

            fn next(&mut self) -> Option<Self::Item> {
                *self.position += 8;
                self.bytes.get(*self.position / 8 - 1).copied()
            }
        }

        let pad_len = 8 - (self.position % 8);
        let byte_index = self.position / 8;
        let pad = self.buf.get(byte_index)?;
        let mask = ((1u16 << pad_len) - 1) as u8;
        if (pad & mask) != 1 {
            return None;
        }
        self.position += pad_len;

        Some(BytesIter {
            bytes: self.buf,
            position: &mut self.position,
        })
    }
}

impl Decode<'_> for Vec<u8> {
    fn decode(reader: &mut Reader<'_>) -> Option<Self> {
        let mut result = Vec::new();
        let mut bytes_iter = reader.read_bytes_padded()?;
        loop {
            let len = bytes_iter.next()?;
            if len == 0 {
                break;
            }
            result.reserve(len as usize);
            for _ in 0..len {
                let byte = bytes_iter.next()?;
                result.push(byte);
            }
        }
        Some(result)
    }
}

impl Decode<'_> for String {
    fn decode(reader: &mut Reader<'_>) -> Option<Self> {
        let bytes = Vec::<u8>::decode(reader)?;
        String::from_utf8(bytes).ok()
    }
}

impl Decode<'_> for Program<DeBruijn> {
    fn decode(reader: &mut Reader<'_>) -> Option<Self> {
        let major = u64::decode(reader)?;
        let minor = u64::decode(reader)?;
        let patch = u64::decode(reader)?;

        enum Frame {
            /// Frame for tracking `case` and `construct` instructions which have a size.
            Sized {
                /// The index of the `case` or `construct` instruction in the program.
                index: u32,
                /// The number of elements it contains so far. (branches for `case`, fields for
                /// `construct`).
                length: u16,
            },
            /// Frame for tracking an application instruction.
            Application {
                /// The index of the `application` instruction in the program.
                /// 
                /// Used to write back where the second argument starts.
                index: u32,
            },
            /// Frame for tracking a lambda. After the lambda is read, we need to pop a variable
            /// off the stack.
            Variable,
            /// Frame for tracking other instructions which do not need special handling.
            Other,
            
        }
        
        // Ok(<frame>): element is part of a list of terms of unknown length. Used by `case` and
        // `construct`.
        //
        // Err((remaining, variable_bound?)):
        //  - `remaining`: number of terms remaining to be read in the current level.
        //  - `variable_bound`: whether a variable was bound in this level.
        let mut stack: Vec<Frame> = vec![Frame::Other];
        let mut instructions = Vec::new();
        let mut constants = Vec::new();
        let mut variable_count: u32 = 0;

        while !stack.is_empty() {
            match reader.read_bits::<4>()? {
                0 => {
                    let var = u32::decode(reader)?;
                    instructions.push(Instruction::Variable(DeBruijn(
                        variable_count.checked_sub(var)?,
                    )));
                    decrement(&mut stack, reader, &mut instructions, &mut variable_count)?;
                }
                1 => {
                    instructions.push(Instruction::Delay);
                }
                2 => {
                    instructions.push(Instruction::Lambda(DeBruijn(variable_count)));
                    variable_count += 1;
                    stack.push(Frame::Variable);
                }
                3 => {
                    let index = instructions.len() as u32;
                    instructions.push(Instruction::Application(TermIndex(0)));
                    stack.push(Frame::Application {
                        index,
                    });
                }
                4 => {
                    let constant = Constant::decode(reader)?;
                    let index = constants.len();
                    constants.push(constant);
                    instructions.push(Instruction::Constant(ConstantIndex(index as u32)));
                    decrement(&mut stack, reader, &mut instructions, &mut variable_count)?;
                }
                5 => {
                    instructions.push(Instruction::Force);
                }
                6 => {
                    instructions.push(Instruction::Error);
                    decrement(&mut stack, reader, &mut instructions, &mut variable_count)?;
                }
                7 => {
                    let builtin = reader.read_bits::<7>()?;
                    instructions.push(Instruction::Builtin(Builtin::from_repr(builtin)?));
                    decrement(&mut stack, reader, &mut instructions, &mut variable_count)?;
                }
                8 => {
                    let discriminant = u64::decode(reader)?;
                    let index = instructions.len() as u32;
                    if discriminant > u32::MAX as u64 {
                        instructions.push(Instruction::Construct {
                            discriminant: constants.len() as u32,
                            large_discriminant: true,
                            length: 0,
                        });
                        let discriminant_constant = Constant::Integer(Integer::from(discriminant));
                        constants.push(discriminant_constant);
                    } else {
                        instructions.push(Instruction::Construct {
                            discriminant: discriminant as u32,
                            large_discriminant: false,
                            length: 0,
                        });
                    }

                    stack.push(Frame::Sized { index, length: 0 });
                    decrement(&mut stack, reader, &mut instructions, &mut variable_count)?;
                }
                9 => {
                    let index = instructions.len() as u32;
                    instructions.push(Instruction::Case { count: 0 });

                    stack.push(Frame::Sized { index, length: 0 });
                    stack.push(Frame::Other);
                }
                _ => return None,
            }
        }

        if reader.read_bytes_padded()?.next().is_some() {
            return None;
        }

        return Some(Program {
            version: Version {
                major,
                minor,
                patch,
            },
            program: instructions,
            constants,
        });

        fn decrement(
            stack: &mut Vec<Frame>,
            reader: &mut Reader<'_>,
            program: &mut [Instruction<DeBruijn>],
            variable_count: &mut u32,
        ) -> Option<()> {
            while let Some(top) = stack.last_mut() {
                match top {
                    Frame::Sized { index, length } => {
                        let bit = reader.read_bits::<1>()?;
                        if bit == 1 {
                            *length += 1;
                            break;
                        }
                        
                        let (Instruction::Construct { length: count, .. }
                        | Instruction::Case { count }) = &mut program[*index as usize]
                        else {
                            panic!("Instruction at index should be Construct or Case")
                        };
                        *count = *length;
                        stack.pop();
                    },
                    Frame::Application { index } => {
                        let next = program.len() as u32;
                        let Instruction::Application(i) = &mut program[*index as usize] else {
                            panic!("Instruction at index should be Application");
                        };
                        i.0 = next;
                        *top = Frame::Other;
                        break;
                    },
                    Frame::Variable => {
                        *variable_count -= 1;
                        stack.pop();
                    },
                    Frame::Other => {
                        stack.pop();
                    },
                }
            }
            Some(())
        }
    }
}

impl Decode<'_> for Constant {
    fn decode(reader: &mut Reader<'_>) -> Option<Self> {
        fn decode_with_type(ty: &constant::Type, reader: &mut Reader<'_>) -> Option<Constant> {
            Some(match ty {
                constant::Type::Integer => {
                    let integer = Integer::decode(reader)?;
                    Constant::Integer(integer)
                }
                constant::Type::Bytes => {
                    let bytes = Vec::<u8>::decode(reader)?;
                    Constant::Bytes(bytes)
                }
                constant::Type::String => {
                    let string = String::decode(reader)?;
                    Constant::String(string)
                }
                constant::Type::Unit => Constant::Unit,
                constant::Type::Boolean => {
                    let b = reader.read_bits::<1>()? != 0;
                    Constant::Boolean(b)
                }
                constant::Type::List(element_ty) => {
                    let mut elements = Vec::new();
                    while reader.read_bits::<1>()? == 1 {
                        let element = decode_with_type(element_ty, reader)?;
                        elements.push(element);
                    }
                    Constant::List(List::from_vec_ty(elements, *element_ty.clone()))
                }
                constant::Type::Pair(pair_tys) => {
                    let first = decode_with_type(&pair_tys.0, reader)?;
                    let second = decode_with_type(&pair_tys.1, reader)?;
                    Constant::Pair(Box::new((first, second)))
                }
                constant::Type::Data => {
                    let data = Data::decode(reader)?;
                    Constant::Data(data)
                }
                constant::Type::BLSG1Element
                | constant::Type::BLSG2Element
                | constant::Type::MillerLoopResult => return None,
                constant::Type::Array(element_ty) => {
                    let mut elements = Vec::new();
                    while reader.read_bits::<1>()? == 1 {
                        let element = decode_with_type(element_ty, reader)?;
                        elements.push(element);
                    }
                    Constant::Array(Array::from_boxed_ty(
                        elements.into_boxed_slice(),
                        *element_ty.clone(),
                    ))
                }
            })
        }

        let ty = constant::Type::decode(reader)?;
        decode_with_type(&ty, reader)
    }
}

impl Decode<'_> for constant::Type {
    fn decode(reader: &mut Reader<'_>) -> Option<Self> {
        fn inner_decode(reader: &mut Reader<'_>) -> Option<constant::Type> {
            let 1 = reader.read_bits::<1>()? else {
                return None;
            };
            let ty = match reader.read_bits::<4>()? {
                0 => constant::Type::Integer,
                1 => constant::Type::Bytes,
                2 => constant::Type::String,
                3 => constant::Type::Unit,
                4 => constant::Type::Boolean,
                7 if reader.read_bits::<1>()? == 1 => match reader.read_bits::<4>()? {
                    5 => {
                        let elem_ty = Box::new(inner_decode(reader)?);
                        constant::Type::List(elem_ty)
                    }
                    7 if reader.read_bits::<1>()? == 1 && reader.read_bits::<4>()? == 6 => {
                        let first_ty = inner_decode(reader)?;
                        let second_ty = inner_decode(reader)?;
                        constant::Type::Pair(Box::new((first_ty, second_ty)))
                    }
                    12 => {
                        let elem_ty = Box::new(inner_decode(reader)?);
                        constant::Type::Array(elem_ty)
                    }
                    _ => return None,
                },
                8 => constant::Type::Data,
                9 => constant::Type::BLSG1Element,
                10 => constant::Type::BLSG2Element,
                11 => constant::Type::MillerLoopResult,
                _ => return None,
            };
            Some(ty)
        }
        let ty = inner_decode(reader)?;
        let 0 = reader.read_bits::<1>()? else {
            return None;
        };
        Some(ty)
    }
}

impl Decode<'_> for u64 {
    fn decode(reader: &mut Reader<'_>) -> Option<Self> {
        let mut result = 0u64;
        let mut shift = 0;
        loop {
            if shift >= 64 {
                return None;
            }
            let byte = reader.read_bits::<8>()?;
            result |= u64::from(byte & 0x7F) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        Some(result)
    }
}

impl Decode<'_> for u32 {
    fn decode(reader: &mut Reader<'_>) -> Option<Self> {
        let value = u64::decode(reader)?;
        if value > u32::MAX as u64 {
            return None;
        }
        Some(value as u32)
    }
}

impl Decode<'_> for Data {
    fn decode(reader: &mut Reader<'_>) -> Option<Self> {
        let mut decoder = tinycbor::Decoder(&Vec::<u8>::decode(reader)?);
        tinycbor::Decode::decode(&mut decoder).ok()
    }
}

impl<'a> Decode<'a> for Integer {
    fn decode(reader: &mut Reader<'a>) -> Option<Self> {
        let mut integer = Integer::new();
        let mut bytes = Vec::new();
        loop {
            let byte = reader.read_bits::<8>()?;
            bytes.push(byte & 0x7F);
            if byte & 0x80 == 0 {
                break;
            }
        }
        for byte in bytes.iter().rev() {
            integer <<= 7;
            integer |= byte;
        }

        if integer.is_even() {
            integer >>= 1;
        } else {
            integer += 1;
            integer >>= 1;
            integer = -integer;
        }
        Some(integer)
    }
}
