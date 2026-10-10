# Release Assurance

## Listening Bundle

`scripts/package_listening_demos.sh` renders the curated PCM24 demo catalog, rejects silent output, writes metrics and a playlist, and packages the result as `particelle-listening-demos.tar.gz`. Release and nightly workflows publish this bundle beside binaries so listeners can evaluate the engine without local setup.

## Provenance and SBOM

Release workflows generate two machine-readable files:

- `particelle.sbom.cargo-metadata.json`: Cargo package graph metadata.
- `particelle.provenance.json`: revision, generation timestamp, and SHA-256 digests for release/demo artifacts.

Binary archives and checksum manifests remain Sigstore-signed. Verify both the archive signature and its digest before installation.

## Cross-Platform Render Policy

`scripts/check_render_parity.sh` runs deterministic PCM16 renders twice for each canonical scenario and rejects any same-platform byte mismatch. CI runs this policy on Linux and macOS and uploads a report from each platform. Cross-platform reports are retained for release review; an unexplained PCM16 hash divergence blocks release promotion until the numerical/output-format cause is recorded.

## First-Run Gate

`scripts/check_first_run.sh` validates the shell-safe path used in the README: generate a patch, validate it both as a file and from stdin, render PCM24 audio, and reject an empty output. CI runs this gate on every change.

## New Audio Delivery Checks

The example gate validates and renders every shipped YAML patch, rejects silent output, and compares PCM16 hashes against `examples/golden_fingerprints.tsv`. The baseline was regenerated only after two complete 229-example runs produced identical hashes following the sample-position scheduler change.

The M0 portable gate checks audible output and zero audio-thread allocations. The browser CI job tests the Rust renderer and WAV encoder and builds the `wasm32-unknown-unknown` artifact. The local Homebrew Rust installation used for this change lacks `rust-lld`; its native renderer and JavaScript tests passed, but local WASM playback was not verified. A CI build and browser playback check are still required before publishing a hosted playground.

The JUCE VST3/AU wrapper builds and passes an in-process MIDI/audio/state smoke test on macOS with a local JUCE CMake package. These checks do not qualify a named DAW, physical MPE controller, MTS-ESP bridge, code signing, or notarization. Record those separately in [`OBELISK_M0_QUALIFICATION.md`](OBELISK_M0_QUALIFICATION.md).
