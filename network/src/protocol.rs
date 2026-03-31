use crate::{
    mux::{Egress, Handle, header::ProtocolNumber},
    state::InitialState,
    agency::{Client, Server},
};
use tokio::sync::mpsc;

pub trait Protocol {
    /// Handles returned upon initialization.
    type Handles;
    /// State used by the task to track each protocol.
    type State;

    /// Initialize the protocol, returning the handles and initial state.
    fn initialize(sender: mpsc::Sender<Egress>) -> (Self::Handles, Self::State);

    /// Obtain the state for the given protocol, if it exists.
    fn get_state(
        protocol: ProtocolNumber,
        state: &mut Self::State,
    ) -> Option<&mut crate::mux::task::State>;
}

macro_rules! protocol {
    (@replace $t:tt => $sub:path) => {
        $sub
    };

    ($($T:ident),+) => {
        #[allow(non_snake_case)]
        impl<$($T: InitialState),+> Protocol for ($($T,)+) {
            type Handles = ($((Handle<Client, $T>, Handle<Server, $T>)),+);

            type State = ($(protocol!(@replace $T => crate::mux::task::State)),+);

            fn initialize(sender: mpsc::Sender<Egress>) -> (Self::Handles, Self::State) {
                let ($($T),+) = ($(crate::mux::handle::components::<$T>(sender.clone())),+);
                (($(($T.0, $T.1),)+), ($( $T.2,)+))
            }

            fn get_state(protocol: ProtocolNumber, ($($T),+): &mut Self::State) -> Option<&mut crate::mux::task::State> {
                match protocol.number() {
                    $(
                        x if x == $T::PROTOCOL_ID => Some($T),
                    )+
                    _ => None,
                }
            }
        }
    }
}

protocol!(A, B);
protocol!(A, B, C);
protocol!(A, B, C, D);
protocol!(A, B, C, D, E);
protocol!(A, B, C, D, E, F);
