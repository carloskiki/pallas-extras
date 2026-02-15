use bip32::{ExtendedSecretKey, HardIndex, SoftIndex};
use criterion::{Criterion, criterion_group, criterion_main};
use ed25519_bip32::XPrv;
use rand::random;
use std::hint::black_box;

pub fn private_derive(c: &mut Criterion) {
    let mut group = c.benchmark_group("Private Key Derivation");
    let master: [u8; 32] = random();
    let cc: [u8; 32] = random();
    let ours = ExtendedSecretKey::from_nonextended(master, cc);
    let baseline = XPrv::from_nonextended_force(&master, &cc);

    group.bench_function("ours", |b| {
        b.iter_with_setup(random, |i: u32| {
            black_box(ours.derive_child(HardIndex::new(i)));
        })
    });
    group.bench_function("baseline", |b| {
        b.iter_with_setup(random, |i: u32| {
            black_box(baseline.derive(ed25519_bip32::DerivationScheme::V2, i | 0x80000000));
        })
    });
}

pub fn public_derive(c: &mut Criterion) {
    let mut group = c.benchmark_group("Public Key Derivation");
    let master: [u8; 32] = random();
    let cc: [u8; 32] = random();
    let ours = ExtendedSecretKey::from_nonextended(master, cc);
    let baseline = XPrv::from_nonextended_force(&master, &cc);
    let ours = ours.verifying_key();
    let baseline = baseline.public();

    group.bench_function("ours", |b| {
        b.iter_with_setup(
            || random::<u32>() >> 1,
            |i: u32| {
                let _ = black_box(ours.derive_child(SoftIndex::new(i)));
            },
        )
    });

    group.bench_function("baseline", |b| {
        b.iter_with_setup(
            || random::<u32>() >> 1,
            |i: u32| {
                black_box(
                    baseline
                        .derive(ed25519_bip32::DerivationScheme::V2, i)
                        .unwrap(),
                );
            },
        )
    });
}

criterion_group!(benches, private_derive, public_derive);
criterion_main!(benches);
