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
    ConstantIndex, Data, DeBruijn, Instruction, Program, TermIndex, Version,
    builtin::Builtin,
    constant::{self, Array, Constant, List},
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

    fn encode_iter<'a, I: ?Sized + Encode + 'a, T: IntoIterator<Item = &'a I>>(&mut self, iter: T) {
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
fn leb128<const DOUBLE: bool, const SUB1: bool>(data: &[u64], buffer: &mut Buffer) {
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

impl Encode for Program<'_, DeBruijn> {
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
                    encode_type(constant, buffer)?;
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
                } => {
                    buffer.write_bits::<4>(0b1000);
                    let Constant::Integer(discriminant) = &self.constants[discriminant.0 as usize]
                    else {
                        panic!("large_discriminant should point to an Integer constant");
                    };
                    discriminant
                        .to_u64()
                        .expect("discriminant should fit in u64")
                        .encode(buffer)?;
                    if *length == 0 {
                        buffer.write_bits::<1>(0);
                        decrement(&mut list_stack, buffer, &mut var_count);
                    } else {
                        *top -= 1;
                        list_stack.push(Ok(*length));
                    }
                }
                Instruction::Case { count, .. } => {
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

impl Encode for bool {
    fn encode(&self, buffer: &mut Buffer) -> Option<()> {
        buffer.write_bits::<1>(u64::from(*self));
        Some(())
    }
}

impl Encode for () {
    fn encode(&self, _buffer: &mut Buffer) -> Option<()> {
        Some(())
    }
}

impl Encode for (Data, Data) {
    fn encode(&self, buffer: &mut Buffer) -> Option<()> {
        self.0.encode(buffer)?;
        self.1.encode(buffer)
    }
}

impl Encode for Constant<'_> {
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
            Constant::Unit => {
                ().encode(buffer)?;
            }
            Constant::Boolean(b) => {
                b.encode(buffer)?;
            }
            Constant::List(list) | Constant::Array(Array(list)) => match list {
                List::Integer(integers) => buffer.encode_iter(integers.iter()),
                List::Data(datas) => buffer.encode_iter(datas.iter()),
                List::PairData(items) => buffer.encode_iter(items.iter()),
                List::Generic(constants) => match constants {
                    Ok(constants) => buffer.encode_iter(constants.iter()),
                    Err(_) => buffer.encode_iter::<(), _>(std::iter::empty()),
                },
                _ => return None,
            },
            Constant::Pair(first, second) => {
                first.encode(buffer)?;
                second.encode(buffer)?;
            }
            Constant::PairData((a, b)) => {
                a.encode(buffer)?;
                b.encode(buffer)?;
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

fn encode_type(ty: &Constant<'_>, buffer: &mut Buffer) -> Option<()> {
    let mut ty_stack = vec![ty];
    while let Some(ty) = ty_stack.pop() {
        buffer.write_bits::<1>(1);
        match ty {
            Constant::Integer(_) => {
                buffer.write_bits::<4>(0);
            }
            Constant::Bytes(_) => {
                buffer.write_bits::<4>(1);
            }
            Constant::String(_) => {
                buffer.write_bits::<4>(2);
            }
            Constant::Unit => {
                buffer.write_bits::<4>(3);
            }
            Constant::Boolean(_) => {
                buffer.write_bits::<4>(4);
            }
            Constant::List(list_ty) => {
                buffer.write_bits::<4>(7);
                buffer.write_bits::<1>(1);
                buffer.write_bits::<4>(5);
                ty_stack.push(list_ty.type_of());
            }
            Constant::Pair(first_ty, second_ty) => {
                buffer.write_bits::<4>(7);
                buffer.write_bits::<1>(1);
                buffer.write_bits::<4>(7);
                buffer.write_bits::<1>(1);
                buffer.write_bits::<4>(6);
                ty_stack.push(second_ty);
                ty_stack.push(first_ty);
            }
            Constant::PairData(_) => {
                buffer.write_bits::<4>(7);
                buffer.write_bits::<1>(1);
                buffer.write_bits::<4>(7);
                buffer.write_bits::<1>(1);
                buffer.write_bits::<4>(6);
                buffer.write_bits::<1>(1);
                buffer.write_bits::<4>(8);
                buffer.write_bits::<1>(1);
                buffer.write_bits::<4>(8);
            }
            Constant::Data(_) => {
                buffer.write_bits::<4>(8);
            }
            Constant::Array(Array(array_ty)) => {
                buffer.write_bits::<4>(7);
                buffer.write_bits::<1>(1);
                buffer.write_bits::<4>(12);
                ty_stack.push(array_ty.type_of());
            }
            _ => {
                return None;
            }
        }
    }
    buffer.write_bits::<1>(0);
    Some(())
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

pub fn decode_program<'a>(
    reader: &mut Reader<'_>,
    arena: &'a constant::Arena,
) -> Option<Program<'a, DeBruijn>> {
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
        /// Frame for tracking an `application` or the  instruction.
        Application {
            /// The index of the `application` instruction in the program.
            ///
            /// Used to write back where the second argument starts.
            index: u32,
        },
        /// Frame for tracking the scrutinee of a `case` instruction.
        Scrutinee {
            /// The index of the `case` instruction in the program.
            ///
            /// Used to write back when the first branch starts.
            index: u32,
        },
        /// Frame for tracking a lambda. After the lambda is read, we need to pop a variable
        /// off the stack.
        Variable,
        /// Frame for tracking other instructions which do not need special handling.
        Other,
    }

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
                stack.push(Frame::Application { index });
            }
            4 => {
                let index = ConstantIndex(constants.len() as u32);
                let constant = decode_constant(reader, arena)?;
                constants.push(constant);
                instructions.push(Instruction::Constant(index));
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
                let discriminant_value = u64::decode(reader)?;
                let index = instructions.len() as u32;
                let discriminant = ConstantIndex(constants.len() as u32);
                constants.push(Constant::Integer(
                    arena.integer(Integer::from(discriminant_value)),
                ));
                instructions.push(Instruction::Construct {
                    discriminant,
                    length: 0,
                });

                stack.push(Frame::Sized { index, length: 0 });
                decrement(&mut stack, reader, &mut instructions, &mut variable_count)?;
            }
            9 => {
                let index = instructions.len() as u32;
                instructions.push(Instruction::Case { count: 0, next: TermIndex(0) });

                stack.push(Frame::Sized { index, length: 0 });
                stack.push(Frame::Scrutinee { index });
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
        arena,
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
                    | Instruction::Case { count, .. }) = &mut program[*index as usize]
                    else {
                        panic!("Instruction at index should be Construct or Case")
                    };
                    *count = *length;
                    stack.pop();
                }
                Frame::Application { index } => {
                    let next = program.len() as u32;
                    let Instruction::Application(i) = &mut program[*index as usize] else {
                        panic!("Instruction at index should be Application");
                    };
                    i.0 = next;
                    *top = Frame::Other;
                    break;
                }
                Frame::Scrutinee { index } => {
                    let next = program.len() as u32;
                    let Instruction::Case { next: i, .. } = &mut program[*index as usize] else {
                        panic!("Instruction at index should be Case");
                    };
                    i.0 = next;
                    stack.pop();
                }
                Frame::Variable => {
                    *variable_count -= 1;
                    stack.pop();
                }
                Frame::Other => {
                    stack.pop();
                }
            }
        }
        Some(())
    }
}

fn decode_constant<'a>(
    reader: &mut Reader<'_>,
    arena: &'a constant::Arena,
) -> Option<Constant<'a>> {
    fn list_with_type<'a>(
        mut ty: List<'a>,
        reader: &mut Reader<'_>,
        arena: &'a constant::Arena,
    ) -> Option<List<'a>> {
        fn from_fn<'a, T: 'a>(
            reader: &mut Reader<'_>,
            arena: &'a constant::Arena,
            f: impl Fn(&mut Reader<'_>, &'a constant::Arena) -> Option<T>,
        ) -> Option<Vec<T>> {
            let mut elements = Vec::new();
            while reader.read_bits::<1>()? == 1 {
                elements.push(f(reader, arena)?);
            }
            Some(elements)
        }

        match &mut ty {
            List::Integer(i) => {
                *i = from_fn(reader, arena, |r, _| Integer::decode(r))
                    .map(|ints| arena.integers(ints))?;
            }
            List::PairData(p) => {
                *p = from_fn(reader, arena, |r, _| {
                    let first = Data::decode(r)?;
                    let second = Data::decode(r)?;
                    Some((first, second))
                })
                .map(|pairs| arena.pair_datas(pairs))?;
            }
            List::Data(d) => {
                *d = from_fn(reader, arena, |r, _| Data::decode(r))
                    .map(|datas| arena.datas(datas))?;
            }
            List::Generic(Err(list_ty)) => {
                ty = from_fn(reader, arena, |r, a| decode_with_type(**list_ty, r, a)).map(
                    |constants| {
                        if constants.is_empty() {
                            return List::Generic(Err(*list_ty));
                        }
                        // we know the content of `Constant` is not `Integer` or `Data`, so we are not
                        // leaking memory by not tracking `Integer` or `Data`.
                        List::Generic(Ok(mitsein::slice1::Slice1::try_from_slice(
                            arena.slice_fill(constants),
                        )
                        .expect("constants checked to be non-empty")))
                    },
                )?;
            }
            _ => return None,
        }
        Some(ty)
    }

    fn decode_with_type<'a>(
        mut ty: Constant<'a>,
        reader: &mut Reader<'_>,
        arena: &'a constant::Arena,
    ) -> Option<Constant<'a>> {
        match &mut ty {
            Constant::Integer(i) => {
                *i = arena.integer(Integer::decode(reader)?);
            }
            Constant::Bytes(b) => {
                *b = arena.slice_fill(Vec::<u8>::decode(reader)?);
            }
            Constant::String(s) => {
                *s = arena.string(&String::decode(reader)?);
            }
            Constant::Unit => {}
            Constant::Boolean(b) => {
                *b = reader.read_bits::<1>()? != 0;
            }
            Constant::List(element_ty) | Constant::Array(Array(element_ty)) => {
                *element_ty = list_with_type(*element_ty, reader, arena)?;
            }
            Constant::Pair(first, second) => {
                *first = arena.alloc(decode_with_type(**first, reader, arena)?);
                *second = arena.alloc(decode_with_type(**second, reader, arena)?);
            }
            Constant::Data(d) => {
                *d = arena.data(Data::decode(reader)?);
            }
            Constant::PairData(pair) => {
                *pair = arena.pair_data((Data::decode(reader)?, Data::decode(reader)?));
            }
            _ => return None,
        }
        Some(ty)
    }

    let ty = decode_type(reader, arena)?;
    decode_with_type(ty, reader, arena)
}

fn decode_type<'a>(reader: &mut Reader<'_>, arena: &'a constant::Arena) -> Option<Constant<'a>> {
    fn list_decode<'a>(reader: &mut Reader<'_>, arena: &'a constant::Arena) -> Option<List<'a>> {
        let save = Reader {
            buf: reader.buf,
            position: reader.position,
        };
        let 1 = reader.read_bits::<1>()? else {
            return None;
        };

        Some(match reader.read_bits::<4>()? {
            0 => List::INTEGER_TYPE,
            8 => List::DATA_TYPE,
            // Pair Data
            7 if reader.read_bits::<5>()? == 0b10111
                && reader.read_bits::<5>()? == 0b10110
                && reader.read_bits::<5>()? == 0b11000
                && reader.read_bits::<5>()? == 0b11000 =>
            {
                List::PAIRDATA_TYPE
            }
            _ => {
                *reader = save;
                List::Generic(Err(arena.alloc(inner_decode(reader, arena)?)))
            }
        })
    }

    fn inner_decode<'a>(
        reader: &mut Reader<'_>,
        arena: &'a constant::Arena,
    ) -> Option<Constant<'a>> {
        let 1 = reader.read_bits::<1>()? else {
            return None;
        };
        let ty = match reader.read_bits::<4>()? {
            0 => Constant::INTEGER_TYPE,
            1 => Constant::BYTES_TYPE,
            2 => Constant::STRING_TYPE,
            3 => Constant::UNIT_TYPE,
            4 => Constant::BOOLEAN_TYPE,
            7 if reader.read_bits::<1>()? == 1 => match reader.read_bits::<4>()? {
                5 => Constant::List(list_decode(reader, arena)?),
                7 if reader.read_bits::<1>()? == 1 && reader.read_bits::<4>()? == 6 => {
                    let first_ty = inner_decode(reader, arena)?;
                    let second_ty = inner_decode(reader, arena)?;
                    Constant::Pair(arena.alloc(first_ty), arena.alloc(second_ty))
                }
                12 => Constant::Array(Array(list_decode(reader, arena)?)),
                _ => return None,
            },
            8 => Constant::DATA_TYPE,
            _ => return None,
        };
        Some(ty)
    }
    let ty = inner_decode(reader, arena)?;
    let 0 = reader.read_bits::<1>()? else {
        return None;
    };
    Some(ty)
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
