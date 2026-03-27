#!/usr/bin/env bash
set -euo pipefail

export LANG=C
export LC_ALL=C

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

"$repo_root/scripts/prepare_example_samples.sh"

if ! command -v sox >/dev/null 2>&1; then
  echo "Missing required dependency: sox" >&2
  exit 1
fi

if ! command -v perl >/dev/null 2>&1; then
  echo "Missing required dependency: perl" >&2
  exit 1
fi

cargo build --release -p particelle-cli

bin="$repo_root/target/release/particelle"
perf_dir="$repo_root/target/performance"
mkdir -p "$perf_dir"
report_path="$perf_dir/report.md"
render_tsv="$perf_dir/render_throughput.tsv"
latency_txt="$perf_dir/realtime_block_latency.txt"
soak_txt="$perf_dir/realtime_soak_latency.txt"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/particelle-perf.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT

stereo_min_factor="${PERF_MIN_FACTOR_STEREO:-4.0}"
multichannel_min_factor="${PERF_MIN_FACTOR_MULTICHANNEL:-1.5}"
highrate_min_factor="${PERF_MIN_FACTOR_HIGHRATE:-0.8}"
max_avg_block_ratio="${MAX_AVG_BLOCK_RATIO:-0.45}"
max_p95_block_ratio="${MAX_P95_BLOCK_RATIO:-0.90}"
max_soak_avg_block_ratio="${MAX_SOAK_AVG_BLOCK_RATIO:-0.55}"
max_soak_p99_block_ratio="${MAX_SOAK_P99_BLOCK_RATIO:-0.98}"
max_soak_xrun_ratio="${MAX_SOAK_XRUN_RATIO:-0.005}"
max_soak_consecutive_xruns="${MAX_SOAK_CONSECUTIVE_XRUNS:-2}"
bench_soak_blocks="${BENCH_SOAK_BLOCKS:-12000}"

printf 'scenario\tpatch\tduration_s\telapsed_s\trealtime_factor\tmin_factor\n' >"$render_tsv"

bench_render() {
  local scenario="$1"
  local patch="$2"
  local duration_s="$3"
  local min_factor="$4"
  local out_path="$tmp_dir/${scenario}.wav"

  local start end elapsed factor
  start="$(perl -MTime::HiRes=time -e 'printf "%.9f\n", time')"
  "$bin" render "$patch" -o "$out_path" --duration "$duration_s" >/dev/null 2>&1
  end="$(perl -MTime::HiRes=time -e 'printf "%.9f\n", time')"

  elapsed="$(awk -v s="$start" -v e="$end" 'BEGIN { printf "%.6f", (e - s) }')"
  factor="$(
    awk -v d="$duration_s" -v e="$elapsed" 'BEGIN {
      if (e <= 0.0) { printf "0.000"; }
      else { printf "%.3f", d / e; }
    }'
  )"

  printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$scenario" \
    "$patch" \
    "$duration_s" \
    "$elapsed" \
    "$factor" \
    "$min_factor" >>"$render_tsv"

  if ! awk -v f="$factor" -v min="$min_factor" 'BEGIN { exit !(f >= min) }'; then
    echo "Render throughput below threshold for $scenario: ${factor}x < ${min_factor}x realtime" >&2
    exit 1
  fi
}

bench_render "stereo_48k" "examples/stereo/drone_stereo_05.yaml" "2.0" "$stereo_min_factor"
bench_render "multichannel_48k" "examples/multichannel/drone_multichannel_21.yaml" "2.0" "$multichannel_min_factor"
bench_render "highrate_96k" "examples/complex/max_density_swarm.yaml" "1.0" "$highrate_min_factor"

MAX_AVG_BLOCK_RATIO="$max_avg_block_ratio" \
MAX_P95_BLOCK_RATIO="$max_p95_block_ratio" \
cargo run --quiet --release -p particelle-core --example realtime_block_benchmark \
  >"$latency_txt" 2>&1

MAX_SOAK_AVG_BLOCK_RATIO="$max_soak_avg_block_ratio" \
MAX_SOAK_P99_BLOCK_RATIO="$max_soak_p99_block_ratio" \
MAX_SOAK_XRUN_RATIO="$max_soak_xrun_ratio" \
MAX_SOAK_CONSECUTIVE_XRUNS="$max_soak_consecutive_xruns" \
BENCH_SOAK_BLOCKS="$bench_soak_blocks" \
cargo run --quiet --release -p particelle-core --example realtime_soak_benchmark \
  >"$soak_txt" 2>&1

{
  printf '# Performance Gate Report\n\n'
  printf -- '- render_thresholds: stereo `%sx`, multichannel `%sx`, highrate `%sx`\n' \
    "$stereo_min_factor" "$multichannel_min_factor" "$highrate_min_factor"
  printf -- '- block_latency_thresholds: avg_ratio `%s`, p95_ratio `%s`\n' \
    "$max_avg_block_ratio" "$max_p95_block_ratio"
  printf -- '- soak_thresholds: avg_ratio `%s`, p99_ratio `%s`, xrun_ratio `%s`, max_consecutive_xruns `%s`, blocks `%s`\n' \
    "$max_soak_avg_block_ratio" "$max_soak_p99_block_ratio" "$max_soak_xrun_ratio" "$max_soak_consecutive_xruns" "$bench_soak_blocks"
  printf -- '- render_results: [`target/performance/render_throughput.tsv`](/Users/cleider/dev/Particelle/target/performance/render_throughput.tsv)\n'
  printf -- '- realtime_block_latency: [`target/performance/realtime_block_latency.txt`](/Users/cleider/dev/Particelle/target/performance/realtime_block_latency.txt)\n'
  printf -- '- realtime_soak_latency: [`target/performance/realtime_soak_latency.txt`](/Users/cleider/dev/Particelle/target/performance/realtime_soak_latency.txt)\n'
  printf '\n## Realtime Block Benchmark Output\n\n'
  printf '```text\n'
  cat "$latency_txt"
  printf '```\n'
  printf '\n## Realtime Soak Benchmark Output\n\n'
  printf '```text\n'
  cat "$soak_txt"
  printf '```\n'
} >"$report_path"

echo "Performance gate passed."
echo "Wrote performance report to $report_path."
