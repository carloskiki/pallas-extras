//! Implementation of the [network specification][net-spec] for the Cardano protocol.
//!
//! [net-spec]: https://ouroboros-network.cardano.intersectmbo.org/pdfs/network-spec/network-spec.pdf

use tinycbor_derive::{CborLen, Decode, Encode};

pub mod mux;
pub mod protocol;
pub(crate) mod traits;
pub(crate) mod typefu;

mod point;
pub use point::Point;
mod tip;
pub use tip::Tip;
mod encoded;
pub use encoded::Encoded;

#[doc(inline)]
pub use mux::mux;

#[repr(u32)]
#[derive(Debug, Encode, Decode, CborLen)]
#[cbor(naked)]
pub enum NetworkMagic {
    #[n(1)]
    Preprod = 1,
    #[n(2)]
    Preview = 2,
    #[n(764824073)]
    Mainnet = 764824073,
}
