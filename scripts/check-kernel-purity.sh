#!/usr/bin/env bash
# check-kernel-purity.sh — enforce sbw-kernel purity (AGENTS.md, principle 1).
#
# Exits non-zero if any file under crates/sbw-kernel/src/ matches a forbidden
# pattern, or if `cargo tree -p sbw-kernel -e normal` lists a crate not in the
# approved list (updated only together with an approved dependency).

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KERNEL_SRC="$REPO_ROOT/crates/sbw-kernel/src"

# Forbidden patterns in kernel source (AGENTS.md, quality gate).
FORBIDDEN_PATTERNS=(
    'std::fs'
    'std::net'
    'std::time'
    'std::env'
    'std::process'
    'std::thread'
    'HashMap'
    'HashSet'
    '\bf32\b'
    '\bf64\b'
    'getrandom'
    'thread_rng'
    'OsRng'
)

# Approved normal dependencies of sbw-kernel (updated only with an approved dep).
APPROVED_DEPS=(
    'sbw-kernel'
)

fail=0

# --- Check source files for forbidden patterns ---
if [ -d "$KERNEL_SRC" ]; then
    for pattern in "${FORBIDDEN_PATTERNS[@]}"; do
        # shellcheck disable=SC2086
        matches=$(grep -rn -E "$pattern" "$KERNEL_SRC" 2>/dev/null || true)
        if [ -n "$matches" ]; then
            echo "PURITY VIOLATION: pattern '$pattern' found in sbw-kernel source:"
            echo "$matches"
            fail=1
        fi
    done
fi

# --- Check dependency tree ---
TREE_OUTPUT=$(cd "$REPO_ROOT" && cargo tree -p sbw-kernel -e normal 2>/dev/null || true)

if [ -n "$TREE_OUTPUT" ]; then
    # Extract crate names from the tree (lines like "├── crate-name v1.2.3" or "sbw-kernel v0.1.0 (...)")
    # Skip the root line and any "(*)" deduplication markers.
    CRATES=$(echo "$TREE_OUTPUT" | grep -oE '[a-zA-Z0-9_-]+ v[0-9]' | awk '{print $1}' | sort -u || true)

    for crate in $CRATES; do
        approved=false
        for approved_dep in "${APPROVED_DEPS[@]}"; do
            if [ "$crate" = "$approved_dep" ]; then
                approved=true
                break
            fi
        done
        if [ "$approved" = false ]; then
            echo "PURITY VIOLATION: unapproved dependency '$crate' in sbw-kernel."
            echo "  If this dependency was approved by Luca, add it to APPROVED_DEPS in scripts/check-kernel-purity.sh."
            fail=1
        fi
    done
fi

if [ "$fail" -ne 0 ]; then
    echo ""
    echo "Kernel purity check FAILED."
    exit 1
fi

echo "Kernel purity check passed."
