#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# WASM_BUILD_STD=1 supports Homebrew Rust with matching rust-src but no target.
args=()
if [[ "${WASM_BUILD_STD:-0}" == 1 ]]; then
  export RUSTC_BOOTSTRAP=1
  args+=(-Z build-std=std,panic_abort)
elif ! compgen -G "$(rustc --print target-libdir --target wasm32-unknown-unknown)/libstd-*.rlib" >/dev/null; then
  printf 'wasm32-unknown-unknown standard library is missing. Install the target, or run WASM_BUILD_STD=1 with matching rust-src.\n' >&2
  exit 1
fi
cargo build "${args[@]}" --manifest-path "$ROOT/obelisk-wasm/Cargo.toml" \
  --release --lib --target wasm32-unknown-unknown --no-default-features
mkdir -p "$ROOT/web/pkg"
cp "$ROOT/obelisk-wasm/target/wasm32-unknown-unknown/release/obelisk_wasm.wasm" "$ROOT/web/pkg/obelisk_wasm.wasm.tmp"
mv "$ROOT/web/pkg/obelisk_wasm.wasm.tmp" "$ROOT/web/pkg/obelisk_wasm.wasm"
printf 'Built real M0 WASM: %s\n' "$ROOT/web/pkg/obelisk_wasm.wasm"
