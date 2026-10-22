# Obelisk M0 browser study

This is a dependency-free offline renderer for the fixed-topology Obelisk M0 synthesizer. The browser uses the Rust M0 core compiled to WASM, not a JavaScript sound-alike. It renders a four-note phrase and two-second effects tail at 48 kHz, plays through Web Audio, and exports stereo 16-bit PCM WAV. Rendering happens in a Web Worker; nothing is uploaded.

From the repository root:

```sh
./scripts/build_wasm.sh
python3 -m http.server 8000 --directory web
```

Then open `http://localhost:8000`. Serve over HTTP rather than opening `index.html` as a `file://` URL, because module workers and WASM fetching require a web origin. The page loads fonts from Google Fonts when online and falls back to local serif/sans/monospace fonts offline; synthesis itself needs no network after the local page and WASM load.

On a Homebrew Rust toolchain without a prebuilt `wasm32-unknown-unknown` standard library, install matching `rust-src` and a WASM linker (`rust-lld` or an appropriate `wasm-ld` configured for Rust), then run `WASM_BUILD_STD=1 ./scripts/build_wasm.sh`. With rustup, install the target using `rustup target add wasm32-unknown-unknown`.

Run `node --test web/audio.test.mjs` for the WAV encoder tests. After a WASM build, run `node web/wasm-smoke.mjs` to instantiate the actual module and check for finite, nonzero stereo output. For a real Chromium render, playback-control, and WAV-download check, run `npm ci`, `npx playwright install --only-shell chromium`, then `npm run test:browser`. If Chrome is already installed locally, set `PLAYWRIGHT_CHROME_CHANNEL=chrome` to use it instead of downloading Chromium. CI and release preflight run the browser test with the built WASM. The generated `web/pkg/` directory is intentionally ignored; rebuild it when the M0 core or wrapper changes.

Release archives include `obelisk-m0-browser-preview_<tag>.tar.gz`. Verify it using the release checksum and signature manifest, extract it, enter the extracted folder, run `python3 -m http.server 8000`, and open `http://localhost:8000/`. The archive includes the tested WASM; no Rust build is needed for listening. This is Chromium evidence for a preview, not a claim of Safari/Firefox support, accessibility conformance, or hosted-playground qualification. See [the browser qualification matrix](../docs/BROWSER_QUALIFICATION.md).
