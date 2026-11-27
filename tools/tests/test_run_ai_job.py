"""HOST_ONLY serial-parser counterexamples for run_ai_job; generated text is no native evidence."""
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_ai_job as gate
from common import DEVICE_EXECUTION_MARKER, EXPECTED_ACPI_MARKER

BITS = [(0x3f800000 + 7 * i) & 0xffffffff for i in range(64)]
ORACLE = {"macs": 4992, "digest": 17951869293258106953, "argmax": [6, 6, 6, 6], "bits": BITS}


def run_lines(number, start, end, bits=BITS, irqs=25):
    values = {"ITERATIONS": 2101, "START_TICKS": start, "END_TICKS": end, "TIMER_IRQS": irqs, "MACS": 4992,
              "DIGEST": ORACLE["digest"], "ARGMAX": int.from_bytes(bytes([6, 6, 6, 6]), "little")}
    values.update({f"BITS{i:02d}": bits[2 * i] | (bits[2 * i + 1] << 32) for i in range(32)})
    return [f"AIJOB:RUN{number}:{name}:{values[name]:020d}" for name in gate.RUN_FIELDS]


def fixture(run2_bits=BITS, totals=None, run2_irqs=25):
    boot = list(gate.BOOT)
    at = boot.index("KS-02:PASS") + 1
    boot[at:at] = [EXPECTED_ACPI_MARKER, DEVICE_EXECUTION_MARKER]
    values = {"RUNS": 2, "SLOT_REUSES": 1, "TIMER_A": 50, "TIMER_B": 57, "TIMER_IRQS": 107,
              "SYSCALL_CALLS": 545, "SAVES": 652, "RESTORES": 652, "BASELINE_FRAMES": 49063,
              "FREE_FRAMES": 49063, "CONTEXT_BYTES_WIPED": 8192, **(totals or {})}
    lines = (["noise before handoff"] + boot + ["AIJOB:MANIFEST:PASS"] + run_lines(1, 1, 51)
             + ["AIJOB:SURVIVOR:PASS", "AIJOB:SLOT_REUSE:PASS"] + run_lines(2, 56, 106, run2_bits, run2_irqs)
             + [f"AIJOB:{name}:{values[name]:020d}" for name in gate.TOTAL_FIELDS] + list(gate.END))
    return "\n".join(lines) + "\n"


class SerialParser(unittest.TestCase):
    def test_complete_fixture_only_tests_the_parser(self):
        observed = gate.validate_sequence(fixture(), ORACLE)
        self.assertEqual([run["ticks"] for run in observed["runs"]], [50, 50])
        self.assertEqual(observed["runs"][0]["macs_per_second"], 4992 * 2101 * 100 // 50)
        self.assertEqual(observed["runs"][1]["bits"], BITS)

    def test_one_flipped_bit_in_run_two_is_rejected(self):
        bits = list(BITS)
        bits[17] ^= 1
        with self.assertRaises(ValueError):
            gate.validate_sequence(fixture(run2_bits=bits), ORACLE)

    def test_oracle_mismatch_is_rejected(self):
        with self.assertRaises(ValueError):
            gate.validate_sequence(fixture(), {**ORACLE, "digest": ORACLE["digest"] ^ 1})

    def test_failed_unknown_reordered_or_truncated_output_is_rejected(self):
        text = fixture()
        for bad in (text.replace("AIJOB:PASS\n", "AIJOB:FAIL:LIFECYCLE\n"),
                    text.replace("AIJOB:SLOT_REUSE:PASS\n", ""),
                    text.replace("AIJOB:SURVIVOR:PASS\nAIJOB:SLOT_REUSE:PASS\n",
                                 "AIJOB:SLOT_REUSE:PASS\nAIJOB:SURVIVOR:PASS\n"),
                    text + "AIJOB:EXTRA:PASS\n",
                    text.removesuffix("M1:PASS\n"),
                    text.replace("AIJOB:RUN1:MACS:", "AIJOB:RUN1:MACS:1")):
            with self.subTest(bad=bad[-80:]), self.assertRaises(ValueError):
                gate.validate_sequence(bad, ORACLE)

    def test_inconsistent_counts_are_rejected(self):
        for totals in ({"SAVES": 653}, {"RESTORES": 651}, {"FREE_FRAMES": 49062},
                       {"TIMER_IRQS": 108}, {"SLOT_REUSES": 0}, {"TIMER_B": 1, "TIMER_IRQS": 51},
                       {"TIMER_A": 49, "TIMER_IRQS": 106}):
            with self.subTest(totals=totals), self.assertRaises(ValueError):
                gate.validate_sequence(fixture(totals=totals), ORACLE)

    def test_run_without_job_preemption_is_rejected(self):
        # The job can reach its duration by yielding while every tick lands on the canary.
        with self.assertRaises(ValueError):
            gate.validate_sequence(fixture(run2_irqs=2, totals={"TIMER_A": 27, "TIMER_IRQS": 84}), ORACLE)


if __name__ == "__main__":
    unittest.main()
