# v0.5.0 Release Scope

This is a CLI-first release with two sound-design workflows: portable patch handoff and cloud stems. The patch schema remains v2. Linux x86_64, macOS Intel, and macOS Apple Silicon CLI archives, the installer, Homebrew, and the versioned user guide are supported. Obelisk M0 host and browser archives remain previews.

## User-Visible Features

1. `particelle bundle patch.yaml -o project-bundle` copies the patch and all referenced audio, analysis, curve, and Scala assets into one relocatable directory. It normalizes legacy patch syntax, rewrites asset paths, and records SHA-256 hashes. It never overwrites an existing directory. `particelle verify-bundle project-bundle` rejects missing, changed, unlisted, or external references.
2. `particelle render patch.yaml -o mix.wav --stems-dir stems` writes the ordinary mix plus one WAV per cloud. Each stem retains the declared channel layout, sample rate, duration, and encoding. The manifest maps filenames to cloud IDs. Stems reconstruct the mix within floating-point/encoding tolerance; integer PCM rounding can prevent byte-exact summation.
3. Invalid offline durations fail before an output WAV is opened.

## Acceptance Gates

- Integration tests move a bundle, remove the original assets, validate and render it, verify checksums, detect tampering, and reject overwrite. A multichannel stem test checks audible outputs, frame/channel counts, and mix reconstruction.
- Keep formatting, lint, workspace tests, performance, first-run, all 229 example audibility/fingerprint checks, user-guide examples, portable M0, Chromium browser checks, render parity, and extracted-archive sound checks green.
- Rebuild and inspect the v0.5.0 PDF. Publish it with signed/checksummed CLI, M0 preview, and browser preview assets.
- RC and GA tags must point to the same commit. Keep the GA candidate a prerelease until pinned installer, temporary-tap Homebrew, formula update, and public-tap installation pass. A failure leaves the current stable release unchanged; never rewrite published tags.

Named VST3/AU hosts, physical MPE controllers, MTS-ESP wrapper discovery, additional browser/OS combinations, and accessibility qualification are **not** claimed by this release. Their matrices remain in [M0 qualification](OBELISK_M0_QUALIFICATION.md) and [browser qualification](BROWSER_QUALIFICATION.md).
