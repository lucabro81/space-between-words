//! K4 acceptance test: the kernel, in its compilation, depends on nothing
//! that accesses network, filesystem, clock or system randomness.
//!
//! Observable outcome: every crate in `cargo tree -p sbw-kernel -e normal`
//! is individually approved — the entries of `APPROVED_DEPS` in
//! `scripts/check-kernel-purity.sh`, the enforcement AGENTS.md designates
//! for kernel purity. Each approved crate is known to be pure at runtime:
//! the only approved dependency so far is a compile-time derive macro.
//!
//! This file lives under `tests/` (not `src/`), so `std::process` is allowed
//! here: the purity script scans `crates/sbw-kernel/src/` only, and
//! integration-test targets are never part of the shipped crate.

use std::path::PathBuf;
use std::process::Command;

/// The approved crate names, read from the purity script's `APPROVED_DEPS`
/// block: the single source of truth for what the kernel may depend on.
fn approved_deps() -> Vec<String> {
    let script =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/check-kernel-purity.sh");
    let text = std::fs::read_to_string(&script).expect("purity script readable");
    let marker = "APPROVED_DEPS=(";
    let start = text.find(marker).expect("APPROVED_DEPS block") + marker.len();
    let end = start + text[start..].find(')').expect("closed APPROVED_DEPS block");
    text[start..end]
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed.strip_prefix('\'')?.strip_suffix('\'')
        })
        .map(str::to_string)
        .collect()
}

#[test]
fn k4_kernel_depends_only_on_approved_crates() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let output = Command::new("cargo")
        .args([
            "tree",
            "-p",
            "sbw-kernel",
            "-e",
            "normal",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()
        .expect("cargo tree should run");
    assert!(
        output.status.success(),
        "cargo tree failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Extract crate names: each line names a crate followed by "v<version>",
    // possibly after tree-drawing glyphs.
    let mut crates: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start_matches([' ', '│', '├', '└', '─']);
            let (name, rest) = trimmed.split_once(' ')?;
            let version = rest.strip_prefix('v')?;
            let first = version.chars().next()?;
            first.is_ascii_digit().then(|| name.to_string())
        })
        .collect();
    assert!(
        !crates.is_empty(),
        "no crates parsed from the dependency tree:\n{stdout}"
    );
    crates.sort();
    crates.dedup();

    let approved = approved_deps();
    let unapproved: Vec<&String> = crates.iter().filter(|c| !approved.contains(c)).collect();
    assert!(
        unapproved.is_empty(),
        "the kernel's normal dependency tree contains unapproved crates: {unapproved:?}\n\
         approved: {approved:?}\n\
         full tree:\n{stdout}"
    );
}
