"""HOST_ONLY S5 verifier counterexamples; fixtures are not native evidence."""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_power as gate

ACPI = "ACPI:UNAVAILABLE:NO_DMAR\nDEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT\n"


def transcript(tail, boots=1):
    boot = "\n".join(gate.BOOT).replace("KS-02:PASS\n", "KS-02:PASS\n" + ACPI)
    return "BdsDxe: firmware\n" + "\n".join([boot] * boots + list(tail)) + "\n"


class PowerVerifier(unittest.TestCase):
    def test_poweroff_and_rejection_cases_pass_with_exact_exit(self):
        for feature, (code, tail) in gate.CASES.items():
            with self.subTest(feature=feature):
                self.assertEqual(gate.evaluate(transcript(tail), code, False, feature), list(tail))

    def test_reset_wrong_exit_timeout_and_extra_output_fail(self):
        feature = "native-acpi-s5"
        tail = gate.CASES[feature][1]
        bad = [
            (transcript(tail, boots=2), 0, False),  # reset re-ran firmware
            (transcript(tail), 33, False),  # debug-exit instead of poweroff
            (transcript(tail), None, True),  # write without effect
            (transcript(tail + ("M1:PASS",)), 0, False),
            (transcript(tail[:1]), 0, False),  # poweroff without WRITE marker
            (transcript(("POWER:S5:REJECTED:InvalidChecksum",)), 0, False),
        ]
        for stdout, code, timed_out in bad:
            with self.subTest(stdout=stdout[-60:], code=code):
                with self.assertRaises(ValueError):
                    gate.evaluate(stdout, code, timed_out, feature)

    def test_rejected_profile_must_not_write(self):
        feature = "test-acpi-s5-corrupt-dsdt"
        stdout = transcript(("POWER:S5:REJECTED:InvalidChecksum", "POWER:S5:WRITE"))
        with self.assertRaises(ValueError):
            gate.evaluate(stdout, 33, False, feature)


if __name__ == "__main__":
    unittest.main()
