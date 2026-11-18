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
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from agent_sandbox import Sandbox
import agent_gate


@unittest.skipUnless(os.environ.get("BARYON_SANDBOX_TEST_CONFIG"), "NOT RUN: isolated fixture runtime not provided")
class NativeBoundary(unittest.TestCase):
    def setUp(self):
        self.sandbox = Sandbox(json.loads(Path(os.environ["BARYON_SANDBOX_TEST_CONFIG"]).read_text()))
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.work = self.root / "disposable"
        self.work.mkdir(mode=0o700)

    def python(self, code, **options):
        return self.sandbox.run(
            ["/usr/bin/python3", "-I", "-B", "-c", code], self.work,
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
                "try: Path('/usr/bin/unaccepted').write_text('x')\n"
                "except OSError as error: assert error.errno in (errno.EROFS, errno.EACCES)\n"
                "else: raise AssertionError('writable runtime')\n")
        done = self.python(code)
        self.assertEqual(done.returncode, 0, done.stderr)

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
            ["/usr/bin/python3", "-B", "-c", code], self.work,
            profile="gate", read_only=True)
        self.assertEqual(done.returncode, 0, done.stderr)

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


if __name__ == "__main__":
    unittest.main()
