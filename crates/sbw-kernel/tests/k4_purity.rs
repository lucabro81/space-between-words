//! K4 acceptance test (spec-01 §10, criterion 4): the kernel, in its
//! compilation, depends on nothing that accesses network, filesystem, clock
//! or system randomness.
//!
//! Observable outcome: `cargo tree -p sbw-kernel -e normal` lists no
//! dependency crate — only `sbw-kernel` itself. A crate with zero normal
//! dependencies cannot pull in anything that touches I/O or OS randomness.
//!
//! This file lives under `tests/` (not `src/`), so `std::process` is allowed
//! here: `scripts/check-kernel-purity.sh` guards `src/` only, and test-only
//! code may use the standard library freely (AGENTS.md principle 6).

use std::path::PathBuf;
use std::process::Command;

#[test]
fn k4_kernel_compiles_without_io_dependencies() {
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
    let crates: Vec<&str> = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    assert_eq!(
        crates.len(),
        1,
        "the kernel must have zero normal dependencies, got:\n{stdout}"
    );
    assert!(
        crates[0].starts_with("sbw-kernel v"),
        "unexpected crate in the dependency tree: {}",
        crates[0]
    );
}
