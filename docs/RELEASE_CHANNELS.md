# Release Artifacts and Install Channels

Particelle ships signed release artifacts for macOS and Linux plus channel-aware install paths.

## Channels

- `stable`: latest tagged release (`v*`)
- `nightly`: rolling prerelease on tag `nightly`
- `source`: build locally from the checked-out repository

## Installer

Use the root installer script:

```sh
./install.sh --channel stable
./install.sh --channel nightly
./install.sh --channel source
./install.sh --channel stable --version v0.2.0
./install.sh --channel stable --verify-signatures
```

For prebuilt channels, the installer verifies SHA-256 checksums by default. Sigstore verification modes:

- `auto` (default): verify signatures when `cosign` is installed; otherwise continue with checksum verification.
- `--verify-signatures`: require signature verification and fail if `cosign` or signature assets are unavailable.
- `--skip-signature-verify`: disable signature verification and use checksum verification only.

## Release Assets

Each release publishes target-specific tarballs:

- `particelle_<version>_x86_64-unknown-linux-gnu.tar.gz`
- `particelle_<version>_x86_64-apple-darwin.tar.gz`
- `particelle_<version>_aarch64-apple-darwin.tar.gz`

Alongside:

- `*.sha256` per artifact
- consolidated `SHA256SUMS`
- Sigstore keyless signatures/certificates (`*.sig`, `*.pem`)

When signatures are verified, the installer validates keyless certificates against:

- OIDC issuer: `https://token.actions.githubusercontent.com`
- Workflow identity regex:
  - release tags: `.../.github/workflows/release.yml@...`
  - nightly channel: `.../.github/workflows/nightly.yml@...`

## CI/Automation

- Tagged releases are built/published by [`.github/workflows/release.yml`](/Users/cleider/dev/Particelle/.github/workflows/release.yml).
- Nightly channel is built/published by [`.github/workflows/nightly.yml`](/Users/cleider/dev/Particelle/.github/workflows/nightly.yml).
- Packaging logic is centralized in [`scripts/package_release.sh`](/Users/cleider/dev/Particelle/scripts/package_release.sh).
