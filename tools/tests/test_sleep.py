"""HOST_ONLY S3 verifier counterexamples; fixtures are not native evidence."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
import unittest.mock

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
                observed = gate.evaluate(case, transcript(tail), gate.EXIT[case], False,
                                         events(case))
                self.assertEqual(observed["case"], case)
        self.assertEqual(gate.EXIT, {"cycles": 33, "corrupt-facs": 33, "stale-token": 37})
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
        for code in (33, 37):
            with self.assertRaises(ValueError):
                gate.evaluate("stale-token", transcript(stale), code, False, events("stale-token"))
        with self.assertRaises(ValueError):
            gate.evaluate("stale-token", transcript(TAILS["stale-token"]), 37, False, [])

    def test_a_rejected_wake_and_a_pass_never_share_an_exit(self):
        # The exact rejection tail with the pass exit, the generic failure exit
        # or no exit fails; so does any other case ending with the rejection exit.
        for code, timed_out in [(33, False), (35, False), (0, False), (None, True)]:
            with self.subTest(code=code), self.assertRaises(ValueError):
                gate.evaluate("stale-token", transcript(TAILS["stale-token"]), code, timed_out,
                              events("stale-token"))
        for case in ("cycles", "corrupt-facs"):
            with self.subTest(case=case), self.assertRaises(ValueError):
                gate.evaluate(case, transcript(TAILS[case]), 37, False, events(case))
        with self.assertRaises(ValueError):
            gate.expected_wakeups("unknown")


FAKE_QMP = """
import json, sys
emit = lambda value: print(json.dumps(value), flush=True)
emit({"QMP": {"version": {}}})
sys.stdin.readline()
emit({"return": {}, "id": 1})
for event in sys.argv[1:]:
    if event == "TRUNCATED":
        # A final message cut off by the end of stream (no newline).
        sys.stdout.write('{"event": "WAK')
        sys.stdout.flush()
    else:
        emit({"event": event, "timestamp": {}})
"""


FAKE_QEMU = """
import json, sys, time
args = sys.argv[1:]
serial = args[args.index("-serial") + 1].removeprefix("file:")
def out(line):
    with open(serial, "a") as stream:
        stream.write(line + "\\n")
def emit(value):
    sys.stdout.write(json.dumps(value) + "\\n")
    sys.stdout.flush()
emit({"QMP": {"version": {}}})
out("POWER:S3:ENTER:CYCLE=1:SCI_EN=1")
suspended = True
for line in sys.stdin:
    request = json.loads(line)
    name, ident = request["execute"], request.get("id")
    if name == "qmp_capabilities":
        emit({"return": {}, "id": ident})
        emit({"event": "SUSPEND"})
    elif name == "query-status":
        emit({"return": {"status": "suspended" if suspended else "running"}, "id": ident})
    elif name == "system_wakeup":
        suspended = False
        emit({"return": {}, "id": ident})
        # QEMU's main loop reports the wake only after the command reply.
        time.sleep(0.05)
        emit({"event": "WAKEUP"})
        out("POWER:S3:WAKE:REJECTED:TOKEN:ARMED=1:PAGE=0x5a5a0001")
        sys.exit(37)
"""


class ControllerLateWakeup(unittest.TestCase):
    def test_wakeup_after_the_final_reply_reaches_the_controller(self):
        with tempfile.TemporaryDirectory() as directory:
            cwd = Path(directory)
            run_dir = cwd / "run"
            run_dir.mkdir()
            args = [sys.executable, "-I", "-c", FAKE_QEMU, "-serial", "stdio"]
            result = gate.controlled_guest(args, cwd=cwd, env=dict(os.environ), timeout=30,
                                           run_dir=run_dir, case="stale-token")
        self.assertIsNone(result["error"])
        self.assertEqual(result["returncode"], 37)
        self.assertEqual([e for e in result["events"] if e in ("SUSPEND", "WAKEUP")],
                         ["SUSPEND", "WAKEUP"])
        self.assertEqual(len(result["observations"]), 1)
        self.assertIn(b"WAKE:REJECTED:TOKEN", result["stdout"])


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
        control = self.control("SUSPEND", "TRUNCATED")
        with self.assertRaises(RuntimeError):
            control.drain()
        self.assertEqual(control.events, ["SUSPEND"])


class MachinePin(unittest.TestCase):
    def test_other_machine_types_are_refused_before_launch(self):
        with unittest.mock.patch.dict(os.environ, {"NOS_QEMU_MACHINE": "pc-q35-10.0"}):
            with unittest.mock.patch.object(gate, "save_json"), \
                    self.assertRaisesRegex(ValueError, "pinned"):
                gate.run_case("cycles")


class FakeSystemQmp:
    """Scripted query-status answers; QEMU events are appended as they would arrive."""

    def __init__(self, wake, statuses, events=("SUSPEND", "WAKEUP")):
        self.wake, self.statuses, self.events, self.sent = wake, list(statuses), list(events), []

    def command(self, name, arguments=None):
        self.sent.append(name)
        if name == "system_wakeup":
            return {}
        status = self.statuses.pop(0)
        if status == "suspended" and "SUSPEND" in self.events:
            self.wake.events.append(self.events.pop(self.events.index("SUSPEND")))
        if status == "running" and "system_wakeup" in self.sent and "WAKEUP" in self.events:
            self.wake.events.append(self.events.pop(self.events.index("WAKEUP")))
        return {"status": status}

    def close(self):
        pass


class Alive:
    @staticmethod
    def poll():
        return None


class SystemWakeControl(unittest.TestCase):
    """H05-T4 power-s3 hook; HOST_ONLY, the native run is the evidence."""

    def wake(self, statuses, **options):
        holder = {}

        def connect(run_dir, process, deadline):
            holder["qmp"] = FakeSystemQmp(control, statuses, **options)
            return holder["qmp"], None
        control = gate.SystemWake(time.monotonic() + 30, connect=connect)
        return control, holder

    def test_connects_early_wakes_once_and_records_the_event_order(self):
        control, holder = self.wake(["running", "suspended", "suspended", "running"])
        with unittest.mock.patch.object(gate, "DWELL_S", 0):
            control("BOOT:EBS_OK", Alive, Path("."))
            self.assertIn("qmp", holder)  # connected on the first line, not on ENTER
            control("SYSTEM:MANIFEST:PASS", Alive, Path("."))
            control(gate.ENTER.format(1), Alive, Path("."))
            control(resumed(1), Alive, Path("."))
        control.check()
        self.assertEqual(control.evidence()["trace"],
                         ["query-status", "query-status", "query-status", "system_wakeup", "query-status"])
        self.assertEqual(control.evidence()["events"], ["SUSPEND", "WAKEUP"])

    def test_leaving_s3_during_the_dwell_is_refused(self):
        control, _ = self.wake(["suspended", "running"])
        with unittest.mock.patch.object(gate, "DWELL_S", 0), self.assertRaisesRegex(RuntimeError, "left S3"):
            control(gate.ENTER.format(1), Alive, Path("."))

    def test_a_second_entry_or_an_unrequested_resume_is_refused(self):
        control, _ = self.wake(["suspended", "suspended", "running"])
        with self.assertRaisesRegex(RuntimeError, "without a harness wake"):
            control(resumed(1), Alive, Path("."))
        with unittest.mock.patch.object(gate, "DWELL_S", 0):
            control(gate.ENTER.format(1), Alive, Path("."))
            with self.assertRaisesRegex(RuntimeError, "second S3 entry"):
                control(gate.ENTER.format(1), Alive, Path("."))

    def test_a_guest_that_never_suspends_before_exit_fails(self):
        class Exited:
            @staticmethod
            def poll():
                return 0
        control, _ = self.wake(["running"] * 4)
        with self.assertRaisesRegex(RuntimeError, "never suspended"):
            control(gate.ENTER.format(1), Exited, Path("."))

    def test_missing_events_or_wake_fail_the_check_and_the_allowlist_holds(self):
        control, _ = self.wake(["suspended", "suspended", "running"], events=("SUSPEND",))
        with unittest.mock.patch.object(gate, "DWELL_S", 0):
            control(gate.ENTER.format(1), Alive, Path("."))
        with self.assertRaisesRegex(ValueError, "S3 control evidence"):
            control.check()
        idle, _ = self.wake([])
        idle("BOOT:EBS_OK", Alive, Path("."))
        with self.assertRaisesRegex(ValueError, "S3 control evidence"):
            idle.check()
        with self.assertRaisesRegex(ValueError, "allowlist"):
            idle.command("quit")


class RunnerImportOrder(unittest.TestCase):
    def test_runner_modules_import_in_every_order(self):
        import itertools
        tools = Path(__file__).resolve().parents[1]
        for order in itertools.permutations(("run_sleep", "run_system", "run_net_link")):
            with self.subTest(order=order):
                subprocess.run([sys.executable, "-B", "-c", "; ".join(f"import {name}" for name in order)],
                               cwd=tools, check=True, capture_output=True)


if __name__ == "__main__":
    unittest.main()
