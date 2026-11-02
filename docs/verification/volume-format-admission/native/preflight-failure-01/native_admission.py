"""Disposable-image native admission checks; no filesystem driver or host disk access.

Builds and runs only after an explicit expected source commit has been supplied.
All generated files stay beside this script. Existing candidate images are only
read and copied. Assertions use actual QEMU output plus complete image hashes.
"""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT.parent / "source"
PREVIOUS = ROOT.parent.parent / "native-storage-gate"
os.environ["PYTHONDONTWRITEBYTECODE"] = "1"
sys.dont_write_bytecode = True
sys.path.insert(0, str(SOURCE / "tools"))
from common import environment, firmware, source_hash  # noqa: E402
import volume_disk  # noqa: E402
from run_volume import expected_markers, controlled, numbers, check_state  # noqa: E402

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "VOLUME:MANIFEST:PASS")
CORRUPT = "VOLUME:SERVICE_ERROR:18446744073709551598"
STARTUP_FAILURE = "VOLUME:FAIL:SERVICE_STARTUP"
PROFILES = ("native-volume", "test-volume-recover")
CANDIDATES = {
    "btrfs": (PREVIOUS / "btrfs/baseline.img", "0f3e5d9631ac9e8747abd7d6744538772620c0d2d9c6c5f40f2ed7e864edac39"),
    "openzfs-leaf": (PREVIOUS / "openzfs/images/nieuwos_n3.0a", "23a409ac5e9a844056d9096ead0453112f1d1e6e45d4b7d5db084a397baea4ea"),
}


def sha(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def save(path, value):
    path = local(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def local(path):
    path = Path(path).resolve()
    path.relative_to(ROOT)
    return path


def source_state(expected_head):
    head = subprocess.check_output(["git", "-C", str(SOURCE), "rev-parse", "HEAD"], text=True).strip()
    if head != expected_head:
        raise RuntimeError(f"source HEAD changed: expected {expected_head}, got {head}")
    tests = SOURCE / "tests"
    paths = set(tests.rglob("test_volume_*.py")) | set(tests.rglob("__init__.py"))
    return {"head": head, "source_sha256": source_hash(),
            "volume_test_inputs": {p.relative_to(SOURCE).as_posix(): sha(p) for p in sorted(paths)},
            "runner_sha256": sha(__file__)}


def frozen(expected_head):
    current = source_state(expected_head)
    path = ROOT / "freeze.json"
    if path.exists():
        if json.loads(path.read_text(encoding="utf-8")) != current:
            raise RuntimeError("source/test/runner fingerprint differs from frozen checkpoint")
    else:
        save(path, current)
    return current


def build_environment():
    env = environment()
    code, variables = firmware()
    env.update(CARGO_HOME=str(PREVIOUS / "native/cargo-cache"),
               LOCALAPPDATA=str(ROOT / "process-local-appdata"),
               PYTHONDONTWRITEBYTECODE="1", RUSTUP_AUTO_INSTALL="0", CARGO_NET_OFFLINE="true",
               OS_OVMF_CODE=str(code), OS_OVMF_VARS=str(variables))
    return env


def build_profiles(expected_head):
    state = frozen(expected_head)
    env = build_environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("QEMU unavailable")
    trace = subprocess.run([qemu, "-trace", "help"], env=env, capture_output=True, text=True, timeout=30)
    trace_text = trace.stdout + trace.stderr
    for event in ("virtio_blk_handle_read", "virtio_blk_handle_write"):
        if event not in trace_text.splitlines():
            raise RuntimeError(f"missing QEMU trace event: {event}")
    versions = {"qemu": subprocess.check_output([qemu, "--version"], env=env, text=True).strip(),
                "rustc": subprocess.check_output(["rustc", "+1.90.0", "-vV"], env=env, text=True).strip(),
                "python": sys.version, "machine": "pc-q35-10.0", "accel": "tcg,thread=single", "smp": 1,
                "memory": "256M", "source": state}
    save(ROOT / "environment.json", versions)
    for profile in PROFILES:
        destination = local(ROOT / "builds" / profile)
        destination.mkdir(parents=True, exist_ok=False)
        profile_env = dict(env, NOS_BUILD_DIR=str(destination), NOS_TARGET_DIR=str(ROOT / "target"),
                           CARGO_TARGET_DIR=str(ROOT / "target"))
        command = [sys.executable, "-B", str(SOURCE / "tools/build.py"), "--kernel-feature", profile]
        started = time.monotonic()
        result = subprocess.run(command, cwd=SOURCE, env=profile_env, capture_output=True, timeout=600)
        (destination / "build.stdout.txt").write_bytes(result.stdout)
        (destination / "build.stderr.txt").write_bytes(result.stderr)
        report = {"command": command, "cwd": str(SOURCE), "returncode": result.returncode,
                  "duration_s": time.monotonic() - started, "source": state,
                  "source_unchanged": frozen(expected_head) == state}
        save(destination / "build-invocation.json", report)
        print(f"build {profile}: exit={result.returncode}", flush=True)
        if result.returncode:
            raise RuntimeError(f"build failed; see {destination}")


def prepare(expected_head, profile, label):
    if not label or any(ch not in "abcdefghijklmnopqrstuvwxyz0123456789-_" for ch in label):
        raise ValueError("invalid label")
    state = frozen(expected_head)
    directory = local(ROOT / "runs" / label)
    directory.mkdir(parents=True, exist_ok=False)
    built = ROOT / "builds" / profile
    manifest = json.loads((built / "manifest.json").read_text(encoding="utf-8"))
    if manifest.get("kernel_feature") != profile or manifest.get("source_sha256") != state["source_sha256"]:
        raise RuntimeError("build does not match frozen source/profile")
    shutil.copytree(built / "esp", directory / "esp")
    if sha(directory / "esp/kernel.elf") != manifest["kernel"]["sha256"]:
        raise RuntimeError("kernel bytes differ from build manifest")
    if sha(directory / "esp/EFI/BOOT/BOOTX64.EFI") != manifest["loader"]["sha256"]:
        raise RuntimeError("loader bytes differ from build manifest")
    code, variables = firmware()
    shutil.copy2(code, directory / "OVMF_CODE.fd")
    shutil.copy2(variables, directory / "OVMF_VARS.fd")
    save(directory / "build-manifest.json", manifest)
    return directory, state


def run(expected_head, label, image, *, readonly, control=None, timeout):
    profile = "test-volume-recover" if readonly else "native-volume"
    directory, state = prepare(expected_head, profile, label)
    candidate = Path(image)
    if candidate.is_symlink() or not candidate.is_file() or candidate.stat().st_nlink != 1:
        raise ValueError("source must be a single-link, non-symlink regular image")
    original = candidate.resolve(strict=True)
    source_before = sha(original)
    target = directory / "data.img"
    shutil.copyfile(original, target)
    before = sha(target)
    if target.stat().st_nlink != 1 or before != source_before:
        raise RuntimeError("disposable copy differs from source")
    with target.open("rb") as stream:
        stream.seek(volume_disk.FIRST * volume_disk.SECTOR)
        first = stream.read(volume_disk.TABLE_SECTORS * volume_disk.SECTOR)
        second = stream.read(volume_disk.TABLE_SECTORS * volume_disk.SECTOR)
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("QEMU unavailable")
    command = [qemu, "-machine", "pc-q35-10.0", "-accel", "tcg,thread=single", "-cpu", "qemu64",
               "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
               "-serial", "stdio", "-nic", "none", "-no-reboot",
               "-trace", "enable=virtio_blk_handle_read", "-trace", "enable=virtio_blk_handle_write",
               "-drive", "if=pflash,unit=0,format=raw,readonly=on,file=OVMF_CODE.fd",
               "-drive", "if=pflash,unit=1,format=raw,file=OVMF_VARS.fd",
               "-drive", "if=virtio,index=0,format=raw,readonly=on,file=fat:esp",
               "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
               "-drive", f"if=none,id=volume,format=raw,cache=writeback,readonly={'on' if readonly else 'off'},file=data.img",
               "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-M6E-TEST"]
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    timed_out = False
    try:
        proc = subprocess.run(command, cwd=directory, env=env, capture_output=True, timeout=timeout)
        code, out, err = proc.returncode, proc.stdout, proc.stderr
    except subprocess.TimeoutExpired as exc:
        timed_out = True
        code, out, err = None, exc.stdout or b"", exc.stderr or b""
    (directory / "serial.txt").write_bytes(out)
    (directory / "stderr-trace.txt").write_bytes(err)
    lines = out.decode("utf-8", "replace").splitlines()
    trace_lines = err.decode("utf-8", "replace").splitlines()
    reads = sum("virtio_blk_handle_read " in line for line in trace_lines)
    writes = sum("virtio_blk_handle_write " in line for line in trace_lines)
    after, source_after = sha(target), sha(original)
    report = {"schema_version": 1, "label": label, "profile": profile, "control": control,
              "passed": False, "started_utc": started, "duration_s": time.monotonic() - start,
              "timeout_s": timeout, "timed_out": timed_out, "raw_qemu_exit": code,
              "command": command, "cwd": str(directory), "source": state,
              "source_unchanged": frozen(expected_head) == state,
              "source_image": str(original), "source_image_size": original.stat().st_size,
              "source_image_sha256_before": source_before, "source_image_sha256_after": source_after,
              "probe_image_sha256_before": before, "probe_image_sha256_after": after,
              "source_image_unchanged": source_before == source_after, "probe_image_unchanged": before == after,
              "data_disk_read_only": readonly, "guest_block_rights": "READ" if readonly else "READ|WRITE|FLUSH",
              "kernel_sha256": sha(directory / "esp/kernel.elf"),
              "loader_sha256": sha(directory / "esp/EFI/BOOT/BOOTX64.EFI"),
              "firmware_code_sha256": sha(directory / "OVMF_CODE.fd"),
              "first_table_all_zero": not any(first), "second_table_all_zero": not any(second),
              "trace_scope": "all virtio-blk devices, including boot ESP; zero writes covers both devices",
              "trace_read_requests": reads, "trace_write_requests": writes,
              "serial_sha256": sha(directory / "serial.txt"), "trace_sha256": sha(directory / "stderr-trace.txt"),
              "native_filesystem_support_demonstrated": False, "native_foreign_content_recovery_demonstrated": False}
    try:
        if timed_out or not all(marker in lines for marker in BOOT):
            raise RuntimeError("timeout or missing boot markers")
        if reads == 0:
            raise RuntimeError("read trace absent; trace instrumentation not demonstrated")
        if source_before != source_after:
            raise RuntimeError("source image changed")
        if control:
            if code != 33 or controlled("\n".join(lines)) != list(expected_markers(control)):
                raise RuntimeError("positive-control exit or ordered marker sequence mismatch")
            if any("FAIL" in line or "PANIC" in line for line in lines):
                raise RuntimeError("positive-control failure marker")
            oracle = volume_disk.analyse(target.read_bytes())
            check_state(control, oracle, numbers("\n".join(lines)))
            if readonly and (before != after or writes != 0):
                raise RuntimeError("cold read-only control changed bytes or issued a write")
            if not readonly and (before == after or writes == 0):
                raise RuntimeError("normal control failed to demonstrate actual guest writes")
            report.update(oracle_generation=oracle["generation"], oracle_digests=oracle["digests"])
        else:
            if code != 35 or lines.count(CORRUPT) != 1 or lines.count(STARTUP_FAILURE) != 1:
                raise RuntimeError("negative case did not terminate with startup Corrupt/-18 and raw exit35")
            if [line for line in lines if line.startswith("VOLUME:FAIL:")] != [STARTUP_FAILURE]:
                raise RuntimeError("unexpected failure marker")
            if any(line.startswith(("VOLUME:READY", "VOLUME:APP_", "VOLUME:CASE:", "VOLUME:PASS", "M1:PASS")) or "PANIC" in line for line in lines):
                raise RuntimeError("negative case reached app/ready/pass or panic")
            if before != after or writes != 0:
                raise RuntimeError("negative case changed bytes or issued guest writes")
        report["passed"] = True
    except Exception as exc:
        report["error"] = f"{type(exc).__name__}: {exc}"
    save(directory / "result.json", report)
    print(f"{label}: passed={report['passed']} exit={code} reads={reads} writes={writes} duration={report['duration_s']:.1f}s", flush=True)
    if not report["passed"]:
        raise RuntimeError(f"{label}: {report.get('error')}")
    return report, target


def matrix(expected_head):
    state = frozen(expected_head)
    seeds = local(ROOT / "seeds")
    seeds.mkdir(parents=True, exist_ok=False)
    for label, (path, digest) in CANDIDATES.items():
        if sha(path) != digest:
            raise RuntimeError(f"immutable {label} source differs from recorded baseline")
    volume_disk.create_image(seeds / "own-empty.img", "empty")
    own = volume_disk.analyse((seeds / "own-empty.img").read_bytes())
    if own["generation"] != 0 or any(own["clocks"]):
        raise RuntimeError("explicit own empty fixture is not generation zero")
    # Both synthetic negative candidates contain no format identity in either
    # table region. The latter has content inside the permitted data extent
    # and well beyond it, so a zero metadata prefix does not imply blank media.
    (seeds / "zero.img").write_bytes(bytes(1024 * 1024))
    tail = bytearray(1024 * 1024)
    payload = b"FOREIGN-PAYLOAD-MUST-SURVIVE\0"
    tail[80 * 512:80 * 512 + len(payload)] = payload
    tail[900 * 512:900 * 512 + len(payload)] = payload
    (seeds / "zero-prefix-payload-tail.img").write_bytes(tail)
    summary = {"schema_version": 1, "source": state, "passed": False, "runs": [],
               "seed_own_empty_oracle": own, "seed_hashes": {p.name: sha(p) for p in seeds.glob("*.img")}}
    save(ROOT / "matrix.json", summary)
    report, final_image = run(expected_head, "own-normal", seeds / "own-empty.img", readonly=False, control="normal", timeout=300)
    summary["runs"].append(report)
    save(ROOT / "matrix.json", summary)
    cold, _ = run(expected_head, "own-cold-recover", final_image, readonly=True, control="recover", timeout=300)
    if (cold["source_image_sha256_before"] != report["probe_image_sha256_after"] or
            cold["oracle_generation"] != report["oracle_generation"] or cold["oracle_digests"] != report["oracle_digests"]):
        raise RuntimeError("cold recovery did not preserve exact normal bytes/state")
    summary["runs"].append(cold)
    save(ROOT / "matrix.json", summary)
    candidates = [("btrfs", CANDIDATES["btrfs"][0]), ("zero", seeds / "zero.img"),
                  ("zero-prefix-payload-tail", seeds / "zero-prefix-payload-tail.img"), ("openzfs-leaf", CANDIDATES["openzfs-leaf"][0])]
    for label, path in candidates:
        for readonly in (True, False):
            report, _ = run(expected_head, label + ("-readonly" if readonly else "-writable"), path, readonly=readonly, timeout=120)
            summary["runs"].append(report)
            save(ROOT / "matrix.json", summary)
    summary.update(passed=len(summary["runs"]) == 10 and all(r["passed"] for r in summary["runs"]),
                   source_unchanged=frozen(expected_head) == state)
    save(ROOT / "matrix.json", summary)
    print(f"matrix: passed={summary['passed']} runs={len(summary['runs'])}", flush=True)


def regression_child(expected_head):
    frozen(expected_head)
    import run_volume
    import test_volume
    # Only run_volume's QEMU working directory/path base changes. All source
    # fingerprints and build/host-test paths still use the real source tree.
    run_volume.ROOT = ROOT
    return test_volume.main()


def regression(expected_head):
    state = frozen(expected_head)
    destination = local(ROOT / "regression")
    destination.mkdir(parents=True, exist_ok=False)
    env = build_environment()
    env.update(NOS_BUILD_DIR=str(destination / "build"), NOS_TARGET_DIR=str(destination / "target"),
               CARGO_TARGET_DIR=str(destination / "target"))
    command = [sys.executable, "-B", str(Path(__file__).resolve()), "regression-child", "--expected-head", expected_head]
    start = time.monotonic()
    proc = subprocess.run(command, cwd=SOURCE, env=env, capture_output=True, timeout=7200)
    (destination / "stdout.txt").write_bytes(proc.stdout)
    (destination / "stderr.txt").write_bytes(proc.stderr)
    gate_path = destination / "build/volume-gate.json"
    gate = json.loads(gate_path.read_text(encoding="utf-8")) if gate_path.exists() else {}
    report = {"command": command, "cwd": str(SOURCE), "returncode": proc.returncode,
              "duration_s": time.monotonic() - start, "source": state, "source_unchanged": frozen(expected_head) == state,
              "gate_path": str(gate_path), "gate_passed": gate.get("passed") is True,
              "stage_count": len(gate.get("stages", [])),
              "passed": proc.returncode == 0 and gate.get("passed") is True and len(gate.get("stages", [])) == 9}
    save(destination / "invocation.json", report)
    print(f"regression: passed={report['passed']} exit={proc.returncode} stages={report['stage_count']}", flush=True)
    if not report["passed"]:
        raise RuntimeError(f"regression failed; inspect {destination}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("build", "matrix", "regression", "regression-child"))
    parser.add_argument("--expected-head", required=True)
    args = parser.parse_args()
    if len(args.expected_head) != 40 or any(ch not in "0123456789abcdef" for ch in args.expected_head):
        parser.error("expected-head must be the full lowercase source commit")
    if args.action == "build":
        build_profiles(args.expected_head)
    elif args.action == "matrix":
        matrix(args.expected_head)
    elif args.action == "regression":
        regression(args.expected_head)
    else:
        return regression_child(args.expected_head)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
