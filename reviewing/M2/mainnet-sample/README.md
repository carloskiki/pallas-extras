# Mainnet immutable database sample

This directory contains the first ten complete immutable file groups from Cardano mainnet,
`00000` through `00009` inclusive. Each group contains the original `.chunk`, `.primary`, and
`.secondary` files copied byte-for-byte from a fully synchronized node database.

The 30 database files total 173,536,873 bytes and contain 215,977 blocks. The `.primary` files are
included for compatibility with cardano-node and the Haskell `db-analyser`; the Rust `database`
crate reads the `.chunk` and `.secondary` files directly.

Verify the sample from this directory with:

```shell
sha256sum --check SHA256SUMS
```

On macOS, use `shasum -a 256 --check SHA256SUMS` instead. This is an immutable-history sample, not a
complete runnable node database: it intentionally contains no ledger snapshots or volatile database.
