use tinycbor::Any;
use tinycbor_derive::{CborLen, Decode, Encode};

use crate::byron::{Update, delegation, transaction};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Body<'a> {
    transactions: Vec<transaction::Payload<'a>>,
    ssc: Any<'a>,
    delegations: Vec<delegation::Certificate<'a>>,
    update: Update<'a>,
}
