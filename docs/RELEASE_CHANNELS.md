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
```

The installer verifies SHA-256 checksums by default for prebuilt channels.

## Release Assets

Each release publishes target-specific tarballs:

- `particelle_<version>_x86_64-unknown-linux-gnu.tar.gz`
- `particelle_<version>_x86_64-apple-darwin.tar.gz`
- `particelle_<version>_aarch64-apple-darwin.tar.gz`

Alongside:

- `*.sha256` per artifact
- consolidated `SHA256SUMS`
- Sigstore keyless signatures/certificates (`*.sig`, `*.pem`)

## CI/Automation

- Tagged releases are built/published by [`.github/workflows/release.yml`](/Users/cleider/dev/Particelle/.github/workflows/release.yml).
- Nightly channel is built/published by [`.github/workflows/nightly.yml`](/Users/cleider/dev/Particelle/.github/workflows/nightly.yml).
- Packaging logic is centralized in [`scripts/package_release.sh`](/Users/cleider/dev/Particelle/scripts/package_release.sh).
