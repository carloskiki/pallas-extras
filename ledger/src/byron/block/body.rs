use tinycbor_derive::{CborLen, Decode, Encode};

use crate::byron::{Update, delegation, transaction};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Body {
    transactions: transaction::Payload,
    delegations: Vec<delegation::Certificate>,
    update: Update,
}
