use std::str::FromStr;

use plutus::{Budget, DeBruijn, Program};

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/conformance/cost-model.rs"
));

/// Parse an expected result file with the following format:
///
/// ```text
/// CPU:               185_243_960
/// Memory:                831_092
/// AST Size:                3_685
/// Flat Size:               3_722
///
/// <program>
/// ```
///
/// Where `<program>` is the expected textual program output.
pub fn parse_expected(input: &str) -> Option<(Budget, Program<DeBruijn>)> {
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
