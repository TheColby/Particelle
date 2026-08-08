#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/realtime_soak.sh <patch.yaml> [options]

Options:
  --duration <seconds>       Device soak duration (default: 60)
  --out <telemetry.json>     Diagnostics output (default: target/realtime-soak.json)
  --max-deadline-misses <n>  Allowed deadline misses (default: 0)
  --max-dropped <n>          Allowed dropped callbacks (default: 0)
USAGE
}

patch="${1:-}"
[[ -n "$patch" ]] || { usage >&2; exit 1; }
shift
duration=60
out="target/realtime-soak.json"
max_deadline_misses=0
max_dropped=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --duration) duration="$2"; shift 2 ;;
    --out) out="$2"; shift 2 ;;
    --max-deadline-misses) max_deadline_misses="$2"; shift 2 ;;
    --max-dropped) max_dropped="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
mkdir -p "$(dirname "$out")"
cargo build --release -p particelle-cli
"$repo_root/target/release/particelle" run "$patch" --duration "$duration" --telemetry-file "$out"

deadline_misses="$(sed -n 's/.*"deadline_misses": \([0-9][0-9]*\).*/\1/p' "$out" | head -n 1)"
dropped="$(sed -n 's/.*"dropped_callbacks": \([0-9][0-9]*\).*/\1/p' "$out" | head -n 1)"
[[ -n "$deadline_misses" && -n "$dropped" ]] || { echo "Invalid telemetry report: $out" >&2; exit 1; }
if (( deadline_misses > max_deadline_misses || dropped > max_dropped )); then
  echo "Realtime soak failed: deadline_misses=$deadline_misses (max $max_deadline_misses), dropped_callbacks=$dropped (max $max_dropped)" >&2
  exit 1
fi
echo "Realtime soak passed: $out"
