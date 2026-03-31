use crate::{Agency, agency::Client};
use std::convert::Infallible;

/// The initial state of the protocol.
///
/// This stores extra information used to initialize the handles for the protocol.
pub trait InitialState: State {
    /// The ID of the protocol that is initialized in this state.
    const PROTOCOL_ID: u16;
    /// The size of the buffer used to receive messages from the peer.
    ///
    /// If the buffer overflows, the connection will be closed.
    const INGRESS_BUFFER_SIZE: usize;
}

/// A state in a protocol.
pub trait State {
    /// The maximum size of messages that can be received in this state.
    const SIZE_LIMIT: usize;
    /// The maximum amount of time that can elapse waiting for a message in this state.
    const TIMEOUT: std::time::Duration;

    /// The agency in this state.
    type Agency: Agency;
    /// The message received in this state.
    type Message;
}

/// The end state of the protocol.
#[derive(Default)]
pub struct Done;

impl State for Done {
    const SIZE_LIMIT: usize = 0;
    const TIMEOUT: std::time::Duration = std::time::Duration::MAX;

    // This could be either.
    type Agency = Client;
    type Message = Infallible;
}

/// Implements the `State` trait, and generates a `Message` enum with `Contains` and `FromParts`
/// implementations.
macro_rules! state {
    ($name:ident {
        size_limit: $size_limit:expr,
        timeout: $timeout:expr,
        agency: $agency:ty,
        message: [$($message:tt)*]
    }) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name;

        impl $crate::State for $name {
            const SIZE_LIMIT: usize = $size_limit;
            const TIMEOUT: ::std::time::Duration = $timeout;
            type Agency = $agency;
            type Message = Message;
        }

        $crate::state!(@message <$agency as $crate::Agency>::Inverse | $($message)+);
    };
    (@message $agency:ty | $ty:ty) => {
        type Message = (crate::Encoded<$ty>, crate::mux::Handle<$agency, <$ty as $crate::Message>::ToState>);
    };
    (@message $agency:ty | $($message:ident$(<$($args:tt),*>)?),+) => {
        pub enum Message {
            $($message(
                $crate::Encoded<$message$(<$($args),*>)?>,
                $crate::mux::Handle<
                    $agency,
                    <$message$(<$($args),*>)? as $crate::Message>::ToState
                >
            ),)+
        }

        $(
            impl $crate::message::Contains<$message$(<$($args),*>)?> for Message {}
        )+

        impl $crate::message::FromParts<$agency> for Message {
            fn from_parts<S>(
                tag: u64,
                bytes: ::bytes::Bytes,
                handle: $crate::mux::Handle<$agency, S>,
            ) -> Option<Self> {
                match tag {
                    $(
                        <$message$(<$($args),*>)? as $crate::Message>::TAG => Some(Message::$message(
                            $crate::Encoded::new(bytes),
                            handle.transition()
                        )),
                    )+
                    _ => None,
                }
            }
        }
    }
}
pub(crate) use state;
