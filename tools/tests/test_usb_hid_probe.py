"""Negative oracles and process-lifetime checks for the native USB fixture."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_usb_hid as probe


def transcript(case="reports"):
    return "\n".join(probe.BOOT + ("XHCI:BEGIN",) + probe.markers(case) + ("XHCI:PASS",)) + "\n"


class OracleTests(unittest.TestCase):
    def test_reports_and_readonly_are_distinct(self):
        for case in probe.FEATURES:
            result = probe.evaluate(case, transcript(case), 33, False)
            self.assertEqual(result["usb_reports"], case != "readonly" and case not in probe.REFUSALS)
            self.assertEqual(result["same_port_reattach"], case in ("reattach", "reattach-mouse"))
            self.assertEqual(result["reattach_device"],
                             {"reattach": "keyboard", "reattach-mouse": "mouse"}.get(case))
            self.assertEqual(result["refusal_reason"], probe.REFUSALS.get(case))
            self.assertFalse(result["input_policy_authority"])
            for other in probe.FEATURES:
                # Both are refused on speed before Enable Slot; the guest cannot tell them apart.
                if {case, other} == {"refuse-hs-kbd", "refuse-storage"}:
                    self.assertEqual(transcript(case), transcript(other))
                elif other != case:
                    with self.subTest(case=case, other=other), self.assertRaises(ValueError):
                        probe.evaluate(case, transcript(other), 33, False)

    def test_reattach_markers_are_mandatory_unique_and_ordered(self):
        self.assertEqual(probe.FEATURES["reattach"], "native-usb-hid-reattach")
        expected = probe.markers("reattach")
        self.assertEqual(expected[:len(probe.START) + len(probe.REPORTS)], probe.START + probe.REPORTS)
        self.assertEqual(expected[-len(probe.END):], probe.END)
        for marker in probe.REATTACH:
            with self.subTest(missing=marker), self.assertRaises(ValueError):
                probe.evaluate("reattach", transcript("reattach").replace(marker + "\n", ""), 33, False)
            with self.subTest(duplicate=marker), self.assertRaises(ValueError):
                probe.evaluate("reattach", transcript("reattach").replace(marker + "\n", marker + "\n" + marker + "\n"),
                               33, False)
        swaps = [("USB:REATTACH:KEY_DOWN", "USB:REATTACH:KEY_UP"),
                 ("USB:REATTACH:DETACHED", "USB:REATTACH:ENUM:KEYBOARD"),
                 ("USB:REATTACH:STALE_DENIED", "USB:REATTACH:READY"),
                 ("USB:REATTACH:UNPLUG:PASS", "USB:ATTACH:BUDGET_DENIED"),
                 ("USB:REPORTS:PASS", "USB:REATTACH:DETACHED"),
                 ("USB:RETIRED:UNCHANGED", "USB:GUARD:UNCHANGED")]
        for first, second in swaps:
            swapped = transcript("reattach").replace(first + "\n" + second, second + "\n" + first)
            self.assertNotEqual(swapped, transcript("reattach"))
            with self.subTest(swap=first), self.assertRaises(ValueError):
                probe.evaluate("reattach", swapped, 33, False)
        # A different slot is an unaccepted observation, and an extra attach
        # after budget denial is evidence of a missing refusal.
        for corrupt in (transcript("reattach").replace("SLOT_REUSED", "SLOT_NEW"),
                        transcript("reattach").replace("USB:ATTACH:BUDGET_DENIED\n",
                                                       "USB:ATTACH:BUDGET_DENIED\nUSB:ENUM:KEYBOARD\n"),
                        transcript("reattach") + "USB-DIAG:REJECT:45\n"):
            with self.assertRaises(ValueError):
                probe.evaluate("reattach", corrupt, 33, False)

    def test_mouse_reattach_markers_are_mandatory_unique_and_ordered(self):
        self.assertEqual(probe.FEATURES["reattach-mouse"], "native-usb-hid-mouse-reattach")
        good = transcript("reattach-mouse")
        expected = probe.markers("reattach-mouse")
        self.assertEqual(expected, probe.START + probe.REPORTS + probe.REATTACH_MOUSE + probe.END)
        self.assertEqual(probe.evaluate("reattach-mouse", good, 33, False)["markers"], list(expected))
        for marker in probe.REATTACH_MOUSE:
            with self.subTest(missing=marker), self.assertRaises(ValueError):
                probe.evaluate("reattach-mouse", good.replace(marker + "\n", ""), 33, False)
            with self.subTest(duplicate=marker), self.assertRaises(ValueError):
                probe.evaluate("reattach-mouse", good.replace(marker + "\n", marker + "\n" + marker + "\n"),
                               33, False)
        swaps = [("USB:REPORTS:PASS", "USB:REATTACH:HELD_BUTTON"),
                 ("USB:REATTACH:HELD_BUTTON", "USB:REATTACH:DETACHED"),
                 ("USB:REATTACH:DETACHED", "USB:REATTACH:ENUM:MOUSE"),
                 ("USB:REATTACH:MOUSE_DOWN", "USB:REATTACH:MOUSE_UP"),
                 ("USB:REATTACH:UNPLUG:PASS", "USB:ATTACH:BUDGET_DENIED")]
        for first, second in swaps:
            swapped = good.replace(first + "\n" + second, second + "\n" + first)
            self.assertNotEqual(swapped, good)
            with self.subTest(swap=first), self.assertRaises(ValueError):
                probe.evaluate("reattach-mouse", swapped, 33, False)
        # Only reuse of the retired keyboard's slot is the accepted observation.
        slot = "USB:REATTACH:SLOT_REUSED:KEYBOARD\n"
        corrupt = [good.replace(slot, "USB:REATTACH:SLOT_NEW\n"),
                   good.replace(slot, "USB:REATTACH:SLOT_REUSED:MOUSE\n"),
                   good.replace(slot, "USB:REATTACH:SLOT_REUSED\n"),
                   good.replace("USB:REATTACH:ENUM:MOUSE\n", "USB:REATTACH:ENUM:KEYBOARD\n")]
        for extra in ("USB:ENUM:MOUSE", "USB:REATTACH:ENUM:MOUSE", "USB:ENUM:KEYBOARD"):
            corrupt.append(good.replace("USB:ATTACH:BUDGET_DENIED\n", "USB:ATTACH:BUDGET_DENIED\n" + extra + "\n"))
        corrupt += [good + f"USB-DIAG:REJECT:{code}\n" for code in range(52, 57)]
        corrupt.append(good.replace("USB:REATTACH:MOUSE_DOWN\n", "USB:REATTACH:KEY_DOWN\n"))
        for value in corrupt:
            self.assertNotEqual(value, good)
            with self.subTest(corrupt=value[-80:]), self.assertRaises(ValueError):
                probe.evaluate("reattach-mouse", value, 33, False)
        for exit_code, timed_out in ((0, False), (33, True)):
            with self.assertRaises(ValueError):
                probe.evaluate("reattach-mouse", good, exit_code, timed_out)

    def test_refusal_markers_are_mandatory_unique_and_ordered(self):
        self.assertEqual(probe.REFUSALS, {"refuse-hub": "HUB", "refuse-tablet": "NOT_BOOT",
                                          "refuse-hs-kbd": "SPEED", "refuse-storage": "SPEED"})
        for case, reason in probe.REFUSALS.items():
            expected = probe.markers(case)
            self.assertEqual(probe.FEATURES[case], "native-usb-hid-refusal")
            slot = ("USB:REFUSE:SLOT_DISABLED",) if reason != "SPEED" else ()
            self.assertEqual(expected, probe.START + slot + ("USB:REFUSE:" + reason,) + probe.REFUSAL + probe.END)
            self.assertNotIn("USB:REPORTS:PASS", expected)
            good = transcript(case)
            for marker in slot + ("USB:REFUSE:" + reason,) + probe.REFUSAL:
                with self.subTest(case=case, missing=marker), self.assertRaises(ValueError):
                    probe.evaluate(case, good.replace(marker + "\n", ""), 33, False)
                with self.subTest(case=case, duplicate=marker), self.assertRaises(ValueError):
                    probe.evaluate(case, good.replace(marker + "\n", marker + "\n" + marker + "\n"), 33, False)
            swapped = good.replace("USB:REFUSE:MOUSE_DOWN\nUSB:REFUSE:MOUSE_UP",
                                   "USB:REFUSE:MOUSE_UP\nUSB:REFUSE:MOUSE_DOWN")
            self.assertNotEqual(swapped, good)
            with self.subTest(case=case, reordered=True), self.assertRaises(ValueError):
                probe.evaluate(case, swapped, 33, False)
            # Serving the refused device would add evidence the oracle must reject.
            with self.subTest(case=case, served=True), self.assertRaises(ValueError):
                probe.evaluate(case, good.replace("USB:REFUSE:PORT_HELD\n", "USB:ENUM:KEYBOARD\nUSB:REFUSE:PORT_HELD\n"), 33, False)

    def test_all_mandatory_markers_are_required(self):
        for marker in probe.BOOT + ("XHCI:BEGIN",) + probe.markers("reports") + ("XHCI:PASS",):
            with self.subTest(marker=marker), self.assertRaises(ValueError):
                probe.evaluate("reports", transcript().replace(marker + "\n", ""), 33, False)

    def test_duplicates_reordering_and_unknown_evidence_reject(self):
        corrupt = [transcript() + "USB:PASS\n", transcript() + "USB:UNKNOWN\n",
                   transcript().replace("USB:KEY_DOWN\nUSB:KEY_UP", "USB:KEY_UP\nUSB:KEY_DOWN"),
                   transcript() + "BOOT:EBS_OK\n", transcript() + "XHCI:NOOP_COMPLETION:PASS\n"]
        for value in corrupt:
            with self.assertRaises(ValueError):
                probe.evaluate("reports", value, 33, False)

    def test_failure_timeout_and_wrong_exit_reject(self):
        for output, exit_code, timed_out in [(transcript() + "PANIC\n", 33, False),
                                            (transcript() + "USB-DIAG:REJECT:1\n", 33, False),
                                            (transcript(), 0, False), (transcript(), 33, True)]:
            with self.assertRaises(ValueError):
                probe.evaluate("reports", output, exit_code, timed_out)

    def test_case_is_closed(self):
        with self.assertRaises(ValueError):
            probe.markers("physical")


class FakeProcess:
    def __init__(self, *args, **kwargs):
        self.returncode = None
        self.killed = False
        self.reaped = False

    def poll(self):
        return self.returncode

    def kill(self):
        self.killed = True
        self.returncode = -9

    def wait(self, timeout=None):
        if timeout is not None and self.returncode is None:
            raise subprocess.TimeoutExpired("fixture", timeout)
        self.reaped = True
        return self.returncode


class ExitingProcess(FakeProcess):
    def wait(self, timeout=None):
        self.returncode = 33
        self.reaped = True
        return self.returncode


class FakeQmp:
    fail = None

    def __init__(self, process, deadline):
        self.deadline = deadline
        self.trace = []
        self.closed = False

    def command(self, name, arguments=None):
        request = {"execute": name, "arguments": arguments}
        if (name, (arguments or {}).get("id")) == self.fail:
            # QmpPipe records an error reply before raising it.
            self.trace.append({"request": request, "response": {"error": {"class": "GenericError"}}})
            raise RuntimeError("QMP command failed: GenericError")
        self.trace.append({"request": request, "response": {"return": {}}})

    def close(self):
        self.closed = True


class FailingAddQmp(FakeQmp):
    fail = ("device_add", "hid-kbd-g2")


class FailingMouseAddQmp(FakeQmp):
    fail = ("device_add", "hid-mouse-g2")


def waits(seen, fail=None):
    def wait(process, path, marker, deadline):
        seen.append(marker)
        if marker == fail:
            raise RuntimeError("Guest rejected USB fixture before " + marker)
    return wait


REATTACH_WAITS = ["USB:READY", "USB:KEY_DOWN", "USB:KEY_UP", "USB:MOUSE_DOWN", "USB:MOUSE_UP",
                  "USB:HELD_KEY", "USB:STALE_TOKEN:DENIED", "USB:REATTACH:DETACHED",
                  "USB:REATTACH:READY", "USB:REATTACH:KEY_DOWN", "USB:REATTACH:KEY_UP",
                  "USB:REATTACH:UNPLUG:PASS"]


MOUSE_WAITS = REATTACH_WAITS[:7] + ["USB:REPORTS:PASS", "USB:REATTACH:HELD_BUTTON",
                                     "USB:REATTACH:DETACHED", "USB:REATTACH:READY",
                                     "USB:REATTACH:MOUSE_DOWN", "USB:REATTACH:MOUSE_UP",
                                     "USB:REATTACH:UNPLUG:PASS"]


REFUSAL_WAITS = ["USB:REFUSE:READY", "USB:REFUSE:MOUSE_DOWN", "USB:REFUSE:MOUSE_UP",
                 "USB:REFUSE:UNPLUG:PASS", "USB:REFUSE:RECOVER:READY", "USB:REFUSE:RECOVER:KEY_DOWN"]


class FailingRecoverAddQmp(FakeQmp):
    fail = ("device_add", "hid-kbd-g2")


class LifecycleTests(unittest.TestCase):
    def test_early_guest_failure_is_not_waited_to_timeout(self):
        with tempfile.TemporaryDirectory() as directory:
            serial = Path(directory) / "stdout"
            serial.write_text("USB-DIAG:REJECT:10\n")
            with self.assertRaisesRegex(RuntimeError, "Guest rejected"):
                probe._wait_marker(FakeProcess(), serial, "USB:READY", time.monotonic() + 30)

    def test_early_exit_and_expired_deadline_reject(self):
        with tempfile.TemporaryDirectory() as directory:
            serial = Path(directory) / "stdout"
            serial.write_bytes(b"")
            process = FakeProcess()
            process.returncode = 33
            with self.assertRaisesRegex(RuntimeError, "Guest exited"):
                probe._wait_marker(process, serial, "USB:READY", time.monotonic() + 30)
            with self.assertRaises(TimeoutError):
                probe._wait_marker(FakeProcess(), serial, "USB:READY", time.monotonic() - 1)

    def test_output_is_bounded(self):
        with tempfile.TemporaryDirectory() as directory:
            serial = Path(directory) / "stdout"
            with serial.open("wb") as stream:
                stream.truncate(probe.MAX_OUTPUT + 1)
            with self.assertRaisesRegex(RuntimeError, "exceeds"):
                probe._output(serial)

    def test_failure_releases_held_key_and_reaps_process(self):
        process = FakeProcess()
        with tempfile.TemporaryDirectory() as directory, \
                patch.object(probe.subprocess, "Popen", return_value=process), \
                patch.object(probe, "QmpPipe", FakeQmp), \
                patch.object(probe, "_wait_marker", side_effect=[None, RuntimeError("guest failed")]):
            args = ["qemu", "-serial", "stdio"]
            result = probe.controlled_guest(args, cwd=directory, env={}, timeout=5, run_dir=directory, case="reports")
            self.assertIn("guest failed", result["error"])
            self.assertTrue(process.killed and process.reaped)
            self.assertEqual(result["trace"][-1]["request"]["arguments"], {"events": [probe._key("a", False)]})
            self.assertEqual(args[-2], "-qmp")
            self.assertEqual(args[-1], "stdio")
            self.assertEqual(args[2], "file:usb-stdout.bin")

    def test_readonly_never_waits_for_input_and_timeout_reaps(self):
        process = FakeProcess()
        with tempfile.TemporaryDirectory() as directory, \
                patch.object(probe.subprocess, "Popen", return_value=process), \
                patch.object(probe, "QmpPipe", FakeQmp), \
                patch.object(probe, "_wait_marker") as marker:
            result = probe.controlled_guest(["qemu", "-serial", "stdio"], cwd=directory, env={}, timeout=5, run_dir=directory, case="readonly")
            marker.assert_not_called()
            self.assertEqual(result["trace"], [])
            self.assertTrue(result["timed_out"])
            self.assertTrue(process.killed and process.reaped)

    def _reattach(self, process, qmp, fail=None, case="reattach"):
        seen = []
        with tempfile.TemporaryDirectory() as directory, \
                patch.object(probe.subprocess, "Popen", return_value=process), \
                patch.object(probe, "QmpPipe", qmp), \
                patch.object(probe, "_wait_marker", side_effect=waits(seen, fail)):
            result = probe.controlled_guest(["qemu", "-serial", "stdio"], cwd=directory, env={}, timeout=5,
                                            run_dir=directory, case=case)
        requests = [(entry["request"]["execute"], entry["request"]["arguments"]) for entry in result["trace"]]
        return result, seen, requests

    def test_reattach_sequence_is_exact_and_ends_with_a_refused_attach(self):
        process = ExitingProcess()
        result, seen, requests = self._reattach(process, FakeQmp)
        self.assertIsNone(result["error"])
        self.assertEqual(result["returncode"], 33)
        self.assertEqual(seen, REATTACH_WAITS)
        self.assertEqual(requests[-7:], [
            ("device_del", {"id": "hid-kbd"}),
            ("input-send-event", {"events": [probe._key("b", False)]}),
            ("device_add", probe._keyboard("hid-kbd-g2")),
            ("input-send-event", {"events": [probe._key("c", True)]}),
            ("input-send-event", {"events": [probe._key("c", False)]}),
            ("device_del", {"id": "hid-kbd-g2"}),
            ("device_add", probe._keyboard("hid-kbd-g3"))])
        self.assertEqual(probe._keyboard("hid-kbd-g3"), {"driver": "usb-kbd", "id": "hid-kbd-g3",
                                                          "bus": "xhci.0", "port": "1", "usb_version": 1})
        self.assertFalse(process.killed)
        self.assertTrue(process.reaped)

    def test_reattach_device_add_failure_reaps_without_further_input(self):
        process = FakeProcess()
        result, seen, requests = self._reattach(process, FailingAddQmp)
        self.assertIn("QMP command failed", result["error"])
        self.assertEqual(seen[-1], "USB:REATTACH:DETACHED")
        # Key b was already released; nothing is held, so no cleanup input.
        self.assertEqual(requests[-1], ("device_add", probe._keyboard("hid-kbd-g2")))
        self.assertNotIn(("device_add", probe._keyboard("hid-kbd-g3")), requests)
        self.assertTrue(process.killed and process.reaped)

    def test_reattach_failures_release_every_held_key(self):
        for fail, held in (("USB:STALE_TOKEN:DENIED", "b"), ("USB:REATTACH:KEY_DOWN", "c")):
            with self.subTest(fail=fail):
                process = FakeProcess()
                result, _, requests = self._reattach(process, FakeQmp, fail)
                self.assertIn("Guest rejected", result["error"])
                self.assertEqual(requests[-1], ("input-send-event", {"events": [probe._key(held, False)]}))
                self.assertTrue(process.killed and process.reaped)

    def test_mouse_reattach_sequence_is_exact_and_ends_with_a_refused_attach(self):
        process = ExitingProcess()
        result, seen, requests = self._reattach(process, FakeQmp, case="reattach-mouse")
        self.assertIsNone(result["error"])
        self.assertEqual(result["returncode"], 33)
        self.assertEqual(seen, MOUSE_WAITS)
        self.assertEqual(len(requests), 15)
        self.assertEqual(requests, [
            ("input-send-event", {"events": [probe._key("a", True)]}),
            ("input-send-event", {"events": [probe._key("a", False)]}),
            ("input-send-event", {"events": [probe._move("x", 7), probe._move("y", -3), probe._button(True)]}),
            ("input-send-event", {"events": [probe._button(False)]}),
            ("input-send-event", {"events": [probe._key("b", True)]}),
            ("device_del", {"id": "hid-kbd"}),
            ("input-send-event", {"events": [probe._key("b", False)]}),
            ("input-send-event", {"events": [probe._button(True)]}),
            ("device_del", {"id": "hid-mouse"}),
            ("input-send-event", {"events": [probe._button(False)]}),
            ("device_add", probe._mouse("hid-mouse-g2")),
            ("input-send-event", {"events": [probe._move("x", -4), probe._move("y", 6), probe._button(True)]}),
            ("input-send-event", {"events": [probe._button(False)]}),
            ("device_del", {"id": "hid-mouse-g2"}),
            ("device_add", probe._mouse("hid-mouse-g3"))])
        self.assertEqual(probe._mouse("hid-mouse-g3"), {"driver": "usb-mouse", "id": "hid-mouse-g3",
                                                        "bus": "xhci.0", "port": "2", "usb_version": 1})
        self.assertFalse(process.killed)
        self.assertTrue(process.reaped)

    def test_mouse_reattach_device_add_failure_reaps_without_further_input(self):
        process = FakeProcess()
        result, seen, requests = self._reattach(process, FailingMouseAddQmp, case="reattach-mouse")
        self.assertIn("QMP command failed", result["error"])
        self.assertEqual(seen[-1], "USB:REATTACH:DETACHED")
        # The button was already released; nothing is held, so no cleanup input.
        self.assertEqual(requests[-1], ("device_add", probe._mouse("hid-mouse-g2")))
        self.assertNotIn(("device_add", probe._mouse("hid-mouse-g3")), requests)
        self.assertTrue(process.killed and process.reaped)

    def test_mouse_reattach_failures_release_every_held_input(self):
        released_b = ("input-send-event", {"events": [probe._key("b", False)]})
        released_left = ("input-send-event", {"events": [probe._button(False)]})
        for fail, cleanup in (("USB:STALE_TOKEN:DENIED", released_b),
                              # B was released by the sequence; nothing is held yet.
                              ("USB:REPORTS:PASS", released_b),
                              ("USB:REATTACH:HELD_BUTTON", released_left),
                              ("USB:REATTACH:DETACHED", released_left),
                              ("USB:REATTACH:MOUSE_DOWN", released_left)):
            with self.subTest(fail=fail):
                process = FakeProcess()
                result, _, requests = self._reattach(process, FakeQmp, fail, case="reattach-mouse")
                self.assertIn("Guest rejected", result["error"])
                self.assertEqual(requests[-1], cleanup)
                if fail == "USB:REPORTS:PASS":
                    # Only the sequence's own release: no extra cleanup input.
                    self.assertEqual(requests.count(released_b), 1)
                self.assertTrue(process.killed and process.reaped)

    def test_refusal_sequence_is_exact(self):
        for case in probe.REFUSALS:
            with self.subTest(case=case):
                process = ExitingProcess()
                result, seen, requests = self._reattach(process, FakeQmp, case=case)
                self.assertIsNone(result["error"])
                self.assertEqual(seen, REFUSAL_WAITS)
                self.assertEqual(requests, [
                    ("input-send-event", {"events": [probe._move("x", 7), probe._move("y", -3), probe._button(True)]}),
                    ("input-send-event", {"events": [probe._button(False)]}),
                    ("device_del", {"id": "usb-refused"}),
                    ("device_add", probe._keyboard("hid-kbd-g2")),
                    ("input-send-event", {"events": [probe._key("a", True)]}),
                    ("input-send-event", {"events": [probe._key("a", False)]})])
                self.assertFalse(process.killed)
                self.assertTrue(process.reaped)

    def test_refusal_failures_release_every_held_input(self):
        released_left = ("input-send-event", {"events": [probe._button(False)]})
        released_a = ("input-send-event", {"events": [probe._key("a", False)]})
        for fail, cleanup in (("USB:REFUSE:MOUSE_DOWN", released_left),
                              ("USB:REFUSE:RECOVER:KEY_DOWN", released_a)):
            with self.subTest(fail=fail):
                process = FakeProcess()
                result, _, requests = self._reattach(process, FakeQmp, fail, case="refuse-hub")
                self.assertIn("Guest rejected", result["error"])
                self.assertEqual(requests[-1], cleanup)
                self.assertTrue(process.killed and process.reaped)
        process = FakeProcess()
        result, seen, requests = self._reattach(process, FailingRecoverAddQmp, case="refuse-tablet")
        self.assertIn("QMP command failed", result["error"])
        self.assertEqual(seen[-1], "USB:REFUSE:UNPLUG:PASS")
        self.assertEqual(requests[-1], ("device_add", probe._keyboard("hid-kbd-g2")))
        self.assertTrue(process.killed and process.reaped)

    def test_refused_device_arguments(self):
        self.assertEqual(probe.refused_device("refuse-hub", None)[-1].split(",")[:2], ["usb-hub", "id=usb-refused"])
        with self.assertRaises(ValueError):
            probe.refused_device("refuse-storage", "/a,b/disk.img")

    def test_failed_manifest_leaves_failed_report(self):
        with tempfile.TemporaryDirectory() as directory, patch.object(probe, "BUILD", Path(directory)), \
                patch.object(probe, "source_hash", return_value="current"):
            manifest = Path(directory) / "manifest.json"
            manifest.write_text(json.dumps({"source_sha256": "stale"}))
            with self.assertRaises(ValueError):
                probe.run_case("reports")
            saved = json.loads((Path(directory) / "usb-hid-reports-report.json").read_text())
            self.assertFalse(saved["passed"])
            self.assertIn("Build the current", saved["error"])


if __name__ == "__main__":
    unittest.main()
