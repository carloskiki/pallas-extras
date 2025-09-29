# minicbor-derive

A companion crate to [`minicbor`][1] to allow deriving `minicbor::Encode`
and `minicbor::Decode` traits.

This is a fork of twittner's `minicbor-derive` crate, adapted to make deriving
`Encode` and `Decode` for ledger types easier.

## TODO's

- [ ] Change context parameter `C` to not collide with user-defined type parameters (something like `__C`).

## Breaking Changes

Breaking changes from the original `minicbor-derive` crate:
- Removed the `transparent` attribute. Implement manually if desired.
- There is no notion of "borrow". Things are borrowed when their decode implementation relies on the lifetime parameter.
- no `skip`, no `default`
- `skip` only skips the field when encoding. When decoding, the field is decoded as usual. Use `default` to skip
    decoding and use the default value instead.

[1]: https://crates.io/crates/minicbor
