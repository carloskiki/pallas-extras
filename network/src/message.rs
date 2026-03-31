use crate::State;
use tinycbor_derive::{CborLen, Decode, Encode};

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

pub(crate) type Single<A, M> = (
    crate::Encoded<M>,
    crate::mux::Handle<A, <M as Message>::ToState>,
);
impl<A, M: Message> Contains<M> for Single<A, M> {}
impl<A, M: Message> FromParts<A> for Single<A, M> {
    fn from_parts<S>(
        tag: u64,
        bytes: bytes::Bytes,
        handle: crate::mux::Handle<A, S>,
    ) -> Option<Self> {
        if tag == M::TAG {
            Some((
                crate::Encoded {
                    bytes,
                    _phantom: core::marker::PhantomData,
                },
                handle.transition(),
            ))
        } else {
            None
        }
    }
}
