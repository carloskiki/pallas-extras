# Things we could have

A Dependabot system that checks for upstream updates of slightly modified crates we rework in-tree.

# Network

Queries needed:
- Given (protocol #, message ID), what was the agency of the peer, and what is the next ageny?

## message

Enum of messages, each message is its own type.
If encoded with indef array, messages don't need to know their size

Size limits should be declared on the state directly (or the state's message enum).

Bounds for send:

## The enum

This is the only enum that gets generated.

`Message`: Enum (`Lazy<message>`, `handle<message::to_state>`).

- Traits for send:
    - `From<(M, Handle<M::ToState>)>` where M is the message type.

- Traits for receive:
    - `TryFrom<Tag, Bytes>`: Create a message from the tag received and the bytes.

- Traits for task:
    - `AgencyInfo`: Given (message ID) return Option<(agency, next_agency)>.


possibilities:
- Each state also knows its mp. This is not nice because you have to specify mp <-> state twice (in mp and in state).
- The client stores the mp ID at runtime. This is just weird, there should not be a runtime value for something known at compile time.

Both of these weird for the user of the handle to not have it identified for a specific mp.

- The message specifies the mp in the macro to create the message. We ahve to specify message -> mp, which is not nice.

# Buffer pools

A good allocator mostly neglects the benefits of complex buffer pooling, until it has been proven that the workflow
is allocator bound.
