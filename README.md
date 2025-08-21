# Pallas Extras

Covering the things pallas does not currently handle.

This is not meant to be an alternative to `pallas`, but rather a place where new components
are developed and experimented with, and then merged into `pallas` once mature enough.

## Current modules

- [`ledger`](ledger): an alternative ledger implementation that focuses on minimizing the amount of structs.
- [`network`](network): an alternative network implementation that represents all protocol state machines at
    the type level.
- `kes`: A fully generic KES implementation based on the MMM paper.
- `bip32`: A BIP32 implementation based on the Ed25519 BIP32 paper.
- `bip39`: A BIP39 implementation that minimizes the amount of dependencies.
