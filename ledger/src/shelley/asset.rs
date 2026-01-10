pub mod name;
pub use name::Name;

pub type Asset<'a, T> = Vec<(&'a crate::crypto::Blake2b224Digest, Bundle<'a, T>)>;

pub type Bundle<'a, T> = Vec<(Name<'a>, T)>;
