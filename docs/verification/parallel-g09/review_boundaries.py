"""Read-only G09 implementation review using isolated synthetic B07 fixtures.

Argument: directory containing immutable reviewed tools/ and tests/ snapshots.
Never accesses a real graphics run or starts QEMU.
"""
from pathlib import Path
import json
import os
import sys
from unittest.mock import patch

snapshot = Path(sys.argv[1]).resolve(strict=True)
sys.path.insert(0, str(snapshot / "tests"))
from test_graphics_capture import GraphicsCapture, c


def fixture():
    case = GraphicsCapture(methodName="runTest")
    case.setUp()
    return case


def changed_inode():
    case = fixture()
    try:
        path = case.run / "serial.txt"
        original_bytes, original_inode = path.read_bytes(), path.stat().st_ino
        original_encoder = c.encode_png
        replaced = False

        def encoder(frame):
            nonlocal replaced
            if not replaced:
                replacement = Path(case.temp.name) / "replacement-serial.txt"
                replacement.write_bytes(original_bytes)
                os.replace(replacement, path)
                replaced = True
            return original_encoder(frame)

        with patch.object(c, "encode_png", side_effect=encoder):
            try:
                case.capture()
            except ValueError as error:
                reason = str(error)
            else:
                raise AssertionError("Input inode replacement was accepted")
        assert original_bytes == path.read_bytes()
        assert original_inode != path.stat().st_ino
        assert not case.target.exists()
        assert not list((case.root / "docs/verification").iterdir())
        return {"rejected": True, "reason": reason, "same_bytes_new_inode": True}
    finally:
        case.doCleanups()


def extra_staged_file():
    case = fixture()
    try:
        original_encoder = c.encode_png
        injected = False
        payload = b"Synthetic unexpected staged bytes; not an OS artifact.\n"

        def encoder(frame):
            nonlocal injected
            if not injected:
                staged = next((case.root / "docs/verification").glob(".g0-capture-*"))
                (staged / "unexpected.bin").write_bytes(payload)
                injected = True
            return original_encoder(frame)

        with patch.object(c, "encode_png", side_effect=encoder):
            try:
                case.capture()
            except (ValueError, OSError) as error:
                assert not case.target.exists()
                return {"rejected": True, "reason": str(error), "published_extra_file": False}
        metadata = json.loads((case.target / "capture.json").read_text())
        assert (case.target / "unexpected.bin").read_bytes() == payload
        assert metadata["files"]["unexpected.bin"] == c.digest(payload)
        return {"rejected": False, "published_extra_file": True,
                "extra_file_in_capture_index": True, "published_files": sorted(p.name for p in case.target.iterdir())}
    finally:
        case.doCleanups()


print(json.dumps({"scope": "Two isolated synthetic publication-boundary probes; no native execution",
                  "same_bytes_input_replacement": changed_inode(),
                  "extra_staged_file_before_index": extra_staged_file()}, indent=2))
