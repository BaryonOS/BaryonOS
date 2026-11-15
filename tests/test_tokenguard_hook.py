# SPDX-License-Identifier: MPL-2.0
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
import unittest


CLI = Path(__file__).resolve().parents[1] / "tools" / "tokenguard.py"


class ReadHookTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.file = self.root / "source file.py"
        self.file.write_text("def contract(value: int) -> int:\n    return value\n" + "# filler\n" * 180, encoding="utf-8")

    def hook(self, tool, inputs, *, enabled=True):
        env = {**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}
        env.pop("TOKENGUARD_AUTO_WRAP", None)
        env.pop("TOKENGUARD_READ_GUARD", None)
        if enabled:
            env["TOKENGUARD_READ_GUARD"] = "1"
        payload = {"hook_event_name": "PreToolUse", "cwd": str(self.root), "tool_name": tool, "tool_input": inputs}
        return subprocess.run([sys.executable, "-B", str(CLI), "hook"], input=json.dumps(payload),
                              text=True, capture_output=True, env=env, timeout=5)

    def test_default_is_noop_and_enabled_large_read_includes_outline_once(self):
        inputs = {"file_path": str(self.file)}
        off = self.hook("Read", inputs, enabled=False)
        self.assertEqual((off.returncode, off.stdout, off.stderr), (0, "", ""))
        on = self.hook("Read", inputs)
        self.assertEqual(on.returncode, 2)
        self.assertIn("def contract(value: int) -> int", on.stderr)
        self.assertIn("sha256=", on.stderr)
        self.assertEqual(on.stdout, "")

    def test_bounded_native_read_and_instructions_remain_available(self):
        bounded = self.hook("Read", {"file_path": str(self.file), "offset": 1, "limit": 160})
        self.assertEqual(bounded.returncode, 0)
        oversized = self.hook("Read", {"file_path": str(self.file), "limit": 161})
        self.assertEqual(oversized.returncode, 2)
        doc = self.root / "AGENTS.md"
        doc.write_text("mandatory instruction\n" * 200, encoding="utf-8")
        self.assertEqual(self.hook("Read", {"file_path": str(doc)}).returncode, 0)

    def test_literal_cat_and_powershell_reads_receive_outline(self):
        for command in [f"cat {shlex.quote(str(self.file))}", f'Get-Content -LiteralPath "{self.file}"']:
            result = self.hook("Bash", {"command": command})
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertIn("def contract", result.stderr)
        result = self.hook("Bash", {"command": f'Get-Content "{self.file}" -TotalCount 20'})
        self.assertEqual(result.returncode, 0)

    def test_token_guard_substring_is_not_a_blanket_bypass(self):
        renamed = self.root / "token-guard-source.py"
        renamed.write_bytes(self.file.read_bytes())
        self.assertEqual(self.hook("Read", {"file_path": str(renamed)}).returncode, 2)
        self.assertEqual(self.hook("Bash", {"command": f"python3 {CLI} inspect anything.py"}).returncode, 0)

    @unittest.skipIf(os.name == "nt", "POSIX shell quoting regression")
    def test_literal_quotes_in_filename_do_not_select_another_file(self):
        literal = self.root / "'quoted.py'"
        literal.write_text("expected_literal_file\n", encoding="utf-8")
        other = self.root / "quoted.py"
        other.write_text("def UNINTENDED_TARGET_MARKER(): pass\n" + "# filler\n" * 180, encoding="utf-8")
        result = self.hook("Bash", {"command": "cat " + shlex.quote(str(literal))})
        self.assertEqual((result.returncode, result.stderr), (0, ""))

    def test_shell_comment_is_not_a_requested_filename(self):
        other = self.root / "#unread.py"
        other.write_text("def COMMENT_TARGET_MARKER(): pass\n" + "# filler\n" * 180, encoding="utf-8")
        result = self.hook("Bash", {"command": "cat #unread.py"})
        self.assertEqual((result.returncode, result.stderr), (0, ""))
        quoted = self.hook("Bash", {"command": 'cat "#unread.py"'})
        self.assertEqual(quoted.returncode, 2)
        self.assertIn("COMMENT_TARGET_MARKER", quoted.stderr)

    def test_scoped_dependency_reads_allowed_and_broad_cache_scan_guided(self):
        for tool, inputs in [("Glob", {"path": ".codex/plugins/cache", "pattern": "**/*"}),
                             ("Grep", {"path": "node_modules", "pattern": ".*"})]:
            self.assertEqual(self.hook(tool, inputs).returncode, 2)
        for tool, inputs in [("Glob", {"path": ".codex/plugins/cache", "pattern": "*/SKILL.md"}),
                             ("Grep", {"path": "node_modules", "pattern": ".*", "head_limit": 20}),
                             ("Grep", {"path": "node_modules", "pattern": "specific_contract"})]:
            self.assertEqual(self.hook(tool, inputs).returncode, 0)

    def test_outside_source_and_unknown_shell_are_not_disclosed(self):
        outside = self.hook("Read", {"file_path": str(CLI)})
        self.assertEqual((outside.returncode, outside.stderr), (0, ""))
        compound = self.hook("Bash", {"command": f'echo token-guard; cat "{self.file}"'})
        self.assertEqual((compound.returncode, compound.stderr), (0, ""))


if __name__ == "__main__":
    unittest.main()
