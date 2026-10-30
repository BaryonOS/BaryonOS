import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_tpm as probe
from tools.tests import test_ipc_wait


class NativeTpmTranscriptTests(unittest.TestCase):
    def transcript(self, case):
        text = test_ipc_wait.NativeWaitTranscriptTests().transcript()
        values = {"WRITES": 12000 if case == "normal" else 0,
                  "BOUNDS_REJECTED": 7, "QUARANTINED": int(case == "absent")}
        if case == "normal":
            values["FAMILY"] = 0x322E3000
        extra = ["TPMCRB:BOUNDS:PASS", "TPMCRB:MAPPING:PASS"]
        extra += (["TPMCRB:RELEASED:PASS", "TPMCRB:PASS"] if case == "normal"
                  else ["TPMCRB:ABSENT:PASS"])
        return text + "\n".join(extra + [f"TPMCRB:{k}:{v:020d}" for k, v in values.items()]) + "\n"

    def test_completed_and_missing_device_cases(self):
        for case in probe.FEATURES:
            self.assertEqual(probe.evaluate(self.transcript(case), case)["BOUNDS_REJECTED"], 7)

    def test_case_substitution_and_malformed_evidence_are_rejected(self):
        for case in probe.FEATURES:
            other = "absent" if case == "normal" else "normal"
            good = self.transcript(case)
            for bad in (
                self.transcript(other),
                good.replace("TPMCRB:MAPPING:PASS\n", ""),
                good + "TPMCRB:WRITES:00000000000000000000\n",
                good.replace("TPMCRB:BOUNDS_REJECTED:00000000000000000007",
                             "TPMCRB:BOUNDS_REJECTED:7"),
                good.replace("TPMCRB:BOUNDS_REJECTED:00000000000000000007",
                             "TPMCRB:BOUNDS_REJECTED:00000000000000000006"),
                good.replace(f"TPMCRB:WRITES:{12000 if case == 'normal' else 0:020d}",
                             f"TPMCRB:WRITES:{0 if case == 'normal' else 1:020d}"),
                good.replace("TPMCRB:QUARANTINED:00000000000000000001",
                             "TPMCRB:QUARANTINED:00000000000000000000") if case == "absent"
                else good.replace(f"TPMCRB:FAMILY:{0x322E3000:020d}", f"TPMCRB:FAMILY:{0:020d}"),
            ):
                with self.subTest(case=case, bad=bad), self.assertRaises(ValueError):
                    probe.evaluate(bad, case)


if __name__ == "__main__":
    unittest.main()
