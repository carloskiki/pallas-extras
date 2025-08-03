#[derive(SparseStruct)]
enum Enum {
    VariantA(i32),
    VariantB(String),
    VariantC(u32, u64),
}

fn main() {}
