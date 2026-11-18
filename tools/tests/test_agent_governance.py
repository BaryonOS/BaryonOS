"""Shared role adapters resolve, retain tool limits and cannot edit their policy."""
from __future__ import annotations

import json
import re
import sys
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
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


if __name__ == "__main__":
    unittest.main()
