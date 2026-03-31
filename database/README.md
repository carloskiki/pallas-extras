# Database

This module implements database functionality of the cardano node. It is the equivalent of
`cardano-db`.

## Implementation

This implementation is purposefully very simple.

It may be worth implementing the following if proven useful:
1. Caching the most recent chunk.
2. Index file internalization.
3. A more general cache for any chunk.
4. Read ahead.
