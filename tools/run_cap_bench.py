#!/usr/bin/env python3
"""Read-only diagnostic digest comparison on the real kernel, not GUI acceptance."""
from __future__ import annotations
import argparse
import hashlib
import json
import re
import shutil
import stat
import subprocess
import time
import uuid
from pathlib import Path

from common import (ROOT, BUILD, TARGET, environment, firmware, qemu_machine, sha256, source_hash,
                    without_acpi_diagnostics, DEVICE_EXECUTION_MARKER, EXPECTED_ACPI_MARKER)
from run_fpu import private_path, publish_json, publish
from run_system import check_esp
from build_users import inspect_user
import volume_disk

FEATURE = "native-cap-digest-bench"
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "M2:PASS", "SYSTEM:MANIFEST:PASS", "CAPBENCH:MANIFEST:PASS")
HEADER = ("DIGEST", "PROCESSES", "HANDLES", "NODES", "ENDPOINTS", "WARMUP", "ROUNDS", "BATCH")
VARIANTS = ("REF", "DIRECT", "ZERO")
FIELDS = HEADER + tuple(f"{v}_{k}" for v in VARIANTS for k in ("COUNT", "TSC_SUM", "TSC_MIN", "TSC_MAX"))
END = ("CAPBENCH:PARITY:PASS", "CAPBENCH:STATE:PASS", "CAPBENCH:PASS", "M1:PASS")


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def evaluate(serial: str, raw_exit: int, timed_out: bool) -> dict:
    require(type(raw_exit) is int and raw_exit == 33 and timed_out is False, "Native benchmark did not exit successfully")
    require(len(serial.encode()) < 128 * 1024 and not any(t in serial for t in ("FAIL", "PANIC", "\0")), "Invalid serial")
    lines = without_acpi_diagnostics(serial).splitlines()
    require(lines.count(BOOT[0]) == 1, "Missing/repeated boot")
    start = lines.index(BOOT[0])
    require(not any(re.search(r"BOOT:|KERNEL:|M[0-9]+:|CAPBENCH:", l) for l in lines[:start]), "Forged boot prefix")
    lines = lines[start:]
    require(tuple(lines[:len(BOOT)]) == BOOT, "Unexpected kernel manifest")
    at = len(BOOT)
    values = {}
    for name in FIELDS:
        match = re.fullmatch(re.escape("CAPBENCH:" + name + ":") + r"([0-9]{20})", lines[at] if at < len(lines) else "")
        require(match is not None and int(match[1]) < 1 << 64, "Missing/altered numeric field " + name)
        values[name] = int(match[1]); at += 1
    require([values[n] for n in HEADER[1:]] == [4, 25, 35, 11, 4, 9, 16], "Wrong native manifest or measurement workload")
    rounds = {v: [] for v in VARIANTS}
    execution_order = []
    for round_index in range(values["ROUNDS"]):
        order = []
        for position in range(len(VARIANTS)):
            match = re.fullmatch(r"CAPBENCH:ROUND_VARIANT:([0-9]{20})", lines[at] if at < len(lines) else "")
            expected = (round_index + position) % len(VARIANTS)
            require(match is not None and int(match[1]) == expected, "Missing/altered execution order")
            order.append(VARIANTS[expected]); at += 1
        execution_order.append(order)
        for name in VARIANTS:
            match = re.fullmatch(f"CAPBENCH:ROUND_{name}_TSC:([0-9]{{20}})", lines[at] if at < len(lines) else "")
            require(match is not None and 0 < int(match[1]) < 1 << 64, "Missing/zero/reordered batch")
            rounds[name].append(int(match[1])); at += 1
    require(tuple(lines[at:]) == END, "Unexpected benchmark trailer")
    for name in VARIANTS:
        require(values[name + "_COUNT"] == values["ROUNDS"] * values["BATCH"] and values[name + "_TSC_SUM"] == sum(rounds[name])
                and values[name + "_TSC_MIN"] == min(rounds[name]) and values[name + "_TSC_MAX"] == max(rounds[name]), "Totals do not match measured batches")
    return {"values": values, "rounds": rounds, "execution_order": execution_order,
            "tsc_ratio_reference_over_direct": values["REF_TSC_SUM"] / values["DIRECT_TSC_SUM"],
            "tsc_ratio_reference_over_zero": values["REF_TSC_SUM"] / values["ZERO_TSC_SUM"]}


SOURCE_FILES = (
    "tools/run_cap_bench.py", "tools/tests/test_cap_bench.py", "tools/build.py",
    "kernel/src/m2/m3/cap_bench.rs", "kernel/src/m2/m3.rs", "kernel/Cargo.toml", "kernel/build.rs",
    "crates/capability-core/src/lib.rs", "crates/capability-core/Cargo.toml",
    "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo/config.toml",
)
MAX_INPUT = 64 * 1024 * 1024


def digest(data):
    return hashlib.sha256(data).hexdigest()


def json_bytes(value):
    return (json.dumps(value, indent=2, ensure_ascii=False) + "\n").encode()


def manifest_bytes(data):
    def unique(pairs):
        values = {}
        for key, value in pairs:
            require(key not in values, "Duplicate manifest field")
            values[key] = value
        return values
    require(len(data) <= 1024 * 1024, "Oversized manifest")
    result = json.loads(data.decode("utf-8"), object_pairs_hook=unique)
    require(isinstance(result, dict), "Manifest must be an object")
    return result


class FrozenInputs:
    """Expected source bytes are frozen BEFORE copy, never learned after VM."""
    def __init__(self):
        self.sources = {}
        self.expected = {}
        self.payloads = {}

    @staticmethod
    def guard(path, *, cargo=False, external=False):
        path = Path(path).absolute()
        if not external:
            private_path(path, regular=True, cargo_output=cargo)
        # Selected host executable/firmware may use an explicit installation
        # link. Its resolved target is recorded and must remain identical.
        resolved = path.resolve(strict=True)
        details = resolved.stat()
        require(stat.S_ISREG(details.st_mode) and details.st_size <= MAX_INPUT,
                "Input must be a bounded regular file")
        return path, resolved

    def source(self, label, path, *, cargo=False, external=False):
        path, resolved = self.guard(path, cargo=cargo, external=external)
        data = path.read_bytes()
        require(len(data) <= MAX_INPUT, "Input grew while reading")
        self.sources[label] = {"path": str(path), "resolved": str(resolved), "bytes": len(data),
                               "sha256": digest(data), "cargo_output": cargo, "external": external}
        return data

    def expect(self, name, data):
        require(name not in self.expected, "Duplicate snapshot destination")
        self.expected[name] = {"sha256": digest(data), "bytes": len(data)}
        self.payloads[name] = data

    def attest_sources(self):
        for entry in self.sources.values():
            path, resolved = self.guard(entry["path"], cargo=entry["cargo_output"], external=entry["external"])
            require(str(resolved) == entry["resolved"] and path.stat().st_size == entry["bytes"]
                    and sha256(path) == entry["sha256"], "Frozen source input changed: " + str(path))

    def publish(self, directory):
        for name, data in self.payloads.items():
            path = private_path(directory / name)
            path.parent.mkdir(parents=True, exist_ok=True)
            publish(path, data)

    def attest_copies(self, directory):
        for name, expected in self.expected.items():
            path = private_path(directory / name, regular=True)
            require(path.stat().st_size == expected["bytes"] and sha256(path) == expected["sha256"],
                    "Frozen guest/archive input changed: " + name)


def run(timeout=30):
    # Withdraw an earlier PASS as soon as a safe current-index publication is
    # possible, before timeout/TARGET/source validation or any launch attempt.
    private_path(BUILD)
    index = private_path(BUILD / "cap-bench-report.json")
    attempt = {"passed": False, "status": "starting", "scope": "kernel diagnostic comparison only"}
    publish_json(index, attempt)
    try:
        return _run(timeout, index)
    except Exception as error:
        attempt.update(status="failed", error=f"{type(error).__name__}: {error}")
        publish_json(index, attempt)
        raise


def _run(timeout, index):
    require(type(timeout) is int and 0 < timeout <= 60, "Unbounded timeout")
    private_path(TARGET)
    frozen = source_hash()
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    bindings = FrozenInputs()
    for name in SOURCE_FILES:
        bindings.source("source/" + name, ROOT / name)
    original_manifest = bindings.source("build/manifest", BUILD / "manifest.json")
    manifest = manifest_bytes(original_manifest)
    require(type(manifest.get("schema_version")) is int and manifest["schema_version"] == 1
            and manifest.get("source_fingerprint_version") == 3
            and manifest.get("kernel_feature") == FEATURE and manifest.get("source_sha256") == frozen
            and manifest.get("loader_feature") is None, "Build the frozen benchmark first")
    check_esp(BUILD / "esp", manifest)
    kernel = bindings.source("build/kernel", BUILD / "esp/kernel.elf")
    loader = bindings.source("build/loader", BUILD / "esp/EFI/BOOT/BOOTX64.EFI")
    bindings.expect("esp/kernel.elf", kernel)
    bindings.expect("esp/EFI/BOOT/BOOTX64.EFI", loader)
    bindings.expect("inputs/manifest.json", original_manifest)
    binaries = {"system_init": TARGET / "system-system/x86_64-unknown-none/release/system-init",
                "system_app": TARGET / "system-system/x86_64-unknown-none/release/system-app",
                "system_worker": TARGET / "system-system/x86_64-unknown-none/release/system-worker",
                "keyring_service": TARGET / "system-keyring/x86_64-unknown-none/release/keyring-service",
                "volume_service": TARGET / "system-volume/x86_64-unknown-none/release/volume-service"}
    require(set(manifest.get("system", {})) == set(binaries), "Unexpected process images")
    for name, path in binaries.items():
        data = bindings.source("elf/" + name, path, cargo=True)
        require(manifest["system"][name] == {"sha256": digest(data), "elf": inspect_user(path)}
                and data in kernel, "Unbound embedded ELF")
        bindings.expect("inputs/" + name + ".elf", data)
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    require(qemu is not None, "Missing QEMU")
    bindings.source("qemu", Path(qemu), external=True)
    qemu_hash = bindings.sources["qemu"]["sha256"]
    qemu_version = subprocess.check_output([qemu, "--version"], env=env, text=True).strip()
    code, variables = firmware()
    code_bytes = bindings.source("firmware/code", code, external=True)
    vars_bytes = bindings.source("firmware/template", variables, external=True)
    bindings.expect("OVMF_CODE.fd", code_bytes)
    bindings.expect("OVMF_VARS.template.fd", vars_bytes)
    bindings.attest_sources()
    require(source_hash() == frozen, "Source changed before snapshot publication")
    run_dir = private_path(BUILD / "run" / ("cap-bench-" + uuid.uuid4().hex))
    run_dir.mkdir(parents=True, exist_ok=False)
    # Copy only the already frozen approved bytes. No copytree dereferences an
    # unexpected source-ESP entry between validation and publication.
    bindings.publish(run_dir)
    publish(run_dir / "OVMF_VARS.fd", vars_bytes)
    image = run_dir / "volume.img"
    volume_disk.create_image(image, "empty")
    disk_before = sha256(image)
    for name in ("volume.img", "volume.json"):
        bindings.expect(name, (run_dir / name).read_bytes())
    initial_mutable = {"OVMF_VARS.fd": {"sha256": digest(vars_bytes), "bytes": len(vars_bytes)}}
    record = json_bytes({"source_sha256": frozen, "frozen_sources": bindings.sources,
                         "expected_artifacts": dict(bindings.expected), "initial_mutable_artifacts": initial_mutable})
    bindings.expect("inputs/frozen-inputs.json", record)
    publish(run_dir / "inputs/frozen-inputs.json", record)
    bindings.attest_copies(run_dir)
    bindings.attest_sources()
    check_esp(BUILD / "esp", manifest)
    check_esp(run_dir / "esp", manifest)
    command = [qemu, "-machine", qemu_machine(), "-accel", "tcg,thread=single", "-cpu", "qemu64", "-smp", "1", "-m", "256M",
               "-nodefaults", "-display", "none", "-monitor", "none", "-serial", "stdio", "-nic", "none", "-no-reboot",
               "-drive", "if=pflash,unit=0,format=raw,readonly=on,file=OVMF_CODE.fd",
               "-drive", "if=pflash,unit=1,format=raw,file=OVMF_VARS.fd", "-drive", "if=virtio,index=0,format=raw,readonly=on,file=fat:esp",
               "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
               "-drive", "if=none,id=volume,format=raw,cache=writeback,file=volume.img",
               "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-SYSTEM-TEST",
               "-object", "rng-random,id=rng0,filename=/dev/urandom",
               "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000"]
    # VARS is mutable only after this launch. Its initial contents must still
    # match the same frozen template used to publish the immutable archive.
    initial_vars = private_path(run_dir / "OVMF_VARS.fd", regular=True)
    require(initial_vars.stat().st_size == len(vars_bytes)
            and sha256(initial_vars) == digest(vars_bytes), "Initial writable firmware differs from frozen template")
    started = time.monotonic()
    try:
        completed = subprocess.run(command, env=env, cwd=run_dir, timeout=timeout,
                                   capture_output=True, text=True)
        raw, stdout, stderr, timed_out = completed.returncode, completed.stdout, completed.stderr, False
    except subprocess.TimeoutExpired as error:
        raw, timed_out = None, True
        stdout = error.stdout or b""
        stderr = error.stderr or b""
        if isinstance(stdout, bytes): stdout = stdout.decode("utf-8", "replace")
        if isinstance(stderr, bytes): stderr = stderr.decode("utf-8", "replace")
    report = {"passed": False, "source_commit": commit, "source_fingerprint_version": 3, "source_sha256": frozen,
              "source_files_sha256": {name: bindings.sources["source/" + name]["sha256"] for name in SOURCE_FILES},
              "frozen_sources": bindings.sources, "expected_artifacts": bindings.expected,
              "initial_mutable_artifacts": initial_mutable,
              "scope": "Read-only CPL0 digest cost in own kernel before userspace/timer; no GUI, app, boot-latency or physical-cycle acceptance",
              "command": command, "cwd": str(run_dir), "run_directory": str(run_dir), "manifest": manifest,
              "qemu_sha256": qemu_hash, "qemu_version": qemu_version,
              "firmware_sha256": {"code": digest(code_bytes), "vars_template": digest(vars_bytes)},
              "raw_qemu_exit": raw, "timed_out": timed_out, "duration_s": time.monotonic() - started,
              "disk_before_sha256": disk_before, "disk_after_sha256": None}
    publish(run_dir / "serial.txt", stdout.encode())
    publish(run_dir / "stderr.txt", stderr.encode())
    try:
        # A live run must carry the native ACPI pair; fixtures may omit it.
        live = stdout.splitlines()
        if live.count(EXPECTED_ACPI_MARKER) != 1 or live.count(DEVICE_EXECUTION_MARKER) != 1:
            raise ValueError("Missing native ACPI boot diagnostics")
        report["observed"] = evaluate(stdout, raw, timed_out)
        bindings.attest_sources()
        bindings.attest_copies(run_dir)
        # Attest regular single-link geometry before opening this post-run
        # input, as for every other immutable snapshot. Never follow a link
        # (or block on a special file) merely to populate a failure report.
        post_image = private_path(image, regular=True)
        require(post_image.stat().st_size == bindings.expected["volume.img"]["bytes"], "Wrong disk geometry")
        report["disk_after_sha256"] = sha256(post_image)
        require(source_hash() == frozen, "Source changed")
        check_esp(run_dir / "esp", manifest)
        check_esp(BUILD / "esp", manifest)
        writable = private_path(run_dir / "OVMF_VARS.fd", regular=True)
        require(writable.stat().st_size == len(vars_bytes), "Writable firmware geometry changed")
        actual = {}
        directories = {"esp", "esp/EFI", "esp/EFI/BOOT", "inputs"}
        for path in run_dir.rglob("*"):
            private_path(path)
            if path.is_dir():
                require(path.relative_to(run_dir).as_posix() in directories, "Unexpected artifact directory")
            else:
                actual[path.relative_to(run_dir).as_posix()] = sha256(path)
        require(set(actual) == set(bindings.expected) | {"OVMF_VARS.fd", "serial.txt", "stderr.txt"},
                "Unexpected/missing evidence file")
        report["artifact_sha256"] = actual
        report["passed"] = True
    except (ValueError, KeyError, OSError) as error:
        report["error"] = str(error)
    report["status"] = "completed" if report["passed"] else "failed"
    publish_json(run_dir / "result.json", report)
    publish_json(index, report)
    print(f"CAPBENCH passed={report['passed']} raw={raw}; {run_dir}")
    if "error" in report: print(report["error"])
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout", type=int, default=30)
    raise SystemExit(0 if run(parser.parse_args().timeout)["passed"] else 1)
