#!/usr/bin/env python3
"""Keep bounded, source-bound CI reports without depending on artifact storage.

This collector does not run tests or turn a job result into an acceptance claim.
The upload action handles archive availability separately. Only explicit input
paths are inspected; raw images and build products never enter the compact copy.
"""

from __future__ import annotations

import argparse
from collections import Counter
from datetime import datetime, timezone
import fnmatch
import hashlib
import html
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
from typing import Mapping, TextIO
import unicodedata


MAX_FILE_BYTES = 1024 * 1024
MAX_TOTAL_BYTES = 8 * 1024 * 1024
MAX_FILES = 500  # Includes the receipt; reserve one slot and 1 MiB for metadata.
MAX_SCAN_ENTRIES = 10_000
MAX_PATTERNS = 100
MAX_PATH_BYTES = 512
MAX_CONSOLE_BYTES = 256 * 1024
MAX_PREVIEW_BYTES = 16 * 1024
TEXT_SUFFIXES = {".json", ".txt", ".log", ".md", ".sha256", ".xml", ".csv"}
STATUSES = {"success", "failure", "cancelled", "skipped", "unknown"}
SENSITIVE_NAME = re.compile(
    r"(?:^|[-_.])(secrets?|credentials?|passwords?|tokens?|private[-_]key)"
    r"(?:$|[-_.])",
    re.IGNORECASE,
)
SENSITIVE_CONTENT = re.compile(
    r"-----BEGIN (?:[A-Z0-9 ]+ )?PRIVATE KEY-----"
    r"|\bgh[pousr]_[A-Za-z0-9]{20,}\b"
    r"|\bgithub_pat_[A-Za-z0-9_]{20,}\b"
    r"|\bAKIA[A-Z0-9]{16}\b"
    r"|(?im:^\s*(?:AWS_SECRET_ACCESS_KEY|Authorization)\s*[:=]\s*\S+)"
    r"|(?i:[\"'](?:password|secret|api[_-]?key|access[_-]?token|refresh[_-]?token)"
    r"[\"']\s*:\s*[\"'][^\"']{8,}[\"'])"
)


class EvidenceError(RuntimeError):
    """A collector/configuration failure, never a synthetic passing test."""


def _prefix_bytes(value: str, limit: int) -> str:
    return value.encode("utf-8")[:limit].decode("utf-8", errors="ignore")


def _inert(value: str) -> str:
    """Prefix every physical line and neutralize both runner command syntaxes."""
    lines = []
    for line in value.split("\n"):
        escaped = "".join(
            f"\\u{ord(char):04x}"
            if unicodedata.category(char) in {"Cc", "Cf", "Zl", "Zp"}
            else char
            for char in line
        )
        escaped = escaped.replace("::", ": :").replace("##[", "# #[")
        lines.append("[ci-evidence] | " + escaped)
    return "\n".join(lines) + "\n"


class Console:
    def __init__(self) -> None:
        self.parts: list[str] = []
        self.size = 0
        self.omitted_blocks = 0

    def add(self, value: str) -> bool:
        rendered = _inert(value)
        size = len(rendered.encode("utf-8"))
        # Keep room for an explicit final cap notice, even for hostile input.
        if self.size + size > MAX_CONSOLE_BYTES - 512:
            self.omitted_blocks += 1
            return False
        self.parts.append(rendered)
        self.size += size
        return True

    def emit(self, stream: TextIO) -> None:
        stream.write("".join(self.parts))
        if self.omitted_blocks:
            stream.write(_inert(
                "CONSOLE LIMIT: "
                f"{self.omitted_blocks} preview block(s) omitted; "
                "the compact files and receipt remain unchanged."
            ))


def _path_label(value: str) -> str:
    if len(value.encode("utf-8")) <= MAX_PATH_BYTES:
        return value
    digest = hashlib.sha256(value.encode("utf-8")).hexdigest()
    return _prefix_bytes(value, MAX_PATH_BYTES) + f" [path truncated; sha256={digest}]"


class Selection:
    """Expand globs without following symlinks, including at intermediate levels."""

    def __init__(self, workspace: Path, output: Path) -> None:
        self.workspace = workspace
        self.output = output
        self.files: dict[str, Path] = {}
        self.omitted: dict[str, dict] = {}
        self.missing: list[str] = []
        self.warnings: list[str] = []
        self.scanned = 0

    def omit(self, path: Path | str, reason: str, size: int | None = None) -> None:
        label = str(path)
        if isinstance(path, Path):
            label = path.relative_to(self.workspace).as_posix()
        self.omitted.setdefault(label, {
            "path": _path_label(label), "reason": reason, "size_bytes": size,
        })

    def blocked(self, path: Path) -> bool:
        relative = path.relative_to(self.workspace)
        if path == self.output or self.output in path.parents:
            self.omit(path, "collector-output")
            return True
        if any(part.startswith(".") for part in relative.parts):
            self.omit(path, "hidden-path")
            return True
        if relative.parts and relative.parts[0] == "_meta":
            self.omit(path, "reserved-metadata-path")
            return True
        if any(SENSITIVE_NAME.search(part) for part in relative.parts):
            self.omit(path, "sensitive-name")
            return True
        if path.is_symlink():
            self.omit(path, "symlink")
            return True
        return False

    def entries(self, path: Path) -> list[Path]:
        entries = []
        with os.scandir(path) as iterator:
            for entry in iterator:
                self.scanned += 1
                if self.scanned > MAX_SCAN_ENTRIES:
                    raise EvidenceError("Input traversal exceeded the 10,000-entry safety limit")
                entries.append(Path(entry.path))
        return sorted(entries)

    def expand(self, current: Path, parts: tuple[str, ...]):
        if self.blocked(current):
            return
        if not parts:
            if current.exists():
                yield current
            return
        if not current.is_dir():
            return
        part, rest = parts[0], parts[1:]
        if part == "**":
            yield from self.expand(current, rest)
            for child in self.entries(current):
                if not self.blocked(child) and child.is_dir():
                    yield from self.expand(child, parts)
        elif any(char in part for char in "*?["):
            for child in self.entries(current):
                if fnmatch.fnmatchcase(child.name, part):
                    yield from self.expand(child, rest)
        else:
            yield from self.expand(current / part, rest)

    def add(self, path: Path) -> None:
        if self.blocked(path):
            return
        info = path.lstat()
        if stat.S_ISDIR(info.st_mode):
            for child in self.entries(path):
                self.add(child)
        elif stat.S_ISREG(info.st_mode):
            self.files.setdefault(path.relative_to(self.workspace).as_posix(), path)
        else:
            self.omit(path, "non-regular-file")

    def select(self, patterns: list[str]) -> None:
        for pattern in patterns:
            raw = Path(pattern)
            # Reject traversal before normalization; never expand '~' or variables.
            if ".." in raw.parts or pattern.startswith(("!", "~")):
                self.missing.append(_path_label(pattern))
                self.warnings.append(f"Rejected unsafe/unsupported pattern: {_path_label(pattern)}")
                continue
            if raw.is_absolute():
                try:
                    raw = raw.relative_to(self.workspace)
                except ValueError:
                    self.missing.append(_path_label(pattern))
                    self.warnings.append(f"Rejected pattern outside workspace: {_path_label(pattern)}")
                    continue
            matched = False
            for path in self.expand(self.workspace, raw.parts):
                matched = True
                self.add(path)
            if not matched:
                self.missing.append(_path_label(pattern))


def _read_report(workspace: Path, relative: str) -> bytes:
    """Open relative to anchored directory descriptors, refusing symlink races."""
    if not hasattr(os, "O_NOFOLLOW") or os.open not in os.supports_dir_fd:
        raise EvidenceError("Safe no-follow descriptor-relative reads are unavailable")
    flags = os.O_RDONLY | os.O_NOFOLLOW
    directory = os.open(workspace, flags | os.O_DIRECTORY)
    try:
        parts = Path(relative).parts
        for part in parts[:-1]:
            following = os.open(part, flags | os.O_DIRECTORY, dir_fd=directory)
            os.close(directory)
            directory = following
        descriptor = os.open(parts[-1], flags, dir_fd=directory)
        with os.fdopen(descriptor, "rb") as handle:
            if not stat.S_ISREG(os.fstat(handle.fileno()).st_mode):
                raise EvidenceError(f"Input changed to a non-regular file: {relative}")
            return handle.read(MAX_FILE_BYTES + 1)
    finally:
        os.close(directory)


def _checkout_sha(workspace: Path) -> str | None:
    try:
        result = subprocess.run(
            ["git", "-C", str(workspace), "rev-parse", "HEAD"],
            capture_output=True, text=True, check=False, timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    value = result.stdout.strip()
    if result.returncode == 0 and re.fullmatch(r"[0-9a-f]{40}(?:[0-9a-f]{24})?", value):
        return value
    return None


def _json_bytes(value: dict) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n").encode("utf-8")


def _receipt_bytes(receipt: dict) -> bytes:
    # Metadata is bounded too. Keep every retained hash; explicitly count omitted
    # entries dropped from the receipt if a huge binary tree exhausts its budget.
    receipt["omitted_list_truncated_count"] = 0
    while True:
        encoded = _json_bytes(receipt)
        if len(encoded) <= MAX_FILE_BYTES:
            return encoded
        if not receipt["omitted"]:
            raise EvidenceError("Receipt exceeds the metadata safety limit")
        if receipt["omitted_list_truncated_count"] == 0:
            receipt["warnings"].append(
                "Omitted-path listing exceeded metadata budget; truncation count and reason totals are recorded"
            )
        drop = max(1, len(receipt["omitted"]) // 2)
        receipt["omitted"] = receipt["omitted"][:-drop]
        receipt["omitted_list_truncated_count"] += drop


def _preview(console: Console, entry: dict, data: bytes) -> None:
    path = entry["path"]
    if len(data) <= MAX_PREVIEW_BYTES:
        label = "COMPLETE FILE (inert display; stored bytes unchanged)"
        preview = data.decode("utf-8")
    elif Path(path).suffix.lower() == ".json":
        label = f"TRUNCATED JSON PREVIEW (first {MAX_PREVIEW_BYTES} bytes of {len(data)})"
        preview = data[:MAX_PREVIEW_BYTES].decode("utf-8", errors="replace")
    else:
        label = f"LOG/TEXT TAIL (last {MAX_PREVIEW_BYTES} bytes of {len(data)}; display truncated)"
        preview = data[-MAX_PREVIEW_BYTES:].decode("utf-8", errors="replace")
    console.add(f"{label}: {path}\nsha256={entry['sha256']}\n{preview}\nEND FILE: {path}")


def _summary(receipt: dict) -> str:
    source = receipt["source"]
    lines = [
        f"### Compact CI evidence: {html.escape(receipt['name'])}",
        "",
        f"- Job status **before archiving**: <code>{html.escape(receipt['job_status_before_archive'])}</code>.",
        f"- Checked-out commit: <code>{source['checkout_sha'] or 'unavailable'}</code>.",
        f"- Event SHA: <code>{html.escape(source['github_sha'] or 'unavailable')}</code>.",
        f"- Retained: **{len(receipt['retained'])} report(s)**, {receipt['retained_bytes']} bytes.",
        f"- Omitted: **{receipt['omitted_count']} path(s)**; missing patterns: **{len(receipt['missing_patterns'])}**.",
        "- Receipt: <code>_meta/receipt.json</code>. Upload availability is reported separately.",
        "- This is compact report evidence; it is not a complete raw-image or release archive.",
    ]
    if not receipt["retained"]:
        lines += ["", "**WARNING: No eligible test evidence was found. The receipt is metadata only.**"]
    if receipt["omitted_list_truncated_count"]:
        lines += ["", f"Omitted-path listing truncated by {receipt['omitted_list_truncated_count']} entries; reason counts remain complete."]
    if receipt["warnings"]:
        lines += ["", "Warnings:", "<pre>" + html.escape("\n".join(receipt["warnings"][:20])) + "</pre>"]
    return "\n".join(lines) + "\n"


def collect(
    *, name: str, paths: str, output: Path, status: str,
    workspace: Path | None = None, environ: Mapping[str, str] | None = None,
    stdout: TextIO | None = None,
) -> dict:
    """Collect exact small report bytes; return the receipt, never a test verdict."""
    env = os.environ if environ is None else environ
    stream = sys.stdout if stdout is None else stdout
    root = (workspace or Path(env.get("GITHUB_WORKSPACE", os.getcwd()))).resolve(strict=True)
    if not root.is_dir():
        raise EvidenceError("Workspace must be a directory")
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._ -]{0,127}", name):
        raise EvidenceError("Evidence name must be a simple single-line artifact label")
    if status not in STATUSES:
        raise EvidenceError("Unknown pre-archive job status")
    destination = output if output.is_absolute() else root / output
    destination = Path(os.path.abspath(destination))
    if any(unicodedata.category(c) in {"Cc", "Cf", "Zl", "Zp"} for c in str(destination)):
        raise EvidenceError("Output path contains control characters")
    if destination == root or destination in root.parents:
        raise EvidenceError("Output must not replace the workspace or an ancestor")
    if any(parent.is_symlink() for parent in [destination, *destination.parents]):
        raise EvidenceError("Output path must not contain symlinks")
    if destination.exists() and (not destination.is_dir() or any(destination.iterdir())):
        raise EvidenceError("Output directory must be new or empty; stale evidence is refused")
    patterns = [line.strip() for line in paths.splitlines() if line.strip()]
    if len(patterns) > MAX_PATTERNS or any(len(p.encode("utf-8")) > 4096 for p in patterns):
        raise EvidenceError("Too many or oversized input patterns")
    selection = Selection(root, destination)
    selection.select(patterns)
    destination.mkdir(parents=True, exist_ok=True)
    retained = []
    previews = []
    total = 0
    for relative, path in sorted(selection.files.items(), key=lambda item: (Path(item[0]).suffix.lower() != ".json", item[0])):
        info = path.lstat()
        size = info.st_size
        if not stat.S_ISREG(info.st_mode):
            selection.omit(path, "changed-to-non-regular-file", size)
        elif len(relative.encode("utf-8")) > MAX_PATH_BYTES:
            selection.omit(path, "path-length-limit", size)
        elif Path(relative).suffix.lower() not in TEXT_SUFFIXES:
            selection.omit(path, "not-compact-report-type", size)
        elif size > MAX_FILE_BYTES:
            selection.omit(path, "per-file-byte-limit", size)
        elif len(retained) >= MAX_FILES - 1:
            selection.omit(path, "file-count-limit", size)
        elif total + size > MAX_TOTAL_BYTES - MAX_FILE_BYTES:
            selection.omit(path, "total-byte-limit", size)
        else:
            data = _read_report(root, relative)
            if len(data) > MAX_FILE_BYTES:
                selection.omit(path, "grew-over-per-file-limit", len(data))
                continue
            if total + len(data) > MAX_TOTAL_BYTES - MAX_FILE_BYTES:
                selection.omit(path, "grew-over-total-byte-limit", len(data))
                continue
            try:
                content = data.decode("utf-8")
            except UnicodeDecodeError:
                selection.omit(path, "not-utf8-text", len(data))
                continue
            if any(ord(char) < 32 and char not in "\t\r\n\x1b" for char in content):
                selection.omit(path, "binary-control-bytes", len(data))
                continue
            if SENSITIVE_CONTENT.search(content):
                selection.omit(path, "credential-like-content", len(data))
                continue
            target = destination / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            entry = {"path": relative, "size_bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
            retained.append(entry)
            previews.append((entry, data))
            total += len(data)
    checkout_sha = _checkout_sha(root)
    if checkout_sha is None:
        selection.warnings.append("Actual checkout SHA unavailable; do not infer it from GITHUB_SHA")
    if selection.missing:
        selection.warnings.append(f"{len(selection.missing)} input pattern(s) were missing, rejected or inaccessible via safe paths")
    if not retained:
        selection.warnings.append("No eligible test evidence found; receipt is metadata only, not a passing test")
    omitted = list(selection.omitted.values())
    receipt = {
        "schema_version": 1,
        "name": name,
        "collected_at": datetime.now(timezone.utc).isoformat(),
        "source": {
            "checkout_sha": checkout_sha,
            "github_sha": _prefix_bytes(env.get("GITHUB_SHA", ""), 256) or None,
            "run_id": _prefix_bytes(env.get("GITHUB_RUN_ID", ""), 256) or None,
            "run_attempt": _prefix_bytes(env.get("GITHUB_RUN_ATTEMPT", ""), 256) or None,
            "job": _prefix_bytes(env.get("GITHUB_JOB", ""), 256) or None,
        },
        "job_status_before_archive": status,
        "evidence_state": "compact-reports-retained" if retained else "no-evidence",
        "retained_bytes": total,
        "retained": retained,
        "omitted_count": len(omitted),
        "omitted_reason_counts": dict(sorted(Counter(item["reason"] for item in omitted).items())),
        "omitted": omitted,
        "missing_patterns": selection.missing,
        "warnings": selection.warnings,
        "limits": {"file_bytes": MAX_FILE_BYTES, "total_bytes_including_receipt": MAX_TOTAL_BYTES, "files_including_receipt": MAX_FILES},
        "scope": "Compact reports only; binary/raw-image retention and upload availability are separate.",
    }
    encoded = _receipt_bytes(receipt)
    metadata = destination / "_meta"
    metadata.mkdir(exist_ok=True)
    (metadata / "receipt.json").write_bytes(encoded)
    console = Console()
    console.add("RECEIPT SUMMARY\n" + json.dumps({
        "name": name, "source": receipt["source"], "job_status_before_archive": status,
        "evidence_state": receipt["evidence_state"], "retained_files": len(retained),
        "retained_bytes": total, "omitted_count": len(omitted),
        "omitted_reason_counts": receipt["omitted_reason_counts"],
        "omitted_list_truncated_count": receipt["omitted_list_truncated_count"],
        "missing_patterns": selection.missing, "warnings": selection.warnings,
    }, indent=2, ensure_ascii=False))
    for entry, data in previews:
        _preview(console, entry, data)
    console.emit(stream)
    if env.get("GITHUB_STEP_SUMMARY"):
        with open(env["GITHUB_STEP_SUMMARY"], "a", encoding="utf-8") as handle:
            handle.write(_summary(receipt))
    if env.get("GITHUB_OUTPUT"):
        with open(env["GITHUB_OUTPUT"], "a", encoding="utf-8") as handle:
            handle.write(f"path={destination}\nhas-files={'true' if retained else 'false'}\n")
    return receipt


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    command = commands.add_parser("collect", help="Collect bounded reports and a receipt")
    command.add_argument("--name", required=True)
    command.add_argument("--paths", required=True, help="Newline-separated explicit upload globs")
    command.add_argument("--output", type=Path, required=True)
    command.add_argument("--status", choices=sorted(STATUSES), required=True)
    args = parser.parse_args(argv)
    try:
        collect(name=args.name, paths=args.paths, output=args.output, status=args.status)
    except (EvidenceError, OSError) as error:
        sys.stderr.write(_inert(f"COLLECTION ERROR: {error}"))
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
