"""Release-packaging regression tests; run with python3 -m unittest."""
import hashlib
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class ReleasePipelineTests(unittest.TestCase):
    def run_script(self, script, *args):
        return subprocess.run([sys.executable, str(ROOT / "scripts" / script), *map(str, args)],
                              capture_output=True, text=True)

    def test_release_version_accepts_rc_and_rejects_old_tag(self):
        self.assertEqual(self.run_script("check_release_version.py", "--tag", "v0.3.0-rc.1").returncode, 0)
        self.assertNotEqual(self.run_script("check_release_version.py", "--tag", "v0.2.3").returncode, 0)

    def test_stage_and_manifest_cover_nested_assets_and_sidecars(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            nested = root / "presets"
            nested.mkdir()
            archive = root / "particelle_v0.3.0_x86_64-unknown-linux-gnu.tar.gz"
            archive.write_bytes(b"archive")
            (nested / "particelle-presets-1.0.0.tar.gz").write_bytes(b"preset")
            (root / f"{archive.name}.sha256").write_text(
                f"{hashlib.sha256(archive.read_bytes()).hexdigest()}  {archive.name}\n")
            self.assertEqual(self.run_script("stage_release_assets.py", root).returncode, 0)
            self.assertTrue((root / "particelle-presets-1.0.0.tar.gz").exists())
            self.assertEqual(self.run_script("finalize_release_assets.py", root).returncode, 0)
            manifest = (root / "SHA256SUMS").read_text()
            for name in (archive.name, f"{archive.name}.sha256", "particelle-presets-1.0.0.tar.gz"):
                self.assertIn(name, manifest)
            (root / f"{archive.name}.sha256").write_text("0" * 64 + f"  {archive.name}\n")
            self.assertNotEqual(self.run_script("finalize_release_assets.py", root).returncode, 0)

    def test_stage_rejects_duplicate_names(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "same.tar.gz").write_bytes(b"a")
            (root / "nested").mkdir()
            (root / "nested" / "same.tar.gz").write_bytes(b"b")
            self.assertNotEqual(self.run_script("stage_release_assets.py", root).returncode, 0)


if __name__ == "__main__":
    unittest.main()
