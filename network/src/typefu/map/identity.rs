use super::TypeMap;

/// The Identity TypeMap; a signature that maps a type to its.
pub enum Identity {}
impl<T> TypeMap<T> for Identity {
    type Output = T;
}
