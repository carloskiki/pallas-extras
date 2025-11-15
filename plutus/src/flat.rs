// Non obvious thing from the spec:
// - The msb (most significant bit) is read first in the bit stream. For exmaple, If I have 9 bits
// to encode (0b011011011), they are encoded as such in memory: `01101101` `10000000`.

use std::{
    convert::Infallible,
    ffi::c_ulong,
    num::NonZeroU8,
    ops::{Deref, DerefMut},
};

use minicbor::CborLen;
use rug::{Complete, Integer};

use crate::{
    ConstantIndex, DeBruijn, Version,
    builtin::Builtin,
    constant::{self, Array, Constant, List},
    data::Data,
    program::{Instruction, Program},
};

pub trait Encode {
    fn encode(&self, buffer: &mut Buffer);
}

pub trait Decode<'a>: Sized {
    fn decode(reader: &mut Reader<'a>) -> Option<Self>;
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
        let skip: usize = u8::from(self.remaining) as usize / 8;
        self.buf.extend(&self.partial.to_be_bytes()[skip..]);
        f(CleanBuffer { buf: self })
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

    fn encode_iter<'a, I: Encode + 'a, T: IntoIterator<Item = &'a I>>(&mut self, iter: T) {
        for item in iter {
            self.write_bits(1, 1);
            item.encode(self);
        }
        self.write_bits(0, 1);
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

impl Encode for Program<DeBruijn> {
    // TODO: test
    fn encode(&self, buffer: &mut Buffer) {
        self.version.major.encode(buffer);
        self.version.minor.encode(buffer);
        self.version.patch.encode(buffer);
        // Ok(x): element is part of a list.
        // Err(x): element is not part of a list.
        let mut list_stack: Vec<Result<u16, u16>> = vec![Err(1)];
        for instruction in &self.program {
            let top = match list_stack.last_mut().expect("stack is not empty") {
                Ok(x) => {
                    buffer.write_bits(1, 1);
                    x
                }
                Err(x) => x,
            };

            match instruction {
                Instruction::Variable(DeBruijn(var)) => {
                    buffer.write_bits(0, 4);
                    (*var as u64).encode(buffer);
                    decrement(&mut list_stack, buffer);
                }
                Instruction::Delay => {
                    buffer.write_bits(1, 4);
                }
                Instruction::Lambda(_) => {
                    buffer.write_bits(0b10, 4);
                }
                Instruction::Application => {
                    buffer.write_bits(0b11, 4);
                    increment(&mut list_stack, 1);
                }
                Instruction::Constant(constant_index) => {
                    buffer.write_bits(0b100, 4);
                    let constant = &self.constants[constant_index.0 as usize];
                    constant.type_of().encode(buffer);
                    constant.encode(buffer);
                    decrement(&mut list_stack, buffer);
                }
                Instruction::Force => {
                    buffer.write_bits(0b101, 4);
                }
                Instruction::Error => {
                    buffer.write_bits(0b110, 4);
                    decrement(&mut list_stack, buffer);
                }
                Instruction::Builtin(builtin) => {
                    buffer.write_bits(0b111, 4);
                    buffer.write_bits(*builtin as u64, 7);
                    decrement(&mut list_stack, buffer);
                }
                Instruction::Construct {
                    determinant,
                    length,
                } => {
                    buffer.write_bits(0b1000, 4);
                    (*determinant as u64).encode(buffer);
                    *top -= 1;
                    list_stack.push(Ok(*length));
                }
                Instruction::Case { count } => {
                    buffer.write_bits(0b1001, 4);
                    *top -= 1;
                    list_stack.push(Ok(*count as u16));
                    list_stack.push(Err(1));
                }
            }
        }

        fn increment(stack: &mut Vec<Result<u16, u16>>, count: u16) {
            match stack.last_mut().expect("stack is not empty") {
                Ok(_) => stack.push(Err(count)),
                Err(x) => *x += count,
            }
        }

        fn decrement(stack: &mut Vec<Result<u16, u16>>, buffer: &mut Buffer) {
            match stack.last_mut().expect("stack is not empty") {
                Ok(x) | Err(x) => {
                    *x -= 1;
                }
            }

            loop {
                match stack.last().expect("stack is not empty") {
                    Ok(0) => {
                        stack.pop();
                        buffer.write_bits(0, 1);
                    }
                    Err(0) => {
                        stack.pop();
                    }
                    Ok(_) => {
                        buffer.write_bits(1, 1);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Encode for Constant {
    fn encode(&self, buffer: &mut Buffer) {
        match self {
            Constant::Integer(integer) => {
                integer.encode(buffer);
            }
            Constant::Bytes(bytes) => {
                bytes.encode(buffer);
            }
            Constant::String(str) => {
                str.encode(buffer);
            }
            Constant::Unit => {}
            Constant::Boolean(b) => {
                buffer.write_bits(u64::from(*b), 1);
            }
            Constant::List(list) => {
                buffer.encode_iter(list.iter());
            }
            Constant::Array(array) => {
                buffer.encode_iter(array.iter());
            }
            Constant::Pair(pair) => {
                pair.0.encode(buffer);
                pair.1.encode(buffer);
            }
            Constant::Data(data) => {
                data.encode(buffer);
            }
            Constant::BLSG1Element(g1_projective) => {
                g1_projective.to_compressed().encode(buffer);
            }
            Constant::BLSG2Element(g2_projective) => {
                g2_projective.to_compressed().encode(buffer);
            }
            Constant::MillerLoopResult(_) => {
                panic!("Cannot serialize MillerLoopResult constants");
            }
        }
    }
}

impl Encode for constant::Type {
    fn encode(&self, buffer: &mut Buffer) {
        let mut ty_stack = vec![self];
        while let Some(ty) = ty_stack.pop() {
            buffer.write_bits(1, 1);
            match ty {
                constant::Type::Integer => {
                    buffer.write_bits(0, 4);
                }
                constant::Type::Bytes => {
                    buffer.write_bits(1, 4);
                }
                constant::Type::String => {
                    buffer.write_bits(2, 4);
                }
                constant::Type::Unit => {
                    buffer.write_bits(3, 4);
                }
                constant::Type::Boolean => {
                    buffer.write_bits(4, 4);
                }
                constant::Type::List(elem_ty) => {
                    buffer.write_bits(7, 4);
                    buffer.write_bits(1, 1);
                    buffer.write_bits(5, 4);
                    ty_stack.push(elem_ty);
                }
                constant::Type::Pair(pair_tys) => {
                    buffer.write_bits(7, 4);
                    buffer.write_bits(1, 1);
                    buffer.write_bits(7, 4);
                    buffer.write_bits(1, 1);
                    buffer.write_bits(6, 4);
                    ty_stack.push(&pair_tys.1);
                    ty_stack.push(&pair_tys.0);
                }
                constant::Type::Data => {
                    buffer.write_bits(8, 4);
                }
                constant::Type::BLSG1Element => {
                    buffer.write_bits(9, 4);
                }
                constant::Type::BLSG2Element => {
                    buffer.write_bits(10, 4);
                }
                constant::Type::MillerLoopResult => {
                    buffer.write_bits(11, 4);
                }
                constant::Type::Array(elem_ty) => {
                    buffer.write_bits(7, 4);
                    buffer.write_bits(1, 1);
                    buffer.write_bits(12, 4);
                    ty_stack.push(elem_ty);
                }
            }
        }
        buffer.write_bits(0, 1);
    }
}

impl Encode for u64 {
    fn encode(&self, buffer: &mut Buffer) {
        let mut x = *self;
        while x.leading_zeros() != u64::BITS {
            let byte = (x & 0x7F) as u8;
            x >>= 7;
            let not_last_byte = (x != 0) as u8;
            buffer.write_bits(u64::from(byte | (not_last_byte << 7)), 8);
        }
    }
}

impl Encode for Data {
    fn encode(&self, buffer: &mut Buffer) {
        struct DataWriter<'a> {
            writer: CleanBuffer<'a>,
            len: usize,
            written: usize,
        }

        impl minicbor::encode::Write for DataWriter<'_> {
            type Error = Infallible;

            fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Self::Error> {
                let written_in_slot = self.written % 255;
                if written_in_slot != 0 {
                    let to_write_in_slot = 255 - written_in_slot;
                    let write_now = to_write_in_slot.min(buf.len());
                    self.writer.write_bytes(&buf[..write_now]);
                    self.written += write_now;
                    buf = &buf[write_now..];
                }
                buf.chunks(255).for_each(|chunk| {
                    let len = chunk.len() as u8;
                    assert!(
                        chunk.len() <= self.len - self.written,
                        "Data length exceeded during encoding"
                    );

                    self.writer.write_bytes(&[len]);
                    self.writer.write_bytes(chunk);
                    self.written += chunk.len();
                });

                Ok(())
            }
        }

        buffer.with_pad(|writer| {
            let writer = DataWriter {
                writer,
                len: self.cbor_len(&mut ()),
                written: 0,
            };
            minicbor::encode(self, writer).expect("Data should encode properly");
        })
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

pub struct Reader<'a> {
    buf: &'a [u8],
    // bit position.
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
        let mask = 1u8.wrapping_shl(pad_len as u32).wrapping_sub(1);
        if pad & mask != 1 {
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

        struct Frame {
            index: u32,
            length: u16,
        }

        let mut stack: Vec<(u16, Option<Frame>)> = vec![(1, None)];
        let mut instructions = Vec::new();
        let mut constants = Vec::new();

        while let Some(opcode) = reader.read_bits::<4>() {
            match opcode {
                0 => {
                    // TODO: should be > 0?
                    // TODO: impl decode on u32 directly?
                    let var = u64::decode(reader)?;
                    instructions.push(Instruction::Variable(DeBruijn(var as u32)));
                    decrement(&mut stack, reader, &mut instructions)?;
                }
                1 => {
                    instructions.push(Instruction::Delay);
                }
                2 => {
                    instructions.push(Instruction::Lambda(DeBruijn(0)));
                }
                3 => {
                    instructions.push(Instruction::Application);
                    increment(&mut stack, 1);
                }
                4 => {
                    let constant = Constant::decode(reader)?;
                    let index = constants.len();
                    constants.push(constant);
                    instructions.push(Instruction::Constant(ConstantIndex(index as u32)));
                    decrement(&mut stack, reader, &mut instructions)?;
                }
                5 => {
                    instructions.push(Instruction::Force);
                }
                6 => {
                    instructions.push(Instruction::Error);
                }
                7 => {
                    let builtin = reader.read_bits::<7>()?;
                    instructions.push(Instruction::Builtin(Builtin::from_repr(builtin)?));
                    decrement(&mut stack, reader, &mut instructions)?;
                }
                8 => {
                    let determinant = u64::decode(reader)? as u32;
                    let index = instructions.len() as u32;
                    instructions.push(Instruction::Construct {
                        determinant,
                        length: 0,
                    });
                    stack.push((1, Some(Frame { index, length: 0 })));
                }
                9 => {
                    let index = instructions.len() as u32;
                    instructions.push(Instruction::Case { count: 0 });
                    stack.push((1, Some(Frame { index, length: 0 })));
                    stack.push((1, None));
                }
                _ => return None,
            }
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
            stack: &mut Vec<(u16, Option<Frame>)>,
            reader: &mut Reader<'_>,
            program: &mut [Instruction<DeBruijn>],
        ) -> Option<()> {
            let (x, frame) = stack.last_mut().expect("stack is not empty");
            *x -= 1;
            if let Some(Frame { length, .. }) = frame {
                *length += 1;
            }

            while let Some(top) = stack.last_mut() {
                match top {
                    (0, Some(Frame { length, index })) => {
                        let 0 = reader.read_bits::<1>()? else {
                            return None;
                        };
                        let (Instruction::Construct { length: count, .. }
                        | Instruction::Case { count }) = &mut program[*index as usize]
                        else {
                            panic!("Instruction at index should be Construct or Case")
                        };
                        *count = *length;
                        stack.pop();
                    }
                    (0, None) => {
                        stack.pop();
                    }
                    (_, Some(_)) => {
                        let 1 = reader.read_bits::<1>()? else {
                            return None;
                        };
                        break;
                    }
                    (_, None) => break,
                }
            }
            Some(())
        }
        fn increment(stack: &mut Vec<(u16, Option<Frame>)>, count: u16) {
            match stack.last_mut().expect("stack is not empty") {
                (x, None) => *x += count,
                (_, Some(_)) => stack.push((count, None)),
            }
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
                    let elem_ty = Box::new(constant::Type::decode(reader)?);
                    constant::Type::List(elem_ty)
                }
                7 if reader.read_bits::<1>()? == 1 && reader.read_bits::<4>()? == 6 => {
                    let second_ty = constant::Type::decode(reader)?;
                    let first_ty = constant::Type::decode(reader)?;
                    constant::Type::Pair(Box::new((first_ty, second_ty)))
                }
                12 => {
                    let elem_ty = Box::new(constant::Type::decode(reader)?);
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

impl Decode<'_> for Data {
    fn decode(reader: &mut Reader<'_>) -> Option<Self> {
        let bytes = Vec::<u8>::decode(reader)?;
        minicbor::decode(&bytes).ok()
    }
}

impl<'a> Decode<'a> for Integer {
    fn decode(reader: &mut Reader<'a>) -> Option<Self> {
        let mut integer = Integer::new();
        loop {
            let byte = reader.read_bits::<8>()?;
            integer |= byte & 0x7F;
            if byte & 0x80 == 0 {
                break;
            }
            integer <<= 7;
        }

        Some(integer)
    }
}
