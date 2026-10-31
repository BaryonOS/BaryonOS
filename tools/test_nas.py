#!/usr/bin/env python3
"""Local NAS normal/cold acceptance using real guest IPC and an independent disk oracle."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import time
import uuid

from build import build
from common import ROOT, BUILD, environment, firmware, qemu_machine, save_json, sha256, source_hash
from run_fpu import BOOT, private_path
from run_system import check_esp, check_image
from run_volume import execute
import volume_disk

sys.path.insert(0, str(ROOT / "services/nas/native-probe"))
sys.path.insert(0, str(ROOT / "services/nas/native-probe/tests"))
from disk_oracle import check_image as check_nas_disk
import replay_handoff

FEATURES = {"normal": "native-system-nas-probe", "recover": "test-system-nas-recover"}
# #110 private handoff. Token bytes only ever exist in files inside a private
# 0700 directory outside the checkout; reports, serial and CI artifacts carry
# at most their SHA-256 (provenance binding, not a credential).
REPLAY_MAGIC = b"NOSRPL01"
REPLAY_PORT = "0x4f8"
REPLAY_FW_CFG = "opt/nieuw-os/nas-replay"
CAPTURE_STAGE = "normal-after-stage"


def private_handoff_dir() -> Path:
    directory = Path(tempfile.mkdtemp(prefix="nos-nas-replay-"))
    directory.chmod(0o700)
    resolved, root = directory.resolve(strict=True), ROOT.resolve(strict=True)
    if resolved == root or root in resolved.parents:
        raise ValueError("NAS replay handoff directory must be outside the checkout")
    return resolved


def new_private_file(path: Path) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0), 0o600)
    os.close(descriptor)


def leaks_payload(text: str, payload: bytes) -> bool:
    return payload.hex() in text.lower() or payload.decode("latin-1") in text


def export_handoff(handoff: Path, capture: Path, report: dict, serial: str) -> dict:
    """Turn the normal VM's private debugcon capture into a bound handoff."""
    raw = replay_handoff._private_file(capture, ROOT, len(REPLAY_MAGIC) + replay_handoff.TOKEN_PAIR_BYTES)
    if len(raw) != len(REPLAY_MAGIC) + replay_handoff.TOKEN_PAIR_BYTES or not raw.startswith(REPLAY_MAGIC):
        raise ValueError("NAS replay capture is not exactly one framed 80-byte pair")
    payload = raw[len(REPLAY_MAGIC):]
    if not any(payload[:40]) or not any(payload[40:]):
        raise ValueError("NAS replay capture lacks a session or upload")
    if leaks_payload(serial, payload):
        raise ValueError("NAS replay token bytes reached the archived serial log")
    payload_path, manifest_path = handoff / "payload.bin", handoff / "manifest.json"
    for path in (payload_path, manifest_path):
        new_private_file(path)
    payload_path.write_bytes(payload)
    manifest = {"schema_version": 1, "source_commit": report["source_commit"],
                "origin_run_id": Path(report["run_directory"]).name, "origin_role": "guest",
                "origin_stage": "normal", "capture_stage": CAPTURE_STAGE, "completed": True,
                "payload_length": len(payload), "payload_sha256": hashlib.sha256(payload).hexdigest()}
    manifest_path.write_text(json.dumps(manifest, sort_keys=True))
    capture.unlink()
    replay_handoff.load_handoff(manifest_path, payload_path, ROOT, report["source_commit"],
                                manifest["origin_run_id"])
    return {"origin_run_id": manifest["origin_run_id"], "payload_sha256": manifest["payload_sha256"],
            "capture_stage": CAPTURE_STAGE}


def check_immutable(run, manifest, platform, code, variables):
    check_esp(run / "esp", manifest)
    if (sha256(run / "OVMF_CODE.fd") != platform["code_sha256"]
            or sha256(code) != platform["code_sha256"]
            or sha256(variables) != platform["vars_template_sha256"]
            or sha256(Path(platform["qemu_path"])) != platform["qemu_sha256"]):
        raise ValueError("NAS immutable execution artifacts changed")


def evaluate(case, serial, raw_exit, timed_out):
    if case not in FEATURES or type(raw_exit) is not int or raw_exit != 33 or timed_out:
        raise ValueError("NAS guest did not complete successfully")
    if any(marker in serial for marker in ("FAIL", "PANIC")):
        raise ValueError("NAS guest failure, including post-success output")
    if case == "normal":
        expected = (*BOOT, "NAS:MANIFEST:PASS", "NAS:REPLAY_EXPORT:PASS", "NAS:BROKERS:PASS",
                    "NAS:CLEANUP:PASS", "NAS:CASE:NORMAL:PASS", "M1:PASS")
    else:
        expected = (*BOOT, "NAS:REPLAY_INPUT:PASS", "NAS:MANIFEST:PASS", "NAS:BROKERS:PASS",
                    "NAS:CLEANUP:PASS", f"NAS:CASE:{case.upper()}:PASS", "M1:PASS")
    controlled = [line for line in serial.splitlines()
                  if line in BOOT or line == "M1:PASS"
                  or line.startswith(("NAS:MANIFEST:", "NAS:BROKERS:", "NAS:CLEANUP:", "NAS:CASE:",
                                      "NAS:REPLAY_"))]
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
    # Cold: the exact-token COMMIT replay adds one Stale reply at NAS and guest.
    counts = [(6, 1), (12, 6), (6, 5)] if case == "normal" else [(4, 0), (4, 5), (0, 5)]
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


def run_case(case, parent=None, handoff=None):
    if handoff is None:
        raise ValueError("NAS runs require the private replay handoff directory")
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
    qemu = str(Path(qemu).resolve(strict=True))
    with Path(qemu).open("rb") as stream:
        if stream.read(4) != b"\x7fELF":
            raise RuntimeError("NAS gate requires the actual QEMU ELF executable in PATH, not a wrapper")
    platform = {
        "machine": qemu_machine(),
        "qemu_path": qemu,
        "qemu_sha256": sha256(Path(qemu)),
        "qemu_version": subprocess.check_output([qemu, "--version"], env=env, text=True).strip(),
        "code_sha256": sha256(code),
        "vars_template_sha256": sha256(variables),
    }
    if parent is not None and parent.get("platform") != platform:
        raise ValueError("Cold NAS platform differs from the frozen normal parent")
    if (sha256(run / "OVMF_CODE.fd") != platform["code_sha256"]
            or sha256(run / "OVMF_VARS.fd") != platform["vars_template_sha256"]):
        raise ValueError("Copied NAS firmware differs from the frozen platform")
    check_immutable(run, manifest, platform, code, variables)
    relative = run.relative_to(ROOT).as_posix()
    readonly = "on" if case == "recover" else "off"
    command = [
        qemu, "-machine", platform["machine"], "-accel", "tcg,thread=single", "-cpu", "qemu64",
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
    capture = cold_input = cold_digest = None
    if case == "normal":
        # A new normal run starts a new handoff; nothing from an earlier
        # attempt may be bound to this run's cold child.
        for name in ("normal-capture.bin", "payload.bin", "manifest.json", "cold-input.bin"):
            stale = handoff / name
            if stale.exists():
                stale.chmod(0o600)
                stale.unlink()
        capture = handoff / "normal-capture.bin"
        new_private_file(capture)
        command += ["-chardev", f"file,id=nasreplay,path={capture}",
                    "-device", f"isa-debugcon,iobase={REPLAY_PORT},chardev=nasreplay"]
    else:
        origin = parent.get("replay_handoff") or {}
        bound, payload = replay_handoff.load_handoff(
            handoff / "manifest.json", handoff / "payload.bin", ROOT,
            parent["source_commit"], origin.get("origin_run_id", ""))
        if bound["payload_sha256"] != origin.get("payload_sha256"):
            raise ValueError("NAS replay handoff differs from the normal parent's binding")
        cold_input = handoff / "cold-input.bin"
        cold_digest = replay_handoff.create_cold_input(payload, cold_input, ROOT)
        command += ["-fw_cfg", f"name={REPLAY_FW_CFG},file={cold_input}"]
    report = dict(case=case, passed=False, source_sha256=frozen,
                  source_commit=subprocess.check_output(
                      ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
                  manifest=manifest, command=command, image=image.relative_to(ROOT).as_posix(),
                  platform=platform, qemu_sha256=platform["qemu_sha256"],
                  qemu_version=platform["qemu_version"],
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
        check_immutable(run, manifest, platform, code, variables)
        evidence = evaluate(case, serial, raw, timed_out)
        if parent and (evidence["admin_pid"], evidence["guest_pid"]) != (
                parent["evidence"]["admin_pid"], parent["evidence"]["guest_pid"]):
            raise ValueError("Cold local caller identities changed")
        report["oracle"] = check_nas_disk(
            image.read_bytes(), evidence["admin_pid"], evidence["guest_pid"],
            parent["after_sha256"] if parent else None)
        report["evidence"] = evidence
        if case == "normal":
            report["replay_handoff"] = export_handoff(handoff, capture, report, serial)
        else:
            payload = replay_handoff.verify_cold_input(cold_input, cold_digest, ROOT)
            if leaks_payload(serial, payload):
                raise ValueError("NAS replay token bytes reached the archived serial log")
            report["replay_input"] = {"payload_sha256": cold_digest,
                                      "origin_run_id": parent["replay_handoff"]["origin_run_id"]}
        if source_hash() != frozen:
            raise ValueError("NAS source changed during execution")
        report["passed"] = True
    except (ValueError, replay_handoff.HandoffError) as error:
        report["error"] = str(error)
        # Never keep token bytes from a failed run.
        if capture is not None and capture.exists():
            capture.unlink()
    save_json(run / "result.json", report)
    print(f"NAS {case}: passed={report['passed']} raw={raw} {run}")
    return report


def main():
    frozen = source_hash()
    summary = {"schema_version": 1, "source_sha256": frozen, "passed": False, "stages": []}
    path = private_path(BUILD / "nas-system-gate.json")
    save_json(path, summary)
    handoff = None
    try:
        handoff = private_handoff_dir()
        for case in FEATURES:
            stage = run_case(case, summary["stages"][0] if case == "recover" else None, handoff)
            summary["stages"].append(stage)
            save_json(path, summary)
            if stage["passed"] is not True:
                raise ValueError(stage["error"])
        if source_hash() != frozen:
            raise ValueError("Source changed during normal/cold NAS gate")
        summary["passed"] = True
    except (ValueError, RuntimeError, OSError, subprocess.SubprocessError) as error:
        summary["error"] = f"{type(error).__name__}: {error}"
    finally:
        # Disposable-VM test handles only; never kept after the gate.
        if handoff is not None:
            shutil.rmtree(handoff, ignore_errors=True)
    save_json(path, summary)
    print(json.dumps({"passed": summary["passed"], "error": summary.get("error"), "report": str(path)}))
    return 0 if summary["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
