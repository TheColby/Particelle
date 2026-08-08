# Realtime Operations

## Diagnostics

Use a bounded run when collecting a reproducible bug report or validating a device:

```sh
particelle run patch.yaml --duration 60 --telemetry-file telemetry.json
```

The JSON report includes callback count, average and maximum callback time, callback deadline, deadline misses, lock-contention drops, MIDI/OSC control event counts, and maximum pending control batch depth. Deadline misses are the portable underrun-risk signal exposed by the CPAL callback contract. The report is emitted only after a bounded run exits normally.

## Hardware Soak Gate

Hardware output cannot be meaningfully exercised by hosted CI. Run the gate on the target interface:

```sh
./scripts/realtime_soak.sh examples/texture_cloud.yaml --duration 600
```

The default budget permits zero callback deadline misses and zero dropped callbacks. Use explicit `--max-deadline-misses` and `--max-dropped` values only when documenting a known device limitation. The gate writes `target/realtime-soak.json` for attachment to an issue or release qualification record.

The command opens the patch-selected device, or the system default output device. Ensure the patch layout, sample rate, and channel count are supported by that device before starting a soak.
