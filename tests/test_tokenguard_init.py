# SPDX-License-Identifier: MPL-2.0
"""Host tests for preserving workspace configuration during explicit MCP setup."""

from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile
import unittest
import unittest.mock as mock


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "plugins/token-guard/skills/token-guard/scripts/tokenguard_init.py"
SPEC = importlib.util.spec_from_file_location("tokenguard_init_tests_module", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
tg = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = tg
SPEC.loader.exec_module(tg)


class InitWorkspaceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.base = ROOT / "build/test-tokenguard-init"
        cls.base.mkdir(parents=True, exist_ok=True)

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="case-", dir=self.base)
        self.addCleanup(self.temporary.cleanup)
        self.case = Path(self.temporary.name)
        self.workspace = self.case / "workspace with spaces"
        self.workspace.mkdir()

    def invoke(self, clients=None, apply=False):
        return json.loads(tg.init_workspace(self.workspace, clients or ["all"], apply=apply))

    def write(self, name, content):
        path = self.workspace / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content.encode() if isinstance(content, str) else content)
        return path

    def assert_no_temporary_files(self):
        self.assertEqual(list(self.workspace.rglob("*.tmp")), [])

    def test_cli_preview_and_conflict_exit_status(self):
        command = [sys.executable, "-B", str(ROOT / "tools/tokenguard.py"), "--root", str(self.workspace), "init"]
        preview = subprocess.run(command, stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=10)
        self.assertEqual(preview.returncode, 0, preview.stderr)
        self.assertTrue(json.loads(preview.stdout)["ok"])
        self.assertEqual(list(self.workspace.iterdir()), [])
        existing = b'{"mcpServers":{"token-guard":{"command":"another-server"},"graphify":{"url":"https://example.invalid"}}}'
        config = self.workspace / ".mcp.json"
        config.write_bytes(existing)
        conflict = subprocess.run([*command, "--clients", "claude", "vscode", "--apply"], stdin=subprocess.DEVNULL,
                                  capture_output=True, text=True, timeout=10)
        self.assertEqual(conflict.returncode, 2, conflict.stdout)
        self.assertFalse(json.loads(conflict.stdout)["ok"])
        self.assertEqual(config.read_bytes(), existing)
        self.assertFalse((self.workspace / ".vscode").exists())

    @unittest.skipIf(os.name == "nt", "symlink creation privileges vary on Windows")
    def test_cli_does_not_resolve_away_symlink_root_validation(self):
        link = self.case / "link"
        link.symlink_to(self.workspace, target_is_directory=True)
        result = subprocess.run([sys.executable, "-B", str(ROOT / "tools/tokenguard.py"), "--root", str(link),
                                 "init", "--clients", "claude", "--apply"], stdin=subprocess.DEVNULL,
                                capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 2)
        self.assertFalse(json.loads(result.stdout)["ok"])
        self.assertEqual(list(self.workspace.iterdir()), [])

    def test_preview_is_concrete_and_has_no_filesystem_side_effects(self):
        result = self.invoke()
        self.assertTrue(result["ok"])
        self.assertEqual(result["status"], "planned")
        self.assertEqual(list(self.workspace.iterdir()), [])
        self.assertEqual({change["path"] for change in result["changes"]}, {
            ".mcp.json", ".codex/config.toml", ".vscode/mcp.json", "opencode.json",
        })
        shared = next(change for change in result["changes"] if change["path"] == ".mcp.json")
        self.assertEqual(shared["clients"], ["claude", "copilot"])
        self.assertEqual(shared["server"]["args"], [
            "-B", str(SCRIPT.with_name("tokenguard.py")), "--root", str(self.workspace), "mcp",
        ])
        self.assertIn("stable extracted directory", " ".join(result["notes"]))

    def test_apply_uses_native_shapes_and_second_apply_is_idempotent(self):
        result = self.invoke(apply=True)
        self.assertTrue(result["ok"], result)
        shared = json.loads((self.workspace / ".mcp.json").read_text())
        vscode = json.loads((self.workspace / ".vscode/mcp.json").read_text())
        opencode = json.loads((self.workspace / "opencode.json").read_text())
        codex = tg.tomllib.loads((self.workspace / ".codex/config.toml").read_text())
        server = shared["mcpServers"]["token-guard"]
        self.assertEqual(set(server), {"type", "command", "args"})
        self.assertEqual(server, vscode["servers"]["token-guard"])
        self.assertEqual(server["command"], str(Path(sys.executable).absolute()))
        self.assertEqual(opencode["mcp"]["servers"]["token-guard"]["command"], [server["command"], *server["args"]])
        self.assertEqual(opencode["mcp"]["servers"]["token-guard"]["timeout"], {"execution": 3660000})
        self.assertEqual(codex["mcp_servers"]["token-guard"]["tool_timeout_sec"], 3660)
        before = {str(path.relative_to(self.workspace)): path.read_bytes()
                  for path in self.workspace.rglob("*") if path.is_file()}
        again = self.invoke(apply=True)
        self.assertTrue(again["ok"], again)
        self.assertEqual(again["status"], "unchanged")
        self.assertEqual(again["backups"], [])
        after = {str(path.relative_to(self.workspace)): path.read_bytes()
                 for path in self.workspace.rglob("*") if path.is_file()}
        self.assertEqual(before, after)
        self.assertFalse((self.workspace / ".claude/settings.json").exists())
        self.assert_no_temporary_files()

    def test_jsonc_comments_strings_trailing_commas_and_crlf_survive(self):
        original = (
            '{\r\n  // Keep this comment and formatting.\r\n  "servers": {\r\n'
            '    "graphify": {"type":"http", "url":"https://example.invalid/a/*literal*/",},\r\n'
            '    /* Keep the existing server. */\r\n  },\r\n'
            '  "inputs": [{"id":"keep", "description":"} // not a comment",}],\r\n}\r\n'
        )
        path = self.write(".vscode/mcp.json", original)
        result = self.invoke(["vscode"], apply=True)
        self.assertTrue(result["ok"], result)
        actual = path.read_bytes().decode()
        self.assertIn('"graphify": {"type":"http", "url":"https://example.invalid/a/*literal*/",}', actual)
        self.assertIn('"inputs": [{"id":"keep", "description":"} // not a comment",}],', actual)
        self.assertIn("/* Keep the existing server. */", actual)
        self.assertNotIn("\n", actual.replace("\r\n", ""))
        self.assertEqual(len(result["backups"]), 1)
        self.assertEqual((self.workspace / result["backups"][0]).read_bytes(), original.encode())

    def test_insert_after_line_comment_keeps_existing_json_value(self):
        self.write(".mcp.json", '{"mcpServers":{"graphify":{"command":"graphify"} // preserve\n}}\n')
        result = self.invoke(["claude"], apply=True)
        self.assertTrue(result["ok"], result)
        text = (self.workspace / ".mcp.json").read_text()
        self.assertIn('{"command":"graphify"}, // preserve', text)
        parsed, _, _ = tg._json_document(text)
        self.assertEqual(parsed["mcpServers"]["graphify"], {"command": "graphify"})

    def test_missing_nested_objects_do_not_replace_unrelated_keys(self):
        self.write("opencode.jsonc", '{\n // project\n "model": "existing", "mcp": {"timeout":{"catalog":1234}},\n}\n')
        result = self.invoke(["opencode"], apply=True)
        self.assertTrue(result["ok"], result)
        text = (self.workspace / "opencode.jsonc").read_text()
        self.assertIn('"timeout":{"catalog":1234}', text)
        parsed, _, _ = tg._json_document(text)
        self.assertEqual(parsed["model"], "existing")
        self.assertEqual(parsed["mcp"]["timeout"], {"catalog": 1234})
        self.assertIn("token-guard", parsed["mcp"]["servers"])
        self.assertFalse((self.workspace / "opencode.json").exists())

    def test_identical_owned_launch_preserves_extra_user_policy(self):
        self.invoke(["claude"], apply=True)
        path = self.workspace / ".mcp.json"
        value = json.loads(path.read_text())
        value["mcpServers"]["token-guard"]["env"] = {"USER_CHOICE": "preserve"}
        path.write_text(json.dumps(value))
        original = path.read_bytes()
        result = self.invoke(["claude", "copilot"], apply=True)
        self.assertTrue(result["ok"], result)
        self.assertEqual(result["status"], "unchanged")
        self.assertEqual(path.read_bytes(), original)

    def test_conflicting_entry_prevents_every_target_write(self):
        path = self.write(".mcp.json", '{"mcpServers":{"token-guard":{"command":"user-owned"}}}')
        original = path.read_bytes()
        result = self.invoke(apply=True)
        self.assertFalse(result["ok"])
        self.assertIn("differs", " ".join(result["errors"]))
        self.assertEqual(path.read_bytes(), original)
        self.assertEqual(list(self.workspace.iterdir()), [path])

    def test_invalid_json_and_ambiguous_parents_are_never_rewritten(self):
        invalid = [
            '{"mcpServers":{},"mcpServers":{}}',
            '{"mcpServers":[]}', '{"mcpServers":null}',
            '{/* unclosed', '{"other": NaN}', '{"other": Infinity}',
            '{"other":1,,}', '{"other": "bad\\x"}',
        ]
        for original in invalid:
            with self.subTest(original=original):
                path = self.write(".mcp.json", original)
                result = self.invoke(["vscode", "claude"], apply=True)
                self.assertFalse(result["ok"], result)
                self.assertEqual(path.read_text(), original)
                self.assertFalse((self.workspace / ".vscode").exists())

    def test_existing_toml_comments_models_and_graphify_remain_identical(self):
        original = '# keep model\r\nmodel = "existing"\r\n\r\n[mcp_servers.graphify]\r\nurl = "https://example.invalid/mcp"\r\n'
        path = self.write(".codex/config.toml", original)
        result = self.invoke(["codex"], apply=True)
        self.assertTrue(result["ok"], result)
        updated = path.read_bytes()
        self.assertTrue(updated.startswith(original.encode()))
        value = tg.tomllib.loads(updated.decode())
        self.assertEqual(value["model"], "existing")
        self.assertEqual(value["mcp_servers"]["graphify"], {"url": "https://example.invalid/mcp"})
        self.assertEqual((self.workspace / result["backups"][0]).read_bytes(), original.encode())

    def test_inline_toml_table_is_refused_instead_of_corrupted(self):
        original = 'mcp_servers = { graphify = { url = "https://example.invalid" } }\n'
        path = self.write(".codex/config.toml", original)
        result = self.invoke(["claude", "codex"], apply=True)
        self.assertFalse(result["ok"])
        self.assertIn("inline", " ".join(result["errors"]))
        self.assertEqual(path.read_text(), original)
        self.assertFalse((self.workspace / ".mcp.json").exists())

    def test_python_310_has_an_explicit_codex_limitation(self):
        with mock.patch.object(tg, "tomllib", None):
            result = self.invoke(["claude", "codex"], apply=True)
            self.assertFalse(result["ok"])
            self.assertIn("Python 3.11", " ".join(result["errors"]))
            self.assertEqual(list(self.workspace.iterdir()), [])
            self.assertTrue(self.invoke(["claude"], apply=True)["ok"])

    def test_existing_hidden_opencode_config_is_used(self):
        path = self.write(".opencode/opencode.jsonc", '{"model":"keep"}\n')
        result = self.invoke(["opencode"], apply=True)
        self.assertTrue(result["ok"], result)
        self.assertEqual(result["changes"][0]["path"], ".opencode/opencode.jsonc")
        self.assertFalse((self.workspace / "opencode.json").exists())
        self.assertIn("token-guard", path.read_text())

    def test_ambiguous_opencode_files_require_manual_precedence_resolution(self):
        self.write("opencode.json", "{}")
        self.write(".opencode/opencode.jsonc", "{}")
        result = self.invoke(["claude", "opencode"], apply=True)
        self.assertFalse(result["ok"])
        self.assertIn("Multiple OpenCode", " ".join(result["errors"]))
        self.assertFalse((self.workspace / ".mcp.json").exists())

    @unittest.skipIf(os.name == "nt", "Symlink creation can require Windows privileges")
    def test_symlink_file_and_directory_cannot_escape_workspace(self):
        external = self.case / "outside"
        external.mkdir()
        external_file = external / "settings.json"
        external_file.write_text("untouched")
        link = self.workspace / ".mcp.json"
        link.symlink_to(external_file)
        result = self.invoke(["claude"], apply=True)
        self.assertFalse(result["ok"])
        self.assertEqual(external_file.read_text(), "untouched")
        link.unlink()
        (self.workspace / ".codex").symlink_to(external, target_is_directory=True)
        result = self.invoke(["claude", "codex"], apply=True)
        self.assertFalse(result["ok"])
        self.assertFalse((external / "config.toml").exists())
        self.assertFalse((self.workspace / ".mcp.json").exists())

    @unittest.skipIf(os.name == "nt", "Symlink creation can require Windows privileges")
    def test_symlink_workspace_root_is_refused(self):
        alias = self.case / "alias"
        alias.symlink_to(self.workspace, target_is_directory=True)
        result = json.loads(tg.init_workspace(alias, ["claude"], apply=True))
        self.assertFalse(result["ok"])
        self.assertEqual(list(self.workspace.iterdir()), [])

    def test_replace_failure_rolls_back_bytes_modes_and_new_directories(self):
        original = '{"mcpServers":{"graphify":{"command":"keep"}}}\n'
        path = self.write(".mcp.json", original)
        path.chmod(0o640)
        with mock.patch.object(tg, "_publish_new", side_effect=OSError("injected write failure")):
            result = self.invoke(["claude", "codex"], apply=True)
        self.assertFalse(result["ok"])
        self.assertEqual(result["status"], "rolled_back")
        self.assertEqual(path.read_text(), original)
        self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o640)
        self.assertFalse((self.workspace / ".codex").exists())
        self.assertEqual((self.workspace / result["backups"][0]).read_text(), original)
        self.assert_no_temporary_files()

    def test_rollback_removes_new_configuration_instead_of_leaving_partial_setup(self):
        publish = tg._publish_new
        calls = 0

        def fail_second(source, destination):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise OSError("injected write failure")
            publish(source, destination)

        with mock.patch.object(tg, "_publish_new", side_effect=fail_second):
            result = self.invoke(["claude", "vscode"], apply=True)
        self.assertEqual(result["status"], "rolled_back")
        self.assertEqual(list(self.workspace.iterdir()), [])

    def test_failed_rollback_reports_partial_failure_and_keeps_backup(self):
        original = '{"mcpServers":{"graphify":{"command":"keep"}}}'
        path = self.write(".mcp.json", original)
        replace = tg._replace
        calls = 0

        def fail_after_first(source, destination):
            nonlocal calls
            calls += 1
            if calls > 1:
                raise OSError("injected persistent failure")
            replace(source, destination)

        with mock.patch.object(tg, "_replace", side_effect=fail_after_first), \
                mock.patch.object(tg, "_publish_new", side_effect=OSError("injected publication failure")):
            result = self.invoke(["claude", "codex"], apply=True)
        self.assertFalse(result["ok"])
        self.assertEqual(result["status"], "partial_failure")
        self.assertTrue(result["rollback_errors"])
        self.assertIn("token-guard", path.read_text())
        self.assertEqual((self.workspace / result["backups"][0]).read_text(), original)
        self.assert_no_temporary_files()

    def test_concurrent_change_before_commit_is_not_overwritten(self):
        path = self.write(".codex/config.toml", 'model = "original"\n')
        stage = tg._stage
        changed = False

        def edit_during_staging(*args, **kwargs):
            nonlocal changed
            staged = stage(*args, **kwargs)
            if not changed:
                changed = True
                path.write_text('model = "user changed"\n')
            return staged

        with mock.patch.object(tg, "_stage", side_effect=edit_during_staging):
            result = self.invoke(["claude", "codex"], apply=True)
        self.assertFalse(result["ok"])
        self.assertEqual(path.read_text(), 'model = "user changed"\n')
        self.assertFalse((self.workspace / ".mcp.json").exists())
        self.assert_no_temporary_files()

    def test_invalid_clients_nonfiles_and_oversized_configs_are_refused(self):
        result = json.loads(tg.init_workspace(self.workspace, [], apply=True))
        self.assertFalse(result["ok"])
        self.assertFalse(self.invoke(["unknown"], apply=True)["ok"])
        (self.workspace / ".mcp.json").mkdir()
        self.assertFalse(self.invoke(["claude"], apply=True)["ok"])
        (self.workspace / ".mcp.json").rmdir()
        self.write(".mcp.json", " " * (tg.MAX_CONFIG_BYTES + 1))
        self.assertFalse(self.invoke(["claude"], apply=True)["ok"])

    def test_literal_paths_cannot_accidentally_expand_client_variables(self):
        self.workspace = self.case / "workspace ${TOKEN_GUARD_ROOT}"
        self.workspace.mkdir()
        result = self.invoke(["claude", "codex"], apply=True)
        self.assertFalse(result["ok"])
        self.assertIn("substitution syntax", " ".join(result["errors"]))
        self.assertEqual(list(self.workspace.iterdir()), [])

    def test_interrupt_after_successful_replace_restores_original(self):
        original = '{"mcpServers":{"graphify":{"command":"keep"}}}\n'
        path = self.write(".mcp.json", original)
        replace = tg._replace
        calls = 0

        def interrupt_after_first_replace(source, destination):
            nonlocal calls
            calls += 1
            replace(source, destination)
            if calls == 1:
                raise KeyboardInterrupt()

        with mock.patch.object(tg, "_replace", side_effect=interrupt_after_first_replace):
            result = self.invoke(["claude"], apply=True)
        self.assertFalse(result["ok"])
        self.assertEqual(result["status"], "rolled_back")
        self.assertEqual(path.read_text(), original)
        self.assertEqual((self.workspace / result["backups"][0]).read_text(), original)
        self.assert_no_temporary_files()

    def test_interrupt_after_new_file_publication_removes_owned_file(self):
        publish = tg._publish_new

        def interrupt_after_publish(source, destination):
            publish(source, destination)
            raise KeyboardInterrupt()

        with mock.patch.object(tg, "_publish_new", side_effect=interrupt_after_publish):
            result = self.invoke(["claude"], apply=True)
        self.assertFalse(result["ok"])
        self.assertEqual(result["status"], "rolled_back")
        self.assertEqual(list(self.workspace.iterdir()), [])

    def test_concurrent_creation_at_publication_is_never_overwritten(self):
        publish = tg._publish_new
        concurrent = b'{"mcpServers":{"user-owned":{"command":"preserve"}}}\n'

        def create_before_publish(source, destination):
            destination.write_bytes(concurrent)
            publish(source, destination)

        with mock.patch.object(tg, "_publish_new", side_effect=create_before_publish):
            result = self.invoke(["claude"], apply=True)
        self.assertFalse(result["ok"])
        self.assertEqual(result["status"], "partial_failure")
        self.assertEqual((self.workspace / ".mcp.json").read_bytes(), concurrent)
        self.assertEqual(result["backups"], [])
        self.assert_no_temporary_files()

    def test_equal_concurrent_file_is_not_mistaken_for_owned_publication(self):
        publish = tg._publish_new

        def create_before_publish(source, destination):
            destination.write_bytes(source.read_bytes())
            publish(source, destination)

        with mock.patch.object(tg, "_publish_new", side_effect=create_before_publish):
            result = self.invoke(["claude"], apply=True)
        self.assertFalse(result["ok"])
        self.assertEqual(result["status"], "partial_failure")
        self.assertTrue((self.workspace / ".mcp.json").is_file())
        self.assert_no_temporary_files()

    def test_divergence_after_replace_is_reported_without_overwriting_user_change(self):
        self.write(".mcp.json", '{"mcpServers":{"graphify":{"command":"keep"}}}')
        concurrent = '{"mcpServers":{"user-owned":{"command":"preserve"}}}'
        replace = tg._replace

        def change_after_replace(source, destination):
            replace(source, destination)
            destination.write_text(concurrent)
            raise KeyboardInterrupt()

        with mock.patch.object(tg, "_replace", side_effect=change_after_replace):
            result = self.invoke(["claude"], apply=True)
        self.assertFalse(result["ok"])
        self.assertEqual(result["status"], "partial_failure")
        self.assertEqual((self.workspace / ".mcp.json").read_text(), concurrent)
        self.assertTrue(result["rollback_errors"])
        self.assert_no_temporary_files()

    def test_legacy_opencode_server_shape_is_refused_without_any_writes(self):
        for legacy in (
            {"type": "remote", "url": "https://example.invalid/mcp", "enabled": True},
            {"type": "local", "command": ["graphify"], "enabled": True},
        ):
            with self.subTest(legacy=legacy["type"]):
                original = json.dumps({"mcp": {"graphify": legacy}})
                path = self.write("opencode.json", original)
                result = self.invoke(["claude", "opencode"], apply=True)
                self.assertFalse(result["ok"])
                self.assertIn("OpenCode V1", " ".join(result["errors"]))
                self.assertEqual(path.read_text(), original)
                self.assertEqual(list(self.workspace.iterdir()), [path])

    def test_modern_opencode_servers_and_documented_timeout_are_preserved(self):
        graphify = {"type": "remote", "url": "https://example.invalid/mcp"}
        timeout = {"startup": 45000, "catalog": 30000, "execution": 600000}
        self.write("opencode.json", json.dumps({"mcp": {
            "servers": {"graphify": graphify}, "timeout": timeout,
        }}))
        result = self.invoke(["opencode"], apply=True)
        self.assertTrue(result["ok"], result)
        parsed = json.loads((self.workspace / "opencode.json").read_text())
        self.assertEqual(parsed["mcp"]["servers"]["graphify"], graphify)
        self.assertEqual(parsed["mcp"]["timeout"], timeout)


if __name__ == "__main__":
    unittest.main()
