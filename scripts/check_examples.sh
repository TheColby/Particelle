#!/usr/bin/env bash
set -euo pipefail

export LANG=C
export LC_ALL=C

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

"$repo_root/scripts/prepare_example_samples.sh"

if ! command -v sox >/dev/null 2>&1; then
  echo "Missing required dependency: sox" >&2
  exit 1
fi

cargo build --release -p particelle-cli

bin="$repo_root/target/release/particelle"
render_dir="$(mktemp -d "${TMPDIR:-/tmp}/particelle-examples.XXXXXX")"
trap 'rm -rf "$render_dir"' EXIT
metrics_dir="$repo_root/target/example-metrics"
mkdir -p "$metrics_dir"

shard_total="${EXAMPLE_SHARD_TOTAL:-1}"
shard_index="${EXAMPLE_SHARD_INDEX:-0}"
clip_suspect_threshold="${CLIP_SUSPECT_THRESHOLD:-0.999999}"
low_rms_threshold="${LOW_RMS_THRESHOLD:-0.00001}"
max_allowed_clip_suspects="${MAX_ALLOWED_CLIP_SUSPECTS:-10}"
max_allowed_low_rms="${MAX_ALLOWED_LOW_RMS:-10}"

if ! [[ "$shard_total" =~ ^[0-9]+$ ]] || (( shard_total < 1 )); then
  echo "EXAMPLE_SHARD_TOTAL must be a positive integer (got '$shard_total')" >&2
  exit 1
fi
if ! [[ "$shard_index" =~ ^[0-9]+$ ]] || (( shard_index < 0 || shard_index >= shard_total )); then
  echo "EXAMPLE_SHARD_INDEX must be in [0, EXAMPLE_SHARD_TOTAL) (got '$shard_index')" >&2
  exit 1
fi

metrics_suffix=""
if (( shard_total > 1 )); then
  metrics_suffix=".shard${shard_index}-of-${shard_total}"
fi
metrics_path="$metrics_dir/examples${metrics_suffix}.tsv"
summary_path="$metrics_dir/summary${metrics_suffix}.md"
printf 'patch\tchannels\tmax_amplitude\trms_amplitude\tcrest_factor\tactive_channels\tchannel_rms\n' >"$metrics_path"

mapfile -t example_files < <(find examples -type f -name '*.yaml' | sort)
selected_count=0
clip_suspects=0
low_rms_count=0

for idx in "${!example_files[@]}"; do
  patch="${example_files[$idx]}"
  if (( idx % shard_total != shard_index )); then
    continue
  fi
  selected_count=$((selected_count + 1))

  "$bin" validate "$patch" >/dev/null 2>&1

  out_name="${patch#examples/}"
  out_path="$render_dir/${out_name//\//__}"
  out_path="${out_path%.yaml}.wav"

  "$bin" render "$patch" -o "$out_path" --duration 1.0 >/dev/null 2>&1

  stats_output="$(sox "$out_path" -n stat 2>&1)"
  max_amp="$(printf '%s\n' "$stats_output" | awk -F': *' '/Maximum amplitude/ { print $2; exit }')"
  rms_amp="$(printf '%s\n' "$stats_output" | awk -F': *' '/RMS.*amplitude/ { print $2; exit }')"

  if [[ -z "$max_amp" || -z "$rms_amp" ]]; then
    echo "Missing amplitude stat for $patch" >&2
    exit 1
  fi

  if ! awk -v amp="$max_amp" 'BEGIN { exit !(amp > 0.0 && amp <= 1.0) }'; then
    echo "Peak amplitude out of range for $patch: $max_amp" >&2
    exit 1
  fi
  if awk -v amp="$max_amp" -v threshold="$clip_suspect_threshold" 'BEGIN { exit !(amp >= threshold) }'; then
    clip_suspects=$((clip_suspects + 1))
  fi

  if ! awk -v rms="$rms_amp" 'BEGIN { exit !(rms > 0.0000001 && rms < 0.95) }'; then
    echo "RMS amplitude out of range for $patch: $rms_amp" >&2
    exit 1
  fi
  if awk -v rms="$rms_amp" -v threshold="$low_rms_threshold" 'BEGIN { exit !(rms < threshold) }'; then
    low_rms_count=$((low_rms_count + 1))
  fi

  crest_factor="$(awk -v peak="$max_amp" -v rms="$rms_amp" 'BEGIN { if (rms <= 0.0) print 0.0; else printf "%.6f", peak / rms }')"
  if ! awk -v crest="$crest_factor" 'BEGIN { exit !(crest >= 1.0 && crest <= 120.0) }'; then
    echo "Crest factor out of range for $patch: $crest_factor" >&2
    exit 1
  fi

  channels="$(soxi -c "$out_path" 2>/dev/null)"
  active_channels=0
  channel_rms_values=()
  for ((channel = 1; channel <= channels; channel++)); do
    channel_stats="$(sox "$out_path" -n remix "$channel" stat 2>&1)"
    channel_rms="$(printf '%s\n' "$channel_stats" | awk -F': *' '/RMS.*amplitude/ { print $2; exit }')"
    if [[ -z "$channel_rms" ]]; then
      echo "Missing channel RMS stat for $patch channel $channel" >&2
      exit 1
    fi
    channel_rms_values+=("$channel_rms")
    if awk -v rms="$channel_rms" 'BEGIN { exit !(rms > 0.0000001) }'; then
      active_channels=$((active_channels + 1))
    fi
  done

  if (( active_channels == 0 )); then
    echo "Silent render for $patch" >&2
    exit 1
  fi

  channel_rms_csv="$(IFS=,; printf '%s' "${channel_rms_values[*]}")"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$patch" \
    "$channels" \
    "$max_amp" \
    "$rms_amp" \
    "$crest_factor" \
    "$active_channels" \
    "$channel_rms_csv" >>"$metrics_path"
done

if (( selected_count == 0 )); then
  echo "Shard ${shard_index}/${shard_total} had no examples to process." >&2
  exit 1
fi

if (( clip_suspects > max_allowed_clip_suspects )); then
  echo "Clip-suspect renders exceed threshold (${clip_suspects} > ${max_allowed_clip_suspects})." >&2
  exit 1
fi

if (( low_rms_count > max_allowed_low_rms )); then
  echo "Low-RMS renders exceed threshold (${low_rms_count} > ${max_allowed_low_rms})." >&2
  exit 1
fi

summary_line="$(
  awk -F'\t' -v rows="$selected_count" -v clips="$clip_suspects" -v low="$low_rms_count" '
    NR==2 {
      min_rms=$4; max_rms=$4; max_peak=$3; max_crest=$5;
    }
    NR>1 {
      if ($4 < min_rms) min_rms=$4;
      if ($4 > max_rms) max_rms=$4;
      if ($3 > max_peak) max_peak=$3;
      if ($5 > max_crest) max_crest=$5;
    }
    END {
      printf "rows=%d min_rms=%s max_rms=%s max_peak=%s max_crest=%s clip_suspects=%d low_rms=%d",
        rows, min_rms, max_rms, max_peak, max_crest, clips, low;
    }
  ' "$metrics_path"
)"

{
  printf '# Example Regression Summary\n\n'
  printf -- '- shard: `%s/%s`\n' "$shard_index" "$shard_total"
  printf -- '- selected_examples: `%s`\n' "$selected_count"
  printf -- '- clip_suspects: `%s` (threshold `%s`)\n' "$clip_suspects" "$max_allowed_clip_suspects"
  printf -- '- low_rms: `%s` (threshold `%s`)\n' "$low_rms_count" "$max_allowed_low_rms"
  printf -- '- stats: `%s`\n' "$summary_line"
} >"$summary_path"

echo "Validated and rendered ${selected_count} example patches (shard ${shard_index}/${shard_total})."
echo "Wrote regression metrics to $metrics_path."
echo "Wrote regression summary to $summary_path."
