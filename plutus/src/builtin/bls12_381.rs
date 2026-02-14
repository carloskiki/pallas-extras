use bwst::{
    g1, g2,
    group::{GroupEncoding, ff::PrimeField},
    miller_loop,
    scalar::{MODULUS, Scalar},
};
use rug::ops::RemRounding;

fn scalar_from_integer(scalar: &rug::Integer) -> Scalar {
    let integer = scalar.rem_floor(rug::Integer::from_digits(
        &MODULUS,
        rug::integer::Order::Lsf,
    ));
    let mut scalar_bytes = [0; 32];
    integer.write_digits(&mut scalar_bytes, rug::integer::Order::Lsf);
    Scalar::from_repr(scalar_bytes).expect("scalar is valid")
}

pub fn g1_add(p: g1::Projective, q: &g1::Projective) -> g1::Projective {
    p + q
}

pub fn g1_neg(p: g1::Projective) -> g1::Projective {
    -p
}

pub fn g1_scalar_mul(scalar: &rug::Integer, p: g1::Projective) -> g1::Projective {
    let scalar = scalar_from_integer(scalar);
    p * scalar
}

pub fn g1_equals(p: &g1::Projective, q: &g1::Projective) -> bool {
    p == q
}

pub fn g1_hash_to_group(msg: &[u8], domain: &[u8]) -> Option<g1::Projective> {
    if domain.len() > 255 {
        return None;
    }
    Some(g1::Projective::hash_to_curve(&msg, &domain, &[]))
}

pub fn g1_compress(p: &g1::Projective) -> Vec<u8> {
    p.to_bytes().0.to_vec()
}

pub fn g1_uncompress(bytes: &[u8]) -> Option<g1::Projective> {
    let compressed = g1::Compressed(bytes.try_into().ok()?);
    g1::Projective::from_bytes(&compressed).into_option()
}

pub fn g2_add(p: g2::Projective, q: &g2::Projective) -> g2::Projective {
    p + q
}

pub fn g2_neg(p: g2::Projective) -> g2::Projective {
    -p
}

pub fn g2_scalar_mul(scalar: &rug::Integer, p: g2::Projective) -> g2::Projective {
    let scalar = scalar_from_integer(&scalar);
    p * scalar
}

pub fn g2_equals(p: &g2::Projective, q: &g2::Projective) -> bool {
    p == q
}

pub fn g2_hash_to_group(msg: &[u8], domain: &[u8]) -> Option<g2::Projective> {
    if domain.len() > 255 {
        return None;
    }
    Some(g2::Projective::hash_to_curve(&msg, &domain, &[]))
}

pub fn g2_compress(p: &g2::Projective) -> Vec<u8> {
    p.to_bytes().0.to_vec()
}

pub fn g2_uncompress(bytes: &[u8]) -> Option<g2::Projective> {
    let compressed = g2::Compressed(bytes.try_into().ok()?);
    g2::Projective::from_bytes(&compressed).into_option()
}

pub fn miller_loop(p: &g1::Projective, q: &g2::Projective) -> miller_loop::Result {
    miller_loop::Result::miller_loop(p, q)
}

pub fn mul_ml_result(a: miller_loop::Result, b: miller_loop::Result) -> miller_loop::Result {
    a * b
}

pub fn final_verify(ml_result: &miller_loop::Result, target: &miller_loop::Result) -> bool {
    ml_result.final_verify(target)
}

pub fn g1_multi_scalar_mul(scalars: &[rug::Integer], points: &[g1::Projective]) -> g1::Projective {
    let scalars: Vec<_> = scalars.iter().map(scalar_from_integer).collect();
    bwst::g1::Projective::linear_combination(&points, &scalars)
}

pub fn g2_multi_scalar_mul(scalars: &[rug::Integer], points: &[g2::Projective]) -> g2::Projective {
    let scalars: Vec<_> = scalars.iter().map(scalar_from_integer).collect();
    bwst::g2::Projective::linear_combination(&points, &scalars)
}
