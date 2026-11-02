"""Opt-in, one-shot Windows CPU observation before the native runner's first Enter.

This helper never injects input, pauses the guest, changes a deadline or grants a
native acceptance result. The caller must run the original native gate to its
real exit after this method returns, and bind this helper's hash in its report.
"""
from __future__ import annotations

import ctypes
from ctypes import wintypes as w
import hashlib
import json
import math
import os
from pathlib import Path
import re
import time

from windows_inputs import assert_plain_path

SECONDS = 15.0
MAX_FRAME = 1280 * 720 * 3 + 128
ENTER = {"keys": [{"type": "qcode", "data": "ret"}], "hold-time": 1}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def _filetime(value):
    return (value.dwHighDateTime << 32) | value.dwLowDateTime


class Win32CpuCounters:
    """Read the existing Popen handle and one held, owner-checked vCPU handle."""
    def __init__(self, process, thread_id, *, kernel=None, clock=time.monotonic):
        self.process, self.clock = process, clock
        self.thread = None
        require(type(thread_id) is int and 0 < thread_id <= 0xFFFFFFFF, "Invalid vCPU thread ID")
        self.thread_id = thread_id
        handle = getattr(process, "_handle", None)
        require(handle is not None and not getattr(handle, "closed", False), "Missing Popen handle")
        self.handle = int(handle)
        require(self.handle > 0, "Invalid Popen handle")
        if kernel is None:
            require(os.name == "nt", "Win32 counters require Windows")
            kernel = ctypes.WinDLL("kernel32", use_last_error=True)
            for name in ("GetProcessTimes", "GetThreadTimes"):
                function = getattr(kernel, name)
                function.argtypes = [w.HANDLE] + [ctypes.POINTER(w.FILETIME)] * 4
                function.restype = w.BOOL
            kernel.GetProcessId.argtypes = [w.HANDLE]
            kernel.GetProcessId.restype = w.DWORD
            kernel.GetProcessIdOfThread.argtypes = [w.HANDLE]
            kernel.GetProcessIdOfThread.restype = w.DWORD
            kernel.GetThreadId.argtypes = [w.HANDLE]
            kernel.GetThreadId.restype = w.DWORD
            kernel.OpenThread.argtypes = [w.DWORD, w.BOOL, w.DWORD]
            kernel.OpenThread.restype = w.HANDLE
            kernel.GetExitCodeThread.argtypes = [w.HANDLE, ctypes.POINTER(w.DWORD)]
            kernel.GetExitCodeThread.restype = w.BOOL
            kernel.CloseHandle.argtypes = [w.HANDLE]
            kernel.CloseHandle.restype = w.BOOL
        self.kernel = kernel
        try:
            require(kernel.GetProcessId(self.handle) == process.pid, "Popen handle PID differs")
            self.thread = kernel.OpenThread(0x0800, False, thread_id)
            if not self.thread:
                self.thread = None
                raise ctypes.WinError(ctypes.get_last_error())
            self._identity()
        except BaseException:
            self.close()
            raise

    def _identity(self):
        require(self.process.poll() is None, "QEMU exited during CPU observation")
        require(self.kernel.GetProcessId(self.handle) == self.process.pid, "Process handle changed")
        require(self.kernel.GetThreadId(self.thread) == self.thread_id, "Thread handle ID differs")
        require(self.kernel.GetProcessIdOfThread(self.thread) == self.process.pid, "vCPU thread has another owner")
        code = w.DWORD()
        if not self.kernel.GetExitCodeThread(self.thread, ctypes.byref(code)):
            raise ctypes.WinError(ctypes.get_last_error())
        require(code.value == 259, "vCPU thread exited")

    def _read(self, handle, function):
        creation, unused_exit, system, user = [w.FILETIME() for _ in range(4)]
        started = self.clock()
        if not function(handle, ctypes.byref(creation), ctypes.byref(unused_exit),
                        ctypes.byref(system), ctypes.byref(user)):
            raise ctypes.WinError(ctypes.get_last_error())
        completed = self.clock()
        require(math.isfinite(started) and math.isfinite(completed) and completed >= started,
                "Invalid counter sampling clock")
        return {"creation_100ns": _filetime(creation), "kernel_100ns": _filetime(system),
                "user_100ns": _filetime(user), "total_100ns": _filetime(system) + _filetime(user),
                "read_started": started, "read_completed": completed}

    def snapshot(self):
        self._identity()
        result = {"process": self._read(self.handle, self.kernel.GetProcessTimes),
                  "vcpu": self._read(self.thread, self.kernel.GetThreadTimes)}
        self._identity()
        return result

    def close(self):
        if self.thread is not None:
            if not self.kernel.CloseHandle(self.thread):
                raise ctypes.WinError(ctypes.get_last_error())
            self.thread = None
        # The Popen handle belongs to the orchestrator and is never closed here.


def counter_delta(before, after):
    require(before["creation_100ns"] == after["creation_100ns"] and before["creation_100ns"] > 0,
            "Counter identity changed")
    for sample in (before, after):
        require(sample["read_completed"] >= sample["read_started"], "Counter clock moved backwards")
        require(sample["total_100ns"] == sample["kernel_100ns"] + sample["user_100ns"], "Invalid CPU total")
    cpu = {}
    for name in ("kernel", "user", "total"):
        delta = after[name + "_100ns"] - before[name + "_100ns"]
        require(delta >= 0, "CPU counter moved backwards")
        cpu[name + "_seconds"] = delta / 1e7
    lower = after["read_started"] - before["read_completed"]
    upper = after["read_completed"] - before["read_started"]
    wall = (lower + upper) / 2
    require(math.isfinite(wall) and lower > 0 and upper >= lower, "Invalid CPU measurement interval")
    return {**cpu, "wall_seconds_midpoint": wall, "wall_seconds_lower": lower,
            "wall_seconds_upper": upper, "percent_one_logical_cpu": 100 * cpu["total_seconds"] / wall}


def cpu_measurement(process, counters, deadline, *, clock=time.monotonic, sleep=time.sleep):
    require(process.poll() is None, "QEMU already exited")
    before = counters.snapshot()
    quiet_start = clock()
    end = quiet_start + SECONDS
    require(end + 2 < deadline, "Insufficient unchanged deadline for CPU sample and capture")
    previous = quiet_start
    while True:
        now = clock()
        require(math.isfinite(now) and now >= previous, "Quiet-interval clock moved backwards")
        previous = now
        require(now < deadline and process.poll() is None, "Deadline or QEMU exit during CPU sample")
        if now >= end:
            break
        sleep(min(0.2, end - now))
    quiet_end = now
    after = counters.snapshot()
    require(clock() < deadline and process.poll() is None, "Deadline or QEMU exit at final CPU read")
    return {"before": before, "after": after, "quiet_seconds": quiet_end - quiet_start,
            "process": counter_delta(before["process"], after["process"]),
            "vcpu": counter_delta(before["vcpu"], after["vcpu"])}


def vcpu_id(reply):
    require(type(reply) is list and len(reply) == 1 and type(reply[0]) is dict,
            "Expected exactly one QMP vCPU")
    cpu = reply[0]
    require(type(cpu.get("cpu-index")) is int and cpu["cpu-index"] == 0,
            "Unexpected QMP vCPU index")
    tid = cpu.get("thread-id")
    require(type(tid) is int and 0 < tid <= 0xFFFFFFFF, "Missing or invalid QMP vCPU thread ID")
    return tid


class WindowsIdleMeasurement:
    """One instance per QEMU child; before_first_enter never sends the Enter."""
    def __init__(self, runner, fixture, process, directory, deadline,
                 expected_initial_sha256=None, *, counters_factory=Win32CpuCounters,
                 clock=time.monotonic, sleep=time.sleep):
        self.runner, self.fixture, self.process = runner, fixture, process
        self.directory = assert_plain_path(directory, root=runner.BUILD)
        self.deadline, self.clock, self.sleep = deadline, clock, sleep
        self.counters_factory = counters_factory
        require(math.isfinite(deadline) and deadline > clock(), "Expired diagnostic deadline")
        require(expected_initial_sha256 is None or (type(expected_initial_sha256) is str
                and re.fullmatch("[0-9a-f]{64}", expected_initial_sha256)), "Invalid reference hash")
        self.expected_initial_sha256 = expected_initial_sha256
        self.used = False
        self.record = {"measurement_passed": False, "native_gate": False,
                       "scope": "One 15s headless CPU interval, no guest input or stop/cont. Equal endpoint images do not prove no redraw or ACK activity.",
                       "counter_scope": "Existing probe snapshots only. App snapshots emit on generation changes; compositor snapshots are conditional/rate-limited. Retained values are not fresh synchronous counter reads."}
        self.files = {}

    def _check_live(self):
        require(self.clock() < self.deadline and self.process.poll() is None,
                "Diagnostic deadline or QEMU exit")

    def _path(self, path, *, regular=False):
        return self.runner.private_path(assert_plain_path(path, root=self.directory, regular=regular), regular=regular)

    def _save(self, name, raw):
        path = self._path(self.output / name)
        with path.open("xb") as stream:
            stream.write(raw)
        self.files[name] = digest(raw)
        return path

    def _frame(self, path):
        path = self._path(path, regular=True)
        require(0 < path.stat().st_size <= MAX_FRAME, "Oversized or empty diagnostic frame")
        raw = path.read_bytes()
        require(len(raw) <= MAX_FRAME and self.runner.ppm(raw)[:2] == (1280, 720), "Wrong full framebuffer geometry")
        require(self.runner.material_matches(raw, self.fixture["frames"]["initial"]), "Source initial Material oracle differs")
        return raw

    def _capture(self, command, name, reference):
        self._check_live()
        path = self._path(self.output / name)
        command("screendump", {"filename": str(path)})
        self._check_live()
        raw = self._frame(path)
        require(raw == reference, "Full framebuffer differs: " + name)
        self.files[name] = digest(raw)
        return raw

    def _observed(self, name):
        self._check_live()
        raw = self.runner.serial(self.directory / "serial.txt")
        require(self.runner.numbers(raw, "WINDOW:INITMASK")[-1:] == [31], "Initial ready mask missing")
        require(self.runner.numbers(raw, "WINDOW:RECOVERIES")[-1:] == [0], "Not the initial service generation")
        require(not self.runner.numbers(raw, "WINDOW:FAULT_SLOT"), "Guest already faulted")
        state = {}
        for role in ("MATERIAL", "CALCULATOR", "COMPOSITOR"):
            entries = self.runner.snapshots(raw, role)
            require(bool(entries), "Missing source snapshot: " + role)
            state[role] = entries[-1]
        for role in ("MATERIAL", "CALCULATOR"):
            app = state[role]
            require(app["SERVICE_GENERATION"] == 1 and app["READY"] == 1 and app["PROBES"] == 3
                    and app["GEN"] > 0 and app["FRAMES"] >= 1, "Incomplete initial app frame: " + role)
        comp = state["COMPOSITOR"]
        require(all(comp.get(key) == value for key, value in {
            "GENERATION": 1, "FOCUSED": 0, "POINTER_X": 1260, "POINTER_Y": 50, "BUTTONS": 0,
            "FATAL": 0, "MATERIAL": state["MATERIAL"]["GEN"], "CALCULATOR": state["CALCULATOR"]["GEN"]}.items()),
            "Initial compositor position/frame observations differ")
        progress = {role: {key: self.runner.numbers(raw, f"WINDOW:{role}:{key}")[-1:]
                    for key in ("KEYS", "INPUT_ACKS")} for role in ("MATERIAL", "CALCULATOR")}
        self._save(name, raw)
        return {"snapshots": state, "input_progress": progress, "serial_sha256": digest(raw), "serial_bytes": len(raw)}

    def before_first_enter(self, command, name, arguments):
        """command must be the original bound QMP method, bypassing the hook."""
        require(not self.used, "Idle measurement may run only once")
        self.used = True
        require(name == "send-key" and type(arguments) is dict and arguments == ENTER
                and type(arguments.get("hold-time")) is int, "Hook is not the exact first Enter")
        self.output = self._path(self.directory / "idle-diagnostic")
        self.output.mkdir()  # Exclusive: never overwrite another attempt's evidence.
        counters = None
        try:
            self._check_live()
            frozen = self.runner.source_hash()
            require(self.fixture["metadata"]["source_sha256"] == frozen, "Fixture/source binding differs")
            reference = self._frame(self.directory / "initial.ppm")
            reference_hash = digest(reference)
            require(self.expected_initial_sha256 is None or reference_hash == self.expected_initial_sha256,
                    "Initial full frame differs from the frozen A/B reference")
            self._save("initial-reference.ppm", reference)
            self.record.update(source_sha256=frozen, source_initial_sha256=reference_hash,
                               expected_initial_sha256=self.expected_initial_sha256,
                               unchanged_deadline=self.deadline, process_id=self.process.pid)
            status = command("query-status")
            require(type(status) is dict and status.get("running") is True and status.get("status") == "running",
                    "Guest is not running")
            mapping = command("query-cpus-fast")
            thread_id = vcpu_id(mapping)
            self.record["vcpu_mapping_before"] = mapping
            counters = self.counters_factory(self.process, thread_id)
            self.record["before"] = self._observed("serial-before.txt")
            self._capture(command, "before.ppm", reference)
            self.record["cpu"] = cpu_measurement(self.process, counters, self.deadline,
                                                  clock=self.clock, sleep=self.sleep)
            self._capture(command, "after.ppm", reference)
            self.record["after"] = self._observed("serial-after.txt")
            # Snapshot equality is a consistency guard, not a claim that these
            # event-driven logs provide fresh reads of every counter at endpoints.
            require(self.record["before"]["snapshots"] == self.record["after"]["snapshots"]
                    and self.record["before"]["input_progress"] == self.record["after"]["input_progress"],
                    "Reported initial app/compositor state changed during the interval")
            mapping_after = command("query-cpus-fast")
            require(vcpu_id(mapping_after) == thread_id, "QMP vCPU thread changed")
            self.record["vcpu_mapping_after"] = mapping_after
            status_after = command("query-status")
            require(type(status_after) is dict and status_after.get("running") is True
                    and status_after.get("status") == "running", "Guest stopped during observation")
            require(self.runner.source_hash() == frozen and self._frame(self.directory / "initial.ppm") == reference,
                    "Source or initial frame changed")
            for filename, expected in self.files.items():
                require(digest(self._path(self.output / filename, regular=True).read_bytes()) == expected,
                        "Stored diagnostic evidence changed")
            self._check_live()
            counters.close()
            counters = None
            self.record["measurement_passed"] = True
        except Exception as error:
            self.record["error"] = type(error).__name__ + ": " + str(error)
            raise
        finally:
            cleanup_error = None
            if counters is not None:
                try:
                    counters.close()
                except Exception as error:
                    self.record["measurement_passed"] = False
                    self.record["counter_cleanup_error"] = type(error).__name__ + ": " + str(error)
                    cleanup_error = error
            self.record["files_sha256"] = self.files.copy()
            self._save("result.json", json.dumps(self.record, indent=2).encode("utf-8"))
            if cleanup_error is not None:
                raise cleanup_error
        return self.record
