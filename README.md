# space-between-worlds

sbw is a framework to build and run MUDs (text worlds) and federations of
worlds. This repo currently holds two blocks:

| Block | Crate | What it does |
|---|---|---|
| 1 — Kernel (Nucleo) | `sbw-kernel` | pure deterministic core: `decide`, `apply`, state, effects, the rule interface, dice, threads, fiction time |
| 2 — Event log (Registro eventi) | `sbw-eventlog` | append-only event log on SQLite: deterministic CBOR bodies, SHA-256 hash chain, single-writer file lock, indexes, snapshots |

Code, comments, commit messages, issues and PRs are in English. See
`AGENTS.md` for the development workflow and quality gate.
