# SPDX-License-Identifier: MPL-2.0
"""Flag active first-party Markdown that is still written mainly in Dutch.

Issue #274 requires English active documentation. This check uses a stopword
heuristic, the same method as the 11 November 2026 inventory on #274: prose
outside code fences is classified Dutch when it holds more than 20 Dutch
stopwords and more Dutch than half the English stopword count.

Dated evidence, history, audit snapshots, research notes and third-party
material are excluded; they are records, not rewritten in place. The check
finds regressions only. It does not establish language quality, runtime UI
localisation or completion of #274.
"""
from __future__ import annotations

import os
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

DUTCH = frozenset(
    "de het een en van is dat niet op te voor met zijn worden wordt naar ook "
    "bij als maar dit deze die geen nog om aan door kan moet zonder alleen".split()
)
ENGLISH = frozenset(
    "the a an and of is that not on to for with be are was this these no also "
    "by as but only without must can from".split()
)
MIN_DUTCH = 20

# Path prefixes for records and third-party text that are never rewritten.
EXCLUDED_PREFIXES = (
    "docs/verification/",
    "docs/history/",
    "docs/audit/",
    "docs/audits/",
    "docs/research/",
    "docs/third-party/",
    ".agents/skills/os-dev/references/",
    "experiments/quantum/third-party/",
    "services/keyring/vendor/",
    "services/image-codec/vendor/",
)
# Component evidence directories, for example drivers/usb/verification/.
EXCLUDED_SEGMENT = "/verification/"

# CommonMark fence line: up to three spaces, then three or more ` or ~.
_FENCE = re.compile(r" {0,3}(`{3,}|~{3,})(.*)$")
_WORD = re.compile(r"[a-z]+")


def is_excluded(path: str) -> bool:
    return path.startswith(EXCLUDED_PREFIXES) or EXCLUDED_SEGMENT in f"/{path}"


def prose(text: str) -> str:
    """Drop fenced code blocks; a closing fence must use the opening character,
    be at least as long and carry nothing but whitespace."""
    kept, fence = [], None
    for line in text.splitlines():
        match = _FENCE.match(line)
        if fence is None:
            # A backtick fence whose info string holds a backtick is not a fence.
            if match and not (match[1][0] == "`" and "`" in match[2]):
                fence = match[1]
            else:
                kept.append(line)
        elif match and match[1][0] == fence[0] and len(match[1]) >= len(fence) and not match[2].strip():
            fence = None
    return "\n".join(kept)


def counts(text: str) -> tuple[int, int]:
    words = _WORD.findall(prose(text).lower())
    return sum(w in DUTCH for w in words), sum(w in ENGLISH for w in words)


def is_dutch(text: str) -> bool:
    dutch, english = counts(text)
    return dutch > MIN_DUTCH and dutch > 0.5 * english


def git_env() -> dict[str, str]:
    """Environment without inherited GIT_* variables such as GIT_DIR, so `-C`
    alone selects the repository."""
    return {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}


def tracked_markdown(root: Path = ROOT) -> list[str]:
    out = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z", "--", "*.md"],
        check=True,
        capture_output=True,
        env=git_env(),
    ).stdout
    return sorted(p for p in out.decode("utf-8").split("\0") if p)


def find_dutch(root: Path = ROOT) -> list[str]:
    found = []
    for path in tracked_markdown(root):
        if is_excluded(path):
            continue
        file = root / path
        if not file.is_file():  # deleted in the working tree
            continue
        if is_dutch(file.read_text(encoding="utf-8", errors="replace")):
            found.append(path)
    return found

