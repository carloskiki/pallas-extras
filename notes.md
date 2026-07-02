# Project Wide

- A Dependabot system that checks for upstream updates of slightly modified crates we rework in-tree.

# Ledger

- Switch from `Vec<T>` to `Box<[T]>`, for that we need `mitsein::BoxSlice1: Clone`, which needs `CloneToUninit`/`CloneUnsized`.

We need a "move bytes" trait to move the allocations of some of the types, such as `Output`.

UTxO set validation:
- Need bytes for tx body for hash calculation.
- To realloc outputs when inserted into the utxo set. 

# CBOR

- Anonymous error types when `impl Trait` in associated type is stable.

# Network

- Make some clients bufferable by having an async mutex handle that is held since the message was sent.
