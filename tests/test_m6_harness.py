"""Host child-process evidence tests. No QEMU/device result is claimed here."""
import os
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from run_m6 import execute, normal_success, case_markers, run_m6
from capture_m6 import required_groups, validate_coverage


class CrashEvidence(unittest.TestCase):
    def child(self, suffix, cut=1, timeout=2):
        script = ("import time,os; "
                  "print('BOOT:EBS_OK',flush=True); print('KERNEL:ENTRY',flush=True); "
                  "print('BOOTINFO:OK',flush=True); " + suffix)
        return execute([sys.executable, "-u", "-c", script], dict(os.environ), cut=cut, timeout=timeout)

    def test_live_child_must_really_be_killed(self):
        rc, _, _, timeout, evidence = self.child("print('M6A:CUT:1',flush=True); time.sleep(5)")
        self.assertFalse(timeout); self.assertNotIn(rc, (None, 33))
        self.assertTrue(evidence["passed"])
        self.assertTrue(evidence["kill_requested"] and evidence["was_running_before_kill"])

    def test_marker_and_self_exit_is_not_host_crash(self):
        for code in (0, 1, 33):
            _, _, _, _, evidence = self.child(f"print('M6A:CUT:1',flush=True); os._exit({code})")
            self.assertFalse(evidence["passed"])

    def test_wrong_duplicate_and_success_markers_are_rejected(self):
        for output in ("M6A:CUT:2", "M6A:CUT:1\\nM6A:CUT:1", "M6A:CUT:1\\nM6A:PASS",
                       "M6A:CUT:1\\nM6A:FAIL:BOGUS"):
            _, _, _, _, evidence = self.child(f"print('{output}',flush=True); time.sleep(5)", timeout=.25)
            self.assertFalse(evidence["passed"])

    def test_timeout_does_not_count_as_expected_crash(self):
        _, _, _, timed_out, evidence = self.child("time.sleep(5)", timeout=.1)
        self.assertTrue(timed_out); self.assertFalse(evidence["passed"])

    def test_no_cut_mode_never_marks_host_kill(self):
        rc, _, _, timed_out, evidence = self.child("raise SystemExit(33)", cut=None)
        self.assertEqual(rc, 33); self.assertFalse(timed_out)
        self.assertFalse(evidence["kill_requested"])

    def test_normal_markers_are_unique_and_case_exact(self):
        normal = "\n".join(case_markers("normal"))
        self.assertTrue(normal_success(33, normal, False, "normal"))
        for wrong in (normal+"\n"+normal, normal+"\nM6A:CASE:OTHER:PASS",
                      normal+"\nM6A:PASS-bogus", normal+"\nM6A:CUT:1"):
            self.assertFalse(normal_success(33, wrong, False, "normal"))

    def test_recovery_refuses_a_fresh_volume_without_prior_guest(self):
        with self.assertRaises(ValueError):
            run_m6("recover")

    def test_capture_rejects_missing_duplicate_reordered_coverage(self):
        groups = [{"name": name, "evidence": "not-a-guest"} for name, _, _ in required_groups()]
        self.assertEqual(len(groups), 54)  # Frozen M6a contract, not a VM result.
        for altered in ([], groups[:-1], groups+groups[:1], list(reversed(groups))):
            with self.assertRaises(ValueError): validate_coverage(altered, [])
        with self.assertRaises(ValueError): validate_coverage(groups, [])


if __name__ == "__main__":
    unittest.main()
