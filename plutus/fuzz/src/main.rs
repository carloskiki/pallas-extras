use plutus::Program;

fn main() {
    afl::fuzz!(|data: &[u8]| {
        let arena = plutus::Arena::default();
        let _ = Program::from_flat(data, &arena);
    })
}
