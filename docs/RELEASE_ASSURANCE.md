# Release Assurance

## Listening Bundle

`scripts/package_listening_demos.sh` renders the curated PCM24 demo catalog, rejects silent output, writes metrics and a playlist, and packages the result as `particelle-listening-demos.tar.gz`. Release and nightly workflows publish this bundle beside binaries so listeners can evaluate the engine without local setup.

## Provenance and SBOM

Release workflows generate two machine-readable files:

- `particelle.sbom.cargo-metadata.json`: Cargo package graph metadata.
- `particelle.provenance.json`: revision, generation timestamp, and SHA-256 digests for release/demo artifacts.

All unsigned release deliverables (archives, PDF guide, SBOM, provenance, and per-asset sidecars) appear in `SHA256SUMS`; each and the manifest receive Sigstore signatures. Verify both the archive signature and its digest before installation. Signature files cannot be included in the manifest they sign.

## v0.4.0 Promotion Gate

The release workflow runs locked-dependency version checks, workspace formatting/lint/tests, performance, first-run, all example audibility/fingerprints, portable M0, browser build tests, and native Linux/macOS Intel/arm64 render parity before packaging. Each extracted CLI archive must report the tagged workspace version and produce non-silent audio through `init`, `validate`, and `render`; a local archive installer test also checks exit status. The guide PDF is rebuilt and visually inspected before commit; CI verifies its teaching examples, and the publisher checks the committed PDF's version, page count, fonts, and Homebrew commands before attaching the versioned copy. This avoids a release-time TeX package download. An RC passes to a GA tag on the same commit only after pinned RC installer and Homebrew tests pass; GA remains a prerelease until its own pinned installer and Homebrew installation succeed, the formula update lands, and the public tap installs successfully. The previous stable release stays available until that happens. Historical prerelease tags and assets are not rewritten.

## Cross-Platform Render Policy

`scripts/check_render_parity.sh` runs deterministic PCM16 renders twice for each canonical scenario and rejects any same-platform byte mismatch. CI runs this policy on Linux and macOS and uploads a report from each platform. Cross-platform reports are retained for release review; an unexplained PCM16 hash divergence blocks release promotion until the numerical/output-format cause is recorded.

## First-Run Gate

`scripts/check_first_run.sh` validates the shell-safe path used in the README: generate a patch, validate it both as a file and from stdin, render PCM24 audio, and reject an empty output. CI runs this gate on every change.

## New Audio Delivery Checks

The example gate validates and renders every shipped YAML patch, rejects silent output, and compares PCM16 hashes against `examples/golden_fingerprints.tsv`. The baseline was regenerated only after two complete 229-example runs produced identical hashes following the sample-position scheduler change.

The M0 portable gate checks audible output and zero audio-thread allocations. The browser CI job tests the Rust renderer and WAV encoder, builds the `wasm32-unknown-unknown` artifact, and runs a real Chromium render/playback-control/WAV-export test. Release preflight runs the same browser gate. The Linux build packages a versioned browser preview archive, checked alongside the other unsigned deliverables and signed in the same pass. This is evidence for the Chromium preview path, not a blanket claim for Safari, Firefox, accessibility, or hosted-playground support. See [the browser matrix](BROWSER_QUALIFICATION.md).

The JUCE VST3/AU wrapper builds and passes an in-process MIDI/audio/state smoke test on macOS with a local JUCE CMake package. These checks do not qualify a named DAW, physical MPE controller, MTS-ESP bridge, code signing, or notarization. Record those separately in [`OBELISK_M0_QUALIFICATION.md`](OBELISK_M0_QUALIFICATION.md).
