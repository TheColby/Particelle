#!/usr/bin/env bash
set -euo pipefail

export LANG=C
export LC_ALL=C

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest_path="$repo_root/samples/manifest.sha256"

hash_file() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{ print $1 }'
  else
    shasum -a 256 "$file" | awk '{ print $1 }'
  fi
}

if ! command -v sox >/dev/null 2>&1; then
  echo "Missing required dependency: sox" >&2
  exit 1
fi

"$repo_root/scripts/generate_sample_pack.sh"
if [[ ! -f "$manifest_path" ]]; then
  echo "Missing sample-pack manifest: $manifest_path" >&2
  exit 1
fi
first_manifest_hash="$(hash_file "$manifest_path")"

"$repo_root/scripts/generate_sample_pack.sh"
second_manifest_hash="$(hash_file "$manifest_path")"

if [[ "$first_manifest_hash" != "$second_manifest_hash" ]]; then
  echo "Sample-pack manifest changed between consecutive generations." >&2
  exit 1
fi

missing=0
while IFS= read -r rel_path; do
  sample_path="$repo_root/${rel_path}"
  if [[ ! -f "$sample_path" ]]; then
    echo "Missing referenced sample asset: $rel_path" >&2
    missing=$((missing + 1))
  fi
done < <(
  cd "$repo_root" &&
    rg -No 'samples/[A-Za-z0-9_./-]+\.wav' README.md docs examples -g '*.md' -g '*.yaml' |
    sed 's/^[^:]*://' |
    sort -u
)

if (( missing > 0 )); then
  echo "Sample-pack verification failed with $missing missing files." >&2
  exit 1
fi

echo "Verified canonical sample pack determinism and coverage."
