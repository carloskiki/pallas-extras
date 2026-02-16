use criterion::{Criterion, criterion_group, criterion_main};
use plutus::{Budget, Context, Program};

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/cost-model.rs"));

pub fn bench(c: &mut Criterion) {
    let dir = std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/benches/validation"));
    for entry in dir.unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let bench_name = path.file_stem().unwrap().to_str().unwrap();
        let flat = std::fs::read(&path).unwrap();

        let mut group = c.benchmark_group(bench_name);
        group.bench_with_input("decode", &flat, |b, input| {
            b.iter(|| {
                let arena = plutus::Arena::default();
                let program = Program::from_flat(input, &arena).unwrap();
                std::hint::black_box(program);
            });
        });
        group.bench_with_input("full", &flat, |b, flat| {
            b.iter(|| {
                let arena = plutus::Arena::default();
                let program = Program::from_flat(flat, &arena).unwrap();
                let mut context = Context {
                    model: COST_MODEL,
                    budget: Budget {
                        execution: u64::MAX,
                        memory: u64::MAX,
                    },
                };
                let result = program.evaluate(&mut context).unwrap();
                std::hint::black_box(result);
            });
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
