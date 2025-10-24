use macro_rules_attribute::apply;
use rug::ops::{DivRounding, RemRounding};

use super::builtin;

#[apply(builtin)]
pub fn add(x: rug::Integer, y: rug::Integer) -> rug::Integer {
    x + y
}

#[apply(builtin)]
pub fn subtract(x: rug::Integer, y: rug::Integer) -> rug::Integer {
    x - y
}

#[apply(builtin)]
pub fn multiply(x: rug::Integer, y: rug::Integer) -> rug::Integer {
    x * y
}

#[apply(builtin)]
pub fn divide(x: rug::Integer, y: rug::Integer) -> Option<rug::Integer> {
    if y.is_zero() {
        None
    } else {
        Some(x.div_floor(y))
    }
}

#[apply(builtin)]
pub fn modulo(x: rug::Integer, y: rug::Integer) -> Option<rug::Integer> {
    if y.is_zero() {
        None
    } else {
        Some(x.rem_floor(y))
    }
}

#[apply(builtin)]
pub fn quotient(x: rug::Integer, y: rug::Integer) -> Option<rug::Integer> {
    if y.is_zero() { None } else { Some(x / y) }
}

#[apply(builtin)]
pub fn remainder(x: rug::Integer, y: rug::Integer) -> Option<rug::Integer> {
    if y.is_zero() { None } else { Some(x % y) }
}

#[apply(builtin)]
pub fn equals(x: rug::Integer, y: rug::Integer) -> bool {
    x == y
}

#[apply(builtin)]
pub fn less_than(x: rug::Integer, y: rug::Integer) -> bool {
    x < y
}

#[apply(builtin)]
pub fn less_than_or_equal(x: rug::Integer, y: rug::Integer) -> bool {
    x <= y
}

#[apply(builtin)]
pub fn to_bytes(big_endian: bool, width: rug::Integer, num: rug::Integer) -> Option<Vec<u8>> {
    let width = width.to_usize()?;
    if width > 8192 || num.cmp0() == std::cmp::Ordering::Less {
        return None;
    }
    let num_len = num.significant_digits::<u8>();
    if num_len > 8192 {
        return None;
    }

    let (mut bytes, padding) = if width == 0 {
        (vec![0; num_len], 0)
    } else {
        let padding = width.checked_sub(num_len)?;
        (vec![0; width], padding)
    };

    if big_endian {
        num.write_digits(&mut bytes[padding..], rug::integer::Order::Msf);
    } else {
        num.write_digits(&mut bytes[..num_len], rug::integer::Order::Lsf);
    }

    Some(bytes)
}

#[apply(builtin)]
pub fn exp_mod(
    base: rug::Integer,
    exponent: rug::Integer,
    modulus: rug::Integer,
) -> Option<rug::Integer> {
    base.pow_mod(&exponent, &modulus).ok()
}
