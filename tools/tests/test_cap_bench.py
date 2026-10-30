"""HOST_ONLY verifier counterexamples; generated text is no native evidence."""
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_cap_bench as gate


def fixture():
    values = dict(zip(gate.HEADER, (1, 4, 21, 29, 9, 4, 9, 16)))
    rounds = {name: list(range(10 + index, 19 + index)) for index, name in enumerate(gate.VARIANTS)}
    for name, numbers in rounds.items():
        values.update({name + "_COUNT": 144, name + "_TSC_SUM": sum(numbers),
                       name + "_TSC_MIN": min(numbers), name + "_TSC_MAX": max(numbers)})
    lines = list(gate.BOOT) + [f"CAPBENCH:{name}:{values[name]:020d}" for name in gate.FIELDS]
    for index in range(values["ROUNDS"]):
        lines += [f"CAPBENCH:ROUND_VARIANT:{(index + position) % 3:020d}" for position in range(3)]
        lines += [f"CAPBENCH:ROUND_{name}_TSC:{rounds[name][index]:020d}" for name in gate.VARIANTS]
    return "\n".join(lines + list(gate.END)) + "\n"


class DiagnosticVerifier(unittest.TestCase):
    def test_complete_fixture_only_tests_the_parser(self):
        observed = gate.evaluate(fixture(), 33, False)
        self.assertEqual(observed["values"]["DIRECT_COUNT"], 144)
        self.assertEqual(observed["execution_order"],
                         [[gate.VARIANTS[(r + p) % 3] for p in range(3)] for r in range(9)])

    def test_exit_timeout_and_truncation_cannot_pass(self):
        for raw, timeout in [(35, False), (0, False), (33, True), (None, False)]:
            with self.subTest(raw=raw, timeout=timeout), self.assertRaises(ValueError):
                gate.evaluate(fixture(), raw, timeout)
        with self.assertRaises(ValueError):
            gate.evaluate(fixture().removesuffix("M1:PASS\n"), 33, False)

    def test_altered_totals_rounds_or_workload_are_rejected(self):
        for old, new in [("REF_COUNT:00000000000000000144", "REF_COUNT:00000000000000000143"),
                         ("REF_TSC_SUM:00000000000000000126", "REF_TSC_SUM:00000000000000000127"),
                         ("ROUND_REF_TSC:00000000000000000010", "ROUND_REF_TSC:00000000000000000000"),
                         ("PROCESSES:00000000000000000004", "PROCESSES:00000000000000000005"),
                         ("ROUND_VARIANT:00000000000000000000", "ROUND_VARIANT:00000000000000000002")]:
            value = fixture().replace(old, new)
            self.assertNotEqual(value, fixture())
            with self.subTest(old=old), self.assertRaises(ValueError):
                gate.evaluate(value, 33, False)


if __name__ == "__main__":
    unittest.main()
