//! sbw-kernel — block 1: Nucleo
//!
//! Pure deterministic kernel: `decide` (state + command → events or refusal)
//! and `apply` (state + event → new state). No I/O, no clock, no floats,
//! no hash-based collections (ordered structures only).
//!
//! Spec: `docs/specs/spec-01-nucleo.md` (v1.1).
