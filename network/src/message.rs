use crate::State;
use bytes::Bytes;
use std::marker::PhantomData;
use tinycbor::Decode;
use tinycbor_derive::{CborLen, Decode, Encode};

/// A mini-protocol payload retained in its original CBOR representation.
///
/// Decoding is deferred so protocol values borrowing from the wire bytes remain valid for as long
/// as this value is alive.
pub struct Lazy<T> {
    bytes: Bytes,
    marker: PhantomData<T>,
}

impl<T> Lazy<T> {
    /// Return the unmodified CBOR payload bytes.
    pub fn bytes(&self) -> &Bytes {
        &self.bytes
    }

    /// Decode the payload.
    pub fn decode<'a>(&'a self) -> Result<T, T::Error>
    where
        T: Decode<'a>,
    {
        T::decode(&mut tinycbor::Decoder(&self.bytes))
    }
}

impl<T> From<Bytes> for Lazy<T> {
    fn from(bytes: Bytes) -> Self {
        Self {
            bytes,
            marker: PhantomData,
        }
    }
}

/// Trait implemented by messages that can be sent between peers.
pub trait Message {
    const TAG: u64;

    type ToState: State;
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen,
)]
#[cbor(naked)]
pub struct Done<const TAG: u64>;

impl<const T: u64> crate::Message for Done<T> {
    const TAG: u64 = T;
    type ToState = crate::state::Done;
}

/// Trait implemented by message enums that contain `M`.
pub trait Contains<M> {}

pub trait FromParts<A>: Sized {
    fn from_parts<S>(
        tag: u64,
        bytes: bytes::Bytes,
        handle: crate::mux::Handle<A, S>,
    ) -> Option<Self>;
}

pub(crate) type Single<A, M> = (Lazy<M>, crate::mux::Handle<A, <M as Message>::ToState>);
impl<A, M: Message> Contains<M> for Single<A, M> {}
impl<A, M: Message> FromParts<A> for Single<A, M> {
    fn from_parts<S>(
        tag: u64,
        bytes: bytes::Bytes,
        handle: crate::mux::Handle<A, S>,
    ) -> Option<Self> {
        if tag == M::TAG {
            Some((Lazy::from(bytes), handle.transition()))
        } else {
            None
        }
    }
}
