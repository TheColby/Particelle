# ai2yaml Recipe Book
# 
# This folder contains ready-to-run YAML patches generated via the ai2yaml tool.
# Each file begins with the prompt that generated it so you can reproduce or remix it.
#
# How to use any recipe:
#   particelle validate examples/ai2yaml/<recipe>.yaml
#   particelle render examples/ai2yaml/<recipe>.yaml -o out.wav --duration 30.0
#   particelle run examples/ai2yaml/<recipe>.yaml
#
# To regenerate from the prompt (requires OPENAI_API_KEY or similar):
#   ./ai2yaml "<prompt text>" output.yaml
#
# ─────────────────────────────────────────────────────────────────
# RECIPE INDEX
# ─────────────────────────────────────────────────────────────────
# 01_31edo_drone.yaml        — 60-second 31-EDO drone, slow circle pan
# 02_pitch_follower.yaml     — F0-driven grain durations from vocal source
# 03_spectral_shimmer.yaml   — Spectral flatness cross-maps window size
# 04_percussive_freeze.yaml  — Freeze & stretch a percussive transient
# 05_atmos_rain.yaml         — 12-channel Atmos spatial rain cloud
# 06_sidechain_gate.yaml     — RMS sidechain gates a second cloud's amplitude
# 07_chaotic_lfo_swarm.yaml  — Three interlocking phasor LFOs controlling density
# 08_chroma_driven_pan.yaml  — Dominant pitch class sweeps spatial position
# 09_mfcc_texture.yaml       — MFCC-3 modulates grain width for timbral morphing
# 10_polyrhythmic_bursts.yaml — Three clouds with prime-ratio density LFOs
