"""Shared role adapters resolve, retain tool limits and cannot edit their policy."""
from __future__ import annotations

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
        paths = ["tools/tests/test_agent_governance.py"]
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
                claude = (ROOT / f".claude/agents/{slug}.md").read_text()
                self.assertEqual(json.loads(re.search(r"^description: (.+)$", claude, re.M)[1]),
                                 description)
                copilot = ROOT / f".github/agents/{slug}.agent.md"
                self.assertEqual(copilot.is_file(), tool_class == "builder")
                if copilot.is_file():
                    tools = json.loads(re.search(r"^tools: (.+)$", copilot.read_text(), re.M)[1])
                    self.assertEqual(tools[:4], ["read", "search", "edit", "execute"])
                    self.assertTrue(all(t.startswith("graphify/") for t in tools[4:]))
                    self.assertNotIn("graphify/remember", tools)
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


if __name__ == "__main__":
    unittest.main()
