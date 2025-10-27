use std::path::PathBuf;

use libtest2_mimic::{Harness, RunContext, RunError, Trial};
use plutus::program::Program;

const BASE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/conformance");

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

                    // Filter for dbg
                    // if test_name != "uplc/evaluation/example/churchSucc" {
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
    }) {
        return ctx.ignore_for("Requires value built-in type support");
    } else if program_path
        .components()
        .any(|c| c.as_os_str() == "constr-08")
    {
        return ctx.ignore_for("Requires large construct index support");
    }

    let program = std::fs::read_to_string(program_path).unwrap();
    let expected_path = program_path.to_string_lossy().to_string() + ".expected";
    let expected_output = std::fs::read_to_string(&expected_path)
        .unwrap()
        .trim()
        .to_string();

    let program: Program<String> = match (program.parse(), expected_output.as_str()) {
        (Ok(_), "parse error") => return Err(RunError::fail("Expected parse error")),
        (Err(_), "parse error") => return Ok(()),
        (Ok(program), _) => {
            program
        }
        (Err(_), _) => return Err(RunError::fail("Unexpected parse error")),
    };
    let cannonical = match (program.into_de_bruijn(), expected_output.as_str()) {
        (Some(program), _) => program,
        (None, "evaluation failure") => return Ok(()),
        (None, _) => return Err(RunError::fail("Unexpected evaluation error when converting to de Bruijn indices")),
    };
    let output =  match (cannonical.evaluate(), expected_output.as_str()) {
        (Some(_), "evaluation failure") => return Err(RunError::fail("Expected evaluation failure")),
        (None, "evaluation failure") => return Ok(()),
        (Some(p), _) => p,
        (None, _) => return Err(RunError::fail("Unexpected evaluation failure")),
    };

    Ok(())
}
