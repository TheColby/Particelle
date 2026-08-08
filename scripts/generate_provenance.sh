#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
out_dir="${1:-$repo_root/dist}"
mkdir -p "$out_dir"
revision="$(git rev-parse HEAD)"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

cargo metadata --format-version 1 --no-deps > "$out_dir/particelle.sbom.cargo-metadata.json"
{
  printf '{\n'
  printf '  "schema_version": 1,\n'
  printf '  "project": "particelle",\n'
  printf '  "revision": "%s",\n' "$revision"
  printf '  "generated_at_utc": "%s",\n' "$generated_at"
  printf '  "artifacts": [\n'
  first=1
  for artifact in "$out_dir"/*.tar.gz "$out_dir"/*.wav; do
    [[ -f "$artifact" ]] || continue
    digest="$(shasum -a 256 "$artifact" | awk '{print $1}')"
    if (( first == 0 )); then
      printf ',\n'
    fi
    first=0
    printf '    {"path":"%s","sha256":"%s"}' "${artifact#$out_dir/}" "$digest"
  done
  printf '\n  ]\n}\n'
} > "$out_dir/particelle.provenance.json"
echo "Wrote SBOM and provenance metadata to $out_dir"
