use macro_rules_attribute::apply;
use rug::Integer;

use super::builtin;

#[apply(builtin)]
pub fn append(mut x: Vec<u8>, y: Vec<u8>) -> Vec<u8> {
    x.extend(y);
    x
}

#[apply(builtin)]
pub fn cons_v1(x: Integer, mut y: Vec<u8>) -> Vec<u8> {
    let byte = x.to_u8_wrapping();
    y.insert(0, byte);
    y
}

#[apply(builtin)]
pub fn cons_v2(x: Integer, mut y: Vec<u8>) -> Option<Vec<u8>> {
    let byte = x.to_u8()?;
    y.insert(0, byte);
    Some(y)
}

#[apply(builtin)]
pub fn slice(start: Integer, end: Integer, bytes: Vec<u8>) -> Vec<u8> {
    let (Some(start), Some(end)) = (start.to_usize(), end.to_usize()) else {
        return vec![];
    };

    bytes.get(start..end).unwrap_or(&[]).to_vec()
}

#[apply(builtin)]
pub fn length(bytes: Vec<u8>) -> Integer {
    Integer::from(bytes.len())
}

#[apply(builtin)]
pub fn index(index: Integer, bytes: Vec<u8>) -> Option<Integer> {
    let index = index.to_usize()?;
    let byte = *bytes.get(index)?;
    Some(Integer::from(byte))
}

#[apply(builtin)]
pub fn equals(x: Vec<u8>, y: Vec<u8>) -> bool {
    x == y
}

#[apply(builtin)]
pub fn less_than(x: Vec<u8>, y: Vec<u8>) -> bool {
    x < y
}

#[apply(builtin)]
pub fn less_than_or_equal(x: Vec<u8>, y: Vec<u8>) -> bool {
    x <= y
}

#[apply(builtin)]
pub fn to_integer(big_endian: bool, bytes: Vec<u8>) -> Integer {
    Integer::from_digits(
        &bytes,
        if big_endian {
            rug::integer::Order::Msf
        } else {
            rug::integer::Order::Lsf
        },
    )
}

#[apply(builtin)]
pub fn and(extend: bool, mut x: Vec<u8>, y: Vec<u8>) -> Vec<u8> {
    x.iter_mut().zip(y.iter()).for_each(|(a, b)| *a &= b);
    if extend && y.len() > x.len() {
        x.extend_from_slice(&y[x.len()..]);
    } else if !extend && x.len() > y.len() {
        x.truncate(y.len());
    }
    x
}

#[apply(builtin)]
pub fn or(extend: bool, mut x: Vec<u8>, y: Vec<u8>) -> Vec<u8> {
    x.iter_mut().zip(y.iter()).for_each(|(a, b)| *a |= b);
    if extend && y.len() > x.len() {
        x.extend_from_slice(&y[x.len()..]);
    } else if !extend && x.len() > y.len() {
        x.truncate(y.len());
    }
    x
}

#[apply(builtin)]
pub fn xor(extend: bool, mut x: Vec<u8>, y: Vec<u8>) -> Vec<u8> {
    x.iter_mut().zip(y.iter()).for_each(|(a, b)| *a ^= b);
    if extend && y.len() > x.len() {
        x.extend_from_slice(&y[x.len()..]);
    } else if !extend && x.len() > y.len() {
        x.truncate(y.len());
    }
    x
}

#[apply(builtin)]
pub fn complement(mut x: Vec<u8>) -> Vec<u8> {
    x.iter_mut().for_each(|b| *b = !*b);
    x
}

#[apply(builtin)]
pub fn shift(mut x: Vec<u8>, by: Integer) -> Vec<u8> {
    let by = match by.to_isize() {
        Some(n) => n,
        None => {
            x.fill(0);
            return x;
        },
    };
    if by == 0 {
        return x;
    }
    let byte_shift = by.unsigned_abs() / 8;
    let bit_shift = by.unsigned_abs() % 8;
    let len = x.len();

    if by > 0 {
        for i in byte_shift..len {
            x[i - byte_shift] |= x[i] << bit_shift;
            x[i - byte_shift] |= x.get(i + 1).unwrap_or(&0) >> (8 - bit_shift);
        }
        x[len - byte_shift..].fill(0);
    } else {
        for i in (0..len - byte_shift).rev() {
            x[i + byte_shift] |= x[i] >> bit_shift;
            x[i + byte_shift] |= x.get(i.wrapping_sub(1)).unwrap_or(&0) << (8 - bit_shift);
        }
        x[..byte_shift].fill(0);
    }
    x
}

#[apply(builtin)]
pub fn rotate(mut x: Vec<u8>, by: Integer) -> Vec<u8> {
    let len_bits = x.len() * 8;
    let by = match by.to_isize() {
        Some(n) => n % (len_bits as isize),
        None => return x,
    };
    if by == 0 {
        return x;
    }
    let byte_shift = by.unsigned_abs() / 8;
    let bit_shift = by.unsigned_abs() % 8;
    let len = x.len();

    if by > 0 {
        x.rotate_left(byte_shift);
        if bit_shift != 0 {
            for i in 0..len {
                let next = x[(i + 1) % len];
                x[i] = (x[i] << bit_shift) | (next >> (8 - bit_shift));
            }
        }
    } else {
        x.rotate_right(byte_shift);
        if bit_shift != 0 {
            for i in (0..len).rev() {
                let prev = x[(i + len - 1) % len];
                x[i] = (x[i] >> bit_shift) | (prev << (8 - bit_shift));
            }
        }
    }
    x
}

#[apply(builtin)]
pub fn count_set_bits(x: Vec<u8>) -> Integer {
    let count: usize = x.iter().map(|b| b.count_ones() as usize).sum();
    Integer::from(count)
}

#[apply(builtin)]
pub fn first_set_bit(x: Vec<u8>) -> Integer {
    let mut index = 0;
    for byte in x.iter().rev() {
        if byte.trailing_zeros() < 8 {
            index += byte.trailing_zeros() as usize;
            return Integer::from(index);
        }
        index += 8;
    }
    Integer::from(-1)
}

#[apply(builtin)]
pub fn read_bit(x: Vec<u8>, index: Integer) -> Option<bool> {
    let index = index.to_usize()?;
    let byte_index = index / 8;
    let bit_index = index % 8;
    let byte = *x.get(byte_index)?;
    Some((byte & (1 << bit_index)) != 0)
}

#[apply(builtin)]
pub fn write_bits(mut x: Vec<u8>, indices: Vec<Integer>, bit: bool) -> Option<Vec<u8>> {
    for index in indices {
        let index = index.to_usize()?;
        let byte_index = index / 8;
        let bit_index = index % 8;
        let byte = x.get_mut(byte_index)?;
        if bit {
            *byte |= 1 << bit_index;
        } else {
            *byte &= !(1 << bit_index);
        }
    }
    Some(x)
}

#[apply(builtin)]
pub fn replicate_byte(count: Integer, byte: Integer) -> Option<Vec<u8>> {
    let byte = byte.to_u8()?;
    let count = count.to_usize()?;
    if count > 8192 {
        return None;
    }
    Some(vec![byte; count])
}
