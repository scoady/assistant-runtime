#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  ./scripts/install-and-activate-runtime.sh [target-repo]

Behavior:
  - builds the assistant-runtime binary
  - packages the runtime bundle
  - installs it into <target-repo>/.assistant-runtime
  - writes <target-repo>/.assistant-runtime/activate.sh

Defaults:
  - target-repo defaults to the current working directory
EOF
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_REPO="$(cd "$SCRIPT_DIR/.." && pwd)"
TARGET_REPO="${1:-$(pwd)}"

if [[ ! -d "$TARGET_REPO" ]]; then
  echo "target repo does not exist: $TARGET_REPO" >&2
  exit 1
fi

TARGET_REPO="$(cd "$TARGET_REPO" && pwd)"
PACKAGE_DIR="$(mktemp -d "${TMPDIR:-/tmp}/assistant-runtime-package.XXXXXX")"
INSTALL_ROOT="$TARGET_REPO/.assistant-runtime"
ACTIVATE_PATH="$INSTALL_ROOT/activate.sh"

cleanup() {
  rm -rf "$PACKAGE_DIR"
}
trap cleanup EXIT

echo "[assistant-runtime] building binary"
cargo build --bin assistant-runtime --manifest-path "$SOURCE_REPO/Cargo.toml"

RUNTIME_BIN="$SOURCE_REPO/target/debug/assistant-runtime"
if [[ ! -x "$RUNTIME_BIN" ]]; then
  echo "missing built runtime binary: $RUNTIME_BIN" >&2
  exit 1
fi

echo "[assistant-runtime] packaging runtime bundle"
"$RUNTIME_BIN" runtime package --output "$PACKAGE_DIR"

echo "[assistant-runtime] installing into $TARGET_REPO"
"$PACKAGE_DIR/install.sh" "$TARGET_REPO"

mkdir -p "$INSTALL_ROOT"
cat > "$ACTIVATE_PATH" <<EOF
#!/usr/bin/env bash
set -euo pipefail

ASSISTANT_RUNTIME_ROOT="$INSTALL_ROOT"
export ASSISTANT_RUNTIME_ROOT
export PATH="\$ASSISTANT_RUNTIME_ROOT/bin:\$PATH"

cat <<'ACTIVATED'
assistant.runtime activated for:
  $TARGET_REPO

Commands now available on PATH for this shell:
  assistant-runtime
  assistant-loop-runtime
  assistant-conversation-runtime
  assistant-host-runtime
  assistant-os-runtime

Run these from the target repository root so the runtime uses:
  $TARGET_REPO/.runtime
ACTIVATED
EOF
chmod 755 "$ACTIVATE_PATH"

echo
echo "[assistant-runtime] activation script written to $ACTIVATE_PATH"
echo "[assistant-runtime] next steps:"
echo "  cd \"$TARGET_REPO\""
echo "  source ./.assistant-runtime/activate.sh"
echo "  assistant-runtime runtime manifest"
