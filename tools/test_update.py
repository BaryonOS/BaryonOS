#!/usr/bin/env python3
"""B03 gate: host tests, three kernel builds, hybrid TEST-key signing, then
eighteen real QEMU stages including key rotation and lifecycle rejections.
Only the final summary with passed=true counts."""
from __future__ import annotations
import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from common import BUILD, ROOT, environment, save_json, sha256, source_hash
from build import build
import run_update

TEST_INPUTS = "test_update_*.py"
HOST = [
    ["cargo", "+1.90.0", "test", "--locked", "-p", "update-core", "-p", "update-tool",
     "--features", "update-core/test-signing-anchors"],
    [sys.executable, "-B", "-m", "unittest", "discover", "-s", "tests", "-p", TEST_INPUTS],
]
KERNELS = {"k1": ("native-update", 1, "kernel-v1.elf"), "k2": ("native-update", 2, "kernel-v2.elf"),
           "kc": ("test-update-cut", 1, "kernel-v1-cut.elf")}
MANIFESTS = {"m1": ("k1", 1, "TEST", "manifest-v1.bin"), "m2": ("k2", 2, "TEST", "manifest-v2.bin"),
             "mc": ("kc", 1, "TEST", "manifest-v1-cut.bin"), "mf": ("k2", 2, "TEST-foreign", "manifest-v2-foreign.bin"),
             "mj": ("kj", 2, "TEST", "manifest-v2-junk.bin"),
             "mn": ("k2", 2, "TEST-next", "manifest-v2-next.bin"),
             "mr": ("k2", 2, "TEST-revoked", "manifest-v2-revoked.bin"),
             "me": ("k2", 2, "TEST-expired", "manifest-v2-expired.bin"),
             "mp": ("k1", 1, "TEST-next", "manifest-v1-premature.bin")}
# Correctly signed but not an ELF: the loader must reject it as FORMAT and
# fall back without raising the floor.
JUNK_IMAGE = ("kj", "junk-v2.bin", 200_000)

# ADR-B03K2: check the Rust boundary itself, including both loader implementations.
POLICY_GUARDS = [
    ("loader", "os-loader", ["-p", "os-loader", "--target", "x86_64-unknown-uefi"],
     "signed-boot-test", "signed-boot",
     "signed-boot requires an explicit signing policy; TEST anchors require signed-boot-test"),
    ("loader-legacy", "os-loader", ["-p", "os-loader", "--target", "x86_64-unknown-uefi"],
     "signed-boot-test,smp-handoff", "signed-boot,smp-handoff",
     "signed-boot requires an explicit signing policy; TEST anchors require signed-boot-test"),
    ("service", "update-service",
     ["--manifest-path", "sdk/Cargo.toml", "-p", "nieuw-os-update-example", "--bin", "update-service",
      "--target", "x86_64-unknown-none"],
     "native,test-signing-anchors", "native",
     "the update service requires an explicit signing policy; TEST anchors are test-only"),
]


def check_policy_guards(frozen: str) -> dict:
    """Cross-compilation only; a compiler failure alone never proves a guard."""
    report = {"schema_version": 1, "source_fingerprint_version": 3, "source_sha256": frozen,
              "passed": False, "checks": []}
    path = BUILD / "update-policy-guards.json"
    save_json(path, report)
    env = environment()
    for name, target, arguments, positive, negative, diagnostic in POLICY_GUARDS:
        for mode, features in [("positive", positive), ("negative", negative)]:
            command = ["cargo", "+1.90.0", "check", "--locked", "--no-default-features",
                       "--message-format=json", *arguments, "--features", features]
            try:
                result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True,
                                        timeout=300)
            except (OSError, subprocess.TimeoutExpired) as error:
                report["checks"].append({"name": f"{name}-{mode}", "command": command,
                                         "exit": None, "passed": False, "error": str(error)})
                save_json(path, report)
                return report
            errors = []
            for line in result.stdout.splitlines():
                try:
                    item = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if (item.get("reason") == "compiler-message"
                        and item.get("target", {}).get("name") == target
                        and item.get("message", {}).get("level") == "error"):
                    errors.append(item["message"]["message"])
            passed = (result.returncode == 0 if mode == "positive"
                      else result.returncode == 101 and diagnostic in errors)
            report["checks"].append({"name": f"{name}-{mode}", "command": command,
                                     "exit": result.returncode, "passed": passed,
                                     "expected_diagnostic": None if mode == "positive" else diagnostic,
                                     "stdout": result.stdout, "stderr": result.stderr})
            save_json(path, report)
            print(f"B03 policy guard {name}-{mode} passed={passed} exit={result.returncode}")
            if not passed:
                return report
    report["passed"] = source_hash() == frozen
    save_json(path, report)
    return report


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
        manifest = build("signed-boot-test", feature, f"update-{name}", image_version=version)
        kernel = ROOT / "target" / "x86_64-unknown-none/release/os-kernel"
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
        keys = str(foreign) if label == "TEST-foreign" else str(ROOT / "tools/update-keys")
        run_update.update_tool(env, "sign", "--keys", keys, "--label", label,
                               "--image", str(out / images["files"][kernel]), "--version", str(version),
                               "--out", str(out / filename))
        images["files"][name] = filename
        images["manifests"][name] = sha256(out / filename)
    save_json(run_update.IMAGES, images)
    return images


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--policy-guards-only", action="store_true",
                        help="check TEST-policy compiler guards without running QEMU")
    args = parser.parse_args()
    frozen = source_hash()
    tests = test_inputs()
    summary = {"schema_version": 1, "source_fingerprint_version": 3, "source_sha256": frozen,
               "test_inputs": tests, "status": "running", "passed": False, "host": [], "images": None,
               "stages": [], "policy_guards": None}
    path = BUILD / "update-gate.json"
    if not args.policy_guards_only:
        save_json(path, summary)
    guards = check_policy_guards(frozen)
    if args.policy_guards_only:
        return 0 if guards["passed"] else 1
    summary["policy_guards"] = guards
    save_json(path, summary)
    if not guards["passed"]:
        summary["status"] = "failed"
        save_json(path, summary)
        return 1
    for command in HOST:
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        totals = [line for line in (result.stdout + result.stderr).splitlines()
                  if line.startswith(("test result:", "Ran ", "OK", "FAILED"))]
        summary["host"].append({"command": command, "exit": result.returncode, "totals": totals,
                                "stdout": result.stdout, "stderr": result.stderr})
        if result.returncode != 0:
            summary["status"] = "failed"
            save_json(path, summary)
            print(f"host check failed: {' '.join(command)}")
            return 1
    summary["images"] = prepare_images(frozen)
    save_json(path, summary)
    states: dict[str, object] = {}
    for stage in run_update.ORDER:
        plan = run_update.STAGES[stage]
        state = None if plan["fresh"] else states[plan["sequence"]]
        report, state_dir = run_update.run_update(stage, state)
        states[plan["sequence"]] = state_dir
        summary["stages"].append({"stage": stage, "sequence": plan["sequence"], "passed": report["passed"],
                                  "run_directory": report["run_directory"],
                                  "state_directory": str(state_dir),
                                  "raw_qemu_exit": report["raw_qemu_exit"], "host_kill": report["host_kill"],
                                  "duration_s": report["duration_s"], "evidence": report.get("evidence"),
                                  "store_after_sha256": report["store_after_sha256"],
                                  "error": report.get("error")})
        save_json(path, summary)
        if not report["passed"]:
            summary["status"] = "failed"
            save_json(path, summary)
            return 1
    try:
        build()
    except Exception:
        summary["status"] = "failed"
        save_json(path, summary)
        raise
    summary.update(status="completed", passed=source_hash() == frozen and test_inputs() == tests)
    save_json(path, summary)
    print(f"B03 gate passed={summary['passed']} stages={len(summary['stages'])}")
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
