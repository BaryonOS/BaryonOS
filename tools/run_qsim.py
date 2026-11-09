#!/usr/bin/env python3
"""Run and strictly validate the source-bound native qsim acceptance image."""
from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import re
import shutil
import uuid

from common import ROOT, BUILD, environment, save_json, sha256, source_hash, without_acpi_diagnostics
from run import run_guest

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "M2:PASS")
CASES = ("QSIM:BELL4096:PASS", "QSIM:SIX_QUBIT:PASS", "QSIM:CANCEL:PASS", "QSIM:DEADLINE:PASS", "QSIM:RESOURCE_LIMITS:PASS", "QSIM:PROCESS_ISOLATION:PASS")
END = ("QSIM:CLEANUP:PASS", "QSIM:PASS", "M1:PASS")
REQUIRED = BOOT + ("QSIM:MANIFEST:PASS",) + CASES + END
FORBIDDEN = ("FAIL", "PANIC")
FLOAT_SYMBOLS = ["__adddf3", "__muldf3", "__divdf3", "__floatdidf"]


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


def artifact_builder():
    path = ROOT / "experiments/quantum/native/build.py"
    spec = importlib.util.spec_from_file_location("qsim_artifact_builder", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def validate_sequence(stdout: str) -> dict:
    if not isinstance(stdout, str) or len(stdout) > 128 * 1024 or any(word in stdout for word in FORBIDDEN):
        raise ValueError("Malformed or failed native serial output")
    stdout = without_acpi_diagnostics(stdout)
    lines = stdout.splitlines()
    if lines.count(BOOT[0]) != 1:
        raise ValueError("Missing or repeated EBS handoff")
    start = lines.index(BOOT[0])
    # UEFI may write banners/ANSI escapes before handoff. It may not supply
    # another test marker, and after handoff every line belongs to this grammar.
    if any(re.search(r"(?:BOOT:|BOOTINFO:|KERNEL:|KS-|M[0-9]+:|G[0-9]+:|QSIM:)", line) for line in lines[:start]):
        raise ValueError("Unexpected control output before EBS")
    observed = lines[start:]
    if len(observed) != 28 or tuple(observed[:8]) != BOOT + ("QSIM:MANIFEST:PASS",):
        raise ValueError("Unexpected boot or quantum marker sequence")

    def number(at: int, label: str) -> int:
        match = re.fullmatch(re.escape(label) + r"([0-9]{20})", observed[at])
        if not match:
            raise ValueError("Malformed native numeric field")
        value = int(match.group(1))
        if value > (1 << 64) - 1:
            raise ValueError("Native numeric field exceeds u64")
        return value

    processes = []
    for slot in range(2):
        at = 8 + 4 * slot
        item = {
            "pid": number(at, "QSIM:PROCESS_PID:"),
            "checkpoints": number(at + 1, "QSIM:PROCESS_CHECKPOINTS:"),
            "useful_arena": number(at + 2, "QSIM:PROCESS_USEFUL_ARENA:"),
            "max_work_gap": number(at + 3, "QSIM:PROCESS_MAX_WORK_GAP:"),
        }
        if item["pid"] != slot + 1 or item["checkpoints"] == 0 or item["useful_arena"] != 7216 or item["max_work_gap"] != 448:
            raise ValueError("Native process result mismatch")
        processes.append(item)
    if tuple(observed[16:22]) != CASES or tuple(observed[25:]) != END:
        raise ValueError("Missing, repeated, reordered or unknown quantum marker")
    result = {
        "processes": processes,
        "timer_irqs": number(22, "QSIM:TIMER_IRQS:"),
        "syscall_calls": number(23, "QSIM:SYSCALL_CALLS:"),
        "cleanup_frames": number(24, "QSIM:CLEANUP_FRAMES:"),
    }
    if min(result["timer_irqs"], result["syscall_calls"], result["cleanup_frames"]) == 0:
        raise ValueError("Missing timer, syscall or actual frame cleanup")
    return result


def validate_build(build: Path, frozen: str) -> dict:
    manifest = load_json(build / "manifest.json")
    if (manifest.get("kernel_feature") != "native-qsim" or manifest.get("loader_feature") is not None
            or manifest.get("source_fingerprint_version") != 3 or manifest.get("source_sha256") != frozen
            or manifest.get("sdk_runtime") is not False):
        raise ValueError("Build the exclusive native-qsim profile on the current source first")
    folder = build / "users/qsim"
    artifact = load_json(folder / "build-report.json")
    builder = artifact_builder()
    if (manifest.get("qsim_user") != artifact or artifact.get("passed") is not True
            or artifact.get("native_runtime_performed") is not False
            or artifact.get("source_sha256") != builder.sources()
            or artifact.get("software_float_symbols") != FLOAT_SYMBOLS):
        raise ValueError("Unbound or stale scalar qsim build report")
    user = folder / "qsim-native-probe.elf"
    disassembly = folder / "disassembly.stdout.txt"
    if (sha256(user) != artifact.get("binary_sha256")
            or sha256(disassembly) != artifact.get("disassembly_sha256")
            or builder.inspect_user(user) != artifact.get("elf")
            or builder.inspect_instructions(disassembly.read_text()) != artifact.get("scalar_gate")):
        raise ValueError("Quantum ELF or scalar artifact gate mismatch")
    expected = {"esp/kernel.elf": manifest["kernel"]["sha256"], "esp/EFI/BOOT/BOOTX64.EFI": manifest["loader"]["sha256"]}
    for name, value in expected.items():
        if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value) or sha256(build / name) != value:
            raise ValueError("Kernel or loader artifact mismatch")
    if user.read_bytes() not in (build / "esp/kernel.elf").read_bytes():
        raise ValueError("The selected kernel does not embed this exact qsim ELF")
    return {"manifest": manifest, "expected": expected, "user_sha256": sha256(user), "build_report_sha256": sha256(folder / "build-report.json")}


def validate_guest(report: dict, bound: dict, frozen: str, machine: str, qemu: str, build: Path) -> dict:
    if (report.get("raw_qemu_exit") != 33 or type(report.get("raw_qemu_exit")) is not int
            or report.get("timed_out") is not False or report.get("passed") is not True
            or report.get("expected_exit") != 33 or report.get("artifact_binding_passed") is not True
            or report.get("required_markers") != list(REQUIRED) or report.get("forbidden_markers") != list(FORBIDDEN)
            or report.get("source_fingerprint_version") != 3 or report.get("source_sha256") != frozen):
        raise ValueError("Native raw exit, source or expected profile is not accepted")
    directory = Path(report["run_directory"]).resolve(strict=True)
    if not directory.is_relative_to((build / "run").resolve()) or not directory.is_relative_to(ROOT):
        raise ValueError("VM evidence escaped its private build root")
    relative = directory.relative_to(ROOT).as_posix()
    expected_command = [qemu, "-machine", machine, "-accel", "tcg,thread=single", "-cpu", "qemu64", "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none", "-serial", "stdio", "-nic", "none", "-no-reboot",
        "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={relative}/OVMF_CODE.fd",
        "-drive", f"if=pflash,unit=1,format=raw,file={relative}/OVMF_VARS.fd",
        "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{relative}/esp",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
    share = Path(qemu).parent / "share"
    if share.is_dir():
        expected_command += ["-L", str(share)]
    if (report.get("command") != expected_command
            or report.get("profile") != f"development-package; {machine}/qemu64/TCG/1CPU/256MiB"
            or not isinstance(report.get("qemu_version"), str)
            or not report["qemu_version"].startswith("QEMU emulator version ")):
        raise ValueError("VM profile, command or QEMU version mismatch")
    artifacts = report.get("artifact_sha256", {})
    if (set(artifacts) != set(bound["expected"]) | {"OVMF_CODE.fd", "OVMF_VARS.template.fd"}
            or report.get("expected_artifact_sha256") != bound["expected"]):
        raise ValueError("Unexpected guest artifact set")
    for name, value in artifacts.items():
        if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value) or sha256(directory / name) != value:
            raise ValueError("Guest evidence bytes do not match artifact hashes")
    if any(artifacts[name] != value for name, value in bound["expected"].items()):
        raise ValueError("Guest kernel or loader differs from bound build")
    if (directory / "serial.txt").read_bytes().decode("utf-8", "replace") != report.get("stdout"):
        raise ValueError("Reported stdout differs from actual serial artifact")
    return validate_sequence(report["stdout"])


def run_qsim() -> dict:
    from common import qemu_machine
    build = BUILD.resolve(strict=True)
    if not build.is_relative_to(ROOT):
        raise ValueError("The existing guest runner requires NOS_BUILD_DIR inside the checkout")
    run_parent = build / "run"
    run_parent.mkdir(exist_ok=True)
    if run_parent.is_symlink() or not run_parent.resolve().is_relative_to(build):
        raise ValueError("Private run root must remain inside this build directory")
    frozen = source_hash()
    bound = validate_build(build, frozen)
    machine = qemu_machine()
    qemu = shutil.which("qemu-system-x86_64", path=environment()["PATH"])
    if qemu is None:
        raise RuntimeError("Missing QEMU")
    qemu_hash = sha256(Path(qemu))
    inputs = build / ("qsim-inputs-" + uuid.uuid4().hex[:10])
    inputs.mkdir()
    for name in ("qsim-native-probe.elf", "build-report.json", "disassembly.stdout.txt", "symbols.stdout.txt"):
        shutil.copyfile(build / "users/qsim" / name, inputs / name)
    save_json(inputs / "build-manifest.json", bound["manifest"])
    checks = {"before_launch_source_unchanged": source_hash() == frozen}
    report = {"schema_version": 1, "scope": "two isolated ring-3 qsim reference jobs, cancellation, resource bounds and cleanup; no QPU or general service API", "passed": False, "checks": checks, "source_fingerprint_version": 3, "source_sha256": frozen, "qemu_entry_path": qemu, "qemu_entry_sha256": qemu_hash, "inputs_directory": str(inputs)}
    guest = None
    try:
        if not all(checks.values()):
            raise ValueError("Source changed while freezing quantum build inputs")
        guest = run_guest("native-qsim", esp=build / "esp", timeout=60, required_markers=REQUIRED, expected_exit=33, forbidden_markers=FORBIDDEN,
                          expected_kernel_sha256=bound["expected"]["esp/kernel.elf"], expected_loader_sha256=bound["expected"]["esp/EFI/BOOT/BOOTX64.EFI"])
        report["guest"] = guest
        report["observed"] = validate_guest(guest, bound, frozen, machine, qemu, build)
        checks["guest_exit_sequence_profile_and_artifacts"] = True
        checks["after_run_source_unchanged"] = source_hash() == frozen
        checks["qemu_entry_unchanged"] = sha256(Path(qemu)) == qemu_hash
        checks["quantum_binary_snapshot"] = sha256(inputs / "qsim-native-probe.elf") == bound["user_sha256"]
        checks["scalar_report_snapshot"] = sha256(inputs / "build-report.json") == bound["build_report_sha256"]
        validate_build(build, frozen)
        checks["build_artifacts_unchanged"] = True
        report["passed"] = all(checks.values())
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    report["input_artifact_sha256"] = {p.name: sha256(p) for p in inputs.iterdir() if p.is_file()}
    save_json(inputs / "result.json", report)
    if guest is not None:
        destination = Path(guest["run_directory"]).resolve()
        if destination.is_relative_to(run_parent.resolve()):
            save_json(destination / "qsim-result.json", report)
    save_json(build / "native-qsim-result.json", report)
    print(f"Native qsim accepted={report['passed']}; {inputs}")
    if "error" in report:
        print(report["error"])
    return report


if __name__ == "__main__":
    raise SystemExit(0 if run_qsim()["passed"] else 1)
