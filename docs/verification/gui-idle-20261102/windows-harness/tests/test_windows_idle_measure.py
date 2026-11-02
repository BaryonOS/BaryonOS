"""Deterministic CPU/Win32/idle-hook negatives. No QEMU or guest execution."""
from __future__ import annotations

import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from windows_idle_measure import (ENTER, Win32CpuCounters, WindowsIdleMeasurement,
                                 counter_delta, cpu_measurement, digest, vcpu_id)


def image(width=1280, height=720, value=17):
    return f"P6\n{width} {height}\n255\n".encode() + bytes([value]) * (width * height * 3)


FRAME = image()
ORACLE = image(880, 510)


class Clock:
    def __init__(self):
        self.now = 100.0
        self.sleeps = []
        self.on_sleep = None

    def __call__(self):
        return self.now

    def sleep(self, seconds):
        self.sleeps.append(seconds)
        self.now += seconds
        if self.on_sleep:
            self.on_sleep()


class Process:
    pid = 123
    _handle = 1000
    returncode = None

    def poll(self):
        return self.returncode


class Counters:
    def __init__(self, process, tid, clock):
        self.clock, self.tid = clock, tid
        self.closed = 0
        self.reads = 0
        self.close_error = False

    def snapshot(self):
        self.reads += 1
        result = {}
        for role, rate in (("process", 0.5), ("vcpu", 0.25)):
            start = self.clock.now
            self.clock.now += 0.001
            cpu = round((start + 0.0005) * 1e7 * rate)
            result[role] = {"creation_100ns": 1, "kernel_100ns": 0, "user_100ns": cpu,
                            "total_100ns": cpu, "read_started": start, "read_completed": self.clock.now}
        return result

    def close(self):
        self.closed += 1
        if self.close_error:
            raise OSError("fake close failure")


class Runner:
    def __init__(self, build):
        self.BUILD = build
        self.hash = "a" * 64
        self.calls = 0
        self.change_source = False

    def source_hash(self):
        self.calls += 1
        return "b" * 64 if self.change_source and self.calls > 1 else self.hash

    @staticmethod
    def private_path(path, **kwargs):
        return Path(path)

    @staticmethod
    def ppm(raw):
        magic, geometry, limit, pixels = raw.split(b"\n", 3)
        width, height = map(int, geometry.split())
        if magic != b"P6" or limit != b"255" or len(pixels) != width * height * 3:
            raise ValueError("Malformed PPM")
        return width, height, pixels

    def material_matches(self, raw, expected):
        width, height, pixels = self.ppm(raw)
        ew, eh, wanted = self.ppm(expected)
        if (ew, eh) != (880, 510) or width < 908 or height < 624:
            return False
        crop = b"".join(pixels[((114 + y) * width + 28) * 3:((114 + y) * width + 908) * 3] for y in range(510))
        return crop == wanted

    @staticmethod
    def serial(path):
        raw = Path(path).read_bytes()
        if b"WINDOW:FAIL:" in raw or b"PANIC" in raw:
            raise ValueError("Guest failure")
        return raw

    @staticmethod
    def numbers(raw, name):
        return json.loads(raw)["numbers"].get(name, [])

    @staticmethod
    def snapshots(raw, role):
        return json.loads(raw)["snapshots"].get(role, [])


def initial_state():
    app = {"PID": 10, "SERVICE_GENERATION": 1, "READY": 1, "PROBES": 3, "GEN": 1, "FRAMES": 1}
    comp = {"PID": 20, "GENERATION": 1, "FOCUSED": 0, "POINTER_X": 1260, "POINTER_Y": 50,
            "BUTTONS": 0, "FATAL": 0, "MATERIAL": 1, "CALCULATOR": 1, "COMMITS": 310}
    return {"snapshots": {"MATERIAL": [app.copy()], "CALCULATOR": [app.copy()], "COMPOSITOR": [comp]},
            "numbers": {"WINDOW:INITMASK": [31], "WINDOW:RECOVERIES": [0],
                        "WINDOW:MATERIAL:KEYS": [0], "WINDOW:MATERIAL:INPUT_ACKS": [0],
                        "WINDOW:CALCULATOR:KEYS": [0], "WINDOW:CALCULATOR:INPUT_ACKS": [0]}}


class HookTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        build = Path(self.temp.name)
        self.directory = build / "run"
        self.directory.mkdir()
        (self.directory / "initial.ppm").write_bytes(FRAME)
        self.state = initial_state()
        self.write_state()
        self.clock, self.process, self.runner = Clock(), Process(), Runner(build)
        self.fixture = {"metadata": {"source_sha256": self.runner.hash}, "frames": {"initial": ORACLE}}
        self.commands, self.counters = [], []
        self.change_after_frame = False
        self.change_after_state = False
        self.change_thread = False
        self.map_reads = 0

    def write_state(self):
        (self.directory / "serial.txt").write_bytes(json.dumps(self.state).encode())

    def factory(self, process, tid):
        counters = Counters(process, tid, self.clock)
        self.counters.append(counters)
        return counters

    def command(self, name, arguments=None):
        self.commands.append((name, copy.deepcopy(arguments), self.clock.now))
        if name == "query-status":
            return {"running": True, "status": "running"}
        if name == "query-cpus-fast":
            self.map_reads += 1
            return [{"cpu-index": 0, "thread-id": 78 if self.change_thread and self.map_reads > 1 else 77}]
        if name == "screendump":
            path = Path(arguments["filename"])
            if path.name == "after.ppm" and self.change_after_state:
                self.state["snapshots"]["COMPOSITOR"][-1]["COMMITS"] += 1
                self.write_state()
            # Change a system-chrome pixel, leaving the Material oracle exact.
            raw = FRAME[:-1] + b"\x12" if path.name == "after.ppm" and self.change_after_frame else FRAME
            path.write_bytes(raw)
            return {}
        raise AssertionError("Unapproved QMP command: " + name)

    def helper(self, **kwargs):
        return WindowsIdleMeasurement(self.runner, self.fixture, self.process, self.directory,
            kwargs.pop("deadline", 200), counters_factory=self.factory, clock=self.clock, sleep=self.clock.sleep, **kwargs)

    def invoke(self, helper=None):
        helper = helper or self.helper()
        return helper.before_first_enter(self.command, "send-key", copy.deepcopy(ENTER))

    def test_success_has_separate_process_vcpu_windows_and_no_input_during_15s(self):
        helper = self.helper(expected_initial_sha256=digest(FRAME))
        result = self.invoke(helper)
        self.assertTrue(result["measurement_passed"])
        self.assertFalse(result["native_gate"])
        self.assertGreaterEqual(result["cpu"]["quiet_seconds"], 15)
        self.assertAlmostEqual(result["cpu"]["process"]["percent_one_logical_cpu"], 50, places=4)
        self.assertAlmostEqual(result["cpu"]["vcpu"]["percent_one_logical_cpu"], 25, places=4)
        self.assertEqual([c[0] for c in self.commands], ["query-status", "query-cpus-fast", "screendump",
                                                         "screendump", "query-cpus-fast", "query-status"])
        self.assertGreaterEqual(self.commands[3][2] - self.commands[2][2], 15)
        self.assertEqual(self.counters[0].closed, 1)
        saved = json.loads((self.directory / "idle-diagnostic/result.json").read_text())
        self.assertTrue(saved["measurement_passed"])
        for filename, expected in saved["files_sha256"].items():
            self.assertEqual(digest((self.directory / "idle-diagnostic" / filename).read_bytes()), expected)
        with self.assertRaisesRegex(ValueError, "only once"):
            self.invoke(helper)

    def test_wrong_enter_and_boolean_hold_time_never_query_or_sample(self):
        for args in ({"keys": [], "hold-time": 1}, {**ENTER, "hold-time": True}):
            with self.assertRaisesRegex(ValueError, "exact first Enter"):
                self.helper().before_first_enter(self.command, "send-key", args)
        self.assertFalse(self.commands)

    def test_reference_hash_cannot_be_silently_replaced(self):
        with self.assertRaisesRegex(ValueError, "A/B reference"):
            self.invoke(self.helper(expected_initial_sha256="f" * 64))
        self.assertFalse(self.commands)

    def test_fixture_source_mismatch_fails_before_queries(self):
        self.fixture["metadata"]["source_sha256"] = "b" * 64
        with self.assertRaisesRegex(ValueError, "Fixture/source"):
            self.invoke()
        self.assertFalse(self.commands)

    def test_initial_geometry_is_not_just_a_material_crop(self):
        (self.directory / "initial.ppm").write_bytes(image(1280, 721))
        with self.assertRaisesRegex(ValueError, "Oversized|geometry"):
            self.invoke()

    def test_after_chrome_pixel_change_fails_even_with_matching_material(self):
        self.change_after_frame = True
        with self.assertRaisesRegex(ValueError, "Full framebuffer"):
            self.invoke()
        self.assertEqual(self.counters[0].closed, 1)
        self.assertFalse(json.loads((self.directory / "idle-diagnostic/result.json").read_text())["measurement_passed"])

    def test_incomplete_second_app_prevents_cpu_sample(self):
        self.state["snapshots"]["CALCULATOR"][-1]["FRAMES"] = 0
        self.write_state()
        with self.assertRaisesRegex(ValueError, "Incomplete initial app"):
            self.invoke()
        self.assertEqual(self.counters[0].reads, 0)
        self.assertEqual(self.counters[0].closed, 1)

    def test_missing_ready_marker_prevents_cpu_sample(self):
        self.state["numbers"]["WINDOW:INITMASK"] = [15]
        self.write_state()
        with self.assertRaisesRegex(ValueError, "ready mask"):
            self.invoke()

    def test_changed_reported_commit_count_is_not_silent(self):
        self.change_after_state = True
        with self.assertRaisesRegex(ValueError, "state changed"):
            self.invoke()

    def test_changed_vcpu_mapping_fails(self):
        self.change_thread = True
        with self.assertRaisesRegex(ValueError, "vCPU thread changed"):
            self.invoke()

    def test_source_change_after_measurement_fails(self):
        self.runner.change_source = True
        with self.assertRaisesRegex(ValueError, "Source or initial"):
            self.invoke()

    def test_insufficient_original_deadline_does_not_sleep(self):
        with self.assertRaisesRegex(ValueError, "Insufficient unchanged deadline"):
            self.invoke(self.helper(deadline=116))
        self.assertFalse(self.clock.sleeps)

    def test_guest_exit_inside_quiet_interval_fails_and_closes(self):
        self.clock.on_sleep = lambda: setattr(self.process, "returncode", 35)
        with self.assertRaisesRegex(ValueError, "QEMU exit during CPU"):
            self.invoke()
        self.assertEqual(self.counters[0].closed, 1)

    def test_existing_diagnostic_directory_is_not_overwritten(self):
        (self.directory / "idle-diagnostic").mkdir()
        sentinel = self.directory / "idle-diagnostic/sentinel"
        sentinel.write_text("keep")
        with self.assertRaises(FileExistsError):
            self.invoke()
        self.assertEqual(sentinel.read_text(), "keep")

    def test_cleanup_failure_still_writes_a_failed_result(self):
        original = self.factory
        def failing_factory(process, tid):
            counters = original(process, tid)
            counters.close_error = True
            return counters
        helper = self.helper()
        helper.counters_factory = failing_factory
        with self.assertRaisesRegex(OSError, "fake close failure"):
            self.invoke(helper)
        saved = json.loads((self.directory / "idle-diagnostic/result.json").read_text())
        self.assertFalse(saved["measurement_passed"])
        self.assertIn("counter_cleanup_error", saved)
        self.assertIn("before.ppm", saved["files_sha256"])

    def test_retired_compositor_generation_cannot_be_measured_as_initial(self):
        self.state["snapshots"]["COMPOSITOR"][-1]["GENERATION"] = 2
        self.write_state()
        with self.assertRaisesRegex(ValueError, "Initial compositor"):
            self.invoke()
        self.assertEqual(self.counters[0].reads, 0)


class CounterTests(unittest.TestCase):
    def test_qmp_mapping_rejects_ambiguous_boolean_and_missing_ids(self):
        for reply in ([], [{}, {}], [{}], [{"cpu-index": True, "thread-id": 1}],
                      [{"cpu-index": 0, "thread-id": True}], [{"cpu-index": 0, "thread-id": -1}]):
            with self.subTest(reply=reply), self.assertRaises(ValueError):
                vcpu_id(reply)
        self.assertEqual(vcpu_id([{"cpu-index": 0, "thread-id": 77}]), 77)

    def test_identity_counter_and_clock_regressions_are_rejected(self):
        clock = Clock()
        counters = Counters(Process(), 77, clock)
        old = counters.snapshot()["process"]
        clock.now += 15
        new = counters.snapshot()["process"]
        for changes in ({"creation_100ns": 2}, {"total_100ns": -1},
                        {"read_started": old["read_started"], "read_completed": old["read_completed"]},
                        {"read_completed": new["read_started"] - 1}):
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                counter_delta(old, {**new, **changes})

    def test_exit_after_final_counter_read_is_not_accepted(self):
        clock, process = Clock(), Process()
        counters = Counters(process, 77, clock)
        original = counters.snapshot
        def final_exit():
            result = original()
            if counters.reads == 2:
                process.returncode = 0
            return result
        counters.snapshot = final_exit
        with self.assertRaisesRegex(ValueError, "final CPU read"):
            cpu_measurement(process, counters, 200, clock=clock, sleep=clock.sleep)

    def test_backwards_quiet_clock_does_not_extend_or_restart_interval(self):
        clock, process = Clock(), Process()
        clock.on_sleep = lambda: setattr(clock, "now", clock.now - 1)
        with self.assertRaisesRegex(ValueError, "clock moved backwards"):
            cpu_measurement(process, Counters(process, 77, clock), 200,
                            clock=clock, sleep=clock.sleep)


class Kernel:
    def __init__(self):
        self.pid, self.thread_pid, self.tid = 123, 123, 77
        self.exit_code = 259
        self.closed, self.opened, self.read_handles = [], [], []
        self.open_fail = False

    def GetProcessId(self, handle):
        return self.pid

    def OpenThread(self, rights, inherit, tid):
        self.opened.append((rights, inherit, tid))
        return 0 if self.open_fail else 2000

    def GetThreadId(self, handle):
        return self.tid

    def GetProcessIdOfThread(self, handle):
        return self.thread_pid

    def GetExitCodeThread(self, handle, output):
        output._obj.value = self.exit_code
        return True

    def CloseHandle(self, handle):
        self.closed.append(handle)
        return True

    def GetProcessTimes(self, handle, creation, exited, system, user):
        self.read_handles.append(handle)
        for ptr, value in ((creation, 1), (exited, 0), (system, 50), (user, 75)):
            ptr._obj.dwLowDateTime = value
            ptr._obj.dwHighDateTime = 0
        return True

    GetThreadTimes = GetProcessTimes


class Win32Tests(unittest.TestCase):
    def test_failed_close_retains_handle_until_successful_retry(self):
        class RetryKernel(Kernel):
            def CloseHandle(self, handle):
                self.closed.append(handle)
                return len(self.closed) > 1
        api = RetryKernel()
        counters = Win32CpuCounters(Process(), 77, kernel=api, clock=Clock())
        with self.assertRaises(OSError):
            counters.close()
        self.assertEqual(counters.thread, 2000, "Failed close must retain the owned handle")
        self.assertEqual(api.closed, [2000])
        counters.close()
        self.assertIsNone(counters.thread)
        self.assertEqual(api.closed, [2000, 2000])
        counters.close()
        self.assertEqual(api.closed, [2000, 2000], "Successful close must remain idempotent")

    def test_repeated_close_failure_keeps_raising_and_retaining_handle(self):
        class FailingKernel(Kernel):
            def CloseHandle(self, handle):
                self.closed.append(handle)
                return False
        api = FailingKernel()
        counters = Win32CpuCounters(Process(), 77, kernel=api, clock=Clock())
        for attempt in (1, 2):
            with self.assertRaises(OSError):
                counters.close()
            self.assertEqual(counters.thread, 2000, "Retry failure cannot become a successful no-op")
            self.assertEqual(api.closed, [2000] * attempt)

    def test_actual_popen_handle_and_single_held_thread_handle_are_used(self):
        api = Kernel()
        counters = Win32CpuCounters(Process(), 77, kernel=api, clock=Clock())
        values = counters.snapshot()
        self.assertEqual(api.read_handles, [1000, 2000])
        self.assertEqual(values["vcpu"]["total_100ns"], 125)
        self.assertEqual(api.opened, [(0x0800, False, 77)])
        counters.close()
        counters.close()
        self.assertEqual(api.closed, [2000])

    def test_foreign_or_exited_thread_is_closed_before_failure(self):
        for field, value in (("thread_pid", 456), ("tid", 78), ("exit_code", 0)):
            api = Kernel()
            setattr(api, field, value)
            with self.subTest(field=field), self.assertRaises(ValueError):
                Win32CpuCounters(Process(), 77, kernel=api)
            self.assertEqual(api.closed, [2000])

    def test_wrong_process_handle_does_not_open_thread(self):
        api = Kernel()
        api.pid = 456
        with self.assertRaisesRegex(ValueError, "Popen handle PID"):
            Win32CpuCounters(Process(), 77, kernel=api)
        self.assertFalse(api.opened)

    def test_failed_thread_open_is_not_closed_as_null_handle(self):
        api = Kernel()
        api.open_fail = True
        with self.assertRaises(OSError):
            Win32CpuCounters(Process(), 77, kernel=api)
        self.assertFalse(api.closed)


if __name__ == "__main__":
    unittest.main(verbosity=2)
