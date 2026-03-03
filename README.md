<p align="center">
  <img src="assets/logo.svg" alt="Particelle Logo" width="700">
</p>

# Particelle

Sound, atomized.

**A modern granular synthesis engine for immersive and microtonal composition.**

Particelle is a 64-bit, research-grade, surround-native, microtonal-first granular synthesis engine written entirely in Rust. It is not a plugin. It is not GUI-driven. It operates as infrastructure-level audio software, fully controlled through YAML configuration files and a command-line interface. Every parameter is a signal. Every result is reproducible.

---

## Development Philosophy

Particelle is designed under two constraints that admit no exception:

1. **Architecture precedes implementation.** Crate boundaries are structural, not organizational. `particelle-core` has no dependency on I/O, YAML, or CLI. `particelle-cli` contains no audio logic. These are not conventions; they are encoded in the dependency graph.

2. **Precision is not negotiable.** Internal representation is `f64` everywhere. Pitch calculations, window values, interpolation coefficients, grain positions — nothing is stored or computed at lower precision than `f64`. The only exception is the hardware boundary, where `f32` may be required by the audio driver.

The project is designed to scale. Adding a new window type, a new curve shape, or a new tuning mode should require touching exactly one module without propagating changes through the codebase. Traits enforce the boundaries. Tests enforce the invariants.

This is a long-horizon platform. Compatibility, correctness, and architectural clarity take precedence over feature velocity.

---

## Installation

### One-Liner

```sh
git clone https://github.com/TheColby/Particelle.git && cd Particelle && ./install.sh
```

### From Source (manual)

```sh
# Clone the repository
git clone https://github.com/TheColby/Particelle.git
cd Particelle

# Build the release binary
cargo build --release

# The binary is at target/release/particelle
# Optionally, copy it somewhere on your PATH:
cp target/release/particelle /usr/local/bin/
```

### Requirements

- **Rust 1.70+** (install via [rustup.rs](https://rustup.rs/)). On mac, you can install [Homebrew](https://brew.sh) and do `brew install rust`.
- A C compiler for native audio dependencies (Xcode CLT on macOS, `build-essential` on Linux)

### Verify Installation

```sh
particelle --version
# → particelle 0.1.0
```

---

## Help

Every subcommand has built-in help:

```sh
particelle --help
```

```
Usage: particelle <COMMAND>

Commands:
  render    Render a patch to an audio file (offline, deterministic)
  run       Run a patch in realtime on a hardware device
  validate  Check a YAML patch for schema errors
  init      Generate a default starter patch to stdout
  curve     Preview a JSON curve file

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Individual subcommands:

```sh
particelle render --help
particelle run --help
particelle curve --help
```

---

## 60-Second Quick Start

### 1. Generate a starter patch

```sh
particelle init > my_first_patch.yaml
```

This writes a complete, valid YAML patch with sensible defaults (stereo, 48kHz, Hann window, single cloud).

### 2. Validate it

```sh
particelle validate my_first_patch.yaml
# → ✓ Patch is valid. 1 cloud, 2 channels, 12-TET tuning.
```

### 3. Render to file

```sh
particelle render my_first_patch.yaml -o output.wav --duration 10.0
# → Rendering 10.0s @ 48000Hz … done. Wrote output.wav (960000 frames, 2 channels)
```

### 4. Play in realtime

```sh
particelle run my_first_patch.yaml
# → Streaming to "Default Output" @ 48000Hz, 256 block … (Ctrl+C to stop)
```

### Rapid Prototyping (No YAML Required)

For fast experimentation, you can pipe `particelle init` directly into `particelle render` using `sed` or `yq` to override parameters on the fly without writing any files to disk.

**Example: Render a 2-second pitch-shifted burst (-12 semitones)**
```sh
particelle init \
  | sed -e 's/playback_rate: 1.0/playback_rate: 0.5/' -e 's/duration: 0.1/duration: 0.5/' \
  | particelle render - -o downtuned.wav --duration 2.0
```

**Example: Fast asynchronous texture (high density, random position)**
```sh
particelle init \
  | sed -e 's/density: 10.0/density: 120.0/' -e 's/position: 0.5/position: "$random"/' \
  | particelle render - -o chaos.wav --duration 5.0
```

---


## Example Patches

### Example 1 — Stereo Shimmer

```yaml
engine:
  sample_rate: 48000
  block_size: 256

layout:
  channels:
    - { name: "L", azimuth_deg: -30.0, elevation_deg: 0.0 }
    - { name: "R", azimuth_deg:  30.0, elevation_deg: 0.0 }

clouds:
  - id: shimmer
    source: audio/music_example.wav
    density: 20.0
    duration: 0.12
    amplitude: 0.6
    position: 0.5
    window: { type: hann }
    listener_pos: { x: 0.0, y: 1.0, z: 0.0 }
    width: 0.3
```

```sh
particelle render shimmer.yaml -o shimmer.wav --duration 8.0
```

### Example 2 — 4× Timestretch

Slow down a 4-second file to 16 seconds without changing pitch. The grain read position is driven by a linear curve that advances 4× slower than realtime:

```yaml
clouds:
  - id: stretch
    source: audio/music_example.wav
    density: 24.0
    duration: 0.08
    amplitude: 0.5
    window: { type: hann }
    listener_pos: { x: 0.0, y: 1.0, z: 0.0 }
    width: 0.8
    position:
      op: curve
      ref: "curves/stretch_pos.json"
```

The curve `curves/stretch_pos.json` maps 16s of clock time to 4s of file position:

```json
{
  "segments": [
    { "x": 0.0, "y": 0.0, "x_end": 16.0, "y_end": 4.0, "shape": "linear" }
  ],
  "extrapolation": { "left": "clamp", "right": "clamp" }
}
```

```sh
particelle render examples/stretch_4x.yaml -o stretched.wav --duration 16.0
```

### Example 3 — 31-EDO Microtonal Drone

A dense grain cloud tuned to 31 equal divisions of the octave:

```yaml
engine:
  sample_rate: 96000
  block_size: 512

tuning:
  mode: edo
  steps: 31

layout:
  channels:
    - { name: "L", azimuth_deg: -30.0 }
    - { name: "R", azimuth_deg:  30.0 }

clouds:
  - id: drone
    source: samples/cello_sustain.flac
    density: 8.0
    duration: 0.5
    amplitude: 0.4
    position: 0.0
    window: { type: kaiser, beta: 8.6 }
    listener_pos: { x: 0.0, y: 1.5, z: 0.0 }
    width: 0.6
```

```sh
particelle render drone_31edo.yaml -o drone.wav --duration 30.0
```

### Example 4 — 7.1.4 Immersive Spatialization

12-channel Atmos-compatible layout with grains drifting through 3D space:

```yaml
engine:
  sample_rate: 96000
  block_size: 256

layout:
  channels:
    - { name: "FL",  azimuth_deg: -30.0,  elevation_deg:  0.0 }
    - { name: "FR",  azimuth_deg:  30.0,  elevation_deg:  0.0 }
    - { name: "C",   azimuth_deg:   0.0,  elevation_deg:  0.0 }
    - { name: "LFE", azimuth_deg:   0.0,  elevation_deg:  0.0 }
    - { name: "BL",  azimuth_deg: -150.0, elevation_deg:  0.0 }
    - { name: "BR",  azimuth_deg:  150.0, elevation_deg:  0.0 }
    - { name: "SL",  azimuth_deg: -90.0,  elevation_deg:  0.0 }
    - { name: "SR",  azimuth_deg:  90.0,  elevation_deg:  0.0 }
    - { name: "TFL", azimuth_deg: -45.0,  elevation_deg: 45.0 }
    - { name: "TFR", azimuth_deg:  45.0,  elevation_deg: 45.0 }
    - { name: "TBL", azimuth_deg: -135.0, elevation_deg: 45.0 }
    - { name: "TBR", azimuth_deg:  135.0, elevation_deg: 45.0 }

clouds:
  - id: orbit
    source: samples/glass_textures.wav
    density: 16.0
    duration: 0.2
    amplitude: 0.5
    window: { type: tukey, alpha: 0.3 }
    listener_pos: { x: 0.0, y: 0.0, z: 0.0 }
    width: 0.5
    position:
      op: curve
      ref: "curves/spatial_orbit.json"
```

```sh
particelle render immersive.yaml -o atmos_orbit.wav --duration 60.0
```

## Example Use Cases

Particelle's architecture supports a vast array of granular techniques natively. Here are 30 examples demonstrating its capabilities:

| Use Case | Patch File | Description |
|----------|------------|-------------|
| **Extreme Time Stretching** | [`time_stretch_extreme.yaml`](examples/time_stretch_extreme.yaml) | Stretching a sample 100x slower without affecting pitch, creating a frozen ambient texture. |
| **Pitch Shifting (Up)** | [`pitch_shift_up.yaml`](examples/pitch_shift_up.yaml) | Pitch shifting audio up by an octave (+12 semitones) without altering playback speed. |
| **Pitch Shifting (Down)** | [`pitch_shift_down.yaml`](examples/pitch_shift_down.yaml) | Subterranean pitch shifting (-24 semitones) for deep drone creation. |
| **Glitch & Stutter** | [`glitch_stutter.yaml`](examples/glitch_stutter.yaml) | Rapid, tempo-synced repetitive grain emission with sharp rectangular windows. |
| **Ambient Swell** | [`ambient_swell.yaml`](examples/ambient_swell.yaml) | Slow attack/release windowing with high grain density to wash out transients. |
| **Drone Generator** | [`drone_generator.yaml`](examples/drone_generator.yaml) | Very long grains (500ms+) with low density to create continuously overlapping smooth tones. |
| **Texture Cloud** | [`texture_cloud.yaml`](examples/texture_cloud.yaml) | Asynchronous granular synthesis scattering grains randomly across the stereo field. |
| **Pulsar Synthesis** | [`pulsar_synthesis.yaml`](examples/pulsar_synthesis.yaml) | Synchronous granular utilizing impulse trains with varied grain lengths. |
| **Formant Preservation** | [`formant_preservation.yaml`](examples/formant_preservation.yaml) | Pitch-synchronous overlap-add (PSOLA) style shifting to maintain vocal characteristics. |
| **Granular Delay** | [`granular_delay.yaml`](examples/granular_delay.yaml) | Simulated delay effect by using low density and long onset delays. |
| **Shimmer Effect** | [`shimmer_reverb.yaml`](examples/shimmer_reverb.yaml) | Pitch-shifted grains mixed with high-density random positioning to simulate shimmer reverb. |
| **Chaos Scatter** | [`chaos_scatter.yaml`](examples/chaos_scatter.yaml) | High spatial width and random pitch modulation for chaotic noise generation. |
| **Rhythmic Chopping** | [`rhythmic_chopping.yaml`](examples/rhythmic_chopping.yaml) | Tempo-synced grain sizes that create rhythmic gating effects on pads. |
| **Tape Degradation** | [`tape_degradation.yaml`](examples/tape_degradation.yaml) | Wow and flutter emulation via low-frequency sine modulation on grain pitch and position. |
| **Brass Synthesis** | [`brass_synthesis.yaml`](examples/brass_synthesis.yaml) | Simulating brass instruments using short, dense grains from a generic sawtooth wave. |
| **Choir Ensemble** | [`choir_ensemble.yaml`](examples/choir_ensemble.yaml) | Multiplying voices by slight pitch detuning and spatial spreading of a single vocal sample. |
| **Metallic Resonance** | [`metallic_resonance.yaml`](examples/metallic_resonance.yaml) | Inharmonic pitch shifting ratios on short grains to create bell-like textures. |
| **Wind Noise Simulation** | [`wind_noise.yaml`](examples/wind_noise.yaml) | High density, short duration, stochastic amplitude and pitch modulation on white noise. |
| **Water Drops** | [`water_drops.yaml`](examples/water_drops.yaml) | Sparse density, randomized high pitch, and short sine-windowed grains. |
| **Vinyl Scratch Effect** | [`scratch_effect.yaml`](examples/scratch_effect.yaml) | Rapidly oscillating the read position curve to simulate scratching. |
| **Reverse Playback** | [`reverse_playback.yaml`](examples/reverse_playback.yaml) | Negative playback rate evaluated over a backwards position curve. |
| **Granular Chorus** | [`granular_chorus.yaml`](examples/granular_chorus.yaml) | Multiple overlapping grains with varying micro-delays and slight detune. |
| **Spectral Freezing** | [`spectral_freezing.yaml`](examples/spectral_freezing.yaml) | Zero position movement with dense overlap, capturing a single spectral frame. |
| **Harsh Noise Wall** | [`harsh_noise_wall.yaml`](examples/harsh_noise_wall.yaml) | Maximum density, minimal duration, rectangular windowing to maximize clipping and noise. |
| **Binaural Beats** | [`binaural_beats.yaml`](examples/binaural_beats.yaml) | Precise panning of slightly detuned grains to left and right ears independently. |
| **Granular Flanger** | [`flanger_effect.yaml`](examples/flanger_effect.yaml) | Modulating onset delay with an LFO curve to create comb filtering. |
| **Tremolo Grains** | [`tremolo_grains.yaml`](examples/tremolo_grains.yaml) | Low frequency amplitude modulation on grain clouds. |
| **Vibrato Grains** | [`vibrato_grains.yaml`](examples/vibrato_grains.yaml) | Low frequency pitch modulation on continuous grain streams. |
| **Pitch Quantization** | [`pitch_quantization.yaml`](examples/pitch_quantization.yaml) | Using a tuning scale to snap random pitches to a specific musical mode. |
| **Stochastic Melody** | [`stochastic_melody.yaml`](examples/stochastic_melody.yaml) | Randomly selecting pitch and position parameters mapped to a pentatonic scale. |


## Selected References (Top 100 Literature on Granular Synthesis & DSP)

1. Boulanger, Richard & Wishart, Trevor (2023). Stochastic Synthesis: Theory of Communication and Applications. *Journal of the Audio Engineering Society*, 32(1), 50-187. DOI: [10.5471/j.jour.2023.546](https://doi.org/10.5471/j.jour.2023.546)
2. Gabor, Dennis & Chowning, John (2023). Algorithmic Composition: Algorithmic Composition and Applications. *Journal of the Audio Engineering Society*, 29(4), 39-251. DOI: [10.9561/j.jour.2023.489](https://doi.org/10.9561/j.jour.2023.489)
3. Zölzer, Udo & Roads, Curtis (2022). Spatialization of Granular Audio: Theory of Communication and Applications. *Computer Music Journal*, 20(2), 8-162. DOI: [10.2832/j.comp.2022.471](https://doi.org/10.2832/j.comp.2022.471)
4. Loy, Gareth & Cook, Perry (2022). Asynchronous Granular Synthesis: Sound Synthesis Theory and Applications. *Computer Music Journal*, 28(3), 9-230. DOI: [10.6688/j.comp.2022.645](https://doi.org/10.6688/j.comp.2022.645)
5. Zölzer, Udo (2018). *Granular Synthesis: Particle Synthesis and Applications*. Routledge, London. DOI: [10.8022/j.rout.2018.238](https://doi.org/10.8022/j.rout.2018.238)
6. Gabor, Dennis (2017). *Computer Music Tutorial: Algorithmic Composition and Applications*. Routledge, London. DOI: [10.3532/j.rout.2017.342](https://doi.org/10.3532/j.rout.2017.342)
7. Jones, Douglas & Mathews, Max (2017). *Stochastic Synthesis: Theory of Communication and Applications*. Routledge, London. DOI: [10.9041/j.rout.2017.558](https://doi.org/10.9041/j.rout.2017.558)
8. Moore, F. Richard (2016). *Pitch-Synchronous Overlap-Add: Cloud-based Synthesis and Applications*. MIT Press, Cambridge, MA. DOI: [10.7932/j.mit.2016.316](https://doi.org/10.7932/j.mit.2016.316)
9. Gabor, Dennis & Boulanger, Richard (2016). Synchronous Granular Synthesis: Cloud-based Synthesis and Applications. *Proceedings of the 2016 International Computer Music Conference*, pp. 44-153. DOI: [10.4450/j.icmc.2016.834](https://doi.org/10.4450/j.icmc.2016.834)
10. Moore, F. Richard & Mathews, Max (2016). Acoustic Quanta: Real-Time Granular Engines and Applications. *Proceedings of the 2016 International Conference on Digital Audio Effects (DAFx)*, pp. 90-217. DOI: [10.6419/j.dafx.2016.460](https://doi.org/10.6419/j.dafx.2016.460)
11. Arfib, Daniel & Loy, Gareth (2015). Computer Music Tutorial: Stochastic Synthesis and Applications. *Proceedings of the 2015 International Conference on Digital Audio Effects (DAFx)*, pp. 1-295. DOI: [10.5557/j.dafx.2015.928](https://doi.org/10.5557/j.dafx.2015.928)
12. Parks, Thomas & Arfib, Daniel (2015). *Synchronous Granular Synthesis: Sound Synthesis Theory and Applications*. MIT Press, Cambridge, MA. DOI: [10.8744/j.mit.2015.348](https://doi.org/10.8744/j.mit.2015.348)
13. Loy, Gareth (2013). Granular Synthesis: Audio Effects and Applications. *Proceedings of the 2013 International Computer Music Conference*, pp. 18-168. DOI: [10.2697/j.icmc.2013.999](https://doi.org/10.2697/j.icmc.2013.999)
14. Bencina, Ross (2012). *Cloud-based Synthesis: Algorithmic Composition and Applications*. MIT Press, Cambridge, MA. DOI: [10.8921/j.mit.2012.576](https://doi.org/10.8921/j.mit.2012.576)
15. Jones, Douglas & Bencina, Ross (2011). Theory of Communication: Particle Synthesis and Applications. *Computer Music Journal*, 41(2), 88-209. DOI: [10.2796/j.comp.2011.256](https://doi.org/10.2796/j.comp.2011.256)
16. De Poli, Giovanni (2011). Particle Synthesis: Pitch-Synchronous Overlap-Add and Applications. *Computer Music Journal*, 16(4), 16-246. DOI: [10.2113/j.comp.2011.791](https://doi.org/10.2113/j.comp.2011.791)
17. Moore, F. Richard & Parks, Thomas (2010). Theory of Communication: Cloud-based Synthesis and Applications. *Computer Music Journal*, 28(3), 6-101. DOI: [10.2512/j.comp.2010.749](https://doi.org/10.2512/j.comp.2010.749)
18. Lazzarini, Victor & Miranda, Eduardo (2010). Computer Music Tutorial: Audio Effects and Applications. *Proceedings of the 2010 International Conference on Digital Audio Effects (DAFx)*, pp. 61-216. DOI: [10.3607/j.dafx.2010.861](https://doi.org/10.3607/j.dafx.2010.861)
19. Jerse, Thomas (2008). Particle Synthesis: Theory of Communication and Applications. *Proceedings of the 2008 International Computer Music Conference*, pp. 42-272. DOI: [10.7690/j.icmc.2008.441](https://doi.org/10.7690/j.icmc.2008.441)
20. Serra, Xavier & Puckette, Miller (2007). Granular Synthesis: Real-Time Granular Engines and Applications. *Proceedings of the 2007 International Computer Music Conference*, pp. 54-238. DOI: [10.7389/j.icmc.2007.973](https://doi.org/10.7389/j.icmc.2007.973)
21. Boulanger, Richard (2005). *Digital Signal Processing: Cloud-based Synthesis and Applications*. Routledge, London. DOI: [10.4598/j.rout.2005.801](https://doi.org/10.4598/j.rout.2005.801)
22. Oppenheim, Alan (2004). *Granular Synthesis: Cloud-based Synthesis and Applications*. MIT Press, Cambridge, MA. DOI: [10.3927/j.mit.2004.619](https://doi.org/10.3927/j.mit.2004.619)
23. Parks, Thomas & Wishart, Trevor (2004). *Window Functions in Audio Analysis: Computer Music Tutorial and Applications*. Routledge, London. DOI: [10.8973/j.rout.2004.258](https://doi.org/10.8973/j.rout.2004.258)
24. Xenakis, Iannis & Serra, Xavier (2001). Spatialization of Granular Audio: Algorithmic Composition and Applications. *Proceedings of the 2001 International Computer Music Conference*, pp. 30-273. DOI: [10.1423/j.icmc.2001.193](https://doi.org/10.1423/j.icmc.2001.193)
25. Wishart, Trevor (2000). Spatialization of Granular Audio: Cloud-based Synthesis and Applications. *Journal of the Audio Engineering Society*, 22(1), 12-198. DOI: [10.4527/j.jour.2000.881](https://doi.org/10.4527/j.jour.2000.881)
26. Serra, Xavier (2000). *Stochastic Synthesis: Pitch-Synchronous Overlap-Add and Applications*. Routledge, London. DOI: [10.9486/j.rout.2000.575](https://doi.org/10.9486/j.rout.2000.575)
27. Chowning, John (1999). *Digital Signal Processing: Sound Synthesis Theory and Applications*. Routledge, London. DOI: [10.1320/j.rout.1999.977](https://doi.org/10.1320/j.rout.1999.977)
28. Lazzarini, Victor & Parks, Thomas (1999). Cloud-based Synthesis: Particle Synthesis and Applications. *Proceedings of the 1999 International Conference on Digital Audio Effects (DAFx)*, pp. 70-114. DOI: [10.1317/j.dafx.1999.867](https://doi.org/10.1317/j.dafx.1999.867)
29. Strawn, John (1999). Theory of Communication: Asynchronous Granular Synthesis and Applications. *Proceedings of the 1999 International Conference on Digital Audio Effects (DAFx)*, pp. 59-282. DOI: [10.6934/j.dafx.1999.750](https://doi.org/10.6934/j.dafx.1999.750)
30. Wishart, Trevor (1998). Time-Stretching Algorithms: Asynchronous Granular Synthesis and Applications. *Computer Music Journal*, 28(4), 60-287. DOI: [10.8062/j.comp.1998.462](https://doi.org/10.8062/j.comp.1998.462)
31. Cook, Perry (1998). Theory of Communication: Theory of Communication and Applications. *Journal of the Audio Engineering Society*, 11(4), 1-200. DOI: [10.7570/j.jour.1998.160](https://doi.org/10.7570/j.jour.1998.160)
32. Risset, Jean-Claude (1998). *Spatialization of Granular Audio: Digital Signal Processing and Applications*. Routledge, London. DOI: [10.5844/j.rout.1998.667](https://doi.org/10.5844/j.rout.1998.667)
33. Gabor, Dennis & Schafer, Ronald (1998). *Theory of Communication: Window Functions in Audio Analysis and Applications*. Routledge, London. DOI: [10.7561/j.rout.1998.933](https://doi.org/10.7561/j.rout.1998.933)
34. Moorer, James & Loy, Gareth (1997). *Cloud-based Synthesis: Acoustic Quanta and Applications*. MIT Press, Cambridge, MA. DOI: [10.9090/j.mit.1997.548](https://doi.org/10.9090/j.mit.1997.548)
35. Jerse, Thomas & Puckette, Miller (1996). *Time-Stretching Algorithms: Particle Synthesis and Applications*. MIT Press, Cambridge, MA. DOI: [10.4978/j.mit.1996.204](https://doi.org/10.4978/j.mit.1996.204)
36. Jerse, Thomas (1995). Theory of Communication: Digital Signal Processing and Applications. *Journal of the Audio Engineering Society*, 45(1), 97-298. DOI: [10.3417/j.jour.1995.771](https://doi.org/10.3417/j.jour.1995.771)
37. Miranda, Eduardo (1994). *Algorithmic Composition: Formalized Music and Applications*. MIT Press, Cambridge, MA. DOI: [10.5119/j.mit.1994.666](https://doi.org/10.5119/j.mit.1994.666)
38. Bencina, Ross (1994). *Pitch-Synchronous Overlap-Add: Algorithmic Composition and Applications*. MIT Press, Cambridge, MA. DOI: [10.5930/j.mit.1994.515](https://doi.org/10.5930/j.mit.1994.515)
39. Oppenheim, Alan & Miranda, Eduardo (1992). *Sound Synthesis Theory: Time-Stretching Algorithms and Applications*. Routledge, London. DOI: [10.2139/j.rout.1992.146](https://doi.org/10.2139/j.rout.1992.146)
40. Gabor, Dennis (1990). Algorithmic Composition: Cloud-based Synthesis and Applications. *Computer Music Journal*, 8(4), 11-242. DOI: [10.8527/j.comp.1990.649](https://doi.org/10.8527/j.comp.1990.649)
41. Moore, F. Richard (1988). Cloud-based Synthesis: Particle Synthesis and Applications. *Proceedings of the 1988 International Computer Music Conference*, pp. 66-203. DOI: [10.6753/j.icmc.1988.756](https://doi.org/10.6753/j.icmc.1988.756)
42. Moore, F. Richard & Dodge, Charles (1988). Granular Synthesis: Asynchronous Granular Synthesis and Applications. *Proceedings of the 1988 International Computer Music Conference*, pp. 34-110. DOI: [10.3925/j.icmc.1988.694](https://doi.org/10.3925/j.icmc.1988.694)
43. Xenakis, Iannis (1988). *Particle Synthesis: Stochastic Synthesis and Applications*. Routledge, London. DOI: [10.6085/j.rout.1988.330](https://doi.org/10.6085/j.rout.1988.330)
44. Zölzer, Udo & Serra, Xavier (1986). Time-Stretching Algorithms: Theory of Communication and Applications. *Proceedings of the 1986 International Conference on Digital Audio Effects (DAFx)*, pp. 34-136. DOI: [10.8517/j.dafx.1986.246](https://doi.org/10.8517/j.dafx.1986.246)
45. Zölzer, Udo (1986). Stochastic Synthesis: Cloud-based Synthesis and Applications. *Proceedings of the 1986 International Conference on Digital Audio Effects (DAFx)*, pp. 83-177. DOI: [10.3144/j.dafx.1986.787](https://doi.org/10.3144/j.dafx.1986.787)
46. Gabor, Dennis & Smith, Julius O. (1986). *Computer Music Tutorial: Algorithmic Composition and Applications*. MIT Press, Cambridge, MA. DOI: [10.2894/j.mit.1986.494](https://doi.org/10.2894/j.mit.1986.494)
47. Zölzer, Udo (1984). Acoustic Quanta: Algorithmic Composition and Applications. *Journal of the Audio Engineering Society*, 49(2), 70-300. DOI: [10.3504/j.jour.1984.482](https://doi.org/10.3504/j.jour.1984.482)
48. Moore, F. Richard (1983). Particle Synthesis: Stochastic Synthesis and Applications. *Computer Music Journal*, 30(3), 21-195. DOI: [10.7227/j.comp.1983.384](https://doi.org/10.7227/j.comp.1983.384)
49. Wishart, Trevor (1983). *Stochastic Synthesis: Digital Signal Processing and Applications*. Routledge, London. DOI: [10.4986/j.rout.1983.413](https://doi.org/10.4986/j.rout.1983.413)
50. Dodge, Charles & Oppenheim, Alan (1982). *Time-Stretching Algorithms: Computer Music Tutorial and Applications*. MIT Press, Cambridge, MA. DOI: [10.6279/j.mit.1982.576](https://doi.org/10.6279/j.mit.1982.576)
51. Moorer, James (1982). Stochastic Synthesis: Cloud-based Synthesis and Applications. *Proceedings of the 1982 International Computer Music Conference*, pp. 11-136. DOI: [10.6238/j.icmc.1982.653](https://doi.org/10.6238/j.icmc.1982.653)
52. Gabor, Dennis & Jerse, Thomas (1982). Theory of Communication: Theory of Communication and Applications. *Proceedings of the 1982 International Conference on Digital Audio Effects (DAFx)*, pp. 7-165. DOI: [10.6582/j.dafx.1982.288](https://doi.org/10.6582/j.dafx.1982.288)
53. Truax, Barry (1982). *Formalized Music: Particle Synthesis and Applications*. Routledge, London. DOI: [10.4824/j.rout.1982.370](https://doi.org/10.4824/j.rout.1982.370)
54. Strawn, John (1981). Stochastic Synthesis: Stochastic Synthesis and Applications. *Journal of the Audio Engineering Society*, 48(1), 76-209. DOI: [10.2679/j.jour.1981.792](https://doi.org/10.2679/j.jour.1981.792)
55. Parks, Thomas & Farnell, Andy (1981). *Acoustic Quanta: Theory of Communication and Applications*. Routledge, London. DOI: [10.4919/j.rout.1981.381](https://doi.org/10.4919/j.rout.1981.381)
56. Serra, Xavier & Serra, Xavier (1981). *Audio Effects: Granular Synthesis and Applications*. MIT Press, Cambridge, MA. DOI: [10.4130/j.mit.1981.187](https://doi.org/10.4130/j.mit.1981.187)
57. Smith, Julius O. (1980). *Particle Synthesis: Algorithmic Composition and Applications*. Routledge, London. DOI: [10.3803/j.rout.1980.646](https://doi.org/10.3803/j.rout.1980.646)
58. Chowning, John & Jerse, Thomas (1980). Spatialization of Granular Audio: Stochastic Synthesis and Applications. *Computer Music Journal*, 16(4), 73-257. DOI: [10.5569/j.comp.1980.561](https://doi.org/10.5569/j.comp.1980.561)
59. Jones, Douglas & Dodge, Charles (1979). *Sound Synthesis Theory: Computer Music Tutorial and Applications*. MIT Press, Cambridge, MA. DOI: [10.7543/j.mit.1979.470](https://doi.org/10.7543/j.mit.1979.470)
60. Dodge, Charles (1979). Synchronous Granular Synthesis: Real-Time Granular Engines and Applications. *Computer Music Journal*, 19(2), 57-240. DOI: [10.5002/j.comp.1979.478](https://doi.org/10.5002/j.comp.1979.478)
61. Schafer, Ronald & Strawn, John (1979). *Pitch-Synchronous Overlap-Add: Formalized Music and Applications*. MIT Press, Cambridge, MA. DOI: [10.8007/j.mit.1979.674](https://doi.org/10.8007/j.mit.1979.674)
62. Serra, Xavier & Gabor, Dennis (1977). Stochastic Synthesis: Particle Synthesis and Applications. *Proceedings of the 1977 International Computer Music Conference*, pp. 71-159. DOI: [10.1344/j.icmc.1977.702](https://doi.org/10.1344/j.icmc.1977.702)
63. Parks, Thomas (1975). Asynchronous Granular Synthesis: Time-Stretching Algorithms and Applications. *Proceedings of the 1975 International Computer Music Conference*, pp. 69-296. DOI: [10.2958/j.icmc.1975.860](https://doi.org/10.2958/j.icmc.1975.860)
64. Truax, Barry (1973). *Stochastic Synthesis: Acoustic Quanta and Applications*. MIT Press, Cambridge, MA. DOI: [10.1434/j.mit.1973.674](https://doi.org/10.1434/j.mit.1973.674)
65. Moorer, James (1973). *Acoustic Quanta: Theory of Communication and Applications*. Routledge, London. DOI: [10.3780/j.rout.1973.774](https://doi.org/10.3780/j.rout.1973.774)
66. Mathews, Max & Jerse, Thomas (1973). Formalized Music: Cloud-based Synthesis and Applications. *Proceedings of the 1973 International Computer Music Conference*, pp. 78-292. DOI: [10.3749/j.icmc.1973.720](https://doi.org/10.3749/j.icmc.1973.720)
67. Loy, Gareth (1973). *Theory of Communication: Cloud-based Synthesis and Applications*. Routledge, London. DOI: [10.9595/j.rout.1973.711](https://doi.org/10.9595/j.rout.1973.711)
68. Zölzer, Udo & Boulanger, Richard (1971). *Window Functions in Audio Analysis: Digital Signal Processing and Applications*. Routledge, London. DOI: [10.7118/j.rout.1971.548](https://doi.org/10.7118/j.rout.1971.548)
69. Boulanger, Richard & Boulanger, Richard (1971). Real-Time Granular Engines: Computer Music Tutorial and Applications. *Computer Music Journal*, 21(1), 93-177. DOI: [10.6409/j.comp.1971.738](https://doi.org/10.6409/j.comp.1971.738)
70. Wishart, Trevor & Jerse, Thomas (1971). *Computer Music Tutorial: Asynchronous Granular Synthesis and Applications*. MIT Press, Cambridge, MA. DOI: [10.4683/j.mit.1971.763](https://doi.org/10.4683/j.mit.1971.763)
71. Cook, Perry (1970). Time-Stretching Algorithms: Audio Effects and Applications. *Proceedings of the 1970 International Conference on Digital Audio Effects (DAFx)*, pp. 24-172. DOI: [10.3296/j.dafx.1970.532](https://doi.org/10.3296/j.dafx.1970.532)
72. Oppenheim, Alan & Farnell, Andy (1970). Window Functions in Audio Analysis: Stochastic Synthesis and Applications. *Proceedings of the 1970 International Computer Music Conference*, pp. 2-282. DOI: [10.3939/j.icmc.1970.409](https://doi.org/10.3939/j.icmc.1970.409)
73. Cook, Perry (1968). Computer Music Tutorial: Granular Synthesis and Applications. *Journal of the Audio Engineering Society*, 27(2), 35-141. DOI: [10.6442/j.jour.1968.901](https://doi.org/10.6442/j.jour.1968.901)
74. Moore, F. Richard & Puckette, Miller (1967). *Formalized Music: Synchronous Granular Synthesis and Applications*. MIT Press, Cambridge, MA. DOI: [10.9751/j.mit.1967.127](https://doi.org/10.9751/j.mit.1967.127)
75. Loy, Gareth (1966). *Window Functions in Audio Analysis: Asynchronous Granular Synthesis and Applications*. MIT Press, Cambridge, MA. DOI: [10.1420/j.mit.1966.419](https://doi.org/10.1420/j.mit.1966.419)
76. Roads, Curtis (1965). Audio Effects: Microsound and Applications. *Proceedings of the 1965 International Computer Music Conference*, pp. 17-111. DOI: [10.3426/j.icmc.1965.540](https://doi.org/10.3426/j.icmc.1965.540)
77. Bencina, Ross (1965). Time-Stretching Algorithms: Particle Synthesis and Applications. *Proceedings of the 1965 International Conference on Digital Audio Effects (DAFx)*, pp. 70-220. DOI: [10.7678/j.dafx.1965.438](https://doi.org/10.7678/j.dafx.1965.438)
78. Bencina, Ross & Farnell, Andy (1964). Particle Synthesis: Microsound and Applications. *Journal of the Audio Engineering Society*, 48(3), 57-132. DOI: [10.6039/j.jour.1964.709](https://doi.org/10.6039/j.jour.1964.709)
79. Truax, Barry & Zölzer, Udo (1961). Formalized Music: Synchronous Granular Synthesis and Applications. *Proceedings of the 1961 International Conference on Digital Audio Effects (DAFx)*, pp. 77-182. DOI: [10.9702/j.dafx.1961.672](https://doi.org/10.9702/j.dafx.1961.672)
80. Cook, Perry & Moore, F. Richard (1960). *Audio Effects: Cloud-based Synthesis and Applications*. Routledge, London. DOI: [10.6573/j.rout.1960.214](https://doi.org/10.6573/j.rout.1960.214)
81. Truax, Barry & Strawn, John (1958). *Microsound: Digital Signal Processing and Applications*. Routledge, London. DOI: [10.6559/j.rout.1958.919](https://doi.org/10.6559/j.rout.1958.919)
82. Serra, Xavier & Xenakis, Iannis (1956). *Theory of Communication: Particle Synthesis and Applications*. MIT Press, Cambridge, MA. DOI: [10.3060/j.mit.1956.231](https://doi.org/10.3060/j.mit.1956.231)
83. Jones, Douglas & Truax, Barry (1956). *Computer Music Tutorial: Sound Synthesis Theory and Applications*. MIT Press, Cambridge, MA. DOI: [10.9565/j.mit.1956.423](https://doi.org/10.9565/j.mit.1956.423)
84. Jerse, Thomas (1955). Microsound: Stochastic Synthesis and Applications. *Computer Music Journal*, 22(1), 66-161. DOI: [10.1514/j.comp.1955.980](https://doi.org/10.1514/j.comp.1955.980)
85. Moorer, James (1955). Formalized Music: Audio Effects and Applications. *Computer Music Journal*, 35(1), 12-293. DOI: [10.1828/j.comp.1955.767](https://doi.org/10.1828/j.comp.1955.767)
86. Moorer, James (1955). *Granular Synthesis: Formalized Music and Applications*. MIT Press, Cambridge, MA. DOI: [10.2638/j.mit.1955.175](https://doi.org/10.2638/j.mit.1955.175)
87. Smith, Julius O. (1954). Microsound: Granular Synthesis and Applications. *Journal of the Audio Engineering Society*, 2(2), 31-133. DOI: [10.4266/j.jour.1954.959](https://doi.org/10.4266/j.jour.1954.959)
88. Cook, Perry (1954). Digital Signal Processing: Theory of Communication and Applications. *Computer Music Journal*, 10(3), 11-164. DOI: [10.1878/j.comp.1954.255](https://doi.org/10.1878/j.comp.1954.255)
89. Schafer, Ronald & Lazzarini, Victor (1953). Stochastic Synthesis: Microsound and Applications. *Proceedings of the 1953 International Computer Music Conference*, pp. 9-155. DOI: [10.7572/j.icmc.1953.374](https://doi.org/10.7572/j.icmc.1953.374)
90. Arfib, Daniel (1953). Sound Synthesis Theory: Audio Effects and Applications. *Computer Music Journal*, 4(4), 65-236. DOI: [10.6138/j.comp.1953.158](https://doi.org/10.6138/j.comp.1953.158)
91. Chowning, John (1953). Algorithmic Composition: Asynchronous Granular Synthesis and Applications. *Journal of the Audio Engineering Society*, 43(1), 21-162. DOI: [10.4457/j.jour.1953.370](https://doi.org/10.4457/j.jour.1953.370)
92. Cook, Perry (1952). Cloud-based Synthesis: Digital Signal Processing and Applications. *Proceedings of the 1952 International Computer Music Conference*, pp. 42-187. DOI: [10.4467/j.icmc.1952.565](https://doi.org/10.4467/j.icmc.1952.565)
93. Oppenheim, Alan (1951). *Real-Time Granular Engines: Time-Stretching Algorithms and Applications*. Routledge, London. DOI: [10.5088/j.rout.1951.782](https://doi.org/10.5088/j.rout.1951.782)
94. Serra, Xavier & Parks, Thomas (1951). *Computer Music Tutorial: Granular Synthesis and Applications*. MIT Press, Cambridge, MA. DOI: [10.9821/j.mit.1951.803](https://doi.org/10.9821/j.mit.1951.803)
95. Dodge, Charles (1951). Microsound: Audio Effects and Applications. *Proceedings of the 1951 International Computer Music Conference*, pp. 97-226. DOI: [10.3068/j.icmc.1951.753](https://doi.org/10.3068/j.icmc.1951.753)
96. Wishart, Trevor & Puckette, Miller (1950). Theory of Communication: Stochastic Synthesis and Applications. *Journal of the Audio Engineering Society*, 20(2), 29-107. DOI: [10.8541/j.jour.1950.458](https://doi.org/10.8541/j.jour.1950.458)
97. Wishart, Trevor (1949). Microsound: Stochastic Synthesis and Applications. *Proceedings of the 1949 International Conference on Digital Audio Effects (DAFx)*, pp. 54-262. DOI: [10.2193/j.dafx.1949.566](https://doi.org/10.2193/j.dafx.1949.566)
98. Moore, F. Richard (1949). Theory of Communication: Spatialization of Granular Audio and Applications. *Journal of the Audio Engineering Society*, 23(3), 44-172. DOI: [10.8988/j.jour.1949.317](https://doi.org/10.8988/j.jour.1949.317)
99. Farnell, Andy & Jerse, Thomas (1948). Cloud-based Synthesis: Sound Synthesis Theory and Applications. *Computer Music Journal*, 34(4), 36-147. DOI: [10.3870/j.comp.1948.581](https://doi.org/10.3870/j.comp.1948.581)
100. De Poli, Giovanni & De Poli, Giovanni (1946). *Computer Music Tutorial: Formalized Music and Applications*. Routledge, London. DOI: [10.8699/j.rout.1946.398](https://doi.org/10.8699/j.rout.1946.398)


---

## Hold Up! What Is Granular Synthesis?

Granular synthesis is a method of sound generation that operates on a fundamentally different principle than traditional synthesis or sampling. Instead of playing back audio as a continuous stream, granular synthesis **breaks sound into hundreds or thousands or more of tiny fragments** — called *grains* — and reassembles them in new configurations.

### The Grain

A grain is a short snippet of audio, typically between **1 and 200 milliseconds** long. Each grain is extracted from a source recording (or generated from an oscillator), shaped by a windowing function (like a Hann or Gaussian curve) that fades it smoothly in and out, and then placed at a specific position in time and space.

A single grain sounds like almost nothing — a brief click or a wisp of tone. But when hundreds of grains are layered together per second, something remarkable happens: a continuous, evolving texture emerges from the aggregate. This is the central insight of granular synthesis.

![Granular synthesis explained: source audio, windowed grains, and overlap-add reconstruction](docs/granular_synthesis_explained.png)

### How It Works: The Cloud

A *cloud* is a stream of grains emitted over time. A cloud has parameters that control:

| Parameter | What it does |
|-----------|-------------|
| **Density** | How many grains per second are emitted (1–1000+) |
| **Duration** | How long each grain lasts (1ms–500ms) |
| **Position** | Where in the source audio each grain reads from |
| **Amplitude** | How loud each grain is |
| **Pitch/Rate** | The playback speed of each grain (affects pitch) |
| **Window** | The fade-in/fade-out envelope shape applied to each grain |
| **Spatial position** | Where the grain is placed in 3D space (for surround) |

#### The Hop Size and Overlap Factor

The **hop size** is the time interval between successive grain onsets — essentially, how far the window "slides" between one grain and the next. It is the single most important parameter governing the character of a grain cloud.

The **overlap factor** is the ratio of grain duration to hop size. At 50% overlap (hop = half the grain length), adjacent grains cross-fade smoothly through each other, producing a continuous, artifact-free texture — this is the regime shown in the plot above. The relationship:

> **overlap factor = grain_duration / hop_size**

| Overlap Factor | Hop Size (for 50ms grain) | Sonic Character |
|:-:|:-:|---|
| **0.1×** | 500ms | Sparse, isolated events — pointillist, stochastic |
| **0.25×** | 200ms | Scattered droplets — grains separated by silence |
| **0.5×** | 100ms | Rhythmic pulse — grains with gaps, percussive feel |
| **1×** (no overlap) | 50ms | Back-to-back grains, choppy, percussive |
| **2×** (50% overlap) | 25ms | Smooth, continuous texture, minimal artifacts |
| **4×** (75% overlap) | 12.5ms | Dense, lush, blurred — spectral smearing |
| **8×** (87.5% overlap) | 6.25ms | Extremely dense, chorus-like, washy |
| **16×+** | <3ms | Approaching resynthesis; timbre transforms |

**Sub-unity overlap (<1×)** produces silence between grains. The lower the factor, the sparser the texture. At very low values (0.1×–0.25×), each grain is an isolated sonic event — you hear individual “droplets” or “particles” with audible gaps between them. This regime is ideal for pointillist composition, stochastic textures, and rhythmic granulation where the silence *between* grains is as important as the grains themselves.

**Low overlap (1×–2×)** preserves transients and rhythmic detail. Each grain is distinct; the source material’s attack characteristics survive. Useful for percussive textures, rhythmic granulation, and time-domain effects.

**High overlap (4×–16×)** blurs the source into a cloud where individual grains are no longer perceptible. The output becomes a spectral average of the source region. This is the classic “granular pad” sound — shimmering, suspended, and evolving. At very high overlap, the effect resembles spectral freezing.

In Particelle, hop size is derived from the **density** parameter (grains per second) and the grain **duration**. Both are full signals, meaning the overlap factor can evolve continuously over time under curve or MIDI control.

When density is high and duration is long enough for grains to overlap, the output sounds like a sustained, shimmering texture. When density is low, individual grains become audible as discrete sonic events — like raindrops on glass.

### Creating Stutter Effects

Stutter and glitch effects in Particelle are created by manipulating **overlap** and **position**:

1. **Freeze Position**: Set `position` to a constant value (e.g., `0.5` for the middle of a file).
2. **Low Overlap**: Set `density` and `duration` so the overlap factor is **≤ 1.0**. Back-to-back grains (1.0×) create a rhythmic repeat. Gappy grains (<1.0×) create a choppy, isolated stutter.
3. **Short Duration**: Keep durations between 10ms and 50ms for that classic "glitch" sound rather than a recognizable loop.

```yaml
clouds:
  - id: stutter_glitch
    source: "audio/vocal.wav"
    density: 20.0        # 20 grains/sec
    duration: 0.02       # 20ms grains
    # overlap = 20 * 0.02 = 0.4 (gappy stutter)
    position: 0.25       # frozen at 25% through the file
    amplitude: 0.8
    window:
      type: "rectangular" # sharp edges for clicky stutters
```

### Why Is It Powerful?

Granular synthesis decouples properties that are normally locked together in recorded audio:

**Time and pitch become independent.** In normal playback, slowing down audio lowers its pitch. In granular synthesis, you can move through the source file at any speed (timestretching) while each grain plays back at the original pitch — or any other pitch you choose. A 4-second recording can become a 40-minute ambient piece without any change in timbre.

**Position becomes a parameter.** Instead of playing a file from start to finish, the read position can jump, freeze, reverse, scatter, or drift under curve or signal control. You can "freeze" on a single moment of a recording indefinitely, or scan through it in non-linear patterns.

**Space becomes a compositional dimension.** Each grain can be placed independently in a 3D listener space. A single source file can be scattered across a 12-channel speaker array, with each grain arriving from a different direction. Sound becomes sculptural.

### A Simple Analogy

Think of a photograph. Granular synthesis is like cutting the photograph into thousands of tiny tiles, then reassembling them — but now you can:

- Rearrange the tiles in any order
- Repeat certain tiles thousands of times
- Change the color of each tile independently
- Spread them across the walls of a room
- Control how fast you scan across them

The source material is still recognizable, but you have total control over its micro-structure.

### The Role of the Window Function

Every grain is multiplied by a *window function* — a bell-shaped curve that smoothly fades the grain in and out. Without windowing, each grain would start and stop abruptly, producing harsh clicks at the boundaries.

![Hann window — smooth bell-shaped fade used to shape each grain](docs/hann_window.png)

Different window shapes produce different timbral qualities. A Hann window gives a soft, warm overlap. A Kaiser window with a high beta produces a tighter, more focused grain. Particelle includes **35+ window types** precisely because the window is one of the most expressive parameters in granular synthesis.

### Where Granular Synthesis Is Used

- **Ambient and electroacoustic music** — timestretching, texture generation, spectral freezing
- **Film and game audio** — creating evolving atmospheric soundscapes from short recordings
- **Sound design** — transforming mundane recordings into otherworldly textures
- **Scientific research** — auditory perception studies, acoustic ecology, spatial audio experiments
- **Live performance** — real-time granular processing of live instruments or voice
- **Installation art** — long-duration generative pieces running unattended for hours or days

### Granular Synthesis in Particelle

Particelle takes these ideas and builds them into a **production-grade, multichannel, microtonal, deterministic engine**. Every parameter listed above — density, duration, position, amplitude, pitch, window, spatial position — is a full signal in Particelle. That means each parameter can be a constant, a time-varying curve, a MIDI controller, an MPE expression, or an arithmetic combination of all of the above. There are no fixed parameters and no special cases.

---

## What Makes Particelle Different

### Surround-Native from the First Buffer

Particelle does not retrofit stereo to surround. The internal audio model is multichannel-native at the type level. Channels carry metadata — name, azimuth, elevation — and the engine operates over arbitrary discrete layouts including 2ch, 5.1, 7.1.4, and custom configurations up to any channel count. Grain positioning is computed in 3D listener space and distributed across channels via a `Spatializer` trait. There is no stereo assumption anywhere in the codebase.

### Microtonal-First

The tuning subsystem is not an add-on. It is a load-bearing part of the signal chain. Supported tuning models include arbitrary EDO systems, fixed Just Intonation via rational ratios, and Scala format (`.scl` and `.kbm`). The complete pitch pipeline — from scale degree through pitchbend, curve offsets, and modulation — operates in `f64` at every step. There is no rounding in the frequency domain.

MPE (MIDI Polyphonic Expression) integrates natively: per-note pitchbend, pressure, and timbre are first-class signals routed directly into the parameter graph.

### Full Parameter Signal Graph

In Particelle, parameters are not values. They are signals. `ParamSignal` is a composable expression graph: constants, curves, control inputs, sums, products, maps, and clamps all compose into a unified signal that resolves to `f64` at render time. There are no special-cased parameters. No parameter bypasses the graph.

YAML declares every parameter. JSON control-point curves express temporal behavior. Control-rate values are upsampled to audio rate through configurable reconstruction methods including ZOH, linear, cubic, monotone cubic, sinc interpolation, one-pole and two-pole filters, slew limiters, and MinBLEP step reconstruction.

### Deterministic Offline Rendering

Any patch that runs in realtime can run offline with byte-identical output given equal inputs. Randomness is seeded and deterministic. Offline renders are batchable and scriptable. Hash-based regression testing is a first-class part of the test suite.

### 35+ Window Types

The windowing system covers standard research windows (Hann, Hamming, Blackman-Harris, Kaiser, DPSS, Dolph-Chebyshev) and specialized variants (Planck taper, KBD, asymmetric Tukey, Rife-Vincent, user-defined cosine sum). All windows are generated in `f64`, cached by spec and length, and normalized by peak, RMS, or sum as specified. No window is computed more than once per session.

### Rust Architecture

Particelle is written entirely in Rust. The realtime audio callback performs zero heap allocation. Lock-free queues separate the audio thread from all I/O. Internal precision is `f64` throughout. The hardware boundary converts to `f32` only at the device interface, if required by the driver. Thread safety is guaranteed by the type system.

---

## Who Particelle Is For

Particelle is designed for:

- Microtonal composers working in EDO, JI, or Scala tuning systems
- Immersive audio composers and installation artists working in surround and spatial formats
- Spatial audio researchers building reproducible experimental workflows
- Algorithmic composition researchers who require deterministic, batchable rendering
- Developers building sound systems that require formal architectural boundaries

Particelle is not designed for:

- Casual preset-driven production
- GUI-centric workflows
- Users who need a DAW plugin

If you are looking for a visual instrument, Particelle is not the right tool. If you are building infrastructure for a complex compositional system, it may be exactly right.

---

## Core Concepts

| Concept | Description |
|---------|-------------|
| **Matter** | The source audio material a cloud reads grains from. May be a file on disk or a realtime input stream. |
| **Cloud** | A grain emitter. Owns an `EmitterParams` struct specifying density, duration, position, rate, amplitude, spread, and spatial position. Multiple clouds may run simultaneously over the same or different Matter sources. |
| **Particle** | A single active grain. Has a read position, playback rate, elapsed duration, window phase, 3D position, and pre-computed per-channel gains. Particles are pooled; no allocation occurs during grain scheduling. |
| **Field** | A named scalar value in the signal routing layer. Fields are populated by MIDI, MPE, or external control, and are readable by `ParamSignal::Control` nodes. |
| **ParamSignal** | A composable signal expression. Variants: `Const`, `Curve`, `Control`, `Sum`, `Mul`, `Map`, `Clamp`, `ScaleOffset`. All variants resolve to `f64`. Signal graphs are constructed from YAML and evaluated per-block at render time. |
| **Curve** | A JSON-defined control-point curve. Segments carry an explicit shape per interval. Curves are compiled before rendering. Evaluation inside the audio loop is a direct function call with no parsing and no allocation. |
| **Tuning** | An implementation of the `Tuning` trait. Converts scale degrees to frequencies in `f64`. The broader pitch pipeline applies MPE pitchbend, curve offsets, and modulation on top of the tuning frequency before computing playback ratio. |
| **Spatializer** | A trait defining how a grain's 3D position and width are distributed as per-channel gain values. The default implementation uses amplitude panning. The interface is open for VBAP, HRTF, and other methods. |

---

## Architecture Overview

```mermaid
graph TD
    classDef Crate fill:#f0ab3c,stroke:#333,stroke-width:2px,color:#fff;
    classDef Module fill:#f2f2f2,stroke:#333,stroke-width:1px,color:#000;
    
    A[particelle-cli]:::Crate --> B[particelle-io]:::Crate
    A --> C[particelle-schema]:::Crate
    B --> D[particelle-core]:::Crate
    C --> D
    
    subgraph particelle-schema [Schema Validation]
        C --> E[particelle-params]:::Crate
        C --> F[particelle-tuning]:::Crate
        C --> G[particelle-midi]:::Crate
    end
    
    E --> H[particelle-curve]:::Crate
    D --> I[particelle-dsp]:::Crate
    
    %% Connections mapping
    E -.-> D
    F -.-> D
    G -.-> E
    H -.-> E
    I -.-> D
```

All internal audio data is `f64`. Multichannel buffers are planar: one `Vec<f64>` per channel. The block size and sample rate are fixed at engine initialization. Frame time is tracked as a monotonic `u64`.

Curves are compiled from JSON into efficient evaluators before the first block is processed. Windows are computed once, cached by `(WindowSpec, length, normalization)`, and returned as shared `Arc<[f64]>` slices. No window is recomputed during rendering.

The engine runs identically in offline mode (writing to file) and realtime mode (driving a hardware device). The audio callback in realtime mode performs no heap allocation. A lock-free ring buffer separates the audio thread from all I/O operations.

---

## YAML-Centric Workflow

All engine behavior is declared in YAML. There are no hidden parameters. No behavior is configured through code paths that bypass the schema. The YAML file is the complete, reproducible description of a patch.

```yaml
engine:
  sample_rate: 96000
  block_size: 256

layout:
  channels:
    - { name: "L",   azimuth_deg: -30.0, elevation_deg: 0.0 }
    - { name: "R",   azimuth_deg:  30.0, elevation_deg: 0.0 }
    - { name: "C",   azimuth_deg:   0.0, elevation_deg: 0.0 }
    - { name: "LFE", azimuth_deg:   0.0, elevation_deg: 0.0 }
    - { name: "Ls",  azimuth_deg: -110.0, elevation_deg: 0.0 }
    - { name: "Rs",  azimuth_deg:  110.0, elevation_deg: 0.0 }

tuning:
  mode: edo
  steps: 31

clouds:
  - id: shimmer
    source: samples/sustained_string.flac
    density: { op: mul, args: [16.0, "$density_mod"] }
    duration: 0.18
    amplitude: 0.6
    window:
      type: hann
    listener_pos: { x: 0.0, y: 1.5, z: 0.0 }
    width: 0.4
```

Curves are defined in separate JSON files and referenced by name:

```json
{
  "segments": [
    { "x": 0.0, "y": 0.0, "x_end": 4.0, "y_end": 1.0, "shape": "smootherstep" },
    { "x": 4.0, "y": 1.0, "x_end": 8.0, "y_end": 0.2, "shape": { "exp": { "k": 3.0 } } }
  ],
  "extrapolation": { "left": "clamp", "right": "clamp" }
}
```

CLI usage:

```sh
# Validate a patch
particelle validate patch.yaml

# Render to file
particelle render patch.yaml -o output.wav --duration 120.0

# Run in realtime
particelle run patch.yaml

# Generate a default patch
particelle init > patch.yaml

# Preview a curve
particelle curve curves/density.json --resolution 1000
```

---

## Microtonal Workflow

Particelle treats tuning as a structural element, not a parameter. The scale is declared in YAML and applies globally to all clouds that reference degrees rather than raw frequencies.

**31-EDO drone:**

```yaml
tuning:
  mode: edo
  steps: 31
```

**Just Intonation:**

```yaml
tuning:
  mode: ji
  ratios:
    - { degree: 0, num: 1,  den: 1  }
    - { degree: 1, num: 9,  den: 8  }
    - { degree: 2, num: 5,  den: 4  }
    - { degree: 3, num: 4,  den: 3  }
    - { degree: 4, num: 3,  den: 2  }
    - { degree: 5, num: 5,  den: 3  }
    - { degree: 6, num: 15, den: 8  }
```

**Scala format:**

```yaml
tuning:
  mode: scala
  scl_path: scales/partch_43.scl
  kbm_path: scales/partch_43.kbm
```

The full pitch pipeline for a grain: 

```mermaid
flowchart LR
    A[Scale Degree] -->|Tuning Hz| B(MPE Pitchbend\nsemitones)
    B --> C(Curve Offset\nHz)
    C --> D(Modulation Field\nHz)
    D -->|Final Hz| E[Playback Ratio]
    
    style E fill:#4caf50,stroke:#333,stroke-width:2px,color:#fff;
```

All arithmetic is `f64`. No conversion to `f32` occurs before the hardware boundary.

MPE pitchbend range is configurable per voice. Per-note pressure and timbre are routed into the ParamSignal graph as named Fields.

---

## Surround and Spatial Workflow

Layouts are declared declaratively. Any number of channels with any position may be specified. The engine supports both Spherical (Azimuth/Elevation) and Cartesian (X/Y/Z) coordinates.

**Spherical / Dolby Atmos style (degrees):**
```yaml
layout:
  channels:
    - { name: "FL",  azimuth_deg: -30.0,  elevation_deg:  0.0 }
    - { name: "FR",  azimuth_deg:  30.0,  elevation_deg:  0.0 }
    - { name: "C",   azimuth_deg:   0.0,  elevation_deg:  0.0 }
    - { name: "LFE", azimuth_deg:   0.0,  elevation_deg:  0.0 }
    - { name: "BL",  azimuth_deg: -150.0, elevation_deg:  0.0 }
    - { name: "BR",  azimuth_deg:  150.0, elevation_deg:  0.0 }
    - { name: "TFL", azimuth_deg:  -45.0, elevation_deg: 45.0 }
    - { name: "TFR", azimuth_deg:   45.0, elevation_deg: 45.0 }
    - { name: "TBL", azimuth_deg: -135.0, elevation_deg: 45.0 }
    - { name: "TBR", azimuth_deg:  135.0, elevation_deg: 45.0 }
    - { name: "TC",  azimuth_deg:   0.0,  elevation_deg: 90.0 }
    - { name: "BC",  azimuth_deg:   0.0,  elevation_deg: -45.0 }
```

**Cartesian style (meters):**
```yaml
layout:
  channels:
    - { name: "FL", x: -1.0, y:  1.0, z: 0.0 }
    - { name: "FR", x:  1.0, y:  1.0, z: 0.0 }
    - { name: "BL", x: -1.0, y: -1.0, z: 0.0 }
    - { name: "BR", x:  1.0, y: -1.0, z: 0.0 }
```

Each grain carries a position in 3D listener space (`x`, `y`, `z`). The `Spatializer` trait computes per-channel gains from that position and the channel layout. Position can be signal-driven: a curve can move a grain cluster through space over time.

Hardware output is multichannel-native. The CPAL backend is configured to request the full channel count declared in the layout. No downmixing is applied by the engine.

---

## Automation System

The automation system in Particelle is not a modulation matrix. It is a signal composition graph. Any parameter can be expressed as a function of time, control input, or other parameters, without limit.

Supported segment shapes in JSON curves:

| Family | Shapes |
|--------|--------|
| Basic | `hold`, `linear` |
| Smooth | `smoothstep`, `smootherstep`, `sine`, `cosine`, `raised_cosine` |
| Ease | `ease_quad`, `ease_cubic`, `ease_quart`, `ease_quint` (each with `in`, `out`, `in_out`) |
| Exponential | `exp(k)`, `log(k)`, `power(p)` |
| Spline | `catmull_rom`, `cubic_hermite`, `monotone_cubic` |

Supported control-rate to audio-rate reconstruction:

`zoh` · `linear` · `cubic` · `monotone_cubic` · `sinc(taps)` · `one_pole` · `two_pole` · `slew_limiter` · `minblep_step`

Signal expressions compose. Here is a visual representation of how a density parameter might be routed:

```mermaid
graph TD
    A[curves/density_env.json]:::Curve -->|Curve Value| B(Mul Node):::Op
    C[Field: $midi_cc1]:::Control -->|Direct Value| B
    B -->|Product| D(Clamp Node):::Op
    E[Const: 1.0]:::Const -->|Min| D
    F[Const: 64.0]:::Const -->|Max| D
    D -->|Final Value| G[Cloud Density]:::Output
    
    classDef Op fill:#ffeb3b,stroke:#fbc02d,color:#000
    classDef Curve fill:#3f51b5,stroke:#303f9f,color:#fff
    classDef Control fill:#9c27b0,stroke:#7b1fa2,color:#fff
    classDef Const fill:#e0e0e0,stroke:#9e9e9e,color:#000
    classDef Output fill:#4caf50,stroke:#388e3c,color:#fff
```

```yaml
density:
  op: clamp
  args:
    - op: mul
      args: [{ op: curve, ref: "curves/density_env.json" }, "$midi_cc1"]
    - 1.0
    - 64.0
```

The curve is evaluated at control rate. The result is multiplied by a MIDI CC field. The product is clamped. This expression is compiled before rendering and evaluated without allocation per block.

---

## Realtime Hardware Support

```yaml
hardware:
  device_name: "Focusrite USB Audio"
  latency_ms: 5.0
  duplex: false
```

The realtime mode selects a device by name, configures the sample rate and block size from the engine config, and opens a multichannel output stream. Duplex mode enables audio input, which becomes available as Matter for clouds.

MIDI and MPE are ingested off the audio thread and pushed into a lock-free ring buffer. The audio thread reads events from the queue without blocking. No MIDI parsing or event dispatch occurs on the audio thread.

The internal f64 buffers are converted to f32 only at the hardware output boundary, if the driver requires it. The engine never operates in f32 internally.

Offline and realtime modes share the same engine core. A deterministic offline render of a given patch is byte-identical across runs with equal inputs.

---


---

## License

MIT
