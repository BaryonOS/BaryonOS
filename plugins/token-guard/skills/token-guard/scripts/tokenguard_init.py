"""Preview or merge workspace MCP configuration without changing global profiles."""

from __future__ import annotations

import hashlib
import json
import os
import stat
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

try:
    import tomllib
except ImportError:  # Python 3.10 can initialize the JSON-based clients.
    tomllib = None


CLIENTS = ("claude", "codex", "copilot", "vscode", "opencode")
MAX_CONFIG_BYTES = 2_000_000
SERVER_NAME = "token-guard"


class InitError(ValueError):
    pass


@dataclass
class Change:
    path: Path
    original: bytes | None
    proposed: bytes
    mode: int
    clients: list[str]
    entry: dict[str, Any]


def _guard(root: Path, path: Path) -> None:
    try:
        parts = path.relative_to(root).parts
    except ValueError as exc:
        raise InitError("Configuration path escapes the workspace") from exc
    current = root
    for index, part in enumerate(parts):
        current /= part
        try:
            info = current.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise InitError(f"Refusing symlink or reparse point: {current.relative_to(root)}")
        wanted = stat.S_ISREG if index == len(parts) - 1 else stat.S_ISDIR
        if not wanted(info.st_mode):
            raise InitError(f"Unexpected file type: {current.relative_to(root)}")


def _read(root: Path, path: Path) -> tuple[bytes | None, int]:
    _guard(root, path)
    try:
        info = path.stat()
    except FileNotFoundError:
        return None, 0o600
    if info.st_size > MAX_CONFIG_BYTES:
        raise InitError(f"Configuration exceeds {MAX_CONFIG_BYTES} bytes: {path.relative_to(root)}")
    return path.read_bytes(), stat.S_IMODE(info.st_mode)


def _unique(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise InitError("Duplicate JSON object keys require manual resolution")
        result[key] = value
    return result


def _reject_constant(value: str) -> None:
    raise InitError("Non-finite numbers are not valid configuration JSON")


def _json_document(text: str) -> tuple[dict[str, Any], str, str]:
    """Mask JSONC syntax without changing offsets; reject ambiguous JSON values."""
    masked = list(text)
    decoder = json.JSONDecoder()
    index = 0
    while index < len(text):
        if text[index] == '"':
            try:
                _, index = decoder.raw_decode(text, index)
            except ValueError as exc:
                raise InitError("Invalid JSON string") from exc
            continue
        if text.startswith("//", index):
            end = text.find("\n", index + 2)
            end = len(text) if end < 0 else end
        elif text.startswith("/*", index):
            end = text.find("*/", index + 2)
            if end < 0:
                raise InitError("Unterminated JSONC comment")
            end += 2
        else:
            index += 1
            continue
        for offset in range(index, end):
            if text[offset] not in "\r\n":
                masked[offset] = " "
        index = end
    comments_masked = "".join(masked)
    index = 0
    while index < len(masked):
        if masked[index] == '"':
            _, index = decoder.raw_decode(comments_masked, index)
            continue
        if masked[index] == ",":
            following = index + 1
            while following < len(masked) and masked[following].isspace():
                following += 1
            if following < len(masked) and masked[following] in "}]":
                masked[index] = " "
        index += 1
    clean = "".join(masked)
    try:
        value = json.loads(clean, object_pairs_hook=_unique, parse_constant=_reject_constant)
    except InitError:
        raise
    except (ValueError, RecursionError) as exc:
        raise InitError("Invalid or unsupported JSON/JSONC configuration") from exc
    if not isinstance(value, dict):
        raise InitError("Configuration must be a JSON object")
    return value, clean, comments_masked


def _space(text: str, position: int) -> int:
    while position < len(text) and text[position].isspace():
        position += 1
    return position


def _object_span(clean: str, path: tuple[str, ...]) -> tuple[int, int, int | None]:
    decoder = json.JSONDecoder()
    start = _space(clean, 0)
    for wanted in path:
        position = _space(clean, start + 1)
        while clean[position] != "}":
            key, end = decoder.raw_decode(clean, position)
            value_start = _space(clean, _space(clean, end) + 1)
            _, value_end = decoder.raw_decode(clean, value_start)
            if key == wanted:
                start = value_start
                break
            position = _space(clean, value_end)
            if clean[position] == ",":
                position = _space(clean, position + 1)
        else:
            raise InitError("Configuration object could not be located")
    if clean[start] != "{":
        raise InitError("Configuration parent must be an object")
    position = _space(clean, start + 1)
    last_end = None
    while clean[position] != "}":
        _, end = decoder.raw_decode(clean, position)
        value_start = _space(clean, _space(clean, end) + 1)
        _, last_end = decoder.raw_decode(clean, value_start)
        position = _space(clean, last_end)
        if clean[position] == ",":
            position = _space(clean, position + 1)
    return start, position, last_end


def _contains(actual: Any, expected: Any) -> bool:
    if isinstance(expected, dict):
        return isinstance(actual, dict) and all(
            key in actual and _contains(actual[key], value) for key, value in expected.items()
        )
    return type(actual) is type(expected) and actual == expected


def _merge_json(text: str, path: tuple[str, ...], entry: dict[str, Any]) -> str:
    data, clean, comments_masked = _json_document(text)
    if path[:2] == ("mcp", "servers") and isinstance(data.get("mcp"), dict):
        for name, value in data["mcp"].items():
            if (name != "servers" and isinstance(value, dict)
                    and value.get("type") in ("local", "remote")
                    and ("command" in value or "url" in value)):
                raise InitError("OpenCode V1 flat MCP server layout detected; this initializer supports V2 only. Migrate explicitly before retrying")
    current: Any = data
    for index, key in enumerate(path):
        if not isinstance(current, dict):
            raise InitError("Existing configuration parent is not an object")
        if key not in current:
            value: Any = entry
            for nested in reversed(path[index + 1 :]):
                value = {nested: value}
            _, close, last_end = _object_span(clean, path[:index])
            line_start = text.rfind("\n", 0, close) + 1
            prefix = text[line_start:close]
            separate_line = not prefix.strip()
            insertion = line_start if separate_line else close
            indent = (prefix if separate_line else "") + "  "
            newline = "\r\n" if "\r\n" in text else "\n"
            rendered = json.dumps(key) + ": " + json.dumps(value, indent=2, ensure_ascii=False)
            block = newline.join(indent + line for line in rendered.splitlines()) + newline
            if insertion and text[insertion - 1] != "\n":
                block = newline + block
            edits = [(insertion, block)]
            if last_end is not None and "," not in comments_masked[last_end:close]:
                edits.append((last_end, ","))
            for position, addition in sorted(edits, key=lambda edit: edit[0], reverse=True):
                text = text[:position] + addition + text[position:]
            _json_document(text)
            return text
        current = current[key]
    if not _contains(current, entry):
        raise InitError("Existing token-guard entry differs; preserve it and resolve the conflict manually")
    return text


def _merge_toml(text: str, entry: dict[str, Any]) -> str:
    if tomllib is None:
        raise InitError("Codex initialization requires Python 3.11+ (stdlib tomllib); no TOML was changed")
    try:
        parsed = tomllib.loads(text)
    except ValueError as exc:
        raise InitError("Invalid TOML configuration") from exc
    servers = parsed.get("mcp_servers", {})
    if not isinstance(servers, dict):
        raise InitError("Existing mcp_servers must be a TOML table")
    if SERVER_NAME in servers:
        if not _contains(servers[SERVER_NAME], entry):
            raise InitError("Existing token-guard TOML entry differs; resolve the conflict manually")
        return text
    newline = "\r\n" if "\r\n" in text else "\n"
    lines = [f'[mcp_servers."{SERVER_NAME}"]']
    for key, value in entry.items():
        lines.append(f"{key} = {json.dumps(value, ensure_ascii=False)}")
    proposed = text + (newline * 2 if text else "") + newline.join(lines) + newline
    try:
        merged = tomllib.loads(proposed)
    except ValueError as exc:
        raise InitError("TOML cannot be extended safely (for example, an inline mcp_servers table); merge manually") from exc
    if not _contains(merged.get("mcp_servers", {}).get(SERVER_NAME), entry):
        raise InitError("TOML merge validation failed")
    return proposed


def _opencode_path(root: Path) -> Path:
    candidates = [root / name for name in (
        "opencode.json", "opencode.jsonc", ".opencode/opencode.json", ".opencode/opencode.jsonc"
    )]
    for path in candidates:
        _guard(root, path)
    existing = [path for path in candidates if path.exists()]
    if len(existing) > 1:
        raise InitError("Multiple OpenCode config files exist; resolve their precedence before initialization")
    return existing[0] if existing else candidates[0]


def _check_snapshot(root: Path, change: Change, expected: bytes | None) -> None:
    current, mode = _read(root, change.path)
    if current != expected or (expected is not None and mode != change.mode):
        raise InitError(f"Configuration changed during initialization: {change.path.relative_to(root)}")


def _make_parents(root: Path, parent: Path, created: list[Path]) -> None:
    current = root
    for part in parent.relative_to(root).parts:
        current /= part
        if not current.exists():
            current.mkdir(mode=0o700)
            created.append(current)
        _guard(root, current / ".tokenguard-type-check")


def _stage(root: Path, path: Path, data: bytes, mode: int, suffix: str) -> Path:
    _guard(root, path)
    descriptor, name = tempfile.mkstemp(prefix=path.name + ".tokenguard-", suffix=suffix, dir=path.parent)
    staged = Path(name)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            if hasattr(os, "fchmod"):
                os.fchmod(stream.fileno(), mode)
            else:
                os.chmod(staged, mode)
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
    except BaseException:
        staged.unlink(missing_ok=True)
        raise
    return staged


def _replace(source: Path, destination: Path) -> None:
    os.replace(source, destination)


def _publish_new(source: Path, destination: Path) -> None:
    os.link(source, destination)


def _apply(root: Path, changes: list[Change]) -> dict[str, Any]:
    changed = [change for change in changes if change.original != change.proposed]
    staged: dict[Path, Path] = {}
    backups: list[str] = []
    directories: list[Path] = []
    attempted: list[Change] = []
    rollback_errors: list[str] = []
    try:
        for change in changed:
            _check_snapshot(root, change, change.original)
        for change in changed:
            _make_parents(root, change.path.parent, directories)
            _check_snapshot(root, change, change.original)
            if change.original is not None:
                backup = _stage(root, change.path, change.original, 0o600, ".bak")
                backups.append(str(backup.relative_to(root)))
            staged[change.path] = _stage(root, change.path, change.proposed, change.mode, ".tmp")
        for change in changed:
            _check_snapshot(root, change, change.original)
            attempted.append(change)
            if change.original is None:
                _publish_new(staged[change.path], change.path)
            else:
                _replace(staged[change.path], change.path)
        return {"ok": True, "status": "applied" if changed else "unchanged", "backups": backups}
    except (Exception, KeyboardInterrupt) as exc:
        for change in reversed(attempted):
            try:
                current, mode = _read(root, change.path)
                if current == change.original and (current is None or mode == change.mode):
                    continue
                if current != change.proposed or mode != change.mode:
                    raise InitError("Concurrent configuration change prevents rollback")
                if change.original is None:
                    if not os.path.samestat(change.path.lstat(), staged[change.path].lstat()):
                        raise InitError("Concurrent file occupies the new configuration path")
                    change.path.unlink()
                else:
                    restore = _stage(root, change.path, change.original, change.mode, ".tmp")
                    staged[change.path] = restore
                    _replace(restore, change.path)
            except (Exception, KeyboardInterrupt) as restore_error:
                rollback_errors.append(f"{change.path.relative_to(root)}: {type(restore_error).__name__}")
        return {
            "ok": False,
            "status": "partial_failure" if rollback_errors else "rolled_back",
            "errors": [str(exc) or type(exc).__name__],
            "rollback_errors": rollback_errors,
            "backups": backups,
        }
    finally:
        for path in staged.values():
            try:
                _guard(root, path)
                path.unlink(missing_ok=True)
            except (OSError, InitError):
                # Staged-file cleanup is best-effort; retain the original failure result.
                pass
        for directory in reversed(directories):
            try:
                _guard(root, directory / ".tokenguard-type-check")
                directory.rmdir()
            except (OSError, InitError):
                # Parent cleanup may fail if a directory remains nonempty or changes.
                pass


def init_workspace(root: Path, clients: list[str], *, apply: bool = False) -> str:
    """Return a reviewable JSON plan; apply only after validating every target."""
    result: dict[str, Any] = {
        "schema_version": 1, "ok": False, "status": "error", "apply": apply,
        "changes": [], "errors": [], "backups": [],
        "notes": [
            "Configuration points to this installed helper. Plugin cache upgrades can move or remove it; use a stable extracted directory before --apply.",
            "Use workspace setup when the client has no active native Token Guard server; avoid duplicate registration.",
            "No runtimes are copied and no global profiles, approvals, hooks or other servers are changed.",
            "Claude/Copilot use common stdio fields; host tool timeouts and trust remain client-specific.",
            "OpenCode initialization targets V2; flat V1 MCP server layouts require explicit migration.",
            "New files require same-filesystem hard-link support and are published without overwriting an occupied path.",
            "Existing files use a snapshot check followed by replacement; there is no portable compare-and-swap against noncooperating writers between those operations.",
            "Each file replacement is atomic, but multiple files are not one filesystem transaction. Recoverable errors trigger rollback; a crash or concurrent change can prevent complete recovery.",
        ],
    }
    try:
        selected = list(CLIENTS) if clients == ["all"] else list(dict.fromkeys(clients))
        if not selected or any(client not in CLIENTS for client in selected):
            raise InitError("Choose explicit clients: " + ", ".join(CLIENTS) + " (or all)")
        requested = Path(root).absolute()
        if requested.is_symlink() or not requested.is_dir():
            raise InitError("Workspace must be an existing directory, not a symlink")
        root = requested.resolve(strict=True)
        helper = Path(__file__).resolve().with_name("tokenguard.py")
        if not helper.is_file() or not sys.executable:
            raise InitError("The sibling tokenguard.py and current Python executable must be available")
        command = str(Path(sys.executable).absolute())
        if any(marker in value for value in (command, str(helper), str(root))
               for marker in ("${", "{env:", "{file:")):
            raise InitError("Workspace/runtime paths contain client substitution syntax; choose literal paths before initialization")
        args = ["-B", str(helper), "--root", str(root), "mcp"]
        common = {"type": "stdio", "command": command, "args": args}
        result.update(root=str(root), clients=selected)
        targets: dict[Path, tuple[tuple[str, ...], dict[str, Any], list[str]]] = {}
        for client in selected:
            if client in ("claude", "copilot"):
                path, keys, entry = root / ".mcp.json", ("mcpServers", SERVER_NAME), common
            elif client == "vscode":
                path, keys, entry = root / ".vscode/mcp.json", ("servers", SERVER_NAME), common
            elif client == "codex":
                path, keys = root / ".codex/config.toml", ("mcp_servers", SERVER_NAME)
                entry = {"command": command, "args": args, "tool_timeout_sec": 3660}
            else:
                path, keys = _opencode_path(root), ("mcp", "servers", SERVER_NAME)
                entry = {"type": "local", "command": [command, *args], "timeout": {"execution": 3660000}}
            if path in targets:
                targets[path][2].append(client)
            else:
                targets[path] = (keys, entry, [client])
        changes = []
        for path, (keys, entry, owners) in targets.items():
            try:
                original, mode = _read(root, path)
                text = original.decode("utf-8") if original is not None else ("" if path.suffix == ".toml" else "{}\n")
                proposed = (_merge_toml(text, entry) if path.suffix == ".toml" else _merge_json(text, keys, entry)).encode("utf-8")
                change = Change(path, original, proposed, mode, owners, entry)
                changes.append(change)
                result["changes"].append({
                    "path": str(path.relative_to(root)), "clients": owners,
                    "action": "unchanged" if original == proposed else ("create" if original is None else "update"),
                    "before_sha256": hashlib.sha256(original).hexdigest() if original is not None else None,
                    "after_sha256": hashlib.sha256(proposed).hexdigest(),
                    "server_path": list(keys), "server": entry,
                })
            except (OSError, ValueError, RecursionError) as exc:
                result["errors"].append(f"{path.relative_to(root)}: {exc}")
        if not result["errors"]:
            result.update(_apply(root, changes) if apply else {"ok": True, "status": "planned"})
    except (OSError, ValueError, TypeError) as exc:
        result["errors"].append(str(exc))
    return json.dumps(result, indent=2, ensure_ascii=False)
