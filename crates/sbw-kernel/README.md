# sbw-kernel

Block 1 of space-between-worlds: the Nucleo (kernel).

Pure, deterministic core: `decide` (state + command → events, refusal or a
depth-limit error) and `apply` (state + event → new state), with state,
effects, the rule trait, dice, threads and fiction time.

Modules:

- `id` — opaque ID newtypes (player, incarnation, world, element, thread,
  vigil): compare, order and serialize, nothing else
- `time` — wall time, log position, fiction time, dice material
- `command` — the structured request entering the kernel: who, action,
  parameters, wall time, dice material
- `decide` — the response cycle: state + command + rules → events, refusal
  or a reported depth-limit error; stamps every event it produces
- `error` — the structured errors of `decide`
- `meaning` — the open-vocabulary label of an event
- `effect` — the closed vocabulary of state changes, attribute values,
  the fiction-time rate and anchor, deadlines
- `event` — the fact that happened: meaning, effects, who, when
- `refusal` — the structured rejection of a command that makes no sense in
  the current state (code + parameters); never recorded
- `rule` — the rule trait (the four functions the kernel calls into
  author-declared rules), the read-only state view, the execution output
- `state` — the world snapshot: skeleton plus attributes

The kernel is pure: no file system, network, clock, environment, processes,
threads or OS randomness; no floats; no `HashMap`/`HashSet` (ordered
structures only). Enforced by `scripts/check-kernel-purity.sh`.

See the repo root `AGENTS.md` for the development workflow and quality gate.
