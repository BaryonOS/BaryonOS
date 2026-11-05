import struct
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from run_smp import CASES, MAX_CPUS, duplicate_madt, evaluate

PREFIX = ("BdsDxe: starting Boot0002\nBOOT:EBS_OK\nKERNEL:ENTRY\nBOOTINFO:OK\nKS-01:PASS\n"
          "KS-02:PASS\nACPI:UNAVAILABLE:NO_DMAR\nDEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT\n"
          "KS-03:PASS\n")
SUFFIX = "M2:PASS\nM1:PASS\n"


def transcript(lines):
    return PREFIX + "".join(line + "\n" for line in lines) + SUFFIX


def online(cpus):
    started = min(cpus, MAX_CPUS)
    lines = [f"SMP:MADT:CPUS={cpus}:DISABLED=0:HOTPLUG=0"]
    lines += [f"SMP:AP:{apic}:ONLINE" for apic in range(1, started)]
    if cpus > MAX_CPUS:
        lines.append(f"SMP:SKIPPED:CAPACITY={cpus - MAX_CPUS}")
    return lines + [f"SMP:ONLINE:{started}/{cpus}", f"SMP:SYNC:PASS:CPUS={started}"]


def failed(status):
    return ["SMP:MADT:CPUS=4:DISABLED=0:HOTPLUG=0", f"SMP:AP:1:{status}",
            "SMP:BRINGUP:STOPPED", "SMP:ONLINE:1/4", "SMP:SYNC:PASS:CPUS=1"]


VALID = {
    "one": online(1), "two-single": online(2), "four": online(4), "eight": online(8),
    "nine-capacity": online(9), "duplicate-madt": ["SMP:UNAVAILABLE:INVALID_MADT"],
    "ap-stall": failed("TIMEOUT"), "ap-mismatch": failed("MISMATCH"),
}


class SmpSerialOracle(unittest.TestCase):
    def test_every_case_accepts_its_exact_sequence(self):
        self.assertEqual(set(VALID), set(CASES))
        for case, lines in VALID.items():
            with self.subTest(case=case):
                result = evaluate(case, transcript(lines), 33, False)
                self.assertEqual(result["outcome"], CASES[case][3])
        self.assertEqual(evaluate("nine-capacity", transcript(online(9)), 33, False)["online"], 8)
        self.assertEqual(evaluate("ap-stall", transcript(failed("TIMEOUT")), 33, False)["ap_ids"], [1])

    def test_rejects_missing_duplicate_or_overclaimed_output(self):
        four = online(4)
        bad = {
            "four": [
                four[:1] + four[2:],                      # one AP missing
                four[:2] + four[1:],                      # AP reported twice
                four[:-2] + ["SMP:ONLINE:5/4", "SMP:SYNC:PASS:CPUS=4"],
                four[:-1] + ["SMP:SYNC:PASS:CPUS=5"],
                four + ["SMP:SYNC:PASS:CPUS=4"],
                ["SMP:MADT:CPUS=3:DISABLED=0:HOTPLUG=0"] + four[1:],
                four[:1] + ["SMP:AP:1:TIMEOUT"] + four[2:],
                four[:-1],                                # sync result missing
                ["SMP:UNAVAILABLE:INVALID_MADT"],
            ],
            "nine-capacity": [online(9)[:-3] + online(9)[-2:], online(8)],
            "ap-stall": [failed("TIMEOUT")[:2] + failed("TIMEOUT")[3:], failed("MISMATCH"),
                         online(4)],
            "ap-mismatch": [failed("TIMEOUT")],
            "duplicate-madt": [online(4), []],
        }
        for case, transcripts in bad.items():
            for lines in transcripts:
                with self.subTest(case=case, lines=lines), self.assertRaises(ValueError):
                    evaluate(case, transcript(lines), 33, False)

    def test_rejects_fail_markers_exit_codes_and_changed_acpi_inventory(self):
        good = transcript(online(4))
        for text, raw, timeout in (
            (good, 35, False), (good, 33, True), (good, None, True),
            (good.replace("M2:PASS", "M1:FAIL:SMP_SYNC_TIMEOUT\nM2:PASS"), 33, False),
            (good.replace("ACPI:UNAVAILABLE:NO_DMAR", "ACPI:UNAVAILABLE:INVALID_TABLES"), 33, False),
            ("SMP:ONLINE:4/4\n" + good, 33, False),
        ):
            with self.subTest(text=text[-80:], raw=raw), self.assertRaises(ValueError):
                evaluate("four", text, raw, timeout)

    def test_duplicate_madt_fixture_is_a_valid_checksummed_table(self):
        table = duplicate_madt()
        self.assertEqual(table[:4], b"APIC")
        self.assertEqual(struct.unpack_from("<I", table, 4)[0], len(table))
        self.assertEqual(sum(table) % 256, 0)
        self.assertEqual(table[44:52], bytes([0, 8, 0, 0, 1, 0, 0, 0]))


if __name__ == "__main__":
    unittest.main()
