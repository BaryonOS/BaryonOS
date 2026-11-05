import struct
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from run_smp import CASES, MAX_CPUS, duplicate_madt, evaluate

PREFIX = ("BdsDxe: starting Boot0002\nBOOT:EBS_OK\nKERNEL:ENTRY\nBOOTINFO:OK\nKS-01:PASS\n"
          "KS-02:PASS\nACPI:UNAVAILABLE:NO_DMAR\nDEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT\n"
          "KS-03:PASS\n")
END = ["M2:PASS", "M1:PASS"]


def transcript(lines):
    return PREFIX + "".join(line + "\n" for line in lines)


# Hand-written expectations, deliberately independent of run_smp.expected_smp.
def started(cpus):
    count = min(cpus, MAX_CPUS)
    lines = [f"SMP:MADT:CPUS={cpus}:DISABLED=0:HOTPLUG=0"]
    lines += [f"SMP:AP:{apic}:ONLINE" for apic in range(1, count)]
    if cpus > MAX_CPUS:
        lines.append(f"SMP:SKIPPED:CAPACITY={cpus - MAX_CPUS}")
    return lines + [f"SMP:ONLINE:{count}/{cpus}", f"SMP:SYNC:PASS:CPUS={count}"]


def online(cpus):
    count = min(cpus, MAX_CPUS)
    return started(cpus) + [f"SMP:IPI:PASS:ROUNDS=3:CPUS={count}",
                            f"SMP:SHOOTDOWN:PASS:CPUS={count}",
                            f"SMP:SHOOTDOWN:VISIBLE:CPUS={count}",
                            f"SMP:PARKED:APS={count - 1}"] + END


def failed(status):
    return ["SMP:MADT:CPUS=4:DISABLED=0:HOTPLUG=0", f"SMP:AP:1:{status}",
            "SMP:BRINGUP:STOPPED", "SMP:ONLINE:1/4", "SMP:SYNC:PASS:CPUS=1",
            "SMP:IPI:PASS:ROUNDS=3:CPUS=1", "SMP:SHOOTDOWN:PASS:CPUS=1",
            "SMP:SHOOTDOWN:VISIBLE:CPUS=1", "SMP:PARKED:APS=0"] + END


DEAF = started(4) + ["SMP:IPI:TIMEOUT:APIC=1", "SMP:QUARANTINED:APS=3", "SMP:ONLINE:1/4"] + END
NMI = started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4", "SMP:FAULT:AP_NMI_IST:PASS"]
DOUBLE = started(4) + ["SMP:FAULT:AP_DF_IST:PASS"]
SHOOTDOWN_DEAF = started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4", "SMP:SHOOTDOWN:TIMEOUT:APIC=1",
                               "SMP:SHOOTDOWN:FRAME_QUARANTINED", "SMP:QUARANTINED:APS=3",
                               "SMP:ONLINE:1/4"] + END
STALE_CONTROL = started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4", "SMP:SHOOTDOWN:SKIPPED",
                              "SMP:SHOOTDOWN:STALE:APS=3", "SMP:PARKED:APS=3"] + END

VALID = {
    "one": online(1), "two-single": online(2), "four": online(4), "eight": online(8),
    "nine-capacity": online(9), "duplicate-madt": ["SMP:UNAVAILABLE:INVALID_MADT"] + END,
    "ap-stall": failed("TIMEOUT"), "ap-mismatch": failed("MISMATCH"),
    "ap-ipi-deaf": DEAF, "ap-nmi-ist": NMI, "ap-double-fault-ist": DOUBLE,
    "ap-shootdown-deaf": SHOOTDOWN_DEAF, "shootdown-skipped-control": STALE_CONTROL,
}


class SmpSerialOracle(unittest.TestCase):
    def test_every_case_accepts_its_exact_sequence(self):
        self.assertEqual(set(VALID), set(CASES))
        for case, lines in VALID.items():
            with self.subTest(case=case):
                result = evaluate(case, transcript(lines), 33, False)
                self.assertEqual(result["outcome"], CASES[case][3])
        self.assertEqual(evaluate("nine-capacity", transcript(online(9)), 33, False)["online"], 8)
        self.assertEqual(evaluate("ap-ipi-deaf", transcript(DEAF), 33, False)["online"], 1)
        self.assertEqual(evaluate("ap-stall", transcript(failed("TIMEOUT")), 33, False)["ap_ids"], [1])

    def test_rejects_missing_duplicate_or_overclaimed_output(self):
        four = online(4)
        # Mutate by marker, not index: the sequence grows with every K06 step.
        at = lambda prefix: next(i for i, line in enumerate(four) if line.startswith(prefix))
        swap = lambda prefix, new: four[:at(prefix)] + [new] + four[at(prefix) + 1:]
        drop = lambda prefix: four[:at(prefix)] + four[at(prefix) + 1:]
        bad = {
            "four": [
                four[:1] + four[2:],                      # one AP missing
                four[:2] + four[1:],                      # AP reported twice
                swap("SMP:ONLINE:", "SMP:ONLINE:5/4"),
                swap("SMP:SYNC:", "SMP:SYNC:PASS:CPUS=5"),
                four[:-2] + ["SMP:SYNC:PASS:CPUS=4"] + END,
                ["SMP:MADT:CPUS=3:DISABLED=0:HOTPLUG=0"] + four[1:],
                four[:1] + ["SMP:AP:1:TIMEOUT"] + four[2:],
                drop("SMP:SYNC:"),                       # sync result missing
                drop("SMP:IPI:"),                        # IPI result missing
                swap("SMP:IPI:", "SMP:IPI:PASS:ROUNDS=2:CPUS=4"),
                drop("SMP:PARKED:"),                     # park result missing
                swap("SMP:PARKED:", "SMP:PARKED:APS=2"),
                ["SMP:UNAVAILABLE:INVALID_MADT"] + END,
                [four[0], "SMP:AP:1:ONLINE", "SMP:AP:2:ONLINE", "SMP:AP:7:ONLINE"] + four[4:],
                [four[0], four[2], four[1], four[3]] + four[4:],  # unique but out of order
                DEAF,
            ],
            "nine-capacity": [online(8), online(9)[:4] + online(9)[5:]],
            "ap-stall": [failed("TIMEOUT")[:2] + failed("TIMEOUT")[3:], failed("MISMATCH"),
                         online(4)],
            "ap-mismatch": [failed("TIMEOUT"),
                            [failed("MISMATCH")[0], "SMP:AP:2:MISMATCH"] + failed("MISMATCH")[2:]],
            "ap-ipi-deaf": [online(4), DEAF[:-3] + END, DEAF[:-4] + DEAF[-3:],
                            started(4) + ["SMP:IPI:TIMEOUT:APIC=1", "SMP:QUARANTINED:APS=2",
                                          "SMP:ONLINE:1/4"] + END],
            # A terminal IST case must not report a normal finish instead of the marker,
            # and must not accept the other IST marker.
            "ap-nmi-ist": [online(4), NMI[:-1] + ["SMP:FAULT:AP_DF_IST:PASS"], NMI + END,
                           NMI[:-2] + NMI[-1:]],
            "ap-double-fault-ist": [online(4), DOUBLE[:-1] + ["SMP:FAULT:AP_NMI_IST:PASS"],
                                    DOUBLE[:-1]],
            "duplicate-madt": [online(4), END],
            # A normal run must prove the shootdown AND its visibility; a run
            # that skipped it (stale control output) must never count as normal.
            "eight": [[line for line in online(8) if "SHOOTDOWN:VISIBLE" not in line],
                      [line for line in online(8) if "SHOOTDOWN:PASS" not in line],
                      STALE_CONTROL],
            "ap-shootdown-deaf": [online(4), DEAF,
                                  [line for line in SHOOTDOWN_DEAF if "FRAME_QUARANTINED" not in line],
                                  SHOOTDOWN_DEAF[:-3] + ["SMP:SHOOTDOWN:PASS:CPUS=4"] + END],
            "shootdown-skipped-control": [
                online(4),  # the control must not silently become a passing shootdown
                STALE_CONTROL[:-4] + ["SMP:SHOOTDOWN:STALE:APS=2"] + STALE_CONTROL[-3:],
                STALE_CONTROL[:-4] + ["SMP:SHOOTDOWN:STALE:APS=0"] + STALE_CONTROL[-3:]],
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
        nmi = transcript(NMI)
        for text, raw in ((nmi, 35), (nmi + "M1:FAIL:SMP_AP_NMI_NOT_ON_IST\n", 33)):
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                evaluate("ap-nmi-ist", text, raw, False)

    def test_duplicate_madt_fixture_is_a_valid_checksummed_table(self):
        table = duplicate_madt()
        self.assertEqual(table[:4], b"APIC")
        self.assertEqual(struct.unpack_from("<I", table, 4)[0], len(table))
        self.assertEqual(sum(table) % 256, 0)
        self.assertEqual(table[44:52], bytes([0, 8, 0, 0, 1, 0, 0, 0]))


if __name__ == "__main__":
    unittest.main()
