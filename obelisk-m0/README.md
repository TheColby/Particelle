# M0 per-note instrument prototype

`obelisk-m0` is an internal codename for the first vertical slice of the
controller-first MPE instrument. It is intentionally a fixed signal chain, not
an application framework.

The prototype currently proves:

- MPE lower and upper zone configuration.
- Expression state sampled before note-on.
- Correct release-channel tracking and detachment on channel reuse.
- Separate member- and master-channel pitch-bend ranges through RPN 0.
- Channel pressure, polyphonic pressure, pitch bend, CC74, velocity, and release velocity.
- Complete Scala `.scl`/`.kbm` application, including negative degrees,
  unmapped keys, formal-degree repetition, reference notes, and reference frequency.
- A deterministic `f32` audio chain using `f64` tuning, phase, frequency, and
  coefficient calculations.
- Wavetable excitation coupled into a sixteen-mode resonant body.
- Pressure-to-brightness and CC74-to-coupling/morph behavior.
- A TPT low-pass filter, per-note pan, and release tails.
- Same-build deterministic offline rendering.
- Audio-thread allocation counting and fixed-size callback timing telemetry.

## Render the built-in MPE performance

```sh
cargo run --release -p obelisk-m0 --bin obelisk-m0-render -- \
  --output /tmp/obelisk-m0.wav \
  --duration 5
```

Render a sustained sixteen-slot stress case:

```sh
cargo run --release -p obelisk-m0 --bin obelisk-m0-render -- \
  --output /tmp/obelisk-m0-stress.wav \
  --duration 3 \
  --stress
```

External MIDI and mapped tuning can be supplied with `--midi`, `--scl`, and
`--kbm`. Run the binary with `--help` for all options.

## Play from an MPE controller

List detected audio and MIDI devices:

```sh
cargo run --release -p obelisk-m0 --bin obelisk-m0-live -- --list
```

Run with the first MIDI input and the system audio output:

```sh
cargo run --release -p obelisk-m0 --bin obelisk-m0-live
```

Use `--midi` and `--audio` to select named devices. The live path uses a
bounded 8192-event lock-free queue. Overflowed expression events are counted;
if a note-off is lost, the audio side receives an emergency all-notes-off so a
queue overflow cannot leave a permanently stuck note.

## Deliberately absent

M0 does not contain a graph compiler, preset schema, GUI, MTS-ESP, effects
suite, plug-in wrapper, or WASM host. Those remain behind the exit gate: the
morph must first sound compelling to MPE players, sustain the voice budget,
and preserve correct controller behavior.
