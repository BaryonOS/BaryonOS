#!/usr/bin/env python3
"""Revalidate and archive compact FR-I01 evidence without copying binaries."""
from __future__ import annotations

import argparse
from contextlib import suppress
import hashlib
import json
import math
from pathlib import Path
import re
import stat
import subprocess
import sys

from common import ROOT, BUILD, QEMU_MACHINES, source_hash
from run_system import FEATURE, MAX_TIMEOUT, OPAQUE_OBJECTS, check_disk, check_esp, check_image, evaluate, network_arguments
from run_net_system import FEATURE as NETWORK_FEATURE, evaluate_network
from run_tcp_system import FEATURE as TCP_FEATURE, evaluate_tcp
from test_system import inputs
from test_net_system import inputs as network_inputs
from test_tcp_system import inputs as tcp_inputs, host_commands as tcp_host_commands
import volume_disk

TEXT_LIMIT = 8 * 1024 * 1024
BINARY_LIMIT = 64 * 1024 * 1024


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON field {key}")
        result[key] = value
    return result


def _json(data: bytes) -> dict:
    def invalid_constant(value):
        raise ValueError(f"non-JSON constant {value}")
    value = json.loads(data.decode("utf-8"), object_pairs_hook=_pairs, parse_constant=invalid_constant)
    if not isinstance(value, dict):
        raise ValueError("expected a JSON object")
    return value


def _hash(value) -> str:
    if not isinstance(value, str) or re.fullmatch(r"[0-9a-f]{64}", value) is None:
        raise ValueError("invalid SHA-256")
    return value


def _path(value, root: Path) -> Path:
    path = Path(value)
    if ".." in path.parts:
        raise ValueError("parent traversal is not a generated path")
    return path if path.is_absolute() else root / path


def _guard(path: Path, base: Path, *, directory=False) -> Path:
    """Check every component without resolving an untrusted symlink first."""
    relative = path.relative_to(base)
    if not relative.parts:
        raise ValueError("expected a child path")
    current = base
    for index, part in enumerate(relative.parts):
        current /= part
        info = current.lstat()
        last = index == len(relative.parts) - 1
        if not last or directory:
            if not stat.S_ISDIR(info.st_mode):
                raise ValueError(f"generated directory is not a real directory: {current}")
        elif not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
            raise ValueError(f"evidence must be a regular single-link file: {current}")
    return path


class Snapshot:
    def __init__(self, root: Path):
        self.root = root
        self.seen: dict[Path, str] = {}

    def read(self, path: Path, limit: int = TEXT_LIMIT) -> bytes:
        _guard(path, self.root)
        if path.stat().st_size > limit:
            raise ValueError(f"evidence exceeds its size limit: {path}")
        data = path.read_bytes()
        if len(data) > limit:
            raise ValueError("evidence grew while reading")
        old = self.seen.setdefault(path, digest(data))
        if old != digest(data):
            raise ValueError(f"evidence changed during capture: {path}")
        return data

    def stable(self):
        for path in tuple(self.seen):
            self.read(path, BINARY_LIMIT)


def _manifest(manifest: dict, case: str, frozen: str, *, network=False, tcp=False):
    features = TCP_FEATURE if tcp else NETWORK_FEATURE if network else FEATURE
    if (manifest.get("schema_version") != 1 or manifest.get("source_fingerprint_version") != 3
            or manifest.get("source_sha256") != frozen
            or manifest.get("kernel_feature") != features[case]):
        raise ValueError("stale or wrong-profile manifest")
    for key in ("kernel", "loader"):
        _hash(manifest[key]["sha256"])
    system = manifest.get("system", {})
    for name in ("system_init", "system_app", "system_worker", "keyring_service", "volume_service"):
        _hash(system[name]["sha256"])
    if network or tcp:
        _hash(system["net_probe"]["sha256"])


def _esp(snapshot: Snapshot, esp: Path, manifest: dict) -> dict[str, str]:
    hashes = {}
    for relative in ("kernel.elf", "EFI/BOOT/BOOTX64.EFI"):
        hashes[f"esp/{relative}"] = digest(snapshot.read(esp / relative, BINARY_LIMIT))
    check_esp(esp, manifest)
    return hashes


def _command(command, case: str, run: Path, image: Path, root: Path, *, network=False):
    if not isinstance(command, list) or not all(isinstance(arg, str) for arg in command):
        raise ValueError("missing actual QEMU command")
    if (len(command) < 5 or Path(command[0]).name not in ("qemu-system-x86_64", "qemu-system-x86_64.exe")
            or command[1] != "-machine" or command[2] not in QEMU_MACHINES
            or command[3] != "-accel" or command[4] not in ("tcg,thread=single", "kvm")):
        raise ValueError("unsupported QEMU profile")
    rel = run.relative_to(root).as_posix()
    disk = image.relative_to(root).as_posix()
    readonly = "on" if case == "recover" else "off"
    expected = ["-machine", command[2], "-accel", command[4], "-cpu", "qemu64",
                "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
                "-serial", "stdio", "-nic", "none", "-no-reboot",
                "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
                "-drive", f"if=pflash,unit=1,format=raw,file={rel}/OVMF_VARS.fd",
                "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
                "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
                "-drive", f"if=none,id=volume,format=raw,cache=writeback,readonly={readonly},file={disk}",
                "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-SYSTEM-TEST",
                "-object", "rng-random,id=rng0,filename=/dev/urandom",
                "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000"]
    if network:
        peer = re.fullmatch(r"socket,id=net0,connect=127\.0\.0\.1:([0-9]+)", command[-3])
        if peer is None:
            raise ValueError("network command lacks its isolated local peer")
        expected += network_arguments(int(peer.group(1)))
    if command[1:] != expected:
        raise ValueError("QEMU command differs from guarded system profile or readonly cold recovery")


def _git_head(root: Path) -> str:
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()


def capture(destination: Path, *, network=False, tcp=False) -> Path:
    if network and tcp:
        raise ValueError("network and TCP captures are separate profiles")
    root = ROOT.resolve(strict=True)
    build = _guard(_path(BUILD, root), root, directory=True)
    parent = _guard(root / "docs/verification", root, directory=True)
    destination = _path(destination, root)
    if destination.parent != parent or re.fullmatch(r"full-release-[A-Za-z0-9][A-Za-z0-9._-]*", destination.name) is None:
        raise ValueError("destination must be a new docs/verification/full-release-<unique> directory")
    if destination.exists() or destination.is_symlink():
        raise FileExistsError(destination)
    input_function = tcp_inputs if tcp else network_inputs if network else inputs
    features = TCP_FEATURE if tcp else NETWORK_FEATURE if network else FEATURE
    gate_name = "tcp-system-gate.json" if tcp else "net-system-gate.json" if network else "system-gate.json"
    host_prefix = "tcp" if tcp else "net" if network else "system"
    frozen, test_inputs = source_hash(), input_function()
    _hash(frozen)
    snapshot = Snapshot(root)
    archive = {}
    gate_bytes = snapshot.read(build / gate_name)
    gate = _json(gate_bytes)
    if (gate.get("schema_version") != 1 or gate.get("source_fingerprint_version") != 3
            or gate.get("status") != "completed" or gate.get("passed") is not True
            or gate.get("source_sha256") != frozen or gate.get("test_inputs") != test_inputs):
        raise ValueError("only a completed, passing gate on current frozen source/test inputs can be captured")
    stages = gate.get("stages", [])
    if len(stages) != 2 or [stage.get("case") for stage in stages] != ["normal", "recover"]:
        raise ValueError("exactly normal and cold recover stages are required")
    hosts = gate.get("host", [])
    if len(hosts) != (5 if tcp else 3 if network else 2):
        raise ValueError("all profile host checks are required")
    archive[gate_name] = gate_bytes
    host_commands = [
        [sys.executable, "-B", "-m", "unittest", "discover", "-s", "tests", "-p", "test_system_*.py"],
        ["cargo", "+1.90.0", "test", "--locked", "--manifest-path", "sdk/Cargo.toml", "-p", "nieuw-os-sdk-runtime"],
    ]
    if tcp:
        host_commands = tcp_host_commands()
    elif network:
        host_commands[0][-1] = "test_net_*.py"
        host_commands.insert(1, ["cargo", "+1.90.0", "test", "--locked", "-p", "capability-core"])
    for index, host in enumerate(hosts):
        if (type(host.get("exit")) is not int or host["exit"] != 0
                or host.get("command") != host_commands[index]):
            raise ValueError("failed or incomplete host check")
        for stream in ("stdout", "stderr"):
            expected = build / "logs" / f"{host_prefix}-host-{index}.{stream}.txt"
            if _path(host[stream], root) != expected:
                raise ValueError("host log outside its generated slot")
            archive[f"host/{index}.{stream}.txt"] = snapshot.read(expected)
    verified = []
    volume_dir = None
    for index, stage in enumerate(stages):
        case = stage["case"]
        run = _path(stage["run_directory"], root)
        if run.parent != build / "run" or re.fullmatch(f"system-{case}-[0-9a-f]{{32}}", run.name) is None:
            raise ValueError("run is not a generated system child")
        _guard(run, root, directory=True)
        report_bytes = snapshot.read(run / "result.json")
        report = _json(report_bytes)
        for key in ("case", "passed", "run_directory", "raw_qemu_exit", "before_sha256", "after_sha256", "duration_s"):
            if stage.get(key) != report.get(key):
                raise ValueError(f"stage/report mismatch: {key}")
        if (report.get("schema_version") != 1 or report.get("source_fingerprint_version") != 3
                or report.get("case") != case or report.get("passed") is not True
                or report.get("source_sha256") != frozen or report.get("kernel_feature") != features[case]
                or not isinstance(report.get("source_commit"), str)
                or re.fullmatch(r"[0-9a-f]{40}", report["source_commit"]) is None):
            raise ValueError("run is incomplete or not from the frozen source")
        if type(report.get("raw_qemu_exit")) is not int or report.get("timed_out") is not False:
            raise ValueError("missing actual raw exit/timeout state")
        _hash(report["before_sha256"])
        duration, timeout = report["duration_s"], report["timeout_s"]
        if (type(duration) not in (int, float) or not math.isfinite(duration) or duration < 0
                or type(timeout) not in (int, float) or not 0 < timeout <= MAX_TIMEOUT):
            raise ValueError("invalid run timing")
        serial = snapshot.read(run / "serial.txt")
        evidence = evaluate(case, serial.decode("utf-8"), report["raw_qemu_exit"], report["timed_out"])
        if evidence != report.get("evidence"):
            raise ValueError("stored counters differ from raw guest evidence")
        manifest = report["manifest"]
        _manifest(manifest, case, frozen, network=network, tcp=tcp)
        esp = _esp(snapshot, run / "esp", manifest)
        if report.get("esp_sha256") != esp:
            raise ValueError("ESP bytes differ from the actual guest report")
        current_volume = _path(report["volume_directory"], root)
        if (current_volume.parent != build / "run"
                or re.fullmatch(r"system-image-[0-9a-f]{32}", current_volume.name) is None
                or volume_dir is not None and current_volume != volume_dir):
            raise ValueError("cold recovery did not use the same generated disk directory")
        volume_dir = current_volume
        _guard(volume_dir, root, directory=True)
        _command(report["command"], case, run, volume_dir / "volume.img", root, network=network or tcp)
        image_path = _guard(run / "volume.after.img", root)
        check_image(image_path)
        image = snapshot.read(image_path, BINARY_LIMIT)
        image_hash = digest(image)
        if _hash(report["after_sha256"]) != image_hash:
            raise ValueError("stage disk bytes differ from report")
        oracle = volume_disk.analyse(image, OPAQUE_OBJECTS)
        check_disk(oracle)
        if oracle != report.get("oracle"):
            raise ValueError("stored disk oracle differs from real image bytes")
        firmware_code = digest(snapshot.read(run / "OVMF_CODE.fd", BINARY_LIMIT))
        if firmware_code != report["firmware_sha256"]["code"]:
            raise ValueError("firmware code bytes differ from run report")
        vars_hash = digest(snapshot.read(run / "OVMF_VARS.fd", BINARY_LIMIT))
        prefix = f"{index}-{case}"
        archive[f"{prefix}/result.json"] = report_bytes
        archive[f"{prefix}/serial.txt"] = serial
        archive[f"{prefix}/stderr.txt"] = snapshot.read(run / "stderr.txt")
        if network or tcp:
            if stage.get("network_gate") is not True or report.get("network_gate") is not True:
                raise ValueError("network transcript/shutdown gate was not completed")
            if tcp and (stage.get("tcp_gate") is not True or report.get("tcp_gate") is not True):
                raise ValueError("TCP transcript/shutdown gate was not completed")
            peer_bytes = snapshot.read(run / "network-peer.json")
            peer = _json(peer_bytes)
            if peer != report.get("network_peer") or peer.get("rejected_frame") is not None:
                raise ValueError("stored peer transcript differs from the final network report")
            (evaluate_tcp if tcp else evaluate_network)(report, peer)
            archive[f"{prefix}/network-peer.json"] = peer_bytes
        verified.append(dict(case=case, report=report, disk_sha256=image_hash, esp_sha256=esp,
                             vars_after_sha256=vars_hash, oracle=oracle))
    normal, recover = verified
    cold = normal["disk_sha256"]
    if (recover["report"]["before_sha256"] != cold or recover["disk_sha256"] != cold
            or normal["report"]["before_sha256"] == cold):
        raise ValueError("cold disk chain changed, or normal run wrote no new image")
    image_path = _guard(volume_dir / "volume.img", root)
    check_image(image_path)
    if digest(snapshot.read(image_path, BINARY_LIMIT)) != cold:
        raise ValueError("current generated volume differs from the cold-chain bytes")
    if _json(snapshot.read(volume_dir / "last-system-stage.json")) != recover["report"]:
        raise ValueError("last stage does not describe this exact cold recovery")
    final_path = build / "manifest.json"
    if _path(gate["final_manifest"], root) != final_path:
        raise ValueError("final manifest outside the generated build slot")
    final_bytes = snapshot.read(final_path)
    final = _json(final_bytes)
    _manifest(final, "normal", frozen, network=network, tcp=tcp)
    _esp(snapshot, build / "esp", final)
    for key in ("kernel", "loader", "system"):
        if final[key] != normal["report"]["manifest"][key]:
            raise ValueError("final normal artifact differs from the tested normal manifest")
    if gate.get("final_kernel_sha256") != final["kernel"]["sha256"]:
        raise ValueError("final kernel differs from the gate artifact")
    archive["final-manifest.json"] = final_bytes
    validation = dict(schema_version=1, passed=True, source_sha256=frozen, test_inputs=test_inputs,
                      capture_commit=_git_head(root), boundary=("joint native TCP" if tcp else "joint native network" if network else "FR-I01") + " integration only; not full release acceptance",
                      stages=[{key: value for key, value in entry.items() if key != "report"}
                              for entry in verified],
                      source_commits=[entry["report"]["source_commit"] for entry in verified])
    archive["capture-validation.json"] = (json.dumps(validation, indent=2) + "\n").encode()

    def stable():
        snapshot.stable()
        if source_hash() != frozen or input_function() != test_inputs:
            raise ValueError("source/test inputs changed during capture")

    stable()
    # Exclusive creation prevents replacement of even an empty existing archive.
    # Roll back only paths created here; never recursively delete user files.
    destination.mkdir()
    created_files, created_dirs = [], []
    try:
        for name, data in archive.items():
            output = destination / name
            if output.parent != destination and not output.parent.exists():
                output.parent.mkdir()
                created_dirs.append(output.parent)
            with output.open("xb") as stream:
                created_files.append(output)
                stream.write(data)
        stable()
        for name, data in archive.items():
            if (destination / name).read_bytes() != data:
                raise ValueError("archive write differs from captured bytes")
        sums = "".join(f"{digest(data)}  {name}\n" for name, data in sorted(archive.items()))
        output = destination / "SHA256SUMS"
        with output.open("xb") as stream:
            created_files.append(output)
            stream.write(sums.encode("utf-8"))
    except Exception:
        for path in reversed(created_files):
            with suppress(OSError):
                path.unlink(missing_ok=True)
        for path in reversed(created_dirs):
            with suppress(OSError):
                path.rmdir()
        with suppress(OSError):
            destination.rmdir()
        raise
    return destination


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", type=Path, required=True)
    profiles = parser.add_mutually_exclusive_group()
    profiles.add_argument("--network", action="store_true", help="revalidate the isolated network pair and peer transcripts")
    profiles.add_argument("--tcp", action="store_true", help="revalidate the isolated native TCP pair and peer transcripts")
    options = parser.parse_args(argv)
    destination = capture(options.destination, network=options.network, tcp=options.tcp)
    print(f"captured two revalidated system runs into {destination}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
