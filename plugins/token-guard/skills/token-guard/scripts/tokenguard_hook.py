# SPDX-License-Identifier: MPL-2.0
"""Optional bounded-read guidance for Claude's native tools; not access control."""
from __future__ import annotations

import os
from pathlib import Path
import shlex
import sys


SOURCE_SUFFIXES = {".py", ".pyi", ".rs", ".c", ".h", ".cc", ".cpp", ".hpp", ".js", ".jsx", ".ts", ".tsx", ".go", ".java", ".cs"}
SKIP_PARTS = {"node_modules", "target", "build"}


def simple_read(command: str) -> str | None:
    """Only recognize a literal single-file read, never a shell program."""
    if not isinstance(command, str) or any(c in command for c in ";|&><\n\r$`*?[~(){}!"):
        return None
    try:
        words = shlex.split(command, comments=True, posix=os.name != "nt")
    except ValueError:
        return None
    if not words:
        return None

    def literal(token: str) -> str:
        # POSIX shlex already removed syntactic quotes. Removing them again
        # would change filenames that contain literal apostrophes or quotes.
        if os.name == "nt" and len(token) >= 2 and token[0] in "\"'" and token[-1] == token[0]:
            return token[1:-1]
        return token

    command_name = Path(words[0]).name.lower()
    args = words[1:]
    if command_name == "cat":
        if args[:1] == ["--"]:
            args = args[1:]
        if len(args) == 1 and not args[0].startswith("-"):
            return literal(args[0])
    if command_name in {"get-content", "gc"}:
        if len(args) == 1 and not args[0].startswith("-"):
            return literal(args[0])
        if len(args) == 2 and args[0].lower() in {"-path", "-literalpath"}:
            return literal(args[1])
    return None


def guard(payload: dict, inspect_file) -> int:
    if os.environ.get("TOKENGUARD_READ_GUARD") != "1":
        return 0
    inputs = payload.get("tool_input")
    if payload.get("hook_event_name") != "PreToolUse" or not isinstance(inputs, dict):
        return 0
    tool = payload.get("tool_name")
    cwd = payload.get("cwd")
    if not isinstance(cwd, str) or not Path(cwd).is_absolute():
        return 0
    root = Path(cwd).resolve(strict=True)
    if not root.is_dir():
        return 0
    path = None
    if tool == "Read":
        path = inputs.get("file_path")
        limit = inputs.get("limit")
        if type(limit) is int and 0 < limit <= 160:
            return 0
        if limit is not None and (type(limit) is not int or limit <= 0):
            return 0
    elif tool == "Bash":
        path = simple_read(inputs.get("command"))
    elif tool in {"Grep", "Glob"}:
        # Intercept only explicitly broad scans of generated/plugin directories.
        # Scoped queries and bounded results remain available for required evidence.
        query_path = inputs.get("path", "")
        if not isinstance(query_path, str):
            return 0
        parts = Path(query_path.replace("\\", "/")).parts
        generated = bool(set(parts) & SKIP_PARTS) or any(
            parts[i:i + 3] == (".codex", "plugins", "cache") for i in range(max(0, len(parts) - 2)))
        generated = generated or any(parts[i:i + 2] == (".claude", "plugins") for i in range(max(0, len(parts) - 1)))
        if tool == "Grep":
            limit = inputs.get("head_limit")
            if type(limit) is int and 0 < limit <= 160:
                return 0
            broad = inputs.get("pattern") in {".*", ".", "^", ""}
        else:
            broad = inputs.get("pattern") in {"**", "**/*", "*", ""}
        if generated and broad:
            sys.stderr.write("TOKEN GUARD: broad generated/plugin-directory scan paused. Use a specific path/pattern or a bounded Grep result. Required source and instructions remain accessible.\n")
            return 2
        return 0
    if not isinstance(path, str) or not path:
        return 0
    file = Path(path)
    file = (file if file.is_absolute() else root / file).resolve(strict=True)
    # Keep outline production within the explicitly trusted workspace. A hook
    # cannot evaluate the client's file deny rules; its opt-in grants separate
    # source-read trust, before the original tool reaches normal approval.
    if not file.is_relative_to(root) or not file.is_file() or file.suffix not in SOURCE_SUFFIXES:
        return 0
    if file.stat().st_size > 2 * 1024 * 1024:
        return 0
    with file.open("rb") as handle:
        if sum(1 for _ in handle) <= 150:
            return 0
    try:
        outline = inspect_file(root, str(file))
    except (OSError, ValueError, SyntaxError, UnicodeError):
        return 0
    sys.stderr.write("TOKEN GUARD: full source read (>150 lines) paused. Outline follows; no separate inspect call is needed. Use read_slice or Read with offset and limit for the required implementation.\n" + outline + "\n")
    return 2
