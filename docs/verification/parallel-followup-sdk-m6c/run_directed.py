#!/usr/bin/env python3
"""G10: exactly one existing native normal case and its cold recovery.

This directed regression does not claim the full 35-group M6c gate. It calls
the unmodified project build/runner and complete per-run capture validator.
Only its private build directory receives artifacts. No VM runs before the
explicit frozen HEAD/source arguments have been checked.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys


def require(condition, message):
    if not condition:
        raise ValueError(message)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--expected-head", required=True)
    parser.add_argument("--expected-source", required=True)
    args = parser.parse_args()
    root = args.root.resolve(strict=True)
    require(re.fullmatch("[0-9a-f]{40}", args.expected_head), "Invalid frozen HEAD")
    require(re.fullmatch("[0-9a-f]{64}", args.expected_source), "Invalid frozen source hash")
    sys.path.insert(0, str(root / "tools"))
    from common import ROOT, BUILD, TARGET, environment, save_json, sha256, source_hash
    from build import build
    from capture_m6c import check_run
    from run_m6c import run_m6c

    require(ROOT == root and BUILD == root / "build/parallel-followup-sdk-m6c",
            "Use the G10 private build directory inside this checkout")
    require(TARGET == Path("/tmp/nieuw-os-followup-sdk-m6c-target")
            and os.environ.get("CARGO_TARGET_DIR") == str(TARGET), "Use the private native target directory")
    require(os.environ.get("NOS_SDK_RUNTIME") == "1"
            and os.environ.get("CARGO_BUILD_JOBS") == "2", "Require SDK bridge and bounded build parallelism")
    output = BUILD / "directed-result.json"
    require(not output.exists(), "Preserve any earlier G10 result; do not replay this launcher")
    report = dict(schema_version=1, task="G10", status="preparing", passed=False,
        scope="Directed native SDK normal/cold-recovery regression, not the full M6c acceptance suite",
        source_commit=args.expected_head, source_fingerprint_version=3,
        source_sha256=args.expected_source, started_utc=datetime.now(timezone.utc).isoformat(),
        launcher_sha256=sha256(Path(__file__)), command=sys.argv, cwd=str(Path.cwd()),
        build_directory=str(BUILD), target_directory=str(TARGET),
        environment={key: os.environ.get(key) for key in ("NOS_SDK_RUNTIME", "NOS_BUILD_DIR", "NOS_TARGET_DIR",
            "CARGO_TARGET_DIR", "CARGO_HOME", "CARGO_BUILD_JOBS", "NOS_QEMU_MACHINE", "OS_OVMF_CODE", "OS_OVMF_VARS")},
        freeze_checks=[], runs=[], sdk_bindings=[], build_snapshots=[], real_vm_invocations=0,
        limitations=["Only existing normal and cold-recovery cases are repeated; prior full-suite evidence retains its own source binding.",
            "Normal IPC/yield/ABI/cap-close use the SDK bridge; block I/O and deliberately invalid ABI requests retain the original fixture path.",
            "The native acceptance roles are synthetic test programs operating on a real guarded disposable QEMU disk.",
            "No GUI, G1/Materiaal startup, typed document-session native execution, larger-document or physical power-loss durability claim."])

    def freeze(stage):
        head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, env=environment(),
                                       text=True, timeout=10).strip()
        fingerprint = source_hash()
        row = dict(stage=stage, head=head, source_sha256=fingerprint,
                   passed=head == args.expected_head and fingerprint == args.expected_source)
        report["freeze_checks"].append(row)
        save_json(output, report)
        require(row["passed"], "Frozen HEAD or source changed: " + stage)

    def sdk_snapshot(tag):
        destination = BUILD / "sdk-artifacts" / tag
        destination.mkdir(parents=True)
        for name in ("manifest-m6c.json", "m6c_probe.elf", "libnieuw_os_sdk_runtime.rlib"):
            shutil.copyfile(BUILD / "users" / name, destination / name)
        users = json.loads((destination / "manifest-m6c.json").read_text())
        binary = (destination / "m6c_probe.elf").read_bytes()
        require(users.get("sdk_runtime") is True and users["binary_sha256"] == hashlib.sha256(binary).hexdigest(),
                "Compiled user artifact does not bind the SDK bridge")
        require(users["source_sha256"] == sha256(root / "userspace/m6c_probe.rs")
                and users["linker_sha256"] == sha256(root / "userspace/m6c_linker.ld"), "User source/linker binding changed")
        modules = users["module_sha256"]
        required = {"userspace/sdk_probe_runtime.rs"} | {
            path.relative_to(root).as_posix() for path in (root / "sdk/runtime/src").glob("*.rs")}
        require(required <= set(modules) and all(sha256(root / name) == value for name, value in modules.items()),
                "SDK/bridge/service module source binding failed")
        log_directory = BUILD / "build-snapshots" / tag
        log_directory.mkdir(parents=True)
        for path in sorted((BUILD / "logs").iterdir()):
            if path.is_file():
                shutil.copyfile(path, log_directory / path.name)
        snapshot = dict(stage=tag, sdk_artifact_directory=str(destination),
                        files_sha256={path.name:sha256(path) for path in destination.iterdir()},
                        build_logs_directory=str(log_directory),
                        build_logs_sha256={path.name:sha256(path) for path in log_directory.iterdir()})
        report["build_snapshots"].append(snapshot)
        save_json(output, report)
        return users, binary

    def check_actual_run(result, user, binary):
        directory = Path(result["run_directory"])
        require(result["passed"] is True and result["raw_qemu_exit"] == 33 and result["timed_out"] is False,
                "Native M6c case did not complete successfully")
        require(result["base_commit"] == args.expected_head
                and result["source_sha256"] == result["observed_source_sha256"] == args.expected_source,
                "Run did not bind the freeze")
        check_run(directory, result, args.expected_source)
        manifest = json.loads((directory / "build-manifest.json").read_text())
        kernel = (directory / "esp/kernel.elf").read_bytes()
        require(manifest.get("sdk_runtime") is True and binary in kernel,
                "Real native kernel did not contain the actual SDK user ELF")
        require(sha256(Path(result["command"][0])) == result["qemu_sha256"], "QEMU entry changed")
        row = dict(case=result["case"], run_directory=str(directory),
                   sdk_runtime=True, actual_sdk_user_elf_embedded=True,
                   sdk_user_elf_sha256=user["binary_sha256"],
                   kernel_sha256=sha256(directory / "esp/kernel.elf"),
                   raw_qemu_exit=result["raw_qemu_exit"],
                   complete_existing_per_run_capture_validator_passed=True,
                   actual_artifacts_sha256={path.relative_to(directory).as_posix():sha256(path)
                       for path in sorted(directory.rglob("*")) if path.is_file()})
        return row

    try:
        freeze("before-build")
        report["status"] = "running"
        build(kernel_feature="milestone-m6c", label="g10-normal")
        normal_user, normal_binary = sdk_snapshot("normal")
        freeze("before-normal-vm")
        normal, volume = run_m6c("normal", label="g10-normal")
        report["runs"].append(normal)
        report["real_vm_invocations"] += 1
        report["sdk_bindings"].append(check_actual_run(normal, normal_user, normal_binary))
        image = volume.guard()
        initial_identity = (image.stat().st_dev, image.stat().st_ino)
        freeze("after-normal-vm")
        build(kernel_feature="test-m6c-recover", label="g10-recover")
        recovery_user, recovery_binary = sdk_snapshot("recover")
        require(normal_binary == recovery_binary, "Recovery unexpectedly used a different SDK userspace executable")
        freeze("before-recovery-vm")
        recovered, recovered_volume = run_m6c("recover", volume=volume, label="g10-cold-recovery")
        report["runs"].append(recovered)
        report["real_vm_invocations"] += 1
        report["sdk_bindings"].append(check_actual_run(recovered, recovery_user, recovery_binary))
        before = Path(recovered["run_directory"]) / "test-disk.before.img"
        after = Path(recovered["run_directory"]) / "test-disk.after.img"
        earlier = Path(normal["run_directory"]) / "test-disk.after.img"
        recovery_checks = dict(same_volume_object=recovered_volume is volume,
            same_volume=normal["volume_path"] == recovered["volume_path"],
            same_identity=normal["disk_oracle"]["identity"] == recovered["disk_oracle"]["identity"],
            same_image_inode=initial_identity == (image.stat().st_dev, image.stat().st_ino),
            separate_vm=normal["run_directory"] != recovered["run_directory"],
            distinct_qemu_processes=normal["host_crash"]["process_id"] != recovered["host_crash"]["process_id"],
            linked_previous=recovered["previous_stage"] == normal["run_directory"],
            exact_previous_bytes=earlier.read_bytes() == before.read_bytes(),
            no_recovery_writes=before.read_bytes() == after.read_bytes() == image.read_bytes(),
            same_reconstructed_state=normal["disk_oracle"]["state"] == recovered["disk_oracle"]["state"],
            recovery_zero_mutation_acks=recovered["disk_oracle"]["observed"]["acks"] == 0,
            normal_ten_committed_acks=normal["disk_oracle"]["observed"]["acks"] == 10,
            same_sdk_user_elf=normal_user["binary_sha256"] == recovery_user["binary_sha256"])
        report["recovery_checks"] = recovery_checks
        require(all(recovery_checks.values()), "Cold-recovery binding failed")
        for result in report["runs"]:
            check_run(Path(result["run_directory"]), result, args.expected_source)
        require(report["real_vm_invocations"] == 2, "Expected exactly two native runs")
        freeze("after-complete-recheck")
        report.update(status="completed", passed=True)
    except Exception as error:
        report.update(status="failed", passed=False, error=f"{type(error).__name__}: {error}")
        raise
    finally:
        report["finished_utc"] = datetime.now(timezone.utc).isoformat()
        save_json(output, report)
    print(json.dumps({key:report[key] for key in ("passed", "source_commit", "source_sha256",
        "real_vm_invocations", "recovery_checks")}, indent=2))


if __name__ == "__main__":
    main()
