# AGENTS.md — sbw (space-between-worlds)

## Working protocol

The plan for every unit of work lives in its GitHub issue (see Development workflow): the source of truth for scope and decisions, and it outlives the PR. This file covers only stack and conventions; it doesn't duplicate planning. If the scope or a past decision isn't clear from the issues or from the block specs, ask instead of guessing.

## Development workflow

Every unit of work starts as a plan approved in plan mode, becomes a GitHub issue, and ships through its own branch and PR. Nothing goes straight to `main`.

1. **Plan first.** Draft the plan and propose it in plan mode. Nothing else starts, the issue included, until Luca has approved it there: an agreement reached in chat is not an approved plan.
2. **Issue with the approved plan.** Open a GitHub issue before writing code; the approved plan lives inside it. The issue lists the acceptance criteria it covers by ID (see Acceptance criteria).
3. **Branch from the issue**: `gh issue develop <n> --base main --name feat/<n>-<slug>`. Never commit to `main` directly.
4. **TDD on the branch**, atomic commits. Do not push yet.
5. **Review gate, locally, before anything on origin.** Run the review subagents (`cold-reviewer`, `docs-auditor`) given only the issue number, the branch name and the review scope; both must APPROVE while the branch is still local. Their findings and exit conditions are defined in their own files, not here.
6. **Push and PR on approval.** Only after both reviewers APPROVE and Luca has ok'd the operations on origin: push the branch and open a PR with `Closes #N`.
7. **Merge on approval**: squash-merge, sync `main`, delete the branch.
8. **Release on Luca's decision**: after a merge, the release runs on `main` via the `sbw-release` skill — version bump, tag, push, verified locally. One release per issue, supervised by Luca; nothing is published.

Anything decided along the way that is not in the issue's plan (a deferred criterion, a follow-up, a question for a design session) becomes a comment on the issue or a new issue. No tracking files in the repo.

## What it is

sbw is a framework to build and run MUDs (text worlds) and federations of worlds. This repo currently holds **two blocks only**:

| Block | Crate |
|---|---|
| 1 — Kernel (Nucleo) | `sbw-kernel` |
| 2 — Event log (Registro eventi) | `sbw-eventlog` |

The specs and the glossary under `docs/` are the behavioural reference while they exist. They are in Italian; read them as they are. Code, comments, commit messages, issues and PRs are in English.

The specs are written in design sessions outside this repo. **Never edit anything under `docs/`.** If a spec looks wrong, incomplete or contradictory, ask.

## Stack

- Rust stable, edition 2024, version pinned in `rust-toolchain.toml`
- One Cargo workspace, two crates: `sbw-eventlog` depends on `sbw-kernel`; `sbw-kernel` depends on nothing in the workspace
- Event log storage: SQLite, WAL mode, `synchronous=FULL` set explicitly on every open
- Encoding: deterministic CBOR (RFC 8949 §4.2) for event bodies and snapshot state; SHA-256 for hashes
- Rule interface: `wit/rules.wit` plus a Rust trait in `sbw-kernel` that mirrors it one-to-one, kept in sync by hand; no code generation

## Non-negotiable principles

1. **The kernel is pure.** No file system, network, clock, environment, process, threads or OS randomness in `sbw-kernel`. Enforced by `scripts/check-kernel-purity.sh`.
2. **No floats in `sbw-kernel`** (`f32`, `f64`), the fiction-time anchor included: the anchor is state, and state has no floats. The anchor's rate is a ratio of two integers.
3. **No `HashMap` / `HashSet` in `sbw-kernel`.** Ordered structures only (`BTreeMap`, `BTreeSet`, sorted `Vec`) — on purpose stricter than the determinism requirement alone.
4. **Replay never runs rules.** `apply` uses only the effects stored in the event.
5. **The log is append-only.** The `EventLog` API exposes no operation that modifies or deletes an event.
6. **Test rules are test-only.** The rules used by tests (lantern, trap, thread generator, lock-picking, rewind…) are Rust fixtures under `tests/` or `#[cfg(test)]`, never reachable from non-test code. The real rule format is block 8, not in this repo.
7. **Exposure.** Every event the kernel produces has `exposed = false`, and the rule interface offers no way to set it. Who decides exposure is an open design question.
8. **IDs are opaque.** Player, incarnation, world, element, thread and vigil IDs are newtypes; their structure is not designed yet (block 13). Implement compare, order and serialize, nothing else.

## Rust — NEVER

- NEVER: `unwrap()` or `expect()` outside `#[cfg(test)]`
- NEVER: `panic!`, `todo!`, `unreachable!`, `unimplemented!` outside `#[cfg(test)]` — errors are structured types (`thiserror`), not panics
- NEVER: `println!`, `eprintln!`, `dbg!` in `sbw-kernel` — the kernel has no output channel (principle 1); diagnostics belong to the runtime
- NEVER: `unsafe` in `sbw-kernel`
- NEVER: introduce a dependency cycle between crates
- NEVER: commit with clippy warnings — `cargo clippy -- -D warnings` must be clean
- NEVER: commit without rustfmt formatting

## Rust — ALWAYS

- ALWAYS: use `thiserror` for error type definitions
- ALWAYS: use `miette` for CLI-facing error output (when a CLI exists — blocks 1–2 have none, see What NOT to do)
- ALWAYS: prefer `&str` over `String` in function signatures unless ownership is required
- ALWAYS: all public types and functions must be coherent with the relevant block specification

The `thiserror` and `miette` clauses imply future dependencies: their first use still follows the dependency rule under "What NOT to do" (Luca's approval plus the quoted documentation proof), and an approved `sbw-kernel` dependency is also added to `APPROVED_DEPS` in `scripts/check-kernel-purity.sh`.

## Documentation

Shipped documentation stands alone. The specs are design-time documents and may leave the repo one day: nothing that ships with the code may depend on them for its meaning.

- Rustdoc and code comments (test files and `wit/` included) describe the code in its own terms. No references to spec files or spec sections (`spec-01`, `spec-nucleo 5.10`, `§3`, `docs/specs/…`).
- File names follow the same rule: a file's name describes what it contains (`purity.rs`, `single_timeline.rs`), never a design document or its numbering scheme.
- Test names follow it too: a test function's name describes what it asserts. The code is agnostic of the design documents — criterion IDs never appear in it.
- Crate READMEs, the root `README.md` and this file follow the same rule: they describe their subject without pointing into the specs.
- References to external standards (RFC 8949) are fine — they are permanent.
- Criterion IDs belong to the issues, the plans and the PRs that track the work — never to the code: not in file names, not in test names, not in comments. The full text of each criterion is quoted in the issue that covers it.
- The docs-auditor review gate flags any spec reference in the documentation surface of a diff.

## What NOT to do

- Don't build anything outside blocks 1 and 2: no runtime/daemon, SSH, network, LLM, federation, CLI, declarative rule format, reference ruleset, player identity, end-user binary. No `wasmtime`, no `wit-bindgen`. If a criterion seems to need one of these, ask.
- Don't add any dependency (dev-dependencies included) without Luca's approval. When proposing one, give name, version, purpose, link to its docs and **the quoted line of its documentation proving the property needed** (for the CBOR crate: deterministic encoding per RFC 8949 §4.2 and rejection of non-deterministic input; for the PRNG: portability across platforms and versions, no OS entropy pulled into the kernel; for the file lock: released by the OS when the process dies). If no crate proves the property, ask; don't write the property yourself.
- Don't introduce names for game concepts. Code identifiers come from the framework's fixed vocabulary (`Command`, `Event`, `Meaning`, `Effect`, `State`, `Rule`, `Deadline`, `Seed`, `WallTime`, `Position`, `WorldTime`, `decide`, `apply`, `EventLog`; log columns `position`, `group_start`, `format_version`, `exposed`, `body`, `chain_hash`). A technical name not listed there is fine; a name for something a world author would talk about is a question.
- Don't add effect primitives, event fields, state fields, columns or tables the specs don't list. Ask.
- Don't make the gate pass by deleting or weakening a test, `#[ignore]`, `#[allow(...)]` on a clippy lint, editing the purity script, or changing an expected value to match what the code does.
- Don't comply with a request that contradicts a spec, an invariant or this file: say which line it contradicts and stop.

## Acceptance criteria

The framework's acceptance criteria have stable IDs: `K1`…`K22` (kernel) and `L1`…`L27` (event log). The full text of a criterion is quoted in the issue that covers it.

Every criterion an issue covers gets at least one test asserting **the observable outcome written in the criterion**, not a proxy; one criterion per test. Test names describe what they assert — the code carries no criterion IDs (see Documentation). Test first, see it fail for the right reason, then the code.

A criterion that can't be tested as written is decided in the issue's plan (how it's tested, or why it's partial or deferred), never improvised in the code. A deferred criterion gets its own issue.

## Quality gate

From the repo root, all must pass before a PR is opened:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-kernel-purity.sh
```

`scripts/check-kernel-purity.sh` exits non-zero if any file under `crates/sbw-kernel/src/` matches:

```
std::fs  std::net  std::time  std::env  std::process  std::thread
HashMap  HashSet  \bf32\b  \bf64\b  getrandom  thread_rng  OsRng
```

or if `cargo tree -p sbw-kernel -e normal` lists a crate not in the script's approved list (updated only together with an approved dependency).

Never claim the gate passed without its output.

## Repo structure

```
sbw/
├── AGENTS.md
├── README.md
├── Cargo.toml              # workspace
├── rust-toolchain.toml
├── docs/                   # specs and glossary — read-only for agents
│   ├── glossario.md
│   └── specs/
│       ├── spec-01-nucleo.md
│       └── spec-02-registro.md
├── wit/
│   └── rules.wit           # rule interface contract
├── scripts/
│   └── check-kernel-purity.sh
└── crates/
    ├── sbw-kernel/         # block 1: decide, apply, state, effects, rule trait, dice, threads, time
    └── sbw-eventlog/       # block 2: SQLite log, CBOR codec, hash chain, lock, indexes, snapshots
```

## **IMPORTANT**

Never add Co-Authored-By lines to commits
