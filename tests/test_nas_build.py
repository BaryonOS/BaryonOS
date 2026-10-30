"""Negative artifact-binding checks; these mocks do not execute guest code."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import build_nas


class NasBuildTests(unittest.TestCase):
    def setUp(self):
        root = build_nas.ROOT / "build"
        root.mkdir(exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(prefix="nas-build-test-", dir=root)
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.build = self.base / "build"
        self.output = self.build / "users/nas"
        self.target = self.base / "target"
        self.plan = {"entry": 0x40000000, "segments": [{"flags": 5}]}
        self.mutation = None
        for name, value in (
            ("BUILD", self.build),
            ("TARGET", self.target),
            ("inspect_user", lambda _: self.plan),
            ("run_logged", self.fake_build),
        ):
            context = patch.object(build_nas, name, value)
            context.start()
            self.addCleanup(context.stop)

    def fake_build(self, command, label):
        self.assertEqual(label, "build-nas-users")
        self.assertEqual(command[-4:], ["--target-dir", str(self.target / "nas-native"),
                                        "--output-dir", str(self.output)])
        self.output.mkdir(parents=True, exist_ok=True)
        binaries = {}
        for name in ("nas-probe", "nas-volume"):
            data = name.encode()
            (self.output / f"{name}.elf").write_bytes(data)
            binaries[name] = {
                "sha256": hashlib.sha256(data).hexdigest(),
                "bytes": len(data),
                "elf": self.plan,
            }
        report = {"gate": "crosscompile-and-elf-inspection-only",
                  "native_execution": False, "binaries": binaries}
        if self.mutation:
            self.mutation(report)
        (self.output / "build-report.json").write_text(json.dumps(report))

    def test_existing_four_slots_bind_only_the_two_inspected_binaries(self):
        bindings, report = build_nas.build_users()
        probe = str(self.output / "nas-probe.elf")
        self.assertEqual(bindings, {
            "NOS_SYSTEM_INIT_ELF": probe,
            "NOS_SYSTEM_APP_ELF": probe,
            "NOS_KEYRING_SERVICE_ELF": probe,
            "NOS_VOLUME_SERVICE_ELF": str(self.output / "nas-volume.elf"),
        })
        self.assertIs(report["native_execution"], False)

    def test_digest_size_and_load_plan_tampering_are_rejected(self):
        for field, value in (("sha256", "0" * 64), ("bytes", 1), ("elf", {})):
            with self.subTest(field=field):
                self.mutation = lambda report: report["binaries"]["nas-probe"].update({field: value})
                with self.assertRaisesRegex(ValueError, "differs from actual ELF"):
                    build_nas.build_users()

    def test_unearned_native_claim_or_wrong_gate_is_rejected(self):
        for field, value in (("native_execution", True), ("native_execution", 0), ("gate", "native")):
            with self.subTest(field=field, value=value):
                self.mutation = lambda report: report.update({field: value})
                with self.assertRaisesRegex(ValueError, "must not claim native"):
                    build_nas.build_users()

    def test_preexisting_symlink_is_rejected_before_builder_runs(self):
        self.output.mkdir(parents=True)
        victim = self.base / "unrelated"
        victim.write_bytes(b"unchanged")
        (self.output / "nas-volume.elf").symlink_to(victim)
        with patch.object(build_nas, "run_logged") as builder:
            with self.assertRaisesRegex(ValueError, "Symlink"):
                build_nas.build_users()
            builder.assert_not_called()
        self.assertEqual(victim.read_bytes(), b"unchanged")


if __name__ == "__main__":
    unittest.main()
