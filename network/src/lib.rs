pub mod mux;
pub mod protocol;
pub mod traits;
pub(crate) mod typefu;

use minicbor::{Decode, Encode};

#[repr(u32)]
#[derive(Debug, Encode, Decode)]
#[cbor(index_only)]
pub enum NetworkMagic {
    #[n(1)]
    Preprod = 1,
    #[n(2)]
    Preview = 2,
    #[n(764824073)]
    Mainnet = 764824073,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Point {
    Genesis,
    Block { slot: u64, hash: [u8; 32] },
}

impl<C> Encode<C> for Point {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        if let Point::Block { slot, hash } = self {
            e.array(2)?;
            e.u64(*slot)?;
            e.bytes(hash).map(|_| ())
        } else {
            e.array(0).map(|_| ())
        }
    }
}

impl<C> Decode<'_, C> for Point {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let len = d.array()?;
        if len == Some(0) {
            Ok(Point::Genesis)
        } else if len.is_none_or(|len| len == 2) {
            let slot = d.u64()?;
            let hash: minicbor::bytes::ByteArray<32> = d.decode()?;
            if len.is_none() {
                let ty = d.datatype()?;
                if ty != minicbor::data::Type::Break {
                    return Err(minicbor::decode::Error::type_mismatch(ty));
                }
                d.skip()?;
            }
            Ok(Point::Block {
                slot,
                hash: hash.into(),
            })
        } else {
            Err(minicbor::decode::Error::message("invalid array size"))
        }
    }
}

impl From<Tip> for Point {
    fn from(value: Tip) -> Self {
        match value {
            Tip::Genesis => Point::Genesis,
            Tip::Block { slot, hash, .. } => Point::Block { slot, hash },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tip {
    Genesis,
    Block {
        slot: u64,
        hash: [u8; 32],
        block_number: u64,
    },
}


impl<C> Encode<C> for Tip {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(2)?;
        let block_no = match self {
            Tip::Genesis => 0,
            Tip::Block { block_number, .. } => *block_number,
        };
        e.encode(Point::from(*self))?;
        e.u64(block_no).map(|_| ())
    }
}

impl<C> Decode<'_, C> for Tip {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let arr_len = d.array()?;
        if arr_len.is_some_and(|len| len != 2) {
            return Err(minicbor::decode::Error::message("invalid array size"));
        }
        let point = Point::decode(d, &mut ())?;
        let block_number = d.u64()?;
        if arr_len.is_none() {
            let ty = d.datatype()?;
            if  ty != minicbor::data::Type::Break {
                return Err(minicbor::decode::Error::type_mismatch(ty));
            }
            d.skip()?;
        }
        Ok(match point {
            Point::Genesis => Tip::Genesis,
            Point::Block { slot, hash } => Tip::Block {
                slot,
                hash,
                block_number,
            },
        })
    }
}
