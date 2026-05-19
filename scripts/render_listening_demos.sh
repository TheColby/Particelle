#!/usr/bin/env bash
set -euo pipefail

export LANG=C
export LC_ALL=C

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

catalog_path="${LISTENING_DEMO_CATALOG:-$repo_root/examples/listening_demos.tsv}"
out_dir="${LISTENING_DEMO_OUT_DIR:-$repo_root/target/listening-demos}"
duration_override="${LISTENING_DEMO_DURATION_OVERRIDE:-}"

if [[ ! -f "$catalog_path" ]]; then
  echo "Listening demo catalog not found: $catalog_path" >&2
  exit 1
fi

"$repo_root/scripts/prepare_example_samples.sh"

if ! command -v sox >/dev/null 2>&1; then
  echo "Missing required dependency: sox" >&2
  exit 1
fi

cargo build --release -p particelle-cli
bin="$repo_root/target/release/particelle"

mkdir -p "$out_dir"
metrics_path="$out_dir/metrics.tsv"
playlist_path="$out_dir/playlist.m3u"
manifest_path="$out_dir/README.md"

printf 'id\tpatch\ttitle\tfocus\tduration_s\tchannels\tsample_rate\tseconds\tpeak\tfile\n' >"$metrics_path"
printf '#EXTM3U\n' >"$playlist_path"

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
{
  printf '# Curated Listening Demos\n\n'
  printf -- '- generated_at_utc: `%s`\n' "$generated_at"
  printf -- '- catalog: [`examples/listening_demos.tsv`](/Users/cleider/dev/Particelle/examples/listening_demos.tsv)\n'
  printf -- '- output_dir: `%s`\n' "$out_dir"
  printf '\n'
  printf '| Demo | Patch | Focus | Duration (s) | Output |\n'
  printf '|---|---|---|---:|---|\n'
} >"$manifest_path"

demo_count=0
while IFS=$'\t' read -r id patch duration_s title focus; do
  if [[ "$id" == "id" ]] || [[ -z "$id" ]]; then
    continue
  fi
  if [[ "$id" == \#* ]]; then
    continue
  fi

  if [[ ! -f "$patch" ]]; then
    echo "Patch not found for demo '$id': $patch" >&2
    exit 1
  fi

  render_duration="$duration_s"
  if [[ -n "$duration_override" ]]; then
    render_duration="$duration_override"
  fi

  out_path="$out_dir/${id}.wav"
  "$bin" render "$patch" -o "$out_path" --duration "$render_duration" --pcm24 >/dev/null

  channels="$(soxi -c "$out_path" 2>/dev/null)"
  sample_rate="$(soxi -r "$out_path" 2>/dev/null)"
  seconds="$(soxi -D "$out_path" 2>/dev/null)"
  peak="$(sox "$out_path" -n stat 2>&1 | awk -F': *' '/Maximum amplitude/ { print $2; exit }')"

  if [[ -z "$peak" ]]; then
    echo "Unable to compute peak amplitude for '$id'" >&2
    exit 1
  fi
  if ! awk -v p="$peak" 'BEGIN { exit !(p > 0.0000001) }'; then
    echo "Rendered demo appears silent: '$id' ($out_path)" >&2
    exit 1
  fi

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$id" \
    "$patch" \
    "$title" \
    "$focus" \
    "$render_duration" \
    "$channels" \
    "$sample_rate" \
    "$seconds" \
    "$peak" \
    "$out_path" >>"$metrics_path"

  printf '#EXTINF:%.0f,%s - %s\n%s\n' "$render_duration" "$title" "$focus" "$out_path" >>"$playlist_path"

  printf '| %s | `%s` | %s | %s | `%s` |\n' \
    "$title" \
    "$patch" \
    "$focus" \
    "$render_duration" \
    "$out_path" >>"$manifest_path"

  demo_count=$((demo_count + 1))
done <"$catalog_path"

if (( demo_count == 0 )); then
  echo "No demos were rendered; catalog may be empty." >&2
  exit 1
fi

echo "Rendered ${demo_count} curated listening demos."
echo "Wrote metrics to $metrics_path"
echo "Wrote playlist to $playlist_path"
echo "Wrote manifest to $manifest_path"
