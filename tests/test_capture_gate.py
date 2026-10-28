"""Capture-gate tests use synthetic evidence and never claim a guest run."""
import copy
import hashlib
import json
import sys
import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import capture_gate
import m5_disk
from gate_contract import guest_expectation
from m5_evidence import two_image_evidence, validate_two_image_evidence


class CaptureGateArchive(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.build = self.root / "build"
        self.destination = self.root / "docs/verification/m2-m4/m2"
        self.old_root = capture_gate.ROOT
        self.old_build = capture_gate.BUILD
        capture_gate.ROOT = self.root
        capture_gate.BUILD = self.build
        self.addCleanup(setattr, capture_gate, "ROOT", self.old_root)
        self.addCleanup(setattr, capture_gate, "BUILD", self.old_build)
        self.destination.mkdir(parents=True)
        (self.destination / "host.stdout.txt").write_text("old host output\n", encoding="utf-8")
        (self.destination / "suite.json").write_text('{"passed": true}\n', encoding="utf-8")
        (self.destination / "guest-runs.json").write_text('[{"passed": true}]\n', encoding="utf-8")
        (self.destination / "preserved.txt").write_text("previous archive\n", encoding="utf-8")
        self._write_suite()

    def _write_suite(self, *, valid_hash=False):
        host = self.build / "logs/host.stdout.txt"
        host.parent.mkdir(parents=True, exist_ok=True)
        host.write_text("new host output\n", encoding="utf-8")
        host.with_name("host.json").write_text('{"returncode": 0}\n', encoding="utf-8")
        host.with_name("host.stderr.txt").write_text("", encoding="utf-8")

        guest = self.build / "run/guest-1"
        guest.mkdir(parents=True, exist_ok=True)
        required, expected_exit, forbidden = guest_expectation("m2", "test-m2-nx")
        stdout = "\n".join(required) + "\n"
        (guest / "serial.txt").write_text(stdout, encoding="utf-8")
        (guest / "stderr.txt").write_text("", encoding="utf-8")
        artifact = guest / "artifact.bin"
        artifact.write_bytes(b"synthetic artifact")
        digest = hashlib.sha256(artifact.read_bytes()).hexdigest() if valid_hash else "0" * 64
        result = {
            "stdout": stdout,
            "stderr": "",
            "raw_qemu_exit": expected_exit,
            "timed_out": False,
            "required_markers": list(required),
            "expected_exit": expected_exit,
            "forbidden_markers": list(forbidden),
            "artifact_sha256": {"artifact.bin": digest},
        }
        (guest / "result.json").write_text(json.dumps(result), encoding="utf-8")
        suite = {
            "status": "completed",
            "passed": True,
            "results": [
                {"name": "host-tests", "passed": True, "evidence": str(host)},
                {"name": "test-m2-nx", "passed": True, "evidence": str(guest)},
            ],
        }
        self.build.mkdir(parents=True, exist_ok=True)
        (self.build / "m2-test-results.json").write_text(json.dumps(suite), encoding="utf-8")

    def _snapshot(self):
        return {path.relative_to(self.destination).as_posix(): path.read_bytes()
                for path in self.destination.rglob("*") if path.is_file()}

    def test_validation_failure_leaves_existing_archive_unchanged(self):
        before = self._snapshot()

        with self.assertRaisesRegex(ValueError, "Artifact changed"):
            capture_gate.capture("m2")

        self.assertEqual(self._snapshot(), before)

    def test_valid_capture_replaces_archive_only_after_validation(self):
        self._write_suite(valid_hash=True)

        capture_gate.capture("m2")

        self.assertEqual((self.destination / "host.stdout.txt").read_text(encoding="utf-8"),
                         "new host output\n")
        self.assertFalse((self.destination / "preserved.txt").exists())
        self.assertEqual(json.loads((self.destination / "suite.json").read_text())["status"],
                         "completed")
        self.assertEqual(len(json.loads((self.destination / "guest-runs.json").read_text())), 1)

    def test_guest_output_cannot_be_captured_as_a_host_log(self):
        guest_log = self.build / "run/guest.stdout.txt"
        guest_log.parent.mkdir(parents=True, exist_ok=True)
        guest_log.write_text("unverified guest output\n", encoding="utf-8")
        guest_log.with_name("guest.json").write_text('{"returncode": 0}\n', encoding="utf-8")
        guest_log.with_name("guest.stderr.txt").write_text("", encoding="utf-8")
        suite_path = self.build / "m2-test-results.json"
        suite = json.loads(suite_path.read_text(encoding="utf-8"))
        suite["results"][0]["evidence"] = str(guest_log)
        suite_path.write_text(json.dumps(suite), encoding="utf-8")
        before = self._snapshot()

        with self.assertRaises(ValueError):
            capture_gate.capture("m2")

        self.assertEqual(self._snapshot(), before)

    def test_guest_cases_cannot_be_substituted_with_a_valid_host_log(self):
        source_suite = json.loads(
            (self.build / "m2-test-results.json").read_text(encoding="utf-8"))
        for phase, group_name in (("m2", "test-m2-nx"), ("m4", "test-m4-copy")):
            suite_path = self.build / f"{phase}-test-results.json"
            suite = json.loads(json.dumps(source_suite))
            suite["results"][0]["name"] = group_name
            suite["results"][0]["evidence"] = str(self.build / "logs/host.stdout.txt")
            suite["results"].pop()
            suite_path.write_text(json.dumps(suite), encoding="utf-8")
            before = self._snapshot()

            with self.assertRaisesRegex(ValueError, "Guest case requires run-directory evidence"):
                capture_gate.capture(phase)

            self.assertEqual(self._snapshot(), before)

    def test_guest_evidence_is_bound_to_phase_and_case_not_record_expectations(self):
        record = {
            "stdout": "BOOT:EBS_OK\nKERNEL:ENTRY\nBOOTINFO:OK\nM1:PASS\n",
            "raw_qemu_exit": 33,
            "timed_out": False,
            "required_markers": ["M1:PASS"],
            "expected_exit": 33,
            "forbidden_markers": [],
        }

        self.assertFalse(capture_gate.guest_matches("m2", "test-m2-nx", record))
        self.assertFalse(capture_gate.guest_matches("m4", "test-m4-copy", record))
        self.assertFalse(capture_gate.guest_matches("m4", "unknown-case", record))

    def test_guest_record_cannot_omit_raw_timeout_status(self):
        record = {
            "stdout": "BOOT:EBS_OK\nKERNEL:ENTRY\nBOOTINFO:OK\nM1:PASS\n",
            "raw_qemu_exit": 33,
        }

        self.assertFalse(capture_gate.guest_matches("m2", "test-m2-nx", record))

    def test_guest_exit_status_is_bound_to_the_phase_and_case(self):
        required, _, _ = guest_expectation("m4", "test-m4-copy")
        record = {
            "stdout": "\n".join(required),
            "raw_qemu_exit": 35,
            "timed_out": False,
            "required_markers": required,
            "expected_exit": 35,
            "forbidden_markers": [],
        }

        self.assertFalse(capture_gate.guest_matches("m4", "test-m4-copy", record))

    def test_m4_case_requires_its_specific_markers(self):
        result = {
            "raw_qemu_exit": 33,
            "stdout": "\n".join(("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK",
                                 "M4:CASE:COPY:PASS", "M4:SURVIVOR_PROGRESS:PASS",
                                 "M4:CLEANUP:PASS", "KS-09:PASS", "M4:PASS", "M1:PASS")),
            "timed_out": False,
            "required_markers": ["M1:PASS"],
            "expected_exit": 33,
            "forbidden_markers": [],
        }

        self.assertTrue(capture_gate.guest_matches("m4", "test-m4-copy", result))

    def test_two_image_booleans_without_guest_binding_cannot_be_captured(self):
        evidence = self.build / "logs/m5-two-image-check.json"
        evidence.write_text(json.dumps({"checks": {"same_kernel": True}, "passed": True}))
        suite = {"status": "completed", "passed": True, "results": [{
            "name": "same-binary-two-actual-disks", "passed": True,
            "evidence": str(evidence)}]}
        (self.build / "m5-test-results.json").write_text(json.dumps(suite))
        before = self._snapshot()

        with self.assertRaisesRegex(ValueError, "requires exactly one guest record"):
            capture_gate.capture("m5", str(self.destination))

        self.assertEqual(self._snapshot(), before)

    def test_m5_comparison_cannot_be_omitted_duplicated_or_replaced_by_host_log(self):
        host_group = {"name": "same-binary-two-actual-disks", "passed": True,
                      "evidence": str(self.build / "logs/host.stdout.txt")}
        for groups in ([], [host_group], [host_group, host_group]):
            with self.subTest(count=len(groups)):
                suite = {"status": "completed", "passed": True, "results": groups}
                (self.build / "m5-test-results.json").write_text(json.dumps(suite))
                before = self._snapshot()
                with self.assertRaisesRegex(ValueError, "M5 .*comparison"):
                    capture_gate.capture("m5", str(self.destination))
                self.assertEqual(self._snapshot(), before)

    def test_m5_capture_binds_comparison_to_verified_disks_and_records(self):
        guests = []
        for index, label in enumerate(("m5-read-a", "m5-read-b")):
            directory = self.build / "run" / f"{label}-0123456789"
            directory.mkdir()
            with patch.object(m5_disk, "BUILD", self.build):
                disk, oracle = m5_disk.create_disk(directory, seed=index)
            stdout = "\n".join(capture_gate.MILESTONE_MARKERS["m5"]) + "\n" + "\n".join(
                f"M5:SECTOR{sector}_DIGEST:{digest}"
                for sector, digest in oracle["expected_digests"].items()) + "\n"
            artifacts = {"test-disk.img": hashlib.sha256(disk.read_bytes()).hexdigest()}
            for name in ("esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI"):
                artifact = directory / name
                artifact.parent.mkdir(parents=True, exist_ok=True)
                artifact.write_bytes(b"synthetic artifact")
                artifacts[name] = hashlib.sha256(artifact.read_bytes()).hexdigest()
            (directory / "serial.txt").write_text(stdout)
            (directory / "stderr.txt").write_text("")
            guest = {"label": label, "run_directory": str(directory), "stdout": stdout,
                     "stderr": "", "raw_qemu_exit": 33, "timed_out": False, "passed": True,
                     "m5_case": "read", "base_commit": "1" * 40,
                     "source_fingerprint_version": 3, "source_sha256": "2" * 64,
                     "artifact_sha256": artifacts,
                     "disk_oracle": m5_disk.verify_disk(disk, oracle, stdout, data_required=True)}
            (directory / "result.json").write_text(json.dumps(guest))
            guests.append(guest)
        comparison = two_image_evidence(guests)
        evidence = self.build / "logs/m5-two-image-check.json"
        evidence.write_text(json.dumps(comparison))
        suite = {"status": "completed", "passed": True, "results": [{
            "name": "same-binary-two-actual-disks", "passed": True,
            "evidence": str(evidence)}] + [{"name": guest["label"], "passed": True,
                                          "evidence": guest["run_directory"]} for guest in guests]}
        (self.build / "m5-test-results.json").write_text(json.dumps(suite))

        capture_gate.capture("m5", str(self.destination))

        self.assertEqual(json.loads((self.destination / "m5-two-image-check.json").read_text()),
                         comparison)
        self.assertEqual(len(json.loads((self.destination / "guest-runs.json").read_text())), 2)

    def test_archive_install_error_restores_the_previous_archive(self):
        staged = self.root / "staged"
        staged.mkdir()
        (staged / "suite.json").write_text('{"passed": false}\n', encoding="utf-8")
        before = self._snapshot()
        real_replace = capture_gate.os.replace

        def fail_install(source, target):
            if Path(source) == staged and Path(target) == self.destination:
                raise OSError("synthetic archive install failure")
            return real_replace(source, target)

        with patch.object(capture_gate.os, "replace", side_effect=fail_install):
            with self.assertRaisesRegex(OSError, "synthetic archive install failure"):
                capture_gate._replace_archive(staged, self.destination)

        self.assertEqual(self._snapshot(), before)
        self.assertTrue(staged.is_dir())


class M5TwoImageBinding(unittest.TestCase):
    """Synthetic record checks enforce M5-ORACLE; these are no native VM runs."""

    def setUp(self):
        self.guests = []
        for label, digit in (("m5-read-a", "a"), ("m5-read-b", "b")):
            digests = {"4": digit * 16, "6": digit * 16}
            self.guests.append({
                "label": label, "run_directory": f"/temporary/build/run/{label}-0123456789",
                "passed": True, "m5_case": "read", "raw_qemu_exit": 33, "timed_out": False,
                "stdout": "\n".join(capture_gate.MILESTONE_MARKERS["m5"]) + "\n" +
                          "\n".join(f"M5:SECTOR{sector}_DIGEST:{value}"
                                    for sector, value in digests.items()) + "\n",
                "base_commit": "1" * 40, "source_fingerprint_version": 3,
                "source_sha256": "2" * 64,
                "artifact_sha256": {"esp/kernel.elf": "3" * 64,
                                    "esp/EFI/BOOT/BOOTX64.EFI": "4" * 64,
                                    "test-disk.img": digit * 64},
                "disk_oracle": {"passed": True, "sha256_before": digit * 64,
                                "sha256_after": digit * 64, "expected_digests": digests,
                                "observed_digests": digests},
            })

    def test_matching_records_preserve_values_under_stable_run_ids(self):
        evidence = two_image_evidence(self.guests)
        validate_two_image_evidence(evidence, self.guests)
        self.assertTrue(evidence["passed"])
        self.assertEqual(evidence["runs"], ["m5-read-a-0123456789", "m5-read-b-0123456789"])
        self.assertEqual(evidence["provenance"][0]["artifact_sha256"],
                         self.guests[0]["artifact_sha256"])

    def test_missing_or_duplicate_guest_is_rejected(self):
        for guests in (self.guests[:1], self.guests + self.guests[:1]):
            with self.subTest(count=len(guests)):
                with self.assertRaisesRegex(ValueError, "requires exactly one guest record"):
                    two_image_evidence(guests)

    def test_saved_provenance_must_match_guest_record(self):
        original = two_image_evidence(self.guests)
        mutations = [
            lambda value: value["runs"].reverse(),
            lambda value: value["provenance"][0].update(base_commit="5" * 40),
            lambda value: value["provenance"][0].update(image_sha256_before="5" * 64),
            lambda value: value["provenance"][0]["observed_digests"].update({"4": "5" * 16}),
            lambda value: value["provenance"][0]["artifact_sha256"].update({"esp/kernel.elf": "5" * 64}),
        ]
        for index, mutate in enumerate(mutations):
            with self.subTest(mutation=index):
                evidence = copy.deepcopy(original)
                mutate(evidence)
                with self.assertRaisesRegex(ValueError, "not bound to verified guest records"):
                    validate_two_image_evidence(evidence, self.guests)

    def test_source_mismatch_cannot_pass_even_with_all_saved_booleans_true(self):
        evidence = two_image_evidence(self.guests)
        self.guests[1]["base_commit"] = "5" * 40
        self.assertFalse(two_image_evidence(self.guests)["passed"])
        with self.assertRaisesRegex(ValueError, "not bound to verified guest records"):
            validate_two_image_evidence(evidence, self.guests)

    def test_image_and_serial_digests_must_match_oracle(self):
        for target in ("artifact", "serial"):
            with self.subTest(target=target):
                guests = copy.deepcopy(self.guests)
                if target == "artifact":
                    guests[0]["artifact_sha256"]["test-disk.img"] = "5" * 64
                else:
                    guests[0]["stdout"] += "M5:SECTOR4_DIGEST:" + "a" * 16 + "\n"
                with self.assertRaisesRegex(ValueError, "disagrees with guest image or digests"):
                    two_image_evidence(guests)


if __name__ == "__main__":
    unittest.main()
