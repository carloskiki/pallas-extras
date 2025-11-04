use blstrs::{G1Affine, G1Projective, G2Affine, G2Projective, Gt, MillerLoopResult, Scalar};
use macro_rules_attribute::apply;
use rug::ops::RemRounding;

use super::builtin;

#[apply(builtin)]
pub fn g1_add(p: G1Projective, q: G1Projective) -> G1Projective {
    p + q
}

#[apply(builtin)]
pub fn g1_neg(p: G1Projective) -> G1Projective {
    -p
}

/// Constant representing the modulus
/// q = 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001
const SCALAR_MODULUS: [u64; 4] = [
    0xffff_ffff_0000_0001,
    0x53bd_a402_fffe_5bfe,
    0x3339_d808_09a1_d805,
    0x73ed_a753_299d_7d48,
];

#[apply(builtin)]
pub fn g1_scalar_mul(scalar: rug::Integer, p: G1Projective) -> G1Projective {
    let integer = scalar.rem_floor(rug::Integer::from_digits(
        &SCALAR_MODULUS,
        rug::integer::Order::Lsf,
    ));
    let mut scalar_bytes = [0; 32];
    integer.write_digits(&mut scalar_bytes, rug::integer::Order::Lsf);
    let scalar = Scalar::from_bytes_le(&scalar_bytes).expect("scalar is valid");
    p * scalar
}

#[apply(builtin)]
pub fn g1_equals(p: G1Projective, q: G1Projective) -> bool {
    p == q
}

#[apply(builtin)]
pub fn g1_hash_to_group(msg: Vec<u8>, domain: Vec<u8>) -> Option<G1Projective> {
    if domain.len() > 255 {
        return None;
    }
    Some(blstrs::G1Projective::hash_to_curve(&msg, &domain, &[]))
}

#[apply(builtin)]
pub fn g1_compress(p: G1Projective) -> Vec<u8> {
    let affine = G1Affine::from(p);
    let compressed = affine.to_compressed();
    compressed.to_vec()
}

#[apply(builtin)]
pub fn g1_uncompress(bytes: Vec<u8>) -> Option<G1Projective> {
    let affine = G1Affine::from_compressed(&bytes.try_into().ok()?).into_option()?;
    Some(G1Projective::from(affine))
}

#[apply(builtin)]
pub fn g2_add(p: G2Projective, q: G2Projective) -> G2Projective {
    p + q
}

#[apply(builtin)]
pub fn g2_neg(p: G2Projective) -> G2Projective {
    -p
}

#[apply(builtin)]
pub fn g2_scalar_mul(scalar: rug::Integer, p: G2Projective) -> G2Projective {
    let integer = scalar.rem_floor(rug::Integer::from_digits(
        &SCALAR_MODULUS,
        rug::integer::Order::Lsf,
    ));
    let mut scalar_bytes = [0; 32];
    integer.write_digits(&mut scalar_bytes, rug::integer::Order::Lsf);
    let scalar = Scalar::from_bytes_le(&scalar_bytes).expect("scalar is valid");
    p * scalar
}

#[apply(builtin)]
pub fn g2_equals(p: G2Projective, q: G2Projective) -> bool {
    p == q
}

#[apply(builtin)]
pub fn g2_hash_to_group(msg: Vec<u8>, domain: Vec<u8>) -> Option<G2Projective> {
    if domain.len() > 255 {
        return None;
    }
    Some(blstrs::G2Projective::hash_to_curve(&msg, &domain, &[]))
}

#[apply(builtin)]
pub fn g2_compress(p: G2Projective) -> Vec<u8> {
    let affine = G2Affine::from(p);
    let compressed = affine.to_compressed();
    compressed.to_vec()
}

#[apply(builtin)]
pub fn g2_uncompress(bytes: Vec<u8>) -> Option<G2Projective> {
    let affine = G2Affine::from_compressed(&bytes.try_into().ok()?).into_option()?;
    Some(G2Projective::from(affine))
}

#[apply(builtin)]
pub fn miller_loop(p: G1Projective, q: G2Projective) -> MillerLoopResult {
    blstrs::miller_loop(&p.into(), &q.into())
}

#[apply(builtin)]
pub fn mul_ml_result(a: MillerLoopResult, b: MillerLoopResult) -> MillerLoopResult {
    // Weird, `blstrs`'s `add` implementation on MillerLoopResult is actually multiplication...
    a + b
}

#[apply(builtin)]
pub fn final_verify(ml_result: MillerLoopResult, target: MillerLoopResult) -> bool {
    ml_result.final_verify(&target)
}

#[apply(builtin)]
pub fn g1_multi_scalar_mul(
    scalars: Vec<rug::Integer>,
    points: Vec<G1Projective>,
) -> G1Projective {
    let count = scalars.len().min(points.len());
    if count == 0 {
        return <G1Projective as k256::elliptic_curve::Group>::identity();
    }
    
    let mut bytes: Vec<u8> = vec![0; scalars.len() * 32];
    scalars
        .iter()
        .take(count)
        .enumerate()
        .for_each(|(i, scalar)| {
            let integer = scalar.rem_floor(rug::Integer::from_digits(
                &SCALAR_MODULUS,
                rug::integer::Order::Lsf,
            ));

            integer.write_digits(&mut bytes[i * 32..(i + 1) * 32], rug::integer::Order::Lsf);
        });

    // Safety:(blstrs) the `G1Projective` struct in blstrs is `repr(transparent)` over
    // `blst::blst_p1`.
    let points: &[blst::blst_p1] = unsafe {
        std::slice::from_raw_parts(
            points.as_ptr() as *const blst::blst_p1,
            count,
        )
    };
    let points = blst::p1_affines::from(points);
    
    // Safety:(blstrs) the return type of `mult` is `blst::blst_p1` which is
    // `repr(transparent)` over `G1Projective`.
    unsafe { std::mem::transmute(points.mult(&bytes, 255)) }
}

#[apply(builtin)]
pub fn g2_multi_scalar_mul(
    scalars: Vec<rug::Integer>,
    points: Vec<G2Projective>,
) -> G2Projective {
    let count = scalars.len().min(points.len());
    if count == 0 {
        return <G2Projective as k256::elliptic_curve::Group>::identity();
    }
    
    let mut bytes: Vec<u8> = vec![0; scalars.len() * 32];
    scalars
        .iter()
        .take(count)
        .enumerate()
        .for_each(|(i, scalar)| {
            let integer = scalar.rem_floor(rug::Integer::from_digits(
                &SCALAR_MODULUS,
                rug::integer::Order::Lsf,
            ));

            integer.write_digits(&mut bytes[i * 32..(i + 1) * 32], rug::integer::Order::Lsf);
        });

    // Safety:(blstrs) the `G2Projective` struct in blstrs is `repr(transparent)` over
    // `blst::blst_p2`.
    let points: &[blst::blst_p2] = unsafe {
        std::slice::from_raw_parts(
            points.as_ptr() as *const blst::blst_p2,
            count,
        )
    };
    let points = blst::p2_affines::from(points);
    
    // Safety:(blstrs) the return type of `mult` is `blst::blst_p2` which is
    // `repr(transparent)` over `G2Projective`.
    unsafe { std::mem::transmute(points.mult(&bytes, 255)) }
}
