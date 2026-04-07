#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="${1:-$ROOT_DIR/docs/proof/artifacts}"

mkdir -p "$OUT_DIR"

cd "$ROOT_DIR"
cargo build >/dev/null

./target/debug/assistant-runtime runtime proof-metrics >"$OUT_DIR/proof-metrics.json"
./target/debug/assistant-runtime runtime benchmark >"$OUT_DIR/benchmark.json"
./target/debug/assistant-runtime runtime transcript-proof --file examples/transcript-proof.sample.json >"$OUT_DIR/transcript-proof.sample.json"
cargo run --bin assistant-runtime-showcase -- proof >"$OUT_DIR/proof-dashboard.ansi.txt"
cargo run --bin assistant-runtime-showcase -- snapshot --turn 8 >"$OUT_DIR/pipeline-snapshot-turn8.ansi.txt"

printf 'generated proof artifacts in %s\n' "$OUT_DIR"
