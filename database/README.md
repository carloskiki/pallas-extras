# Database

This module implements database functionality of the cardano node. It is the equivalent of
`cardano-db`.

## Things to consider

- TOCTOU: should not be an issue since we acquire a lock on the database.
