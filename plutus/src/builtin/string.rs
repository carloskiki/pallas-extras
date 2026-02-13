pub fn append(mut x: String, y: &str) -> String {
    x.push_str(y);
    x
}

pub fn equals(x: &str, y: &str) -> bool {
    x == y
}

pub fn encode_utf8(x: &str) -> &[u8] {
    x.as_bytes()
}

pub fn decode_utf8(x: &[u8]) -> Option<&str> {
    std::str::from_utf8(x).ok()
}
