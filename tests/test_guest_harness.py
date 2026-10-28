"""Evidence-gate tests. Synthetic strings test only the parser, never guest code."""
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from run import is_success, MARKERS, MILESTONE_MARKERS
from capture_gate import guest_matches


class GuestEvidenceGate(unittest.TestCase):
    def test_observed_m5_no_device_output_cannot_masquerade_as_m1(self):
        # Actual control-line excerpt from the rejected cold-3-3255a86c2b
        # diagnostic (kernel 44aca9b044e9b36ef906effe4367fe97996577aef2caa129368ec53533e9cc6a).
        # This is a parser fixture, not a new guest-execution claim.
        log = '\n'.join((
            'BOOT:EBS_OK', 'KERNEL:ENTRY', 'BOOTINFO:OK',
            'KS-01:PASS', 'KS-02:PASS', 'KS-03:PASS', 'M2:PASS',
            'M5:CASE:NO_DEVICE:PASS', 'M5:SURVIVOR_PROGRESS:PASS',
            'M5:RESOURCES:PASS', 'M5:PASS', 'M1:PASS',
        ))
        self.assertFalse(is_success(33, log))
        self.assertFalse(guest_matches('m5', 'm1-regression-after-m5',
                                      dict(raw_qemu_exit=33, stdout=log, timed_out=False)))
        explicit_m5 = MARKERS[:3] + ('M5:CASE:NO_DEVICE:PASS', 'M5:SURVIVOR_PROGRESS:PASS',
                                    'M5:RESOURCES:PASS', 'M5:PASS', 'M1:PASS')
        self.assertTrue(is_success(33, log, required_markers=explicit_m5))

    def test_m1_and_ebs_retry_reject_any_later_milestone_output(self):
        for required in (MARKERS, ('BOOT:EBS_RETRY',) + MARKERS):
            log = '\n'.join(required)
            self.assertTrue(is_success(33, log, required_markers=required))
            for phase in ('M2', 'M3', 'M4', 'M5', 'M6A', 'M6B'):
                with self.subTest(profile=required, phase=phase):
                    self.assertFalse(is_success(33, log+'\n'+phase+':DIAGNOSTIC:1', required_markers=required))

    def test_explicit_milestone_profiles_remain_valid(self):
        for phase, markers in MILESTONE_MARKERS.items():
            with self.subTest(phase=phase):
                self.assertTrue(is_success(33, '\n'.join(markers), required_markers=markers))
        for phase in ('M6A', 'M6B'):
            markers = MARKERS[:3] + (phase+':PASS', 'M1:PASS')
            self.assertTrue(is_success(33, '\n'.join(markers), required_markers=markers))

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
