#!/usr/bin/env python3
"""Reject a release tag whose published version differs from the CLI and book."""
import argparse
from pathlib import Path
import re
import subprocess
import tomllib

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument("--tag", required=True)
parser.add_argument("--binary", type=Path)
parser.add_argument("--archive", type=Path, action="append", default=[])
args = parser.parse_args()

version = tomllib.loads((root / "Cargo.toml").read_text())["workspace"]["package"]["version"]
if not re.fullmatch(rf"v{re.escape(version)}(?:-rc\.[1-9][0-9]*)?", args.tag):
    parser.error(f"tag {args.tag} does not match workspace version {version}")
manifest = tomllib.loads((root / "Cargo.toml").read_text())
for name, config in manifest["workspace"]["dependencies"].items():
    if isinstance(config, dict) and "path" in config and "version" in config:
        if config["version"] != version:
            parser.error(f"{name} dependency version differs from {version}")
for path, needle in (("docs/USERGUIDE.md", f"{version} CLI release"),
                     ("CHANGELOG.md", f"[{version}]")):
    if needle not in (root / path).read_text():
        parser.error(f"{path} does not identify {version}")
if args.binary:
    reported = subprocess.check_output([str(args.binary.resolve()), "--version"], text=True).strip()
    if reported != f"particelle {version}":
        parser.error(f"CLI reports {reported!r}, expected particelle {version}")
for archive in args.archive:
    if not archive.name.startswith(f"particelle_{args.tag}_") or not archive.name.endswith(".tar.gz"):
        parser.error(f"archive name does not match tag: {archive}")
print(f"Release version consistent: {args.tag} / particelle {version}")
