#![feature(test)]

extern crate test;
use test::Bencher;

include!("../cost-model.rs");

#[bench]
fn temp(b: &mut Bencher) {
    b.iter(|| {
        let program = plutus::Program::from_flat(include_bytes!("../benches/validation/auction_1-1.flat")).unwrap();
        let mut context = plutus::Context {
            model: COST_MODEL,
            budget: plutus::Budget {
                execution: u64::MAX,
                memory: u64::MAX,
            },
        };
        let result = program.evaluate(&mut context).unwrap();
        std::hint::black_box(result);
    });
}
