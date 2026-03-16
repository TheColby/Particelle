#!/usr/bin/env bash
set -euo pipefail

export LANG=C
export LC_ALL=C
export SOX_OPTS="-D"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
samples_dir="$repo_root/samples"
manifest_path="$samples_dir/manifest.sha256"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/particelle-samples.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT

sample_rate=44100
bit_depth=16
duration=1.6

if ! command -v sox >/dev/null 2>&1; then
  echo "Missing required dependency: sox" >&2
  exit 1
fi

hash_file() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{ print $1 }'
  else
    shasum -a 256 "$file" | awk '{ print $1 }'
  fi
}

render_sample() {
  local name="$1"
  local out="$tmp_dir/$name"

  case "$name" in
    A.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 440 sine 660 vol 0.18 fade t 0.01 "$duration" 0.18
      ;;
    B.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 493.88 triangle 739.99 vol 0.18 fade t 0.01 "$duration" 0.18
      ;;
    accordion.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sawtooth 220 sawtooth 330 sine 440 vol 0.08 tremolo 4 35 fade t 0.03 "$duration" 0.22
      ;;
    bass.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 55 sine 82.41 vol 0.22 lowpass 900 fade t 0.01 "$duration" 0.22
      ;;
    bell.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 880 sine 1320 sine 1760 vol 0.06 fade t 0.001 "$duration" 0.9
      ;;
    cello.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sawtooth 130.81 sine 196.0 vol 0.1 lowpass 2400 fade t 0.03 "$duration" 0.25
      ;;
    cello_sustain.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" triangle 146.83 sine 220 sine 293.66 vol 0.08 lowpass 2200 tremolo 5 18 fade t 0.05 "$duration" 0.22
      ;;
    choir.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 261.63 sine 329.63 sine 392.0 vol 0.08 tremolo 5 24 fade t 0.06 "$duration" 0.3
      ;;
    crowd.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" triangle 170 triangle 230 sine 310 vol 0.045 tremolo 9 35 bandpass 900 1.2 fade t 0.02 "$duration" 0.2
      ;;
    drums.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 70 square 140 vol 0.08 lowpass 2200 tremolo 6 45 fade t 0.001 "$duration" 0.4
      ;;
    flute.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 523.25 sine 1046.5 vol 0.12 highpass 180 fade t 0.04 "$duration" 0.2
      ;;
    forest.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" triangle 880 sine 1174.66 sine 1568.0 vol 0.03 tremolo 3 28 highpass 250 lowpass 4200 fade t 0.02 "$duration" 0.25
      ;;
    guitar.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" triangle 110 triangle 164.81 sine 220 vol 0.1 fade t 0.002 "$duration" 0.3
      ;;
    glass_textures.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 1046.5 sine 1567.98 sine 2093.0 vol 0.04 highpass 1200 fade t 0.001 "$duration" 1.0
      ;;
    hihat.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" square 7000 square 9100 vol 0.025 highpass 7000 fade t 0.001 "$duration" 1.45
      ;;
    hit.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 180 square 980 vol 0.045 fade t 0.001 "$duration" 1.3
      ;;
    kick.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 48 sine 72 vol 0.07 lowpass 220 fade t 0.001 "$duration" 1.2
      ;;
    noise_texture.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sawtooth 173 sawtooth 262 square 349 vol 0.03 lowpass 3000 fade t 0.01 "$duration" 0.25
      ;;
    ocean.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 48 triangle 72 vol 0.08 lowpass 500 tremolo 0.4 50 fade t 0.03 "$duration" 0.25
      ;;
    orchestra.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 196 sine 246.94 sine 293.66 sine 392.0 vol 0.06 lowpass 2600 tremolo 5 14 fade t 0.04 "$duration" 0.25
      ;;
    pad.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sawtooth 220 sawtooth 277.18 sine 330 vol 0.07 lowpass 1800 tremolo 2 18 fade t 0.08 "$duration" 0.3
      ;;
    piano.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 261.63 sine 523.25 sine 783.99 vol 0.08 fade t 0.001 "$duration" 0.45
      ;;
    rain.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" square 2200 square 3300 vol 0.02 highpass 1200 fade t 0.02 "$duration" 0.15
      ;;
    rim.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" square 2200 triangle 3300 vol 0.02 highpass 2000 fade t 0.001 "$duration" 1.45
      ;;
    sine_tone.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 1000 vol 0.2 fade t 0.01 "$duration" 0.1
      ;;
    speech.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 140 sine 220 triangle 480 vol 0.04 bandpass 1200 0.9 tremolo 7 28 fade t 0.02 "$duration" 0.18
      ;;
    spring_reverb.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 440 sine 880 vol 0.05 highpass 400 fade t 0.001 "$duration" 1.2
      ;;
    strings.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sawtooth 196 sawtooth 293.66 sine 392.0 vol 0.07 lowpass 2400 tremolo 5 12 fade t 0.05 "$duration" 0.26
      ;;
    synth_blip.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" square 660 sine 1320 vol 0.035 fade t 0.001 "$duration" 1.42
      ;;
    texture.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sawtooth 311.13 triangle 466.16 vol 0.04 lowpass 2000 highpass 160 fade t 0.02 "$duration" 0.2
      ;;
    thunder.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 32 sawtooth 48 vol 0.09 lowpass 240 tremolo 0.8 60 fade t 0.01 "$duration" 0.7
      ;;
    violin.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sawtooth 440 sine 660 vol 0.09 lowpass 3200 tremolo 6 10 fade t 0.03 "$duration" 0.2
      ;;
    vocal.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 220 sine 440 triangle 660 vol 0.04 bandpass 900 1.0 tremolo 6 26 fade t 0.02 "$duration" 0.2
      ;;
    vowel.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" sine 247 sine 494 triangle 740 vol 0.03 bandpass 1400 0.8 fade t 0.03 "$duration" 0.2
      ;;
    white_noise.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" square 1800 square 3200 square 5400 vol 0.015 fade t 0.01 "$duration" 0.15
      ;;
    wind.wav)
      sox -n -r "$sample_rate" -b "$bit_depth" -c 1 "$out" \
        synth "$duration" triangle 140 triangle 280 vol 0.06 tremolo 0.6 45 highpass 200 lowpass 1200 fade t 0.02 "$duration" 0.2
      ;;
    *)
      echo "No recipe defined for sample '$name'" >&2
      exit 1
      ;;
  esac
}

mkdir -p "$samples_dir"

collect_sample_refs() {
  cd "$repo_root"
  if command -v rg >/dev/null 2>&1; then
    rg -No 'samples/[A-Za-z0-9_./-]+\.wav' README.md docs examples -g '*.md' -g '*.yaml'
  else
    find README.md docs examples -type f \( -name '*.md' -o -name '*.yaml' \) -print0 \
      | xargs -0 grep -hoE 'samples/[A-Za-z0-9_./-]+\.wav' || true
  fi
}

mapfile -t sample_names < <(
  collect_sample_refs |
    sed 's/^[^:]*://' |
    sed 's#^samples/##' |
    sort -u
)

if [[ ${#sample_names[@]} -eq 0 ]]; then
  echo "No sample references found." >&2
  exit 1
fi

for sample_name in "${sample_names[@]}"; do
  render_sample "$sample_name"
done

new_manifest="$(mktemp "${TMPDIR:-/tmp}/particelle-manifest.XXXXXX")"
trap 'rm -rf "$tmp_dir" "$new_manifest"' EXIT
{
  for sample_name in "${sample_names[@]}"; do
    printf '%s  %s\n' "$(hash_file "$tmp_dir/$sample_name")" "$sample_name"
  done
} >"$new_manifest"

for sample_name in "${sample_names[@]}"; do
  src="$tmp_dir/$sample_name"
  dst="$samples_dir/$sample_name"
  if [[ ! -f "$dst" ]] || ! cmp -s "$src" "$dst"; then
    mv -f "$src" "$dst"
  fi
done

if [[ ! -f "$manifest_path" ]] || ! cmp -s "$new_manifest" "$manifest_path"; then
  mv -f "$new_manifest" "$manifest_path"
fi

echo "Prepared canonical sample pack (${#sample_names[@]} files)."
