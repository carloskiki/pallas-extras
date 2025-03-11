## TODOs

- Implement `LowerHex` and `UpperHex` where it makes sense.

## Future Work

- Check if with the new trait solver the infinite recursion problem is solved.
- Make `Sum` more generic by allowing any seed extending algorithm.
- Have a `CompactSignature` that does not require both sides of the sum to be the same type.

## To report

The `SignatureEncoding` trait is shit.
The `SignerMut` trait is also shit.

## Design Decisions

- For `CompactSignature`, we need trait implementations to have two `impl` blocks because there is a base case
    that we need to cover. Namely, when `CompactSignature<CompactSignature<S, KP>, KP>` versus when
    `CompactSignature<CompactSignature<S, KP>, Double<KP, H>>`.
- We need the `VerifyingKey` of `Sum` to have the `L` and `R` type parameters because verifying 
