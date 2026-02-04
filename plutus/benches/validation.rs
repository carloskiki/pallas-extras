use criterion::{Criterion, criterion_group, criterion_main};
use plutus::{Context, Program};

mod shared;

pub fn bench(c: &mut Criterion) {
    // Pairs of file `*.flat` and `*.expected`.
    let dir = std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/benches/validation"));
    for entry in dir.unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("flat") {
            let bench_name = path.file_stem().unwrap().to_str().unwrap();
            let flat = std::fs::read(&path).unwrap();
            let (budget, output) =
                shared::parse_expected(&std::fs::read_to_string(path.with_extension("expected")).unwrap())
                    .unwrap();

            let mut group = c.benchmark_group(bench_name);
            group.bench_with_input("decode", &flat, |b, input| {
                b.iter(|| {
                    Program::from_flat(input).unwrap();
                });
            });
            group.bench_with_input("evaluate", &(flat, output, budget), |b, (flat, output, budget)| {
                b.iter(|| {
                    let program = Program::from_flat(flat).unwrap();
                    let mut context = Context {
                        model: shared::COST_MODEL,
                        budget: *budget,
                    };
                    let result = program.evaluate(&mut context).unwrap();
                    // assert_eq!(&result, output);
                    // assert_eq!(context.budget.execution, 0);
                    // assert_eq!(context.budget.memory, 0);
                    std::hint::black_box(result);
                });
            });
        }
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
