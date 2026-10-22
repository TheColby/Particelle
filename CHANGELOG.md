# Changelog

All notable changes to the Particelle synthetic engine will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Began the CLI-first v0.4.0 release gates with a real Chromium test of the Obelisk M0 browser preview's render, playback controls, and non-silent WAV export. Browser and native host support claims remain evidence-gated.

## [0.3.2] - 2026-10-22

- Reissued the signed user guide with the working two-step Homebrew tap command. The v0.3.1 assets remain unchanged; use this release for the corrected PDF.
- Added a public-repository Homebrew tap installation test after the formula lands on `main` and before stable promotion.
- Added guide-text and release-note checks so the published instructions agree with the install path.

## [0.3.1] - 2026-10-21

- Fixed the stable installer exit trap, which caused a successful prebuilt installation to return a failure status.
- Made the release workflow derive CLI, guide, and archive version checks from the tag so patch releases do not require hard-coded edits.
- Corrected the Homebrew release gate for tap-only installs and added a plain-language listening chapter to the user guide, grounded in Curtis Roads's *Microsound* and electronic/pop recordings.
- Made the Homebrew formula find the executable both in a nested release tarball and in Homebrew's flattened staging directory.

## [0.3.0] - 2026-10-17 (prerelease; not promoted)

CLI-first candidate: the changes below were included in v0.3.0, but its installer promotion gate found an exit-status bug. The candidate remains a prerelease; v0.3.1 carries the fix. Obelisk M0 C API/host archives and browser code are previews; neither version qualifies VST3/AU hosts, physical MPE controllers, or real-browser playback.

### Added
- **Granular pitch controls:** Connected cloud `playback_rate` and tuning-selected `pitch` to source playback, including mixed source/output sample rates and `loop_source: false`.
- **M0 delivery:** Added JUCE VST3/AU wrapper source with in-process smoke coverage, standalone preset loading, a multichannel spatial voice bus, improved diffusion, and an offline WASM browser study.
- **Example scales:** Added the 24-EDO and Bohlen-Pierce Scala files referenced by shipped examples.
- **Schema Versioning:** Added explicit `schema_version` to the root patch schema with current-version defaulting and starter-patch emission via `particelle init`.
- **Migration Metadata:** Added compatibility migration reporting with stable note IDs and source/target schema-version metadata (`parse_yaml_compat_with_report`).
- **Schema Migration Docs:** Added [`docs/SCHEMA_MIGRATIONS.md`](docs/SCHEMA_MIGRATIONS.md) with migration IDs and policy.
- **Installer Signature Modes:** Added Sigstore verification controls to `install.sh` (`auto`, `--verify-signatures`, `--skip-signature-verify`) for prebuilt channels.
- **Golden Fingerprint Baseline:** Added deterministic example-audio fingerprints in [`examples/golden_fingerprints.tsv`](examples/golden_fingerprints.tsv), plus a regeneration helper script [`scripts/update_golden_fingerprints.sh`](scripts/update_golden_fingerprints.sh).
- **Realtime Soak Benchmark:** Added [`particelle-core/examples/realtime_soak_benchmark.rs`](particelle-core/examples/realtime_soak_benchmark.rs) with XRUN-equivalent budget checks.
- **Homebrew Formula:** Added [`Formula/particelle.rb`](Formula/particelle.rb) for first-party Homebrew installs from release artifacts.
- **Formula Updater:** Added [`scripts/update_homebrew_formula.sh`](scripts/update_homebrew_formula.sh) to regenerate formula URLs/checksums from GitHub release metadata.
- **Render Output Formats:** Added `particelle render --format {f32|pcm24|pcm16}` plus `--pcm24` shortcut for broader playback/tool compatibility.
- **Curated Listening Demos:** Added a 5-demo listening catalog [`examples/listening_demos.tsv`](examples/listening_demos.tsv), one-command renderer [`scripts/render_listening_demos.sh`](scripts/render_listening_demos.sh), and listening guide [`docs/LISTENING_DEMOS.md`](docs/LISTENING_DEMOS.md).
- **Interval-Aware Reconstruction:** Added `create_reconstructor_with_interval` so control-rate interpolation uses the caller's actual sample interval while preserving the 64-sample compatibility factory.

### Changed
- **Granular timing:** Grain onsets now render at their scheduled sample inside a block. The core audio processing path has allocation regression coverage.
- **Patch diagnostics:** Asset lookup prefers the patch directory with an explicit working-directory fallback, and unknown fields report errors. Legacy channel tags and numeric `pitch_ratio` retain narrow compatibility migrations.
- **YAML set:** `particelle set` now updates one structural path instead of replacing every textual key match.
- **Validation Guardrail:** Validation now rejects unsupported future schema versions (`schema_version > CURRENT_SCHEMA_VERSION`).
- **CLI Validation Output:** `particelle validate` now prints applied migration notes and the normalized schema version used for validation.
- **Release Channel Docs:** Documented checksum + signature verification behavior and workflow identity expectations in [`docs/RELEASE_CHANNELS.md`](docs/RELEASE_CHANNELS.md).
- **Example Regression Gate:** `scripts/check_examples.sh` now computes/stores PCM16 SHA-256 fingerprints and fails on mismatch against the golden baseline.
- **Performance Gate:** `scripts/check_performance.sh` now runs both block-latency and soak/XRUN stability benchmarks.
- **Release Channel Docs:** Added Homebrew install/update documentation in [`docs/RELEASE_CHANNELS.md`](docs/RELEASE_CHANNELS.md) and installer docs in [`README.md`](README.md).

### Fixed
- **Exponential Curves:** Normalized exponential segments now remain finite for extreme positive curvature, logarithmic segments treat zero curvature as linear, and invalid logarithmic/power domains are rejected during compilation.
- **Control Reconstruction:** Replaced cubic, monotone-cubic, sinc, two-pole, and MinBLEP linear fallbacks with dedicated processors; one-pole and slew modes now continue advancing between control ticks.
- **Rust 1.98 CI:** Reworked the M0 live stereo copy loop to satisfy the latest strict clippy lint without raising the Rust 1.70 minimum version.
- **Portable M0 Qualification:** The hardware-free M0 release gate now disables realtime audio features, avoiding accidental ALSA dependencies on clean CI runners.

## [0.2.3] - 2026-03-15

Release-workflow compatibility update for macOS archive targets.

## [0.2.2] - 2026-03-15

Linux CI dependency and release/source build corrections.

## [0.2.1] - 2026-03-15

Sample-pack verification fallback when `ripgrep` is unavailable.

## [0.2.0] - 2026-03-15

First tagged 0.2 release. The CLI binary in earlier 0.2 artifacts still reported `0.1.0`; v0.3.0 aligns the binary and release versions.

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
