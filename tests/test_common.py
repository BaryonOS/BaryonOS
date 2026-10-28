"""Tests for host command logging failure paths."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import common


class QuantumSourceBinding(unittest.TestCase):
    def test_inherited_package_manifest_and_job_codec_are_source_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            parent = root / "experiments/quantum/Cargo.toml"
            parent.parent.mkdir(parents=True)
            parent.write_text('[workspace.package]\nedition="2021"\n')
            codec = root / "experiments/quantum/job-protocol/src/lib.rs"
            codec.parent.mkdir(parents=True)
            codec.write_text("pub const VERSION: u16 = 1;\n")
            with patch.object(common, "ROOT", root):
                initial = common.source_hash()
                parent.write_text('[workspace.package]\nedition="2024"\n')
                changed_manifest = common.source_hash()
                self.assertNotEqual(initial, changed_manifest)
                codec.write_text("pub const VERSION: u16 = 2;\n")
                self.assertNotEqual(changed_manifest, common.source_hash())


class MachineSelection(unittest.TestCase):
    def test_only_explicit_versioned_profiles_and_no_silent_fallback(self):
        with patch.dict(os.environ, {}, clear=True):
            self.assertEqual(common.qemu_machine(), "pc-q35-10.0")
        for name in common.QEMU_MACHINES:
            with patch.dict(os.environ, {"NOS_QEMU_MACHINE": name}):
                self.assertEqual(common.qemu_machine(), name)
        for name in ("q35", "", "pc-q35-unknown", "pc-q35-7.2,accel=kvm"):
            with self.subTest(name=name):
                with self.assertRaises(ValueError):
                    common.qemu_machine(name)
        with patch.dict(os.environ, {"NOS_QEMU_MACHINE": "invalid"}):
            with self.assertRaises(ValueError):
                common.qemu_machine()
            self.assertEqual(common.qemu_machine("pc-q35-7.2"), "pc-q35-7.2")


class RunLoggedTimeout(unittest.TestCase):
    def test_timeout_saves_partial_output_and_metadata_before_reraising(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            build = root / "build"
            logs = build / "logs"
            logs.mkdir(parents=True)
            (logs / "timeout.stdout.txt").write_text("stale stdout", encoding="utf-8")
            (logs / "timeout.stderr.txt").write_text("stale stderr", encoding="utf-8")
            (logs / "timeout.json").write_text('{"returncode": 0}', encoding="utf-8")
            error = subprocess.TimeoutExpired(
                [sys.executable], 3, output=b"partial stdout\xff", stderr=b"partial stderr"
            )
            with patch.object(common, "BUILD", build), \
                    patch.object(common, "environment", return_value={"PATH": os.environ["PATH"]}), \
                    patch.object(common.subprocess, "run", side_effect=error):
                with self.assertRaises(subprocess.TimeoutExpired) as raised:
                    common.run_logged([sys.executable, "-c", "pass"], "timeout", cwd=root, timeout=3)

            self.assertIs(raised.exception, error)
            self.assertEqual((logs / "timeout.stdout.txt").read_text(encoding="utf-8"), "partial stdout\ufffd")
            self.assertEqual((logs / "timeout.stderr.txt").read_text(encoding="utf-8"), "partial stderr")
            metadata = json.loads((logs / "timeout.json").read_text(encoding="utf-8"))
            self.assertIsNone(metadata["returncode"])
            self.assertTrue(metadata["timed_out"])
            self.assertEqual(metadata["timeout_seconds"], 3)


if __name__ == "__main__":
    unittest.main()
