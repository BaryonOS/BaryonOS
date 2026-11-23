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

import re
import subprocess
import sys
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

_FENCE = re.compile(r"```.*?```", re.S)
_WORD = re.compile(r"[a-z]+")


def is_excluded(path: str) -> bool:
    return path.startswith(EXCLUDED_PREFIXES) or EXCLUDED_SEGMENT in f"/{path}"


def counts(text: str) -> tuple[int, int]:
    words = _WORD.findall(_FENCE.sub("", text).lower())
    return sum(w in DUTCH for w in words), sum(w in ENGLISH for w in words)


def is_dutch(text: str) -> bool:
    dutch, english = counts(text)
    return dutch > MIN_DUTCH and dutch > 0.5 * english


def tracked_markdown(root: Path = ROOT) -> list[str]:
    out = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z", "--", "*.md"],
        check=True,
        capture_output=True,
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


def main() -> int:
    found = find_dutch()
    for path in found:
        print(f"DUTCH {path}")
    print(f"english-docs: {len(found)} active Markdown file(s) classified Dutch")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
