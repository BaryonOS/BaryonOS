"""HOST_ONLY RTC verifier counterexamples; fixtures are not native evidence."""
from datetime import datetime, timezone
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_rtc as gate
from run import run_guest


def fixture(base):
    date = datetime.strptime(base, "%Y-%m-%dT%H:%M:%S").replace(tzinfo=timezone.utc)
    fields = ",".join(str(n) for n in (date.year, date.month, date.day,
                                      date.hour, date.minute, date.second))
    return "\n".join((*gate.BOOT, f"RTC:UTC:{int(date.timestamp())}",
                      "RTC:DATE:" + fields, *gate.END)) + "\n"


class RtcVerifier(unittest.TestCase):
    def test_calendar_cases_have_independent_epoch_oracles(self):
        for case, base in gate.CASES.items():
            with self.subTest(case=case):
                result = gate.evaluate(fixture(base), 33, False, base)
                self.assertEqual(result["quality"], "UnauthenticatedRtc")
        self.assertEqual(gate.evaluate(fixture(gate.CASES["epoch"]), 33, False,
                                       gate.CASES["epoch"])["unix_utc"], 0)

    def test_exit_timeout_missing_and_duplicate_markers_are_rejected(self):
        base = gate.CASES["current"]
        text = fixture(base)
        for raw, timeout in [(35, False), (0, False), (None, False), (33, True)]:
            with self.subTest(raw=raw, timeout=timeout), self.assertRaises(ValueError):
                gate.evaluate(text, raw, timeout, base)
        for marker in gate.REQUIRED:
            with self.subTest(marker=marker), self.assertRaises(ValueError):
                gate.evaluate(text.replace(marker + "\n", ""), 33, False, base)
        for altered in [text + "RTC:PASS\n", text + "M1:FAIL:RTC\n",
                        text.replace("RTC:RESTORE:PASS", "RTC:PASS")]:
            with self.assertRaises(ValueError):
                gate.evaluate(altered, 33, False, base)

    def test_wrong_date_timestamp_quality_and_host_time_do_not_pass(self):
        base = gate.CASES["epoch"]
        text = fixture(base)
        for old, new in [("RTC:UTC:0", "RTC:UTC:1"),
                         ("1970,1,1", "1970,2,30"),
                         ("UNAUTHENTICATED", "AUTHENTICATED"),
                         ("RTC:UTC:0", "RTC:UTC:-1"),
                         ("RTC:DATE:", "RTC:DATE:bad")]:
            with self.subTest(old=old), self.assertRaises(ValueError):
                gate.evaluate(text.replace(old, new), 33, False, base)
        with self.assertRaises(ValueError):
            gate.evaluate(fixture(gate.CASES["current"]), 33, False, base)

    def test_vm_elapsed_calendar_window_has_exact_boundaries(self):
        base = gate.CASES["epoch"]
        for seconds, accepted in [(0, True), (60, True), (61, False)]:
            date = datetime.fromtimestamp(seconds, timezone.utc).strftime("%Y-%m-%dT%H:%M:%S")
            if accepted:
                gate.evaluate(fixture(date), 33, False, base)
            else:
                with self.assertRaises(ValueError):
                    gate.evaluate(fixture(date), 33, False, base)

    def test_bad_rtc_option_is_refused_before_any_guest_or_firmware_access(self):
        for base in ["2026-1-01T00:00:00", "2026-02-30T00:00:00",
                     "2026-01-01T00:00:00,clock=host", "now", "2026-01-01"]:
            with self.subTest(base=base), patch("run.firmware") as firmware:
                with self.assertRaises(ValueError):
                    run_guest(rtc_base=base)
                firmware.assert_not_called()


if __name__ == "__main__":
    unittest.main()
