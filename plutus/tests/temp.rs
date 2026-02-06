use plutus::{Program, Context, Budget};

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/cost-model.rs"));

#[test]
fn temp() {
    const FLAT: &[u8] = include_bytes!("../benches/validation/coop-2.flat");
    let program = Program::from_flat(FLAT).unwrap();
    let mut context = Context {
        model: COST_MODEL,
        budget: Budget {
            execution: u64::MAX,
            memory: u64::MAX,
        },
    };
    program.evaluate(&mut context).unwrap();
    
}
