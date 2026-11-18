"""The DMA oracle must reject wrong-mode and incomplete native evidence."""
import unittest
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from run_xhci import BOOT, evaluate, markers


def transcript(case):
    return "\n".join((*BOOT, "ACPI:UNAVAILABLE:NO_DMAR" if case == "no-dmar"
                      else "ACPI:DISCOVERY:MCFG_DMAR", *markers(case))) + "\n"


class XhciOracleTests(unittest.TestCase):
    def test_all_distinct_cases(self):
        for case in ("noop", "fault", "no-dmar"):
            self.assertEqual(evaluate(case, transcript(case), 33, False)["dma"], case != "no-dmar")

    def test_fault_does_not_substitute_for_success(self):
        with self.assertRaises(ValueError):
            evaluate("noop", transcript("fault"), 33, False)

    def test_complete_guard_and_quiesce_evidence_required(self):
        for missing in ("XHCI:GUARD:UNCHANGED", "XHCI:QUIESCE:PASS", "XHCI:PAGES:RETAINED"):
            with self.subTest(missing=missing), self.assertRaises(ValueError):
                evaluate("fault", transcript("fault").replace(missing + "\n", ""), 33, False)

    def test_duplicate_unknown_and_reordered_markers(self):
        valid = transcript("noop")
        for invalid in (valid + "XHCI:PASS\n", valid + "XHCI:EXTRA\n",
                        valid.replace("XHCI:PAGING:PASS\nXHCI:DOMAIN:PASS", "XHCI:DOMAIN:PASS\nXHCI:PAGING:PASS")):
            with self.assertRaises(ValueError):
                evaluate("noop", invalid, 33, False)

    def test_timeout_exit_failure_and_firmware_absence(self):
        for status, timeout in ((35, False), (33, True), (None, False)):
            with self.assertRaises(ValueError):
                evaluate("noop", transcript("noop"), status, timeout)
        with self.assertRaises(ValueError):
            evaluate("no-dmar", transcript("no-dmar").replace("NO_DMAR", "DMAR_PRESENT"), 33, False)


if __name__ == "__main__":
    unittest.main()
