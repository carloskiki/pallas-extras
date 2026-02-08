use std::{path::Path, str::FromStr};

use libtest2_mimic::{Harness, RunError, Trial};
use plutus::{Budget, Context, DeBruijn, Program};

const FLAT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/benches/validation");
const EXPECTED_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/validation");
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/cost-model.rs"));

fn main() {
    let dir = std::fs::read_dir(FLAT_DIR).unwrap();

    Harness::with_env()
        .discover(dir.map(|entry| {
            let path = entry.unwrap().path();
            let flat = std::fs::read(&path).unwrap();
            let file_name = path.file_name().unwrap();
            let (budget, output) = parse_expected(
                &std::fs::read_to_string(
                    <_ as AsRef<Path>>::as_ref(EXPECTED_DIR)
                        .join(file_name)
                        .with_extension("expected"),
                )
                .unwrap(),
            )
            .unwrap();
            let test_name = path.file_stem().unwrap().to_str().unwrap().to_string();

            Trial::test(test_name, move |_| perform_test(&flat, budget, &output))
        }))
        .main()
}

fn perform_test(flat: &[u8], budget: Budget, expected: &Program<DeBruijn>) -> Result<(), RunError> {
    let program = Program::from_flat(flat).unwrap();
    let mut context = Context {
        model: COST_MODEL,
        budget,
    };
    let result = program.evaluate(&mut context).unwrap();
    assert_eq!(&result.into_de_bruijn().unwrap(), expected);
    assert_eq!(context.budget.execution, 0);
    assert_eq!(context.budget.memory, 0);
    Ok(())
}

fn parse_expected(input: &str) -> Option<(Budget, Program<DeBruijn>)> {
    let mut lines = input.lines();

    let cpu_line = lines.next()?.trim();
    let memory_line = lines.next()?.trim();

    let cpu_str = cpu_line.strip_prefix("CPU:")?.trim().replace('_', "");
    let memory_str = memory_line.strip_prefix("Memory:")?.trim().replace('_', "");
    let cpu: u64 = cpu_str.parse().ok()?;
    let memory: u64 = memory_str.parse().ok()?;
    let budget = Budget {
        execution: cpu,
        memory,
    };

    let program_str = std::iter::once("(program 1.0.0 ")
        .chain(lines.skip(2))
        .chain(std::iter::once(")"))
        .collect::<String>();
    let program = Program::<String>::from_str(&program_str).ok()?;
    Some((budget, program.into_de_bruijn()?))
}
