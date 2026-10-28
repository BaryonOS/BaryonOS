"""A04 evidence-validator regressions; no native guest or binary execution.

Requirements: exact M1/M6b QEMU profile and executable binding, and a complete
immutable 164-file raw inventory. Archived records are fixtures, not new boots.
"""
import copy
import importlib.util
import json
from pathlib import Path, PurePosixPath
import tempfile
import unittest
from unittest import mock


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "docs/verification/m6d-m6b-review/a04_m6b_audit.py"
SPEC = importlib.util.spec_from_file_location("a04_m6b_audit", SCRIPT)
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)
RUNS = json.loads((ROOT / "docs/verification/m6d-m6b-regression/guest-runs.json").read_text(encoding="utf-8"))
RECORDED_ROOT = PurePosixPath(json.loads(
    (ROOT / "docs/verification/m6d-m6b-regression/m6b-preflight.json").read_text(encoding="utf-8"))['cwd'])
M1 = next(record for record in RUNS if record.get("case", "m1") == "m1")
M6 = next(record for record in RUNS if record.get("case") == "normal")
PROGRAM = M6["command"][0]
QEMU_HASH = M6["qemu_sha256"]


class QemuContractTests(unittest.TestCase):
    def validate(self, record, actual_digest=QEMU_HASH):
        # Supplying the archived digest models the validator's hash input; these
        # host tests do not claim access to the original QEMU executable.
        image = record.get("volume_path")
        AUDIT.validate_qemu(
            record,
            PurePosixPath(record["run_directory"]).relative_to(RECORDED_ROOT).as_posix(),
            PROGRAM,
            QEMU_HASH,
            actual_digest,
            image_relative=None if image is None else PurePosixPath(image).relative_to(RECORDED_ROOT).as_posix(),
            has_share=True,
        )

    def test_archived_m1_profile_matches_shared_qemu_binding(self):
        self.assertNotIn("qemu_sha256", M1)
        self.validate(M1)

    def test_all_archived_m6b_profiles_match(self):
        records = [record for record in RUNS if record.get("case", "m1") != "m1"]
        self.assertEqual(len(records), 13)
        for record in records:
            with self.subTest(case=record["case"], run=record["run_directory"]):
                self.validate(record)

    def test_m1_rejects_changed_vm_profile(self):
        for flag, bad in (("-machine", "q35"), ("-cpu", "host"), ("-smp", "2"),
                          ("-m", "512M"), ("-nic", "user")):
            record = copy.deepcopy(M1)
            record["command"][record["command"].index(flag) + 1] = bad
            with self.subTest(flag=flag), self.assertRaisesRegex(AssertionError, "exact isolated QEMU command"):
                self.validate(record)

    def test_m1_rejects_substituted_executable_path(self):
        record = copy.deepcopy(M1)
        record["command"][0] += "-other"
        with self.assertRaisesRegex(AssertionError, "exact isolated QEMU command"):
            self.validate(record)

    def test_m1_rejects_changed_actual_executable_hash(self):
        with self.assertRaisesRegex(AssertionError, "actual QEMU executable hash"):
            self.validate(M1, actual_digest="0" * 64)

    def test_m1_rejects_contradictory_recorded_executable_hash(self):
        record = copy.deepcopy(M1)
        record["qemu_sha256"] = "0" * 64
        with self.assertRaisesRegex(AssertionError, "record QEMU executable hash"):
            self.validate(record)

    def test_m6b_rejects_changed_recorded_executable_hash(self):
        record = copy.deepcopy(M6)
        record["qemu_sha256"] = "0" * 64
        with self.assertRaisesRegex(AssertionError, "record QEMU executable hash"):
            self.validate(record)

    def test_m1_rejects_extra_disk(self):
        record = copy.deepcopy(M1)
        record["command"] += ["-drive", "file=/dev/sda"]
        with self.assertRaisesRegex(AssertionError, "exact isolated QEMU command"):
            self.validate(record)


class RawInventoryTests(unittest.TestCase):
    def setUp(self):
        self.manifest_bytes = AUDIT.RAW_INVENTORY_PATH.read_bytes()
        self.expected = AUDIT.decode_raw_inventory(self.manifest_bytes)
        self.vars_path = "build/run/m1-after-m6b-421b383906/OVMF_VARS.fd"

    def test_complete_historical_inventory_matches(self):
        self.assertEqual(len(self.expected), 164)
        self.assertIn(self.vars_path, self.expected)
        AUDIT.validate_raw_inventory(dict(self.expected), self.expected)

    def test_missing_m1_firmware_state_is_rejected(self):
        actual = dict(self.expected)
        del actual[self.vars_path]
        with self.assertRaisesRegex(AssertionError, "exact raw-file inventory"):
            AUDIT.validate_raw_inventory(actual, self.expected)

    def test_renamed_file_is_rejected_even_when_count_matches(self):
        actual = dict(self.expected)
        actual[self.vars_path + ".replacement"] = actual.pop(self.vars_path)
        self.assertEqual(len(actual), 164)
        with self.assertRaisesRegex(AssertionError, "exact raw-file inventory"):
            AUDIT.validate_raw_inventory(actual, self.expected)

    def test_changed_raw_bytes_are_rejected(self):
        actual = dict(self.expected)
        actual[self.vars_path] = "0" * 64
        with self.assertRaisesRegex(AssertionError, "exact raw-file SHA-256 values"):
            AUDIT.validate_raw_inventory(actual, self.expected)

    def test_extra_raw_file_is_rejected(self):
        actual = dict(self.expected)
        actual["build/run/unexpected.txt"] = "0" * 64
        with self.assertRaisesRegex(AssertionError, "exact raw-file inventory"):
            AUDIT.validate_raw_inventory(actual, self.expected)

    def test_modified_expected_manifest_is_rejected(self):
        modified = dict(self.expected)
        del modified[self.vars_path]
        with self.assertRaisesRegex(AssertionError, "pinned historical raw inventory hash"):
            AUDIT.decode_raw_inventory(json.dumps(modified).encode("utf-8"))

    def test_output_cannot_replace_expected_manifest(self):
        prefix = AUDIT.RAW_INVENTORY_PATH.with_name("a04_m6b")
        with self.assertRaisesRegex(AssertionError, "cannot overwrite historical raw inventory"):
            AUDIT.validate_output_prefix(prefix)

    def test_separate_output_prefix_is_allowed(self):
        with tempfile.TemporaryDirectory() as directory:
            AUDIT.validate_output_prefix(Path(directory) / "audit")

    def test_output_hardlink_cannot_replace_expected_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            protected = Path(directory) / "expected.json"
            protected.write_text("fixed fixture", encoding="utf-8")
            prefix = Path(directory) / "audit"
            Path(str(prefix) + "_raw_hashes.json").hardlink_to(protected)
            with mock.patch.object(AUDIT, "RAW_INVENTORY_PATH", protected):
                with self.assertRaisesRegex(AssertionError, "cannot overwrite historical raw inventory"):
                    AUDIT.validate_output_prefix(prefix)


if __name__ == "__main__":
    unittest.main()
