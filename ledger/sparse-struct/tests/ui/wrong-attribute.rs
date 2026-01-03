use sparse_struct::SparseStruct;

#[derive(SparseStruct)]
#[struct_name(Hello)]
enum Enum {
    Variant1(u8),
    Variant2(u16),
}

#[derive(SparseStruct)]
enum Enum2 {
    #[struct_name = "Hello"]
    Variant1(u8),
    Variant2(u16),
}

fn main() {}
