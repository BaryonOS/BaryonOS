#!/usr/bin/env python3
"""Verify the exact reviewed JPEG source/patch before native compilation."""
from pathlib import Path
import hashlib
import json
import tarfile
import difflib

BASE = Path(__file__).resolve().parent


def verify(base: Path = BASE) -> dict:
    proof = json.loads((base / "vendor/upstream/source-verification.json").read_text())
    archive = base / "vendor/upstream/zune-jpeg-0.5.16-rc3.crate"
    if hashlib.sha256(archive.read_bytes()).hexdigest() != proof["archive_sha256"]:
        raise ValueError("upstream JPEG archive checksum mismatch")
    original = {}
    with tarfile.open(archive, "r:gz") as tar:
        for member in tar.getmembers():
            if member.isfile():
                parts = Path(member.name).parts
                if not parts or ".." in parts or Path(member.name).is_absolute():
                    raise ValueError("unsafe upstream archive path")
                original["/".join(parts[1:])] = tar.extractfile(member).read()
    vendor = base / "vendor/zune-jpeg-0.5.16-rc3"
    paths = list(vendor.rglob("*"))
    if any(path.is_symlink() for path in paths):
        raise ValueError("vendored JPEG source symlink refused")
    actual = {path.relative_to(vendor).as_posix(): path.read_bytes()
              for path in paths if path.is_file()}
    if set(original) != set(actual) or set(actual) != set(proof["files"]):
        raise ValueError("vendored JPEG source file set changed")
    for name, data in actual.items():
        expected = proof["files"][name]
        if hashlib.sha256(data).hexdigest() != expected["local_sha256"]:
            raise ValueError(f"vendored JPEG source changed: {name}")
        if hashlib.sha256(original[name]).hexdigest() != expected["upstream_sha256"]:
            raise ValueError("upstream file hash mismatch")
    changed = sorted(name for name in original if original[name] != actual[name])
    snapshot_files = ["src/decoder.rs", "src/errors.rs", "src/mcu.rs"]
    if changed != snapshot_files + ["src/upsampler/scalar.rs"]:
        raise ValueError("unexpected vendored JPEG changes")
    name = "src/upsampler/scalar.rs"
    before, after = original[name], actual[name]
    start, end = b"pub fn upsample_hv(", b"pub fn upsample_generic("
    if (before[:before.index(start)] != after[:after.index(start)] or
            before[before.index(end):] != after[after.index(end):]):
        raise ValueError("JPEG change outside reviewed upsample_hv function")
    patch = base / "vendor/upstream/upsample-hv.patch"
    if hashlib.sha256(patch.read_bytes()).hexdigest() != proof["patch_sha256"]:
        raise ValueError("JPEG patch checksum mismatch")
    snapshot_patch = base / "vendor/upstream/scan-header-heap.patch"
    if hashlib.sha256(snapshot_patch.read_bytes()).hexdigest() != proof["snapshot_patch_sha256"]:
        raise ValueError("JPEG scan snapshot patch checksum mismatch")
    exact_diff = "".join("".join(difflib.unified_diff(
        original[name].decode().splitlines(keepends=True),
        actual[name].decode().splitlines(keepends=True),
        fromfile="a/" + name, tofile="b/" + name)) for name in snapshot_files).encode()
    if exact_diff != snapshot_patch.read_bytes():
        raise ValueError("JPEG snapshot patch does not describe the exact source delta")
    return {"verified": True, "changed_files": changed,
            "unchanged_files": len(original) - len(changed), "patch_sha256": proof["patch_sha256"],
            "snapshot_patch_sha256": proof["snapshot_patch_sha256"]}


if __name__ == "__main__":
    print(json.dumps(verify()))
