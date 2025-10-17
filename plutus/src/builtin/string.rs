pub fn append(mut x: String, y: &str) -> String {
    x.push_str(y);
    x
}

pub fn equals(x: &str, y: &str) -> bool {
    x == y
}

pub fn encode_utf8(x: String) -> Vec<u8> {
    x.into_bytes()
}

pub fn decode_utf8(x: &[u8]) -> Option<String> {
    String::from_utf8(x.to_vec()).ok()
}
