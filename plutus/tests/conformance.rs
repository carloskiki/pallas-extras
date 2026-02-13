use std::path::PathBuf;

use libtest2_mimic::{Harness, RunContext, RunError, Trial};
use plutus::{Budget, Context, DeBruijn, Program};

const BASE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/conformance");
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/cost-model.rs"));

fn main() {
    let mut directories = vec![PathBuf::from(BASE_DIR)];

    Harness::with_env()
        .discover(std::iter::from_fn(|| {
            while let Some(dir) = directories.pop() {
                let mut is_dir = false;
                for entry in dir.read_dir().unwrap() {
                    let entry = entry.unwrap();
                    if entry.path().is_dir() {
                        directories.push(entry.path());
                        is_dir = true;
                    }
                }
                if !is_dir {
                    let file_name = dir.file_name().unwrap().to_str().unwrap();
                    let program_path = dir.join(file_name).with_extension("uplc");
                    let test_name = dir
                        .strip_prefix(BASE_DIR)
                        .unwrap()
                        .to_string_lossy()
                        .to_string();

                    // if test_name != "uplc/evaluation/builtin/semantics/headList/headList-02" {
                    //     continue;
                    // }

                    return Some(Trial::test(test_name, move |ctx| {
                        perform_test(ctx, &program_path)
                    }));
                }
            }
            None
        }))
        .main()
}

fn perform_test(ctx: RunContext<'_>, program_path: &PathBuf) -> Result<(), RunError> {
    // Skip these tests for now as they require features not yet supported (not yet in the spec)
    if program_path.components().any(|c| {
        c.as_os_str() == "value"
            || c.as_os_str() == "lookupCoin"
            || c.as_os_str() == "insertCoin"
            || c.as_os_str() == "valueContains"
            || c.as_os_str() == "unionValue"
            || c.as_os_str() == "scaleValue"
            || c.as_os_str() == "valueData"
            || c.as_os_str() == "unValueData"
    }) {
        ctx.ignore_for("Requires value built-in type or constant-case support")?;
    }

    let arena = plutus::Arena::default();
    let program = std::fs::read_to_string(program_path).unwrap();
    let expected_path = program_path.to_string_lossy().to_string() + ".expected";
    let expected_output = std::fs::read_to_string(&expected_path)
        .unwrap()
        .trim()
        .to_string();

    let program: Program<String> = match (Program::from_str(&program, &arena), expected_output.as_str()) {
        (Ok(_), "parse error") => return Err(RunError::fail("Expected parse error")),
        (Err(_), "parse error") => return Ok(()),
        (Ok(program), _) => program,
        (Err(_), _) => return Err(RunError::fail("Unexpected parse error")),
    };
    let program_debruijn = match (program.into_de_bruijn(), expected_output.as_str()) {
        (Some(program), _) => program,
        (None, "evaluation failure") => return Ok(()),
        (None, _) => {
            return Err(RunError::fail(
                "Unexpected evaluation error when converting to de Bruijn indices",
            ));
        }
    };

    let flat_path = program_path.with_extension("flat");
    match (std::fs::read(&flat_path), program_debruijn.to_flat()) {
        (Ok(flat), Some(flat_from_program)) => {
            let Some(program_from_flat) = Program::from_flat(&flat, &arena) else {
                return Err(RunError::fail("Failed to parse flat program"));
            };

            if program_from_flat != program_debruijn {
                return Err(RunError::fail(
                    "Flat program does not match original program",
                ));
            }

            let Some(round_trip_program) = Program::from_flat(&flat_from_program, &arena) else {
                return Err(RunError::fail(
                    "Failed to convert round-tripped flat program to de Bruijn",
                ));
            };
            if round_trip_program != program_debruijn {
                return Err(RunError::fail(
                    "Round-tripped flat program does not match original program",
                ));
            }
        }
        (Err(_), Some(_)) => {
            return Err(RunError::fail("Expected flat encoding to fail."));
        }
        (Ok(_), None) => {
            return Err(RunError::fail("Expected flat encoding to succeed."));
        }
        (Err(_), None) => {}
    }

    let budget_path = program_path.with_extension("uplc.budget.expected");
    let Ok(budget_str) = std::fs::read_to_string(&budget_path) else {
        return Err(RunError::fail("Failed to read expected budget file"));
    };
    let budget = if expected_output == "evaluation failure" {
        Budget {
            memory: u64::MAX,
            execution: u64::MAX,
        }
    } else {
        let parse_err = || RunError::fail("Failed to parse expected budget");
        let (execution, memory) = budget_str
            .trim()
            .strip_prefix("({")
            .ok_or_else(parse_err)?
            .strip_suffix("})")
            .ok_or_else(parse_err)?
            .split_once('|')
            .ok_or_else(parse_err)?;
        let execution = execution
            .trim()
            .strip_prefix("cpu:")
            .ok_or_else(parse_err)?
            .trim_start()
            .parse::<u64>()
            .map_err(|_| parse_err())?;
        let memory = memory
            .trim()
            .strip_prefix("mem:")
            .ok_or_else(parse_err)?
            .trim_start()
            .parse::<u64>()
            .map_err(|_| parse_err())?;
        Budget { memory, execution }
    };
    let mut context = Context {
        model: COST_MODEL,
        budget,
    };
    let output = match (
        program_debruijn.evaluate(&mut context),
        expected_output.as_str(),
    ) {
        (Some(_), "evaluation failure") => {
            return Err(RunError::fail("Expected evaluation failure"));
        }
        (None, "evaluation failure") => return Ok(()),
        (Some(p), _) => p,
        // TODO: We should make sure that the error is due to budget exhaustion, once we have
        // descriptive errors.
        (None, _) if budget.execution == i64::MAX as u64 || budget.memory == i64::MAX as u64 => {
            return Ok(());
        }
        (None, _) => return Err(RunError::fail("Unexpected evaluation failure")),
    };
    let expected_program: Program<ExpectedVariable> = Program::from_str(&expected_output, &arena)
        .map_err(|_| RunError::fail("Failed to parse expected output"))?;

    if expected_program != output.into_de_bruijn().unwrap() {
        return Err(RunError::fail(
            "Output program does not match expected program",
        ));
    }
    if context.budget.execution != 0 || context.budget.memory != 0 {
        return Err(RunError::fail(
            "Budget not fully consumed after evaluation",
        ));
    }

    Ok(())
}

#[derive(Debug)]
struct ExpectedVariable(u32);

impl std::str::FromStr for ExpectedVariable {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_name, number) = s.split_once('-').ok_or(())?;
        let index: u32 = number.parse().map_err(|_| ())?;
        Ok(ExpectedVariable(index))
    }
}

impl PartialEq<DeBruijn> for ExpectedVariable {
    fn eq(&self, DeBruijn(index): &DeBruijn) -> bool {
        self.0 == *index
    }
}
