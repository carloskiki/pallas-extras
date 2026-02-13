use sha2::Digest;

pub fn digest<D: Digest>(data: &[u8]) -> Vec<u8> {
    D::digest(data).to_vec()
}
