#!/usr/bin/env python3
"""N01 gate: three real ring-3 processes and a causal localhost Ethernet peer.

This profile tests a reusable TCP/UDP service, not TLS, real hardware or a
full Internet policy issuer. Images, user ELFs, firmware and actual serial
bytes are bound to one unchanged source and independently rechecked at exit.
"""
from __future__ import annotations
import argparse
import hashlib
from datetime import datetime, timezone
import math
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import time
import threading
import uuid
from common import (ROOT, BUILD, environment, firmware, qemu_machine, sha256,
                    source_hash, without_acpi_diagnostics)
from build_users import inspect_user
from run_fpu import private_path, publish, publish_json, load_json
from run_system import check_esp, network_arguments
from network_service_peer import Peer, validate_transcript, REQUESTS, RESPONSES, MAX_FRAMES

FEATURE = "native-net-service"
FEATURES = {"normal": FEATURE, "client-exit": "test-net-service-client-exit", "revoke": "test-net-service-revoke"}
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "M2:PASS")
OFFSETS = (240, 400, 408, 416, 424, 432, 440, 448, 456, 464, 472, 480, 488, 496, 504, 512, 520, 528, 536, 544, 552)
ROLES = ("SERVICE", "APP_A", "APP_B")
REVOKE_ROLES = ROLES + ("POLICY",)
REQUIRED = BOOT + ("NETSVC:MANIFEST:PASS", "NETSVC:CLEANUP:PASS", "NETSVC:PASS", "M1:PASS")
FORBIDDEN = ("FAIL", "PANIC")



def execute_native(command, env, *, timeout, limit=128 * 1024):
    """Owned QEMU process; bounded binary pipes, lifetime and final wait.

    Never normalize or replace undecodable bytes. Reader threads retain at
    most limit bytes per pipe and signal overflow so the owner kills QEMU.
    They drain/discard only until that termination unblocks both pipes.
    """
    if type(timeout) not in (int, float) or not 0 < timeout <= 180 or type(limit) is not int or not 1 <= limit <= 128 * 1024:
        raise ValueError("Bounded native process lifetime/output required")
    process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    buffers = [bytearray(), bytearray()]
    overflow = threading.Event()
    reader_error = []
    def reader(pipe, output):
        try:
            while True:
                raw = pipe.read(4096)
                if not raw:
                    return
                remaining = limit - len(output)
                output.extend(raw[:remaining])
                if len(raw) > remaining:
                    overflow.set()
        except OSError as error:
            reader_error.append(error)
        finally:
            pipe.close()
    threads = [threading.Thread(target=reader, args=(pipe, output), daemon=True)
               for pipe, output in zip((process.stdout, process.stderr), buffers)]
    for thread in threads:
        thread.start()
    deadline, timed_out = time.monotonic() + timeout, False
    try:
        while process.poll() is None:
            remaining = deadline - time.monotonic()
            if remaining <= 0 or overflow.is_set():
                timed_out = remaining <= 0
                process.kill()
                break
            try:
                process.wait(timeout=min(0.02, remaining))
            except subprocess.TimeoutExpired:
                pass
        process.wait(timeout=2)
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=2)
        for thread in threads:
            thread.join(timeout=2)
    if any(thread.is_alive() for thread in threads) or reader_error:
        raise RuntimeError("Native pipe reader did not complete cleanly")
    return process.returncode, bytes(buffers[0]), bytes(buffers[1]), timed_out, overflow.is_set()


def record_streams(directory, stdout, stderr):
    """Publish exact pipe bytes first; strict UTF-8 is required for grammar."""
    if not isinstance(stdout, bytes) or not isinstance(stderr, bytes):
        raise ValueError("Native evidence must originate as raw pipe bytes")
    for name, raw in (("serial.txt", stdout), ("stderr.txt", stderr)):
        publish(directory / name, raw)
        if private_path(directory / name, regular=True).read_bytes() != raw:
            raise ValueError("Stored native stream differs from actual bytes")
    return stdout.decode("utf-8", "strict"), stderr.decode("utf-8", "strict")

def integer(value, low=0, high=(1 << 64) - 1):
    if type(value) is not int or not low <= value <= high:
        raise ValueError("Missing/out-of-bound native integer")
    return value


def sequence(stdout, case="normal"):
    if case not in FEATURES:
        raise ValueError("Unknown native service case")
    if not isinstance(stdout, str) or len(stdout) > 128 * 1024 or any(s in stdout for s in FORBIDDEN):
        raise ValueError("Failed/oversized actual native serial")
    stdout = without_acpi_diagnostics(stdout)
    lines = stdout.splitlines()
    if any(lines.count(marker) != 1 for marker in REQUIRED):
        raise ValueError("Missing/duplicate required native marker")
    start = lines.index(BOOT[0])
    if any(re.search(r"(?:BOOT:|BOOTINFO:|KERNEL:|KS-|M[0-9]+:|NETSVC:)", line) for line in lines[:start]):
        raise ValueError("Native marker before actual EBS")
    if tuple(lines[start:start + len(BOOT)]) != BOOT:
        raise ValueError("Native boot order differs")
    if lines.index("NETSVC:MANIFEST:PASS") != start + len(BOOT):
        raise ValueError("Native manifest order differs")
    if lines[-3:] != ["NETSVC:CLEANUP:PASS", "NETSVC:PASS", "M1:PASS"]:
        raise ValueError("Native clean ending missing")
    revoke = case == "revoke"
    found, network = {r: {} for r in (REVOKE_ROLES if revoke else ROLES)}, {}
    numeric = re.compile(r"NETSVC:(SERVICE|APP_A|APP_B|POLICY):([0-9]+):([0-9]{20})")
    revoke_ticks = None
    net_numeric = re.compile(r"SYSTEM:(NET_[A-Z_]+):([0-9]{20})")
    entropy, exit_ticks = None, {}
    for line in lines[start + len(BOOT) + 1:-3]:
        tick_match = re.fullmatch(r"NETSVC:(APP_A|APP_B)_EXIT_TICKS:([0-9]{20})", line)
        if tick_match:
            name, value = tick_match.groups()
            if name in exit_ticks:
                raise ValueError("Duplicate actual process exit tick")
            exit_ticks[name] = integer(int(value), 1, 6000)
            continue
        revoke_match = re.fullmatch(r"NETSVC:REVOKE_TICKS:([0-9]{20})", line) if revoke else None
        if revoke_match:
            if revoke_ticks is not None or exit_ticks:
                raise ValueError("Duplicate or late revocation request")
            revoke_ticks = integer(int(revoke_match.group(1)), 1, 6000)
            continue
        entropy_match = re.fullmatch(r"NETSVC:ENTROPY_BYTES:([0-9]{20})", line)
        if entropy_match:
            if entropy is not None:
                raise ValueError("Duplicate entropy observation")
            entropy = integer(int(entropy_match.group(1)))
            continue
        match = numeric.fullmatch(line)
        if match:
            role, offset, value = match.groups()
            offset, value = int(offset), integer(int(value))
            if role not in found or offset not in OFFSETS or offset in found[role]:
                raise ValueError("Unknown/duplicate native process observation")
            found[role][offset] = value
            continue
        match = net_numeric.fullmatch(line)
        if match:
            name, value = match.groups()
            if name in network:
                raise ValueError("Duplicate broker observation")
            network[name] = integer(int(value))
            continue
        raise ValueError("Unknown/reordered native evidence: " + line)
    if set(exit_ticks) != {"APP_A", "APP_B"} or case == "client-exit" and exit_ticks["APP_B"] <= exit_ticks["APP_A"]:
        raise ValueError("Missing actual exit ticks or peer did not survive first app")
    if revoke and (revoke_ticks is None or revoke_ticks >= exit_ticks["APP_A"]):
        raise ValueError("Revocation was not requested while app A was live")
    if entropy != 8:
        raise ValueError("Actual service entropy acquisition missing")
    if any(set(values) != set(OFFSETS) for values in found.values()):
        raise ValueError("Incomplete native process snapshot")
    for role, values in found.items():
        if values[240] != 0 or values[400] != 1 or values[528] != 1:
            raise ValueError("Process failed or did not initialize/cleanly finish: " + role)
    service = found["SERVICE"]
    normal = case == "normal"
    owners = 2 if normal else 1
    fixed = {448: owners, 456: owners, 464: 10 if normal else 6,
             472: 3 * sum(map(len, REQUESTS)) if normal else 3 * len(REQUESTS[1]) + revoke * len(REQUESTS[0]),
             480: 2 * sum(map(len, RESPONSES)) if normal else 2 * len(RESPONSES[1]),
             488: 4 if normal else 2 + revoke, 496: 3, 504: 0, 536: 1 if case == "client-exit" else 0,
             544: int(revoke)}
    if not revoke:
        fixed[552] = 0
    if (any(service[k] != v for k, v in fixed.items()) or service[408] < 1
            # Revoke: app B's client-exit bound minus app A's 2, plus app A's 8.
            or service[432] < (40 if normal else 20) or service[440] < (22 if normal else 19 if revoke else 13)):
        raise ValueError("Native service reuse/cleanup/authority evidence differs")
    if service[512] < 4 * owners or service[520] < owners:
        raise ValueError("Native fragment/malformed UDP/backpressure proof missing")
    for role in ROLES[1:]:
        app = found[role]
        if revoke and role == "APP_A":
            # Helper-route and out-of-grant opens denied, one live request,
            # prompt end after revocation, then five denied uses (offset 432).
            exact = {offset: 0 for offset in OFFSETS if offset not in (400, 448, 504, 528, 552)}
            exact.update({424: 2, 432: 5, 440: 1, 480: 1, 544: 2})
            if any(app[k] != v for k, v in exact.items()) or app[448] < 1 or not 0 <= app[504] < 500:
                raise ValueError("Revoked app did not see a prompt reset and denied reuse")
        elif not normal and role == "APP_A":
            exact = {offset: 0 for offset in OFFSETS if offset not in (400, 528)}
            exact.update({424: 1, 440: 1, 480: 1, 536: 1})
            if any(app[k] != v for k, v in exact.items()):
                raise ValueError("Disconnected app case did not execute exactly its bounded setup")
        else:
            exact = {408: 2, 416: 1, 424: 1, 432: 9, 440: 1, 456: len(RESPONSES[0]),
                     464: 2, 472: 4, 480: 1, 488: 1, 496: 1, 504: 1, 512: 1, 520: 2, 536: 0, 544: 0, 552: 0}
            if any(app[k] != v for k, v in exact.items()) or app[448] < 3:
                raise ValueError("Native app success/refusal/reopen evidence differs: " + role)
    if revoke:
        policy = found["POLICY"]
        if any(policy[k] != (1 if k in (400, 528, 544) else 0) for k in OFFSETS if k != 552):
            raise ValueError("Policy authority did more than one authenticated revocation")
        # One boot-monotonic clock: request <= policy send <= service revoke <= app A sees Closed.
        if not revoke_ticks <= policy[552] <= service[552] <= found["APP_A"][552]:
            raise ValueError("Revocation evidence is not causally ordered")
    expected_net = {"NET_TX_SUBMITTED", "NET_TX_COMPLETED", "NET_RX_COMPLETED", "NET_RX_DELIVERED",
                    "NET_DENIED", "NET_TX_TIMEOUTS", "NET_RESETS", "NET_RESET_ACKS"}
    if set(network) != expected_net:
        raise ValueError("Unknown/missing native broker counters")
    if (network["NET_TX_SUBMITTED"] != network["NET_TX_COMPLETED"]
            or service[416] != network["NET_TX_COMPLETED"]
            or service[424] != network["NET_RX_DELIVERED"]
            or network["NET_RX_COMPLETED"] != network["NET_RX_DELIVERED"]
            or network["NET_DENIED"] != 2 + revoke or network["NET_TX_TIMEOUTS"] != 0
            or network["NET_RESETS"] != 1 or network["NET_RESET_ACKS"] != 1):
        raise ValueError("Native NIC completion/denial/reset evidence differs")
    integer(service[416], 1, MAX_FRAMES)
    integer(service[424], 1, MAX_FRAMES)
    return {"processes": {r: {str(k): v for k, v in values.items()} for r, values in found.items()}, "network": network, "entropy_bytes": entropy, "exit_ticks": exit_ticks, "revoke_ticks": revoke_ticks}


def validate_build(frozen, case="normal"):
    if case not in FEATURES:
        raise ValueError("Unknown native service build case")
    manifest = load_json(private_path(BUILD / "manifest.json", regular=True))
    if (manifest.get("kernel_feature") != FEATURES[case] or manifest.get("loader_feature") is not None
            or manifest.get("source_sha256") != frozen or manifest.get("source_fingerprint_version") != 3
            or manifest.get("sdk_runtime") is not False or manifest.get("document_session") is not False):
        raise ValueError("Build the selected native service profile on the unchanged current source")
    check_esp(private_path(BUILD / "esp"), manifest)
    users = manifest.get("network_service")
    if not isinstance(users, dict) or set(users) != {"service", "client"}:
        raise ValueError("Missing exact inspected network-service ELF manifest")
    kernel = private_path(BUILD / "esp/kernel.elf", regular=True).read_bytes()
    for role, name in (("service", "network-service"), ("client", "network-client")):
        path = private_path(BUILD / f"users/network-service/{name}.elf", regular=True)
        if users[role] != {"sha256": sha256(path), "elf": inspect_user(path)} or path.read_bytes() not in kernel:
            raise ValueError("User ELF hash/inspection/embedding differs: " + role)
    return manifest


def evaluate(report, peer, *, case="normal"):
    if case not in FEATURES or report.get("case") != case:
        raise ValueError("Native service case binding changed")
    if (type(report.get("raw_qemu_exit")) is not int or report["raw_qemu_exit"] != 33
            or report.get("timed_out") is not False or report.get("kernel_feature") != FEATURES[case]
            or not isinstance(report.get("qemu_version"), str) or not report["qemu_version"].startswith("QEMU emulator version ")
            or type(report.get("duration_s")) not in (int, float) or not math.isfinite(report["duration_s"])
            or not 0 <= report["duration_s"] <= report["timeout_s"] + 5):
        raise ValueError("Native QEMU exit/lifetime/profile evidence failed")
    observed = sequence(report["stdout"], case)
    if (peer.get("passed") is not True or peer.get("connected") is not True
            or peer.get("error") is not None or peer.get("rejected_frame") is not None):
        raise ValueError("Actual synthetic packet peer failed: " + str(peer.get("error")))
    replay = validate_transcript(peer, case=case)
    net = observed["network"]
    counts = replay["counts"]
    if (net["NET_TX_COMPLETED"] != counts["client_frames"]
            or net["NET_RX_DELIVERED"] != counts["server_frames"]
            or counts["udp_requests"] != (2 if case == "normal" else 1)
            or any(counts[name] != (2 if case == "normal" else 1) for name in ("udp_corrupt_sent", "udp_wrong_scope_sent", "udp_echo_sent", "udp_overflow_sent"))):
        raise ValueError("Actual wire/service/broker counts do not agree")
    return observed


def run_network_service(*, case="normal", timeout=90):
    if case not in FEATURES:
        raise ValueError("Unknown native service run case")
    if type(timeout) is not int or not 1 <= timeout <= 180:
        raise ValueError("Native timeout must be bounded 1..180 seconds")
    private_path(BUILD)
    frozen = source_hash()
    manifest = validate_build(frozen, case)
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if qemu is None:
        raise RuntimeError("Missing QEMU")
    qemu_hash = sha256(Path(qemu))
    code, variables = firmware()
    for image in (code, variables):
        details = image.lstat()
        if not stat.S_ISREG(details.st_mode) or not 1 <= details.st_size <= 16 * 1024 * 1024:
            raise ValueError("Firmware must be a bounded regular file")
    original_firmware = {"code": sha256(code), "vars_template": sha256(variables)}
    directory = private_path(BUILD / "run" / ("network-service-" + case + "-" + uuid.uuid4().hex))
    directory.mkdir(parents=True, exist_ok=False)
    shutil.copytree(BUILD / "esp", directory / "esp")
    check_esp(directory / "esp", manifest)
    for path, name in ((code, "OVMF_CODE.fd"), (variables, "OVMF_VARS.template.fd"), (variables, "OVMF_VARS.fd")):
        shutil.copy2(path, private_path(directory / name))
    snapshots = {p.relative_to(directory).as_posix(): sha256(p) for p in
                 (directory / "OVMF_CODE.fd", directory / "OVMF_VARS.template.fd", directory / "esp/kernel.elf", directory / "esp/EFI/BOOT/BOOTX64.EFI")}
    rel = directory.relative_to(ROOT).as_posix()
    command = [qemu, "-machine", qemu_machine(), "-accel", "tcg,thread=single", "-cpu", "qemu64", "-smp", "1", "-m", "256M",
               "-nodefaults", "-display", "none", "-monitor", "none", "-serial", "stdio", "-nic", "none", "-no-reboot",
               "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
               "-drive", f"if=pflash,unit=1,format=raw,file={rel}/OVMF_VARS.fd",
               "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
               "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
               "-object", "rng-random,id=rng0,filename=/dev/urandom",
               "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000"]
    if (Path(qemu).parent / "share").is_dir():
        command += ["-L", str(Path(qemu).parent / "share")]
    report = dict(schema_version=1, passed=False, case=case, kernel_feature=FEATURES[case],
                  scope=("four" if case == "revoke" else "three") + " actual isolated ring-3 processes; bounded IPC TCP/UDP; synthetic localhost Ethernet; no TLS or physical hardware claim",
                  source_commit=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
                  source_fingerprint_version=3, source_sha256=frozen, manifest=manifest,
                  run_directory=str(directory), timeout_s=timeout, started_utc=datetime.now(timezone.utc).isoformat(),
                  qemu_sha256=qemu_hash, firmware_sha256=original_firmware, snapshot_sha256=snapshots,
                  qemu_version=subprocess.check_output([qemu, "--version"], env=env, text=True, timeout=10).strip())
    with Peer(timeout=timeout, case=case) as peer:
        command += network_arguments(peer.port)
        begin = time.monotonic()
        rc, stdout_raw, stderr_raw, timed_out, output_overflow = execute_native(command, env, timeout=timeout)
        report.update(command=command, duration_s=time.monotonic() - begin, raw_qemu_exit=rc,
                      timed_out=timed_out, output_overflow=output_overflow,
                      stdout_sha256=hashlib.sha256(stdout_raw).hexdigest(), stderr_sha256=hashlib.sha256(stderr_raw).hexdigest(),
                      stdout_bytes=len(stdout_raw), stderr_bytes=len(stderr_raw))
    evidence = peer.finish()
    publish_json(directory / "network-peer.json", evidence)
    report["peer"] = evidence
    try:
        stdout, stderr = record_streams(directory, stdout_raw, stderr_raw)
        report.update(stdout=stdout, stderr=stderr)
        if output_overflow:
            raise ValueError("Native output exceeded bounded pipe storage")
        report["observed"] = evaluate(report, evidence, case=case)
        if private_path(directory / "serial.txt", regular=True).read_bytes() != stdout_raw:
            raise ValueError("Stored serial differs from actual bytes")
        if load_json(private_path(directory / "network-peer.json", regular=True)) != evidence:
            raise ValueError("Stored peer transcript differs from actual server bytes")
        if private_path(directory / "stderr.txt", regular=True).read_bytes() != stderr_raw:
            raise ValueError("Stored QEMU diagnostics differ from actual bytes")
        check_esp(directory / "esp", manifest)
        if {name: sha256(private_path(directory / name, regular=True)) for name in snapshots} != snapshots:
            raise ValueError("Immutable firmware/ESP snapshot changed during run")
        if {"code": sha256(code), "vars_template": sha256(variables)} != original_firmware or sha256(Path(qemu)) != qemu_hash:
            raise ValueError("QEMU or original firmware changed during run")
        if validate_build(frozen, case) != manifest or source_hash() != frozen:
            raise ValueError("Source/build/user artifacts changed during run")
        report.update(passed=True, network_service_gate=True)
    except (ValueError, KeyError, TypeError, OSError) as error:
        report.update(error=f"{type(error).__name__}: {error}", network_service_gate=False)
    publish_json(directory / "result.json", report)
    publish_json(BUILD / ("network-service-" + case + "-report.json"), report)
    publish_json(BUILD / "network-service-report.json", report)
    print(f"N01 native service passed={report['passed']} raw={rc} duration={report['duration_s']:.3f}s {directory}")
    if "error" in report:
        print(report["error"])
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=tuple(FEATURES), default="normal")
    parser.add_argument("--timeout", type=int, default=90)
    parser.add_argument("--no-build", action="store_true", help="require an existing source-matching build")
    args = parser.parse_args()
    if not args.no_build:
        from build import build
        build(None, FEATURES[args.case], FEATURES[args.case])
    raise SystemExit(0 if run_network_service(case=args.case, timeout=args.timeout)["passed"] else 1)
