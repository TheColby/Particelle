# Obelisk M0 Host API

`obelisk-m0` is a fixed-topology MPE instrument. It ships a C ABI and a portable Rust API; DAW-specific VST3/AU wrappers remain separate deliverables because they require host validation and signing.

Build a dynamic or static library:

```sh
cargo build --release -p obelisk-m0
```

The C header is [`obelisk-m0/include/obelisk_m0.h`](../obelisk-m0/include/obelisk_m0.h).

## Realtime Contract

- Create one handle with `obelisk_m0_create(sample_rate, max_voices)`.
- Send complete short MIDI messages with `obelisk_m0_send_midi3` at a block boundary.
- Call `obelisk_m0_process_stereo` with an interleaved `float` buffer of exactly `frames * 2` samples.
- For MTS-ESP-aware wrappers, fetch the host's 128-note table at a block boundary and call `obelisk_m0_set_tuning_table`. Call `obelisk_m0_clear_tuning_table` to resume the configured Scala/KBM tuning. The core intentionally does not discover MTS-ESP or link its SDK.
- Serialize MIDI dispatch and rendering externally. The API does not take locks or allocate while processing audio.
- Call `obelisk_m0_all_notes_off` during transport stop, then `obelisk_m0_destroy` once no audio callback can access the handle.

The M0 wavetable source selects a harmonic band below 45% of Nyquist before interpolation, avoiding bright-source harmonic foldback at high notes. Its sixteen-mode body and tuning calculations remain deterministic.

## Presets, Effects, and Output

`presets/obelisk-m0/*.json` uses the versioned schema in `obelisk_m0::M0Preset`. Parameters cover wavetable/body morph, slow macro modulation, feedback delay, and diffuse reverb. Preset parsing and graph compilation run on the control thread; the audio thread is allocation-free.

`M0Engine::process_block_diffuse` supports one or more equally-sized output channels. Mono is downmixed power-normalized, stereo preserves the normal render, and layouts above two channels receive a power-normalized diffuse field. It is an arbitrary-bus compatibility output, not a claim of speaker-aware spatial rendering.

`render_interleaved_stereo` has no filesystem, device, or thread dependency and is suitable for a WASM wrapper. Build the core without realtime hardware dependencies with:

```sh
cargo build -p obelisk-m0 --no-default-features
```

## Qualification

Run the portable gate locally:

```sh
./scripts/qualify_obelisk_m0.sh
```

It renders an audible WAV and asserts zero audio-thread allocations under stress. Hardware-controller and DAW evidence are tracked separately in [`OBELISK_M0_QUALIFICATION.md`](OBELISK_M0_QUALIFICATION.md); they are not implied by the portable gate.
