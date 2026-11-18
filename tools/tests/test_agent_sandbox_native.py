"""Opt-in real Linux isolation controls; no model client or hardware acceptance.

Provide BARYON_SANDBOX_TEST_CONFIG with a reviewed offline fixture runtime
containing python3 and bash. Absence is NOT RUN, never isolation evidence.
"""
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time
import unittest
from concurrent.futures import ThreadPoolExecutor
from threading import Event
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from agent_sandbox import Sandbox, SandboxError
import agent_gate


@unittest.skipUnless(os.environ.get("BARYON_SANDBOX_TEST_CONFIG"), "NOT RUN: isolated fixture runtime not provided")
class NativeBoundary(unittest.TestCase):
    def setUp(self):
        self.sandbox = Sandbox(json.loads(Path(os.environ["BARYON_SANDBOX_TEST_CONFIG"]).read_text()))
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.lease = self.enterContext(self.sandbox.lease())
        self.work = self.lease.path

    def python(self, code, **options):
        return self.sandbox.run(
            ["/usr/bin/python3", "-I", "-B", "-c", code], self.lease,
            profile="agent", **options)

    def test_credential_environment_and_controller_files_are_absent(self):
        private = self.root / "controller-only"
        private.write_text("synthetic fixture")
        code = ("import os; from pathlib import Path; "
                "assert 'EXAMPLE_CONTROLLER_CREDENTIAL' not in os.environ; "
                f"assert not Path({str(private)!r}).exists(); "
                "assert not Path('/workspace/.git').exists(); "
                "Path('accepted').write_text('ok')")
        with mock.patch.dict(os.environ, {"EXAMPLE_CONTROLLER_CREDENTIAL": "synthetic fixture"}):
            done = self.python(code)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual((self.work / "accepted").read_text(), "ok")

    def test_runtime_is_readonly_and_process_capabilities_are_empty(self):
        code = ("from pathlib import Path\nimport errno\n"
                "assert 'CapEff:\\t0000000000000000' in Path('/proc/self/status').read_text()\n"
                "assert not Path('/run/user/1000/bus').exists()\n"
                "assert not Path('/run/user/1000/systemd/private').exists()\n"
                "try: Path('/usr/bin/unaccepted').write_text('x')\n"
                "except OSError as error: assert error.errno in (errno.EROFS, errno.EACCES)\n"
                "else: raise AssertionError('writable runtime')\n")
        done = self.python(code)
        self.assertEqual(done.returncode, 0, done.stderr)
        print(json.dumps({"native_agent_resource": done.resource_status}, sort_keys=True))

    def test_devices_are_readonly_and_shared_memory_uses_the_capped_scratch(self):
        capacity = self.sandbox.scratch["capacity_bytes"]
        inodes = self.sandbox.scratch["capacity_inodes"]
        code = ("import errno,os\nfrom pathlib import Path\n"
                "pts=[line.split() for line in Path('/proc/self/mountinfo').read_text().splitlines() "
                "if line.split()[4] == '/dev/pts']\n"
                "assert len(pts) == 1 and 'ro' in pts[0][5].split(',')\n"
                "try: pty=os.open('/dev/ptmx',os.O_RDWR|os.O_NOCTTY)\n"
                "except OSError as error:\n"
                " assert error.errno in (errno.EROFS,errno.EACCES,errno.EPERM,errno.ENODEV,"
                "errno.ENOENT,errno.EIO,errno.ENXIO)\n"
                "else:\n os.close(pty)\n raise AssertionError('writable PTY allocator')\n"
                "try: Path('/dev/unaccepted').write_text('x')\n"
                "except OSError as error:\n"
                " assert error.errno in (errno.EROFS,errno.EACCES,errno.EPERM)\n"
                "else: raise AssertionError('writable device tree')\n"
                "probe=Path('/dev/shm/probe')\n"
                "probe.write_text('ok')\n"
                "assert probe.read_text() == 'ok'\n"
                "assert os.stat('/dev/shm').st_dev == os.stat('/workspace').st_dev\n"
                "stats=os.statvfs('/dev/shm')\n"
                f"assert stats.f_blocks * stats.f_frsize == {capacity}\n"
                f"assert stats.f_files == {inodes}\n")
        done = self.python(code)
        self.assertEqual(done.returncode, 0, done.stderr)

    def test_scratch_byte_limit_returns_enospc_and_recovers(self):
        capacity = self.sandbox.scratch["capacity_bytes"]
        self.assertLessEqual(capacity, 128 * 1024 * 1024, "fixture byte cap is unsafe for this probe")
        code = ("import errno,os\nfrom pathlib import Path\n"
                "path=Path('/workspace/byte-limit-probe')\n"
                "chunk=b'x'*(1024*1024)\nwritten=0\nfailure=None\n"
                "try:\n"
                " with path.open('xb',buffering=0) as output:\n"
                "  while True:\n"
                "   try: written += output.write(chunk)\n"
                "   except OSError as error: failure=error.errno; break\n"
                " stats=os.statvfs('/workspace')\n"
                " assert failure == errno.ENOSPC and written > 0 and stats.f_bavail == 0\n"
                "finally: path.unlink(missing_ok=True)\n"
                "assert os.statvfs('/workspace').f_bavail > 0\n")
        done = self.python(code)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertGreater(os.statvfs(self.work).f_bavail, 0)

    def test_scratch_inode_limit_returns_enospc_and_recovers(self):
        inodes = self.sandbox.scratch["capacity_inodes"]
        self.assertLessEqual(inodes, 8192, "fixture inode cap is unsafe for this probe")
        code = ("import errno,os\nfrom pathlib import Path\n"
                "root=Path('/workspace/inode-limit-probe')\nroot.mkdir()\n"
                "created=[]\nfailure=None\n"
                "try:\n"
                f" for index in range({inodes + 1}):\n"
                "  path=root/str(index)\n"
                "  try: descriptor=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)\n"
                "  except OSError as error: failure=error.errno; break\n"
                "  else: os.close(descriptor); created.append(path)\n"
                " stats=os.statvfs('/workspace')\n"
                " assert failure == errno.ENOSPC and created and stats.f_favail == 0\n"
                "finally:\n"
                " for path in created: path.unlink()\n"
                " root.rmdir()\n"
                "assert os.statvfs('/workspace').f_favail > 0\n")
        done = self.python(code)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertGreater(os.statvfs(self.work).f_favail, 0)

    def test_parallel_leases_use_distinct_roots_and_hide_siblings(self):
        self.assertGreaterEqual(len(self.sandbox.scratch_roots), 2,
                                "native sibling acceptance requires two scratch roots")
        with self.sandbox.lease() as sibling:
            self.assertNotEqual(self.lease.root, sibling.root)
            self.assertNotEqual(self.lease.receipt["mount_id"], sibling.receipt["mount_id"])

            def run(lease, marker, other_root):
                code = ("from pathlib import Path\n"
                        f"assert not Path({str(other_root)!r}).exists()\n"
                        f"Path('sibling-marker').write_text({marker!r})\n")
                return self.sandbox.run(
                    ["/usr/bin/python3", "-I", "-B", "-c", code], lease,
                    profile="agent")

            with ThreadPoolExecutor(max_workers=2) as pool:
                first = pool.submit(run, self.lease, "first", sibling.root)
                second = pool.submit(run, sibling, "second", self.lease.root)
                self.assertEqual(first.result().returncode, 0)
                self.assertEqual(second.result().returncode, 0)
            self.assertEqual((self.lease.path / "sibling-marker").read_text(), "first")
            self.assertEqual((sibling.path / "sibling-marker").read_text(), "second")

    def test_payload_has_no_controller_file_descriptors(self):
        code = ("import os\n"
                "for name in os.listdir('/proc/self/fd'):\n"
                " try: os.fstat(int(name))\n"
                " except OSError: continue\n"
                " assert int(name) <= 2, 'unexpected inherited descriptor'\n")
        done = self.python(code)
        self.assertEqual(done.returncode, 0, done.stderr)

    def test_gate_source_stays_readonly_with_separate_outputs(self):
        source = self.work / "source.txt"
        source.write_text("source")
        source.chmod(0o644)
        code = ("from pathlib import Path\nimport errno\n"
                "try: Path('source.txt').write_text('changed')\n"
                "except OSError as error: assert error.errno in (errno.EROFS, errno.EACCES)\n"
                "else: raise AssertionError('writable source')\n"
                "Path('build/output').write_text('ok')\n"
                "Path('target/output').write_text('ok')\n")
        done = self.python(code, read_only=True)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(source.read_text(), "source")
        self.assertEqual((self.work / "build/output").read_text(), "ok")

    def test_gate_rejects_source_mutation_before_later_evaluation(self):
        source = self.work / "evaluator.py"
        source.write_text("raise SystemExit(9)\n")
        source.chmod(0o644)
        script = "python3 -I -B -c 'from pathlib import Path; Path(\"evaluator.py\").write_text(\"pass\")'"
        log = self.root / "immutable-gate.log"
        with mock.patch.object(agent_gate, "controller_source_commit", return_value="a" * 40):
            code = agent_gate.run_gate([("mutator", script, {}), ("evaluator", "python3 evaluator.py", {})],
                                       self.work, log, sandbox=self.sandbox, source_paths=["evaluator.py"])
        self.assertEqual(code, 2)
        self.assertIn("## FAIL: mutator (exit 1)", log.read_text())
        self.assertNotIn("## step: evaluator", log.read_text())
        self.assertEqual(source.read_text(), "raise SystemExit(9)\n")

    def test_python_cache_inputs_cannot_be_written_by_a_gate(self):
        code = ("import errno,sys\nfrom pathlib import Path\n"
                "assert sys.pycache_prefix == '/usr/.baryonos-pycache'\n"
                "try: Path(sys.pycache_prefix).mkdir(parents=True,exist_ok=True)\n"
                "except OSError as error: assert error.errno in (errno.EROFS,errno.EACCES)\n"
                "else: raise AssertionError('mutable interpreter cache')\n")
        done = self.sandbox.run(
            ["/usr/bin/python3", "-B", "-c", code], self.lease,
            profile="gate", read_only=True)
        self.assertEqual(done.returncode, 0, done.stderr)
        print(json.dumps({"native_gate_resource": done.resource_status}, sort_keys=True))

    def test_host_network_and_session_home_are_not_shared(self):
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            listener.listen(1)
            port = listener.getsockname()[1]
            code = ("import os,socket\nfrom pathlib import Path\n"
                    f"assert os.readlink('/proc/self/ns/net') != {os.readlink('/proc/self/ns/net')!r}\n"
                    "connection=socket.socket(); connection.settimeout(1)\n"
                    f"try: connection.connect(('127.0.0.1', {port}))\n"
                    "except OSError: pass\nelse: raise AssertionError('shared network')\n"
                    "Path('/home/agent/session-only').write_text('fixture')\n")
            done = self.python(code)
        self.assertEqual(done.returncode, 0, done.stderr)
        done = self.python("from pathlib import Path; assert not Path('/home/agent/session-only').exists()")
        self.assertEqual(done.returncode, 0, done.stderr)

    def test_gate_preserves_pipefail_and_does_not_run_later_steps(self):
        log = self.root / "gate.log"
        with mock.patch.object(agent_gate, "controller_source_commit", return_value="a" * 40):
            code = agent_gate.run_gate([("producer", "false | true", {}), ("later", "exit 9", {})],
                                       self.work, log, sandbox=self.sandbox, source_paths=[])
        self.assertEqual(code, 2)
        self.assertIn("## FAIL: producer (exit 1)", log.read_text())
        self.assertNotIn("## step: later", log.read_text())

    def test_normal_exit_and_timeout_stop_descendants(self):
        for timeout in (None, 2):
            with self.subTest(timeout=timeout):
                marker = self.work / "heartbeat"
                marker.unlink(missing_ok=True)
                body = ("from pathlib import Path\nimport time\n"
                        "Path('heartbeat').write_text(str(time.time_ns()))\n"
                        "print('ready',flush=True)\n"
                        "while True:\n Path('heartbeat').write_text(str(time.time_ns())); time.sleep(0.02)")
                code = ("import subprocess,time; child=subprocess.Popen("
                        f"['/usr/bin/python3','-I','-B','-c',{body!r}],"
                        "stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True); "
                        "assert child.stdout.readline() == 'ready\\n'; "
                        "print('started',flush=True); time.sleep(" + ("30" if timeout else "0.2") + ")")
                if timeout:
                    with self.assertRaises(subprocess.TimeoutExpired) as raised:
                        self.python(code, timeout=timeout)
                    self.assertIn("started", raised.exception.stdout)
                else:
                    done = self.python(code)
                    self.assertEqual(done.returncode, 0, done.stderr)
                first = marker.read_text()
                time.sleep(0.15)
                self.assertEqual(marker.read_text(), first)

    # The quarantined mount intentionally persists until the outer fixture exits.
    def test_z_live_survivor_cleanup_failure_quarantines_the_root(self):
        marker = self.work / "quarantine-heartbeat"
        body = ("from pathlib import Path\nimport time\n"
                "deadline=time.monotonic()+20\n"
                "Path('quarantine-heartbeat').write_text(str(time.time_ns()))\n"
                "print('ready',flush=True)\n"
                "while time.monotonic() < deadline:\n"
                " Path('quarantine-heartbeat').write_text(str(time.time_ns())); time.sleep(0.02)")
        code = ("import subprocess,time; child=subprocess.Popen("
                f"['/usr/bin/python3','-I','-B','-c',{body!r}],"
                "stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True,start_new_session=True); "
                "assert child.stdout.readline() == 'ready\\n'; print('x'*2048,flush=True); time.sleep(20)")
        captured = {}
        termination_started = Event()
        release_termination = Event()

        def uncertain_termination(_control, _environment, unit):
            captured["unit"] = unit
            termination_started.set()
            if not release_termination.wait(5):
                raise SandboxError("termination uncertainty probe was not released")
            raise SandboxError("synthetic termination uncertainty")

        def uncertain_cleanup(_control, _environment, unit, _state):
            self.assertEqual(captured.get("unit"), unit)
            raise SandboxError("synthetic cleanup uncertainty")

        try:
            with mock.patch.object(self.sandbox, "_terminate_scope", side_effect=uncertain_termination), \
                    mock.patch.object(self.sandbox, "_cleanup_scope", side_effect=uncertain_cleanup), \
                    ThreadPoolExecutor(max_workers=1) as pool:
                future = pool.submit(self.python, code, output_limit=1024)
                try:
                    self.assertTrue(termination_started.wait(5), "termination path did not start")
                    first = marker.read_text()
                    time.sleep(0.15)
                    self.assertNotEqual(marker.read_text(), first)
                finally:
                    release_termination.set()
                with self.assertRaisesRegex(SandboxError, "captured output exceeded") as raised:
                    future.result(timeout=15)
            notes = "\n".join(raised.exception.__notes__)
            self.assertIn("isolated resource cleanup failed", notes)
            self.assertIn("isolated resource termination failed", notes)
            self.assertTrue(self.lease.receipt["quarantined"])
            self.assertFalse(self.lease.receipt["cleaned"])
            self.assertTrue(self.lease.container.exists())
            self.assertTrue(any(self.lease.root.iterdir()))
        finally:
            release_termination.set()
            unit = captured.get("unit")
            if unit is not None:
                with self.sandbox._verified_resource_tools() as (_manager, control, environment):
                    self.sandbox._terminate_scope(control, environment, unit)
                    state = self.sandbox._unit_properties(control, environment, unit)
                    self.sandbox._cleanup_scope(control, environment, unit, state)
        stopped = marker.read_text()
        time.sleep(0.15)
        self.assertEqual(marker.read_text(), stopped)


if __name__ == "__main__":
    unittest.main()
