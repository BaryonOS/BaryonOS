#!/usr/bin/env python3
"""Build and validate the opt-in native x87/SSE context-isolation fixture."""
from __future__ import annotations

import json
from pathlib import Path
import re
import shutil
import stat
import uuid

from common import ROOT, BUILD, TARGET, environment, qemu_machine, run_logged, save_json, sha256, source_hash
from build_users import inspect_user
from run import run_guest

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "M2:PASS")
CASES = ("FPU:MANIFEST:PASS", "FPU:UNSUPPORTED_AVX:PASS", "FPU:SURVIVOR:PASS", "FPU:SLOT_REUSE:PASS")
END = ("FPU:CONTEXT_ISOLATION:PASS", "FPU:CLOCK:PASS", "FPU:CLEANUP:PASS", "FPU:PASS", "M1:PASS")
REQUIRED = BOOT + CASES + END
FORBIDDEN = ("FAIL", "PANIC")
FIELDS = ("CLEAN_STARTS", "VERIFIED_LOOPS", "CLOCK_READS", "CLOCK_DENIED", "TIMER_A", "TIMER_B", "TIMER_IRQS", "SYSCALL_CALLS", "SAVES", "RESTORES", "FAULTS", "EXITS", "SLOT_REUSES", "FREE_FRAMES", "CONTEXT_BYTES_WIPED")
WAIT_FIELDS = ("WAIT_CALLS", "WAIT_BLOCKED", "WAIT_TIMEOUTS", "WAIT_IMMEDIATE", "WAIT_IDLE_IRQS")
NAMES = ("fpu-a", "fpu-b")


def private_path(path: Path, *, regular: bool = False, cargo_output: bool = False) -> Path:
    """Refuse links, special files and paths outside this disposable checkout."""
    path = path.absolute()
    try:
        relative = path.relative_to(ROOT)
    except ValueError as error:
        raise ValueError("FPU artifact escaped checkout") from error
    current = ROOT
    for part in relative.parts:
        current /= part
        if current.is_symlink():
            raise ValueError("Symlink in FPU artifact path")
        if current.exists():
            metadata = current.lstat()
            if not (stat.S_ISREG(metadata.st_mode) or stat.S_ISDIR(metadata.st_mode)):
                raise ValueError("Special file in FPU artifact path")
            # Cargo creates internal hardlinks for its final binaries. The only
            # exception is a read-only source after the trusted Cargo invocation;
            # published inputs, reports and existing outputs must have one link.
            if stat.S_ISREG(metadata.st_mode) and metadata.st_nlink != 1 and not (cargo_output and current == path):
                raise ValueError("Hardlinked FPU input/output")
    if not path.resolve().is_relative_to(ROOT.resolve()) or (regular and not path.is_file()):
        raise ValueError("Missing/nonregular FPU artifact")
    return path


def publish(path: Path, data: bytes) -> None:
    """Exclusive temporary inode + atomic replace; never truncate an old link."""
    path = private_path(path)
    temporary = private_path(path.with_name(path.name + ".tmp-" + uuid.uuid4().hex))
    try:
        with temporary.open("xb") as output:
            output.write(data)
        temporary.replace(path)
    finally:
        if temporary.exists():
            temporary.unlink()


def publish_json(path: Path, value: object) -> None:
    publish(path, (json.dumps(value, indent=2, ensure_ascii=False) + "\n").encode())


def load_json(path: Path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError("Duplicate evidence key")
            result[key] = value
        return result
    raw = path.read_bytes()
    if len(raw) > 1024 * 1024:
        raise ValueError("Oversized evidence JSON")
    return json.loads(raw, object_pairs_hook=unique)


def build_users() -> tuple[dict, dict]:
    """Shared builder hook: kernel ELF environment and inspected artifact report."""
    private_path(BUILD)
    private_path(TARGET)
    output = private_path(BUILD / "users/fpu")
    output.mkdir(parents=True, exist_ok=True)
    for name in (*[f"{name}.elf" for name in NAMES], "build-report.json"):
        private_path(output / name)  # Refuse bad existing outputs before Cargo.
    linker = ROOT / "sdk/examples/fpu/linker.ld"
    flags = ("-C code-model=small -C relocation-model=static -C no-redzone=yes "
             f"-C link-arg=-T{linker} -C link-arg=--no-pie -C link-arg=--build-id=none -C link-arg=-zmax-page-size=0x1000")
    run_logged(["cargo", "+1.90.0", "build", "--offline", "--locked", "--release", "--target", "x86_64-unknown-none",
                "--manifest-path", str(ROOT / "sdk/examples/fpu/Cargo.toml"), "--features", "native", "--bins"],
               "build-fpu-users", extra_env={"CARGO_TARGET_DIR": str(TARGET / "fpu"), "RUSTFLAGS": flags})
    apps = []
    for name in NAMES:
        binary = private_path(output / f"{name}.elf")
        source = private_path(TARGET / "fpu/x86_64-unknown-none/release" / name, regular=True, cargo_output=True)
        publish(binary, source.read_bytes())
        apps.append({"name": name, "sha256": sha256(binary), "elf": inspect_user(binary)})
    report = {"passed": True, "native_runtime_performed": False, "apps": apps}
    publish_json(output / "build-report.json", report)
    return {"NOS_FPU_A_ELF": str((output / "fpu-a.elf").resolve()),
            "NOS_FPU_B_ELF": str((output / "fpu-b.elf").resolve())}, report


def validate_sequence(stdout: str, *, wait: bool = False) -> dict:
    if not isinstance(stdout, str) or len(stdout) > 128 * 1024 or any(word in stdout for word in FORBIDDEN):
        raise ValueError("Malformed or failed native serial output")
    lines = stdout.splitlines()
    if lines.count(BOOT[0]) != 1:
        raise ValueError("Missing or repeated EBS handoff")
    start = lines.index(BOOT[0])
    if any(re.search(r"(?:BOOT:|BOOTINFO:|KERNEL:|KS-|M[0-9]+:|FPU:)", line) for line in lines[:start]):
        raise ValueError("Unexpected marker before EBS")
    observed = lines[start:]
    prefix = BOOT + CASES
    fields = FIELDS + (WAIT_FIELDS if wait else ())
    end = (("FPU:WAIT:PASS",) if wait else ()) + END
    if len(observed) != len(prefix) + len(fields) + len(end):
        raise ValueError("Unexpected native output length")
    if tuple(observed[:len(prefix)]) != prefix or tuple(observed[len(prefix) + len(fields):]) != end:
        raise ValueError("Missing, repeated, reordered or unknown marker")
    numbers = {}
    for at, name in enumerate(fields, len(prefix)):
        match = re.fullmatch(re.escape(f"FPU:{name}:") + r"([0-9]{20})", observed[at])
        if not match or int(match.group(1)) > (1 << 64) - 1:
            raise ValueError("Malformed or overflowing native numeric field")
        numbers[name] = int(match.group(1))
    for name, expected in {"CLEAN_STARTS": 3, "CLOCK_DENIED": 18, "FAULTS": 1, "EXITS": 2,
                           "SLOT_REUSES": 1, "CONTEXT_BYTES_WIPED": 8192}.items():
        if numbers[name] != expected:
            raise ValueError(f"Wrong native counter {name}")
    idle_irqs = numbers["WAIT_IDLE_IRQS"] if wait else 0
    user_irqs = numbers["TIMER_A"] + numbers["TIMER_B"]
    if (numbers["VERIFIED_LOOPS"] < 24 or numbers["CLOCK_READS"] != numbers["VERIFIED_LOOPS"] + 3
            or numbers["TIMER_A"] < 2 or numbers["TIMER_B"] < 2 or numbers["FREE_FRAMES"] == 0
            or numbers["TIMER_IRQS"] != user_irqs + idle_irqs
            or numbers["SYSCALL_CALLS"] != 2 * numbers["VERIFIED_LOOPS"] + 26
            or numbers["SAVES"] != user_irqs + numbers["SYSCALL_CALLS"] + 1
            or numbers["RESTORES"] != numbers["SAVES"]):
        raise ValueError("Inconsistent or incomplete native context/clock evidence")
    if wait and (numbers["WAIT_CALLS"] != numbers["VERIFIED_LOOPS"]
                 or numbers["WAIT_BLOCKED"] < 3
                 or numbers["WAIT_TIMEOUTS"] != numbers["WAIT_BLOCKED"]
                 or numbers["WAIT_IMMEDIATE"] + numbers["WAIT_BLOCKED"] != numbers["WAIT_CALLS"]
                 or numbers["WAIT_IDLE_IRQS"] == 0):
        raise ValueError("Missing or inconsistent native blocking-wait evidence")
    return numbers


def validate_build(build: Path, frozen: str) -> dict:
    private_path(build)
    manifest = load_json(private_path(build / "manifest.json", regular=True))
    if (manifest.get("kernel_feature") not in ("native-fpu", "test-fpu-wait") or manifest.get("loader_feature") is not None
            or manifest.get("source_fingerprint_version") != 3 or manifest.get("source_sha256") != frozen
            or manifest.get("sdk_runtime") is not False or manifest.get("document_session") is not False):
        raise ValueError("Build the native-fpu or test-fpu-wait profile on the current source first")
    artifact = load_json(private_path(build / "users/fpu/build-report.json", regular=True))
    if (manifest.get("fpu") != artifact or artifact.get("passed") is not True
            or artifact.get("native_runtime_performed") is not False
            or not isinstance(artifact.get("apps"), list) or len(artifact["apps"]) != 2):
        raise ValueError("Unbound or malformed FPU user build report")
    kernel = private_path(build / "esp/kernel.elf", regular=True)
    kernel_bytes = kernel.read_bytes()
    for app, name in zip(artifact["apps"], NAMES):
        binary = private_path(build / f"users/fpu/{name}.elf", regular=True)
        if (app.get("name") != name or sha256(binary) != app.get("sha256")
                or inspect_user(binary) != app.get("elf") or binary.read_bytes() not in kernel_bytes):
            raise ValueError("FPU ELF inspection/hash or embedded bytes mismatch")
    expected = {"esp/kernel.elf": manifest["kernel"]["sha256"], "esp/EFI/BOOT/BOOTX64.EFI": manifest["loader"]["sha256"]}
    for name, value in expected.items():
        if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value) or sha256(private_path(build / name, regular=True)) != value:
            raise ValueError("Kernel or loader hash mismatch")
    files = set()
    for path in (build / "esp").rglob("*"):
        private_path(path)
        if path.is_file():
            files.add(path.relative_to(build).as_posix())
    if files != set(expected):
        raise ValueError("Unexpected ESP artifact")
    return {"manifest": manifest, "expected": expected, "artifact": artifact}


def validate_guest(report: dict, bound: dict, frozen: str, machine: str, qemu: str, build: Path) -> dict:
    wait = bound.get("manifest", {}).get("kernel_feature") == "test-fpu-wait"
    required = BOOT + CASES + (("FPU:WAIT:PASS",) if wait else ()) + END
    if (type(report.get("raw_qemu_exit")) is not int or report["raw_qemu_exit"] != 33
            or report.get("timed_out") is not False or report.get("passed") is not True
            or report.get("expected_exit") != 33 or report.get("artifact_binding_passed") is not True
            or report.get("required_markers") != list(required) or report.get("forbidden_markers") != list(FORBIDDEN)
            or report.get("source_fingerprint_version") != 3 or report.get("source_sha256") != frozen):
        raise ValueError("Native exit, expected profile or source binding mismatch")
    directory = private_path(Path(report["run_directory"])).resolve(strict=True)
    if not directory.is_relative_to((build / "run").resolve()) or not directory.is_relative_to(ROOT):
        raise ValueError("VM evidence escaped its private build root")
    relative = directory.relative_to(ROOT).as_posix()
    expected_command = [qemu, "-machine", machine, "-accel", "tcg,thread=single", "-cpu", "qemu64", "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none", "-serial", "stdio", "-nic", "none", "-no-reboot",
        "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={relative}/OVMF_CODE.fd",
        "-drive", f"if=pflash,unit=1,format=raw,file={relative}/OVMF_VARS.fd",
        "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{relative}/esp",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
    if (Path(qemu).parent / "share").is_dir():
        expected_command += ["-L", str(Path(qemu).parent / "share")]
    if (report.get("command") != expected_command
            or report.get("profile") != f"development-package; {machine}/qemu64/TCG/1CPU/256MiB"
            or not isinstance(report.get("qemu_version"), str) or not report["qemu_version"].startswith("QEMU emulator version ")):
        raise ValueError("VM machine, CPU, RAM, network or command mismatch")
    artifacts = report.get("artifact_sha256", {})
    if (set(artifacts) != set(bound["expected"]) | {"OVMF_CODE.fd", "OVMF_VARS.template.fd"}
            or report.get("expected_artifact_sha256") != bound["expected"]):
        raise ValueError("Unexpected guest artifact set")
    for name, value in artifacts.items():
        if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value) or sha256(private_path(directory / name, regular=True)) != value:
            raise ValueError("Guest snapshot differs from recorded hash")
    if any(artifacts[name] != value for name, value in bound["expected"].items()):
        raise ValueError("Guest kernel or loader differs from selected build")
    if private_path(directory / "serial.txt", regular=True).read_bytes().decode("utf-8", "replace") != report.get("stdout"):
        raise ValueError("Reported stdout differs from raw serial artifact")
    return validate_sequence(report["stdout"], wait=wait)


def run_fpu() -> dict:
    build = private_path(BUILD).resolve(strict=True)
    if not build.is_relative_to(ROOT):
        raise ValueError("NOS_BUILD_DIR must be inside the checkout")
    frozen = source_hash()
    bound = validate_build(build, frozen)
    wait = bound["manifest"]["kernel_feature"] == "test-fpu-wait"
    required = BOOT + CASES + (("FPU:WAIT:PASS",) if wait else ()) + END
    qemu = shutil.which("qemu-system-x86_64", path=environment()["PATH"])
    if qemu is None:
        raise RuntimeError("Missing QEMU")
    qemu_hash = sha256(Path(qemu))
    private_path(build / "run")
    private_path(build / "fpu-report.json")
    inputs = private_path(build / ("fpu-inputs-" + uuid.uuid4().hex[:10]))
    inputs.mkdir()
    for name in (*[f"{name}.elf" for name in NAMES], "build-report.json"):
        shutil.copyfile(build / "users/fpu" / name, inputs / name)
    save_json(inputs / "build-manifest.json", bound["manifest"])
    checks = {"before_launch_source_unchanged": source_hash() == frozen}
    report = {"schema_version": 1, "scope": "eager x87/SSE state on qemu64, three ring3 lifetimes, actual unsupported-AVX fault, timer/syscall switches, boot-monotonic PIT clock; no AVX support, walltime or physical-CPU claim",
              "passed": False, "blocking_wait_required": wait, "checks": checks, "source_fingerprint_version": 3, "source_sha256": frozen,
              "qemu_entry_path": qemu, "qemu_entry_sha256": qemu_hash, "inputs_directory": str(inputs)}
    try:
        if not all(checks.values()):
            raise ValueError("Source changed while freezing build inputs")
        guest = run_guest(bound["manifest"]["kernel_feature"], timeout=60, required_markers=required, expected_exit=33,
                          forbidden_markers=FORBIDDEN, expected_kernel_sha256=bound["expected"]["esp/kernel.elf"],
                          expected_loader_sha256=bound["expected"]["esp/EFI/BOOT/BOOTX64.EFI"])
        report["guest"] = guest
        report["observed"] = validate_guest(guest, bound, frozen, qemu_machine(), qemu, build)
        checks["guest_exit_sequence_profile_and_artifacts"] = True
        checks["after_run_source_unchanged"] = source_hash() == frozen
        checks["qemu_entry_unchanged"] = sha256(Path(qemu)) == qemu_hash
        checks["user_binary_snapshots"] = all(sha256(inputs / f"{app['name']}.elf") == app["sha256"] for app in bound["artifact"]["apps"])
        checks["report_snapshot"] = load_json(inputs / "build-report.json") == bound["artifact"]
        checks["build_manifest_snapshot"] = load_json(inputs / "build-manifest.json") == bound["manifest"]
        validate_build(build, frozen)
        checks["build_artifacts_unchanged"] = True
        report["passed"] = all(checks.values())
    except (ValueError, KeyError, TypeError, OSError) as error:
        report["error"] = f"{type(error).__name__}: {error}"
    publish_json(build / "fpu-report.json", report)
    return report


if __name__ == "__main__":
    raise SystemExit(0 if run_fpu()["passed"] else 1)
