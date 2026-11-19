"""B04 same-source build comparison; fake builders are host unit fixtures only."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import repro_check as repro


def fixture(build, target, kernel=b"kernel", loader=b"loader"):
    for path, data in [(target / "x86_64-unknown-none/release/os-kernel", kernel),
                       (target / "x86_64-unknown-uefi/release/os-loader.efi", loader),
                       (build / "esp/kernel.elf", kernel),
                       (build / "esp/EFI/BOOT/BOOTX64.EFI", loader)]:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    manifest = {name: {"sha256": hashlib.sha256(data).hexdigest()}
                for name, data in [("kernel", kernel), ("loader", loader)]}
    manifest["source_sha256"] = "frozen"
    (build / "manifest.json").write_text(json.dumps(manifest))


class ReproTests(unittest.TestCase):
    def test_cargo_hardlinked_output_is_read_without_mutation(self):
        def builder(build, target):
            fixture(build, target)
            path = target / "x86_64-unknown-none/release/os-kernel"
            os.link(path, path.with_suffix(".link"))
        with tempfile.TemporaryDirectory() as root:
            report = repro.compare(Path(root), builder=builder, fingerprint=lambda: "frozen")
            self.assertTrue(report["passed"])

    def test_identical_artifacts_in_distinct_directories(self):
        with tempfile.TemporaryDirectory() as root:
            report = repro.compare(Path(root), builder=fixture, fingerprint=lambda: "frozen")
            self.assertTrue(report["passed"])
            self.assertEqual(report["differences"], [])
            self.assertNotEqual(report["runs"][0]["target"], report["runs"][1]["target"])

    def test_real_content_difference_names_kernel_and_esp_payload(self):
        def builder(build, target):
            fixture(build, target, kernel=build.parent.name.encode())
        with tempfile.TemporaryDirectory() as root:
            report = repro.compare(Path(root), builder=builder, fingerprint=lambda: "frozen")
            self.assertFalse(report["passed"])
            self.assertEqual(report["differences"], ["kernel", "image_payload"])

    def test_source_change_fails_before_accepting_artifacts(self):
        stamps = iter(["frozen", "changed"])
        with tempfile.TemporaryDirectory() as root:
            with self.assertRaisesRegex(ValueError, "source changed"):
                repro.compare(Path(root), builder=fixture, fingerprint=lambda: next(stamps))

    def test_build_failure_and_missing_artifacts_are_not_reproducible(self):
        def fail(*args):
            raise ValueError("build exit 7")
        for builder in [fail, lambda *args: None]:
            with tempfile.TemporaryDirectory() as root:
                with self.assertRaises((ValueError, FileNotFoundError)):
                    repro.compare(Path(root), builder=builder, fingerprint=lambda: "frozen")

    def test_esp_or_target_tampering_rejected(self):
        for relative in ["esp/kernel.elf", "target"]:
            def builder(build, target):
                fixture(build, target)
                path = (build / relative if relative != "target" else
                        target / "x86_64-unknown-none/release/os-kernel")
                path.write_bytes(b"different")
            with tempfile.TemporaryDirectory() as root:
                with self.assertRaises(ValueError):
                    repro.compare(Path(root), builder=builder, fingerprint=lambda: "frozen")

    def test_existing_output_never_overwritten(self):
        with tempfile.TemporaryDirectory() as root:
            build = Path(root) / "a/build"
            build.mkdir(parents=True)
            marker = build / "keep"
            marker.write_bytes(b"wip")
            with self.assertRaises(FileExistsError):
                repro.compare(Path(root), builder=fixture, fingerprint=lambda: "frozen")
            self.assertEqual(marker.read_bytes(), b"wip")

    def test_builder_isolates_environment_and_retains_raw_failure(self):
        with tempfile.TemporaryDirectory() as root:
            build, target = Path(root) / "build", Path(root) / "target"
            build.mkdir()
            before = dict(os.environ)
            with patch.object(repro.subprocess, "run", return_value=subprocess.CompletedProcess([], 7)) as run:
                with self.assertRaisesRegex(ValueError, "build exit 7"):
                    repro.build_once(build, target)
            env = run.call_args.kwargs["env"]
            self.assertEqual(env["NOS_BUILD_DIR"], str(build))
            self.assertEqual(env["NOS_TARGET_DIR"], str(target))
            self.assertEqual(env["CARGO_TARGET_DIR"], str(target))
            self.assertEqual(dict(os.environ), before)
            self.assertTrue((build / "repro-build.log").exists())


if __name__ == "__main__":
    unittest.main()
