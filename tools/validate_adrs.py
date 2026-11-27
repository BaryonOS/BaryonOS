#!/usr/bin/env python3
"""Advisory validation for individually stored architecture decisions."""

import re
from datetime import datetime
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ID = r"ADR-[A-Z0-9]+(?:-[A-Z0-9]+)*-[0-9]{8}"
HEADING = re.compile(rf"^# ({ID}) — \S.*$")
HISTORICAL = re.compile(r"^## (ADR-[A-Za-z0-9-]+)(?:\s|:|$)", re.MULTILINE)


def validate(root: Path) -> list[str]:
    errors: list[str] = []
    historical = set(HISTORICAL.findall((root / "docs/DECISIONS.md").read_text(encoding="utf-8")))
    directory = root / "docs/decisions"
    if not directory.exists():
        return errors
    seen: set[str] = set()
    for path in sorted(directory.rglob("*")):
        if path.is_dir():
            continue
        if path == directory / "README.md":
            continue
        if path.parent != directory or path.suffix != ".md":
            errors.append(f"{path}: ADR files must be direct .md children of docs/decisions")
            continue
        text = path.read_text(encoding="utf-8")
        lines = text.splitlines()
        heading = HEADING.fullmatch(lines[0]) if lines else None
        if not heading or heading.group(1) != path.stem:
            errors.append(f"{path}: filename and first heading must match a dated ADR ID")
            continue
        identifier = heading.group(1)
        try:
            datetime.strptime(identifier[-8:], "%Y%m%d")
        except ValueError:
            errors.append(f"{path}: invalid ADR date")
        if identifier in seen or identifier in historical:
            errors.append(f"{path}: duplicate ADR ID {identifier}")
        seen.add(identifier)
        outside_fences: list[str] = []
        fence_char = ""
        fence_length = 0
        in_comment = False
        for line in lines:
            if fence_char:
                fence = re.match(r"^ {0,3}(`{3,}|~{3,})[ \t]*$", line)
                if fence and fence.group(1)[0] == fence_char and len(fence.group(1)) >= fence_length:
                    fence_char, fence_length = "", 0
                continue
            visible = ""
            remaining = line
            while remaining:
                if in_comment:
                    _, end_marker, remaining = remaining.partition("-->")
                    if not end_marker:
                        break
                    in_comment = False
                else:
                    before, start_marker, remaining = remaining.partition("<!--")
                    visible += before
                    if not start_marker:
                        break
                    in_comment = True
            fence = re.match(r"^ {0,3}(`{3,}|~{3,})", visible)
            if fence:
                marker = fence.group(1)
                fence_char, fence_length = marker[0], len(marker)
            else:
                outside_fences.append(visible)
        for field in ("Status", "Decision", "Rollback"):
            if not any(line.startswith(f"**{field}:**") for line in outside_fences):
                errors.append(f"{path}: missing {field}")
    return errors


def main() -> int:
    errors = validate(ROOT)
    for error in errors:
        print(error, file=sys.stderr)
    if errors:
        return 1
    print("ADR files: valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
