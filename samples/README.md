# Canonical Example Sample Pack

This directory holds the deterministic example asset set used by the Particelle examples, CI regression gate, and documentation snippets.

## Provenance

- Source: generated locally from repository-owned `sox` recipes in [`scripts/generate_sample_pack.sh`](/Users/cleider/dev/Particelle/scripts/generate_sample_pack.sh)
- License: MIT, same as the repository
- Format: PCM WAV, 44.1 kHz, 16-bit, mono
- Stability: `manifest.sha256` records the canonical hashes for every generated asset

## Regeneration

```sh
./scripts/generate_sample_pack.sh
```

The script scans `README.md`, `docs/`, and `examples/` for `samples/*.wav` references, regenerates the full pack deterministically, and refreshes [`manifest.sha256`](/Users/cleider/dev/Particelle/samples/manifest.sha256) if the canonical content changes.

To validate determinism and reference coverage end-to-end:

```sh
./scripts/verify_sample_pack.sh
```

## Coverage

The pack intentionally favors compact, synthetic fixtures over realism. The examples need stable, audible, category-distinct sources for regression testing, not licensed commercial recordings. Instrument and ambience names therefore indicate the sonic role each file approximates.
