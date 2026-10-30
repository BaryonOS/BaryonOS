"""Counterexamples against the memory evidence gate; no native execution here."""
import unittest
from pathlib import Path
import os
import tempfile
from unittest.mock import patch

import run_memory as m
import run_fpu as f
import test_memory_system as matrix


def evidence(case="normal", image_pages=6):
    lives = 16 if case == "oom" else 3
    pages = 12800 if case == "oom" else (2048 - image_pages - 32) * lives
    values = dict(CLEAN_STARTS=lives, GROWN_PAGES=pages, ZERO_BYTES=pages * 4096,
                  ARG_DENIED=7 * lives, QUOTA_DENIED=8 if case == "oom" else 6,
                  PRIVATE_CHECKS=lives, COLLECTION_RUNS=lives, ALLOC_OOM_DENIED=lives,
                  FAULTS=int(case in ("nx", "guard", "peer")),
                  EXITS=lives - int(case in ("nx", "guard", "peer")),
                  SLOT_REUSES=int(case != "oom"), OOM_FAILURES=8 if case == "oom" else 0,
                  OOM_RECOVERIES=int(case == "oom"), RETRY_ZERO_BYTES=4096 * int(case == "oom"),
                  TIMER_IRQS=19, LIVE_PROCESSES=0, FRAMES_RECOVERED=1, KERNEL_TABLES_RETAINED=10,
                  CLEANUP_ZERO_BYTES=pages * 4096 + lives * 65536)
    return "\n".join(m.BOOT + ("MEMORY:MANIFEST:PASS",)
                     + tuple(f"MEMORY:{name}:{values[name]:020d}" for name in m.FIELDS)
                     + m.endings(case)) + "\n"


class EvidenceTests(unittest.TestCase):
    def test_each_complete_profile_has_distinct_lifecycle_and_real_oom(self):
        for case in m.CASES:
            with self.subTest(case=case):
                result = m.validate_sequence("UEFI banner\n" + evidence(case), case, 6)
                self.assertEqual(result["CLEAN_STARTS"], 16 if case == "oom" else 3)

    def test_missing_duplicate_unknown_reordered_or_preboot_evidence_is_rejected(self):
        text = evidence()
        for bad in (text.replace("MEMORY:GROWTH:PASS\n", ""),
                    text + "M1:PASS\n", text + "UNKNOWN\n", "MEMORY:GROWTH:PASS\n" + text,
                    text.replace("KS-01:PASS", "KS-01:FAIL"),
                    text.replace("MEMORY:GROWTH:PASS\nMEMORY:ISOLATION:PASS",
                                 "MEMORY:ISOLATION:PASS\nMEMORY:GROWTH:PASS"),
                    text.replace("MEMORY:MANIFEST:PASS", "MEMORY:MANIFEST:PASS\nMEMORY:MANIFEST:PASS")):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                m.validate_sequence(bad, "normal", 6)

    def test_wrong_zeroing_quota_cleanup_or_incomplete_lifetime_is_rejected(self):
        for name, value in (("CLEAN_STARTS", 2), ("ZERO_BYTES", 1), ("GROWN_PAGES", 6031),
                            ("ARG_DENIED", 20), ("QUOTA_DENIED", 5), ("PRIVATE_CHECKS", 2),
                            ("COLLECTION_RUNS", 0), ("ALLOC_OOM_DENIED", 0), ("FAULTS", 1),
                            ("EXITS", 2), ("SLOT_REUSES", 0), ("OOM_FAILURES", 1),
                            ("OOM_RECOVERIES", 1), ("RETRY_ZERO_BYTES", 4096),
                            ("TIMER_IRQS", 0), ("LIVE_PROCESSES", 1),
                            ("FRAMES_RECOVERED", 0), ("KERNEL_TABLES_RETAINED", 257),
                            ("CLEANUP_ZERO_BYTES", 0)):
            text = evidence()
            old = next(line for line in text.splitlines() if line.startswith(f"MEMORY:{name}:"))
            with self.subTest(name=name), self.assertRaises(ValueError):
                m.validate_sequence(text.replace(old, f"MEMORY:{name}:{value:020d}"), "normal", 6)

    def test_wrong_case_or_different_linked_image_cannot_reuse_a_pass(self):
        for case in m.CASES[1:]:
            with self.subTest(case=case), self.assertRaises(ValueError):
                m.validate_sequence(evidence(), case, 6)
        with self.assertRaises(ValueError):
            m.validate_sequence(evidence(), "normal", 7)

    def test_oom_requires_actual_exhaustion_and_recovery(self):
        text = evidence("oom")
        for name, value in (("OOM_FAILURES", 0), ("OOM_FAILURES", 17),
                            ("OOM_RECOVERIES", 0), ("RETRY_ZERO_BYTES", 0),
                            ("QUOTA_DENIED", 7), ("GROWN_PAGES", 100)):
            old = next(line for line in text.splitlines() if line.startswith(f"MEMORY:{name}:"))
            with self.subTest(name=name), self.assertRaises(ValueError):
                m.validate_sequence(text.replace(old, f"MEMORY:{name}:{value:020d}"), "oom", 6)

    def test_unsigned_fixed_width_counters_and_image_bound_are_required(self):
        text = evidence()
        old = next(line for line in text.splitlines() if line.startswith("MEMORY:TIMER_IRQS:"))
        for value in ("-0000000000000000001", "19", "0000000000000000001x", "18446744073709551616"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                m.validate_sequence(text.replace(old, "MEMORY:TIMER_IRQS:" + value), "normal", 6)
        for image_pages in (True, 0, 1500, "6"):
            with self.subTest(image_pages=image_pages), self.assertRaises(ValueError):
                m.validate_sequence(text, "normal", image_pages)

    def test_matrix_refuses_actual_linked_report_before_any_build_or_external_write(self):
        for kind in ("hardlink", "symlink"):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as temp:
                base = Path(temp)
                root = base / "checkout"
                build = root / "build"
                build.mkdir(parents=True)
                outside = base / "keep"
                outside.write_bytes(b"preserve-external-content")
                report = build / "memory-gate.json"
                if kind == "hardlink":
                    os.link(outside, report)
                else:
                    report.symlink_to(outside)
                with patch.object(f, "ROOT", root), patch.object(matrix, "ROOT", root), \
                        patch.object(matrix, "BUILD", build), patch.object(matrix, "TARGET", root / "target"), \
                        patch.object(matrix.subprocess, "check_output") as git, \
                        patch.object(matrix, "run_logged") as command:
                    with self.assertRaises(ValueError):
                        matrix.main()
                    git.assert_not_called()
                    command.assert_not_called()
                self.assertEqual(outside.read_bytes(), b"preserve-external-content")

    def test_wrapper_refuses_actual_run_directory_link_before_snapshot_or_launch(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            root = base / "checkout"
            build = root / "build"
            build.mkdir(parents=True)
            outside = base / "outside"
            outside.mkdir()
            (build / "run").symlink_to(outside, target_is_directory=True)
            qemu = base / "qemu"
            qemu.write_bytes(b"unused-qemu-marker")
            with patch.object(f, "ROOT", root), patch.object(m, "BUILD", build), \
                    patch.object(m, "validate_build", return_value={}), \
                    patch.object(m.shutil, "which", return_value=str(qemu)), \
                    patch.object(m, "run_guest") as guest:
                with self.assertRaises(ValueError):
                    m.run_memory()
                guest.assert_not_called()
            self.assertEqual(list(outside.iterdir()), [])

    def test_guest_requires_real_bounded_runtime_and_identified_emulator(self):
        # All other fields are sufficient to reach artifact validation. An
        # invalid time/version must fail before any evidence path is opened.
        good = dict(raw_qemu_exit=33, timed_out=False, passed=True, expected_exit=33,
                    artifact_binding_passed=True, required_markers=list(m.required("normal")),
                    forbidden_markers=list(m.FORBIDDEN), source_fingerprint_version=3,
                    source_sha256="frozen", timeout_s=60, duration_s=1.0,
                    qemu_version="QEMU emulator version test")
        for change in ({"timeout_s": 9999}, {"timeout_s": None}, {"timeout_s": True},
                       {"duration_s": 9999}, {"duration_s": None}, {"duration_s": float("nan")},
                       {"duration_s": float("inf")}, {"duration_s": -1}, {"duration_s": True},
                       {"qemu_version": None}, {"qemu_version": "unidentified"}):
            with self.subTest(change=change), patch.object(m, "private_path") as path:
                with self.assertRaises(ValueError):
                    m.validate_guest(good | change, {}, "frozen", "normal", Path("unused"), "unused")
                path.assert_not_called()


if __name__ == "__main__":
    unittest.main()
