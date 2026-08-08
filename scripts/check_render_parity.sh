#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
out_dir="$repo_root/target/render-parity"
mkdir -p "$out_dir"
"$repo_root/scripts/prepare_example_samples.sh"
cargo build --release -p particelle-cli
bin="$repo_root/target/release/particelle"
patches=(examples/texture_cloud.yaml examples/chaos_scatter.yaml examples/shimmer_reverb.yaml)
printf '{"schema_version":1,"policy":"PCM16 output must be byte-identical per platform; cross-platform reports are compared in release CI","scenarios":[' > "$out_dir/report.json"
first=1
for patch in "${patches[@]}"; do
  a="$out_dir/$(basename "${patch%.yaml}")-a.wav"
  b="$out_dir/$(basename "${patch%.yaml}")-b.wav"
  "$bin" render "$patch" -o "$a" --duration 1 --format pcm16 >/dev/null
  "$bin" render "$patch" -o "$b" --duration 1 --format pcm16 >/dev/null
  hash_a="$(shasum -a 256 "$a" | awk '{print $1}')"
  hash_b="$(shasum -a 256 "$b" | awk '{print $1}')"
  [[ "$hash_a" == "$hash_b" ]] || { echo "Render parity failed for $patch" >&2; exit 1; }
  if (( first == 0 )); then
    printf ',' >> "$out_dir/report.json"
  fi
  first=0
  printf '{"patch":"%s","pcm16_sha256":"%s","status":"pass"}' "$patch" "$hash_a" >> "$out_dir/report.json"
done
printf ']}\n' >> "$out_dir/report.json"
echo "Render parity passed; report: $out_dir/report.json"
