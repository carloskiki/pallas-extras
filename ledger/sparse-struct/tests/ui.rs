#[ignore = "compiler diagnostics can be flaky on nightly"]
#[test]
fn tests() {
    let ui = trybuild::TestCases::new();
    ui.compile_fail("tests/ui/*.rs");
}
