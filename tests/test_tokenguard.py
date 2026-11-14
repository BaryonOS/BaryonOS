# SPDX-License-Identifier: MPL-2.0
"""Token Guard must reduce displayed output without changing command evidence."""

import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
import time
import unittest


CLI = Path(__file__).resolve().parents[1] / "tools" / "tokenguard.py"


class TokenGuardTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.env = {**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}

    def cli(self, *args, stdin=None, env=None):
        return subprocess.run(
            [sys.executable, "-B", str(CLI), "--root", str(self.root), *args],
            input=stdin, capture_output=True, text=True, timeout=15,
            env=self.env if env is None else env,
        )

    def run_python(self, source, *options):
        return self.cli("run", *options, "--", sys.executable, "-c", source)

    def receipt(self):
        paths = list((self.root / "build" / "tokenguard").glob("*.json"))
        self.assertEqual(len(paths), 1)
        return json.loads(paths[0].read_text())

    def test_failed_command_retains_exit_and_late_rust_diagnostic(self):
        source = self.root / "source.rs"
        source.write_text("\n".join(f"line_{n}" for n in range(1, 16)))
        result = self.run_python(
            "import sys; print('progress line\\n' * 4000); "
            "print('error[E0308]: incompatible type\\n --> source.rs:7:1'); "
            "sys.exit(7)"
        )
        self.assertEqual(result.returncode, 7, result.stderr)
        self.assertIn("error[E0308]", result.stdout)
        self.assertIn("line_7", result.stdout)
        self.assertLess(len(result.stdout.encode()), 12000)
        receipt = self.receipt()
        self.assertEqual(receipt["exit_code"], 7)
        self.assertEqual(receipt["emitted_bytes"], len(result.stdout.encode()))
        self.assertGreater(receipt["raw_bytes"], receipt["emitted_bytes"])
        logs = list((self.root / "build" / "tokenguard").glob("*.log"))
        raw = logs[0].read_bytes()
        self.assertEqual(len(raw), receipt["raw_bytes"])
        self.assertIn(b"progress line\nprogress line", raw)

    def test_success_keeps_warning_and_test_summary(self):
        result = self.run_python(
            "print('working\\n' * 300); print('warning: deprecated API'); "
            "print('Ran 7 tests in 0.01s'); print('OK')"
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("warning:", result.stdout)
        self.assertIn("7 tests", result.stdout)
        self.assertEqual(self.receipt()["exit_code"], 0)

    def test_literal_arguments_do_not_become_shell_programs(self):
        dangerous = f"; touch {self.root / 'unexpected'}"
        result = self.cli(
            "run", "--output", "bounded", "--", sys.executable,
            "-c", "import sys; print(repr(sys.argv[1:]))", dangerous, "$HOME",
        )
        self.assertEqual(result.returncode, 0)
        self.assertIn("$HOME", result.stdout)
        self.assertFalse((self.root / "unexpected").exists())

    def test_bounded_output_mode_does_not_hide_successful_data(self):
        result = self.run_python("print('tracked change: source.rs')", "--output", "bounded")
        self.assertEqual(result.returncode, 0)
        self.assertIn("tracked change: source.rs", result.stdout)

    def test_bounded_short_data_keeps_every_line_once(self):
        result = self.run_python("print('\\n'.join('unique-row-%03d' % n for n in range(30)))", "--output", "bounded")
        self.assertEqual(result.returncode, 0)
        for n in range(30):
            self.assertEqual(result.stdout.count(f"unique-row-{n:03d}"), 1)

    def test_terminal_error_survives_many_early_warnings(self):
        result = self.run_python(
            "import sys; print('warning: early diagnostic\\n'*90); "
            "print('ERROR: terminal_failure_marker'); sys.exit(9)"
        )
        self.assertEqual(result.returncode, 9)
        self.assertIn("terminal_failure_marker", result.stdout)
        self.assertRegex(result.stdout.lower(), "sampl|omitt|truncat")

    def test_missing_program_is_failure(self):
        result = self.cli("run", "--", "tokenguard-command-that-does-not-exist")
        self.assertEqual(result.returncode, 127)
        self.assertEqual(self.receipt()["exit_code"], 127)

    def fake_rtk_env(self):
        directory = self.root / "fake-bin"
        directory.mkdir()
        executable = directory / "rtk"
        executable.write_text(f"#!{sys.executable}\nprint('filtered')\n")
        executable.chmod(0o755)
        return {**self.env, "PATH": str(directory) + os.pathsep + self.env.get("PATH", "")}

    @unittest.skipUnless(os.name == "posix", "executable fixture uses POSIX shebang")
    def test_rtk_filter_cannot_change_failed_command_exit(self):
        result = self.cli("run", "--", sys.executable, "-c",
                          "import sys; print('progress line\\n'*300); sys.exit(9)", env=self.fake_rtk_env())
        self.assertEqual(result.returncode, 9)
        self.assertTrue(self.receipt()["rtk_used"])

    @unittest.skipUnless(os.name == "posix", "executable fixture uses POSIX shebang")
    def test_bounded_data_never_uses_rtk_log_filter(self):
        result = self.cli("run", "--output", "bounded", "--", sys.executable, "-c",
                          "print('useful-query-result\\n'*10)", env=self.fake_rtk_env())
        self.assertEqual(result.returncode, 0)
        self.assertIn("useful-query-result", result.stdout)
        self.assertFalse(self.receipt()["rtk_used"])

    def test_timeout_is_failure(self):
        result = self.run_python("import time; time.sleep(10)", "--timeout", "0.1")
        self.assertEqual(result.returncode, 124)
        self.assertEqual(self.receipt()["exit_code"], 124)

    @unittest.skipUnless(os.name == "posix", "POSIX process group contract")
    def test_timeout_stops_descendant_process(self):
        marker = self.root / "descendant-survived"
        child = f"import time,pathlib; time.sleep(1); pathlib.Path({str(marker)!r}).touch()"
        parent = f"import subprocess,sys,time; subprocess.Popen([sys.executable,'-c',{child!r}]); time.sleep(10)"
        result = self.run_python(parent, "--timeout", "0.2")
        self.assertEqual(result.returncode, 124)
        time.sleep(1.1)
        self.assertFalse(marker.exists(), "timeout left a child running")

    @unittest.skipUnless(os.name == "posix", "POSIX exit convention")
    def test_signal_exit_is_not_success(self):
        result = self.run_python("import os,signal; os.kill(os.getpid(),signal.SIGTERM)")
        self.assertEqual(result.returncode, 143)

    @unittest.skipUnless(os.name == "posix", "POSIX cancellation contract")
    def test_outer_cancellation_stops_child_and_records_failure(self):
        ready = self.root / "ready"
        survived = self.root / "survived"
        source = f"import time,pathlib; pathlib.Path({str(ready)!r}).touch(); time.sleep(0.5); pathlib.Path({str(survived)!r}).touch()"
        process = subprocess.Popen(
            [sys.executable, "-B", str(CLI), "--root", str(self.root), "run", "--", sys.executable, "-c", source],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=self.env,
        )
        try:
            deadline = time.monotonic() + 5
            while not ready.exists() and time.monotonic() < deadline:
                time.sleep(0.01)
            self.assertTrue(ready.exists())
            process.terminate()
            stdout, stderr = process.communicate(timeout=5)
            self.assertEqual(process.returncode, 143, stderr)
            self.assertIn("cancelled", stdout)
            time.sleep(0.6)
            self.assertFalse(survived.exists())
            self.assertEqual(self.receipt()["exit_code"], 143)
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()

    @unittest.skipUnless(os.name == "posix", "POSIX cancellation contract")
    def test_cancel_signal_inside_wait_does_not_wait_recursively(self):
        core = CLI.parents[1] / "plugins/token-guard/skills/token-guard/scripts/tokenguard.py"
        source = (
            "import importlib.util,os,signal,subprocess,sys\n"
            "from pathlib import Path\n"
            f"spec=importlib.util.spec_from_file_location('token_guard_probe',{str(core)!r})\n"
            "module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)\n"
            "original=subprocess.Popen\n"
            "class TimedSignal(original):\n"
            "    sent=False\n"
            "    def _try_wait(self, wait_flags):\n"
            "        if not self.sent and self._waitpid_lock.locked():\n"
            "            self.sent=True\n"
            "            os.kill(os.getpid(),signal.SIGTERM)\n"
            "        return super()._try_wait(wait_flags)\n"
            "module.subprocess.Popen=TimedSignal\n"
            f"sys.exit(module.run_command(Path({str(self.root)!r}),[sys.executable,'-c','import time; time.sleep(2)']))\n"
        )
        result = subprocess.run([sys.executable, "-B", "-c", source], capture_output=True,
                                text=True, timeout=5, env=self.env)
        self.assertEqual(result.returncode, 143, result.stderr)

    @unittest.skipUnless(os.name == "posix", "POSIX cancellation contract")
    def test_signals_during_spawn_stop_child_before_handle_returns(self):
        core = CLI.parents[1] / "plugins/token-guard/skills/token-guard/scripts/tokenguard.py"
        for signal_name, expected in (("SIGTERM", 143), ("SIGINT", 130)):
            with self.subTest(signal_name=signal_name):
                marker = self.root / signal_name
                child = f"import time,pathlib; time.sleep(.3); pathlib.Path({str(marker)!r}).touch()"
                source = (
                    "import importlib.util,os,signal,subprocess,sys\nfrom pathlib import Path\n"
                    f"spec=importlib.util.spec_from_file_location('token_guard_probe',{str(core)!r})\n"
                    "module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)\n"
                    "original=subprocess.Popen\n"
                    "class TimedSignal(original):\n"
                    "    def __init__(self,*args,**kwargs):\n"
                    "        super().__init__(*args,**kwargs)\n"
                    f"        os.kill(os.getpid(),signal.{signal_name})\n"
                    "module.subprocess.Popen=TimedSignal\n"
                    f"sys.exit(module.run_command(Path({str(self.root)!r}),[sys.executable,'-c',{child!r}]))\n"
                )
                result = subprocess.run([sys.executable, "-B", "-c", source], capture_output=True,
                                        text=True, timeout=5, env=self.env)
                self.assertEqual(result.returncode, expected, result.stderr)
                time.sleep(.4)
                self.assertFalse(marker.exists())

    def test_python_traceback_includes_local_source(self):
        script = self.root / "failure with spaces.py"
        script.write_text("value = 0\nraise ValueError('source-marker')\n")
        result = self.cli("run", "--", sys.executable, str(script))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("source-marker", result.stdout)
        self.assertIn("value = 0", result.stdout)

    def test_long_traceback_retains_final_exception(self):
        result = self.run_python(
            "import sys; print('Traceback (most recent call last):'); "
            "print('  nested stack frame\\n'*40); "
            "print('FileNotFoundError: missing-program-marker'); sys.exit(1)"
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("FileNotFoundError: missing-program-marker", result.stdout)
        self.assertIn("OMITTED", result.stdout)

    def test_diagnostic_cannot_disclose_source_outside_root(self):
        with tempfile.TemporaryDirectory() as external:
            secret = Path(external) / "outside.py"
            secret.write_text("do_not_disclose_outside_source = 1\n")
            result = self.run_python(f"import sys; print(' --> {secret}:1:1'); sys.exit(1)")
        self.assertEqual(result.returncode, 1)
        self.assertNotIn("do_not_disclose_outside_source", result.stdout)

    def test_inspect_python_reads_signatures_without_body(self):
        (self.root / "sample.py").write_text(
            "def calculate(\n    amount: int,\n    factor: int = 2,\n) -> int:\n"
            "    internal_payload_should_not_appear = 'private data'\n"
            "    return amount * factor\n"
        )
        result = self.cli("inspect", "sample.py")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("calculate", result.stdout)
        self.assertIn("factor", result.stdout)
        self.assertNotIn("internal_payload_should_not_appear", result.stdout)

    def test_non_python_outline_is_labelled_heuristic(self):
        (self.root / "sample.rs").write_text("pub fn value() -> u32 { 42 }\n")
        result = self.cli("inspect", "sample.rs")
        self.assertEqual(result.returncode, 0)
        self.assertIn("heuristic", result.stdout.lower())

    def test_slice_validates_version_and_range(self):
        path = self.root / "source.py"
        path.write_text("first\nsecond\nthird\n")
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        good = self.cli("slice", "source.py", "2", "2", "--expect-sha", digest)
        self.assertEqual(good.returncode, 0)
        self.assertIn("second", good.stdout)
        self.assertNotIn("third", good.stdout)
        path.write_text("changed\n")
        self.assertNotEqual(self.cli("slice", "source.py", "1", "1", "--expect-sha", digest).returncode, 0)
        for start, end in ((0, 1), (2, 1), (1, 201)):
            with self.subTest(start=start, end=end):
                self.assertNotEqual(self.cli("slice", "source.py", str(start), str(end)).returncode, 0)

    def test_slice_rejects_outside_path_and_symlink(self):
        with tempfile.TemporaryDirectory() as external:
            path = Path(external) / "secret.txt"
            path.write_text("external data")
            self.assertNotEqual(self.cli("slice", str(path), "1", "1").returncode, 0)
            if os.name == "posix":
                (self.root / "link.txt").symlink_to(path)
                self.assertNotEqual(self.cli("slice", "link.txt", "1", "1").returncode, 0)

    def test_output_is_bounded_for_a_single_giant_line(self):
        result = self.run_python("print('X'*1000000)", "--output", "bounded")
        self.assertEqual(result.returncode, 0)
        self.assertLess(len(result.stdout.encode()), 12000)
        self.assertRegex(result.stdout.lower(), "truncat|omitt|limit")

    def test_stats_reports_measurements_not_token_guarantees(self):
        self.run_python("print('progress\\n'*200)")
        result = self.cli("stats")
        self.assertEqual(result.returncode, 0)
        self.assertIn("bytes", result.stdout)
        self.assertNotIn("tokens saved", result.stdout.lower())

    def hook(self, command, **extra):
        payload = {"hook_event_name": "PreToolUse", "tool_name": "Bash",
                   "cwd": str(self.root), "tool_input": {"command": command, "timeout": 12345}}
        payload.update(extra)
        return self.cli("hook", stdin=json.dumps(payload))

    def test_claude_hook_preserves_permissions_arguments_and_tool_fields(self):
        result = self.hook("python3 -B tools/preflight.py --strict")
        self.assertEqual(result.returncode, 0, result.stderr)
        data = json.loads(result.stdout)
        output = data["hookSpecificOutput"]
        self.assertEqual(output["hookEventName"], "PreToolUse")
        self.assertNotIn("permissionDecision", output)
        self.assertEqual(output["updatedInput"]["timeout"], 12345)
        argv = shlex.split(output["updatedInput"]["command"])
        self.assertEqual(argv[argv.index("--") + 1:], ["python3", "-B", "tools/preflight.py", "--strict"])

    def test_hook_leaves_reads_shell_expressions_and_existing_wrappers_unchanged(self):
        commands = ["git diff", "cat source.py", "cargo test && echo done",
                    "cargo test > log.txt", "cargo test $FLAGS", "FOO=1 cargo test",
                    "cargo test | tail -10", "rtk cargo test", "python3 tools/tokenguard.py run -- cargo test",
                    "python3 -m unittest discover -p 'test_*.py'"]
        for command in commands:
            with self.subTest(command=command):
                result = self.hook(command)
                self.assertEqual(result.returncode, 0)
                self.assertNotIn("updatedInput", result.stdout)

    def test_malformed_hook_never_blocks_an_operation(self):
        for payload in ("", "{broken", "[]", "null", '{"tool_input": null}'):
            with self.subTest(payload=payload):
                result = self.cli("hook", stdin=payload)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertNotIn("permissionDecision", result.stdout)


if __name__ == "__main__":
    unittest.main()
