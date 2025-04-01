pub mod handshake;
pub mod node_to_client;
pub mod node_to_node;

pub use node_to_client::NodeToClient;
pub use node_to_node::NodeToNode;

// You have a client/server in a given state. You send a message, you end up in a new state.
// To receive a message, you typemap over the message coproduct to generate a
// coproduct of the new states.
//
// You are able to clone clients & servers, so the receiver thread needs to know which message goes
// to which instance.
//
// When receiving a message:
// If the associated state of the message you just received is the same as the one that was sent by
// the message, then keep your receiving spot in the queue. Otherwise, hand your spot to the next
// one in line.
