# TODOs

- Full mux
- Complete ledger implementation
- Standardize ledger types - Should we use Box<[]> or Cow<[]>? - I think Box<[]> because we need the types to be
    'static when sending them to the network or doing async stuff so we don't get a lot of value from Cow. Also,
    data types coming from the network are never leaked into memory.
- Use newtypes instead of numeric types for the ledger.
- Implement VRF in RustCrypto

- [ ] Make sure to apply recommendations from https://corrode.dev/blog/pitfalls-of-safe-rust/
    - Check arithmetic overflows
    - Make sure Debug is only implemented on types that are not secret
    - Use constant time where it makes sense
    - Be mindful of Default

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

## Other
- [The UC modeling system](https://eprint.iacr.org/2000/067.pdf)
- [The UC model with Responsive Environments](https://eprint.iacr.org/2016/034.pdf)
- [KES Keys under UC](https://eprint.iacr.org/2007/011.pdf)
- [Tokio wrapper for Spawn](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c63e153f8a0eae7af6f84e7a7f76fb73)
