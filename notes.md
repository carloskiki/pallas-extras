# Project Wide

- A Dependabot system that checks for upstream updates of slightly modified crates we rework in-tree.

# Ledger

- Switch from `Vec<T>` to `Box<[T]>`, for that we need `mitsein::BoxSlice1: Clone`, which needs `CloneToUninit`/`CloneUnsized`.

We need a "move bytes" trait to move the allocations of some of the types, such as `Output`.

UTxO set validation:
- Need bytes for tx body for hash calculation.
- To realloc outputs when inserted into the utxo set. 

## Do full transactions keep their bytes, or realloc?

Even in conway, most things borrow (except for `plutus::Data`, which may be fixed later),
so worth it keeping the bytes.

## Unresolved questions (mostly interop)

> What is the type for `network::tx_submission` `Transactions` (transaction list)?
> Does it use `Vec<With<Tx>>` or `Vec<Tx>` or something else?

> In general, what are the types used by the network interface to maximize UX and minimize copies and re-encodes?

## tracking bytes - respecting hashes - construction from different perspectives.


### Needed interfaces

1. Encode with no bytes
2. Encode with full bytes
3. Decode with full bytes + extra

#### For Block

- each `transaction::Body` ___needs___ a `With`.
1. Encode normally.
2. Encode wrapped in a Yoke/`With`.
3. `With` + Some structure that stores hashes needed only.

#### For Transaction

1. Encode normally.
2. Encode wrapped in a yoke/with.
3. `With` + transaction hash.

### The fuck it use `With` case


#### For Block

1. You need to pre-encode some stuff, but maybe a bit more efficient.
2. Normal encode.
3. Normal decode.

#### For Transaction

1. You need to pre-encode the tx body, but may be slightly more efficient.
2. Normal encode.
3. Normal decode.


#### Reading blocks from the network (and validating them)

Need the bytes for body commitment to verify (can be done in `Decode`, is this a good idea?)

Choice: either bytes in struct or hash check in decode.

Decision: `With` for header commit/proof components, and check after decode.

#### Creating a block from txs

Need to keep the TX bytes as is, can't only use the hash because we need the CBOR bytes to make a block.

-> Need both the TXBody _bytes_ and the hash (can get hash from bytes).

-> May be slightly more efficient to keep the witness + data bytes for hash compute.

Decision: `Descriptor` like constructor to make a block.
- tx body needs to be `With`, both in block and tx, for witnesses and ids to work.
- in byron, tx witness need `With` because they are used in block commitments (tx structure is used in block, not just body).

IDK: In shelley, `With` are nice to have in tx from a performance perspective, but can be anoying on construction (since we already have
constructors, this is fine).

#### Creating a tx

Don't want to bother serializing the body.

Decision: provide a constructor (`Descriptor`) that serializes and constructs.

### Byron

In concrete for byron: `With` around tx body + `With` around witnesses in payload. `With` around things needed for block commitment check.

# CBOR

- Anonymous error types when `impl Trait` in associated type is stable. When this is possible, functions/modules instead of types
    will be soooo much nicer for attributes.
- should use `u64` correctly instead of `usize` for lengths.

# Network

- Make some clients bufferable by having an async mutex handle that is held since the message was sent.
