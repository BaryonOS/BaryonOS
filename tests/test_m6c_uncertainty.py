"""Adversarial host checks of fault evidence; synthetic logs are not VM proof."""
from pathlib import Path
import sys
import struct
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import m6c_disk as disk
from m6c_faults import FAULT_CASES, outcome_valid
from run_m6c import case_markers, normal_success
from test_m6c_disk import record, seal


def fault_output(case):
    """Explicit host fixture, never used as real guest evidence."""
    counters = ("M6C:UNKNOWN_REPLIES:11", "M6C:POST_UNKNOWN:BLOCKCALLS:0",
                "M6C:POST_UNKNOWN:PUBLICATIONS:0", "M6C:POST_UNKNOWN:COMPLETIONS:0",
                "M6C:LIVE_SEQUENCE:1", "M6C:ACKS:1", "M6C:REJECTED_NO_IO:22",
                "M6C:CATALOG_REPLIES:0")
    return "\n".join(case_markers(case) + counters) + "\n"


def prefix_image(initial, count):
    data = bytearray(initial)
    for slot in range(count):
        offset = (disk.FIRST + slot) * disk.SECTOR
        entry = bytearray(record(slot, 0, slot + 1, operation=2 if slot == 0 else 4))
        struct.pack_into("<Q", entry, 24, 10 + slot)
        seal(entry)
        data[offset:offset+disk.SECTOR] = entry
    return bytes(data)


class FaultEvidence(unittest.TestCase):
    def test_all_faults_require_terminal_unknown_and_real_case_identity(self):
        for case in FAULT_CASES:
            good = fault_output(case)
            with self.subTest(case=case):
                self.assertTrue(normal_success(33, good, False, case))
                self.assertFalse(normal_success(0, good, False, case))
                self.assertFalse(normal_success(33, good, True, case))
                self.assertFalse(outcome_valid(good, "normal"))
                self.assertFalse(outcome_valid(good, "recover"))
                for other in FAULT_CASES:
                    if other != case:
                        self.assertFalse(outcome_valid(good, other))

    def test_missing_duplicate_malformed_or_successful_uncertain_evidence_rejected(self):
        good = fault_output("write-error")
        required = ("M6C:FAULT_INJECTED:WRITE_ERROR", "M6C:OUTCOME:UNKNOWN",
                    "M6C:UNKNOWN_REPLIES:11", "M6C:NO_REPLAY:PASS",
                    "M6C:POST_UNKNOWN:BLOCKCALLS:0", "M6C:POST_UNKNOWN:PUBLICATIONS:0",
                    "M6C:POST_UNKNOWN:COMPLETIONS:0", "M6C:LIVE_SEQUENCE:1",
                    "M6C:ACKS:1", "M6C:REJECTED_NO_IO:22", "M6C:CATALOG_REPLIES:0")
        for line in required:
            with self.subTest(line=line):
                for bad in (good.replace(line + "\n", ""), good + line + "\n",
                            good.replace(line, line + ":EXTRA")):
                    self.assertFalse(normal_success(33, bad, False, "write-error"))
        for old, new in (("ACKS:1", "ACKS:2"), ("LIVE_SEQUENCE:1", "LIVE_SEQUENCE:2"),
                         ("BLOCKCALLS:0", "BLOCKCALLS:1"), ("PUBLICATIONS:0", "PUBLICATIONS:1"),
                         ("COMPLETIONS:0", "COMPLETIONS:1"), ("UNKNOWN_REPLIES:11", "UNKNOWN_REPLIES:10")):
            self.assertFalse(outcome_valid(good.replace(old, new), "write-error"))

    def test_quarantine_is_required_only_for_unacknowledged_reset(self):
        good = fault_output("reset-timeout")
        self.assertTrue(outcome_valid(good, "reset-timeout"))
        self.assertFalse(outcome_valid(good.replace("M6C:QUARANTINE:PASS\n", ""), "reset-timeout"))
        self.assertFalse(outcome_valid(fault_output("write-error") + "M6C:QUARANTINE:PASS\n", "write-error"))

    def test_unexpected_fault_markers_cannot_enter_normal_or_cut_runs(self):
        base = "\n".join(case_markers("normal")) + "\n"
        for line in ("M6C:OUTCOME:UNKNOWN", "M6C:UNKNOWN_REPLIES:1",
                     "M6C:FAULT_INJECTED:WRITE_ERROR", "M6C:POST_UNKNOWN:BLOCKCALLS:0"):
            self.assertFalse(normal_success(33, base + line + "\n", False, "normal"))
            self.assertFalse(outcome_valid(base + line + "\n", "vm-cut-before-rename"))


class FaultDiskOracle(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="m6c-uncertainty-oracle-")
        self.addCleanup(temporary.cleanup)
        patcher = patch.object(disk, "BUILD", Path(temporary.name))
        patcher.start()
        self.addCleanup(patcher.stop)
        self.volume = disk.create_volume("fault-unit", seed=31)

    def verify(self, case, count, output=None):
        self.volume.path.write_bytes(prefix_image(self.volume.initial, count))
        return disk.verify_volume(self.volume, output or fault_output(case),
                                  case=case, before=self.volume.initial)

    def test_actual_prefix_depends_on_publication_boundary(self):
        for case in FAULT_CASES:
            allowed = {1} if case == "bad-descriptor" else {1, 2} if case in (
                "write-timeout", "reset-timeout") else {2}
            for count in (0, 1, 2):
                with self.subTest(case=case, records=count):
                    result = self.verify(case, count)
                    self.assertEqual(result["passed"], count in allowed, result)

    def test_uncertainty_never_claims_recovered_disk_state(self):
        for line in ("M6C:SEQUENCE:2", "M6C:OBJECT0:REVISION:2", "M6C:DOMAIN0:REVISION:2"):
            self.assertFalse(self.verify("write-error", 2,
                fault_output("write-error") + line + "\n")["passed"])

    def test_extra_replay_foreign_write_or_torn_tail_is_rejected(self):
        good = prefix_image(self.volume.initial, 2)
        for extra in (record(2, 0, 3), record(2, 2, 1), b"\xff" + bytes(511)):
            data = bytearray(good)
            at = (disk.FIRST + 2) * disk.SECTOR
            data[at:at+disk.SECTOR] = extra
            self.volume.path.write_bytes(data)
            report = disk.verify_volume(self.volume, fault_output("write-error"),
                case="write-error", before=self.volume.initial)
            self.assertFalse(report["passed"], report)

    def test_fault_cannot_hide_wrong_payload_name_or_outside_write(self):
        for replacement in (record(1, 0, 2, operation=4, payload=b"wrong"),
                            record(1, 0, 2, operation=4, name=b"wrong")):
            data = bytearray(prefix_image(self.volume.initial, 2))
            at = (disk.FIRST+1) * disk.SECTOR
            data[at:at+disk.SECTOR] = replacement
            self.volume.path.write_bytes(data)
            self.assertFalse(disk.verify_volume(self.volume, fault_output("flush-error"),
                case="flush-error", before=self.volume.initial)["passed"])
        data = bytearray(prefix_image(self.volume.initial, 2))
        data[-1] ^= 1
        self.volume.path.write_bytes(data)
        self.assertFalse(disk.verify_volume(self.volume, fault_output("flush-error"),
            case="flush-error", before=self.volume.initial)["passed"])

    def test_valid_checksums_cannot_hide_wrong_request_identity(self):
        for request_id in (1, 12, (1 << 64) - 1):
            data = bytearray(prefix_image(self.volume.initial, 2))
            at = (disk.FIRST + 1) * disk.SECTOR
            entry = data[at:at+disk.SECTOR]
            struct.pack_into("<Q", entry, 24, request_id)
            seal(entry)
            data[at:at+disk.SECTOR] = entry
            self.volume.path.write_bytes(data)
            report = disk.verify_volume(self.volume, fault_output("used-flush"),
                case="used-flush", before=self.volume.initial)
            self.assertFalse(report["passed"])
            self.assertFalse(report["checks"]["fault_records_are_only_confirmed_put_and_optional_rename"])


class ArchiveBoundary(unittest.TestCase):
    def test_verification_root_and_escape_cannot_be_replaced(self):
        import capture_m6c
        for path in ("docs/verification", "docs/verification/.", "docs/verification/..",
                     "docs/verification/m6d/..", "docs", "build", "docs/verification/m6d/nested"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                capture_m6c.archive_destination(path)
        self.assertEqual(capture_m6c.archive_destination(),
                         (capture_m6c.ROOT/"docs/verification/m6d").resolve())


if __name__ == "__main__":
    unittest.main()
