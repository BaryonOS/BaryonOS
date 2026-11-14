# SPDX-License-Identifier: MPL-2.0
"""Token Guard must reduce displayed output without changing command evidence."""

import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
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
        self.env.pop("TOKENGUARD_AUTO_WRAP", None)

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

    def test_only_large_failed_summary_filters_cargo_progress(self):
        progress = "   Compiling progress_crate v1.2.3\n"
        diagnostic = "error[E0308]: keep this failure\n   Compiling compiler v1.0.0 warning: retain\n"
        for mode, code, count in (("summary", 1, 400), ("bounded", 1, 400),
                                   ("summary", 0, 400), ("summary", 1, 1)):
            with self.subTest(mode=mode, code=code, count=count):
                result = self.run_python(f"import sys; print({progress!r}*{count}, end=''); print({diagnostic!r}); sys.exit({code})",
                                         "--output", mode)
                receipt = json.loads(result.stdout.splitlines()[0])
                self.assertEqual(result.returncode, code)
                self.assertIn("keep this failure", result.stdout)
                self.assertIn("warning: retain", result.stdout)
                if mode == "summary" and code and count > 1:
                    self.assertNotIn("Compiling progress_crate", result.stdout)
                    self.assertFalse(receipt["output_complete"])
                else:
                    self.assertIn("Compiling progress_crate", result.stdout)
                self.assertIn(progress.encode(), (self.root / receipt["log"]).read_bytes())

    def test_literal_arguments_do_not_become_shell_programs(self):
        dangerous = f"; touch {self.root / 'unexpected'}"
        result = self.cli(
            "run", "--output", "bounded", "--", sys.executable,
            "-c", "import sys; print(repr(sys.argv[1:]))", dangerous, "$HOME",
        )
        self.assertEqual(result.returncode, 0)
        self.assertIn("$HOME", result.stdout)
        self.assertFalse((self.root / "unexpected").exists())

    def test_native_and_wrapped_stdin_environment_and_argv_match(self):
        names = ("CARGO_TERM_COLOR", "NO_COLOR", "PYTHONDONTWRITEBYTECODE", "RTK_TELEMETRY_DISABLED")
        env = {k: v for k, v in self.env.items() if k not in names}
        code = ("import json,os,sys; print(json.dumps({'stdin':sys.stdin.read(),"
                f"'env':{{k:os.environ.get(k) for k in {names!r}}},'argv':sys.argv[1:]}}))")
        command = [sys.executable, "-c", code, "literal spaces", "$HOME", "*.rs"]
        native = subprocess.run(command, cwd=self.root, env=env, input="input π\n",
                                capture_output=True, text=True, timeout=5)
        wrapped = self.cli("run", "--", *command, stdin="input π\n", env=env)
        self.assertEqual(wrapped.returncode, native.returncode, wrapped.stderr)
        receipt = self.receipt()
        self.assertEqual((self.root / receipt["log"]).read_bytes(), native.stdout.encode())
        self.assertEqual(receipt["stdin"], "inherited")
        self.assertIsNone(receipt["timeout_seconds"])
        self.assertIn("not a transparent TTY", receipt["capture_mode"])

    def test_installed_helper_operates_outside_project(self):
        core = CLI.parents[1] / "plugins/token-guard/skills/token-guard/scripts/tokenguard.py"
        with tempfile.TemporaryDirectory(prefix="installed plugin ") as external:
            helper = Path(external) / "tokenguard.py"
            shutil.copyfile(core, helper)
            result = subprocess.run(
                [sys.executable, "-B", str(helper), "--root", str(self.root), "run", "--",
                 sys.executable, "-c", "from pathlib import Path; print(Path.cwd())"],
                capture_output=True, text=True, timeout=5, env=self.env)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn(str(self.root), result.stdout)
            self.assertEqual(self.receipt()["exit_code"], 0)
            payload = {"hook_event_name": "PreToolUse", "tool_name": "Bash", "cwd": str(self.root),
                       "tool_input": {"command": "python3 -m unittest"}}
            hooked = subprocess.run([sys.executable, "-B", str(helper), "hook"],
                                    input=json.dumps(payload), capture_output=True, text=True, timeout=5,
                                    env={**self.env, "TOKENGUARD_AUTO_WRAP": "1"})
            argv = shlex.split(json.loads(hooked.stdout)["hookSpecificOutput"]["updatedInput"]["command"])
            self.assertEqual(argv[2], str(helper))
            self.assertEqual(argv[argv.index("--root") + 1], str(self.root))

    def test_bounded_output_mode_does_not_hide_successful_data(self):
        result = self.run_python("print('tracked change: source.rs')", "--output", "bounded")
        self.assertEqual(result.returncode, 0)
        self.assertIn("tracked change: source.rs", result.stdout)

    def test_bounded_short_data_keeps_every_line_once(self):
        result = self.run_python("print('\\n'.join('unique-row-%03d' % n for n in range(30)))", "--output", "bounded")
        self.assertEqual(result.returncode, 0)
        for n in range(30):
            self.assertEqual(result.stdout.count(f"unique-row-{n:03d}"), 1)

    def test_short_ui_measurements_and_results_are_complete_in_both_modes(self):
        output = "frame_p95_ms=14.2\ncontrast_ratio=7.1\n" + "UI detail\n" * 30 + "12 passed\n"
        for mode in ("summary", "bounded"):
            with self.subTest(mode=mode):
                result = self.run_python(f"print({output!r}, end='')", "--output", mode)
                receipt = json.loads(result.stdout.splitlines()[0])
                self.assertEqual(result.returncode, 0)
                self.assertIn(output, result.stdout)
                self.assertTrue(receipt["output_complete"])

    @unittest.skipUnless(os.name == "posix", "executable fixture uses POSIX shebang")
    def test_short_single_line_json_bypasses_rtk_in_both_modes(self):
        payload = json.dumps({"data": "x" * 4500, "frame_p95_ms": 14.2, "result": "12 passed"})
        env = self.fake_rtk_env()
        for mode in ("summary", "bounded"):
            with self.subTest(mode=mode):
                result = self.cli("run", "--output", mode, "--", sys.executable, "-c",
                                  f"print({payload!r})", env=env)
                receipt = json.loads(result.stdout.splitlines()[0])
                self.assertEqual(result.returncode, 0)
                self.assertIn(payload, result.stdout)
                self.assertTrue(receipt["output_complete"])
                self.assertFalse(receipt["rtk_used"])
                self.assertNotIn("filtered", result.stdout)

    def test_complete_means_sanitized_text_and_raw_bytes_remain_available(self):
        result = self.run_python("import sys; sys.stdout.buffer.write(b'\\x1b[31mred\\x1b[0m\\x00\\xff')")
        receipt = self.receipt()
        self.assertTrue(receipt["output_complete"])
        self.assertIn("UTF-8 replacement", receipt["sanitization"])
        self.assertIn("red�", result.stdout)
        self.assertEqual((self.root / receipt["log"]).read_bytes(), b"\x1b[31mred\x1b[0m\x00\xff")
        displayed = json.loads(result.stdout.splitlines()[0])
        self.assertEqual(displayed["presentation_scope"], "sanitized_text")
        self.assertEqual(displayed["emitted_bytes"], len(result.stdout.encode("utf-8")))
        self.assertEqual(receipt["emitted_bytes"], displayed["emitted_bytes"])

    def test_large_summary_always_declares_selection(self):
        result = self.run_python("print('frame_p95_ms=14.2'); print('progress line\\n'*2000); print('12 passed')")
        receipt = self.receipt()
        self.assertEqual(result.returncode, 0)
        self.assertFalse(receipt["output_complete"])
        self.assertIn("OUTPUT INCOMPLETE", result.stdout)
        self.assertIn(b"frame_p95_ms=14.2", (self.root / receipt["log"]).read_bytes())

    def test_terminal_error_survives_many_early_warnings(self):
        result = self.run_python(
            "import sys; print('warning: early diagnostic\\n'*400); "
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
                          "import sys; print('progress line\\n'*3000); sys.exit(9)", env=self.fake_rtk_env())
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

    def test_invalid_explicit_timeouts_do_not_run_the_command(self):
        for value in ("0", "-1", "nan", "inf"):
            with self.subTest(timeout=value):
                result = self.run_python("from pathlib import Path; Path('unexpected').touch()", "--timeout", value)
                self.assertEqual(result.returncode, 2)
                self.assertFalse((self.root / "unexpected").exists())

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
            "print('  nested stack frame\\n'*500); "
            "print('FileNotFoundError: missing-program-marker'); sys.exit(1)"
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("FileNotFoundError: missing-program-marker", result.stdout)
        self.assertIn("OMITTED", result.stdout)

    def test_oversized_diagnostic_line_number_cannot_replace_command_exit(self):
        diagnostic = 'File "x.py", line ' + '9' * 5000 + '\n'
        result = self.run_python(f"import sys; print({diagnostic!r}, end=''); sys.exit(7)")
        self.assertEqual(result.returncode, 7, result.stderr)
        receipt = self.receipt()
        self.assertEqual(receipt["exit_code"], 7)
        self.assertTrue(receipt["output_complete"])
        self.assertIn(diagnostic, result.stdout)
        self.assertEqual((self.root / receipt["log"]).read_text(), diagnostic)

    def test_auto_source_is_labelled_as_post_command_current_file(self):
        source = self.root / "mutated.rs"
        source.write_text("before_run = 1\n")
        result = self.run_python(
            "from pathlib import Path; import sys; print('error: changed source\\n --> mutated.rs:1:1'); "
            "Path('mutated.rs').write_text('after_run = 2\\n'); sys.exit(1)"
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("after_run = 2", result.stdout)
        self.assertIn("post-command/current file; not tested-source identity", result.stdout)
        self.assertEqual(self.receipt()["source_context"], "post-command/current file; not tested-source identity")

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

    def test_cli_uses_utf8_without_changing_streams_on_import(self):
        (self.root / "unicode.py").write_text("def π(κ: int) -> str:\n    return 'Unicode'\n", encoding="utf-8")
        env = {**self.env, "PYTHONIOENCODING": "ascii"}
        result = subprocess.run([sys.executable, "-B", str(CLI), "--root", str(self.root), "inspect", "unicode.py"],
                                capture_output=True, timeout=5, env=env)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("π(κ: int)", result.stdout.decode("utf-8"))
        core = CLI.parents[1] / "plugins/token-guard/skills/token-guard/scripts/tokenguard.py"
        probe = ("import importlib.util,json,sys; "
                 "before=[s.encoding for s in (sys.stdin,sys.stdout,sys.stderr)]; "
                 f"spec=importlib.util.spec_from_file_location('core',{str(core)!r}); "
                 "module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module); "
                 "print(json.dumps([before,[s.encoding for s in (sys.stdin,sys.stdout,sys.stderr)]]))")
        imported = subprocess.run([sys.executable, "-B", "-c", probe], capture_output=True, timeout=5, env=env)
        self.assertEqual(imported.returncode, 0, imported.stderr)
        before, after = json.loads(imported.stdout)
        self.assertEqual(before, after)
        self.assertEqual(before, ["ascii"] * 3)

    def test_non_python_outline_is_labelled_heuristic(self):
        (self.root / "sample.rs").write_text("pub fn value() -> u32 { 42 }\n")
        result = self.cli("inspect", "sample.rs")
        self.assertEqual(result.returncode, 0)
        self.assertIn("heuristic", result.stdout.lower())

    def test_rust_multiline_signature_retains_paths_lifetimes_and_where_clause(self):
        header = (
            "pub fn process<'a, T>(\n"
            "    request: crate::ipc::Request<'a, T>,\n"
            "    capability: &crate::capability::Handle,\n"
            "    length: usize,\n"
            "    retries: u8,\n"
            ") -> Result<crate::types::Reply<T>, crate::errors::Error>\n"
            "where\n"
            "    T: core::fmt::Debug + Send,\n"
            "{"
        )
        (self.root / "long.rs").write_text(header + "\n    body_must_not_appear();\n}\n")
        result = self.cli("inspect", "long.rs")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(header, result.stdout)
        self.assertIn("1-9:", result.stdout)
        self.assertIn("body omitted: lines 9-11", result.stdout)
        self.assertNotIn("body_must_not_appear", result.stdout)

    def test_typescript_multiline_return_type_and_arrow_signature(self):
        function = ("export async function render<T>(\n"
                    "    model: Model<T>,\n"
                    "    options: {\n"
                    "        theme: string;\n"
                    "    },\n"
                    "): {\n"
                    "    width: number;\n"
                    "    height: number;\n"
                    "} {")
        arrow = "export const choose = async (\n    item: Model,\n    index: number,\n): Promise<Model> =>"
        (self.root / "long.ts").write_text(function + "\n    function_body_must_not_appear();\n}\n" +
                                           arrow + " {\n    arrow_body_must_not_appear();\n};\n")
        result = self.cli("inspect", "long.ts")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(function, result.stdout)
        self.assertIn(arrow, result.stdout)
        self.assertIn("body/expression omitted", result.stdout)
        self.assertNotIn("body_must_not_appear", result.stdout)

    def test_outline_data_ranges_and_nested_comment_delimiters(self):
        source = ("pub const SMALL: usize = 42;\n"
                  "pub static DATA: [u8; 50] = [\n" + "    217,  // omitted_data_marker\n" * 50 + "];\n"
                  "pub const fn read<T: Trait</* outer { /* inner */ } */ Item>>(\n"
                  "    input: T, // fake opening {\n"
                  ") -> crate::Result<T> {\n    hidden_function_body();\n}\n")
        (self.root / "data.rs").write_text(source)
        result = self.cli("inspect", "data.rs")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("pub const SMALL: usize = 42;", result.stdout)
        self.assertIn("initializer/data omitted: lines 2-53", result.stdout)
        self.assertIn(") -> crate::Result<T> {", result.stdout)
        self.assertNotIn("omitted_data_marker", result.stdout)
        self.assertNotIn("hidden_function_body", result.stdout)

    def test_outline_marks_signature_budget_exhaustion(self):
        source = "pub fn oversized(\n" + "    argument: crate::types::LongParameterName,\n" * 200 + ") {\n    body();\n}\n"
        (self.root / "oversized.rs").write_text(source)
        result = self.cli("inspect", "oversized.rs")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("signature incomplete", result.stdout)
        self.assertIn("4000-byte scan budget", result.stdout)
        self.assertIn("continue with slice", result.stdout)
        self.assertNotIn("body();", result.stdout)
        self.assertLess(len(result.stdout.encode()), 8100)

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

    def test_byte_window_replays_the_end_of_a_giant_line(self):
        path = self.root / "giant.log"
        path.write_bytes(b"x" * 1_000_000 + b"END_OF_GIANT_LINE")
        result = self.cli("log", "giant.log", "--offset", "999997", "--length", "12000")
        self.assertEqual(result.returncode, 0, result.stderr)
        metadata = json.loads(result.stdout.splitlines()[0])
        self.assertIn("xxxEND_OF_GIANT_LINE", result.stdout)
        self.assertEqual(metadata["offset"], 999997)
        self.assertEqual(metadata["next_offset"], path.stat().st_size)
        self.assertEqual(metadata["file_size"], path.stat().st_size)
        self.assertTrue(metadata["eof"])
        self.assertEqual(metadata["offset_unit"], "raw bytes")
        self.assertIn("control characters removed", metadata["sanitization"])

    def test_byte_window_validates_range_mode_and_root(self):
        (self.root / "small.log").write_bytes(b"12345")
        cases = [("--offset", "-1"), ("--offset", "6"), ("--length", "0"),
                 ("--length", "12001"), ("--offset", "0", "--start", "1"),
                 ("--length", "5", "--lines", "1")]
        for options in cases:
            with self.subTest(options=options):
                self.assertEqual(self.cli("log", "small.log", *options).returncode, 2)
        with tempfile.TemporaryDirectory() as external:
            path = Path(external) / "outside.log"
            path.write_text("outside")
            self.assertEqual(self.cli("log", str(path), "--offset", "0").returncode, 2)
            if os.name == "posix":
                (self.root / "outside-link.log").symlink_to(path)
                self.assertEqual(self.cli("log", "outside-link.log", "--offset", "0").returncode, 2)

    def test_stats_reports_measurements_not_token_guarantees(self):
        self.run_python("print('progress\\n'*200)")
        result = self.cli("stats")
        self.assertEqual(result.returncode, 0)
        self.assertIn("bytes", result.stdout)
        self.assertNotIn("tokens saved", result.stdout.lower())

    def hook(self, command, *, enabled=False, **extra):
        payload = {"hook_event_name": "PreToolUse", "tool_name": "Bash",
                   "cwd": str(self.root), "tool_input": {"command": command, "timeout": 12345}}
        payload.update(extra)
        env = {**self.env, "TOKENGUARD_AUTO_WRAP": "1"} if enabled else self.env
        return self.cli("hook", stdin=json.dumps(payload), env=env)

    def test_claude_hook_is_a_noop_without_explicit_opt_in(self):
        result = self.hook("python3 -B tools/preflight.py --strict")
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertEqual(result.stderr, "")

    def test_claude_hook_preserves_permissions_arguments_and_tool_fields(self):
        result = self.hook("python3 -B tools/preflight.py --strict", enabled=True)
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
                result = self.hook(command, enabled=True)
                self.assertEqual(result.returncode, 0)
                self.assertNotIn("updatedInput", result.stdout)

    def test_opted_in_hook_skips_background_watch_and_interactive_commands(self):
        for command in ("python3 -m pytest --pdb", "python3 -m pytest -s", "cargo test --watch",
                        "python3 -m pytest --watch=true", "python3 tools/run.py"):
            with self.subTest(command=command):
                self.assertEqual(self.hook(command, enabled=True).stdout, "")
        result = self.hook("cargo test", enabled=True,
                           tool_input={"command": "cargo test", "run_in_background": True})
        self.assertEqual(result.stdout, "")
        self.assertEqual(result.returncode, 0)

    def test_malformed_hook_never_blocks_an_operation(self):
        for payload in ("", "{broken", "[]", "null", '{"tool_input": null}'):
            with self.subTest(payload=payload):
                result = self.cli("hook", stdin=payload, env={**self.env, "TOKENGUARD_AUTO_WRAP": "1"})
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertNotIn("permissionDecision", result.stdout)


if __name__ == "__main__":
    unittest.main()
