#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/package_release.sh --target <rust-target> [options]

Required:
  --target <target>         Rust target triple (e.g., x86_64-unknown-linux-gnu)

Optional:
  --version <version>       Release version/tag string (default: git describe --tags --always)
  --binary <path>           Path to built particelle binary
  --out-dir <dir>           Output directory (default: dist)
USAGE
}

target=""
version=""
binary=""
out_dir="dist"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      target="${2:-}"
      shift 2
      ;;
    --version)
      version="${2:-}"
      shift 2
      ;;
    --binary)
      binary="${2:-}"
      shift 2
      ;;
    --out-dir)
      out_dir="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$target" ]]; then
  echo "Missing required --target argument." >&2
  usage
  exit 1
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if [[ -z "$version" ]]; then
  version="$(git describe --tags --always)"
fi

if [[ -z "$binary" ]]; then
  binary="$repo_root/target/$target/release/particelle"
fi

if [[ ! -f "$binary" ]]; then
  echo "Binary not found: $binary" >&2
  exit 1
fi

mkdir -p "$out_dir"

asset_base="particelle_${version}_${target}"
asset_name="${asset_base}.tar.gz"
asset_path="$out_dir/$asset_name"

stage_dir="$(mktemp -d "${TMPDIR:-/tmp}/particelle-release.XXXXXX")"
trap 'rm -rf "$stage_dir"' EXIT

mkdir -p "$stage_dir/$asset_base/bin"
cp "$binary" "$stage_dir/$asset_base/bin/particelle"
chmod 0755 "$stage_dir/$asset_base/bin/particelle"
cp "$repo_root/README.md" "$stage_dir/$asset_base/README.md"
cp "$repo_root/CHANGELOG.md" "$stage_dir/$asset_base/CHANGELOG.md"
cp "$repo_root/install.sh" "$stage_dir/$asset_base/install.sh"

tar -C "$stage_dir" -czf "$asset_path" "$asset_base"

(
  cd "$out_dir"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$asset_name" >"${asset_name}.sha256"
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$asset_name" >"${asset_name}.sha256"
  else
    echo "Neither sha256sum nor shasum is available." >&2
    exit 1
  fi
)

echo "Packaged release artifact:"
echo "  - $asset_path"
echo "  - ${asset_path}.sha256"
