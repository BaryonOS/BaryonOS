#!/usr/bin/env python3
"""Render the fixed 100-point OS roadmap; check by default, never run OS tests."""
from __future__ import annotations

import argparse
import datetime as dt
import html
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "docs/progress.json"
README_MARKERS = ("<!-- OS-PROGRESS:BEGIN -->", "<!-- OS-PROGRESS:END -->")
RUBRIC_MARKERS = ("<!-- OS-RUBRIC:BEGIN -->", "<!-- OS-RUBRIC:END -->")


def require(condition: bool, message: str) -> None:
    """Raise ValueError when a progress invariant is not satisfied."""
    if not condition:
        raise ValueError(message)


def integer(value: object, low: int, high: int) -> bool:
    """Return whether value is a non-boolean integer inside the inclusive range."""
    return type(value) is int and low <= value <= high


def label(value: object) -> bool:
    """Return whether value is safe for the compact Markdown progress tables."""
    return isinstance(value, str) and bool(value.strip()) and not any(c in value for c in "\n\r|<>")


def load_progress() -> dict:
    """Load and validate the canonical fixed 100-point progress register."""
    data = json.loads(DATA.read_text(encoding="utf-8"))
    require(data.get("schema_version") == 1, "Unknown progress.json schema version")
    require(integer(data.get("rubric_version"), 1, 1000), "Invalid rubric version")
    assessment = data["assessment"]
    require(dt.date.fromisoformat(assessment["date"]).isoformat() == assessment["date"], "Date must use YYYY-MM-DD")
    require(re.fullmatch(r"[0-9a-f]{40}", assessment["source_commit"]) is not None, "Full source commit is missing")
    for key in ("branch", "task", "summary"):
        require(label(assessment[key]), f"Invalid assessment field: {key}")
    require(assessment["previous_score"] is None or integer(assessment["previous_score"], 0, 100), "Invalid previous score")
    tracks = data["tracks"]
    require(isinstance(tracks, list) and bool(tracks), "No rubric tracks")
    identifiers: set[str] = set()
    total = 0
    for track in tracks:
        require(label(track["title"]), "Invalid track title")
        require(re.fullmatch(r"[A-Z]", track["id"]) is not None, "Invalid track ID")
        require(track["id"] not in identifiers, "Duplicate track ID")
        identifiers.add(track["id"])
        require(integer(track["points"], 1, 100), "Invalid track weight")
        subtotal = 0
        for gate in track["gates"]:
            require(re.fullmatch(track["id"] + r"[0-9]{2}", gate["id"]) is not None, "Invalid criterion ID")
            require(gate["id"] not in identifiers, "Duplicate criterion ID")
            identifiers.add(gate["id"])
            require(integer(gate["points"], 1, 100), "Invalid criterion weight")
            require(label(gate["title"]) and label(gate["acceptance"]), "Invalid criterion text")
            require(gate["status"] in ("open", "accepted"), "Status must be open or accepted")
            evidence = gate["evidence"]
            require(isinstance(evidence, list), "Evidence must be a list")
            require(gate["status"] != "accepted" or bool(evidence), f"{gate['id']}: no points without an evidence reference")
            for path in evidence:
                require(isinstance(path, str) and re.fullmatch(r"[A-Za-z0-9_./-]+", path) is not None, "Evidence must be a local repository path")
                target = (ROOT / path).resolve()
                require(target.is_relative_to(ROOT) and target.is_file(), f"Missing/invalid evidence file: {path}")
            subtotal += gate["points"]
        require(subtotal == track["points"], f"{track['id']}: criterion weights do not add up to the track weight")
        total += subtotal
    require(total == 100, f"Rubric totals {total}; exactly 100 required")
    return data


def score(track: dict) -> int:
    """Return accepted points for one product track."""
    return sum(g["points"] for g in track["gates"] if g["status"] == "accepted")


def total_score(data: dict) -> int:
    """Return accepted points across all product tracks."""
    return sum(score(track) for track in data["tracks"])


def assessment_delta(data: dict) -> str:
    """Describe the score change recorded by the current assessment."""
    previous = data["assessment"]["previous_score"]
    current = total_score(data)
    if previous is None:
        return f"Initial measurement: **{current}%**."
    delta = current - previous
    return f"This task: **{previous}% → {current}% ({delta:+d} percentage point)**."


def readme_block(data: dict) -> str:
    """Render the compact README progress section from the canonical rubric."""
    current = total_score(data)
    return f"""{README_MARKERS[0]}
## Progress toward a complete independent OS

![Baryon OS: {current}% of the fixed product roadmap](docs/assets/progress.svg)

**{current}/100 weighted acceptance points.** This is a planning metric for demonstrably accepted functionality, not a time estimate, security guarantee, or claim of parity with existing operating systems.

[Criteria and evidence](docs/PROGRESS.md) · [Roadmap](docs/ROADMAP.md) · [Current project status](docs/PROJECT-STATE.md)
{README_MARKERS[1]}"""

def rubric_block(data: dict) -> str:
    """Render the generated criteria register, including the current total score."""
    chunks = [RUBRIC_MARKERS[0], f"**Current score: {total_score(data)}/100.**", ""]
    for track in data["tracks"]:
        chunks.append(f"### {track['id']} — {track['title']} ({score(track)}/{track['points']})\n")
        chunks.append("| Criterion | Points | Status | Acceptance and evidence |\n|---|---:|---|---|")
        for gate in track["gates"]:
            state = "Accepted" if gate["status"] == "accepted" else "Open"
            links = " · ".join(f"[evidence {i + 1}](../{path})" for i, path in enumerate(gate["evidence"]))
            acceptance = gate["acceptance"] + (" " + links if links else "")
            chunks.append(f"| **{gate['id']} — {gate['title']}** | {gate['points']} | {state} | {acceptance} |")
        chunks.append("")
    chunks.append(RUBRIC_MARKERS[1])
    return "\n".join(chunks)


def svg(data: dict) -> str:
    """Render the local progress graphic from the canonical rubric."""
    current = total_score(data)
    date = html.escape(data["assessment"]["date"])
    width = 828 * current / 100
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 960 226" role="img" aria-labelledby="title description">
  <title id="title">Baryon OS: {current}% of the fixed product roadmap</title>
  <desc id="description">{current} of 100 weighted acceptance points. Zero means not started; one hundred means a complete independent OS on supported hardware. Assessed on {date}. Not a time or security score.</desc>
  <rect width="960" height="226" rx="16" fill="#f4f1eb"/>
  <text x="48" y="40" fill="#45504c" font-family="system-ui,sans-serif" font-size="14" font-weight="600" letter-spacing="2">BARYON OS · INDEPENDENT KERNEL · INDEPENDENT PRODUCT</text>
  <text x="48" y="104" fill="#173e34" font-family="system-ui,sans-serif" font-size="56" font-weight="700">{current}%</text>
  <text x="238" y="78" fill="#173e34" font-family="system-ui,sans-serif" font-size="21" font-weight="600">From independent kernel to daily-use OS</text>
  <text x="238" y="105" fill="#45504c" font-family="system-ui,sans-serif" font-size="15">{current} / 100 acceptance points · reassessed {date}</text>
  <rect x="66" y="136" width="828" height="19" rx="9.5" fill="#d5ddd5"/>
  <rect x="66" y="136" width="{width:.2f}" height="19" rx="9.5" fill="#207459"/>
  <g fill="#45504c" font-family="system-ui,sans-serif" font-size="13" text-anchor="middle">
    <text x="66" y="177">0%</text><text x="273" y="177">25%</text><text x="480" y="177">50%</text><text x="687" y="177">75%</text><text x="894" y="177">100%</text>
  </g>
  <text x="48" y="207" fill="#45504c" font-family="system-ui,sans-serif" font-size="13">Not started</text>
  <text x="912" y="207" fill="#45504c" font-family="system-ui,sans-serif" font-size="13" text-anchor="end">Complete independent OS</text>
</svg>
'''


def replace_block(path: Path, markers: tuple[str, str], block: str) -> str:
    """Replace exactly one generated marker block in a UTF-8 text file."""
    text = path.read_text(encoding="utf-8")
    begin, end = markers
    require(text.count(begin) == 1 and text.count(end) == 1, f"{path.name}: expected exactly one marker pair")
    start = text.index(begin)
    stop = text.index(end)
    require(start < stop, f"{path.name}: reversed markers")
    return text[:start] + block + text[stop + len(end):]


def main() -> int:
    """Check or update the generated progress views."""
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="Check rubric, evidence references, and generated views (default)")
    mode.add_argument("--write", action="store_true", help="Update README, criteria table, and local SVG")
    args = parser.parse_args()
    try:
        data = load_progress()
        outputs = {
            ROOT / "README.md": replace_block(ROOT / "README.md", README_MARKERS, readme_block(data)),
            ROOT / "docs/PROGRESS.md": replace_block(ROOT / "docs/PROGRESS.md", RUBRIC_MARKERS, rubric_block(data)),
            ROOT / "docs/assets/progress.svg": svg(data),
        }
        stale = [path for path, content in outputs.items() if not path.is_file() or path.read_text(encoding="utf-8") != content]
        if args.write:
            for path in stale:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(outputs[path], encoding="utf-8")
        elif stale:
            print("Stale progress view: " + ", ".join(str(p.relative_to(ROOT)) for p in stale), file=sys.stderr)
            print("Review docs/progress.json and run tools/progress.py --write.", file=sys.stderr)
            return 1
        print(f"Baryon OS: {total_score(data)}/100 points ({total_score(data)}%); rubric v{data['rubric_version']}; {'updated' if args.write else 'consistent'}.")
        print("This checks calculation, references, and generated views; it does not run native tests or assess evidence content.")
        return 0
    except (ValueError, KeyError, TypeError, OSError) as exc:
        print(f"Progress check rejected: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
