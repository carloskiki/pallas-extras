use sparse_struct::SparseStruct;

#[derive(SparseStruct)]
enum Enum {
    VariantA(i32),
    VariantB {
        field1: String,
        field2: f64,
    },
}

pub fn main() {}
