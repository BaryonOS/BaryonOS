"""Host child-process evidence tests. No QEMU/device result is claimed here."""
import os
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from run_m6 import execute


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


if __name__ == "__main__":
    unittest.main()
