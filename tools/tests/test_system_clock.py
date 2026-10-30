"""Host verifier tests only; no fixture here is native OS clock evidence."""
from datetime import datetime, timezone
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_system_clock as gate


def fixture(first: int, second: int, before: int = 10, after: int = 11) -> str:
    lines = [
        f"SYSTEM:CLOCK:UTC_FIRST:{first:020d}",
        f"SYSTEM:CLOCK:UTC_SECOND:{second:020d}",
        f"SYSTEM:CLOCK:MONOTONIC_BEFORE:{before:020d}",
        f"SYSTEM:CLOCK:MONOTONIC_AFTER:{after:020d}",
        "SYSTEM:CLOCK:INVALID_ARGS:PASS",
    ]
    return "\n".join(lines) + "\n"


class SystemClockVerifier(unittest.TestCase):
    def test_controlled_vm_time_and_monotonic_ticks_are_checked(self):
        base = int(datetime.strptime(gate.RTC_BASE, "%Y-%m-%dT%H:%M:%S")
                   .replace(tzinfo=timezone.utc).timestamp())
        self.assertEqual(
            gate.evaluate(fixture(base, base + 1), gate.RTC_BASE),
            {"UTC_FIRST": base, "UTC_SECOND": base + 1,
             "MONOTONIC_BEFORE": 10, "MONOTONIC_AFTER": 11},
        )

    def test_missing_duplicate_and_bad_clock_evidence_is_rejected(self):
        base = int(datetime.strptime(gate.RTC_BASE, "%Y-%m-%dT%H:%M:%S")
                   .replace(tzinfo=timezone.utc).timestamp())
        valid = fixture(base, base)
        invalid = (
            valid.replace("SYSTEM:CLOCK:INVALID_ARGS:PASS\n", ""),
            valid + "SYSTEM:CLOCK:UTC_FIRST:" + f"{base:020d}" + "\n",
            fixture(base - 1, base),
            fixture(base, base + 61),
            fixture(base + 1, base),
            fixture(base, base, 11, 10),
        )
        for text in invalid:
            with self.subTest(text=text), self.assertRaises(ValueError):
                gate.evaluate(text, gate.RTC_BASE)

    def test_invalid_calendar_base_is_rejected(self):
        base = int(datetime.strptime(gate.RTC_BASE, "%Y-%m-%dT%H:%M:%S")
                   .replace(tzinfo=timezone.utc).timestamp())
        with self.assertRaises(ValueError):
            gate.evaluate(fixture(base, base), "2026-02-30T12:00:00")


if __name__ == "__main__":
    unittest.main()
