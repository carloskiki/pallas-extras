use sparse_struct::SparseStruct;

#[derive(SparseStruct)]
#[struct_name("hello")]
enum Enum {
    Variant1(u8),
    Variant2(u16),
}

#[derive(SparseStruct)]
enum Enum2 {
    Variant1(u8),
    #[struct_name("hello")]
    Variant2(u16),
}

fn main() {}
