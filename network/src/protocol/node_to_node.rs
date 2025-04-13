pub mod block_fetch;
pub mod chain_sync;

use chain_sync::ChainSync;

use crate::typefu::coproduct::Coprod;

use super::handshake::{message::NodeToNodeVersionData, Handshake};

pub type NodeToNode = Coprod![Handshake<NodeToNodeVersionData>, ChainSync];
