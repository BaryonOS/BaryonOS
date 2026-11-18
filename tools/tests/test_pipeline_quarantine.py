"""The legacy issue executor must stay inert while its replacement is unverified."""
import contextlib
import pathlib
import subprocess
import sys
import unittest
import unittest.mock

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
import agent_pipeline as pipeline


class Quarantine(unittest.TestCase):
    def test_direct_issue_selection_stops_before_any_side_effect(self):
        controller = pipeline.Pipeline(["codex"], False, False)
        with unittest.mock.patch.object(pathlib.Path, "read_text", side_effect=AssertionError("read")):
            with unittest.mock.patch.object(pipeline, "sh", side_effect=AssertionError("external command")):
                with self.assertRaisesRegex(pipeline.TaskError, "disabled"):
                    controller.ready_tasks()

    def test_scheduler_stops_before_selecting_issues(self):
        controller = pipeline.Pipeline(["codex"], False, False)
        with unittest.mock.patch.object(controller, "ready_tasks", return_value=[]) as select:
            with self.assertRaisesRegex(pipeline.TaskError, "disabled"):
                controller.run_once()
            select.assert_not_called()

    def test_transaction_ingresses_stop_before_any_side_effect(self):
        task = pipeline.Task(7, "bounded task", "parallel", "K05", ["ci.yml#host-quality"], [], "")
        controller = pipeline.Pipeline(["claude", "codex", "copilot"], True, True)
        for name in ("prepare", "execute", "integrate", "publish", "finish", "discard", "work"):
            with self.subTest(entry=name), contextlib.ExitStack() as stack:
                for target in ("sh", "run_agent", "run_gate"):
                    stack.enter_context(unittest.mock.patch.object(pipeline, target, side_effect=AssertionError(target)))
                stack.enter_context(unittest.mock.patch.object(pathlib.Path, "mkdir", side_effect=AssertionError("mkdir")))
                stack.enter_context(unittest.mock.patch.object(pathlib.Path, "read_text", side_effect=AssertionError("read")))
                stack.enter_context(unittest.mock.patch.object(pipeline.shutil, "rmtree", side_effect=AssertionError("remove")))
                with self.assertRaisesRegex(pipeline.TaskError, "disabled"):
                    getattr(controller, name)(task)

    def test_cli_flags_cannot_release_execution(self):
        for flags in (("--once",), ("--once", "--dry-run", "--auto-merge-parallel")):
            with self.subTest(flags=flags), unittest.mock.patch.object(sys, "argv", ["agent_pipeline.py", *flags]):
                with unittest.mock.patch.object(pipeline, "Pipeline") as constructor:
                    with self.assertRaises(SystemExit) as stopped:
                        pipeline.main()
                    self.assertEqual(stopped.exception.code, 2)
                    constructor.assert_not_called()

    def test_real_cli_refuses_without_credentials_or_external_tools(self):
        result = subprocess.run(
            [sys.executable, "-I", str(pathlib.Path(pipeline.__file__)), "--once", "--dry-run"],
            env={"PATH": ""}, capture_output=True, text=True, timeout=10,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn(pipeline.EXECUTION_HOLD, result.stderr)


if __name__ == "__main__":
    unittest.main()
