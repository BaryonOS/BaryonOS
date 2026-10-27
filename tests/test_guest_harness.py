"""Evidence-gate tests. Synthetic strings test only the parser, never guest code."""
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from run import is_success, MARKERS


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


if __name__ == "__main__":
    unittest.main()
