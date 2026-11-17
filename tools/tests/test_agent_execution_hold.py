"""Execution stays closed until an isolated context is available."""
from __future__ import annotations

import contextlib
import io
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import agent_gate
import agent_pipeline


class ExecutionHoldTests(unittest.TestCase):
    def test_pipeline_stops_before_issue_selection_or_execution(self):
        with mock.patch.object(sys, "argv", ["agent_pipeline.py", "--once"]), \
                mock.patch.object(agent_pipeline, "sh") as controller, \
                mock.patch.object(agent_pipeline.subprocess, "run") as child, \
                contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(agent_pipeline.main(), 2)
        controller.assert_not_called()
        child.assert_not_called()

    def test_agent_helper_has_no_ambient_execution_fallback(self):
        with tempfile.TemporaryDirectory() as directory, \
                mock.patch.object(agent_pipeline.subprocess, "run") as child:
            root = Path(directory)
            with self.assertRaises(agent_pipeline.TaskError):
                agent_pipeline.run_agent("", root, root / "agent.log", ["claude"])
        child.assert_not_called()

    def test_gate_stops_before_environment_discovery_or_execution(self):
        with tempfile.TemporaryDirectory() as directory, \
                mock.patch.object(agent_gate, "environment") as discover, \
                mock.patch.object(agent_gate.subprocess, "call") as child:
            root = Path(directory)
            code = agent_gate.run_gate([], root, root / "gate.log")
            self.assertEqual(code, agent_gate.FAIL_EXIT)
            self.assertTrue((root / "gate.log").is_file())
        discover.assert_not_called()
        child.assert_not_called()


if __name__ == "__main__":
    unittest.main()
