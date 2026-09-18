# Presets

Particelle presets are source-controlled generators, not opaque binary state. A preset emits a normal editable YAML patch; every parameter can be inspected, versioned, and rendered deterministically.

## Dronoify 1.0.0

`dronoify` transforms any WAV file into an atmospheric drone using three complementary grain clouds: a stable foundation, a diffuse halo, and a shorter moving highlight layer.

```sh
particelle preset dronoify field-recording.wav > dronoify.yaml
particelle render dronoify.yaml -o dronoify.wav --duration 60 --format pcm24
```

The `--channels` switch generates layouts for every count from 1 to 256. Controls are `--density`, `--grain-duration`, `--amplitude`, `--position`, `--width`, `--movement-hz`, `--drift-hz`, and `--window`.

The versioned catalog manifest is [`presets/dronoify/manifest.json`](../presets/dronoify/manifest.json). Package it for distribution with:

```sh
./scripts/package_presets.sh
```
