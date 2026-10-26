//! sbw-eventlog — block 2: Registro eventi
//!
//! Append-only event log: SQLite (WAL, `synchronous=FULL`), deterministic CBOR
//! bodies, SHA-256 hash chain, single-writer file lock, derived indexes,
//! snapshots in a separate SQLite file.
//!
//! Spec: `docs/specs/spec-02-registro.md` (v1.1).
