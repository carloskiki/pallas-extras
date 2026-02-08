use plutus::Program;

fn main() {
    afl::fuzz!(|data: &[u8]| {
        let _ = Program::from_flat(data);
    })
}
