//! sbw-kernel — block 1: Nucleo
//!
//! Pure deterministic kernel: `decide` (state + command → events, refusal or
//! a depth-limit error) and `apply` (state + event → new state). No I/O, no
//! clock, no floats, no hash-based collections (ordered structures only).

pub mod apply;
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
