pub fn append(mut x: Vec<u8>, y: &[u8]) -> Vec<u8> {
    x.extend(y);
    x
}

pub fn cons_v1(x: &rug::Integer, mut y: Vec<u8>) -> Vec<u8> {
    let byte = x.to_u8_wrapping();
    y.insert(0, byte);
    y
}

pub fn cons_v2(x: &rug::Integer, mut y: Vec<u8>) -> Option<Vec<u8>> {
    let byte = x.to_u8()?;
    y.insert(0, byte);
    Some(y)
}

pub fn slice(start: &rug::Integer, end: &rug::Integer, bytes: Vec<u8>) -> Vec<u8> {
    let (Some(start), Some(end)) = (start.to_usize(), end.to_usize()) else {
        return vec![];
    };

    bytes.get(start..end).unwrap_or(&[]).to_vec()
}

pub fn length(bytes: &[u8]) -> rug::Integer {
    rug::Integer::from(bytes.len())
}

pub fn index(index: &rug::Integer, bytes: &[u8]) -> Option<rug::Integer> {
    let index = index.to_usize()?;
    let byte = *bytes.get(index)?;
    Some(rug::Integer::from(byte))
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

pub fn to_integer(big_endian: bool, bytes: &[u8]) -> rug::Integer {
    rug::Integer::from_digits(
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

pub fn shift(mut x: Vec<u8>, by: rug::Integer) -> Vec<u8> {
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

pub fn rotate(mut x: Vec<u8>, by: rug::Integer) -> Vec<u8> {
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

pub fn count_set_bits(x: &[u8]) -> rug::Integer {
    let count: usize = x.iter().map(|b| b.count_ones() as usize).sum();
    rug::Integer::from(count)
}

pub fn first_set_bit(x: &[u8]) -> rug::Integer {
    let mut index = 0;
    for byte in x.iter().rev() {
        if byte.trailing_zeros() < 8 {
            index += byte.trailing_zeros() as usize;
            return rug::Integer::from(index);
        }
        index += 8;
    }
    rug::Integer::from(-1)
}

pub fn read_bit(x: &[u8], index: &rug::Integer) -> Option<bool> {
    let index = index.to_usize()?;
    let byte_index = index / 8;
    let bit_index = index % 8;
    let byte = *x.get(byte_index)?;
    Some((byte & (1 << bit_index)) != 0)
}

pub fn write_bits(mut x: Vec<u8>, indices: &[rug::Integer], bit: bool) -> Option<Vec<u8>> {
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

pub fn replicate_byte(count: &rug::Integer, byte: &rug::Integer) -> Option<Vec<u8>> {
    let byte = byte.to_u8()?;
    let count = count.to_usize()?;
    if count > 8192 {
        return None;
    }
    Some(vec![byte; count])
}
