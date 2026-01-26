use crate::shelley::transaction::{Coin, Input};
use displaydoc::Display;
use thiserror::Error;
use tinycbor::{container::bounded, *};

pub mod option;
pub use option::Options;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Body<'a> {
    pub inputs: Vec<Input<'a>>,
    pub outputs: Vec<super::output::Output<'a>>,
    pub fee: Coin,
    pub options: Options<'a>,
}

#[derive(Debug, Display, Error)]
#[prefix_enum_doc_attributes]
/// while decoding `Transaction`
pub enum Error {
    /// in field `inputs`
    Inputs(#[from] container::Error<<Input<'static> as Decode<'static>>::Error>),
    /// in field `outputs`
    Outputs(#[from] container::Error<<super::output::Output<'static> as Decode<'static>>::Error>),
    /// in field `fee`
    Fee(#[from] primitive::Error),
    /// in field `options`
    Options(#[from] <option::Option<'static> as Decode<'static>>::Error),
}

impl Encode for Body<'_> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        e.map(3 + self.options.as_ref().len())?;
        0.encode(e)?;
        self.inputs.encode(e)?;
        1.encode(e)?;
        self.outputs.encode(e)?;
        2.encode(e)?;
        self.fee.encode(e)?;
        self.options.as_ref().encode(e)
    }
}

impl CborLen for Body<'_> {
    fn cbor_len(&self) -> usize {
        let map_len = 3 + self.options.as_ref().len();
        map_len.cbor_len()
            + 0.cbor_len()
            + self.inputs.cbor_len()
            + 1.cbor_len()
            + self.outputs.cbor_len()
            + 2.cbor_len()
            + self.fee.cbor_len()
            + self.options.as_ref().cbor_len()
    }
}

impl<'a, 'b: 'a> Decode<'b> for Body<'a> {
    type Error = container::Error<bounded::Error<Error>>;

    fn decode(d: &mut Decoder<'b>) -> Result<Self, Self::Error> {
        fn wrap(e: impl Into<Error>) -> container::Error<bounded::Error<Error>> {
            container::Error::Content(bounded::Error::Content(e.into()))
        }

        let mut inputs = None;
        let mut outputs = None;
        let mut fee = None;
        let mut options = Options::default();

        let mut decode_elem = |d: &mut Decoder<'b>| -> Result<(), Self::Error> {
            let pre_key = *d;
            let key: u64 = Decode::decode(d)?;
            match key {
                0 if inputs.is_none() => {
                    inputs = Some(Decode::decode(d).map_err(wrap)?);
                }
                1 if outputs.is_none() => {
                    outputs = Some(Decode::decode(d).map_err(wrap)?);
                }
                2 if fee.is_none() => {
                    fee = Some(Decode::decode(d).map_err(wrap)?);
                }
                0..=2 => {
                    return Err(bounded::Error::Surplus.into());
                }
                _ => {
                    *d = pre_key;
                    let option: option::Option<'a> = Decode::decode(d).map_err(wrap)?;
                    if !options.insert(option) {
                        return Err(bounded::Error::Surplus.into());
                    }
                }
            }
            Ok(())
        };

        if let Some(len) = d.map_visitor()?.remaining() {
            for _ in 0..len {
                decode_elem(d)?;
            }
        } else {
            while d.datatype()? != tinycbor::Type::Break {
                decode_elem(d)?;
            }
            d.next().expect("found break").expect("valid break");
        }

        Ok(Body {
            inputs: inputs.ok_or(bounded::Error::Missing)?,
            outputs: outputs.ok_or(bounded::Error::Missing)?,
            fee: fee.ok_or(bounded::Error::Missing)?,
            options,
        })
    }
}
