# Ledger Data Types

An alternative to the `pallas` ledger implementation. The specification is found
[here](https://github.com/IntersectMBO/cardano-ledger).

This crate contains the ledger data types and the logic for encoding and decoding them.

## tinycbor TODOs

- Make sure that when `decode` errors, the decoder points at the problematic byte, not after it.

## implementation practices

- Use exact types when possible.

- use the namespace to separate words.
- Exceptions like `BoundaryBlock` are fine.

- Every era has all the types it needs, no references across eras.

### While `impl Trait` in a associated type is not stable.
- `Error` types reside in module named as the struct name.
