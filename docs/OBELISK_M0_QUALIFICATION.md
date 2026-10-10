# Obelisk M0 Qualification Matrix

This matrix prevents a portable engine test from being mistaken for DAW or hardware certification.

## Automated Gate

`scripts/qualify_obelisk_m0.sh` is required in GitHub Actions. It runs M0 unit tests, renders a one-second stress WAV, verifies the artifact is non-empty, and asserts that the audio path made zero heap allocations. On macOS, `scripts/build_plugin.sh` builds VST3/AU with local JUCE and `ctest --test-dir native/obelisk-plugin/build --output-on-failure` exercises in-process MIDI/audio and state round-trip. This smoke check is not a DAW test.

## Evidence Required Before Host Release

| Area | Required evidence | Current status |
| --- | --- | --- |
| MPE controller | Per-note pitch, pressure, timbre, note release, and reconnect tested on a named physical controller | Pending physical-device run |
| VST3 host | Note/event timing, transport stop, reset, block sizes, automation, and state restore in a named host/version | Wrapper builds and in-process smoke passes on macOS; named-host run pending |
| AU host | Same coverage in a named AU host/version, including sandbox and state restore | Wrapper builds and in-process smoke passes on macOS; named-host run pending |
| MTS-ESP | Wrapper obtains a host table and passes all 128 values through `obelisk_m0_set_tuning_table` | Core bridge implemented; wrapper discovery/integration not wired |
| Multichannel | Named host accepts 1, 2, and >2 outputs without channel reordering | Core spatial bus implemented; shipped JUCE wrapper is stereo only |

Add dated, reproducible test notes and host/controller versions here before describing any row as qualified in release notes.
