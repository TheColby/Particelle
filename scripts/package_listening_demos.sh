#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
out_dir="${1:-$repo_root/dist/listening-demos}"
mkdir -p "$out_dir"

LISTENING_DEMO_OUT_DIR="$out_dir/content" "$repo_root/scripts/render_listening_demos.sh"
tar -C "$out_dir" -czf "$out_dir/particelle-listening-demos.tar.gz" content
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$out_dir/particelle-listening-demos.tar.gz" > "$out_dir/particelle-listening-demos.tar.gz.sha256"
else
  shasum -a 256 "$out_dir/particelle-listening-demos.tar.gz" > "$out_dir/particelle-listening-demos.tar.gz.sha256"
fi
echo "Packaged listening demos in $out_dir"
