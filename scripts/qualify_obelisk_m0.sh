#!/usr/bin/env bash
set -euo pipefail

# Portable M0 release gate. Physical MPE controller and DAW evidence is
# intentionally recorded separately in docs/OBELISK_M0_QUALIFICATION.md.
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

cargo test -p obelisk-m0
cargo run --quiet -p obelisk-m0 --bin obelisk-m0-render -- \
  --output target/obelisk-m0-qualification.wav --duration 1.0 --stress > target/obelisk-m0-qualification.log

rg -q 'audio-thread allocations: 0' target/obelisk-m0-qualification.log
rg -q '^rendered:' target/obelisk-m0-qualification.log
[[ -s target/obelisk-m0-qualification.wav ]]
echo "Obelisk M0 portable qualification passed."
