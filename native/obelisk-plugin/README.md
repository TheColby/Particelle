# Obelisk M0 native plugin

JUCE VST3 (and AU on macOS) instrument wrapper around the `obelisk-m0` C ABI.
It has a stereo output, accepts MIDI 1.0 channel voice messages including MPE
expression, and renders at MIDI sample offsets. The four host-automatable
controls are Morph, Modulation Hz, Delay Mix, and Reverb Mix. Host programs
provide Init, Glass Pressure, and Low Embers; changing a program updates the
four exposed parameters. Session state stores those parameters and the program
index. The generic JUCE editor exposes the controls.

Build with an installed JUCE CMake package, or a local source checkout:

```sh
JUCE_SOURCE_DIR=/path/to/JUCE ./scripts/build_plugin.sh
```

`CMAKE_PREFIX_PATH` may instead point to a JUCE CMake package. The script
builds the Rust static library and the native formats without downloading JUCE.
Set `OBELISK_PLUGIN_BUILD_DIR` to choose an output directory. Run the wrapper
smoke check with `ctest --test-dir native/obelisk-plugin/build --output-on-failure`.

This build is not distribution-signed or notarized (JUCE may apply an ad-hoc
local signature). The smoke check exercises preset/state
round-trip and offset MIDI-to-audio rendering in-process, not a DAW. VST3 and AU
remain unqualified until tested in named hosts for MIDI/MPE timing, automation,
transport, state restoration, and supported block sizes. MTS-ESP discovery and
host tuning-table integration are not wired into this wrapper.
