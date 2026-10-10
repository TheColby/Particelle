#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${OBELISK_PLUGIN_BUILD_DIR:-$ROOT/native/obelisk-plugin/build}"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
if [[ "$TARGET_DIR" != /* ]]; then
  TARGET_DIR="$PWD/$TARGET_DIR"
fi

if [[ -n "${JUCE_SOURCE_DIR:-}" && ! -f "$JUCE_SOURCE_DIR/CMakeLists.txt" ]]; then
  echo "JUCE_SOURCE_DIR must point to a local JUCE checkout" >&2
  exit 1
fi

cargo build --manifest-path "$ROOT/Cargo.toml" --release -p obelisk-m0 --lib
RUST_LIB="$TARGET_DIR/release/libobelisk_m0.a"
if [[ ! -f "$RUST_LIB" ]]; then
  echo "Rust static library not found: $RUST_LIB" >&2
  exit 1
fi

args=(-DOBELISK_RUST_LIB="$RUST_LIB")
if [[ -n "${JUCE_SOURCE_DIR:-}" ]]; then
  args+=(-DJUCE_SOURCE_DIR="$JUCE_SOURCE_DIR")
fi
cmake -S "$ROOT/native/obelisk-plugin" -B "$BUILD_DIR" "${args[@]}" "$@"
cmake --build "$BUILD_DIR" --config Release --parallel "${OBELISK_BUILD_JOBS:-2}"
echo "Built Obelisk M0 plugin artifacts under $BUILD_DIR/ObeliskM0_artefacts"
