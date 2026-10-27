# sbw-kernel

Block 1 of space-between-worlds: the Nucleo (kernel).

Pure, deterministic core: `decide` (state + command → events or refusal) and
`apply` (state + event → new state), with state, effects, the rule trait,
dice, threads and fiction time.

The kernel is pure: no file system, network, clock, environment, processes,
threads or OS randomness; no floats; no `HashMap`/`HashSet` (ordered
structures only). Enforced by `scripts/check-kernel-purity.sh`.

Spec: `docs/specs/spec-01-nucleo.md`. See the repo root `AGENTS.md` for the
development workflow and quality gate.
