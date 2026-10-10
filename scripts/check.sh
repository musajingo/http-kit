#!/usr/bin/env bash

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

command -v jq >/dev/null || { echo "jq is required" >&2; exit 1; }

run() {
  echo "==> $*"
  "$@"
}

if [[ $# -gt 1 ]]; then
  echo "Usage: $0 [--check]" >&2
  exit 2
fi
case "${1:-}" in
  "") run cargo +stable fmt --all ;;
  --check) run cargo +stable fmt --all -- --check ;;
  *) echo "Usage: $0 [--check]" >&2; exit 2 ;;
esac

run cargo +stable clippy --workspace --all-features --all-targets --locked -- -D warnings
run cargo +stable test --workspace --all-features --locked
run cargo +stable test -p http-kit --locked
run cargo +stable test -p http-kit-macros-tests --locked
run cargo +stable test -p http-kit-macros-tests --doc --locked -- --ignored
run env RUSTDOCFLAGS="-D warnings" cargo +stable doc --workspace --all-features --no-deps --locked

# Compile each feature in isolation, plus no features. List read from Cargo.toml.
features=$(cargo +stable metadata --format-version 1 --no-deps |
  jq -r '.packages[] | select(.name == "http-kit").features | keys[] | select(. != "default")')
for feature in "" $features; do
  run cargo +stable check -p http-kit --no-default-features ${feature:+--features "$feature"} --locked
done

echo "All checks passed."
