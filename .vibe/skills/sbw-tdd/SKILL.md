---
name: sbw-tdd
description: Load when working on an sbw branch (feat/<N>-<slug>) after the issue has been created. Enforces TDD — write the failing test first, then implement, then refactor. Atomic commits, one PR per issue.
user-invocable: true
---

# sbw TDD — Phase 4: Development on the Branch

This skill guides development work on branch `feat/<N>-<slug>`, applying
TDD (Test-Driven Development) and atomic commits.

## Prerequisites

- You are on branch `feat/<N>-<slug>` (verify with `git branch --show-current`)
- Issue `#N` exists and contains the approved plan
- The working tree is clean or contains only intentional changes

## TDD cycle

For each unit of work in the plan:

### 1. RED — Write the failing test

- Identify the impacted crate (e.g. `crates/sbw-kernel`, `crates/sbw-eventlog`)
- Write the test in the corresponding test file (co-located with the code:
  `src/foo.rs` → `src/foo.rs` `#[cfg(test)]` module, or `tests/` for integration tests)
- The test must:
  - Describe the expected behavior specifically
  - Cover edge cases and limit cases, not just the happy path
  - Use the standard Rust test harness (`#[test]`)
  - Be named after what it asserts — never after a criterion ID: the code
    is agnostic of the design documents; criterion IDs live in the issue
- Run the test and **verify it fails** for the right reason:
  ```bash
  cargo test -p <crate-name> <test_name>
  ```
- Commit the test:
  ```bash
  git add <test-file>
  git commit -m "test(<scope>): <test description>"
  ```

### 2. GREEN — Implement the minimum to pass the test

- Write the minimum code needed to make the test pass
- Do not optimize, do not add unrequested features
- Run the test: it must pass
- Run all tests in the crate to verify nothing broke:
  ```bash
  cargo test -p <crate-name>
  ```
- Commit the implementation:
  ```bash
  git add <source-files>
  git commit -m "feat(<scope>): <implementation description>"
  ```

### 3. REFACTOR — Improve the code

- Refactor while keeping tests green
- Remove duplication, improve names, simplify
- Run tests and clippy after each change
- Commit the refactor (if significant):
  ```bash
  git commit -m "refactor(<scope>): <description>"
  ```

## Commit rules

- **Atomic**: one commit = one logical change. Do not mix test and
  implementation in the same commit (unless it's a minimal refactor).
- **Message**: follows Conventional Commits: `type(scope): description`
  - `type`: `feat`, `fix`, `test`, `refactor`, `docs`, `chore`
  - `scope`: the crate or area (e.g. `kernel`, `eventlog`)
- **Never commit on main**: always verify the current branch
  (`git branch --show-current`) before committing.
- **No Co-Authored-By lines** in commits.

## Before every push

Run the full quality gate from the repo root:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-kernel-purity.sh
```

All four must pass before pushing.

## When you are done

- All tests pass
- `cargo clippy` is clean with `-D warnings`
- `cargo fmt --check` passes
- The kernel purity script passes
- Every commit is atomic with a meaningful message
- The branch is ready for the review gate

Load the `sbw-review` skill to proceed.

## For bugfixes

Bugfixes also follow TDD:
1. Write a test that reproduces the bug (fails)
2. Fix the bug (test passes)
3. Verify no regressions
