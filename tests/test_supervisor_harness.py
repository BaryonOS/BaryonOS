"""Host-only parser tests for the K05 runner. Synthetic transcripts here are
never emulator evidence; they pin that the verdict rejects what it must."""
from __future__ import annotations
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import run_supervisor as rs  # noqa: E402


def transcript(case: str, values: dict[str, int], extra: str = "") -> str:
    lines = list(rs.BOOT) + ["SUPERVISOR:MANIFEST:PASS", "SUPERVISOR:READY"]
    lines += [f"SUPERVISOR:{key}:{value:020d}" for key, value in values.items()]
    lines += [f"SUPERVISOR:CASE:{rs.CASE_MARKER[case]}:PASS", "SUPERVISOR:RESOURCES:PASS",
              "SUPERVISOR:PASS", "M1:PASS"]
    return "\n".join(lines) + ("\n" + extra if extra else "") + "\n"


def good(case: str) -> dict[str, int]:
    values = dict(rs.EXPECT[case])
    values.update(rs.AT_LEAST[case])
    values.setdefault("TICKS", 1234)
    return values


class Numbers(unittest.TestCase):
    def test_numbers_require_twenty_digits_and_reject_duplicates(self):
        self.assertEqual(rs.numbers("SUPERVISOR:SPAWNS:00000000000000000003\n"), {"SPAWNS": 3})
        self.assertEqual(rs.numbers("SUPERVISOR:SPAWNS:3\n"), {})
        with self.assertRaises(ValueError):
            rs.numbers("SUPERVISOR:SPAWNS:00000000000000000003\nSUPERVISOR:SPAWNS:00000000000000000004\n")

    def test_every_case_has_a_marker_and_expectations(self):
        for case in rs.FEATURE:
            self.assertIn(case, rs.CASE_MARKER)
            self.assertIn(case, rs.EXPECT)
            self.assertIn(case, rs.AT_LEAST)
            self.assertEqual(rs.expected_markers(case)[-1], "M1:PASS")


class Verdict(unittest.TestCase):
    def test_complete_transcripts_pass_for_every_case(self):
        for case in rs.FEATURE:
            found = rs.evaluate(case, transcript(case, good(case)), 33, False)
            self.assertEqual(found["TICKS"], 1234)

    def test_failure_marker_rejects_even_with_all_pass_markers(self):
        text = transcript("normal", good("normal"), "SUPERVISOR:FAIL:LATE")
        with self.assertRaisesRegex(ValueError, "failure marker"):
            rs.evaluate("normal", text, 33, False)
        text = transcript("normal", good("normal"), "M2:PANIC_LINE:0000000012")
        with self.assertRaisesRegex(ValueError, "failure marker"):
            rs.evaluate("normal", text, 33, False)

    def test_wrong_exit_timeout_and_marker_order_reject(self):
        text = transcript("quota", good("quota"))
        with self.assertRaisesRegex(ValueError, "raw exit"):
            rs.evaluate("quota", text, 35, False)
        with self.assertRaisesRegex(ValueError, "raw exit"):
            rs.evaluate("quota", text, 33, True)
        reordered = text.replace("SUPERVISOR:READY\n", "").replace("SUPERVISOR:PASS\n",
                                                                   "SUPERVISOR:PASS\nSUPERVISOR:READY\n")
        with self.assertRaisesRegex(ValueError, "marker sequence"):
            rs.evaluate("quota", reordered, 33, False)
        with self.assertRaisesRegex(ValueError, "marker sequence"):
            rs.evaluate("crash", transcript("normal", good("normal")), 33, False)

    def test_each_expected_number_is_load_bearing(self):
        for case in rs.FEATURE:
            for key in rs.EXPECT[case]:
                values = good(case)
                values[key] += 1
                with self.assertRaisesRegex(ValueError, key):
                    rs.evaluate(case, transcript(case, values), 33, False)
            for key, minimum in rs.AT_LEAST[case].items():
                values = good(case)
                values[key] = minimum - 1
                with self.assertRaisesRegex(ValueError, key):
                    rs.evaluate(case, transcript(case, values), 33, False)
            values = good(case)
            del values["TICKS"]
            with self.assertRaisesRegex(ValueError, "tick budget"):
                rs.evaluate(case, transcript(case, values), 33, False)

    def test_missing_number_rejects(self):
        values = good("denied")
        del values["DENIED_TOTAL"]
        with self.assertRaisesRegex(ValueError, "DENIED_TOTAL"):
            rs.evaluate("denied", transcript("denied", values), 33, False)


if __name__ == "__main__":
    unittest.main()
