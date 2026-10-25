---
name: sbw-plan
description: Load when starting a new unit of work on the sbw framework — a feature, a bugfix, or a decision. Guides drafting a plan for Luca's approval in plan mode. Use at the very beginning, before any code or issue.
user-invocable: true
---

# sbw Plan — Phase 1: Drafting the Plan

Every unit of work on sbw starts with a written plan, proposed to Luca in
plan mode, and approved by him before any issue is opened or code is touched.

## Your role

Help the user structure a clear, atomic, verifiable plan. Do not write code.
Do not create issues. Do not commit. Your output is a plan document.

## Plan structure

The plan must contain these sections, in this order:

### 1. Title
One line, maximum 80 characters. Describes the outcome, not the implementation.

### 2. Context
Why this work is needed. What is the problem or opportunity? Link to existing
issues if relevant (`#N`). Reference the spec section and acceptance criteria
IDs (`K1`–`K22`, `L1`–`L27`) the work covers.

### 3. What changes
Bullet list of what will be modified, added, or removed. Be specific about
files and repo areas (e.g. `crates/sbw-kernel/src/`, `crates/sbw-eventlog/src/`).

### 4. Technical approach
How it will be implemented. Which existing patterns to follow. Which design
decisions are already made and which remain open.

### 5. Tests
How to verify it works. For TDD: which tests will be written before the code.
Which edge cases to cover. How to run them (`cargo test -p sbw-kernel`,
`cargo clippy --workspace`, etc.). Every acceptance criterion the issue covers
must have at least one test named with its ID in lowercase (`k6_…`, `l14_…`).

### 6. Documentation
What needs updating: crate rustdoc, root `README.md`, `AGENTS.md`.
List every impacted doc file. Specs under `docs/` are read-only — never edit them.

### 7. Risks and notes
What could go wrong. External dependencies (every new dependency needs Luca's
approval with the quoted documentation line proving the required property).
Compatibility. Aspects Luca should know before approving.

## Rules

- **Atomic**: one plan = one PR. If the work breaks into multiple
  independent units, tell Luca and split it.
- **Verifiable**: every test point must be executable and give a binary
  result (pass/fail).
- **No code in the plan**: pseudo-code at most, but prefer descriptions.
- **Language**: the plan is in the same language the user writes in.
  The issue body will be in English (it is the repo's language).

## Output

At the end, present the plan to the user and ask:
> "This is the plan. Do you want to modify it, or propose it to Luca for approval?"

Do not proceed further until Luca approves. An agreement in chat is not an
approved plan.
