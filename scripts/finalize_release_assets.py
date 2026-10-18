#!/usr/bin/env python3
"""Write a single checksum manifest for all unsigned release deliverables."""
import argparse
import hashlib
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("directory", type=Path)
args = parser.parse_args()
root = args.directory.resolve()
excluded = {"SHA256SUMS"}
assets = sorted(p for p in root.iterdir() if p.is_file()
                and p.name not in excluded and not p.name.endswith((".sig", ".pem")))
if not assets or not any(p.name.startswith("particelle_") and p.name.endswith(".tar.gz") for p in assets):
    parser.error("release archives missing")
lines = [f"{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n" for p in assets]
(root / "SHA256SUMS").write_text("".join(lines))
for asset in assets:
    sidecar = root / f"{asset.name}.sha256"
    if sidecar.exists() and sidecar.read_text().split()[0] != hashlib.sha256(asset.read_bytes()).hexdigest():
        parser.error(f"stale checksum: {sidecar}")
print(f"Checksummed {len(assets)} release assets")
