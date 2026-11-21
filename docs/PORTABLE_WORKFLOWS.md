# Portable Patches and Cloud Stems

## Make a project someone else can open

```sh
particelle bundle my-patch.yaml -o handoff
particelle verify-bundle handoff
particelle render handoff/patch.yaml -o check.wav --duration 5 --format pcm24
```

`bundle` creates a **new directory** with `patch.yaml`, `assets/`, and `manifest.json`. It copies every source WAV, analysis WAV, curve JSON, and Scala `.scl`/`.kbm` file referenced by the patch. Asset filenames are based on SHA-256 digests; duplicate references are copied once. The patch uses relative paths, so the folder can be moved without depending on the original files or working directory. It normalizes supported legacy patch syntax and does not preserve YAML comments or original formatting. Keep your authored patch separately if those comments matter.

`verify-bundle` checks the files, hashes, lengths, references, and patch validity. It refuses an `INCOMPLETE` bundle. The manifest itself is not self-signed; use a trusted transport or an external signature when authenticity, rather than accidental-change detection, matters. A live `source: input` cannot be bundled. Only bundle source audio you have the right to redistribute.

An existing output directory is never overwritten. A disk or copy failure leaves an `INCOMPLETE` marker so a partial handoff is not mistaken for a finished one.

## Export separate clouds

```sh
particelle render handoff/patch.yaml -o mix.wav --duration 10 \
  --format f32 --stems-dir stems
```

The ordinary mix still goes to `mix.wav`. The new `stems/` directory contains numbered files such as `01-foundation.wav`, one per cloud, and a `manifest.json` mapping files to cloud IDs. Unsafe filename characters in IDs become underscores; the ordinal prevents collisions. Each stem keeps the full declared channel layout, sample rate, duration, and output format, including intentional silent channels. The `--hash` option still reports the mix file only.

Stems are isolated renders using the same patch and control settings. Their sum should track the mix closely, but separate floating-point summation and PCM quantization can change the last bits. Do not promise byte-identical reconstruction after converting to PCM16/24. A failed stem pass leaves an `INCOMPLETE` marker; discard or rerun that directory instead of using a partial delivery.
