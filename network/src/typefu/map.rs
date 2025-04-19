mod cmap;
mod hmap;
mod zip;
mod identity;
mod overwrite;
mod unzip;
mod fold;

pub use cmap::CMap;
pub use hmap::HMap;
pub use zip::Zip;
pub use identity::Identity;
pub use overwrite::Overwrite;
pub use unzip::{Unzip, UnzipLeft, UnzipRight};
pub use fold::Fold;

/// A trait that works as a function signature at the type level
pub trait TypeMap<Input> {
    type Output;
}
