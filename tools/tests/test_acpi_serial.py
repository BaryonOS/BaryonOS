import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from common import with_acpi_markers, without_acpi_diagnostics


class AcpiSerialDiagnostics(unittest.TestCase):
    def test_removes_only_the_expected_pair_between_memory_checks(self):
        prefix = "BOOT:EBS_OK\nKS-02:PASS\n"
        pair = "ACPI:UNAVAILABLE:NO_DMAR\nDEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT\n"
        suffix = "KS-03:PASS\nM2:PASS\n"
        self.assertEqual(without_acpi_diagnostics(prefix + pair + suffix),
                         prefix + suffix)
        self.assertEqual(without_acpi_diagnostics(prefix + suffix), prefix + suffix)

    def test_rejects_unknown_duplicate_or_misplaced_diagnostics(self):
        pair = "ACPI:UNAVAILABLE:NO_DMAR\nDEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT\n"
        for text in (
            "KS-02:PASS\nACPI:UNKNOWN\nDEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT\nKS-03:PASS\n",
            "KS-02:PASS\n" + pair + pair + "KS-03:PASS\n",
            "KS-02:PASS\nDEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT\nACPI:UNAVAILABLE:NO_DMAR\nKS-03:PASS\n",
            "KS-01:PASS\n" + pair + "KS-03:PASS\n",
            # Valid-looking but failed discovery must not pass a q35 gate.
            "KS-02:PASS\nACPI:UNAVAILABLE:BAD_HANDOFF\nDEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT\nKS-03:PASS\n",
            "KS-02:PASS\nACPI:UNAVAILABLE:INVALID_TABLES\nDEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT\nKS-03:PASS\n",
            "KS-02:PASS\nACPI:UNAVAILABLE:NO_HANDOFF\nDEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT\nKS-03:PASS\n",
        ):
            with self.subTest(text=text), self.assertRaises(ValueError):
                without_acpi_diagnostics(text)

    def test_live_run_requires_the_pair_after_ks02(self):
        from run import is_success
        required = with_acpi_markers(("BOOT:EBS_OK", "KS-02:PASS", "KS-03:PASS", "M1:PASS"))
        with_pair = ("BOOT:EBS_OK\nKS-02:PASS\nACPI:UNAVAILABLE:NO_DMAR\n"
                     "DEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT\nKS-03:PASS\nM1:PASS\n")
        without = "BOOT:EBS_OK\nKS-02:PASS\nKS-03:PASS\nM1:PASS\n"
        self.assertTrue(is_success(33, with_pair, required_markers=required))
        self.assertFalse(is_success(33, without, required_markers=required))


if __name__ == "__main__":
    unittest.main()
