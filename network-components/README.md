# Network components

Composable Tokio actors for Cardano node-to-node networking:

- `PeerManager` maintains handshaken peers, peer sharing, and optional keep-alive.
- `ChainSynchronizer` validates a selected in-memory header chain and broadcasts roll-forward and
  rollback events.
- `BlockFetcher` requests chain-selected ranges, verifies them against synchronized headers, caches
  full blocks, and serves them to downstream callers.

The actors depend only on the transport traits in `transport`. `cardano` implements those traits
with the workspace `network` crate and Ouroboros mini-protocol version 14. `simulation` provides a
deterministic in-memory transport with message-drop and connection-loss injection.

The pinned interoperability target is Cardano Node `11.0.1`. Exact Rust dependency revisions are
recorded in the workspace `Cargo.lock`.

Run the focused tests and quality gates with:

```shell
cargo test --locked -p network -p network-components
cargo fmt -p network -p network-components -- --check
cargo clippy --locked -p network -p network-components --all-targets --no-deps -- -D warnings
```

Generate public API documentation with:

```shell
cargo doc --locked -p network-components --no-deps --open
```

Run the local acceptance simulation and write its report with:

```shell
cargo run --locked --release -p network-components --example simulation_report -- \
  reviewing/M3/simulation-report.txt
```

Run all three actors together against the deterministic 20,000-block network. The example writes
length-prefixed blocks, shuts the actors down, rereads the file, and verifies every block against
the captured selected header chain:

```shell
cargo run --locked --release -p network-components --example partial_node -- \
  /tmp/pallas-m3-partial-node.blocks
```

The equivalent preprod command accepts a raw node-to-node relay and a duration in seconds:

```shell
cargo run --locked --release -p network-components --example partial_node -- \
  --preprod preprod-node.world.dev.cardano.org:3001 \
  /tmp/pallas-m3-preprod.blocks 1800
```

To start from a recent reviewer-supplied point, append its decimal slot and 64-character block hash:

```shell
cargo run --locked --release -p network-components --example partial_node -- \
  --preprod preprod-node.world.dev.cardano.org:3001 \
  /tmp/pallas-m3-preprod.blocks 1800 SLOT BLOCK_HASH
```

The preprod profile advertises initiator-only diffusion and disables Peer Sharing, as required by
relays that negotiate that feature off. Peer discovery and arbitrary transport failures remain
fully exercised by the deterministic transport.
