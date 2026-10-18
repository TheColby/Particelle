#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
out_dir="${1:-$repo_root/dist}"
mkdir -p "$out_dir"
cargo metadata --format-version 1 --no-deps > "$out_dir/particelle.sbom.cargo-metadata.json"
python3 - "$out_dir" "$(git rev-parse HEAD)" <<'PY'
import datetime
import hashlib
import json
from pathlib import Path
import sys

root = Path(sys.argv[1])
files = sorted(p for p in root.iterdir() if p.is_file() and p.name not in
               ('SHA256SUMS', 'particelle.provenance.json') and
               not p.name.endswith(('.sig', '.pem')))
record = {
    'schema_version': 1,
    'project': 'particelle',
    'revision': sys.argv[2],
    'generated_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'artifacts': [{'path': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in files],
}
(root / 'particelle.provenance.json').write_text(json.dumps(record, indent=2) + '\n')
PY
echo "Wrote SBOM and provenance metadata to $out_dir"
