use sparse_struct::SparseStruct;

#[derive(SparseStruct)]
enum Enum {
    VariantA(i32),
    VariantB,
}

pub fn main() {}
