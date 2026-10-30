#!/usr/bin/env python3
"""FR-I01: real joint-service guest and cold recovery on a guarded test image.

This is an integration gate, not the full release runner. It never accepts an
arbitrary disk and independently checks the actual volume after the guest exits.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import re
import shutil
import stat
import subprocess
import time
import uuid

from common import ROOT, BUILD, environment, firmware, qemu_machine, save_json, sha256, source_hash
from run_volume import execute
import volume_disk

FEATURE = {"normal": "native-system", "recover": "test-system-recover"}
RANDOM_FEATURE = {"raw":"native-system-random", "sdk":"test-system-random-sdk", "unknown":"test-system-random-unknown", "wrong-caller":"test-system-random-wrong-caller"}
TCP_FEATURE = {"normal": "native-system-tcp", "recover": "test-system-tcp-recover"}
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")
MAX_TIMEOUT = 180


def check_esp(esp: Path, manifest: dict) -> None:
    """Check the entire source tree before copying any byte into guest media."""
    esp = esp.absolute()
    for parent in (esp, *esp.parents):
        details = parent.lstat()
        if not stat.S_ISDIR(details.st_mode):
            raise ValueError("ESP directory/ancestor must be a real directory")
    expected = {"kernel.elf", "EFI/BOOT/BOOTX64.EFI"}
    files = set()
    for entry in esp.rglob("*"):
        details = entry.lstat()
        relative = entry.relative_to(esp).as_posix()
        if stat.S_ISDIR(details.st_mode):
            if relative not in {"EFI", "EFI/BOOT"}:
                raise ValueError("Unexpected ESP directory")
        elif stat.S_ISREG(details.st_mode) and details.st_nlink == 1:
            files.add(relative)
        else:
            raise ValueError("Linked or special ESP entry")
    if files != expected:
        raise ValueError("ESP must contain exactly the inspected loader and kernel")
    for relative, key in (("kernel.elf", "kernel"), ("EFI/BOOT/BOOTX64.EFI", "loader")):
        path = esp / relative
        details = path.lstat()
        if not stat.S_ISREG(details.st_mode) or details.st_nlink != 1:
            raise ValueError(f"ESP {relative} must be a regular single-link copy")
        if sha256(path) != manifest[key]["sha256"]:
            raise ValueError(f"ESP {relative} differs from the inspected build manifest")


def check_image(image: Path) -> None:
    """Reject links/devices/directories before opening or hashing the disk."""
    details = image.lstat()
    if (not stat.S_ISREG(details.st_mode) or details.st_nlink != 1 or
            details.st_size != volume_disk.IMAGE_SECTORS * volume_disk.SECTOR):
        raise ValueError("test disk must be a regular single-link generated image of the exact geometry")


def network_arguments(port: int) -> list[str]:
    if type(port) is not int or not 1024 <= port <= 65535:
        raise ValueError("network test peer requires an ephemeral local port")
    # OVMF can transmit IPv6 during boot, before our driver exists. Disable its
    # network drivers via the documented EDK2 runtime policy, plus the NIC ROM.
    # The PCI NIC remains present for the real Nieuw OS driver and strict peer.
    return ["-fw_cfg", "name=opt/org.tianocore/VirtioNetSupport,string=no",
            "-fw_cfg", "name=opt/org.tianocore/IPv4Support,string=no",
            "-fw_cfg", "name=opt/org.tianocore/IPv6Support,string=no",
            "-netdev", f"socket,id=net0,connect=127.0.0.1:{port}",
            "-device", "virtio-net-pci,netdev=net0,id=net,addr=0x7,disable-legacy=on,mac=02:00:00:00:00:07,romfile="]


def check_parent(previous: dict, frozen: str, image_sha256: str, *, network: bool, tcp=False) -> None:
    if tcp and not network:
        raise ValueError("TCP recovery requires the native network profile")
    parent_feature = TCP_FEATURE["normal"] if tcp else ("native-system-net" if network else FEATURE["normal"])
    if not (previous.get("case") == "normal" and previous.get("passed") is True
            and previous.get("kernel_feature") == parent_feature
            and previous.get("source_sha256") == frozen
            and previous.get("after_sha256") == image_sha256):
        raise ValueError("recovery needs the unchanged image from a passing normal run")
    # Generic system assertions precede the peer's final transcript/shutdown
    # verdict. A process crash in that interval cannot authorize net recovery.
    if network and previous.get("network_gate") is not True:
        raise ValueError("network recovery requires the completed network peer gate")
    if tcp and previous.get("tcp_gate") is not True:
        raise ValueError("TCP recovery requires the completed TCP peer gate")


def expected_markers(case: str) -> tuple[str, ...]:
    if case not in FEATURE:
        raise ValueError("unknown system case")
    return BOOT + ("SYSTEM:MANIFEST:PASS", "SYSTEM:BROKERS:PASS",
                   "SYSTEM:CLEANUP:PASS", f"SYSTEM:CASE:{case.upper()}:PASS", "M1:PASS")


def evaluate(case: str, stdout: str, raw_exit: int | None, timed_out: bool) -> dict[str, int]:
    expected = expected_markers(case)
    controlled = [line for line in stdout.splitlines()
                  if line in BOOT or line == "M1:PASS" or
                  line.startswith(("SYSTEM:MANIFEST:", "SYSTEM:BROKERS:",
                                   "SYSTEM:CLEANUP:", "SYSTEM:CASE:"))]
    if controlled != list(expected):
        raise ValueError(f"system marker sequence mismatch: {controlled}")
    if timed_out or raw_exit != 33:
        raise ValueError(f"raw exit {raw_exit}, timed_out {timed_out}")
    if any("PANIC" in line or "FAIL" in line for line in stdout.splitlines()):
        raise ValueError("guest failure marker")
    found = {}
    for line in stdout.splitlines():
        match = re.fullmatch(r"SYSTEM:([A-Z_]+):([0-9]{20})", line)
        if match:
            key = match.group(1)
            if key in found:
                raise ValueError(f"duplicate system evidence {key}")
            found[key] = int(match.group(2))
    # Checked against real IPC/syscall/driver observations in the kernel, not
    # a host-side substitute for OS functionality. The disk is checked below.
    exact = {"APP_READY": 2, "APP_DOCS": 1, "APP_REVISION": 1, "APP_BYTES": 4096,
             "APP_DENIED": 3, "WORKER_SPAWNS": 2, "WORKER_FAULTS": 1,
             "WORKER_RESTARTS": 1, "WORKER_KILLS": 1,
             "LIVE_PROCESSES_AFTER": 0, "FRAMES_RECOVERED": 1,
             "CRYPTO_SEAL": 2, "CRYPTO_UNSEAL": 2, "CRYPTO_TAMPER_DENIED": 9,
             "RECORD_SEAL": 1, "RECORD_UNSEAL": 1,
             "RECORD_CONTEXT_DENIED": 10, "RECORD_REVOKED_DENIED": 2}
    minimum = {"APP_DENIED": 3, "DISK_READS": 1, "ENTROPY_BYTES": 256,
               "CRYPTO_SEAL": 1, "CRYPTO_UNSEAL": 1, "CRYPTO_TAMPER_DENIED": 1}
    for key, value in exact.items():
        if found.get(key) != value:
            raise ValueError(f"{key}: expected {value}, found {found.get(key)}")
    for key, value in minimum.items():
        if found.get(key, -1) < value:
            raise ValueError(f"{key}: expected at least {value}, found {found.get(key)}")
    if case == "normal" and found.get("DISK_WRITES", 0) < 1:
        raise ValueError("normal guest did not write")
    if case == "recover" and found.get("DISK_WRITES") != 0:
        raise ValueError("recovery guest attempted a write")
    if (case == "normal" and found.get("DISK_FLUSHES", 0) < 2 or
            case == "recover" and found.get("DISK_FLUSHES") != 0):
        raise ValueError("wrong durability/readonly flush evidence")
    expected_sum = volume_disk.fnv(volume_disk.fixture(0, 1, 4096))
    if found.get("APP_CONTENT_SUM") != expected_sum:
        raise ValueError("guest read content differs from the independent fixture")
    return found


def check_disk(oracle: dict) -> None:
    objects = oracle["objects"]
    item = objects[0]
    expected = {"revision": 1, "length": 4096, "name": "systeemproef", "content_revision": 1}
    if not item["present"] or item["revision"] != 1 or len(item["versions"]) != 1:
        raise ValueError("missing or incorrect committed system document")
    if any(item["versions"][0].get(key) != value for key, value in expected.items()):
        raise ValueError("system document bytes/name/revision differ from oracle")
    if (item["collection"], item["tags"], item["kind"]) != (0, 0, 0):
        raise ValueError("unexpected system document metadata")
    if any(obj["present"] or obj["revision"] for obj in objects[1:]):
        raise ValueError("guest touched another object or domain")
    if any(collection["present"] or collection["revision"] for collection in oracle["collections"]):
        raise ValueError("guest changed an unrelated collection")
    if (oracle["generation"], oracle["chunks_used"], oracle["clocks"]) != (1, 1, [1, 0]):
        raise ValueError("unexpected volume generation, allocation or domain clock")


def run_system(case: str, volume_dir: Path | None = None, *, timeout: int = MAX_TIMEOUT,
               kernel_feature: str | None = None, network_port: int | None = None):
    if case not in FEATURE or not 0 < timeout <= MAX_TIMEOUT:
        raise ValueError("unknown case or unbounded timeout")
    selected = FEATURE[case] if kernel_feature is None else kernel_feature
    network_feature = {"normal": "native-system-net", "recover": "test-system-net-recover"}[case]
    diagnostic = case == "normal" and selected in (*RANDOM_FEATURE.values(), "native-system-images")
    if selected not in (FEATURE[case], network_feature, TCP_FEATURE[case]) and not diagnostic:
        raise ValueError("wrong native system feature for this case")
    use_tcp = selected == TCP_FEATURE[case]
    use_network = selected == network_feature or use_tcp
    if use_network != (type(network_port) is int and 1024 <= network_port <= 65535):
        raise ValueError("network profile requires a dedicated local test peer port")
    if not use_network and network_port is not None:
        raise ValueError("ordinary system profile has no networking")
    from run_fpu import private_path
    private_path(BUILD)
    frozen = source_hash()
    manifest = json.loads((BUILD / "manifest.json").read_text())
    if manifest.get("kernel_feature") != selected or manifest.get("source_sha256") != frozen:
        raise ValueError(f"build {selected} on the current source first")
    if not manifest.get("system"):
        raise ValueError("missing inspected system-program manifest")
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("qemu-system-x86_64 not found")
    code, variables = firmware()
    # Reject unknown files/links in the source before copytree can follow them.
    # Checking only the copied known files would expose extra host data to FAT.
    check_esp(BUILD / "esp", manifest)
    run_dir = private_path(BUILD / "run" / f"system-{case}-{uuid.uuid4().hex}")
    run_dir.mkdir(parents=True, exist_ok=False)
    # Runners only support generated paths inside this checkout.
    run_dir.resolve().relative_to(ROOT.resolve())
    shutil.copytree(BUILD / "esp", run_dir / "esp")
    check_esp(run_dir / "esp", manifest)
    shutil.copy2(code, run_dir / "OVMF_CODE.fd")
    shutil.copy2(variables, run_dir / "OVMF_VARS.fd")
    if case == "normal":
        if volume_dir is not None:
            raise ValueError("normal always creates a new disposable disk")
        volume_dir = BUILD / "run" / f"system-image-{uuid.uuid4().hex}"
        volume_dir.mkdir(parents=True, exist_ok=False)
        volume_disk.create_image(volume_dir / "volume.img", "empty")
    else:
        if volume_dir is None:
            raise ValueError("cold recovery needs the previous normal image")
        volume_dir = volume_dir.resolve(strict=True)
        volume_dir.relative_to((BUILD / "run").resolve())
        check_image(volume_dir / "volume.img")
        previous = json.loads((volume_dir / "last-system-stage.json").read_text())
        check_parent(previous, frozen, sha256(volume_dir / "volume.img"), network=use_network, tcp=use_tcp)
    image = volume_dir / "volume.img"
    check_image(image)
    before = sha256(image)
    rel = run_dir.relative_to(ROOT).as_posix()
    disk_rel = image.relative_to(ROOT).as_posix()
    readonly = "on" if case == "recover" else "off"
    command = [qemu, "-machine", qemu_machine(), "-accel", "tcg,thread=single", "-cpu", "qemu64",
               "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
               "-serial", "stdio", "-nic", "none", "-no-reboot",
               "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
               "-drive", f"if=pflash,unit=1,format=raw,file={rel}/OVMF_VARS.fd",
               "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
               "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
               "-drive", f"if=none,id=volume,format=raw,cache=writeback,readonly={readonly},file={disk_rel}",
               "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-SYSTEM-TEST",
               "-object", "rng-random,id=rng0,filename=/dev/urandom",
               "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000"]
    if use_network:
        # An ephemeral port is supplied by the owned synthetic protocol peer.
        # No TAP, bridge, default user network, host share or public listener.
        command += network_arguments(network_port)
    started = datetime.now(timezone.utc).isoformat()
    begin = time.monotonic()
    rc, stdout, stderr, timed_out, _, _ = execute(command, env, timeout=timeout)
    duration = time.monotonic() - begin
    (run_dir / "serial.txt").write_text(stdout)
    (run_dir / "stderr.txt").write_text(stderr)
    after = sha256(image)
    report = dict(schema_version=1, case=case, source_commit=subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), source_fingerprint_version=3,
        source_sha256=frozen, kernel_feature=selected, started_utc=started,
        duration_s=duration, timeout_s=timeout, raw_qemu_exit=rc, timed_out=timed_out,
        command=command, qemu_version=subprocess.check_output([qemu, "--version"], env=env, text=True).strip(),
        manifest=manifest, run_directory=str(run_dir), volume_directory=str(volume_dir),
        before_sha256=before, after_sha256=after,
        firmware_sha256={"code": sha256(code), "vars_template": sha256(variables)},
        esp_sha256={p.relative_to(run_dir).as_posix(): sha256(p)
                    for p in (run_dir / "esp").rglob("*") if p.is_file()}, passed=False)
    if use_network:
        report["network_gate"] = False
    if use_tcp:
        report["tcp_gate"] = False
    try:
        report["evidence"] = evaluate(case, stdout, rc, timed_out)
        if case == "recover" and before != after:
            raise ValueError("read-only cold recovery changed the disk")
        oracle = volume_disk.analyse(image.read_bytes())
        check_disk(oracle)
        report["oracle"] = oracle
        if source_hash() != frozen:
            raise ValueError("source changed during guest run")
        report["passed"] = True
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    save_json(run_dir / "result.json", report)
    save_json(volume_dir / "last-system-stage.json", report)
    shutil.copy2(image, run_dir / "volume.after.img")
    print(f"SYSTEM {case}: passed={report['passed']} raw={rc} duration={duration:.1f}s {run_dir}")
    if "error" in report:
        print(report["error"])
    return report, volume_dir


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=tuple(FEATURE), required=True)
    parser.add_argument("--volume", type=Path)
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    options = parser.parse_args()
    raise SystemExit(0 if run_system(options.case, options.volume, timeout=options.timeout)[0]["passed"] else 1)
