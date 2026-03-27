# Changelog

All notable changes to the Particelle synthetic engine will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Schema Versioning:** Added explicit `schema_version` to the root patch schema with current-version defaulting and starter-patch emission via `particelle init`.
- **Migration Metadata:** Added compatibility migration reporting with stable note IDs and source/target schema-version metadata (`parse_yaml_compat_with_report`).
- **Schema Migration Docs:** Added [`docs/SCHEMA_MIGRATIONS.md`](docs/SCHEMA_MIGRATIONS.md) with migration IDs and policy.
- **Installer Signature Modes:** Added Sigstore verification controls to `install.sh` (`auto`, `--verify-signatures`, `--skip-signature-verify`) for prebuilt channels.
- **Golden Fingerprint Baseline:** Added deterministic example-audio fingerprints in [`examples/golden_fingerprints.tsv`](examples/golden_fingerprints.tsv), plus a regeneration helper script [`scripts/update_golden_fingerprints.sh`](scripts/update_golden_fingerprints.sh).
- **Realtime Soak Benchmark:** Added [`particelle-core/examples/realtime_soak_benchmark.rs`](particelle-core/examples/realtime_soak_benchmark.rs) with XRUN-equivalent budget checks.

### Changed
- **Validation Guardrail:** Validation now rejects unsupported future schema versions (`schema_version > CURRENT_SCHEMA_VERSION`).
- **CLI Validation Output:** `particelle validate` now prints applied migration notes and the normalized schema version used for validation.
- **Release Channel Docs:** Documented checksum + signature verification behavior and workflow identity expectations in [`docs/RELEASE_CHANNELS.md`](docs/RELEASE_CHANNELS.md).
- **Example Regression Gate:** `scripts/check_examples.sh` now computes/stores PCM16 SHA-256 fingerprints and fails on mismatch against the golden baseline.
- **Performance Gate:** `scripts/check_performance.sh` now runs both block-latency and soak/XRUN stability benchmarks.

## [0.1.0] - 2026-03-03

### Added
- **Anisotropic Grain Directivity:** Granular objects are no longer limited to isotropic point sources. They now support continuous $G = \max(0, \delta + (1 - \delta) \cos(\theta))$ cardioid attenuation. Grains can morph continuously from omnidirectional ($\delta = 1.0$) to dipole figure-8 ($\delta = 0.0$).
- **Spatial Orientation AST Controls:** Added `orientation_azimuth`, `orientation_elevation`, and `directivity` continuous nodes to the `CloudConfig` struct to dynamically rotate grain radiation patterns.
- **Chaotic Attractor Oscillators:** Implemented lock-free recursive $f64$ evaluators for the **Lorenz**, **Rössler**, and **Hénon** systems. Attractors can be mapped to any audio-rate parameter in the `ParamSignal` AST.
- **Stochastic Drift Models:** Added a continuous 1D Brownian walk generator $\mathcal{N}(0, \sigma^2 dt)$ for infinite analog parameter drift without external curves.
- **Continuous Pitch Scaling Formulas:** Formalized the documentation and continuous logic for $E$-EDO Just Intonation mappings (e.g., $f(n) = f_{\text{ref}} \cdot 2^{\frac{n}{E}}$).
- **Harmonic Product Spectrum (HPS):** Added offline fundamental pitch extraction capability for noisy timbres lacking clear fundamentals inside `particelle-analysis`.
- **Advanced Stress Testing:** Merged a `complex` examples suite (`dxd_384khz_64ch`, `directional_shimmer`, `multi_cloud_saturation`, `chaos_lorenz`) cataloged via whitepaper.

### Changed
- **Spherical Head HRTF (IID):** Updated documentation to formally separate continuous Interaural Intensity Differences (acoustic shadows) from Woodworth's Interaural Time Differences (ITD) and Spectral Pinna filters.
- **Spatializer Array Architectures:** The `Spatializer::distribute` trait signature was completely rebuilt to accept dynamic spatial orientation vectors natively.
- **Mathematical Typography Mitigation:** Replaced all legacy block `$$` equations across the engine manifest with properly fenced GitHub ````math` blocks to guarantee cross-platform LaTeX rendering.
- **CLI Validation Architecture:** Extended the static schema to enforce continuous AST evaluation of all new chaotic and geometric nodes.

### Removed
- Removed the `(Upcoming)` tags on Phase 24 documentation, integrating the chaotic modulators officially into the main branch.
