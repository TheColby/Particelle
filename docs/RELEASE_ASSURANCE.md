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
