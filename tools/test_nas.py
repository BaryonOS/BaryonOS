#!/usr/bin/env python3
"""Local NAS normal/cold acceptance using real guest IPC and an independent disk oracle."""
from __future__ import annotations

import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
import uuid

from build import build
from common import ROOT, BUILD, environment, firmware, qemu_machine, save_json, sha256, source_hash
from run_fpu import BOOT, private_path
from run_system import check_esp, check_image
from run_volume import execute
import volume_disk

sys.path.insert(0, str(ROOT / "services/nas/native-probe"))
from disk_oracle import check_image as check_nas_disk

FEATURES = {"normal": "native-system-nas-probe", "recover": "test-system-nas-recover"}


def evaluate(case, serial, raw_exit, timed_out):
    if case not in FEATURES or type(raw_exit) is not int or raw_exit != 33 or timed_out:
        raise ValueError("NAS guest did not complete successfully")
    if any(marker in serial for marker in ("FAIL", "PANIC")):
        raise ValueError("NAS guest failure, including post-success output")
    expected = (*BOOT, "NAS:MANIFEST:PASS", "NAS:BROKERS:PASS", "NAS:CLEANUP:PASS",
                f"NAS:CASE:{case.upper()}:PASS", "M1:PASS")
    controlled = [line for line in serial.splitlines()
                  if line in BOOT or line == "M1:PASS"
                  or line.startswith(("NAS:MANIFEST:", "NAS:BROKERS:", "NAS:CLEANUP:", "NAS:CASE:"))]
    if controlled != list(expected):
        raise ValueError("NAS marker sequence differs")
    numbers = []
    for line in serial.splitlines():
        if line.startswith("NAS:") and not line.endswith(":PASS"):
            match = re.fullmatch(r"NAS:([A-Z_]+):([0-9]{20})", line)
            if not match:
                raise ValueError("Malformed NAS evidence")
            numbers.append((match[1], int(match[2])))
    if len(numbers) != 21 or [name for name, _ in numbers[:2]] != ["ADMIN_PID", "GUEST_PID"]:
        raise ValueError("Missing or duplicated NAS evidence")
    admin, guest = (value for _, value in numbers[:2])
    if not admin or not guest or admin == guest:
        raise ValueError("Invalid actual caller identities")
    counts = [(6, 1), (12, 6), (6, 5)] if case == "normal" else [(4, 0), (4, 4), (0, 4)]
    roles = []
    for index, (slot, (ok, errors)) in enumerate(zip((0, 1, 3), counts)):
        group = numbers[2 + index * 5:7 + index * 5]
        if [name for name, _ in group] != [
                "ROLE_SLOT", "RPC_OK", "RPC_ERRORS", "RIGHTS_DENIED", "STACK_TOUCHED"]:
            raise ValueError("NAS role evidence order differs")
        values = [value for _, value in group]
        if values[:4] != [slot, ok, errors, int(slot == 3)] or not 1 <= values[4] <= 65536:
            raise ValueError("NAS role assertion differs")
        roles.append(dict(group))
    broker = dict(numbers[-4:])
    if list(broker) != ["ENTROPY_BYTES", "DISK_READS", "DISK_WRITES", "DISK_FLUSHES"]:
        raise ValueError("NAS broker evidence differs")
    if broker["ENTROPY_BYTES"] <= 0 or broker["DISK_READS"] <= 0:
        raise ValueError("No actual entropy/disk completion")
    writes, flushes = broker["DISK_WRITES"], broker["DISK_FLUSHES"]
    if ((case == "recover" and (writes or flushes))
            or (case == "normal" and (not writes or not flushes))):
        raise ValueError("Wrong block mode/completions")
    return {"admin_pid": admin, "guest_pid": guest, "roles": roles, "brokers": broker}


def run_case(case, parent=None):
    feature = FEATURES[case]
    frozen = source_hash()
    manifest = build(None, feature, feature)
    if manifest["source_sha256"] != frozen or not manifest.get("nas_probe"):
        raise ValueError("NAS build changed source or lacks ELF evidence")
    run = private_path(BUILD / "run" / f"nas-{case}-{uuid.uuid4().hex}")
    run.mkdir(parents=True, exist_ok=False)
    check_esp(BUILD / "esp", manifest)
    shutil.copytree(BUILD / "esp", run / "esp")
    check_esp(run / "esp", manifest)
    code, variables = firmware()
    shutil.copyfile(code, run / "OVMF_CODE.fd")
    shutil.copyfile(variables, run / "OVMF_VARS.fd")
    image = run / "volume.img"
    if case == "normal":
        if parent is not None:
            raise ValueError("Normal NAS run must create a new disk")
        volume_disk.create_image(image, "empty")
    else:
        if (parent is None or parent.get("passed") is not True or parent.get("case") != "normal"
                or parent.get("source_sha256") != frozen):
            raise ValueError("Cold NAS run requires a passing same-source normal parent")
        source = private_path(ROOT / parent["image"], regular=True)
        check_image(source)
        if sha256(source) != parent["after_sha256"]:
            raise ValueError("Normal disk bytes changed before cold run")
        shutil.copyfile(source, image)
    check_image(image)
    before = sha256(image)
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("qemu-system-x86_64 not found")
    relative = run.relative_to(ROOT).as_posix()
    readonly = "on" if case == "recover" else "off"
    command = [
        qemu, "-machine", qemu_machine(), "-accel", "tcg,thread=single", "-cpu", "qemu64",
        "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
        "-serial", "stdio", "-nic", "none", "-no-reboot",
        "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={relative}/OVMF_CODE.fd",
        "-drive", f"if=pflash,unit=1,format=raw,file={relative}/OVMF_VARS.fd",
        "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{relative}/esp",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-drive", f"if=none,id=volume,format=raw,cache=writeback,readonly={readonly},file={relative}/volume.img",
        "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8",
        "-object", "rng-random,id=rng0,filename=/dev/urandom",
        "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000",
    ]
    report = dict(case=case, passed=False, source_sha256=frozen,
                  source_commit=subprocess.check_output(
                      ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
                  manifest=manifest, command=command, image=image.relative_to(ROOT).as_posix(),
                  qemu_sha256=sha256(Path(qemu).resolve(strict=True)),
                  qemu_version=subprocess.check_output([qemu, "--version"], env=env, text=True).strip(),
                  firmware_sha256={"code": sha256(run / "OVMF_CODE.fd"),
                                   "vars_template": sha256(run / "OVMF_VARS.fd")},
                  before_sha256=before, run_directory=str(run))
    start = time.monotonic()
    raw, serial, stderr, timed_out, _, _ = execute(command, env, timeout=240)
    (run / "serial.txt").write_text(serial)
    (run / "stderr.txt").write_text(stderr)
    report.update(raw_qemu_exit=raw, timed_out=timed_out,
                  duration_s=time.monotonic() - start, after_sha256=sha256(image))
    try:
        evidence = evaluate(case, serial, raw, timed_out)
        if parent and (evidence["admin_pid"], evidence["guest_pid"]) != (
                parent["evidence"]["admin_pid"], parent["evidence"]["guest_pid"]):
            raise ValueError("Cold local caller identities changed")
        report["oracle"] = check_nas_disk(
            image.read_bytes(), evidence["admin_pid"], evidence["guest_pid"],
            parent["after_sha256"] if parent else None)
        report["evidence"] = evidence
        if source_hash() != frozen:
            raise ValueError("NAS source changed during execution")
        report["passed"] = True
    except ValueError as error:
        report["error"] = str(error)
    save_json(run / "result.json", report)
    print(f"NAS {case}: passed={report['passed']} raw={raw} {run}")
    return report


def main():
    frozen = source_hash()
    summary = {"schema_version": 1, "source_sha256": frozen, "passed": False, "stages": []}
    path = private_path(BUILD / "nas-system-gate.json")
    save_json(path, summary)
    try:
        for case in FEATURES:
            stage = run_case(case, summary["stages"][0] if case == "recover" else None)
            summary["stages"].append(stage)
            save_json(path, summary)
            if stage["passed"] is not True:
                raise ValueError(stage["error"])
        if source_hash() != frozen:
            raise ValueError("Source changed during normal/cold NAS gate")
        summary["passed"] = True
    except (ValueError, RuntimeError, OSError, subprocess.SubprocessError) as error:
        summary["error"] = f"{type(error).__name__}: {error}"
    save_json(path, summary)
    print(json.dumps({"passed": summary["passed"], "error": summary.get("error"), "report": str(path)}))
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
