#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

out_dir="${1:-$repo_root/dist/presets}"
mkdir -p "$out_dir"
archive="$out_dir/particelle-presets-1.0.0.tar.gz"
tar -czf "$archive" presets
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$archive" > "$archive.sha256"
else
  shasum -a 256 "$archive" > "$archive.sha256"
fi
echo "Packaged versioned preset catalog: $archive"
