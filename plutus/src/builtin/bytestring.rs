use rug::Integer;

pub fn append(mut x: Vec<u8>, y: &[u8]) -> Vec<u8> {
    x.extend_from_slice(y);
    x
}

// FIXME: Check how we want to handle this.
// https://github.com/IntersectMBO/plutus/issues/7426
// pub fn cons_v1(x: Integer, mut y: Vec<u8>) -> Vec<u8> {
//     let byte = x.to_u8_wrapping();
//     y.insert(0, byte);
//     y
// }

pub fn cons_v2(x: u8, mut y: Vec<u8>) -> Vec<u8> {
    y.insert(0, x);
    y
}

pub fn slice(start: i64, len: i64, bytes: &[u8]) -> &[u8] {
    if len <= 0 {
        return &[];
    }

    let start = usize::try_from(start).unwrap_or(0);
    let len = usize::try_from(len).unwrap_or(usize::MAX);
    bytes
        .get(start..bytes.len().min(start.saturating_add(len)))
        .unwrap_or(&[])
}

pub fn length(bytes: &[u8]) -> Integer {
    Integer::from(bytes.len())
}

pub fn index(bytes: &[u8], index: usize) -> Option<Integer> {
    let byte = *bytes.get(index)?;
    Some(Integer::from(byte))
}

pub fn equals(x: &[u8], y: &[u8]) -> bool {
    x == y
}

pub fn less_than(x: &[u8], y: &[u8]) -> bool {
    x < y
}

pub fn less_than_or_equal(x: &[u8], y: &[u8]) -> bool {
    x <= y
}

pub fn to_integer(big_endian: bool, bytes: &[u8]) -> Integer {
    Integer::from_digits(
        bytes,
        if big_endian {
            rug::integer::Order::Msf
        } else {
            rug::integer::Order::Lsf
        },
    )
}

pub fn and(extend: bool, mut x: Vec<u8>, y: &[u8]) -> Vec<u8> {
    x.iter_mut().zip(y.iter()).for_each(|(a, b)| *a &= b);
    if extend && y.len() > x.len() {
        x.extend_from_slice(&y[x.len()..]);
    } else if !extend && x.len() > y.len() {
        x.truncate(y.len());
    }
    x
}

pub fn or(extend: bool, mut x: Vec<u8>, y: &[u8]) -> Vec<u8> {
    x.iter_mut().zip(y.iter()).for_each(|(a, b)| *a |= b);
    if extend && y.len() > x.len() {
        x.extend_from_slice(&y[x.len()..]);
    } else if !extend && x.len() > y.len() {
        x.truncate(y.len());
    }
    x
}

pub fn xor(extend: bool, mut x: Vec<u8>, y: &[u8]) -> Vec<u8> {
    x.iter_mut().zip(y.iter()).for_each(|(a, b)| *a ^= b);
    if extend && y.len() > x.len() {
        x.extend_from_slice(&y[x.len()..]);
    } else if !extend && x.len() > y.len() {
        x.truncate(y.len());
    }
    x
}

pub fn complement(mut x: Vec<u8>) -> Vec<u8> {
    x.iter_mut().for_each(|b| *b = !*b);
    x
}

pub fn shift(mut x: Vec<u8>, by: i64) -> Vec<u8> {
    if by == 0 {
        return x;
    }

    let len = x.len();
    let magnitude = by.unsigned_abs();
    if magnitude >= (len as u64).saturating_mul(8) {
        x.fill(0);
        return x;
    }

    let byte_shift = magnitude as usize / 8;
    let bit_shift = (magnitude % 8) as u32;

    if by > 0 {
        for i in byte_shift..len {
            x[i - byte_shift] = x[i] << bit_shift;
            x[i - byte_shift] |= x.get(i + 1).unwrap_or(&0).unbounded_shr(8 - bit_shift);
        }
        x[len.saturating_sub(byte_shift)..].fill(0);
    } else {
        for i in (0..len.saturating_sub(byte_shift)).rev() {
            x[i + byte_shift] = x[i] >> bit_shift;
            x[i + byte_shift] |= x
                .get(i.wrapping_sub(1))
                .unwrap_or(&0)
                .unbounded_shl(8 - bit_shift);
        }
        x[..byte_shift.min(len)].fill(0);
    }
    x
}

pub fn rotate(mut x: Vec<u8>, by: i64) -> Vec<u8> {
    if by == 0 || x.is_empty() {
        return x;
    }

    let len = x.len();
    let by = by.rem_euclid((len as i64).saturating_mul(8)) as usize;
    if by == 0 {
        return x;
    }

    let byte_shift = by / 8;
    let bit_shift = by % 8;

    x.rotate_left(byte_shift);
    if bit_shift != 0 {
        let first = x.first().copied().unwrap_or(0);
        for i in 0..len {
            let next = x.get(i + 1).copied().unwrap_or(first);
            x[i] = (x[i] << bit_shift) | (next >> (8 - bit_shift));
        }
    }
    x
}

pub fn count_set_bits(x: &[u8]) -> Integer {
    let count: usize = x.iter().map(|b| b.count_ones() as usize).sum();
    Integer::from(count)
}

pub fn first_set_bit(x: &[u8]) -> Integer {
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

pub fn read_bit(x: &[u8], index: usize) -> Option<bool> {
    let byte_index = index / 8;
    let bit_index = index % 8;
    let byte = *x.get(x.len().checked_sub(1 + byte_index)?)?;
    Some((byte & (1 << bit_index)) != 0)
}

pub fn write_bits(mut x: Vec<u8>, indices: &[Integer], bit: bool) -> Option<Vec<u8>> {
    for index in indices {
        let index = index.to_usize()?;
        let byte_index = index / 8;
        let bit_index = index % 8;
        let index = x.len().checked_sub(1 + byte_index)?;
        let byte = x.get_mut(index)?;
        if bit {
            *byte |= 1 << bit_index;
        } else {
            *byte &= !(1 << bit_index);
        }
    }
    Some(x)
}

pub fn replicate_byte(count: usize, byte: u8) -> Option<Vec<u8>> {
    if count > 8192 {
        return None;
    }
    Some(vec![byte; count])
}
