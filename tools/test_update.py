#!/usr/bin/env python3
"""Complete B03 gate: host tests, three kernel builds (v1, v2, v1 with the
labelled cut fixture) behind the signed-boot loader, TEST-key signing with
update-tool (plus one signed non-ELF image and one foreign key pair), then
thirteen real QEMU stages in nine store/variable sequences on one frozen
source fingerprint. Only the final summary with passed=true counts."""
from __future__ import annotations
import hashlib
import json
import shutil
import subprocess
import sys
from common import BUILD, ROOT, TARGET, environment, save_json, sha256, source_hash
from build import build
import run_update
from update_paths import portable
from run_update import ORDER, STAGES

TEST_INPUTS = "test_update_*.py"
HOST = [
    ["cargo", "+1.90.0", "test", "--locked", "-p", "update-core", "-p", "update-tool"],
    ["cargo", "+1.90.0", "test", "--locked", "--manifest-path", "sdk/Cargo.toml", "-p", "nieuw-os-update-example", "--lib"],
    [sys.executable, "-B", "-m", "unittest", "discover", "-s", "tests", "-p", TEST_INPUTS],
]
KERNELS = {"k1": ("native-update", 1, "kernel-v1.elf"), "k2": ("native-update", 2, "kernel-v2.elf"),
           "kc": ("test-update-cut", 1, "kernel-v1-cut.elf")}
MANIFESTS = {"m1": ("k1", 1, "TEST", "manifest-v1.bin"), "m2": ("k2", 2, "TEST", "manifest-v2.bin"),
             "mc": ("kc", 1, "TEST", "manifest-v1-cut.bin"), "mf": ("k2", 2, "TEST-foreign", "manifest-v2-foreign.bin"),
             "mj": ("kj", 2, "TEST", "manifest-v2-junk.bin")}
# Correctly signed but not an ELF: the loader must reject it as FORMAT and
# fall back without raising the floor.
JUNK_IMAGE = ("kj", "junk-v2.bin", 200_000)


def test_inputs() -> dict[str, str]:
    tests = ROOT / "tests"
    paths = set(tests.rglob(TEST_INPUTS)) | set(tests.rglob("__init__.py"))
    return {path.relative_to(ROOT).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(paths)}


def prepare_images(frozen: str) -> dict:
    """Build and sign every image the stages need; record hashes."""
    out = BUILD / "update"
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)
    env = environment()
    images = {"schema_version": 1, "source_sha256": frozen, "files": {}, "kernels": {}, "manifests": {},
              "loader_sha256": None, "builds": {}}
    for name, (feature, version, filename) in KERNELS.items():
        manifest = build("signed-boot", feature, f"update-{name}", image_version=version, test_anchors=True)
        kernel = TARGET / "x86_64-unknown-none/release/os-kernel"
        shutil.copy2(kernel, out / filename)
        images["files"][name] = filename
        images["kernels"][name] = sha256(out / filename)
        images["builds"][name] = {"kernel_feature": feature, "image_version": version,
                                  "kernel_sha256": manifest["kernel"]["sha256"],
                                  "service": manifest.get("update")}
        if images["loader_sha256"] is None:
            images["loader_sha256"] = manifest["loader"]["sha256"]
        elif images["loader_sha256"] != manifest["loader"]["sha256"]:
            raise ValueError("the signed-boot loader differed between builds")
    name, filename, length = JUNK_IMAGE
    (out / filename).write_bytes(bytes((index * 7 + 3) % 251 for index in range(length)))
    images["files"][name] = filename
    images["kernels"][name] = sha256(out / filename)
    foreign = out / "foreign-keys"
    run_update.update_tool(env, "keygen", "--dir", str(foreign), "--label", "TEST-foreign")
    for name, (kernel, version, label, filename) in MANIFESTS.items():
        keys = str(ROOT / "tools/update-keys") if label == "TEST" else str(foreign)
        run_update.update_tool(env, "sign", "--keys", keys, "--label", label,
                               "--image", str(out / images["files"][kernel]), "--version", str(version),
                               "--out", str(out / filename))
        images["files"][name] = filename
        images["manifests"][name] = sha256(out / filename)
    save_json(run_update.IMAGES, images)
    return images


def main() -> int:
    frozen = source_hash()
    tests = test_inputs()
    summary = {"schema_version": 1, "source_fingerprint_version": 3, "source_sha256": frozen,
               "test_inputs": tests, "status": "running", "passed": False, "host": [], "images": None,
               "stages": []}
    path = BUILD / "update-gate.json"
    save_json(path, portable(summary))
    for command in HOST:
        result = subprocess.run(command, cwd=ROOT, env=environment(), capture_output=True, text=True)
        totals = [line for line in (result.stdout + result.stderr).splitlines()
                  if line.startswith(("test result:", "Ran ", "OK", "FAILED"))]
        summary["host"].append({"command": command, "exit": result.returncode, "totals": totals,
                                "stdout": result.stdout, "stderr": result.stderr})
        if result.returncode != 0:
            summary["status"] = "failed"
            save_json(path, portable(summary))
            print(f"host check failed: {' '.join(command)}")
            return 1
    summary["images"] = prepare_images(frozen)
    save_json(path, portable(summary))
    states: dict[str, object] = {}
    for stage in ORDER:
        plan = STAGES[stage]
        state = None if plan["fresh"] else states[plan["sequence"]]
        report, state_dir = run_update.run_update(stage, state)
        states[plan["sequence"]] = state_dir
        summary["stages"].append({"stage": stage, "sequence": plan["sequence"], "passed": report["passed"],
                                  "run_directory": report["run_directory"],
                                  "state_directory": state_dir.relative_to(ROOT).as_posix(),
                                  "raw_qemu_exit": report["raw_qemu_exit"], "host_kill": report["host_kill"],
                                  "duration_s": report["duration_s"], "evidence": report.get("evidence"),
                                  "store_after_sha256": report["store_after_sha256"],
                                  "error": report.get("error")})
        save_json(path, portable(summary))
        if not report["passed"]:
            summary["status"] = "failed"
            save_json(path, portable(summary))
            return 1
    try:
        build()
    except Exception:
        summary["status"] = "failed"
        save_json(path, portable(summary))
        raise
    summary.update(status="completed", passed=source_hash() == frozen and test_inputs() == tests)
    save_json(path, portable(summary))
    print(f"B03 gate passed={summary['passed']} stages={len(summary['stages'])}")
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
