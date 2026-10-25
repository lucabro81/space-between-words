---
name: sbw-release
description: Load after the PR has been squash-merged and main is synced. Runs the release on main — version bump, tags, cargo publish — and verifies the GitHub Release and CI. Supervised by Luca.
user-invocable: true
---

# sbw Release — Phase 6: Post-Merge Release on Main

This skill is used **after** the PR has been squash-merged and `main` is
synced locally. It runs the release on `main` — version bump, tags, publish —
and verifies the downstream CI (GitHub Release, crates.io).

The code changes were developed on the branch (in `sbw-tdd`), reviewed by the
gate, and merged into `main` with the PR.

## Prerequisites

- The PR is merged (squash-merge, issue auto-closed via `Closes #N`)
- You are on `main`: `git checkout main`
- `main` is synced: `git pull`
- The feature branch is deleted: `git branch -d feat/<N>-<slug>`
- Cargo workspace versions are consistent across `Cargo.toml` files

## Steps

### 1. Bump versions

Update the version in both crate manifests and the workspace:

```bash
# Edit Cargo.toml (workspace.package.version) and each crate's Cargo.toml
# to the new version number, following semver.
```

Verify:
```bash
cargo check --workspace   # must compile cleanly with the new version
```

Commit the version bump:
```bash
git add -A && git commit -m "chore(release): bump version to X.Y.Z"
```

### 2. Create the git tag

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
```

### 3. Publish to crates.io

Publish in dependency order (kernel first, then eventlog):

```bash
cargo publish -p sbw-kernel
cargo publish -p sbw-eventlog
```

Note: `cargo publish` requires a clean working tree and a crates.io API token.
Only publish versions not yet on the registry.

### 4. Push with tags

```bash
git push && git push --tags
```

### 5. Verify CI downstream

The push triggers the GitHub Actions workflow on `main`:

- **CI**: runs `cargo fmt --check`, `cargo clippy`, `cargo test`, and the
  kernel purity script on the tagged commit
- **Release**: creates the GitHub Release for the tag

Check the pipeline:
```bash
gh run list --branch main --limit 3
```

### 6. Close the loop

Update project memory with anything worth carrying to a future session, and
bring `README.md` in line if the change affects what it documents.

## Rules

- **One release per issue.** Never batch.
- **Never `cargo publish`** from a feature branch.
- **The release runs on `main`, never on a feature branch.**
- **The release is supervised by Luca.** This skill describes the mechanical
  steps; the decision to release is always his.
- **Specs under `docs/` are never edited by the release process.**

## After the release

The unit of work is complete. Start a new unit with the `sbw-plan` skill.
