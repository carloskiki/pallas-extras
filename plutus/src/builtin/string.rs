use super::builtin;
use macro_rules_attribute::apply;

#[apply(builtin)]
pub fn append(mut x: String, y: String) -> String {
    x.push_str(&y);
    x
}

#[apply(builtin)]
pub fn equals(x: String, y: String) -> bool {
    x == y
}

#[apply(builtin)]
pub fn encode_utf8(x: String) -> Vec<u8> {
    x.into_bytes()
}

#[apply(builtin)]
pub fn decode_utf8(x: Vec<u8>) -> Option<String> {
    String::from_utf8(x).ok()
}
