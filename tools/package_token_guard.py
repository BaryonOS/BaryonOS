# SPDX-License-Identifier: MPL-2.0
"""Validate and deterministically package the tracked Token Guard components."""
from __future__ import annotations

import argparse
import ast
import hashlib
import json
from pathlib import Path
import tempfile
import zipfile


ROOT = Path(__file__).resolve().parents[1]
PLUGIN = ROOT / "plugins/token-guard"
SCRIPT = "skills/token-guard/scripts/tokenguard.py"
FILES = (
    ".claude-plugin/plugin.json", ".codex-plugin/plugin.json", ".mcp.json",
    "LICENSE", "README.md", "config/claude-mcp.json", "hooks/hooks.json", "mcp.json", "plugin.json",
    "skills/token-guard/SKILL.md", "skills/token-guard/agents/openai.yaml",
    "skills/token-guard/references/quality-contract.md", SCRIPT,
    "skills/token-guard/scripts/tokenguard_hook.py",
    "skills/token-guard/scripts/tokenguard_init.py",
    "skills/token-guard/scripts/tokenguard_mcp.py",
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def package(output: Path) -> dict:
    contents = {}
    for name in sorted(FILES):
        path = PLUGIN / name
        require(path.is_file() and not path.is_symlink(), f"missing or linked component: {name}")
        require(path.resolve().is_relative_to(PLUGIN.resolve()), f"outside plugin: {name}")
        data = path.read_bytes()
        contents[name] = data
        if path.suffix == ".json":
            json.loads(data)
        elif path.suffix == ".py":
            ast.parse(data.decode("utf-8"), filename=name)
    root, claude, codex = (json.loads(contents[name]) for name in (
        "plugin.json", ".claude-plugin/plugin.json", ".codex-plugin/plugin.json"))
    for key in ("name", "version", "description"):
        require(root[key] == claude[key] == codex[key], f"manifest mismatch: {key}")
    require(root["name"] == "token-guard", "unexpected plugin identity")
    require(root["$schema"] == "https://agent-plugins.org/schemas/1.0.0/plugin.schema.json", "portable schema missing")
    require(not {"skills", "hooks", "mcpServers", "apps", "interface"} & root.keys(), "native fields at portable root")
    require(not {"skills", "hooks"} & claude.keys(), "duplicate default Claude discovery")
    require(claude.get("mcpServers") == "./config/claude-mcp.json", "dedicated Claude MCP override missing")
    require(codex.get("mcpServers") == "./.mcp.json", "Codex compatibility MCP path missing")
    require(root["extensions"]["com.openai"]["hooks"] == codex["hooks"] == {"hooks": {}}, "Claude hooks must not leak into OpenAI")
    require(len(root["extensions"]["com.openai"]["interface"]["shortDescription"]) <= 30, "subtitle exceeds 30 characters")
    for name, variable in (("mcp.json", "${PLUGIN_ROOT}"), (".mcp.json", "${PLUGIN_ROOT}"),
                           ("config/claude-mcp.json", "${CLAUDE_PLUGIN_ROOT}")):
        server = json.loads(contents[name])["mcpServers"]["token-guard"]
        require(server["args"][1] == variable + "/" + SCRIPT and server["args"][-1] == "mcp", "MCP entrypoint mismatch")
    require(json.loads(contents["mcp.json"])["$schema"] == "https://agent-plugins.org/schemas/1.0.0/mcp.schema.json", "MCP schema missing")
    hook = json.loads(contents["hooks/hooks.json"])["hooks"]["PreToolUse"][0]
    require(hook["matcher"] == "Bash|Read|Grep|Glob", "hook matcher mismatch")
    runtime = ast.parse(contents["skills/token-guard/scripts/tokenguard_mcp.py"].decode())
    versions = [node.value.value for node in runtime.body if isinstance(node, ast.Assign)
                and any(isinstance(target, ast.Name) and target.id == "VERSION" for target in node.targets)
                and isinstance(node.value, ast.Constant)]
    require(versions == [root["version"]], "MCP runtime/manifest version mismatch")
    output = output.absolute()
    require(output.suffix == ".zip" and not output.resolve().is_relative_to(PLUGIN.resolve()), "output must be a ZIP outside the plugin source")
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=output.parent, suffix=".zip", delete=False) as staged:
        staged_path = Path(staged.name)
    try:
        with zipfile.ZipFile(staged_path, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
            for name, data in contents.items():
                info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
                info.create_system = 3
                info.external_attr = (0o100755 if name == SCRIPT else 0o100644) << 16
                info.compress_type = zipfile.ZIP_DEFLATED
                archive.writestr(info, data, compresslevel=9)
        with zipfile.ZipFile(staged_path) as archive:
            require(archive.testzip() is None, "archive CRC failure")
            require(set(archive.namelist()) == set(FILES), "archive inventory mismatch")
            require(all(archive.read(name) == data for name, data in contents.items()), "archive content mismatch")
        staged_path.replace(output)
    finally:
        staged_path.unlink(missing_ok=True)
    record = {"version": root["version"], "archive": str(output), "bytes": output.stat().st_size,
              "sha256": hashlib.sha256(output.read_bytes()).hexdigest(),
              "files": {name: {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data),
                                "mode": "0755" if name == SCRIPT else "0644"}
                        for name, data in contents.items()}}
    output.with_suffix(".manifest.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    return record


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "build/plugin.zip")
    args = parser.parse_args()
    result = package(args.output)
    print(json.dumps({key: value for key, value in result.items() if key != "files"}, sort_keys=True))
