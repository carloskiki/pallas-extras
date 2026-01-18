use crate::shelley::{Credential, transaction::Coin};
use displaydoc::Display;
use thiserror::Error;
use tinycbor::{
    CborLen, Decode, Encode, Encoder,
    container::{self, map},
    primitive,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Target<'a> {
    Other(Coin),
    // TODO: This should be `DeltaCoin` instead of `Coin` which allows negative amounts. Since this
    // is no longer part of the ledger in `conway`, check if DeltaCoin is truly needed, or if
    // positive amounts suffice.
    Accounts(Vec<(Credential<'a>, Coin)>),
}

#[derive(Debug, Display, Error)]
pub enum Error {
    /// error decoding `Other` variant
    Other(#[from] primitive::Error),
    /// error decoding `Accounts` variant
    Accounts(
        #[from]
        container::Error<
            map::Error<
                <Credential<'static> as Decode<'static>>::Error,
                <Coin as Decode<'static>>::Error,
            >,
        >,
    ),
}

impl<'a, 'b: 'a> Decode<'b> for Target<'a> {
    type Error = Error;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        Ok(
            if d.datatype().map_err(|e| Error::Other(e.into()))? == tinycbor::Type::Int {
                Target::Other(Decode::decode(d).map_err(Error::Other)?)
            } else {
                Target::Accounts(Decode::decode(d).map_err(Error::Accounts)?)
            },
        )
    }
}

impl Encode for Target<'_> {
    fn encode<W: tinycbor::Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        match self {
            Target::Other(coin) => coin.encode(e),
            Target::Accounts(accounts) => accounts.encode(e),
        }
    }
}

impl CborLen for Target<'_> {
    fn cbor_len(&self) -> usize {
        match self {
            Target::Other(coin) => coin.cbor_len(),
            Target::Accounts(accounts) => accounts.cbor_len(),
        }
    }
}
