"""Windows binary-pipe transport for the pinned product QMP protocol.

The caller supplies an exclusively owned Popen with binary stdin/stdout pipes,
drains stderr, and retains responsibility for process start, exit and kill.
command() is single-caller; close() is idempotent and never kills the child.
Native synchronous I/O runs on dedicated threads, with cancellation and bounded
queues. close() waits at most CLOSE_SECONDS in total. In the exceptional case
Windows delays cancellation, cleanup_pending stays true until I/O completes;
the orchestrator must still terminate/reap its own unresponsive child.
"""
from __future__ import annotations

import ctypes
from ctypes import wintypes as w
import io
import json
import math
import msvcrt
import os
from pathlib import Path
import queue
import sys
import threading
import time

SOURCE_TOOLS = Path(__file__).resolve().parent / "source" / "tools"
if str(SOURCE_TOOLS) not in sys.path:
    sys.path.insert(0, str(SOURCE_TOOLS))
import run_graphics as _protocol
if Path(_protocol.__file__).resolve() != (SOURCE_TOOLS / "run_graphics.py").resolve():
    raise RuntimeError("QMP protocol was imported from a different source checkout")
Qmp = _protocol.Qmp

MAX_MESSAGE = 1024 * 1024
CHUNK = 4096
CLOSE_SECONDS = 0.20
POLL_SECONDS = 0.02
_EOF = object()
_api = ctypes.WinDLL("kernel32", use_last_error=True)
_api.GetCurrentProcess.restype = w.HANDLE
_api.DuplicateHandle.argtypes = [w.HANDLE, w.HANDLE, w.HANDLE,
                               ctypes.POINTER(w.HANDLE), w.DWORD, w.BOOL, w.DWORD]
_api.DuplicateHandle.restype = w.BOOL
_api.CloseHandle.argtypes = [w.HANDLE]
_api.CloseHandle.restype = w.BOOL
_api.OpenThread.argtypes = [w.DWORD, w.BOOL, w.DWORD]
_api.OpenThread.restype = w.HANDLE
_api.CancelSynchronousIo.argtypes = [w.HANDLE]
_api.CancelSynchronousIo.restype = w.BOOL
for _name in ("ReadFile", "WriteFile"):
    _fn = getattr(_api, _name)
    _fn.argtypes = [w.HANDLE, ctypes.c_void_p, w.DWORD, ctypes.POINTER(w.DWORD), ctypes.c_void_p]
    _fn.restype = w.BOOL


def _duplicate(stream):
    original = msvcrt.get_osfhandle(stream.fileno())
    current = _api.GetCurrentProcess()
    duplicate = w.HANDLE()
    if not _api.DuplicateHandle(current, original, current, ctypes.byref(duplicate), 0, False, 2):
        raise ctypes.WinError(ctypes.get_last_error())
    return duplicate


class _Worker:
    def __init__(self, owner, stream, direction):
        self.owner = owner
        self.direction = direction
        self.handle = _duplicate(stream)
        self.thread_handle = None
        self.lock = threading.Lock()
        self.ready = threading.Event()
        self.thread = threading.Thread(target=self.run, name="windows-qmp-" + direction, daemon=True)
        try:
            self.thread.start()
        except BaseException:
            _api.CloseHandle(self.handle)
            raise

    def cancel(self):
        # Keep the thread HANDLE valid while cancellation uses it. No I/O or
        # queue wait runs while holding this lock; no arbitrary thread is used.
        with self.lock:
            if self.thread_handle and not _api.CancelSynchronousIo(self.thread_handle):
                error = ctypes.get_last_error()
                if error != 1168:  # ERROR_NOT_FOUND: cancellation raced completion.
                    self.owner.cleanup_errors.append("CancelSynchronousIo: " + str(error))

    def publish(self, value):
        while not self.owner._stop.is_set():
            try:
                self.owner._incoming.put(value, timeout=POLL_SECONDS)
                return
            except queue.Full:
                if time.monotonic() >= self.owner.deadline:
                    return

    def run(self):
        try:
            with self.lock:
                self.thread_handle = _api.OpenThread(0x0001, False, threading.get_native_id())
                if not self.thread_handle:
                    raise ctypes.WinError(ctypes.get_last_error())
            self.ready.set()
            if self.direction == "read":
                self.read_loop()
            else:
                self.write_loop()
        except BaseException as error:
            self.owner._worker_errors.put(error)
        finally:
            self.ready.set()
            _api.CloseHandle(self.handle)
            with self.lock:
                if self.thread_handle:
                    _api.CloseHandle(self.thread_handle)
                    self.thread_handle = None

    def read_loop(self):
        buffer = ctypes.create_string_buffer(CHUNK)
        while not self.owner._stop.is_set() and time.monotonic() < self.owner.deadline:
            count = w.DWORD()
            success = _api.ReadFile(self.handle, buffer, CHUNK, ctypes.byref(count), None)
            if not success:
                error = ctypes.get_last_error()
                if self.owner._stop.is_set():
                    return
                if error in (109, 232):  # broken/closed pipe
                    self.publish(_EOF)
                    return
                raise ctypes.WinError(error)
            if not count.value:
                self.publish(_EOF)
                return
            self.publish(buffer.raw[:count.value])

    def write_loop(self):
        while not self.owner._stop.is_set() and time.monotonic() < self.owner.deadline:
            try:
                job = self.owner._outgoing.get(timeout=POLL_SECONDS)
            except queue.Empty:
                continue
            data, done, outcome = job
            try:
                offset = 0
                while offset < len(data):
                    self.owner._check()
                    chunk = ctypes.create_string_buffer(data[offset:offset + CHUNK])
                    count = w.DWORD()
                    if not _api.WriteFile(self.handle, chunk, len(chunk) - 1, ctypes.byref(count), None):
                        raise ctypes.WinError(ctypes.get_last_error())
                    if not count.value:
                        raise RuntimeError("Incomplete QMP command write")
                    offset += count.value
            except BaseException as error:
                outcome.append(error)
            finally:
                done.set()


class WindowsQmpPipe(Qmp):
    """Drop-in command/read/send/trace interface using private Windows pipes."""
    def __init__(self, process, deadline):
        if os.name != "nt":
            raise RuntimeError("WindowsQmpPipe requires Windows")
        if not isinstance(deadline, (int, float)) or not math.isfinite(deadline):
            raise ValueError("A finite absolute monotonic deadline is required")
        self.deadline = deadline
        self.reader, self.writer = process.stdout, process.stdin
        if any(stream is None or isinstance(stream, io.TextIOBase) for stream in (self.reader, self.writer)):
            raise ValueError("QMP requires exclusively owned binary stdin/stdout pipes")
        self.pending = bytearray()
        self.sequence = 0
        self.trace = []
        self.cleanup_errors = []
        self._stop = threading.Event()
        self._closed = False
        self._incoming = queue.Queue(maxsize=4)
        self._outgoing = queue.Queue(maxsize=1)
        self._worker_errors = queue.Queue(maxsize=2)
        self._workers = []
        self._command_lock = threading.Lock()
        try:
            self._check()
            for stream, direction in ((self.reader, "read"), (self.writer, "write")):
                worker = _Worker(self, stream, direction)
                self._workers.append(worker)
                if not worker.ready.wait(max(0, self.deadline - time.monotonic())):
                    raise TimeoutError("QMP worker start exceeded the shared deadline")
                self._check()
            if "QMP" not in self.read():
                raise ValueError("Missing QMP greeting")
            self.command("qmp_capabilities")
        except BaseException:
            self.close()
            raise

    def _check(self):
        if self._stop.is_set():
            raise RuntimeError("QMP transport is closed")
        if time.monotonic() >= self.deadline:
            raise TimeoutError("G0 QMP exceeded the shared deadline")
        try:
            error = self._worker_errors.get_nowait()
        except queue.Empty:
            return
        raise RuntimeError("QMP pipe worker failed") from error

    def command(self, name, arguments=None):
        if not self._command_lock.acquire(blocking=False):
            raise RuntimeError("Concurrent QMP commands are unsupported")
        try:
            self._check()
            # Product validation remains authoritative: exact int correlation,
            # exclusive return/error, event shape and 1024-message bound.
            return super().command(name, arguments)
        except BaseException:
            self.close()
            raise
        finally:
            self._command_lock.release()

    def read(self):
        try:
            while True:
                self._check()
                end = self.pending.find(b"\n")
                if end >= 0:
                    if end >= MAX_MESSAGE:
                        raise RuntimeError("Oversized QMP response")
                    line = bytes(self.pending[:end + 1])
                    del self.pending[:end + 1]
                    response = json.loads(line.decode("utf-8"))
                    if not isinstance(response, dict):
                        raise ValueError("QMP response must be an object")
                    self._check()
                    return response
                if len(self.pending) >= MAX_MESSAGE:
                    raise RuntimeError("Oversized QMP response")
                try:
                    chunk = self._incoming.get(timeout=min(POLL_SECONDS, max(0, self.deadline - time.monotonic())))
                except queue.Empty:
                    continue
                if chunk is _EOF:
                    raise RuntimeError("Missing or truncated QMP response")
                self.pending.extend(chunk)
        except BaseException:
            self.close()
            raise

    def send(self, data):
        try:
            self._check()
            if not isinstance(data, bytes) or len(data) > MAX_MESSAGE:
                raise ValueError("QMP command must be bounded binary bytes")
            done, outcome = threading.Event(), []
            self._outgoing.put_nowait((data, done, outcome))
            while not done.wait(min(POLL_SECONDS, max(0, self.deadline - time.monotonic()))):
                self._check()
            self._check()
            if outcome:
                raise RuntimeError("QMP pipe write failed") from outcome[0]
        except BaseException:
            self.close()
            raise

    @property
    def cleanup_pending(self):
        return any(worker.thread.is_alive() for worker in self._workers)

    def close(self):
        if self._closed:
            return
        self._closed = True
        self._stop.set()
        until = time.monotonic() + CLOSE_SECONDS
        # Repeated cancellation closes the check-to-I/O race. Only these
        # dedicated workers can issue I/O on their duplicate handles.
        while self.cleanup_pending and time.monotonic() < until:
            for worker in self._workers:
                worker.cancel()
            for worker in self._workers:
                worker.thread.join(timeout=max(0, min(POLL_SECONDS, until - time.monotonic())))
        # Workers never enter Python stream methods, so no blocked buffered
        # read/write lock or unflushed Python payload can trap these closes.
        for stream in (self.reader, self.writer):
            try:
                stream.close()
            except (OSError, ValueError) as error:
                self.cleanup_errors.append(str(error))
