"""Independent Python oracle for the S09 ustar name check (S09-ORACLE).

Python does not run the Rust code. This module and the Rust test
`ustar_names_agree_with_the_shared_fixture` both read
services/volume/tests/ustar-names.txt, so the two implementations agree only
transitively through that fixture. The expected bits follow
docs/implementation/S09-BACKUP-ARCHIVE.md; CASE and FILE_AND_DIRECTORY are
pairwise and stay covered by the Rust tests alone.
"""

from __future__ import annotations

import itertools
import unicodedata
import unittest
from pathlib import Path

FIXTURE = Path(__file__).resolve().parents[1] / "services/volume/tests/ustar-names.txt"
CHARACTER, RESERVED, TRAILING, UNCHECKED = 1, 2, 4, 32
SET = [
    chr(code)
    for code in itertools.chain(range(0x20, 0x7F), range(0xA0, 0x180), range(0x2700, 0x27C0))
    if code != 0xAD
]
ALLOWED = frozenset(SET)
DEVICES = {"CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"}


def reasons(raw: bytes) -> int:
    """The single-name refusal bits, written from the specification."""
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError:
        return UNCHECKED
    bits = 0
    for c in text:
        if unicodedata.category(c) == "Cc" or c in '<>:"|?*/\\':
            bits |= CHARACTER
        elif c not in ALLOWED:
            bits |= UNCHECKED
    if text.endswith((".", " ")):
        bits |= TRAILING
    base = text.split(".", 1)[0].rstrip(" ")
    upper = "".join(c.upper() if c.isascii() else c for c in base)
    port = len(base) == 4 and upper[:3] in ("COM", "LPT") and base[3] in "0123456789¹²³"
    if port or upper in DEVICES:
        bits |= RESERVED
    return bits


def key(text: str) -> str:
    """Full Unicode upper case, except that U+0130 is also taken as "i"."""
    return text.replace("İ", "I").upper()


def collide(a: bytes, b: bytes) -> bool:
    def text(raw: bytes) -> str:
        try:
            return raw.decode("utf-8")
        except UnicodeDecodeError:
            return ""

    return key(text(a)) == key(text(b))


class SetTest(unittest.TestCase):
    def test_set_holds_no_combining_mark_or_control(self) -> None:
        self.assertEqual(len(SET), 510)
        for c in SET:
            self.assertFalse(unicodedata.category(c).startswith(("M", "C")), f"{c!r}")

    def test_nfd_is_injective_on_names_over_the_set_and_adds_no_fold_collision(self) -> None:
        # Each character is NFC and decomposes into one starter plus non-starters, so
        # NFD of a longer name is the concatenation of per-character blocks that
        # reordering cannot cross; injectivity on single characters then extends.
        decomposed: dict[str, str] = {}
        for c in SET:
            d = unicodedata.normalize("NFD", c)
            self.assertEqual(unicodedata.normalize("NFC", c), c)
            self.assertEqual(unicodedata.combining(d[0]), 0, f"{c!r}")
            self.assertTrue(all(unicodedata.combining(m) > 0 for m in d[1:]), f"{c!r}")
            self.assertNotIn(d, decomposed, f"{c!r} {decomposed.get(d)!r}")
            decomposed[d] = c
        # Exhaustive over names of one and two characters: NFD stays injective, and a
        # reader that normalises before folding merges no names the fold keeps apart.
        seen: dict[str, str] = {}
        folded: dict[str, str] = {}
        for name in itertools.chain(SET, map("".join, itertools.product(SET, repeat=2))):
            d = unicodedata.normalize("NFD", name)
            self.assertNotIn(d, seen, f"{name!r} {seen.get(d)!r}")
            seen[d] = name
            self.assertEqual(folded.setdefault(key(d), key(name)), key(name), f"{name!r}")


class FixtureTest(unittest.TestCase):
    def test_oracle_agrees_with_the_shared_fixture(self) -> None:
        counts = {"name": 0, "collide": 0, "distinct": 0}
        for line in FIXTURE.read_text(encoding="ascii").splitlines():
            fields = line.split("#", 1)[0].split()
            if not fields:
                continue
            kind, *rest = fields
            with self.subTest(line=line):
                if kind == "name":
                    bits, name = rest
                    self.assertEqual(reasons(bytes.fromhex(name)), int(bits))
                else:
                    a, b = (bytes.fromhex(field) for field in rest)
                    # The case key models the fold only on names over the checked set.
                    self.assertTrue(set(a.decode() + b.decode()) <= ALLOWED)
                    self.assertEqual(collide(a, b), kind == "collide", kind)
                counts[kind] += 1
        self.assertEqual(counts, {"name": 82, "collide": 15, "distinct": 10})


if __name__ == "__main__":
    unittest.main()
