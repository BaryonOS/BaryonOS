#!/usr/bin/env python3
"""Compare two isolated default builds on this host, without booting QEMU.

The default build exports an ESP directory for QEMU's virtual FAT backend, not
a disk image. image_payload hashes its canonical path/content-hash manifest;
this does not compare FAT metadata or prove byte-identical physical disk images.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import stat
from pathlib import Path
import subprocess
import sys
import tempfile

from common import BUILD, ROOT, save_json, sha256, source_hash
from package_windows_qemu import check_esp, load_json, MAX_FIRMWARE_BYTES


def build_once(build: Path, target: Path) -> None:
    env = dict(os.environ, NOS_BUILD_DIR=str(build), NOS_TARGET_DIR=str(target),
               CARGO_TARGET_DIR=str(target))
    log = build / "repro-build.log"
    with log.open("wb") as output:
        result = subprocess.run([sys.executable, "-B", str(ROOT / "tools/build.py")],
                                cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT,
                                check=False)
    if result.returncode:
        raise ValueError(f"build exit {result.returncode}; log: {log}")


def artifacts(build: Path, target: Path, source: str) -> dict[str, str]:
    manifest = load_json(build / "manifest.json", "build manifest")
    if (manifest.get("source_sha256") != source
            or manifest.get("kernel_feature") is not None
            or manifest.get("loader_feature") is not None
            or manifest.get("sdk_runtime") or manifest.get("document_session")):
        raise ValueError("build is not the frozen default profile")
    paths = {"kernel": target / "x86_64-unknown-none/release/os-kernel",
             "loader": target / "x86_64-unknown-uefi/release/os-loader.efi"}
    hashes = {}
    for name, path in paths.items():
        details = path.lstat()
        # Cargo may hardlink its final output to deps/; these files are only read.
        if not stat.S_ISREG(details.st_mode) or details.st_size > MAX_FIRMWARE_BYTES:
            raise ValueError(f"{name} is not a bounded regular artifact")
        hashes[name] = sha256(path)
    for name, digest in hashes.items():
        if digest != manifest[name]["sha256"]:
            raise ValueError(f"{name} differs from build manifest")
    esp = check_esp(build, manifest)
    hashes["image_payload"] = hashlib.sha256(
        json.dumps(esp, sort_keys=True, separators=(",", ":")).encode("ascii")).hexdigest()
    return hashes


def compare(output: Path, *, builder=build_once, fingerprint=source_hash) -> dict:
    source = fingerprint()
    runs = []
    for label in ("a", "b"):
        build, target = output / label / "build", output / label / "target"
        # Never overwrite an earlier run, another checkout or an existing cache.
        build.mkdir(parents=True)
        target.mkdir()
        builder(build, target)
        if fingerprint() != source:
            raise ValueError("source changed during comparison")
        runs.append({"build": str(build), "target": str(target),
                     "sha256": artifacts(build, target, source)})
    differences = [name for name in runs[0]["sha256"]
                   if runs[0]["sha256"][name] != runs[1]["sha256"][name]]
    return {"schema_version": 1, "source_sha256": source,
            "image_kind": "canonical ESP payload manifest, not a FAT disk image",
            "runs": runs, "differences": differences, "passed": not differences}


def main() -> int:
    argparse.ArgumentParser(description=__doc__).parse_args()
    parent = BUILD / "repro-check"
    parent.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix="run-", dir=parent))
    try:
        report = compare(output)
        code = 0 if report["passed"] else 1
    except (OSError, ValueError, KeyError, TypeError) as error:
        report = {"schema_version": 1, "passed": False, "error": str(error)}
        code = 2
    save_json(output / "result.json", report)
    print(json.dumps({"report": str(output / "result.json"), **report}, sort_keys=True))
    return code


if __name__ == "__main__":
    raise SystemExit(main())
