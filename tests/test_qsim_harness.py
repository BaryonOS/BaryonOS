"""Host-only false-green controls. Replayed fixture text never proves a VM run."""
import copy
import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import run_qsim as qsim
import common

SERIAL = (ROOT / "docs/verification/parallel-i03/observed-qsim-serial.txt").read_bytes().decode("utf-8")
FROZEN = "f" * 64


class QuantumSequence(unittest.TestCase):
    def test_observed_native_sequence_decodes_two_processes(self):
        result = qsim.validate_sequence(SERIAL)
        self.assertEqual([p["pid"] for p in result["processes"]], [1, 2])
        self.assertEqual([p["checkpoints"] for p in result["processes"]], [253, 253])
        self.assertEqual(result["timer_irqs"], 3)
        self.assertEqual(result["cleanup_frames"], 96)

    def test_each_marker_must_exist_once_in_exact_order(self):
        for marker in qsim.REQUIRED:
            for replacement in ("", marker + "\n" + marker, marker + ":extra", "prefix-" + marker):
                with self.subTest(marker=marker, replacement=replacement), self.assertRaises(ValueError):
                    qsim.validate_sequence(SERIAL.replace(marker, replacement, 1))
        a, b = qsim.CASES[:2]
        with self.assertRaises(ValueError):
            qsim.validate_sequence(SERIAL.replace(a, "SWAP").replace(b, a).replace("SWAP", b))

    def test_unknown_foreign_and_prefixed_control_lines_are_rejected(self):
        for line in ("QSIM:UNKNOWN:PASS", "M3:PASS", "G0:PASS", "KS-99:PASS", " QSIM:PASS", "", "\ufffd"):
            with self.subTest(line=line), self.assertRaises(ValueError):
                qsim.validate_sequence(SERIAL.replace("QSIM:PASS\n", line + "\nQSIM:PASS\n"))
        with self.assertRaises(ValueError):
            qsim.validate_sequence("firmware QSIM:PASS\n" + SERIAL)

    def test_numbers_have_canonical_width_ascii_and_u64_bounds(self):
        original = "QSIM:TIMER_IRQS:00000000000000000003"
        for value in ("3", "0" * 19, "0" * 21, "+0000000000000000003", "-0000000000000000003", "00000000000000000003 ", str(1 << 64), "٠" * 20):
            with self.subTest(value=value), self.assertRaises(ValueError):
                qsim.validate_sequence(SERIAL.replace(original, "QSIM:TIMER_IRQS:" + value))

    def test_required_progress_resources_and_cleanup_cannot_be_zero_or_wrong(self):
        for label, value in (("QSIM:PROCESS_PID:", 3), ("QSIM:PROCESS_CHECKPOINTS:", 0), ("QSIM:PROCESS_USEFUL_ARENA:", 65537), ("QSIM:PROCESS_MAX_WORK_GAP:", 449), ("QSIM:TIMER_IRQS:", 0), ("QSIM:SYSCALL_CALLS:", 0), ("QSIM:CLEANUP_FRAMES:", 0)):
            lines = SERIAL.splitlines()
            at = next(i for i, line in enumerate(lines) if line.startswith(label))
            lines[at] = label + f"{value:020d}"
            with self.subTest(label=label), self.assertRaises(ValueError):
                qsim.validate_sequence("\n".join(lines))


class BoundArtifacts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="qsim-host-test-", dir=ROOT)
        self.addCleanup(self.temp.cleanup)
        self.build = Path(self.temp.name) / "build"
        self.user = self.build / "users/qsim"
        self.user.mkdir(parents=True)
        (self.build / "esp/EFI/BOOT").mkdir(parents=True)
        (self.user / "qsim-native-probe.elf").write_bytes(b"synthetic host-test ELF bytes")
        (self.user / "disassembly.stdout.txt").write_text("40000000: int 0x80\n")
        (self.user / "symbols.stdout.txt").write_text("host-only fixture\n")
        (self.build / "esp/kernel.elf").write_bytes(b"kernel-prefix" + (self.user / "qsim-native-probe.elf").read_bytes() + b"kernel-suffix")
        (self.build / "esp/EFI/BOOT/BOOTX64.EFI").write_bytes(b"host-only loader")
        self.builder = SimpleNamespace(sources=lambda: {"current.rs": "a" * 64}, inspect_user=lambda _: {"entry": 0x40000000}, inspect_instructions=lambda _: {"decoded_instructions": 1, "int80_sites": 1, "extended_registers_found": False})
        self.addCleanup(patch.stopall)
        patch.object(qsim, "artifact_builder", return_value=self.builder).start()
        self.artifact = {"passed": True, "native_runtime_performed": False, "source_sha256": self.builder.sources(), "software_float_symbols": list(qsim.FLOAT_SYMBOLS), "binary_sha256": qsim.sha256(self.user / "qsim-native-probe.elf"), "disassembly_sha256": qsim.sha256(self.user / "disassembly.stdout.txt"), "elf": self.builder.inspect_user(None), "scalar_gate": self.builder.inspect_instructions(None)}
        self.manifest = {"kernel_feature": "native-qsim", "loader_feature": None, "source_fingerprint_version": 3, "source_sha256": FROZEN, "sdk_runtime": False, "qsim_user": copy.deepcopy(self.artifact), "kernel": {"sha256": qsim.sha256(self.build / "esp/kernel.elf")}, "loader": {"sha256": qsim.sha256(self.build / "esp/EFI/BOOT/BOOTX64.EFI")}}
        self.write_manifests()

    def write_manifests(self):
        (self.build / "manifest.json").write_text(json.dumps(self.manifest))
        (self.user / "build-report.json").write_text(json.dumps(self.artifact))

    def guest(self):
        bound = qsim.validate_build(self.build, FROZEN)
        directory = self.build / "run/vm"
        (directory / "esp/EFI/BOOT").mkdir(parents=True, exist_ok=True)
        for name in bound["expected"]:
            (directory / name).write_bytes((self.build / name).read_bytes())
        for name in ("OVMF_CODE.fd", "OVMF_VARS.template.fd"):
            (directory / name).write_bytes(name.encode())
        (directory / "serial.txt").write_text(SERIAL)
        relative = directory.relative_to(ROOT).as_posix()
        command = ["/host-test/qemu", "-machine", "pc-q35-7.2", "-accel", "tcg,thread=single", "-cpu", "qemu64", "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none", "-serial", "stdio", "-nic", "none", "-no-reboot", "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={relative}/OVMF_CODE.fd", "-drive", f"if=pflash,unit=1,format=raw,file={relative}/OVMF_VARS.fd", "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{relative}/esp", "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
        report = dict(raw_qemu_exit=33, timed_out=False, passed=True, expected_exit=33, artifact_binding_passed=True, required_markers=list(qsim.REQUIRED), forbidden_markers=list(qsim.FORBIDDEN), source_fingerprint_version=3, source_sha256=FROZEN, run_directory=str(directory), command=command, profile="development-package; pc-q35-7.2/qemu64/TCG/1CPU/256MiB", qemu_version="QEMU emulator version host-only fixture", artifact_sha256={name: qsim.sha256(directory / name) for name in (*bound["expected"], "OVMF_CODE.fd", "OVMF_VARS.template.fd")}, expected_artifact_sha256=bound["expected"], stdout=SERIAL)
        return bound, report

    def validate(self, report, bound):
        return qsim.validate_guest(report, bound, FROZEN, "pc-q35-7.2", "/host-test/qemu", self.build)

    def test_matching_host_fixture_and_artifact_set_are_accepted_as_parser_input(self):
        bound, report = self.guest()
        self.assertEqual(self.validate(report, bound)["cleanup_frames"], 96)

    def test_wrong_build_feature_loader_source_or_sdk_profile_fails(self):
        for field, value in (("kernel_feature", "milestone-m3"), ("loader_feature", "graphics-g0"), ("source_sha256", "e" * 64), ("source_fingerprint_version", 2), ("sdk_runtime", True)):
            with self.subTest(field=field):
                old = self.manifest[field]
                self.manifest[field] = value
                self.write_manifests()
                with self.assertRaises(ValueError):
                    qsim.validate_build(self.build, FROZEN)
                self.manifest[field] = old
        self.write_manifests()

    def test_quantum_report_source_and_gate_must_match_exactly(self):
        for field, value in (("passed", "true"), ("native_runtime_performed", True), ("source_sha256", {}), ("software_float_symbols", []), ("elf", {}), ("scalar_gate", {})):
            with self.subTest(field=field):
                old = self.artifact[field]
                self.artifact[field] = value
                self.manifest["qsim_user"] = copy.deepcopy(self.artifact)
                self.write_manifests()
                with self.assertRaises(ValueError):
                    qsim.validate_build(self.build, FROZEN)
                self.artifact[field] = old
        self.manifest["qsim_user"] = copy.deepcopy(self.artifact)
        self.write_manifests()

    def test_mutated_binary_disassembly_and_kernel_cannot_reuse_prior_hashes(self):
        for path in (self.user / "qsim-native-probe.elf", self.user / "disassembly.stdout.txt", self.build / "esp/kernel.elf", self.build / "esp/EFI/BOOT/BOOTX64.EFI"):
            original = path.read_bytes()
            path.write_bytes(original + b"changed")
            with self.subTest(path=path), self.assertRaises(ValueError):
                qsim.validate_build(self.build, FROZEN)
            path.write_bytes(original)

    def test_kernel_hash_alone_does_not_bind_another_embedded_program(self):
        path = self.build / "esp/kernel.elf"
        path.write_bytes(b"other executable")
        self.manifest["kernel"]["sha256"] = qsim.sha256(path)
        self.write_manifests()
        with self.assertRaises(ValueError):
            qsim.validate_build(self.build, FROZEN)

    def test_raw_exit_timeout_stale_source_and_expected_marker_mismatch_fail(self):
        bound, original = self.guest()
        for field, value in (("raw_qemu_exit", 0), ("raw_qemu_exit", 35), ("timed_out", True), ("passed", False), ("source_sha256", "0" * 64), ("expected_exit", 35), ("required_markers", ["M1:PASS"]), ("forbidden_markers", []), ("artifact_binding_passed", False)):
            report = copy.deepcopy(original)
            report[field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.validate(report, bound)

    def test_profile_command_and_guest_artifact_changes_are_rejected(self):
        bound, original = self.guest()
        cases = [("profile", "development-package; pc-q35-10.0/qemu64/TCG/1CPU/256MiB"), ("command", original["command"] + ["-nic", "user"]), ("qemu_version", ""), ("artifact_sha256", {}), ("expected_artifact_sha256", {})]
        for field, value in cases:
            report = copy.deepcopy(original)
            report[field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.validate(report, bound)
        (Path(original["run_directory"]) / "esp/kernel.elf").write_bytes(b"changed after snapshot")
        with self.assertRaises(ValueError):
            self.validate(original, bound)

    def test_reported_stdout_and_private_run_location_must_match(self):
        bound, original = self.guest()
        report = copy.deepcopy(original)
        report["stdout"] += "extra\n"
        with self.assertRaises(ValueError):
            self.validate(report, bound)
        report = copy.deepcopy(original)
        report["run_directory"] = str(self.build)
        with self.assertRaises(ValueError):
            self.validate(report, bound)

    def test_duplicate_json_keys_are_not_evidence(self):
        (self.build / "manifest.json").write_text('{"passed":false,"passed":true}')
        with self.assertRaises(ValueError):
            qsim.validate_build(self.build, FROZEN)

    def test_source_change_during_a_mocked_guest_run_is_never_accepted(self):
        _, guest = self.guest()
        executable = self.build / "host-only-qemu-entry"
        executable.write_bytes(b"not executable; mock fixture only")
        guest["command"][0] = str(executable)
        with patch.object(qsim, "BUILD", self.build), patch.object(common, "qemu_machine", return_value="pc-q35-7.2", create=True), patch.object(qsim, "environment", return_value={"PATH": "/host-only"}), patch.object(qsim.shutil, "which", return_value=str(executable)), patch.object(qsim, "run_guest", return_value=guest), patch.object(qsim, "source_hash", side_effect=[FROZEN, FROZEN, "e" * 64]):
            result = qsim.run_qsim()
        self.assertFalse(result["passed"])
        self.assertFalse(result["checks"]["after_run_source_unchanged"])


if __name__ == "__main__":
    unittest.main()
