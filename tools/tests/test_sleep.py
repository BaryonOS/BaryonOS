"""HOST_ONLY S3 verifier counterexamples; fixtures are not native evidence."""
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_sleep as gate

ACPI = "ACPI:UNAVAILABLE:NO_DMAR\nDEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT\n"
PLAN = ("POWER:S3:PLAN:PM1A=0x0604:SLP_TYP=1:PM1A_STS=0x0600:FACS_VERSION=0:"
        "REAL=0x1000:PM32=0x2000460:POLICY=Edk2Version0Extended")


def resumed(cycle, status="0x8101", sci="RESTORED", entry="PM32", tail=None):
    tail = tail or "HWSIG=SAME:FACS=DISARMED:CANARY=PASS:CR3=SAME"
    return (f"POWER:S3:RESUMED:CYCLE={cycle}:ENTRY={entry}:PM1_STS={status}:WAK_STS=1:"
            f"SCI_EN={sci}:{tail}")


def cycles_tail():
    body = [PLAN]
    for cycle in range(1, gate.CYCLES + 1):
        body += [gate.ENTER.format(cycle), resumed(cycle)]
    return body + ["POWER:S3:PASS"]


TAILS = {
    "cycles": cycles_tail(),
    "corrupt-facs": ["POWER:S3:REJECTED:InvalidSignature"],
    "stale-token": [PLAN, gate.ENTER.format(1), gate.STALE],
}


def transcript(tail, boots=1):
    boot = "\n".join(gate.BOOT).replace("KS-02:PASS\n", "KS-02:PASS\n" + ACPI)
    return "BdsDxe: firmware\n" + "\n".join([boot] * boots + list(tail)) + "\n"


def events(case):
    return ["RTC_CHANGE"] + ["SUSPEND", "WAKEUP"] * gate.expected_wakeups(case)


class SleepVerifier(unittest.TestCase):
    def test_every_case_passes_with_its_exact_tail_exit_and_events(self):
        for case, tail in TAILS.items():
            with self.subTest(case=case):
                observed = gate.evaluate(case, transcript(tail), 33, False, events(case))
                self.assertEqual(observed["case"], case)
        observed = gate.evaluate("cycles", transcript(TAILS["cycles"]), 33, False,
                                 events("cycles"))
        self.assertEqual(len(observed["cycles"]), gate.CYCLES)

    def test_reboot_wrong_exit_timeout_and_missing_events_fail(self):
        tail = TAILS["cycles"]
        bad = [
            (transcript(tail, boots=2), 33, False, events("cycles")),  # resume rebooted
            (transcript(tail), 0, False, events("cycles")),  # firmware reset + -no-reboot
            (transcript(tail), None, True, events("cycles")),
            (transcript(tail), 33, False, ["SUSPEND", "WAKEUP"] * 2),  # one cycle not slept
            (transcript(tail), 33, False, ["WAKEUP", "SUSPEND"] * 3),
            (transcript(tail), 33, False, []),  # never actually suspended
        ]
        for stdout, code, timed_out, observed_events in bad:
            with self.subTest(code=code, events=observed_events):
                with self.assertRaises(ValueError):
                    gate.evaluate("cycles", stdout, code, timed_out, observed_events)

    def test_every_post_wake_check_must_pass(self):
        for field, value in [("ENTRY", "UNKNOWN"), ("ENTRY", "REAL"), ("SCI_EN", "LOST"),
                             ("SCI_EN", "OFF")]:
            tail = cycles_tail()
            tail[2] = tail[2].replace(f"{field}=" + {"ENTRY": "PM32", "SCI_EN": "RESTORED"}[field],
                                      f"{field}={value}")
            with self.subTest(field=field, value=value):
                with self.assertRaises(ValueError):
                    gate.evaluate("cycles", transcript(tail), 33, False, events("cycles"))
        for broken in ["HWSIG=CHANGED:FACS=DISARMED:CANARY=PASS:CR3=SAME",
                       "HWSIG=SAME:FACS=STORE_FAILED:CANARY=PASS:CR3=SAME",
                       "HWSIG=SAME:FACS=DISARMED:CANARY=FAIL:CR3=SAME",
                       "HWSIG=SAME:FACS=DISARMED:CANARY=PASS:CR3=CHANGED"]:
            tail = cycles_tail()
            tail[4] = resumed(2, tail=broken)
            with self.subTest(broken=broken):
                with self.assertRaises(ValueError):
                    gate.evaluate("cycles", transcript(tail), 33, False, events("cycles"))

    def test_cycle_order_count_and_wake_status_are_exact(self):
        tail = cycles_tail()
        tail[2], tail[4] = tail[4], tail[2]  # resumed markers swapped
        variants = [tail, cycles_tail()[:-1], cycles_tail() + ["POWER:S3:PASS"],
                    cycles_tail()[:1] + cycles_tail()[3:]]
        no_wake = cycles_tail()
        no_wake[2] = resumed(1, status="0x0101")
        variants.append(no_wake)
        for variant in variants:
            with self.subTest(variant=variant[1:4]):
                with self.assertRaises(ValueError):
                    gate.evaluate("cycles", transcript(variant), 33, False, events("cycles"))

    def test_plan_bounds_and_policy_are_checked(self):
        for plan in [PLAN.replace("REAL=0x1000", "REAL=0xa0000"),
                     PLAN.replace("REAL=0x1000", "REAL=0x1010"),
                     PLAN.replace("PM32=0x2000460", "PM32=0x2000468"),
                     PLAN.replace("PM32=0x2000460", "PM32=0x100000000"),
                     PLAN.replace("Edk2Version0Extended", "Specification"),
                     PLAN.replace("FACS_VERSION=0", "FACS_VERSION=2")]:
            tail = cycles_tail()
            tail[0] = plan
            with self.subTest(plan=plan[-60:]):
                with self.assertRaises(ValueError):
                    gate.evaluate("cycles", transcript(tail), 33, False, events("cycles"))

    def test_negative_fixtures_cannot_pass_by_resuming_or_sleeping(self):
        # A corrupt FACS must reject before any sleep.
        with self.assertRaises(ValueError):
            gate.evaluate("corrupt-facs", transcript(TAILS["corrupt-facs"]), 33, False,
                          ["SUSPEND", "WAKEUP"])
        with self.assertRaises(ValueError):
            gate.evaluate("corrupt-facs", transcript(cycles_tail()), 33, False, [])
        # A stale token must never reach RESUMED, and must have slept once.
        stale = [PLAN, gate.ENTER.format(1), resumed(1)]
        with self.assertRaises(ValueError):
            gate.evaluate("stale-token", transcript(stale), 33, False, events("stale-token"))
        with self.assertRaises(ValueError):
            gate.evaluate("stale-token", transcript(TAILS["stale-token"]), 33, False, [])
        with self.assertRaises(ValueError):
            gate.expected_wakeups("unknown")


FAKE_QMP = """
import json, sys
emit = lambda value: print(json.dumps(value), flush=True)
emit({"QMP": {"version": {}}})
sys.stdin.readline()
emit({"return": {}, "id": 1})
for event in sys.argv[1:]:
    emit({"event": event, "timestamp": {}})
"""


class EventDrain(unittest.TestCase):
    def control(self, *events):
        process = subprocess.Popen([sys.executable, "-I", "-c", FAKE_QMP, *events],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        self.addCleanup(process.wait)
        control = gate.EventPipe(process, time.monotonic() + 10)
        self.addCleanup(control.close)
        process.wait(timeout=10)
        return control

    def test_wakeup_after_the_final_reply_is_collected(self):
        # QEMU emits WAKEUP from its main loop, after the system_wakeup reply.
        control = self.control("SUSPEND", "WAKEUP")
        control.drain()
        self.assertEqual(control.events, ["SUSPEND", "WAKEUP"])

    def test_clean_end_of_stream_without_events_drains_to_nothing(self):
        control = self.control()
        control.drain()
        self.assertEqual(control.events, [])

    def test_truncated_final_message_is_an_error(self):
        control = self.control("SUSPEND")
        control.pending.extend(b'{"event": "WAK')
        with self.assertRaises(RuntimeError):
            control.drain()


class MachinePin(unittest.TestCase):
    def test_other_machine_types_are_refused_before_launch(self):
        with mock.patch.dict(os.environ, {"NOS_QEMU_MACHINE": "pc-q35-10.0"}):
            with mock.patch.object(gate, "save_json"), self.assertRaisesRegex(ValueError, "pinned"):
                gate.run_case("cycles")


if __name__ == "__main__":
    unittest.main()
