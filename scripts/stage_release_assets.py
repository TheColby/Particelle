#!/usr/bin/env python3
"""Flatten release payloads, refusing duplicate names or unrecognized files."""
import argparse
from pathlib import Path
import shutil

parser = argparse.ArgumentParser()
parser.add_argument("directory", type=Path)
args = parser.parse_args()
root = args.directory.resolve()
for path in sorted(root.rglob("*")):
    if not path.is_file() or path.parent == root:
        continue
    if path.suffix not in (".gz", ".sha256"):
        parser.error(f"unexpected nested release file: {path}")
    target = root / path.name
    if target.exists():
        parser.error(f"duplicate asset name: {target.name}")
    shutil.move(path, target)
for directory in sorted((p for p in root.rglob("*") if p.is_dir()), reverse=True):
    directory.rmdir()
print(f"Staged {len(list(root.iterdir()))} release files")
