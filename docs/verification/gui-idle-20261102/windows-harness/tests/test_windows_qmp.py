"""Windows real-pipe protocol tests; --whpx enables one paused init only."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from windows_qmp import WindowsQmpPipe, MAX_MESSAGE

RUN_WHPX = "--whpx" in sys.argv
if RUN_WHPX:
    sys.argv.remove("--whpx")

CHILD = r'''
import json, os, sys, time
mode = sys.argv[1]
def raw(data):
    sys.stdout.buffer.write(data)
    sys.stdout.buffer.flush()
def send(value):
    raw((json.dumps(value) + '\n').encode('utf-8'))
if mode == 'silent':
    time.sleep(10)
    sys.exit(0)
if mode == 'bad_greeting':
    send({'hello': 1})
    time.sleep(10)
    sys.exit(0)
if mode == 'array_greeting':
    send([])
    time.sleep(10)
    sys.exit(0)
send({'QMP': {'version': {}, 'capabilities': []}})
first = json.loads(sys.stdin.buffer.readline())
send({'return': {}, 'id': True if mode == 'boolean_caps_id' else first['id']})
if mode == 'blocked_write':
    time.sleep(10)
    sys.exit(0)
line = sys.stdin.buffer.readline()
request = json.loads(line)
identity = request['id']
if mode == 'valid':
    send({'event': 'STOP', 'data': {}})
    send({'return': {'lf_only': line.endswith(b'\n') and not line.endswith(b'\r\n'),
                     'arguments': request.get('arguments')}, 'id': identity})
elif mode == 'chunks':
    data = (json.dumps({'return': {'value': 'é'}, 'id': identity}, ensure_ascii=False) + '\n').encode('utf-8')
    for value in data:
        raw(bytes([value])); time.sleep(.001)
elif mode == 'unmatched': send({'return': {}, 'id': identity + 1})
elif mode == 'boolean_id': send({'return': {}, 'id': True})
elif mode == 'string_id': send({'return': {}, 'id': str(identity)})
elif mode == 'both': send({'return': {}, 'error': {}, 'id': identity})
elif mode == 'event_id': send({'event': 'STOP', 'id': identity})
elif mode == 'error': send({'error': {'class': 'GenericError', 'desc': 'fixture'}, 'id': identity})
elif mode == 'array': send([])
elif mode == 'invalid_json': raw(b'{not json}\n')
elif mode == 'invalid_utf8': raw(b'{"return":"\xff","id":2}\n')
elif mode == 'truncated': raw(b'{"id":2'); sys.exit(0)
elif mode == 'eof': sys.exit(0)
elif mode == 'oversize': raw(b' ' * (1024 * 1024) + b'\n')
elif mode == 'oversize_no_lf': raw(b' ' * (1024 * 1024))
elif mode.startswith('events_'):
    for _ in range(int(mode.split('_')[1])): send({'event': 'STOP'})
    send({'return': {}, 'id': identity})
elif mode == 'drip':
    raw(b'{')
    for _ in range(500):
        raw(b' '); time.sleep(.02)
elif mode == 'silent_response': time.sleep(10)
time.sleep(10)
'''


class OwnedChild:
    def __init__(self, command):
        self.process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, bufsize=0, creationflags=subprocess.CREATE_NO_WINDOW)
        self.forced = False
        def stop():
            if self.process.poll() is None:
                self.forced = True
                self.process.kill()
        self.guard = threading.Timer(15, stop)
        self.guard.daemon = True
        self.guard.start()

    def close(self):
        self.guard.cancel()
        if self.process.poll() is None:
            self.process.kill()
        self.process.wait(timeout=3)
        for stream in (self.process.stdout, self.process.stdin):
            if stream:
                stream.close()


class WindowsPipeTests(unittest.TestCase):
    def setUp(self):
        self.children = []
        self.controls = []

    def tearDown(self):
        for control in self.controls:
            control.close()
        for child in self.children:
            child.close()
            self.assertFalse(child.forced, "External fake-child guard fired")
        for control in self.controls:
            end = time.monotonic() + .5
            while control.cleanup_pending and time.monotonic() < end:
                time.sleep(.01)
            self.assertFalse(control.cleanup_pending, "Worker still alive after owned child reaped")

    def child(self, mode):
        child = OwnedChild([sys.executable, "-B", "-u", "-c", CHILD, mode])
        self.children.append(child)
        return child

    def fake(self, mode, duration=5):
        child = self.child(mode)
        control = WindowsQmpPipe(child.process, time.monotonic() + duration)
        self.controls.append(control)
        return child, control

    def test_events_correlation_trace_and_binary_lf(self):
        child, control = self.fake("valid")
        result = control.command("query-status", {"text": "é"})
        self.assertEqual(result, {"lf_only": True, "arguments": {"text": "é"}})
        self.assertEqual([entry["request"]["id"] for entry in control.trace], [1, 2])
        self.assertEqual(control.trace[-1]["response"]["id"], 2)
        control.close()
        before = time.monotonic()
        control.close()
        self.assertLess(time.monotonic() - before, .05)
        self.assertIsNone(child.process.poll(), "Adapter killed the caller-owned child")
        with self.assertRaisesRegex(RuntimeError, "closed"):
            control.command("query-status")

    def test_split_multibyte_utf8(self):
        _, control = self.fake("chunks")
        self.assertEqual(control.command("query-status"), {"value": "é"})

    def test_invalid_protocol_is_terminal(self):
        for mode in ["unmatched", "boolean_id", "string_id", "both", "event_id", "array", "invalid_json", "invalid_utf8"]:
            with self.subTest(mode=mode):
                _, control = self.fake(mode)
                with self.assertRaises((RuntimeError, ValueError, UnicodeError)):
                    control.command("query-status")
                with self.assertRaisesRegex(RuntimeError, "closed"):
                    control.command("query-status")

    def test_command_error_keeps_correlated_trace(self):
        _, control = self.fake("error")
        with self.assertRaisesRegex(RuntimeError, "QMP command failed"):
            control.command("query-status")
        self.assertEqual(control.trace[-1]["response"]["error"]["class"], "GenericError")

    def test_bad_greetings(self):
        for mode in ["bad_greeting", "array_greeting"]:
            with self.subTest(mode=mode):
                with self.assertRaises(ValueError):
                    self.fake(mode)

    def test_boolean_capabilities_id_is_not_integer_one(self):
        with self.assertRaisesRegex(RuntimeError, "Malformed or uncorrelated"):
            self.fake("boolean_caps_id")

    def test_eof_and_truncated_response(self):
        for mode in ["eof", "truncated"]:
            with self.subTest(mode=mode):
                _, control = self.fake(mode)
                with self.assertRaisesRegex(RuntimeError, "Missing or truncated"):
                    control.command("query-status")

    def test_response_size_bound_with_and_without_lf(self):
        for mode in ["oversize", "oversize_no_lf"]:
            with self.subTest(mode=mode):
                _, control = self.fake(mode)
                with self.assertRaisesRegex(RuntimeError, "Oversized"):
                    control.command("query-status")
                self.assertLessEqual(len(control.pending), MAX_MESSAGE + 4096)
                self.assertLessEqual(control._incoming.qsize(), 4)

    def test_event_boundary(self):
        _, allowed = self.fake("events_1023")
        self.assertEqual(allowed.command("query-status"), {})
        _, rejected = self.fake("events_1024")
        with self.assertRaisesRegex(RuntimeError, "Too many QMP events"):
            rejected.command("query-status")

    def test_shared_deadline_cannot_be_extended_by_dripping_bytes(self):
        for mode in ["drip", "silent_response"]:
            with self.subTest(mode=mode):
                child, control = self.fake(mode)
                started = time.monotonic()
                # Shorten the same absolute deadline after process startup;
                # individual bytes/replies still cannot restart it.
                control.deadline = started + .7
                with self.assertRaises(TimeoutError):
                    control.command("query-status")
                self.assertLess(time.monotonic() - started, 1.3)
                self.assertIsNone(child.process.poll())

    def test_silent_greeting_deadline(self):
        child = self.child("silent")
        started = time.monotonic()
        with self.assertRaises(TimeoutError):
            WindowsQmpPipe(child.process, started + .4)
        self.assertLess(time.monotonic() - started, 1)

    def test_blocked_write_deadline_and_bounded_close(self):
        child, control = self.fake("blocked_write")
        started = time.monotonic()
        control.deadline = started + .8
        with self.assertRaises(TimeoutError):
            control.command("large", {"payload": "x" * 800_000})
        self.assertLess(time.monotonic() - started, 1.4)
        self.assertIsNone(child.process.poll())
        started = time.monotonic()
        control.close()
        self.assertLess(time.monotonic() - started, .25)

    def test_outgoing_size_bound(self):
        _, control = self.fake("blocked_write")
        with self.assertRaisesRegex(ValueError, "bounded binary"):
            control.send(b"x" * (MAX_MESSAGE + 1))

    def test_close_interrupts_blocked_read_without_killing_child(self):
        child, control = self.fake("silent_response")
        errors = []
        def read_command():
            try:
                control.command("query-status")
            except Exception as error:
                errors.append(error)
        thread = threading.Thread(target=read_command, daemon=True)
        thread.start()
        time.sleep(.1)
        started = time.monotonic()
        control.close()
        thread.join(timeout=.5)
        self.assertLess(time.monotonic() - started, .7)
        self.assertFalse(thread.is_alive())
        self.assertTrue(errors)
        self.assertIsNone(child.process.poll())


@unittest.skipUnless(RUN_WHPX, "Use --whpx for one authorized paused initialization")
class PausedWhpxTests(unittest.TestCase):
    def test_real_paused_initialization_query_quit(self):
        qemu = Path(r"C:\Program Files\qemu\qemu-system-x86_64.exe")
        qemu_sha256 = hashlib.sha256(qemu.read_bytes()).hexdigest()
        self.assertEqual(qemu_sha256, "47d57a6072e0bb3bd98f87926eb129eb1736dfe818c67b3b81ef7ce4edd0b3cd")
        command = [str(qemu), "-no-user-config", "-nodefaults", "-machine", "pc-q35-8.2",
            "-accel", "whpx", "-cpu", "qemu64", "-smp", "1", "-m", "256M", "-S",
            "-display", "none", "-monitor", "none", "-serial", "none", "-nic", "none", "-qmp", "stdio"]
        child = OwnedChild(command)
        control = None
        started = time.monotonic()
        try:
            control = WindowsQmpPipe(child.process, started + 10)
            status = control.command("query-status")
            self.assertFalse(status["running"])
            self.assertEqual(status["status"], "prelaunch")
            control.command("quit")
            self.assertEqual(child.process.wait(timeout=3), 0)
            self.assertFalse(child.forced)
            print(json.dumps({"paused_whpx": True, "status": status, "exit_code": child.process.returncode,
                "elapsed_seconds": time.monotonic() - started, "command": command,
                "qemu_sha256": qemu_sha256, "trace": control.trace}), flush=True)
        finally:
            if control is not None:
                control.close()
            child.close()
        self.assertFalse(control.cleanup_pending)


if __name__ == "__main__":
    unittest.main(verbosity=2)
