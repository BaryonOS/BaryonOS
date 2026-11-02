"""Use the unchanged native window verifier with explicit Windows dependencies.

The product checkout is never edited. Source, ELF, pixel, recovery, disk,
deadline and exit checks remain in its own run_window_session module.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib
import json
import os
from pathlib import Path
import re
import subprocess as host_process
import sys
import threading
import time
from types import SimpleNamespace


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def expected_command(qemu: Path, machine: str) -> list[str]:
    """Refuse unreviewed devices/options if the source runner later changes."""
    return [str(qemu), "-L", ".", "-machine", machine,
        "-accel", "tcg,thread=single", "-cpu", "qemu64", "-smp", "1", "-m", "256M",
        "-nodefaults", "-display", "none", "-monitor", "none", "-serial", "file:serial.txt",
        "-nic", "none", "-no-reboot", "-device", "VGA,vgamem_mb=16,romfile=VGA.rom",
        "-qmp", "stdio", "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-drive", "if=pflash,unit=0,format=raw,readonly=on,file=OVMF_CODE.fd",
        "-drive", "if=pflash,unit=1,format=raw,file=OVMF_VARS.fd",
        "-drive", "if=virtio,index=0,format=raw,readonly=on,file=fat:esp",
        "-object", "rng-random,id=rng0,filename=/dev/urandom",
        "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000",
        "-drive", "if=none,id=volume,format=raw,cache=writeback,file=volume.img",
        "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-SYSTEM-TEST"]


def translate_command(command: list[str], qemu: Path, machine: str, acceleration: str) -> list[str]:
    expected = expected_command(qemu, machine)
    if (type(command) is list and command and isinstance(command[0], str)
            and Path(command[0]) == qemu):
        expected[0] = command[0]  # Windows PATHEXT may spell the same file .EXE.
    if type(command) is not list or command != expected:
        raise ValueError("Source QEMU command changed; review the adapter before executing it")
    if acceleration not in ("whpx", "tcg"):
        raise ValueError("Acceleration must be selected explicitly")
    result = command.copy()
    result.insert(1, "-no-user-config")
    result[result.index("-accel") + 1] = "whpx" if acceleration == "whpx" else "tcg,thread=single"
    result[result.index("-object") + 1] = "rng-builtin,id=rng0"
    return result


class WindowsProcesses:
    """A module-local dependency; the real subprocess module is unchanged."""
    PIPE = host_process.PIPE
    TimeoutExpired = host_process.TimeoutExpired

    def __init__(self, source, build, qemu, machine, acceleration, timeout, guard, image):
        self.source, self.build, self.qemu = source, build, qemu
        self.machine, self.acceleration, self.timeout = machine, acceleration, timeout
        self.guard, self.image = guard, image
        self.child = None
        self.watchdog = None
        self.watchdog_terminated = False
        self.cleanup_errors = []
        self.original_command = self.actual_command = self.actual_image = None
        self.run_directory = None

    def check_output(self, command, **kwargs):
        if (command != ["git", "rev-parse", "HEAD"]
                or Path(kwargs.get("cwd", "")).resolve() != self.source
                or kwargs.get("text") is not True or set(kwargs) != {"cwd", "text"}):
            raise ValueError("Unexpected subprocess request from the source runner")
        return host_process.check_output(command, timeout=10, **kwargs)

    def Popen(self, command, **kwargs):
        if self.child is not None:
            raise ValueError("The verifier may start exactly one owned QEMU process")
        if (set(kwargs) != {"cwd", "env", "stdin", "stdout", "stderr", "bufsize"}
                or kwargs["stdin"] != self.PIPE or kwargs["stdout"] != self.PIPE
                or kwargs["bufsize"] != 0 or not isinstance(kwargs["env"], dict)):
            raise ValueError("Unexpected process or pipe configuration")
        cwd = self.guard(Path(kwargs["cwd"]), root=self.build)
        self.run_directory = cwd
        self.guard(Path(kwargs["stderr"].name), root=cwd, regular=True)
        self.original_command = command.copy()
        self.actual_command = translate_command(command, self.qemu, self.machine, self.acceleration)
        # report['command'] and command reference the same source-created list.
        # Mutate it in place so the native report records the actual argv.
        command[:] = self.actual_command
        self.child = host_process.Popen(command, creationflags=host_process.CREATE_NO_WINDOW, **kwargs)
        self.actual_image = self.image(self.child, self.qemu)

        def stop_owned_child():
            if self.child is not None and self.child.poll() is None:
                try:
                    self.child.kill()
                    self.watchdog_terminated = True
                except ProcessLookupError:
                    pass

        self.watchdog = threading.Timer(self.timeout + 6, stop_owned_child)
        self.watchdog.daemon = True
        self.watchdog.start()
        return self.child

    def readlink(self, path):
        if self.child is None or path != f"/proc/{self.child.pid}/exe":
            raise ValueError("Unexpected Linux process path requested")
        self.actual_image = self.image(self.child, self.qemu)
        return str(self.actual_image)

    def close(self):
        if self.watchdog is not None:
            self.watchdog.cancel()
            self.watchdog.join(timeout=0.2)
            if self.watchdog.is_alive():
                self.cleanup_errors.append("Watchdog thread did not finish")
        if self.child is not None:
            try:
                if self.child.poll() is None:
                    self.child.kill()
                    self.child.wait(timeout=5)
                    self.watchdog_terminated = True
            finally:
                for name in ("stdin", "stdout"):
                    stream = getattr(self.child, name, None)
                    if stream is not None:
                        try:
                            stream.close()
                        except (OSError, ValueError) as error:
                            self.cleanup_errors.append(name + ": " + str(error))


def tracked_transport(base, records, owner, on_first_key=None):
    """Keep constructor-failure cleanup visible without changing the transport."""
    class Tracked(base):
        def __init__(self, process, deadline):
            if records or process is not owner.child:
                raise ValueError("Unexpected QMP owner or second transport")
            record = {"instance": self, "initialized": False, "initialization_error": None}
            records.append(record)
            self._idle_key_seen = False
            try:
                super().__init__(process, deadline)
                record["initialized"] = True
            except BaseException as error:
                record["initialization_error"] = type(error).__name__ + ": " + str(error)
                raise

        def command(self, name, arguments=None):
            if on_first_key is not None and name == "send-key" and not self._idle_key_seen:
                self._idle_key_seen = True
                on_first_key(self, super().command, name, arguments)
            return super().command(name, arguments)
    return Tracked


def cleanup_transports(records):
    result = []
    for record in records:
        control = record["instance"]
        errors = list(getattr(control, "cleanup_errors", []))
        # Validation can fail before any worker or I/O handle is allocated.
        allocated = hasattr(control, "_workers") or record["initialized"]
        if allocated:
            try:
                control.close()
            except Exception as error:
                errors.append(type(error).__name__ + ": " + str(error))
        pending = bool(control.cleanup_pending) if allocated else False
        result.append({"initialized": record["initialized"], "initialization_error": record["initialization_error"],
                       "pending": pending, "errors": errors})
    return result


def import_runner(source: Path):
    tools_dir = source / "tools"
    # Discover every possible project module before imports, including indirect
    # validators such as run_display_probe.require used by the disk oracle.
    names = {p.stem for p in tools_dir.glob("*.py") if p.name != "__init__.py"}
    for name in names:
        previous = sys.modules.get(name)
        if previous is not None:
            raise ValueError("Start a fresh interpreter; project module already loaded: " + name)
    sys.path.insert(0, str(tools_dir))
    runner = importlib.import_module("run_window_session")
    for name in names.intersection(sys.modules):
        module = sys.modules[name]
        if not Path(module.__file__).resolve().is_relative_to(tools_dir):
            raise ValueError("Module escaped the frozen checkout: " + name)
        if hasattr(module, "ROOT") and Path(module.ROOT).resolve() != source:
            raise ValueError("Module ROOT differs from frozen checkout: " + name)
    return runner


def validate_idle_evidence(helper):
    from windows_inputs import assert_plain_path
    if not helper.used or not helper.record.get("measurement_passed"):
        raise ValueError("Opt-in idle observation failed or did not run")
    for filename, expected in helper.files.items():
        path = assert_plain_path(helper.output / filename, root=helper.directory, regular=True)
        if digest(path) != expected:
            raise ValueError("Stored CPU observation changed: " + filename)
    stored = json.loads((helper.output / "result.json").read_text(encoding="utf-8"))
    if stored != helper.record:
        raise ValueError("Stored CPU report differs from the completed observation")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--expected-commit", required=True)
    parser.add_argument("--build-dir", type=Path, required=True)
    parser.add_argument("--fixture-dir", type=Path, required=True)
    parser.add_argument("--qemu", type=Path, default=Path(r"C:\Program Files\qemu\qemu-system-x86_64.exe"))
    parser.add_argument("--ovmf-code", type=Path, required=True)
    parser.add_argument("--ovmf-vars", type=Path, required=True)
    parser.add_argument("--bios-rom", type=Path, required=True)
    parser.add_argument("--vga-rom", type=Path, required=True)
    parser.add_argument("--acceleration", choices=("whpx", "tcg"), required=True)
    parser.add_argument("--machine", choices=("pc-q35-8.2", "pc-q35-10.0", "pc-q35-7.2"), default="pc-q35-8.2")
    parser.add_argument("--timeout", type=int, default=900)
    parser.add_argument("--check-only", action="store_true")
    parser.add_argument("--measure-idle", action="store_true")
    parser.add_argument("--reference-frame-sha256")
    args = parser.parse_args()
    if args.reference_frame_sha256 and (not args.measure_idle or not re.fullmatch(r"[0-9a-f]{64}", args.reference_frame_sha256)):
        parser.error("A valid reference hash requires --measure-idle")
    if os.name != "nt":
        raise SystemExit("This adapter requires Windows; no platform fallback")
    if not re.fullmatch(r"[0-9a-f]{40}", args.expected_commit) or not 30 <= args.timeout <= 1800:
        raise SystemExit("An exact commit and bounded timeout are required")

    from windows_inputs import assert_plain_path, load_fixture, process_image
    source = assert_plain_path(args.source).resolve(strict=True)
    if source != (Path(__file__).resolve().parent / "source").resolve(strict=True):
        raise SystemExit("Use this adapter's own explicitly frozen source checkout")
    build = assert_plain_path(args.build_dir, root=source)
    fixture_dir = assert_plain_path(args.fixture_dir, root=source)
    qemu = assert_plain_path(args.qemu, regular=True).resolve(strict=True)
    for path in (args.ovmf_code, args.ovmf_vars, args.bios_rom, args.vga_rom):
        assert_plain_path(path, regular=True)
    os.environ["NOS_BUILD_DIR"] = str(build)
    os.environ["NOS_TARGET_DIR"] = str(source / "target/windows-oracle-not-built")
    os.environ["NOS_QEMU_MACHINE"] = args.machine
    os.environ["OS_OVMF_CODE"] = str(args.ovmf_code.resolve(strict=True))
    os.environ["OS_OVMF_VARS"] = str(args.ovmf_vars.resolve(strict=True))
    os.environ["PATH"] = str(qemu.parent) + os.pathsep + os.environ.get("PATH", "")
    actual_commit = host_process.check_output(["git", "rev-parse", "HEAD"], cwd=source, text=True, timeout=10).strip()
    if actual_commit != args.expected_commit:
        raise SystemExit("Checkout differs from the explicitly frozen commit")
    runner = import_runner(source)
    source_environment = runner.environment

    def selected_environment():
        env = source_environment()
        # The repository discovers a second local QEMU installation. Explicit
        # --qemu takes precedence in this child-only environment, without
        # changing the host PATH or weakening actual process-image checks.
        env["PATH"] = str(qemu.parent) + os.pathsep + env.get("PATH", "")
        return env

    runner.environment = selected_environment
    if Path(runner.BUILD).resolve() != build.resolve():
        raise SystemExit("Imported BUILD differs from the selected build")
    frozen = runner.source_hash()
    manifest = runner.validate_build(frozen)
    fixture = load_fixture(runner, fixture_dir, frozen)
    files = [Path(__file__).resolve(), Path(__file__).with_name("windows_inputs.py").resolve(),
             Path(__file__).with_name("windows_qmp.py").resolve()]
    if args.measure_idle:
        files.append(Path(__file__).with_name("windows_idle_measure.py").resolve())
    adapter_hashes = {str(path): digest(assert_plain_path(path, regular=True)) for path in files}
    qemu_hash = digest(qemu)
    # The source verifier permits at most 18 bounded oracle files, all checked
    # again by the original native runner after its complete recovery route.
    if args.check_only:
        print(json.dumps({"passed": True, "check_only": True, "native_gate": False,
            "source_commit": actual_commit, "source_sha256": frozen, "feature": manifest["kernel_feature"],
            "qemu_sha256": qemu_hash, "adapters": adapter_hashes, "vm_started": False}))
        return 0

    from windows_qmp import WindowsQmpPipe
    original_private_path = runner.private_path

    def private_path(path, **kwargs):
        assert_plain_path(Path(path), root=source, regular=kwargs.get("regular", False))
        return original_private_path(path, **kwargs)

    processes = WindowsProcesses(source, build, qemu, args.machine, args.acceleration,
                                 args.timeout, assert_plain_path, process_image)
    # Inject only module-local platform dependencies. Imported global os,
    # subprocess, source_hash, validators and the walkthrough are untouched.
    controls = []
    measurements = []

    def observe_first_key(control, command, name, arguments):
        from windows_idle_measure import WindowsIdleMeasurement
        if measurements or processes.run_directory is None:
            raise ValueError("Unexpected second CPU observation or missing owned run directory")
        helper = WindowsIdleMeasurement(runner, fixture, processes.child, processes.run_directory,
                                       control.deadline, args.reference_frame_sha256)
        measurements.append(helper)
        helper.before_first_enter(command, name, arguments)

    runner.QmpPipe = tracked_transport(WindowsQmpPipe, controls, processes,
                                      observe_first_key if args.measure_idle else None)
    runner.private_path = private_path
    runner.generate_fixture = lambda: load_fixture(runner, fixture_dir, frozen)
    runner.subprocess = processes
    runner.os = SimpleNamespace(readlink=processes.readlink)
    report = None
    try:
        report = runner.run(timeout=args.timeout, bios_rom=args.bios_rom, vga_rom=args.vga_rom, smoke=False)
    finally:
        processes.close()
    cleanup = cleanup_transports(controls)
    if report.get("raw_qemu_exit") is None and processes.child is not None:
        report["raw_qemu_exit"] = processes.child.returncode
    try:
        if (runner.source_hash() != frozen or runner.validate_build(frozen) != manifest
                or digest(qemu) != qemu_hash or processes.watchdog_terminated or processes.cleanup_errors
                or processes.actual_command != report.get("command")):
            raise ValueError("Final source/process/build/QEMU binding differs")
        if args.measure_idle:
            if len(measurements) != 1:
                raise ValueError("Missing exactly one opt-in CPU observation")
            validate_idle_evidence(measurements[0])
        for control in cleanup:
            if control["pending"] or control["errors"] or not control["initialized"]:
                raise ValueError("Windows QMP I/O cleanup did not complete cleanly")
        for path, expected in adapter_hashes.items():
            if digest(assert_plain_path(Path(path), regular=True)) != expected:
                raise ValueError("Windows adapter changed during the run")
        if load_fixture(runner, fixture_dir, frozen)["metadata"] != fixture["metadata"]:
            raise ValueError("Windows fixture metadata changed")
    except Exception as error:
        report["passed"] = report["native_gate"] = False
        report["windows_validation_error"] = type(error).__name__ + ": " + str(error)
    if args.measure_idle:
        report["windows_idle_measurement"] = measurements[0].record if len(measurements) == 1 else {"measurement_passed": False, "error": "Measurement hook was not reached"}
    report["windows_adapter"] = {"adapters_sha256": adapter_hashes, "source_commit": actual_commit,
        "source_sha256": frozen, "qemu_sha256": qemu_hash, "requested_acceleration": args.acceleration,
        "original_linux_command": processes.original_command, "actual_windows_command": processes.actual_command,
        "queried_process_image": str(processes.actual_image), "watchdog_terminated": processes.watchdog_terminated,
        "owned_process_id": processes.child.pid if processes.child is not None else None,
        "owned_process_exit": processes.child.returncode if processes.child is not None else None,
        "process_cleanup_errors": processes.cleanup_errors, "qmp_cleanup": cleanup,
        "deltas": ["Windows binary QMP pipes with original protocol validation", "Win32 process image query replaces /proc",
                   "rng-builtin replaces /dev/urandom backend; virtio-rng configuration unchanged",
                   "explicit QEMU first in child PATH, accelerator, -no-user-config and CREATE_NO_WINDOW",
                   "pre-generated source-bound WSL fixture loaded as plain copies"],
        "scope": "Native probe uses original source pixels, ACKs, fault/recovery, disk oracle, cleanup and raw exit 33. No normal ZIP/release or physical hardware acceptance."}
    runner.publish_json(private_path(Path(report["run_directory"]) / "result.json"), report)
    runner.publish_json(private_path(build / "window-session-report.json"), report)
    print(json.dumps({"passed": report["passed"], "native_gate": report["native_gate"],
        "raw_qemu_exit": report.get("raw_qemu_exit"), "windows_validation_error": report.get("windows_validation_error"),
        "report": str(build / "window-session-report.json")}), flush=True)
    return 0 if report["passed"] and report["native_gate"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
