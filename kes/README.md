## TODOs

- Implement `LowerHex` and `UpperHex` where it makes sense.

## Possible Future Work

- Make `Sum` more generic by allowing any seed extending algorithm.
- Have a `CompactSignature` that does not require both sides of the sum to be the same type.
    This could be done at no cost using `union`s (unsafe code), but we would require that the period
    stored in the signature must be exact otherwise we would cause UB.

## To report

The `SignatureEncoding` trait is shit.
The `SignerMut` trait is also shit.
