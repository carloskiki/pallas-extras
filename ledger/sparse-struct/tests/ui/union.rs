use sparse_struct::SparseStruct;

#[derive(SparseStruct)]
union Union {
    a: u32,
    b: u64,
}

pub fn main() {}
