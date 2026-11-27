"""Shared role adapters resolve, retain tool limits and cannot edit their policy."""
from __future__ import annotations

import contextlib
import io
import json
import re
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import agent_adapters
import agent_pipeline
import roadmap_runner


ROLES = ("os_architect", "os_builder", "os_verifier", "os_security_reviewer")


class AgentGovernanceTests(unittest.TestCase):
    def test_role_adapters_resolve_the_same_canonical_contract(self):
        for role in ROLES:
            contract = f".agents/roles/{role}.md"
            slug = role.replace("_", "-")
            with self.subTest(role=role):
                self.assertTrue((ROOT / contract).is_file())
                codex = tomllib.loads((ROOT / f".codex/agents/{role}.toml").read_text())
                self.assertEqual(codex["name"], role)
                self.assertIn(contract, codex["developer_instructions"])
                for client, path in (
                    ("claude", f".claude/agents/{slug}.md"),
                    ("copilot", f".github/agents/{slug}.agent.md"),
                ):
                    with self.subTest(client=client):
                        adapter = (ROOT / path).read_text()
                        self.assertIn(f"`{contract}`", adapter)

    def test_copilot_tools_retain_existing_restrictions(self):
        for role in ROLES:
            path = ROOT / f".github/agents/{role.replace('_', '-')}.agent.md"
            tools = json.loads(re.search(r"^tools: (.+)$", path.read_text(), re.M)[1])
            expected = ["read", "search", "edit", "execute"] if role == "os_builder" else ["read", "search"]
            with self.subTest(role=role):
                self.assertEqual(tools, expected)

    def test_shared_skill_adapters_resolve(self):
        contract = ".agents/skills/development-loop/SKILL.md"
        self.assertTrue((ROOT / contract).is_file())
        for client in (".claude", ".github"):
            adapter = ROOT / client / "skills/development-loop/SKILL.md"
            with self.subTest(client=client):
                self.assertIn(f"`{contract}`", adapter.read_text())

    def test_workers_cannot_authorize_edits_to_shared_policy(self):
        paths = ["tools/tests/test_agent_governance.py", "tools/agent_adapters.py"]
        for role in ROLES:
            slug = role.replace("_", "-")
            paths.extend((f".agents/roles/{role}.md", f".codex/agents/{role}.toml",
                          f".claude/agents/{slug}.md", f".github/agents/{slug}.agent.md"))
        paths.extend((".agents/skills/development-loop/SKILL.md",
                      ".claude/skills/development-loop/SKILL.md"))
        for path in paths:
            with self.subTest(path=path):
                self.assertTrue(agent_pipeline.is_protected(path))
                self.assertTrue(roadmap_runner.protected(path))
                self.assertEqual(agent_pipeline.out_of_scope([path], [path]), [path])
                spec = "```json\n" + json.dumps({"targets": [path], "reads": []}) + "\n```"
                with self.assertRaises(ValueError):
                    agent_pipeline.parse_spec(spec)

    def test_specialist_adapters_are_generated_without_drift(self):
        contracts = sorted((ROOT / ".agents/roles").glob("specialist_*.md"))
        self.assertTrue(contracts)
        self.assertEqual(agent_adapters.drift(ROOT), [])
        for contract in contracts:
            role, description, tool_class = agent_adapters.parse(contract)
            slug = role.replace("_", "-")
            with self.subTest(role=role):
                codex = tomllib.loads((ROOT / f".codex/agents/{role}.toml").read_text())
                self.assertEqual((codex["name"], codex["description"]), (role, description))
                # No tool, sandbox or approval keys: the session's existing limits apply unchanged.
                self.assertEqual(set(codex), {"name", "description", "developer_instructions"})
                claude = (ROOT / f".claude/agents/{slug}.md").read_text()
                self.assertEqual(re.findall(r"^(\w[\w-]*):", claude.split("---")[1], re.M),
                                 ["name", "description"])
                self.assertEqual(json.loads(re.search(r"^description: (.+)$", claude, re.M)[1]),
                                 description)
                copilot = ROOT / f".github/agents/{slug}.agent.md"
                self.assertEqual(copilot.is_file(), tool_class == "builder")
                if copilot.is_file():
                    tools = json.loads(re.search(r"^tools: (.+)$", copilot.read_text(), re.M)[1])
                    # Exact pin: a widened list (e.g. `graphify/*`) must fail even after a regenerate.
                    self.assertEqual(tools, ["read", "search", "edit", "execute"] + [
                        f"graphify/{t}" for t in (
                            "recall", "graphify_rank_files", "graphify_find", "graphify_callers",
                            "graphify_impact", "graphify_tests_for", "graphify_file_neighbors",
                            "query_graph")])
                for path in (f".codex/agents/{role}.toml", f".claude/agents/{slug}.md",
                             f".github/agents/{slug}.agent.md"):
                    self.assertTrue(agent_pipeline.is_protected(path))

    def test_specialist_tool_class_is_fail_closed(self):
        body = "# specialist_x\n\nOwns x. More text.\n\n## Tools\n"
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / ".agents/roles").mkdir(parents=True)
            role = root / ".agents/roles/specialist_x.md"
            for text in (body, body + "Tool class: `read-only`\n", body + "Tool class: `admin`\n",
                         body + "Tool class: builder\n",
                         body + "Tool class: `builder`\nTool class: `coordinator`\n",
                         body + "Tool class: `builder`\n**Tool class:** coordinator\n",
                         body + "Tool class: `builder`\ntool-class: `admin`\n",
                         "# specialist_y\n\nOwns x.\nTool class: `builder`\n"):
                role.write_text(text)
                with self.subTest(text=text), self.assertRaises(ValueError):
                    agent_adapters.expected(root)
            role.write_text(body + "Tool class: `coordinator`\n")
            self.assertEqual(sorted(agent_adapters.expected(root)),
                             [".claude/agents/specialist-x.md", ".codex/agents/specialist_x.toml"])
            orphan = root / ".github/agents/specialist-x.agent.md"
            orphan.parent.mkdir(parents=True)
            orphan.write_text("hand-written\n")
            self.assertIn("orphan: .github/agents/specialist-x.agent.md", agent_adapters.drift(root))

    def test_specialist_role_names_are_strict(self):
        with tempfile.TemporaryDirectory() as tmp:
            roles = Path(tmp) / ".agents/roles"
            roles.mkdir(parents=True)
            for role in ("specialist_X", "specialist_x-y", "specialist_", "specialist_x y"):
                path = roles / f"{role}.md"
                path.write_text(f"# {role}\n\nOwns x.\n\nTool class: `builder`\n")
                with self.subTest(role=role), self.assertRaisesRegex(ValueError, "role name"):
                    agent_adapters.expected(Path(tmp))
                path.unlink()

    def test_adapter_cli_checks_writes_and_removes_orphans(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / ".agents/roles").mkdir(parents=True)
            contract = root / ".agents/roles/specialist_x.md"
            contract.write_text("# specialist_x\n\nOwns x.\n\nTool class: `builder`\n")
            orphan = root / ".codex/agents/specialist_gone.toml"
            orphan.parent.mkdir(parents=True)
            orphan.write_text("hand-written\n")
            run = lambda *argv: agent_adapters.main(list(argv), root=root)
            with contextlib.redirect_stderr(io.StringIO()) as err:
                self.assertEqual(run(), 1)  # --check is the default and never writes
                self.assertEqual(run("--check"), 1)
            self.assertIn("stale: .github/agents/specialist-x.agent.md", err.getvalue())
            self.assertIn("orphan: .codex/agents/specialist_gone.toml", err.getvalue())
            self.assertFalse((root / ".claude/agents/specialist-x.md").exists())
            self.assertEqual(run("--write"), 0)
            self.assertFalse(orphan.exists())
            self.assertEqual(sorted(p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_file()),
                             sorted([".agents/roles/specialist_x.md", *agent_adapters.expected(root)]))
            self.assertEqual(run("--check"), 0)
            (root / ".claude/agents/specialist-x.md").write_text("edited by hand\n")
            with contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(run("--check"), 1)
                contract.write_text("# specialist_x\n\nOwns x.\n\nTool class: `admin`\n")
                self.assertEqual(run("--write"), 2)  # a contract error writes nothing
            self.assertEqual((root / ".claude/agents/specialist-x.md").read_text(), "edited by hand\n")


if __name__ == "__main__":
    unittest.main()
