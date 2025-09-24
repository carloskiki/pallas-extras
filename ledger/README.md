# Ledger Data Types

An alternative to the `pallas` ledger implementation. The specification is found
[here](https://github.com/IntersectMBO/cardano-ledger).

This crate contains the ledger data types and the logic for encoding and decoding them.


## Things that are not planned on being supported
- Byron era VSS and SSC (They are not considered in validation rules).

## Design Decisions

- All types own their data, so lists are of type `Box<[T]>`, and maps are of type `Box<[(K, V)]>`.
    `pallas` does the same thing as this crate.
- Currently, types ranging from the Shelley era to the Conway era are supported. Plans are made
    to backport to Byron, but this is not a priority.
- A single type is used to represent equivalent types across eras. For example, `Block` can be decoded from
    any supported era, and encoded back into any era as well. This is achieved by providing a `Era` context
    to the `minicbor::Encode` and `minicbor::Decode` implementations.

These decisions were made to (1) minimize the amount of structs to maintain, and (2) make it
easier for users to understand which type to use, as there is only one.

Every types from previous eras translate in some sense to the latest era.

### Pros
- Way less code to maintain.
- Easier to document and understand for new users.
- Ledger analytics is much easier.
- Easier to implement generic algorithms that work across eras.

### Cons
- Have to be careful when validating data, as some fields _must_ be empty or none for some eras.
- Backporting to Byron is difficult.
- Constructing types that are valid for an era which is not the latest one is difficult.
    (Could have types to help with this.)
- More complex `Encode` and `Decode` implementations.
