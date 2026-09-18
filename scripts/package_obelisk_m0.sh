#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: scripts/package_obelisk_m0.sh --target <rust-target> --version <version> [--out-dir <dir>]" >&2
}

target=""
version=""
out_dir="dist"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --target) target="${2:-}"; shift 2 ;;
    --version) version="${2:-}"; shift 2 ;;
    --out-dir) out_dir="${2:-}"; shift 2 ;;
    *) usage; exit 1 ;;
  esac
done
[[ -n "$target" && -n "$version" ]] || { usage; exit 1; }

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
build_dir="$repo_root/target/$target/release"
asset_base="obelisk-m0_${version}_${target}"
stage_dir="$(mktemp -d "${TMPDIR:-/tmp}/obelisk-m0.XXXXXX")"
trap 'rm -rf "$stage_dir"' EXIT
mkdir -p "$stage_dir/$asset_base/bin" "$stage_dir/$asset_base/include"

for binary in obelisk-m0-render obelisk-m0-live; do
  [[ -f "$build_dir/$binary" ]] || { echo "Missing binary: $build_dir/$binary" >&2; exit 1; }
  cp "$build_dir/$binary" "$stage_dir/$asset_base/bin/"
done
for library in "$build_dir"/libobelisk_m0.{a,dylib,so}; do
  [[ -f "$library" ]] && cp "$library" "$stage_dir/$asset_base/"
done
cp "$repo_root/obelisk-m0/include/obelisk_m0.h" "$stage_dir/$asset_base/include/"
cp "$repo_root/obelisk-m0/README.md" "$stage_dir/$asset_base/README.md"
mkdir -p "$out_dir"
tar -C "$stage_dir" -czf "$out_dir/$asset_base.tar.gz" "$asset_base"
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$out_dir/$asset_base.tar.gz" > "$out_dir/$asset_base.tar.gz.sha256"
else
  shasum -a 256 "$out_dir/$asset_base.tar.gz" > "$out_dir/$asset_base.tar.gz.sha256"
fi
echo "Packaged $out_dir/$asset_base.tar.gz"
