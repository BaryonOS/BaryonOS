import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from common import without_acpi_diagnostics


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
        ):
            with self.subTest(text=text), self.assertRaises(ValueError):
                without_acpi_diagnostics(text)


if __name__ == "__main__":
    unittest.main()
