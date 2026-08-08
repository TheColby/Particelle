#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
"$repo_root/scripts/prepare_example_samples.sh"
cargo build --release -p particelle-cli
bin="$repo_root/target/release/particelle"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/particelle-first-run.XXXXXX")"
trap 'rm -rf "$work_dir"' EXIT

"$bin" init > "$work_dir/first.yaml"
"$bin" validate "$work_dir/first.yaml"
"$bin" render "$work_dir/first.yaml" --duration 0.25 --format pcm24 -o "$work_dir/first.wav"
[[ -s "$work_dir/first.wav" ]] || { echo "First-run render did not produce audio." >&2; exit 1; }
cat "$work_dir/first.yaml" | "$bin" validate - >/dev/null
echo "First-run shell flow passed."
