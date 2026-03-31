//! Implementation of the [network specification][net-spec] for the Cardano protocol.
//!
//! [net-spec]: https://ouroboros-network.cardano.intersectmbo.org/pdfs/network-spec/network-spec.pdf

use tinycbor_derive::{CborLen, Decode, Encode};

pub mod agency;
pub use agency::Agency;

mod encoded;
pub use encoded::Encoded;

pub mod handshake;

mod message;
pub use message::Message;

pub mod mux;

pub mod node_to_client;
pub mod node_to_node;

mod protocol;
pub use protocol::Protocol;

mod point;
pub use point::Point;

pub mod state;
pub use state::State;
pub(crate) use state::state;

mod tip;
pub use tip::Tip;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub enum NetworkMagic {
    #[n(1)]
    Preprod = 1,
    #[n(2)]
    Preview = 2,
    #[n(764824073)]
    Mainnet = 764824073,
}
