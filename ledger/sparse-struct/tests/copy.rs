extern crate alloc;

use sparse_struct::SparseStruct;

#[derive(Clone, SparseStruct)]
#[allow(dead_code)]
#[full_name = "FullAttributes"]
#[struct_derive(Clone, Copy)]
enum Attribute {
    Age(u8),
    Active(bool),
}

#[test]
fn full_struct_can_be_copied() {
    let attributes = FullAttributes {
        age: 42,
        active: true,
    };
    let copied = attributes;

    assert_eq!(attributes.age, copied.age);
    assert_eq!(attributes.active, copied.active);
}
