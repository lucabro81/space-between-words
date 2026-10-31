# sbw-eventlog

Block 2 of space-between-worlds: the Registro eventi (event log).

Append-only event log on SQLite (WAL mode, `synchronous=FULL`), deterministic
CBOR (RFC 8949 §4.2) event bodies, SHA-256 hash chain, single-writer file
lock, derived indexes and snapshots in a separate SQLite file. Depends on
`sbw-kernel`.

The log is append-only: the API exposes no operation that modifies or deletes
an event. See the repo root `AGENTS.md` for the development workflow and
quality gate.
