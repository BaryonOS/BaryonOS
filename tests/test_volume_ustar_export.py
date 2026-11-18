"""Host checks for the archive-ustar oracle in tools/run_volume.py: it must
accept a POSIX ustar stream with pax path records only for non-ASCII paths and
reject every stream that differs from the archive it was converted from."""
import io
import sys
import tarfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import run_volume as rv  # noqa: E402

ARCHIVE = dict(collections={1: "Werk", 2: "Privé"}, objects=[
    dict(name="Notitie ✓", collection=2, content=b"n" * 129),
    dict(name="Leeg", collection=1, content=b""),
    dict(name="Concept", collection=0, content=bytes(range(256)) * 3),
])
WANT = [("Privé/Notitie ✓", b"n" * 129), ("Werk/Leeg", b""), ("Concept", bytes(range(256)) * 3)]


def stream(entries, pax_ascii=False, mtime=0, directory=False) -> bytes:
    """Python's own pax writer, cut after exactly two end blocks (it pads to 10 KiB)."""
    out = io.BytesIO()
    with tarfile.open(fileobj=out, mode="w", format=tarfile.PAX_FORMAT, encoding="utf-8") as tar:
        if directory:
            info = tarfile.TarInfo("Werk")
            info.type, info.mode = tarfile.DIRTYPE, 0o755
            tar.addfile(info)
        for path, content in entries:
            info = tarfile.TarInfo(path)
            info.size, info.mode, info.mtime = len(content), 0o644, mtime
            if pax_ascii:
                info.pax_headers = {"path": path}
            tar.addfile(info, io.BytesIO(content))
        end = tar.offset
    return out.getvalue()[:end + 2 * rv.SECTOR]


class UstarOracle(unittest.TestCase):
    def test_accepts_the_converted_archive(self):
        self.assertEqual(rv.check_ustar(stream(WANT), ARCHIVE), [path for path, _ in WANT])

    def test_rejects_streams_that_differ(self):
        good = stream(WANT)
        changed = bytearray(good)
        changed[good.index(b"n" * 129) + 7] ^= 1
        cases = {
            "missing end blocks": good[:-2 * rv.SECTOR],
            "extra zero block": good + bytes(rv.SECTOR),
            "renamed member": stream([WANT[0], WANT[1], ("Concept 2", WANT[2][1])]),
            "changed content": bytes(changed),
            "missing member": stream(WANT[:2]),
            "swapped order": stream([WANT[1], WANT[0], WANT[2]]),
            "pax for an ASCII path": stream(WANT, pax_ascii=True),
            "nonzero mtime": stream(WANT, mtime=1),
            "directory entry": stream(WANT, directory=True),
        }
        for label, data in cases.items():
            with self.subTest(label), self.assertRaises(ValueError):
                rv.check_ustar(data, ARCHIVE)


if __name__ == "__main__":
    unittest.main()
