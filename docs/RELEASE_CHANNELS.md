# Release Artifacts and Install Channels

Particelle ships signed release artifacts for macOS and Linux plus channel-aware install paths.

## Channels

- `stable`: latest promoted, non-prerelease release (not an RC)
- `nightly`: rolling prerelease on tag `nightly`
- `source`: build locally from the checked-out repository

## Installer

Use the root installer script:

```sh
./install.sh --channel stable
./install.sh --channel nightly
./install.sh --channel source
./install.sh --channel stable --version v0.2.3
./install.sh --channel stable --verify-signatures
```

For prebuilt channels, the installer verifies SHA-256 checksums by default. Sigstore verification modes:

- `auto` (default): verify signatures when `cosign` is installed; otherwise continue with checksum verification.
- `--verify-signatures`: require signature verification and fail if `cosign` or signature assets are unavailable.
- `--skip-signature-verify`: disable signature verification and use checksum verification only.

## Homebrew

Particelle ships a first-party formula at [`Formula/particelle.rb`](/Users/cleider/dev/Particelle/Formula/particelle.rb).

Install:

```sh
brew install --formula https://raw.githubusercontent.com/TheColby/Particelle/main/Formula/particelle.rb
```

Formula maintenance:

```sh
./scripts/update_homebrew_formula.sh --tag v0.2.3
```

This script refreshes release URLs and SHA-256 checksums in the formula from GitHub release metadata.
Without `--tag`, it selects the latest non-prerelease release; RC testing always passes an explicit tag. The release check creates a temporary Homebrew tap before installing the generated formula, because current Homebrew rejects direct local formula paths. The formula's version matches the CLI version even when its download URLs point to an RC tag.

## Release Assets

Each release publishes target-specific tarballs:

- `particelle_<version>_x86_64-unknown-linux-gnu.tar.gz`
- `particelle_<version>_x86_64-apple-darwin.tar.gz`
- `particelle_<version>_aarch64-apple-darwin.tar.gz`

Alongside:

- `*.sha256` per artifact
- consolidated `SHA256SUMS`
- Sigstore keyless signatures/certificates (`*.sig`, `*.pem`)
- `Particelle_User_Guide_v<version>.pdf`, plus the listening demos, presets, and `obelisk-m0-preview_<version>_<target>.tar.gz` C API/host archives
- Cargo metadata SBOM and release provenance, also checksummed and signed

`SHA256SUMS` covers all unsigned deliverables, including individual `.sha256` sidecars. Signatures and certificates are produced afterward and therefore cannot be included in that manifest without a circular dependency.

When signatures are verified, the installer validates keyless certificates against:

- OIDC issuer: `https://token.actions.githubusercontent.com`
- Workflow identity regex:
  - release tags: `.../.github/workflows/release.yml@...`
  - nightly channel: `.../.github/workflows/nightly.yml@...`

## CI/Automation

- A maintainer creates an RC tag. [`.github/workflows/release.yml`](/Users/cleider/dev/Particelle/.github/workflows/release.yml) runs release gates and publishes it as a prerelease. The pinned RC installer and Homebrew formula must pass before automation tags the same commit for GA. GA stays a prerelease until its own installer and Homebrew tests pass and the formula update lands on `main`. v0.3.0 was stopped at the GA installer gate; v0.3.1-rc.1 was stopped because Homebrew requires a tap; v0.3.1-rc.2 exposed Homebrew's flattened archive layout. The corrected candidate tests both layouts before running an install through a temporary tap.
- Failed gates leave the current stable release untouched. Corrected candidates require a new RC tag; published tags are never rewritten.
- Nightly channel is built/published by [`.github/workflows/nightly.yml`](/Users/cleider/dev/Particelle/.github/workflows/nightly.yml).
- Packaging logic is centralized in [`scripts/package_release.sh`](/Users/cleider/dev/Particelle/scripts/package_release.sh).
