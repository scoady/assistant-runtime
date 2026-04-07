#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="${1:-$ROOT_DIR/docs/proof/screenshots}"

mkdir -p "$OUT_DIR"

capture_terminal() {
  local command="$1"
  local output="$2"

  osascript <<EOF >/dev/null
tell application "Terminal"
  activate
  do script "clear; cd \"$ROOT_DIR\" && $command"
  delay 1.0
  set bounds of front window to {40, 80, 1440, 980}
end tell
EOF

  sleep 1
  screencapture -x "$output"
  osascript -e 'tell application "Terminal" to close front window saving no' >/dev/null
}

capture_terminal "cargo run --bin assistant-runtime-showcase -- proof" "$OUT_DIR/proof-dashboard.png"
capture_terminal "cargo run --bin assistant-runtime-showcase -- snapshot --turn 8" "$OUT_DIR/pipeline-snapshot-turn8.png"

printf 'captured screenshots in %s\n' "$OUT_DIR"
