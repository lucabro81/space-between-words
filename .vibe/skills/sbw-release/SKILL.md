---
name: sbw-release
description: Load after the PR has been squash-merged and main is synced. Runs the release on main — version bump, tag, push — and verifies locally with the quality gate. Supervised by Luca.
user-invocable: true
---

# sbw Release — Phase 6: Post-Merge Release on Main

This skill is used **after** the PR has been squash-merged and `main` is
synced locally. It runs the release on `main`: version bump, tag, push,
verified locally with the quality gate.

The code changes were developed on the branch (in `sbw-tdd`), reviewed by the
gate, and merged into `main` with the PR.

## Not part of this process

Publishing to crates.io and a GitHub Actions pipeline are **not** part of
sbw's release process: this repo has no CI configuration, and Luca has
decided not to publish the crates. Nothing leaves the repo in a release.
Adding either is Luca's future decision and would go through its own
issue; this skill must not be edited to reintroduce them without one.

## Prerequisites

- The PR is merged (squash-merge, issue auto-closed via `Closes #N`)
- You are on `main`: `git checkout main`
- `main` is synced: `git pull`
- The feature branch is deleted: `git branch -d feat/<N>-<slug>`
- Luca has chosen the version number to release

## Steps

### 1. Bump versions

Update the version in **both crate manifests** —
`crates/sbw-kernel/Cargo.toml` and `crates/sbw-eventlog/Cargo.toml` —
following semver. The workspace root `Cargo.toml` has no `version` field:
there is nothing to bump there.

Verify the tree compiles, then commit the version bump:

```bash
cargo check --workspace    # must compile cleanly with the new version
git add -A && git commit -m "chore(release): bump version to X.Y.Z"
```

### 2. Verify the release commit

Run the full quality gate from the repo root on the release commit:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-kernel-purity.sh
```

All four must pass before the tag is created. If anything fails, stop and
bring it to Luca: do not re-tag or move anything without his decision.

### 3. Create the git tag

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
```

### 4. Push with the tag

```bash
git push && git push --tags
```

### 5. Close the loop

Update project memory with anything worth carrying to a future session, and
bring `README.md` in line if the change affects what it documents.

## Rules

- **One release per issue.** Never batch.
- **The release runs on `main`, never on a feature branch.**
- **The release is supervised by Luca.** This skill describes the mechanical
  steps; the decision to release — and the version number — are always his.
- **Specs under `docs/` are never edited by the release process.**
- **Nothing leaves the repo.** The deliverable of a release is the git tag
  on `main`; see "Not part of this process" above.

## After the release

The unit of work is complete. Start a new unit with the `sbw-plan` skill.
