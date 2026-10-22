# v0.4.0 Release Scope

The next release remains CLI-first. Linux x86_64, macOS Intel, and macOS Apple Silicon archives, the installer, Homebrew, and the versioned user guide are the supported release surface. Obelisk M0 host archives and the browser study stay clearly labeled previews. There is no fixed ship date or patch-schema break planned.

## Release Gates

1. Preserve the v0.3.x CLI acceptance suite: formatting, lint, workspace tests, performance, first-run, all 229 example audibility/fingerprints, portable M0, native render parity, and three extracted-archive sound checks.
2. Add a real-browser Chromium check against the built WASM: load the worker, render, exercise playback controls, download a PCM16 stereo WAV, and reject silence or malformed headers. Keep the Node WASM and encoder tests as faster lower-level checks.
3. Package a versioned static browser preview only after the browser check is green. Record tested browser/OS combinations and accessibility evidence in [the browser matrix](BROWSER_QUALIFICATION.md). Chromium alone does not qualify Safari or Firefox.
4. Collect named VST3/AU host and physical MPE-controller evidence using [the qualification matrix](OBELISK_M0_QUALIFICATION.md). Until those tests, keep native host archives previews; do not claim named DAW or hardware compatibility. MTS-ESP wrapper discovery and expanded bus layouts require implementation plus host tests before promotion.
5. Cut an RC only after the release workflow verifies locked dependencies, guide PDF, checksums, provenance, signatures, pinned installer, and temporary-tap Homebrew install. Promote the same commit only after the GA assets, formula update, and public-tap install pass. Never rewrite published tags.

## Release State

The source version is 0.4.0. CI and release preflight run the Chromium browser test; the Linux release job packages the tested WASM and static files as `obelisk-m0-browser-preview_<tag>.tar.gz`. The browser matrix records what is and is not qualified. Host and controller qualification still requires named physical runs. Published release status is determined by GitHub release metadata, not this document.
