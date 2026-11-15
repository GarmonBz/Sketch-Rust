#!/usr/bin/env bash
# tools/e2e.sh — runs the sketch-app unit and headless e2e test suite.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1
export PATH="$HOME/.cargo/bin:$PATH"
export RUST_TEST_THREADS="${RUST_TEST_THREADS:-1}"

echo "== cargo test -p sketch-app (unit + e2e)"
cargo test -p sketch-app -- --test-threads="$RUST_TEST_THREADS"
