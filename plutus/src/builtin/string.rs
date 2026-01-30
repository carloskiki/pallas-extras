pub fn append(mut x: String, y: String) -> String {
    x.push_str(&y);
    x
}

pub fn equals(x: String, y: String) -> bool {
    x == y
}

pub fn encode_utf8(x: String) -> Vec<u8> {
    x.into_bytes()
}

pub fn decode_utf8(x: Vec<u8>) -> Option<String> {
    String::from_utf8(x).ok()
}
