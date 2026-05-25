use sparse_struct::SparseStruct;

#[derive(SparseStruct)]
#[sparse_name(Hello)]
enum Enum {
    Variant1(u8),
    Variant2(u16),
}

#[derive(SparseStruct)]
#[full_name(Hello)]
enum EnumFull {
    Variant1(u8),
    Variant2(u16),
}

#[derive(SparseStruct)]
enum Enum2 {
    #[sparse_name = "Hello"]
    Variant1(u8),
    Variant2(u16),
}

#[derive(SparseStruct)]
enum Enum3 {
    #[full_name = "Hello"]
    Variant1(u8),
    Variant2(u16),
}

fn main() {}
