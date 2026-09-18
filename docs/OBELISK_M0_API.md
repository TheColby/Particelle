# Obelisk M0 Host API

`obelisk-m0` is a fixed-topology MPE instrument vertical slice. It ships a C ABI for native hosts while the higher-level plug-in-wrapper roadmap remains intentionally separate.

Build a dynamic or static library:

```sh
cargo build --release -p obelisk-m0
```

The C header is [`obelisk-m0/include/obelisk_m0.h`](../obelisk-m0/include/obelisk_m0.h).

## Realtime Contract

- Create one handle with `obelisk_m0_create(sample_rate, max_voices)`.
- Send complete short MIDI messages with `obelisk_m0_send_midi3` at a block boundary.
- Call `obelisk_m0_process_stereo` with an interleaved `float` buffer of exactly `frames * 2` samples.
- Serialize MIDI dispatch and rendering externally. The API does not take locks or allocate while processing audio.
- Call `obelisk_m0_all_notes_off` during transport stop, then `obelisk_m0_destroy` once no audio callback can access the handle.

The M0 wavetable source selects a harmonic band below 45% of Nyquist before interpolation, avoiding bright-source harmonic foldback at high notes. Its sixteen-mode body and tuning calculations remain deterministic.
