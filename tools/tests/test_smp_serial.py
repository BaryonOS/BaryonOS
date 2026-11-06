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


def transcript(lines, lockdep=True):
    """Non-terminal runs end with the step-5a lock self-test before END."""
    if lockdep and lines[-2:] == END:
        lines = lines[:-2] + ["SMP:LOCKDEP:PASS"] + END
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
                            f"SMP:PREEMPT:PASS:CPUS={count}",
                            f"SMP:SPIN:PASS:CPUS={count}",
                            f"SMP:SCHED:PASS:CPUS={count}",
                            f"SMP:PARKED:APS={count - 1}"] + END


def failed(status):
    return ["SMP:MADT:CPUS=4:DISABLED=0:HOTPLUG=0", f"SMP:AP:1:{status}",
            "SMP:BRINGUP:STOPPED", "SMP:ONLINE:1/4", "SMP:SYNC:PASS:CPUS=1",
            "SMP:IPI:PASS:ROUNDS=3:CPUS=1", "SMP:SHOOTDOWN:PASS:CPUS=1",
            "SMP:SHOOTDOWN:VISIBLE:CPUS=1", "SMP:PREEMPT:PASS:CPUS=1", "SMP:SPIN:PASS:CPUS=1",
            "SMP:SCHED:PASS:CPUS=1", "SMP:PARKED:APS=0"] + END


DEAF = started(4) + ["SMP:IPI:TIMEOUT:APIC=1", "SMP:QUARANTINED:APS=3", "SMP:ONLINE:1/4"] + END
NMI = started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4", "SMP:FAULT:AP_NMI_IST:PASS"]
DOUBLE = started(4) + ["SMP:FAULT:AP_DF_IST:PASS"]
SHOOTDOWN_DEAF = started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4", "SMP:SHOOTDOWN:TIMEOUT:APIC=1",
                               "SMP:SHOOTDOWN:FRAME_QUARANTINED", "SMP:QUARANTINED:APS=3",
                               "SMP:ONLINE:1/4"] + END
STALE_CONTROL = started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4", "SMP:SHOOTDOWN:SKIPPED",
                              "SMP:SHOOTDOWN:STALE:APS=3", "SMP:PREEMPT:PASS:CPUS=4",
                              "SMP:SPIN:PASS:CPUS=4", "SMP:SCHED:PASS:CPUS=4",
                              "SMP:PARKED:APS=3"] + END
LOCKDEP_ORDER = online(2)[:-2] + ["SMP:LOCKDEP:ORDER:30->10"]
TIMER_DEAD = started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4", "SMP:SHOOTDOWN:PASS:CPUS=4",
                           "SMP:SHOOTDOWN:VISIBLE:CPUS=4", "SMP:PREEMPT:TIMEOUT:APIC=1",
                           "SMP:QUARANTINED:APS=3", "SMP:ONLINE:1/4"] + END
SPIN_DEAF = started(4) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=4", "SMP:SHOOTDOWN:PASS:CPUS=4",
                          "SMP:SHOOTDOWN:VISIBLE:CPUS=4", "SMP:PREEMPT:PASS:CPUS=4",
                          "SMP:SPIN:SHOOTDOWN_TIMEOUT:APIC=1", "SMP:QUARANTINED:APS=3",
                          "SMP:ONLINE:1/4"] + END
HELD_ACROSS = started(2) + ["SMP:IPI:PASS:ROUNDS=3:CPUS=2", "SMP:SHOOTDOWN:PASS:CPUS=2",
                            "SMP:SHOOTDOWN:VISIBLE:CPUS=2", "SMP:LOCKDEP:HELD_ACROSS_SWITCH"]
DOUBLE_RUN = online(4)[:-4] + ["SMP:SCHED:DOUBLE_RUN"]

VALID = {
    "one": online(1), "two-single": online(2), "four": online(4), "eight": online(8),
    "nine-capacity": online(9), "duplicate-madt": ["SMP:UNAVAILABLE:INVALID_MADT"] + END,
    "ap-stall": failed("TIMEOUT"), "ap-mismatch": failed("MISMATCH"),
    "ap-ipi-deaf": DEAF, "ap-nmi-ist": NMI, "ap-double-fault-ist": DOUBLE,
    "ap-shootdown-deaf": SHOOTDOWN_DEAF, "shootdown-skipped-control": STALE_CONTROL,
    "ap-timer-dead": TIMER_DEAD, "lockdep-order": LOCKDEP_ORDER,
    "spin-irq-off-control": SPIN_DEAF, "held-across-switch": HELD_ACROSS,
    "double-run": DOUBLE_RUN,
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
                STALE_CONTROL[:-6] + ["SMP:SHOOTDOWN:STALE:APS=2"] + STALE_CONTROL[-5:],
                STALE_CONTROL[:-6] + ["SMP:SHOOTDOWN:STALE:APS=0"] + STALE_CONTROL[-5:]],
            "ap-timer-dead": [online(4), TIMER_DEAD[:-4] + END,
                              [line for line in TIMER_DEAD if "QUARANTINED" not in line]],
        }
        for case, transcripts in bad.items():
            for lines in transcripts:
                with self.subTest(case=case, lines=lines), self.assertRaises(ValueError):
                    evaluate(case, transcript(lines), 33, False)

    def test_normal_runs_must_prove_preemption(self):
        # Separate from the table above: a second "four" key there would
        # silently replace the existing "four" rejection cases.
        for lines in ([line for line in online(4) if "PREEMPT" not in line], TIMER_DEAD,
                      [line.replace("PREEMPT:PASS:CPUS=4", "PREEMPT:PASS:CPUS=3")
                       for line in online(4)]):
            with self.subTest(lines=lines), self.assertRaises(ValueError):
                evaluate("four", transcript(lines), 33, False)

    def test_normal_runs_must_prove_the_spin_window(self):
        # A normal run must show the spin phase; the deaf control must never
        # pass as a normal run, report a PASS, or skip the quarantine.
        for case, lines in (
            ("four", [line for line in online(4) if "SMP:SPIN:" not in line]),
            ("four", SPIN_DEAF),
            ("four", [line.replace("SPIN:PASS:CPUS=4", "SPIN:PASS:CPUS=3") for line in online(4)]),
            ("spin-irq-off-control", online(4)),
            ("spin-irq-off-control", SPIN_DEAF[:-5] + ["SMP:SPIN:PASS:CPUS=4"] + END),
            ("spin-irq-off-control", [line for line in SPIN_DEAF if "QUARANTINED" not in line]),
            ("spin-irq-off-control", [line.replace("SPIN:SHOOTDOWN_TIMEOUT", "SHOOTDOWN:TIMEOUT")
                                      for line in SPIN_DEAF]),
        ):
            with self.subTest(case=case, lines=lines[-6:]), self.assertRaises(ValueError):
                evaluate(case, transcript(lines), 33, False)
        self.assertEqual(evaluate("spin-irq-off-control", transcript(SPIN_DEAF), 33, False)["online"], 1)

    def test_switch_with_a_held_lock_is_terminal_and_exact(self):
        # Step 5c: the no-deferral profile must stop at the marker, inside the
        # preemption phase; a PASS there, or the marker elsewhere, is rejected.
        for case, lines in (
            ("held-across-switch", online(2)),
            ("held-across-switch", HELD_ACROSS[:-1] + ["SMP:PREEMPT:PASS:CPUS=2"]),
            ("held-across-switch", HELD_ACROSS + ["SMP:PREEMPT:PASS:CPUS=2"]),
            ("held-across-switch", HELD_ACROSS[:-2] + HELD_ACROSS[-1:]),
            ("two-single", HELD_ACROSS),
            ("two-single", online(2)[:-4] + ["SMP:LOCKDEP:HELD_ACROSS_SWITCH"] + online(2)[-4:]),
        ):
            with self.subTest(case=case, lines=lines[-4:]), self.assertRaises(ValueError):
                evaluate(case, transcript(lines), 33, False)
        self.assertEqual(evaluate("held-across-switch", transcript(HELD_ACROSS), 33, False)["online"], 2)

    def test_scheduler_phase_is_required_and_double_run_is_terminal(self):
        # Step 5d-1: a normal run must show the scheduler phase for every CPU;
        # the double-run profile must stop at its marker right after SPIN.
        for case, lines in (
            ("four", [line for line in online(4) if "SMP:SCHED:" not in line]),
            ("four", [line.replace("SCHED:PASS:CPUS=4", "SCHED:PASS:CPUS=3") for line in online(4)]),
            ("four", DOUBLE_RUN),
            ("double-run", online(4)),
            ("double-run", DOUBLE_RUN[:-1] + ["SMP:SCHED:PASS:CPUS=4", "SMP:SCHED:DOUBLE_RUN"]),
            ("double-run", DOUBLE_RUN + ["SMP:PARKED:APS=3"]),
        ):
            with self.subTest(case=case, lines=lines[-3:]), self.assertRaises(ValueError):
                evaluate(case, transcript(lines), 33, False)
        self.assertEqual(evaluate("double-run", transcript(DOUBLE_RUN), 33, False)["online"], 4)

    def test_lock_self_test_is_required_and_terminal_order_is_exact(self):
        for case, text in (
            ("four", transcript(online(4), lockdep=False)),           # self-test missing
            ("four", transcript(online(4)[:-2] + ["SMP:LOCKDEP:PASS"] * 2 + END, lockdep=False)),
            ("four", transcript(online(4)[:-3] + ["SMP:LOCKDEP:PASS"] + online(4)[-3:],
                                lockdep=False)),                      # before PARKED
            ("ap-ipi-deaf", transcript(DEAF, lockdep=False)),
            ("lockdep-order", transcript(online(2))),                  # order not detected
            ("lockdep-order", transcript(LOCKDEP_ORDER[:-1] + ["SMP:LOCKDEP:ORDER:10->30"])),
            ("lockdep-order", transcript(LOCKDEP_ORDER[:-1] + ["SMP:LOCKDEP:PASS",
                                                               LOCKDEP_ORDER[-1]])),
            ("lockdep-order", transcript(LOCKDEP_ORDER + END)),
        ):
            with self.subTest(case=case, text=text[-90:]), self.assertRaises(ValueError):
                evaluate(case, text, 33, False)
        self.assertEqual(evaluate("lockdep-order", transcript(LOCKDEP_ORDER), 33, False)["online"], 2)

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
