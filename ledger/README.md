# Ledger Data Types

This crate contains the ledger data types and the logic for encoding and decoding them.

## Design Decisions

- All types own their data, so lists are of type `Box<[T]>`, and maps are of type `Box<[(K, V)]>`.
- Currently types ranging from the Shelley era to the Babbage era are supported. We have no plans of backporting
    to the poorly documented Byron era. 
- We only support encoding into the latest era. Currently, this means that all types encode to the Babbage era.
- A single type is used to represent equivalent types across eras. For example, the `Block` can be decoded from
    any supported era, but it is encoded as a Babbage era block.
