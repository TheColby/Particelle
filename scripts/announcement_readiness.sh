#!/usr/bin/env bash
set -euo pipefail

export LANG=C
export LC_ALL=C

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

out_dir="$repo_root/target/announcement-readiness"
mkdir -p "$out_dir"
report_path="$out_dir/report.md"

run_step() {
  local label="$1"
  shift
  echo "→ $label"
  "$@"
}

run_step "Verify canonical sample pack" "$repo_root/scripts/verify_sample_pack.sh"
run_step "Check formatting" cargo fmt --all -- --check
run_step "Lint workspace" cargo clippy --workspace --all-targets -- -D warnings
run_step "Run workspace tests" cargo test --workspace
run_step "Run performance gate" "$repo_root/scripts/check_performance.sh"
run_step "Run full example regression gate" "$repo_root/scripts/check_examples.sh"

git_rev="$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

{
  printf '# Announcement Readiness Report\n\n'
  printf -- '- generated_at_utc: `%s`\n' "$generated_at"
  printf -- '- git_rev: `%s`\n' "$git_rev"
  printf -- '- checks: `verify_sample_pack`, `fmt`, `clippy -D warnings`, `cargo test --workspace`, `check_performance`, `check_examples`\n'
  printf -- '- metrics: [`target/example-metrics/summary.md`](/Users/cleider/dev/Particelle/target/example-metrics/summary.md)\n'
  printf -- '- details: [`target/example-metrics/examples.tsv`](/Users/cleider/dev/Particelle/target/example-metrics/examples.tsv)\n'
  printf -- '- fingerprints: [`target/example-metrics/fingerprints.tsv`](/Users/cleider/dev/Particelle/target/example-metrics/fingerprints.tsv)\n'
  printf -- '- performance: [`target/performance/report.md`](/Users/cleider/dev/Particelle/target/performance/report.md)\n'
} >"$report_path"

echo "Wrote readiness report to $report_path."
