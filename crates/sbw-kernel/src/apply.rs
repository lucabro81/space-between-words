//! `apply`: the kernel's second operation. Given a state and an event, it
//! produces the new state using only the event's effects — never the
//! author's rules. This is what makes replaying the event log independent
//! of rule changes: every effect carries its "before" value, and `apply`
//! verifies each one against the current state before modifying anything.
//!
//! Pure: no I/O, no clock, no randomness, and the rules are never invoked.
//! The input state is never mutated: `apply` works on a clone and returns
//! the modified copy, or an error with nothing modified. Dice outcomes
//! recorded in the event are never re-rolled: they don't reach the state.

use crate::error::ApplyError;
use crate::event::Event;
use crate::state::State;

/// Applies an event's effects, in order, to a state. Returns the new
/// state, or an error that leaves the caller's state untouched. Every
/// "before" value in the effects must match the current state: a
/// mismatch is a reported error, never a silent modification. The rules
/// are never invoked: the effects alone rebuild the state, which is why
/// replaying the log stays correct when the rules change.
pub fn apply(_state: &State, _event: &Event) -> Result<State, ApplyError> {
    unimplemented!("effect application lands with the next commit")
}
