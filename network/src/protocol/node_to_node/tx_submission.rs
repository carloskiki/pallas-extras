use state::{Idle, Init, TransactionIds, Transactions};

use crate::{HList, traits::mini_protocol::MiniProtocol};

pub mod message;
pub mod state;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TxSubmission;

impl MiniProtocol for TxSubmission {
    const NUMBER: u16 = 4;

    const READ_BUFFER_SIZE: usize = 20;

    type States = HList![
        Init,
        Idle,
        TransactionIds<true>,
        TransactionIds<false>,
        Transactions
    ];
}
