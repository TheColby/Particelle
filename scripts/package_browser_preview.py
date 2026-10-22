#!/usr/bin/env python3
"""Package the tested, static Obelisk M0 browser preview."""
import argparse
import gzip
import hashlib
import io
from pathlib import Path
import re
import tarfile

ROOT = Path(__file__).resolve().parents[1]
FILES = (
    "index.html", "style.css", "app.mjs", "audio.mjs",
    "render-worker.mjs", "README.md", "pkg/obelisk_wasm.wasm",
)


def add_bytes(archive, name, data):
    info = tarfile.TarInfo(name)
    info.size = len(data)
    info.mode = 0o644
    info.mtime = 0
    archive.addfile(info, io.BytesIO(data))


parser = argparse.ArgumentParser()
parser.add_argument("--version", required=True)
parser.add_argument("--web-dir", type=Path, default=ROOT / "web")
parser.add_argument("--out-dir", type=Path, default=ROOT / "dist")
args = parser.parse_args()
if not re.fullmatch(r"nightly|v[0-9]+\.[0-9]+\.[0-9]+(?:-rc\.[1-9][0-9]*)?", args.version):
    parser.error("version must be nightly or a version tag")

web = args.web_dir.resolve()
payload = {}
for relative in FILES:
    path = web / relative
    if not path.is_file() or path.is_symlink():
        parser.error(f"browser preview file missing or not regular: {path}")
    payload[relative] = path.read_bytes()
wasm = payload["pkg/obelisk_wasm.wasm"]
if len(wasm) <= 8 or wasm[:8] != b"\x00asm\x01\x00\x00\x00":
    parser.error("browser preview does not contain a WebAssembly v1 module")

base = f"obelisk-m0-browser-preview_{args.version}"
args.out_dir.mkdir(parents=True, exist_ok=True)
destination = args.out_dir / f"{base}.tar.gz"
with destination.open("wb") as output:
    with gzip.GzipFile(filename="", mode="wb", fileobj=output, mtime=0) as compressed:
        with tarfile.open(fileobj=compressed, mode="w") as archive:
            add_bytes(archive, f"{base}/VERSION", f"{args.version}\n".encode())
            add_bytes(archive, f"{base}/BUNDLE_README.txt", (
                "Obelisk M0 browser preview. No host or cross-browser qualification is claimed.\n"
                "Extract this archive, serve its folder over HTTP, then open it in a browser.\n"
                "For example: python3 -m http.server 8000 --directory .\n"
                "Synthesis runs locally; no audio is uploaded.\n"
            ).encode())
            for relative, data in sorted(payload.items()):
                add_bytes(archive, f"{base}/{relative}", data)
digest = hashlib.sha256(destination.read_bytes()).hexdigest()
(args.out_dir / f"{destination.name}.sha256").write_text(f"{digest}  {destination.name}\n")
print(f"Packaged {destination}")
