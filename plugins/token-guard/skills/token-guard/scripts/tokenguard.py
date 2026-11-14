#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Bound agent context while retaining command exit status and raw local evidence.

This is a presentation helper, not a sandbox, test oracle or token accounting API.
Only Python outlines use an AST. Other outlines are explicitly heuristic.
Commands inherit stdin and environment; combined file capture is not a transparent TTY.
"""
from __future__ import annotations

import argparse
import ast
from collections import deque
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import time


SOURCE_LIMIT = 2 * 1024 * 1024
SLICE_LINES = 200
SLICE_BYTES = 12_000
OUTPUT_BYTES = 8_000
SCAN_BYTES = 16 * 1024 * 1024
LINE_BYTES = 2_000
SELECTION_NOTICE = "[OUTPUT INCOMPLETE: selected excerpt; complete raw log retained]\n"
SANITIZATION = "UTF-8 replacement decoding; ANSI and control characters removed"
ANSI = re.compile(r"\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07\x1b]*(?:\x07|\x1b\\))")
DIAGNOSTIC = re.compile(r"\b(error|fatal|failed|failure|panic|exception|traceback|warning)\b", re.I)
RESULT = re.compile(r"(?:test result:|\bRan \d+ tests?|\b\d+ passed\b|^OK\b|^FAILED\b|\bfinished\b)", re.I)
PY_LOCATION = re.compile(r'File "([^"\r\n]+)", line (\d+)')
CODE_LOCATION = re.compile(
    r"^\s*(?:-->|at\s+)?\s*(.+?\.(?:rs|py|pyi|c|h|cc|cpp|hpp|ts|tsx|js|jsx)):(\d+)(?::\d+)?"
)
OUTLINE = re.compile(
    r"^\s*(?:(?:pub(?:\([^)]*\))?|async|unsafe|const|export|default|declare|abstract|static)\s+)*"
    r"(?:fn|struct|enum|trait|impl|type|mod|class|interface|function|namespace)\b"
)


def clean(text: str) -> str:
    text = ANSI.sub("", text)
    return "".join(c for c in text if c in "\n\t" or (ord(c) >= 32 and ord(c) != 127))


def bounded(text: str, limit: int) -> str:
    data = text.encode("utf-8")
    if len(data) <= limit:
        return text
    marker = "\n[TRUNCATED: use a narrower range or the raw log]\n"
    return data[:max(0, limit - len(marker.encode()))].decode("utf-8", errors="ignore") + marker


def rooted(root: Path, value: str | Path) -> Path:
    root = root.resolve(strict=True)
    path = Path(value)
    path = (path if path.is_absolute() else root / path).resolve(strict=True)
    if not path.is_relative_to(root):
        raise ValueError("path resolves outside --root")
    if not path.is_file():
        raise ValueError("path must be a regular file")
    return path


def source(root: Path, value: str | Path) -> tuple[Path, str, str]:
    path = rooted(root, value)
    with path.open("rb") as handle:
        data = handle.read(SOURCE_LIMIT + 1)
    if len(data) > SOURCE_LIMIT:
        raise ValueError(f"source exceeds {SOURCE_LIMIT} bytes; use a targeted tool")
    if b"\x00" in data:
        raise ValueError("binary source is not supported")
    return path, data.decode("utf-8"), hashlib.sha256(data).hexdigest()


def read_slice(root: Path, path: str, start: int, end: int,
               expected_sha: str | None = None) -> str:
    if start < 1 or end < start or end - start + 1 > SLICE_LINES:
        raise ValueError(f"require 1 <= start <= end and at most {SLICE_LINES} lines")
    file, text, digest = source(root, path)
    if expected_sha is not None and digest != expected_sha:
        raise ValueError("source changed: --expect-sha does not match")
    lines = text.splitlines()
    if start > len(lines) or end > len(lines):
        raise ValueError(f"range exceeds the file's {len(lines)} lines")
    body = "\n".join(f"{i + 1}: {lines[i]}" for i in range(start - 1, end))
    if len(body.encode("utf-8")) > SLICE_BYTES:
        raise ValueError(f"slice exceeds {SLICE_BYTES} bytes; request fewer lines")
    return f"{file.relative_to(root.resolve())} sha256={digest} lines={start}-{end}\n{clean(body)}"


def inspect_file(root: Path, path: str) -> str:
    file, text, digest = source(root, path)
    lines = text.splitlines()
    kind = "python-ast" if file.suffix == ".py" else "heuristic; use hosted Graphify for structure"
    out = [f"{file.relative_to(root.resolve())} lines={len(lines)} sha256={digest}", f"outline={kind}"]
    if file.suffix == ".py":
        tree = ast.parse(text, filename=str(file))
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                prefix = "async " if isinstance(node, ast.AsyncFunctionDef) else ""
                signature = f"{prefix}def {node.name}({ast.unparse(node.args)})"
                if node.returns:
                    signature += f" -> {ast.unparse(node.returns)}"
                out.append((node.lineno, f"{node.lineno}-{node.end_lineno}: {signature}"))
            elif isinstance(node, ast.ClassDef):
                bases = ", ".join(ast.unparse(x) for x in node.bases)
                out.append((node.lineno, f"{node.lineno}-{node.end_lineno}: class {node.name}({bases})"))
        out = out[:2] + [s for _, s in sorted(out[2:])]
    else:
        # ponytail: this is navigation only; an actual language parser belongs in Graphify.
        for index, line in enumerate(lines):
            if OUTLINE.match(line):
                out.append(f"{index + 1}: {line.strip()}")
        if len(out) == 2:
            out.append("No supported declarations found; use a targeted slice or hosted Graphify.")
    return bounded(clean("\n".join(out)), OUTPUT_BYTES)


def log_excerpt(path: Path, output: str = "summary") -> tuple[str, list[tuple[str, int]], bool]:
    """Sample diagnostics with bounded memory; keep complete original bytes on disk."""
    with path.open("rb") as handle:
        raw = handle.read(OUTPUT_BYTES + 1)
    text = clean(raw.decode("utf-8", errors="replace"))
    if len(raw) <= OUTPUT_BYTES and len(text.encode("utf-8")) <= OUTPUT_BYTES:
        locations = []
        for line in text.splitlines():
            match = PY_LOCATION.search(line) or CODE_LOCATION.search(line)
            if match and len(match.group(2)) <= 10 and len(locations) < 12:
                value = (match.group(1).strip(), int(match.group(2)))
                if value not in locations:
                    locations.append(value)
        return text, locations, True
    head: list[str] = []
    selected: dict[int, str] = {}
    before: deque[tuple[int, str]] = deque(maxlen=2)
    tail: deque[str] = deque(maxlen=12)
    results: deque[str] = deque(maxlen=8)
    locations: list[tuple[str, int]] = []
    consumed = 0
    following = 0
    line_number = 0
    sampled_diagnostics = False
    with path.open("rb") as handle:
        while consumed < SCAN_BYTES:
            raw = handle.readline(LINE_BYTES + 1)
            if not raw:
                break
            consumed += len(raw)
            line_number += 1
            long_line = len(raw) > LINE_BYTES and not raw.endswith(b"\n")
            line = clean(raw[:LINE_BYTES].decode("utf-8", errors="replace").rstrip("\r\n"))
            if long_line:
                line += " [LINE TRUNCATED; raw log retained]"
                while raw and not raw.endswith(b"\n") and consumed < SCAN_BYTES:
                    raw = handle.readline(LINE_BYTES + 1)
                    consumed += len(raw)
            if len(head) < 5:
                head.append(line)
            tail.append(line)
            if RESULT.search(line):
                results.append(line)
            match = PY_LOCATION.search(line) or CODE_LOCATION.search(line)
            if match and len(match.group(2)) <= 10 and len(locations) < 12:
                value = (match.group(1).strip(), int(match.group(2)))
                if value not in locations:
                    locations.append(value)
            if DIAGNOSTIC.search(line):
                if len(selected) < 60:
                    selected.update(before)
                    following = 4
                else:
                    sampled_diagnostics = True
            if following and len(selected) < 70:
                selected[line_number] = line
                following -= 1
            before.append((line_number, line))
        omitted_scan = handle.read(1) != b""
        if omitted_scan:
            handle.seek(max(0, path.stat().st_size - 24_000))
            last = handle.read(24_000).decode("utf-8", errors="replace").splitlines()
            tail = deque((bounded(clean(x), LINE_BYTES) for x in last[-12:]), maxlen=12)
    sections = [SELECTION_NOTICE.rstrip()]
    if output == "bounded":
        sections.append("[head]\n" + bounded("\n".join(head), 1000))
        sections.append("[MIDDLE OUTPUT OMITTED; complete raw log retained]")
    if selected:
        diagnostics = "\n".join(f"{n}: {s}" for n, s in sorted(selected.items()))
        sections.append("[diagnostics; line numbers refer to raw log]\n" + bounded(diagnostics, 4500))
    if sampled_diagnostics:
        sections.append("[DIAGNOSTICS SAMPLED; omitted lines remain in raw log]")
    if results:
        sections.append("[results]\n" + "\n".join(results))
    needs_tail = bool(selected and line_number > max(selected))
    if needs_tail and max(selected) < line_number - len(tail):
        sections.append("[INTERMEDIATE OUTPUT OMITTED; complete raw log retained]")
    if tail and (output == "bounded" or sampled_diagnostics or omitted_scan or needs_tail or (not selected and not results)):
        tail_text = "\n".join(tail)
        if len(tail_text.encode("utf-8")) > 2000:
            tail_text = "[TAIL START OMITTED]\n" + tail_text.encode("utf-8")[-2000:].decode("utf-8", errors="replace")
        sections.append("[tail]\n" + tail_text)
    if omitted_scan:
        sections.insert(0, f"[SCAN LIMITED to first {SCAN_BYTES} bytes and final 24000 bytes; raw log retained]")
    return bounded("\n".join(sections), OUTPUT_BYTES), locations, False


def snippets(root: Path, locations: list[tuple[str, int]]) -> str:
    out: list[str] = []
    for name, line in locations:
        if len(out) == 2:
            break
        try:
            _, text, _ = source(root, name)
            count = len(text.splitlines())
            if not 1 <= line <= count:
                continue
            out.append(read_slice(root, name, max(1, line - 3), min(count, line + 3)))
        except (OSError, ValueError, UnicodeError):
            continue
    return bounded("\n".join(out), 3_000)


def stop_process(process: subprocess.Popen, *, reap: bool = True) -> None:
    if os.name == "posix":
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    else:
        # Windows cannot portably kill arbitrary descendant trees with stdlib alone.
        process.kill()
    if reap:
        process.wait()


def rtk_excerpt(log_path: Path) -> str | None:
    rtk = shutil.which("rtk")
    if not rtk:
        return None
    descriptor, name = tempfile.mkstemp(prefix="rtk-", suffix=".log", dir=log_path.parent)
    filtered = Path(name)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            result = subprocess.run([rtk, "log", str(log_path)], stdout=handle,
                                    stderr=subprocess.DEVNULL, timeout=3, check=False)
        if result.returncode or not filtered.stat().st_size:
            return None
        with filtered.open("rb") as handle:
            content = handle.read(OUTPUT_BYTES + 1).decode("utf-8", errors="replace")
        return bounded(clean(content), OUTPUT_BYTES)
    except (OSError, subprocess.TimeoutExpired):
        return None
    finally:
        filtered.unlink(missing_ok=True)


def run_command(root: Path, argv: list[str], timeout: float | None = None,
                output: str = "summary") -> int:
    if not argv:
        raise ValueError("run requires a command after --")
    if timeout is not None and (not math.isfinite(timeout) or timeout <= 0):
        raise ValueError("timeout must be a finite positive number")
    root = root.resolve(strict=True)
    directory = root / "build" / "tokenguard"
    if not directory.resolve().is_relative_to(root):
        raise ValueError("log directory resolves outside --root")
    directory.mkdir(parents=True, exist_ok=True)
    if not directory.resolve().is_relative_to(root):
        raise ValueError("log directory resolves outside --root")
    descriptor, name = tempfile.mkstemp(prefix="run-", suffix=".log", dir=directory)
    log_path = Path(name)
    started = time.monotonic()
    process = None
    terminated = 0

    def terminate(signum, _frame):
        nonlocal terminated
        terminated = signum
        if process is not None:
            # Popen.wait may hold its non-reentrant lock when this signal arrives.
            stop_process(process, reap=False)

    previous_handlers = {sig: signal.signal(sig, terminate) for sig in (signal.SIGTERM, signal.SIGINT)}
    try:
        with os.fdopen(descriptor, "wb") as handle:
            try:
                process = subprocess.Popen(argv, cwd=root,
                                           stdout=handle, stderr=subprocess.STDOUT,
                                           start_new_session=os.name == "posix")
            except FileNotFoundError:
                handle.write(b"ERROR: command executable was not found\n")
                code = 127
            except OSError as exc:
                handle.write(f"ERROR: cannot execute command ({type(exc).__name__})\n".encode())
                code = 126
            else:
                if terminated:
                    stop_process(process, reap=False)
                try:
                    code = process.wait(timeout=timeout)
                    if code < 0:
                        code = 128 - code
                except subprocess.TimeoutExpired:
                    stop_process(process)
                    handle.write(f"ERROR: timed out after {timeout:g} seconds\n".encode())
                    code = 124
                except KeyboardInterrupt:
                    stop_process(process)
                    handle.write(b"ERROR: interrupted by user\n")
                    code = 130
            if terminated:
                handle.write(f"ERROR: cancelled by signal {terminated}\n".encode())
                code = 128 + terminated
    finally:
        for sig, previous in previous_handlers.items():
            signal.signal(sig, previous)
    duration = round(time.monotonic() - started, 3)
    excerpt, locations, complete = log_excerpt(log_path, output)
    filtered = rtk_excerpt(log_path) if not complete and output == "summary" and code not in (124, 130, 143) else None
    # RTK is presentation only. Original diagnostics and their locations stay visible.
    use_rtk = bool(filtered and not DIAGNOSTIC.search(excerpt) and not RESULT.search(excerpt)
                   and len(filtered.encode()) < len(excerpt.encode()))
    if use_rtk:
        excerpt = SELECTION_NOTICE + bounded(filtered, OUTPUT_BYTES - len(SELECTION_NOTICE.encode()))
    extra = snippets(root, locations) if code else ""
    context_note = "post-command/current file; not tested-source identity"
    body = excerpt + (f"\n[source context: {context_note}]\n" + extra if extra else "")
    receipt = {
        "schema_version": 1, "exit_code": code, "duration_seconds": duration,
        "log": str(log_path.relative_to(root)), "raw_bytes": log_path.stat().st_size,
        "rtk_used": use_rtk, "emitted_bytes": 0,
        "output_complete": complete, "presentation_scope": "sanitized_text", "sanitization": SANITIZATION,
        "capture_mode": "combined stdout/stderr file; not a transparent TTY",
        "stdin": "inherited", "timeout_seconds": timeout,
        "source_context": context_note if extra else None,
    }
    displayed_fields = ("schema_version", "exit_code", "duration_seconds", "log", "raw_bytes",
                        "rtk_used", "emitted_bytes", "output_complete", "presentation_scope")
    # Include the status line itself in the emitted UTF-8 byte measurement.
    for _ in range(4):
        status = json.dumps({key: receipt[key] for key in displayed_fields}, sort_keys=True)
        emitted = status + "\n" + (body + "\n" if body else "")
        count = len(emitted.encode("utf-8"))
        if receipt["emitted_bytes"] == count:
            break
        receipt["emitted_bytes"] = count
    with log_path.with_suffix(".json").open("x", encoding="utf-8") as handle:
        if os.name == "posix":
            os.fchmod(handle.fileno(), 0o600)
        json.dump(receipt, handle, sort_keys=True)
        handle.write("\n")
    sys.stdout.write(emitted)
    return code


def read_log(root: Path, path: str, start: int, count: int) -> str:
    if start < 1 or not 1 <= count <= SLICE_LINES:
        raise ValueError(f"require start >= 1 and 1 <= lines <= {SLICE_LINES}")
    file = rooted(root, path)
    out = []
    with file.open("rb") as handle:
        number = 0
        while number < start + count - 1:
            line = handle.readline(SLICE_BYTES + 1)
            if not line:
                break
            number += 1
            long = len(line) > SLICE_BYTES and not line.endswith(b"\n")
            if number >= start:
                rendered = clean(line[:SLICE_BYTES].decode("utf-8", errors="replace").rstrip())
                out.append(f"{number}: {rendered}" + (" [LINE TRUNCATED]" if long else ""))
            while long and line and not line.endswith(b"\n"):
                line = handle.readline(SLICE_BYTES + 1)
    if number < start:
        raise ValueError("start exceeds the log's line count")
    return bounded("\n".join(out), SLICE_BYTES)


def read_log_bytes(root: Path, path: str, offset: int, length: int) -> str:
    if offset < 0 or not 1 <= length <= SLICE_BYTES:
        raise ValueError(f"require offset >= 0 and 1 <= length <= {SLICE_BYTES}")
    file = rooted(root, path)
    with file.open("rb") as handle:
        size = os.fstat(handle.fileno()).st_size
        if offset > size:
            raise ValueError("offset exceeds the log's byte size")
        handle.seek(offset)
        data = handle.read(length)
    next_offset = offset + len(data)
    metadata = {"offset": offset, "bytes_read": len(data), "next_offset": next_offset,
                "file_size": size, "eof": next_offset >= size,
                "sanitization": SANITIZATION, "offset_unit": "raw bytes"}
    return json.dumps(metadata, sort_keys=True) + "\n" + clean(data.decode("utf-8", errors="replace"))


def stats(root: Path) -> str:
    totals = {"runs": 0, "raw_bytes": 0, "emitted_bytes": 0}
    for path in (root / "build" / "tokenguard").glob("run-*.json"):
        try:
            file = rooted(root, path)
            if file.stat().st_size > 4096:
                continue
            entry = json.loads(file.read_text(encoding="utf-8"))
            if entry.get("schema_version") != 1:
                continue
            raw, emitted = entry["raw_bytes"], entry["emitted_bytes"]
            if type(raw) is not int or type(emitted) is not int or min(raw, emitted) < 0:
                continue
            totals["runs"] += 1
            totals["raw_bytes"] += raw
            totals["emitted_bytes"] += emitted
        except (OSError, ValueError, KeyError, TypeError):
            continue
    totals["measurement"] = "UTF-8 bytes only; not provider tokens, cache hits or cost savings"
    return json.dumps(totals, sort_keys=True)


def hook() -> int:
    """Rewrite only unambiguous local build/test argv; retain normal permissions."""
    if os.environ.get("TOKENGUARD_AUTO_WRAP") != "1":
        return 0
    try:
        payload = json.loads(sys.stdin.read(64_001))
        if payload.get("hook_event_name") != "PreToolUse" or payload.get("tool_name") != "Bash":
            return 0
        supplied_cwd = Path(payload["cwd"])
        if not supplied_cwd.is_absolute():
            return 0
        cwd = supplied_cwd.resolve(strict=True)
        if not cwd.is_dir():
            return 0
        helper = Path(__file__).resolve()
        inputs = payload["tool_input"]
        if inputs.get("run_in_background"):
            return 0
        command = inputs["command"]
        if not isinstance(command, str) or any(c in command for c in ";|&><\n\r$`*?[~(){}#!"):
            return 0
        words = shlex.split(command)
        if not words or "=" in words[0] or "tokenguard" in command or words[0] == "rtk":
            return 0
        interactive = {"--watch", "-w", "--interactive", "-i", "--pdb", "--trace",
                       "--ui", "--headed", "--looponfail", "-s", "--capture=no"}
        if any(word in interactive or word.startswith(("--watch=", "--interactive=")) for word in words):
            return 0
        executable = Path(words[0]).name
        arguments = words[1:]
        allowed = False
        if executable == "cargo":
            if arguments and arguments[0].startswith("+"):
                arguments = arguments[1:]
            allowed = bool(arguments and (arguments[0] in {"test", "build", "check", "clippy"}
                           or (arguments[0] == "fmt" and "--check" in arguments)))
        elif executable in {"python", "python3"}:
            while arguments and arguments[0] in {"-B", "-u", "-Bu", "-uB"}:
                arguments = arguments[1:]
            if len(arguments) >= 2 and arguments[0] == "-m":
                allowed = arguments[1] in {"unittest", "pytest"}
            elif arguments:
                script = Path(arguments[0])
                allowed = script.parent == Path("tools") and script.suffix == ".py" and (
                    script.stem in {"preflight", "build"} or script.stem.startswith("test_")
                    or (script.stem == "run" and "--smoke" in arguments))
        if allowed:
            updated = dict(inputs)
            updated["command"] = shlex.join([sys.executable, "-B", str(helper), "--root", str(cwd),
                                              "run", "--", *words])
            print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse", "updatedInput": updated}}))
    except (OSError, ValueError, TypeError, KeyError, AttributeError):
        pass
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    sub = parser.add_subparsers(dest="mode", required=True)
    run = sub.add_parser("run", help="Run an argv command, preserve exit status and log bytes")
    run.add_argument("--timeout", type=float, help="Optional explicit command deadline in seconds")
    run.add_argument("--output", choices=("summary", "bounded"), default="summary")
    run.add_argument("command", nargs=argparse.REMAINDER)
    inspect = sub.add_parser("inspect", help="Show bounded declarations and source fingerprint")
    inspect.add_argument("path")
    view = sub.add_parser("slice", help="Read an exact source range")
    view.add_argument("path")
    view.add_argument("start", type=int)
    view.add_argument("end", type=int)
    view.add_argument("--expect-sha")
    log = sub.add_parser("log", help="Replay a bounded range from a raw log")
    log.add_argument("path")
    log.add_argument("--start", type=int)
    log.add_argument("--lines", type=int)
    log.add_argument("--offset", type=int, help="Raw byte offset; excludes --start/--lines")
    log.add_argument("--length", type=int, help=f"Read at most {SLICE_BYTES} raw bytes (default 4096)")
    sub.add_parser("stats", help="Report observed output bytes, not billing tokens")
    sub.add_parser("doctor", help="Report local prerequisites without changing configuration")
    sub.add_parser("hook", help="Conservatively adapt Claude build/test commands")
    args = parser.parse_args(argv)
    if args.mode == "hook":
        return hook()
    try:
        root = args.root.resolve(strict=True)
        if not root.is_dir():
            raise ValueError("--root must be a directory")
        if args.mode == "run":
            command = args.command[1:] if args.command[:1] == ["--"] else args.command
            return run_command(root, command, args.timeout, args.output)
        if args.mode == "inspect":
            result = inspect_file(root, args.path)
        elif args.mode == "slice":
            result = read_slice(root, args.path, args.start, args.end, args.expect_sha)
        elif args.mode == "log":
            if args.offset is not None or args.length is not None:
                if args.start is not None or args.lines is not None:
                    raise ValueError("byte ranges cannot be combined with --start/--lines")
                result = read_log_bytes(root, args.path, 0 if args.offset is None else args.offset,
                                        4096 if args.length is None else args.length)
            else:
                result = read_log(root, args.path, 1 if args.start is None else args.start,
                                  80 if args.lines is None else args.lines)
        elif args.mode == "stats":
            result = stats(root)
        else:
            result = json.dumps({"python": sys.version.split()[0], "root": str(root),
                                 "rtk": bool(shutil.which("rtk")),
                                 "graphify": "hosted MCP is supplied by the client; not probed",
                                 "timeout_cleanup": "POSIX process group" if os.name == "posix" else "direct child only"}, sort_keys=True)
        print(result)
        return 0
    except (OSError, ValueError, UnicodeError, SyntaxError) as exc:
        print(f"ERROR: {clean(str(exc))}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
