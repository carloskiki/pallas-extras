mod cmap;
mod hmap;
mod zip;
mod identity;
mod overwrite;

pub use cmap::CMap;
pub use hmap::HMap;
pub use zip::Zip;
pub use identity::Identity;
pub use overwrite::Overwrite;

/// A trait that works as a function signature at the type level
pub trait TypeMap<Input> {
    type Output;
}
