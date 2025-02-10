use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ed25519_bip32::XPrv;
use ponk::{ExtendedSecretKey, HardIndex, SoftIndex};
use rand::random;

pub fn private_derive(c: &mut Criterion) {
    let mut group = c.benchmark_group("Private Key Derivation");
    let master: [u8; 32] = random();
    let ponk = ExtendedSecretKey::new_force(master);
    let xprv = XPrv::from_nonextended_force(&master, &ponk.chain_code);

    group.bench_function("ponk", |b| {
        b.iter_with_setup(random, |i: u32| {
            black_box(ponk.derive_child(HardIndex::new(i)));
        })
    });
    group.bench_function("Reference", |b| {
        b.iter_with_setup(random, |i: u32| {
            black_box(xprv.derive(ed25519_bip32::DerivationScheme::V2, i));
        })
    });
}

pub fn public_derive(c: &mut Criterion) {
    let mut group = c.benchmark_group("Public Key Derivation");
    let master: [u8; 32] = random();
    let ponk = ExtendedSecretKey::new_force(master);
    let xprv = XPrv::from_nonextended_force(&master, &ponk.chain_code);
    let ponk = ponk.verifying_key();
    let xpub = xprv.public();
    
    group.bench_function("ponk", |b| {
        b.iter_with_setup(
            || random::<u32>() >> 1,
            |i: u32| {
                black_box(ponk.derive_child(SoftIndex::new(i)));
            },
        )
    });

    group.bench_function("Reference", |b| {
        b.iter_with_setup(
            || random::<u32>() >> 1,
            |i: u32| {
                black_box(xpub.derive(ed25519_bip32::DerivationScheme::V2, i).unwrap());
            },
        )
    });
}

criterion_group!(benches, private_derive, public_derive);
criterion_main!(benches);
