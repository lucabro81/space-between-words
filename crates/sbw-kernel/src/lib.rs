//! sbw-kernel — block 1: Nucleo
//!
//! Pure deterministic kernel: `decide` (state + command → events or refusal)
//! and `apply` (state + event → new state). No I/O, no clock, no floats,
//! no hash-based collections (ordered structures only).

pub mod command;
pub mod decide;
pub mod effect;
pub mod error;
pub mod event;
pub mod id;
pub mod meaning;
pub mod refusal;
pub mod rule;
pub mod state;
pub mod time;
