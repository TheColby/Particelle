# Announcement FAQ: Top 5 Expected Complaints

This document captures the five objections most likely to appear immediately after public announcement, and the concrete mitigations now in place.

## 1) "Realtime modulation is fake or misleading."

**Complaint:** Realtime `run` mode should not silently inject synthetic MPE behavior.

**Mitigation:**

- Synthetic MPE injection is now explicit opt-in via `particelle run ... --simulate-mpe`.
- Default realtime behavior no longer injects synthetic events unless requested.
- A runtime note is printed when MIDI bindings exist and synthetic injection is not enabled, so control expectations are explicit.

**Code:** `particelle-cli/src/main.rs` (`Run` command args and `cmd_run` behavior).

## 2) "CI is too slow if every PR renders every example serially."

**Complaint:** Full example rendering on every PR can bottleneck contribution flow.

**Mitigation:**

- Example regression checks now support deterministic sharding with:
  - `EXAMPLE_SHARD_TOTAL`
  - `EXAMPLE_SHARD_INDEX`
- CI now runs example regression in a 4-way matrix after core checks.

**Code:** `scripts/check_examples.sh`, `.github/workflows/ci.yml`.

## 3) "Audio gate says pass/fail, but not enough DSP signal detail."

**Complaint:** "Non-silent" alone is too weak for release confidence.

**Mitigation:**

- Regression gate records:
  - peak amplitude
  - RMS amplitude
  - crest factor
  - active channel count
  - per-channel RMS
- Per-shard summary reports now include clip-suspect and low-RMS counts with configurable thresholds.

**Code:** `scripts/check_examples.sh` output in `target/example-metrics/`.

## 4) "Canonical sample-pack claims are hard to trust."

**Complaint:** Generated fixtures need deterministic verification and coverage checks.

**Mitigation:**

- Added deterministic sample-pack verifier that:
  - regenerates the pack twice and checks manifest stability
  - validates that every `samples/*.wav` reference in docs/examples exists
- CI now runs sample-pack verification before fmt/lint/test.

**Code:** `scripts/verify_sample_pack.sh`, `.github/workflows/ci.yml`.

## 5) "Launch readiness is scattered across multiple manual commands."

**Complaint:** Announcing confidently requires one reproducible launch check.

**Mitigation:**

- Added one-command launch gate:
  - `./scripts/announcement_readiness.sh`
- The script runs sample-pack verification, fmt, clippy (`-D warnings`), workspace tests, and full example regression.
- It emits a timestamped report in `target/announcement-readiness/report.md`.

**Code:** `scripts/announcement_readiness.sh`.
