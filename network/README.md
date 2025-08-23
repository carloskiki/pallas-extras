# Network

Implementation of the
[Ouroboros network specification](https://ouroboros-network.cardano.intersectmbo.org/pdfs/network-spec/network-spec.pdf)
with a focus on type representation of the protocol state machines.

This type security means that it is impossible for a user of this crate to send a message without
having the agency, and it is impossible to await a message while having the agency.
The whole crate is structured around the `mux` function, that spawns an actor into
the provided `Spawner` (runtime agnostic) and returns handles to all miniprotocol instances
in the chosen protocol.
