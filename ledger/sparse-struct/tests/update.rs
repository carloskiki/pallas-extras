extern crate alloc;

use sparse_struct::SparseStruct;

#[derive(Clone, SparseStruct)]
#[full_name = "FullAttributes"]
#[allow(dead_code)]
enum Attribute {
    Age(u8),
    Name(String),
    Active(bool),
}

#[test]
fn full_struct_updates_only_present_fields() {
    let mut attributes = FullAttributes {
        age: 5,
        name: "Alice".into(),
        active: true,
    };

    let update = AttributeSet::from_iter([Attribute::Age(42), Attribute::Name("Bob".into())]);

    attributes.update(&update);

    assert_eq!(attributes.age, 42);
    assert_eq!(attributes.name, "Bob");
    assert!(attributes.active);
    assert_eq!(update.name().map(String::as_str), Some("Bob"));
}
