"""Execution stays closed until an isolated context is available."""
from __future__ import annotations

import contextlib
import io
import json
import sys
import tempfile
import unittest.mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import agent_gate
import agent_pipeline
import common


class ExecutionHoldTests(unittest.TestCase):
    def test_pipeline_stops_before_issue_selection_or_execution(self):
        with tempfile.TemporaryDirectory() as directory, \
                unittest.mock.patch.object(sys, "argv", ["agent_pipeline.py", "--once"]), \
                unittest.mock.patch.object(agent_pipeline, "ROOT", Path(directory)), \
                unittest.mock.patch.object(agent_pipeline, "sh") as controller, \
                unittest.mock.patch.object(agent_pipeline.subprocess, "run") as child, \
                contextlib.redirect_stderr(io.StringIO()):
            root = Path(directory)
            (root / ".github").mkdir()
            (root / ".github/agent-pipeline.json").write_text(json.dumps({"schema": 1, "enabled": False}))
            with self.assertRaises(SystemExit) as stopped:
                agent_pipeline.main()
            self.assertEqual(stopped.exception.code, 2)
        controller.assert_not_called()
        child.assert_not_called()

    def test_agent_helper_has_no_ambient_execution_fallback(self):
        with tempfile.TemporaryDirectory() as directory, \
                unittest.mock.patch.object(agent_pipeline.subprocess, "run") as child:
            root = Path(directory)
            with self.assertRaises(agent_pipeline.TaskError):
                agent_pipeline.run_agent("", root, root / "agent.log", ["claude"])
        child.assert_not_called()

    def test_gate_stops_before_environment_discovery_or_execution(self):
        with tempfile.TemporaryDirectory() as directory, \
                unittest.mock.patch.object(agent_gate, "ROOT", Path(directory)), \
                unittest.mock.patch.object(common, "environment") as discover, \
                unittest.mock.patch.object(agent_gate.subprocess, "Popen") as child:
            root = Path(directory)
            (root / ".github").mkdir()
            (root / ".github/agent-pipeline.json").write_text(json.dumps({"schema": 1, "enabled": False}))
            code = agent_gate.run_gate([], root, root / "gate.log")
            self.assertEqual(code, agent_gate.FAIL_EXIT)
            self.assertTrue((root / "gate.log").is_file())
        discover.assert_not_called()
        child.assert_not_called()


if __name__ == "__main__":
    unittest.main()
