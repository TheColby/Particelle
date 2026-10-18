#!/usr/bin/env bash
set -euo pipefail

tag="${1:?release tag required}"
target="${2:?target required}"
archive_dir="${3:-dist}"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
archive_dir="$(cd "$archive_dir" && pwd)"
asset="particelle_${tag}_${target}.tar.gz"
[[ -s "$archive_dir/$asset" ]] || { echo "Missing $asset" >&2; exit 1; }

work="$(mktemp -d "${TMPDIR:-/tmp}/particelle-installer-test.XXXXXX")"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/shim" "$work/assets" "$work/bin"
cp "$archive_dir/$asset" "$work/assets/$asset"
python3 - "$work/assets/$asset" "$work/assets/SHA256SUMS" <<'PY'
import hashlib
from pathlib import Path
import sys
asset = Path(sys.argv[1])
Path(sys.argv[2]).write_text(f"{hashlib.sha256(asset.read_bytes()).hexdigest()}  {asset.name}\n")
PY
cat > "$work/shim/curl" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
out=''
url=''
while [[ $# -gt 0 ]]; do
  case "$1" in
    -o) out="$2"; shift 2 ;;
    -*) shift ;;
    *) url="$1"; shift ;;
  esac
done
[[ -n "$out" && -n "$url" ]] || exit 1
cp "$PARTICELLE_TEST_ASSET_DIR/$(basename "$url")" "$out"
SH
chmod +x "$work/shim/curl"
PATH="$work/shim:$PATH" PARTICELLE_TEST_ASSET_DIR="$work/assets" \
  "$repo_root/install.sh" --channel stable --version "$tag" \
  --install-dir "$work/bin" --skip-signature-verify
version="${tag#v}"
version="${version%-rc.*}"
[[ "$("$work/bin/particelle" --version)" == "particelle $version" ]]
echo "Local installer archive test passed for $asset"
