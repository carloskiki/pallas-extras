# MMM KES

A partial implementation of the [MMM KES paper](MMM-paper.pdf), relevant for Cardano.

Specifically, this implements the `Sum` construction from the paper with both the
normal `Signature` and the `CompactSignature` variants. This is implemented generically
over any `Signer` and `Verifier`.

## TODOs

- Use `Keypair` instead of `KeypairRef` where possible.
- Make `sum::VerifyingKey` have the summed verifying keys as type parameters (with phantom data).
- Make `sum::Signature` have the summed verifying keys as type parameters instead of signing keys.kla
- For the two above points, need to have a trait that has only the `PERIOD` associated constant for
  verifying keys and/or signatures.
- Rename `sum::Sum` to `sum::SigningKey`.
- Implement `LowerHex` and `UpperHex` where it makes sense.

## Possible Future Work

- `zerocopy` impls.
- Make `Sum` more generic by allowing any seed extending algorithm.
- Have a `CompactSignature` that does not require both sides of the sum to be the same type.
    This could be done at no cost using `union`s (unsafe code), but we would require that the period
    stored in the signature must be exact otherwise we would cause UB.
