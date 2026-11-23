#!/usr/bin/env python3
"""N02 gate: the ring-3 network service configures itself by DHCP on the wire.

Same isolation, grants and causal localhost Ethernet peer as N01, but the
service starts unconfigured, acquires 192.0.2.66/24 by DHCP, withdraws it on a
DHCPNAK (app A's held token goes stale and OpenTcp is refused) and re-acquires
192.0.2.67 for a fresh exchange. App A resolves service.test through the
lease DNS server under its UDP/53 grant (NXDOMAIN cached, spoofs dropped and
counted, cache flushed by the withdrawal); app B has no such grant and is
refused. Link control and lease expiry are out of scope.
Images, user ELFs, firmware and serial bytes are bound to one unchanged source.
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
import time
import uuid
from common import (ROOT, BUILD, environment, firmware, qemu_machine, sha256,
                    source_hash, without_acpi_diagnostics)
from build_users import inspect_user
from run_fpu import private_path, publish_json, load_json
from run_system import check_esp, network_arguments
from run_network_service import BOOT, FORBIDDEN, REQUIRED, execute_native, integer, record_streams
from network_dhcp_peer import LEASES, Peer, validate_transcript

FEATURE = "test-net-service-dhcp"
LAST_TICK = 6000
OFFSETS = (240, 400, 408, 416, 424, 432, 440, 448, 456, 464, 472, 480, 488, 496, 504, 512, 520, 528,
           536, 544, 552, 560, 568, 576, 584, 592, 600, 608, 616, 624, 632, 640, 648)
ROLES = ("SERVICE", "APP_A", "APP_B")
NETWORK = ("NET_TX_SUBMITTED", "NET_TX_COMPLETED", "NET_RX_COMPLETED", "NET_RX_DELIVERED",
           "NET_DENIED", "NET_TX_TIMEOUTS", "NET_RESETS", "NET_RESET_ACKS")


def sequence(stdout):
    if not isinstance(stdout, str) or len(stdout) > 128 * 1024 or any(s in stdout for s in FORBIDDEN):
        raise ValueError("Failed/oversized actual native serial")
    lines = without_acpi_diagnostics(stdout).splitlines()
    if any(lines.count(marker) != 1 for marker in REQUIRED):
        raise ValueError("Missing/duplicate required native marker")
    start = lines.index(BOOT[0])
    if any(re.search(r"(?:BOOT:|BOOTINFO:|KERNEL:|KS-|M[0-9]+:|NETSVC:)", line) for line in lines[:start]):
        raise ValueError("Native marker before actual EBS")
    if tuple(lines[start:start + len(BOOT)]) != BOOT or lines.index("NETSVC:MANIFEST:PASS") != start + len(BOOT):
        raise ValueError("Native boot/manifest order differs")
    if lines[-3:] != ["NETSVC:CLEANUP:PASS", "NETSVC:PASS", "M1:PASS"]:
        raise ValueError("Native clean ending missing")
    found, network, exit_ticks, entropy = {r: {} for r in ROLES}, {}, {}, None
    numeric = re.compile(r"NETSVC:(SERVICE|APP_A|APP_B):([0-9]+):([0-9]{20})")
    for line in lines[start + len(BOOT) + 1:-3]:
        if match := re.fullmatch(r"NETSVC:(APP_A|APP_B)_EXIT_TICKS:([0-9]{20})", line):
            name, value = match.groups()
            if name in exit_ticks:
                raise ValueError("Duplicate actual exit tick")
            exit_ticks[name] = integer(int(value), 1, LAST_TICK)
        elif match := re.fullmatch(r"NETSVC:ENTROPY_BYTES:([0-9]{20})", line):
            if entropy is not None:
                raise ValueError("Duplicate entropy observation")
            entropy = integer(int(match.group(1)))
        elif match := numeric.fullmatch(line):
            role, offset, value = match.groups()
            if int(offset) not in OFFSETS or int(offset) in found[role]:
                raise ValueError("Unknown/duplicate native process observation")
            found[role][int(offset)] = integer(int(value))
        elif match := re.fullmatch(r"SYSTEM:(NET_[A-Z_]+):([0-9]{20})", line):
            name, value = match.groups()
            if name not in NETWORK or name in network:
                raise ValueError("Unknown/duplicate broker observation")
            network[name] = integer(int(value))
        else:
            raise ValueError("Unknown/reordered native evidence: " + line)
    if (any(set(values) != set(OFFSETS) for values in found.values()) or set(network) != set(NETWORK)
            or set(exit_ticks) != {"APP_A", "APP_B"} or entropy is None):
        raise ValueError("Incomplete native process, broker, exit or entropy evidence")
    service = found["SERVICE"]
    if (service[560] != 2 or service[568] != 1 or service[608] != 0
            or entropy != 8 + 4 * service[600] or not 0 < service[592] <= service[616] <= LAST_TICK):
        raise ValueError("Native DHCP lease/withdrawal/entropy evidence differs")
    return {"processes": {r: {str(k): v for k, v in values.items()} for r, values in found.items()},
            "network": network, "entropy_bytes": entropy, "exit_ticks": exit_ticks}


def validate_build(frozen):
    manifest = load_json(private_path(BUILD / "manifest.json", regular=True))
    if (manifest.get("kernel_feature") != FEATURE or manifest.get("loader_feature") is not None
            or manifest.get("source_sha256") != frozen or manifest.get("source_fingerprint_version") != 3
            or manifest.get("sdk_runtime") is not False or manifest.get("document_session") is not False):
        raise ValueError("Build the DHCP native service profile on the unchanged current source")
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


def evaluate(report, peer):
    if (type(report.get("raw_qemu_exit")) is not int or report["raw_qemu_exit"] != 33
            or report.get("timed_out") is not False or report.get("kernel_feature") != FEATURE
            or not isinstance(report.get("qemu_version"), str) or not report["qemu_version"].startswith("QEMU emulator version ")
            or type(report.get("duration_s")) not in (int, float) or not math.isfinite(report["duration_s"])
            or not 0 <= report["duration_s"] <= report["timeout_s"] + 5):
        raise ValueError("Native QEMU exit/lifetime/profile evidence failed")
    observed = sequence(report["stdout"])
    if (peer.get("passed") is not True or peer.get("connected") is not True
            or peer.get("error") is not None or peer.get("rejected_frame") is not None):
        raise ValueError("Actual synthetic DHCP peer failed: " + str(peer.get("error")))
    counts = validate_transcript(peer)["counts"]
    net, service = observed["network"], observed["processes"]["SERVICE"]
    if net["NET_TX_COMPLETED"] != counts["client_frames"] or net["NET_RX_DELIVERED"] != counts["server_frames"]:
        raise ValueError("Actual wire transcript contradicts broker TX/RX counts")
    if [service["576"], service["584"]] != [int.from_bytes(lease, "big") for lease in LEASES]:
        raise ValueError("Native leased addresses differ from the peer's leases")
    # DNS is the only guest UDP; each peer spoof must be a native malformed drop.
    if service["448"] != counts["dns_queries"] or service["512"] != counts["dns_spoofs"]:
        raise ValueError("Native DNS query/spoof counters contradict the peer transcript")
    return observed


def run_network_dhcp(*, timeout=90, frozen=None):
    if type(timeout) is not int or not 1 <= timeout <= 180:
        raise ValueError("Native timeout must be bounded 1..180 seconds")
    private_path(BUILD)
    frozen = source_hash() if frozen is None else frozen
    manifest = validate_build(frozen)
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
    directory = private_path(BUILD / "run" / ("network-dhcp-" + uuid.uuid4().hex))
    directory.mkdir(parents=True, exist_ok=False)
    shutil.copytree(BUILD / "esp", directory / "esp")
    check_esp(directory / "esp", manifest)
    for path, name in ((code, "OVMF_CODE.fd"), (variables, "OVMF_VARS.template.fd")):
        shutil.copy2(path, private_path(directory / name))
    # Boot only firmware bytes that match the recorded digests; VARS starts from the verified template.
    if (sha256(directory / "OVMF_CODE.fd"), sha256(directory / "OVMF_VARS.template.fd")) != (
            original_firmware["code"], original_firmware["vars_template"]):
        raise ValueError("Firmware changed while it was copied")
    shutil.copy2(directory / "OVMF_VARS.template.fd", private_path(directory / "OVMF_VARS.fd"))
    if sha256(directory / "OVMF_VARS.fd") != original_firmware["vars_template"]:
        raise ValueError("Initial writable firmware variables differ from the template")
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
    report = dict(schema_version=1, passed=False, case="dhcp", kernel_feature=FEATURE,
                  scope="three actual isolated ring-3 processes; DHCPv4 acquire/NAK withdrawal/re-acquire on synthetic localhost Ethernet; A-record DNS through the lease server with one synthetic responder; no link control, lease expiry, TCP DNS, DNSSEC, TLS or physical hardware claim",
                  source_commit=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
                  source_fingerprint_version=3, source_sha256=frozen, manifest=manifest,
                  run_directory=str(directory), timeout_s=timeout, started_utc=datetime.now(timezone.utc).isoformat(),
                  qemu_sha256=qemu_hash, firmware_sha256=original_firmware, snapshot_sha256=snapshots,
                  qemu_version=subprocess.check_output([qemu, "--version"], env=env, text=True, timeout=10).strip())
    with Peer(timeout=timeout) as peer:
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
        report["observed"] = evaluate(report, evidence)
        if load_json(private_path(directory / "network-peer.json", regular=True)) != evidence:
            raise ValueError("Stored peer transcript differs from actual server bytes")
        check_esp(directory / "esp", manifest)
        if {name: sha256(private_path(directory / name, regular=True)) for name in snapshots} != snapshots:
            raise ValueError("Immutable firmware/ESP snapshot changed during run")
        if {"code": sha256(code), "vars_template": sha256(variables)} != original_firmware or sha256(Path(qemu)) != qemu_hash:
            raise ValueError("QEMU or original firmware changed during run")
        if validate_build(frozen) != manifest or source_hash() != frozen:
            raise ValueError("Source/build/user artifacts changed during run")
        report.update(passed=True, network_dhcp_gate=True)
    except (ValueError, KeyError, TypeError, OSError) as error:
        report.update(error=f"{type(error).__name__}: {error}", network_dhcp_gate=False)
    publish_json(directory / "result.json", report)
    publish_json(BUILD / "network-dhcp-report.json", report)
    print(f"N02 native DHCP passed={report['passed']} raw={rc} duration={report['duration_s']:.3f}s {directory}")
    if "error" in report:
        print(report["error"])
    return report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout", type=int, default=90, help="host timeout seconds (default 90)")
    parser.add_argument("--no-build", action="store_true", help="require an existing source-matching build")
    args = parser.parse_args(argv)
    # The stable summary never outlives a newer attempt; each run keeps its own run directory.
    summary = BUILD / "network-dhcp-report.json"
    private_path(BUILD).mkdir(parents=True, exist_ok=True)
    failure = dict(schema_version=1, passed=False, case="dhcp", kernel_feature=FEATURE, network_dhcp_gate=False,
                   error="N02 build/run did not complete")
    publish_json(summary, failure)
    try:
        # Fingerprint before the build, so the post-run check compares against the build-start source.
        frozen = source_hash()
        if not args.no_build:
            from build import build
            build(None, FEATURE, FEATURE)
        return 0 if run_network_dhcp(timeout=args.timeout, frozen=frozen)["passed"] else 1
    except BaseException as error:
        failure["error"] = f"{type(error).__name__}: {error}"
        publish_json(summary, failure)
        raise


if __name__ == "__main__":
    raise SystemExit(main())
