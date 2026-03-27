#!/usr/bin/env bash
set -euo pipefail

export LANG=C
export LC_ALL=C

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

FINGERPRINT_MODE=update \
EXAMPLE_SHARD_TOTAL=1 \
EXAMPLE_SHARD_INDEX=0 \
"$repo_root/scripts/check_examples.sh"

echo "Golden fingerprint baseline updated at $repo_root/examples/golden_fingerprints.tsv."
