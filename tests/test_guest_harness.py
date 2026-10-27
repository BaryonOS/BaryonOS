"""Evidence-gate tests. Synthetic strings test only the parser, never guest code."""
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from run import is_success, MARKERS, MILESTONE_MARKERS
from capture_gate import guest_matches


class GuestEvidenceGate(unittest.TestCase):
    def test_complete_ordered_lines_and_raw_exit_required(self):
        log = "firmware banner\r\n" + "\r\n".join(MARKERS) + "\r\n"
        self.assertTrue(is_success(33, log))
        for status in (0, 1, 35, None):
            self.assertFalse(is_success(status, log))
        self.assertFalse(is_success(33, log, True))
        self.assertFalse(is_success(33, "\n".join(reversed(MARKERS))))

    def test_suffix_prefix_missing_and_failure_are_rejected(self):
        for marker in MARKERS:
            for altered in (marker + "-bogus", "bogus-" + marker, ""):
                self.assertFalse(is_success(33, "\n".join(
                    altered if item == marker else item for item in MARKERS)))
        for failure in ("M1:FAIL:x", "M2:FAIL:x", "M3:FAIL:x", "M4:FAIL:x", "M5:FAIL:x"):
            self.assertFalse(is_success(33, "\n".join(MARKERS) + "\n" + failure))

    def test_expected_negative_exit_is_explicit_and_not_timeout(self):
        log = "KERNEL:ENTRY\nM2:FAIL:KERNEL_EXCEPTION\n"
        args = dict(required_markers=("KERNEL:ENTRY", "M2:FAIL:KERNEL_EXCEPTION"),
                    expected_exit=35, forbidden_markers=("M1:PASS", "M2:PASS"))
        self.assertFalse(is_success(35, log))
        self.assertTrue(is_success(35, log, **args))
        self.assertFalse(is_success(33, log, **args))
        self.assertFalse(is_success(35, log, True, **args))
        self.assertFalse(is_success(35, log + "M2:PASS\n", **args))

    def test_m5_capture_requires_disk_oracle_except_explicit_m1_regression(self):
        result = dict(raw_qemu_exit=33, stdout="\n".join(MILESTONE_MARKERS["m5"]),
                      timed_out=False, required_markers=list(MILESTONE_MARKERS["m5"]),
                      expected_exit=33, forbidden_markers=["M5:FAIL"], m5_case="read")
        self.assertFalse(guest_matches("m5", "m5-read-a", result))
        result["disk_oracle"] = {"passed": False}
        self.assertFalse(guest_matches("m5", "m5-read-a", result))
        result["disk_oracle"]["passed"] = True
        self.assertTrue(guest_matches("m5", "m5-read-a", result))
        del result["m5_case"]
        self.assertFalse(guest_matches("m5", "m5-read-a", result))
        result.update(stdout="\n".join(MARKERS), required_markers=list(MARKERS))
        del result["disk_oracle"]
        self.assertTrue(guest_matches("m5", "m1-regression-after-m5", result))
        result["raw_qemu_exit"] = 35
        self.assertFalse(guest_matches("m5", "m1-regression-after-m5", result))


if __name__ == "__main__":
    unittest.main()
