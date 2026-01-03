use sparse_struct::SparseStruct;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SparseStruct)]
#[struct_derive(Copy)]
enum Enum1 {
    VariantA(i32),
    VariantB(i32),
    VariantC(bool),
    VariantD(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SparseStruct)]
#[struct_derive = ""]
enum Enum2 {
    VariantA(i32),
    VariantB(i32),
    VariantC(bool),
    VariantD(char),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SparseStruct)]
#[struct_derive([u8; 3])]
enum Enum3 {
    VariantA(i32),
    VariantB(i32),
    VariantC(bool),
    VariantD(char),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SparseStruct)]
#[struct_derive(NotInScope)]
enum Enum4 {
    VariantA(i32),
    VariantB(i32),
    VariantC(bool),
    VariantD(char),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SparseStruct)]
enum Enum5 {
    #[struct_derive(Clone)]
    VariantA(i32),
    VariantB(i32),
    VariantC(bool),
    VariantD(char),
}

fn main() {}
