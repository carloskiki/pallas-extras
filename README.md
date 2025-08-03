# TODOs

## Data structures

- Chain (Header only) & block DB on disk
- Environment (UTXO + Certificates + blah) (Probably need to store on disk with LRU cache)

## Components

### Peer manager
- Manages Peer sharing client & server
- Sends new peer handles to the components that require peers.
- Other components register to the peer manager

### DB sync
- Runs the Chain sync & block fetch protocols (clients & servers)
- Manager 

### Tx manager
- Manages the Mempool
- Shares tx with other nodes

### Block producer
- Produces blocks & manages certificates

## Ledger

- Fix types so that we can only construct valid ones (e.g., Set, non-empty set)
- Decoding with Era context

## VRF

- Document the VRF crate (with references to the RFC).
- Implement elliptic-curve traits for ed25519-dalek.
- Implement Elligator2 for ed25519-dalek.
- Implement VRF in ed25519-dalek.

## MUX

- [ ] use async-io instead of futures in mux
- Implement Future when client/server has no agency (instead of receive).
- Implement VRF in RustCrypto

- [ ] Make sure to apply recommendations from https://corrode.dev/blog/pitfalls-of-safe-rust/
    - Check arithmetic overflows
    - Make sure Debug is only implemented on types that are not secret
    - Use constant time where it makes sense
    - Be mindful of Default

## DB

- On disk format: https://github.com/IntersectMBO/ouroboros-consensus/blob/main/ouroboros-consensus/src/ouroboros-consensus/Ouroboros/Consensus/Storage/ImmutableDB/Impl.hs

# Sources

- [Introduction to Elliptic Curve Cryptography](https://math.uchicago.edu/~may/REU2020/REUPapers/Shevchuk.pdf)
- [SEC1](https://www.secg.org/sec1-v2.pdf)
- [Elliptic Curve Wikipedia](https://en.wikipedia.org/wiki/Elliptic_curve)
- [ECC Wikipedia](https://en.wikipedia.org/wiki/Elliptic-curve_cryptography)
- [NIST Recommendations](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-186.pdf)

## Cardano

- [Ledger Specification](https://github.com/IntersectMBO/cardano-ledger)
- [Network Specification](https://ouroboros-network.cardano.intersectmbo.org/pdfs/network-spec/network-spec.pdf)
- [Consensus & Storage Layers](https://ouroboros-consensus.cardano.intersectmbo.org/assets/files/report-25a3c881ef92a4cbb93db7038b7eacf2.pdf)
- [Plutus Specification](https://plutus.cardano.intersectmbo.org/resources/plutus-core-spec.pdf) 
- [Hard fork versions](https://cardano.org/hardforks/)

## Other
- [The UC modeling system](https://eprint.iacr.org/2000/067.pdf)
- [The UC model with Responsive Environments](https://eprint.iacr.org/2016/034.pdf)
- [KES Keys under UC](https://eprint.iacr.org/2007/011.pdf)
- [Tokio wrapper for Spawn](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c63e153f8a0eae7af6f84e7a7f76fb73)
