# Curated Listening Demos

If you want to evaluate sound quality quickly (not just read specs), render the curated listening set:

```sh
./scripts/render_listening_demos.sh
```

Outputs are written to `target/listening-demos/`:

- `*.wav` rendered demo files (PCM24 for broad player compatibility)
- `playlist.m3u` playlist
- `metrics.tsv` render metadata + peak levels
- `README.md` manifest with demo descriptions

## Demo Catalog

Source catalog: [`examples/listening_demos.tsv`](/Users/cleider/dev/Particelle/examples/listening_demos.tsv)

Current set:

1. `phase_drift` — Reich-style phase drift
2. `stretch_glass` — four-times time stretch
3. `impact_bloom` — cinematic impact bloom
4. `analysis_crossfade` — cross-file analysis modulation
5. `directional_shimmer` — rotating directional shimmer

## Playback

On macOS:

```sh
afplay target/listening-demos/phase_drift.wav
```

Cross-platform:

```sh
ffplay -nodisp -autoexit target/listening-demos/phase_drift.wav
```

Or load `target/listening-demos/playlist.m3u` in your player of choice.
