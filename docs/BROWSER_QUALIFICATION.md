# Obelisk M0 Browser Preview Qualification

The browser bundle is a **preview**, not a supported cross-browser product. It is an offline, local WASM listening study, not a live MIDI instrument or Particelle YAML renderer. No audio is uploaded.

| Browser and OS | Evidence | Status |
| --- | --- | --- |
| Playwright Chromium on Ubuntu CI | Built WASM, worker render, playback controls, audible PCM16 stereo WAV download | Automated gate for the preview |
| Chrome on macOS | Optional local run via `PLAYWRIGHT_CHROME_CHANNEL=chrome`; no release-matrix record | Not qualified |
| Safari on macOS/iOS | No recorded device test | Not qualified |
| Firefox on desktop | No recorded browser test | Not qualified |
| Mobile Chromium or WebView | No recorded device test | Not qualified |

Accessibility is not certified. Before calling this a supported browser product, record keyboard-only operation, visible focus, control labels and names, screen-reader announcements, color contrast, zoom/reflow, and reduced-motion behavior on named browser/OS/assistive-technology combinations. File and fix failures before changing the support claim. The Chromium audio test does not substitute for those checks.

To test the preview locally, build WASM with `./scripts/build_wasm.sh`, then run `npm ci`, `npx playwright install --only-shell chromium`, and `npm run test:browser`. For a release archive, extract it, enter its folder, serve it with `python3 -m http.server 8000`, and visit `http://localhost:8000/`. Do not open `index.html` using a `file://` URL; workers and WASM need an HTTP origin.
