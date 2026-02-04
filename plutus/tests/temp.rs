

include!("../cost-model.rs");

#[test]
fn temp() {
    let flat = include_bytes!("../benches/validation/coop-2.flat");
    let prog = plutus::Program::from_flat(flat).unwrap();
    let mut context = plutus::Context {
        model: COST_MODEL,
        budget: plutus::Budget {
            memory: u64::MAX,
            execution: u64::MAX,
        },
    };
    let result = prog.evaluate(&mut context).unwrap();
}
