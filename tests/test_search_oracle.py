"""Host oracle for the U05-T3 search adapter (no QEMU; spec N8, decision m8).

The helpers re-implement the search semantics of sdk/search-index independently
of the Rust code: Rust ``str::split_whitespace`` (Unicode White_Space) and
ASCII-only case folding on UTF-8 bytes. Python's ``str.split`` and ``lower``
differ (U+001C-U+001F, non-ASCII folding), so neither is used. The tests check
the helpers against the shared vectors that the Rust host tests also read.
"""
from __future__ import annotations

import unittest
from pathlib import Path

VECTORS = Path(__file__).resolve().parents[1] / "sdk/examples/search-volume/vectors.tsv"

# Unicode White_Space, which is exactly Rust's char::is_whitespace.
WHITESPACE = frozenset(
    [0x9, 0xA, 0xB, 0xC, 0xD, 0x20, 0x85, 0xA0, 0x1680, *range(0x2000, 0x200B),
     0x2028, 0x2029, 0x202F, 0x205F, 0x3000]
)
FNV_OFFSET = 0xCBF29CE484222325
FNV_PRIME = 0x100000001B3
MASK = (1 << 64) - 1


def tokens(field: bytes) -> list[bytes]:
    """Rust split_whitespace over one UTF-8 field."""
    out, current = [], []
    for char in field.decode("utf-8"):
        if ord(char) in WHITESPACE:
            if current:
                out.append("".join(current).encode())
            current = []
        else:
            current.append(char)
    if current:
        out.append("".join(current).encode())
    return out


def fold(token: bytes) -> bytes:
    """ASCII-only case folding (bytes.lower leaves non-ASCII bytes unchanged)."""
    return token.lower()


def matches(fields: list[bytes], query: bytes) -> bool:
    """AND over query tokens; each must equal some field token after folding."""
    have = {fold(t) for field in fields for t in tokens(field)}
    wanted = tokens(query)
    if not wanted:
        raise ValueError("empty query")
    return all(fold(t) in have for t in wanted)


def fnv(value: int, data: bytes) -> int:
    for byte in data:
        value = ((value ^ byte) * FNV_PRIME) & MASK
    return value


def digest(value: int, hits: list[tuple[int, int]]) -> int:
    """Same as search_volume::digest: sorted (object, revision) as LE u64."""
    for obj, rev in sorted(hits):
        value = fnv(value, obj.to_bytes(8, "little") + rev.to_bytes(8, "little"))
    return value


def load(path: Path = VECTORS) -> dict[str, list[list[str]]]:
    # split("\n"), not splitlines(): the latter also breaks on U+2028/U+0085.
    rows: dict[str, list[list[str]]] = {}
    for line in path.read_text(encoding="utf-8").split("\n"):
        if not line or line.startswith("#"):
            continue
        kind, *fields = [f for f in line.split("\t") if not f.startswith("#")]
        rows.setdefault(kind, []).append(fields)
    return rows


def hexlist(field: str) -> list[bytes]:
    return [] if field == "-" else [bytes.fromhex(item) for item in field.split(",")]


def expected(rows: dict[str, list[list[str]]]) -> tuple[list[list[int]], int]:
    """Hit objects per query and the chained digest for the record set."""
    labels = {int(c): bytes.fromhex(text) for c, text in rows["label"]}
    records = [
        (int(o), int(r), [bytes.fromhex(n), labels.get(int(c), b"")])
        for o, r, n, c in rows["record"]
    ]
    value, hits = FNV_OFFSET, []
    for text, _ in rows["query"]:
        found = [(o, r) for o, r, fields in records if matches(fields, bytes.fromhex(text))]
        hits.append([o for o, _ in found])
        value = digest(value, found)
    return hits, value


class OracleTests(unittest.TestCase):
    def setUp(self) -> None:
        self.rows = load()

    def test_whitespace_set_matches_vectors(self) -> None:
        (listed,), = self.rows["ws"]
        self.assertEqual(WHITESPACE, {int(c, 16) for c in listed.split(",")})

    def test_tokenisation_vectors(self) -> None:
        for field, toks in self.rows["token"]:
            with self.subTest(field=field):
                self.assertEqual(tokens(bytes.fromhex(field)), hexlist(toks))

    def test_python_split_differs_where_documented(self) -> None:
        self.assertNotEqual("a\x1fb".split(), [t.decode() for t in tokens(b"a\x1fb")])
        self.assertEqual("É".lower(), "é")
        self.assertEqual(fold("É".encode()), "É".encode())
        self.assertEqual(fold("K".encode()), "K".encode())

    def test_query_vectors_and_digest(self) -> None:
        hits, value = expected(self.rows)
        for (text, objects), found in zip(self.rows["query"], hits):
            with self.subTest(query=bytes.fromhex(text).decode()):
                self.assertEqual(found, [int(o) for o in hexlist_objects(objects)])
        (stored,), = self.rows["digest"]
        self.assertEqual(value, int(stored, 16))

    def test_capacity_fits_token_bound(self) -> None:
        labels = {int(c): bytes.fromhex(t) for c, t in self.rows["label"]}
        for _, _, name, c in self.rows["record"]:
            fields = [bytes.fromhex(name), labels.get(int(c), b"")]
            self.assertLessEqual(sum(map(len, fields)), 64)
            self.assertLessEqual(sum(len(tokens(f)) for f in fields), 32)
        self.assertIn(32, [sum(len(tokens(f)) for f in [bytes.fromhex(n), labels.get(int(c), b"")])
                           for _, _, n, c in self.rows["record"]])

    def test_negative_helpers(self) -> None:
        with self.assertRaises(ValueError):
            matches([b"a"], "　 ".encode())
        with self.assertRaises(UnicodeDecodeError):
            tokens(b"\xff")
        self.assertNotEqual(digest(FNV_OFFSET, [(0, 1)]), digest(FNV_OFFSET, [(0, 2)]))
        self.assertEqual(digest(FNV_OFFSET, [(1, 1), (0, 2)]), digest(FNV_OFFSET, [(0, 2), (1, 1)]))


def hexlist_objects(field: str) -> list[str]:
    return [] if field == "-" else field.split(",")


if __name__ == "__main__":
    unittest.main()
