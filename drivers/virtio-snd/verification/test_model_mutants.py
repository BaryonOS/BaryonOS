"""Exercise mutation-runner verdicts and restoration without invoking Cargo."""
import contextlib
import importlib.util
import io
import pathlib
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("model_mutants", pathlib.Path(__file__).with_name("model-mutants.py"))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


def result(code, stdout, stderr=""):
    return subprocess.CompletedProcess([], code, stdout, stderr)


PASS = result(0, "running 2 tests\n..\ntest result: ok. 2 passed; 0 failed;\n")
FAIL = result(101, "running 2 tests\nfailures:\n    transport_model::tests::regression\n"
              "test result: FAILED. 1 passed; 1 failed;\n", "error: test failed, to rerun pass `--lib`\n")
COMPILE_ERROR = result(101, "", "error: could not compile crate\n")


class RunnerTests(unittest.TestCase):
    def run_runner(self, outcomes):
        with tempfile.TemporaryDirectory() as directory:
            source = pathlib.Path(directory) / "model.rs"
            source.write_text("original")
            with patch.object(runner, "SRC", source), patch.object(runner, "MUTANTS", [("M", "test", "original", "mutated")]), patch.object(runner.subprocess, "run", side_effect=outcomes) as run, contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                status = runner.main()
            self.assertEqual(source.read_text(), "original")
            return status, run.call_count

    def test_baseline_failure_stops_before_mutation(self):
        self.assertEqual(self.run_runner([FAIL]), (2, 1))

    def test_compile_error_is_not_caught(self):
        self.assertEqual(self.run_runner([PASS, COMPILE_ERROR]), (2, 2))

    def test_test_failure_catches_mutant(self):
        self.assertEqual(self.run_runner([PASS, FAIL]), (0, 2))

    def test_surviving_mutant_fails_run(self):
        self.assertEqual(self.run_runner([PASS, PASS]), (1, 2))

    def test_zero_tests_and_mixed_errors_are_rejected(self):
        self.assertEqual(runner.test_outcome(result(0, "running 0 tests\ntest result: ok. 0 passed; 0 failed;\n"))[0], "ERROR")
        self.assertEqual(runner.test_outcome(result(101, FAIL.stdout, FAIL.stderr + "error: could not compile crate\n"))[0], "ERROR")

    def test_missing_cargo_fails_and_restores_source(self):
        self.assertEqual(self.run_runner([FileNotFoundError("cargo")]), (2, 1))


if __name__ == "__main__":
    unittest.main()
