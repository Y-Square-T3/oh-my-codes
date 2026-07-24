#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-}"
DEST="${2:-}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_DIR="$(dirname "$SCRIPT_DIR")"

derive_dest() {
  case "$1" in
    aarch64-apple-darwin)     echo "darwin-arm64" ;;
    x86_64-apple-darwin)      echo "darwin-x64" ;;
    aarch64-unknown-linux-gnu) echo "linux-arm64" ;;
    x86_64-unknown-linux-gnu)  echo "linux-x64" ;;
    x86_64-pc-windows-msvc)   echo "win32-x64" ;;
    *) echo "" ;;
  esac
}

if [[ -z "$TARGET" ]]; then
  ARCH=$(uname -m)
  if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    PLAT="win32"
    ARCH="x64"
  else
      PLAT="$(node -e 'console.log(process.platform)')"
  fi

  case "${PLAT}/${ARCH}" in
    darwin/arm64*|darwin/aarch64)  TARGET="aarch64-apple-darwin";     ;;
    darwin/x86_64|darwin/x64)      TARGET="x86_64-apple-darwin";      ;;
    linux/arm64*|linux/aarch64)    TARGET="aarch64-unknown-linux-gnu"; ;;
    linux/x86_64|linux/x64)        TARGET="x86_64-unknown-linux-gnu";  ;;
    *)
      echo "Unsupported platform: ${PLAT}/${ARCH}"
      exit 1
      ;;
  esac

  DEST="$(derive_dest "$TARGET")"
  echo "Detected ${PLAT}/${ARCH} -> ${DEST} ($TARGET)"
else
  echo "Building $TARGET"
fi

if [[ -z "$DEST" ]]; then
  DEST="$(derive_dest "$TARGET")"
  if [[ -z "$DEST" ]]; then
    echo "Unknown target: $TARGET"
    exit 1
  fi
fi

DEST_DIR="$CARGO_DIR/dist/$DEST"

echo "Target: $TARGET → $DEST_DIR"

rustup target add "$TARGET" 2>/dev/null || true

cargo build --release --target "$TARGET" --manifest-path "$CARGO_DIR/Cargo.toml"

BIN_DIR="$DEST_DIR/bin"
mkdir -p "$BIN_DIR"

if [[ "$TARGET" == *"windows"* ]]; then
  cp "$CARGO_DIR/target/$TARGET/release/omc.exe" "$BIN_DIR/omc.exe"
  cp "$CARGO_DIR/target/$TARGET/release/omcd.exe" "$BIN_DIR/omcd.exe"
else
  cp "$CARGO_DIR/target/$TARGET/release/omc" "$BIN_DIR/omc"
  cp "$CARGO_DIR/target/$TARGET/release/omcd" "$BIN_DIR/omcd"
  chmod +x "$BIN_DIR/omc" "$BIN_DIR/omcd"
fi

echo "Binaries placed in $BIN_DIR"
