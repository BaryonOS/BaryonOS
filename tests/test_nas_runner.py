"""Synthetic runner-verifier negatives, not evidence that the guest ran."""
from pathlib import Path
import contextlib
import io
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import test_nas as runner


PAYLOAD = bytes(range(1, 41)) + bytes(range(101, 141))


def transcript(case):
    numbers = [("ADMIN_PID", 1), ("GUEST_PID", 4)]
    counts = [(6, 1), (12, 6), (6, 5)] if case == "normal" else [(4, 0), (4, 5), (0, 5)]
    for slot, (ok, errors) in zip((0, 1, 3), counts):
        numbers.extend([("ROLE_SLOT", slot), ("RPC_OK", ok), ("RPC_ERRORS", errors),
                        ("RIGHTS_DENIED", int(slot == 3)), ("STACK_TOUCHED", 4096)])
    numbers.extend([("ENTROPY_BYTES", 256), ("DISK_READS", 1),
                    ("DISK_WRITES", int(case == "normal")),
                    ("DISK_FLUSHES", int(case == "normal"))])
    head = (["NAS:MANIFEST:PASS"] if case == "normal"
            else ["NAS:REPLAY_INPUT:PASS", "NAS:MANIFEST:PASS"])
    export = ["NAS:REPLAY_EXPORT:PASS"] if case == "normal" else []
    return "\n".join([*runner.BOOT, *head,
                      *[f"NAS:{name}:{value:020d}" for name, value in numbers],
                      *export, "NAS:BROKERS:PASS", "NAS:CLEANUP:PASS",
                      f"NAS:CASE:{case.upper()}:PASS", "M1:PASS"]) + "\n"


class NasRunnerTests(unittest.TestCase):
    def test_exact_normal_and_cold_evidence(self):
        for case in runner.FEATURES:
            result = runner.evaluate(case, transcript(case), 33, False)
            self.assertEqual((result["admin_pid"], result["guest_pid"]), (1, 4))

    def test_bad_exit_timeout_and_late_failure_rejected(self):
        serial = transcript("normal")
        for raw, timeout, text in [(0, False, serial), (True, False, serial),
                                   (33, True, serial), (33, False, serial + "PANIC\n"),
                                   (33, False, serial + "NAS:FAIL:late\n")]:
            with self.subTest(raw=raw, timeout=timeout, text=text[-20:]):
                with self.assertRaises(ValueError):
                    runner.evaluate("normal", text, raw, timeout)

    def test_missing_duplicated_or_changed_role_assertions_rejected(self):
        serial = transcript("normal")
        line = "NAS:RIGHTS_DENIED:00000000000000000001"
        mutations = [
            serial.replace(line, "NAS:RIGHTS_DENIED:00000000000000000000"),
            serial.replace(line, ""),
            serial.replace(line, line + "\n" + line),
            serial.replace("NAS:STACK_TOUCHED:00000000000000004096",
                           "NAS:STACK_TOUCHED:00000000000000065537"),
            serial.replace("NAS:GUEST_PID:00000000000000000004",
                           "NAS:GUEST_PID:00000000000000000001"),
            serial.replace("NAS:RPC_ERRORS:00000000000000000006",
                           "NAS:RPC_ERRORS:00000000000000000005"),
        ]
        for text in mutations:
            with self.assertRaises(ValueError):
                runner.evaluate("normal", text, 33, False)

    def test_replay_markers_are_case_bound(self):
        normal, cold = transcript("normal"), transcript("recover")
        for case, text in [("normal", normal.replace("NAS:REPLAY_EXPORT:PASS\n", "")),
                           ("normal", normal + "NAS:REPLAY_EXPORT:PASS\n"),
                           ("recover", cold.replace("NAS:REPLAY_INPUT:PASS\n", "")),
                           ("recover", cold.replace("NAS:REPLAY_INPUT:PASS", "NAS:REPLAY_EXPORT:PASS")),
                           # The pre-#110 synthetic cold counters no longer pass.
                           ("recover", cold.replace("NAS:RPC_ERRORS:00000000000000000005",
                                                    "NAS:RPC_ERRORS:00000000000000000004"))]:
            with self.subTest(case=case, text=text[-60:]), self.assertRaises(ValueError):
                runner.evaluate(case, text, 33, False)

    def test_cold_write_and_missing_entropy_or_read_rejected(self):
        serial = transcript("recover")
        for name, old, new in [("DISK_WRITES", 0, 1), ("DISK_FLUSHES", 0, 1),
                                ("DISK_READS", 1, 0), ("ENTROPY_BYTES", 256, 0)]:
            with self.subTest(name=name):
                text = serial.replace(f"NAS:{name}:{old:020d}", f"NAS:{name}:{new:020d}")
                with self.assertRaises(ValueError):
                    runner.evaluate("recover", text, 33, False)


class NasLifecycleTests(unittest.TestCase):
    def setUp(self):
        root = runner.ROOT / "build"
        root.mkdir(exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix="nas-lifecycle-test-", dir=root)
        self.addCleanup(temporary.cleanup)
        self.base = Path(temporary.name)
        self.build = self.base / "build"
        self.code, self.variables = self.base / "code.fd", self.base / "vars.fd"
        self.code.write_bytes(b"synthetic code")
        self.variables.write_bytes(b"synthetic vars")
        self.qemu = self.base / "qemu"
        self.qemu.write_bytes(b"\x7fELF synthetic test implementation")
        self.mutation = None
        self.capture = runner.REPLAY_MAGIC + PAYLOAD
        self.serial_extra = ""
        self.raw_serial_extra = b""
        self.raw_stderr_extra = b""
        self.executions = 0
        self.handoff = runner.private_handoff_dir()
        self.addCleanup(runner.shutil.rmtree, self.handoff, True)
        for target, name, value in [
            (runner, "BUILD", self.build),
            (runner, "source_hash", lambda: "fixed source"),
            (runner, "environment", lambda: {"PATH": ""}),
            (runner, "build", self.fake_build),
            (runner, "firmware", lambda: (self.code, self.variables)),
            (runner, "execute", self.fake_execute),
            (runner, "check_nas_disk", lambda *args: {"synthetic": True}),
            (runner.shutil, "which", lambda *args, **kwargs: str(self.qemu)),
            (runner.subprocess, "check_output",
             lambda command, **kwargs: "a" * 40 if command[0] == "git" else "QEMU test\n"),
        ]:
            context = patch.object(target, name, value)
            context.start()
            self.addCleanup(context.stop)

    def fake_build(self, _loader, feature, _label):
        esp = self.build / "esp"
        (esp / "EFI/BOOT").mkdir(parents=True, exist_ok=True)
        (esp / "kernel.elf").write_bytes(b"synthetic kernel")
        (esp / "EFI/BOOT/BOOTX64.EFI").write_bytes(b"synthetic loader")
        return {
            "source_sha256": "fixed source",
            "nas_probe": {"synthetic": True},
            "kernel_feature": feature,
            "kernel": {"sha256": runner.sha256(esp / "kernel.elf")},
            "loader": {"sha256": runner.sha256(esp / "EFI/BOOT/BOOTX64.EFI")},
        }

    def fake_execute(self, command, _env, **kwargs):
        self.executions += 1
        first_drive = command[command.index("-drive") + 1]
        run = (runner.ROOT / first_drive.split("file=", 1)[1]).parent
        if self.mutation:
            self.mutation(run)
        case = "recover" if "nas-recover-" in str(run) else "normal"
        if case == "normal":
            chardev = command[command.index("-chardev") + 1]
            Path(chardev.split("path=", 1)[1]).write_bytes(self.capture)
        else:
            fw_cfg = command[command.index("-fw_cfg") + 1]
            self.cold_input_seen = Path(fw_cfg.split("file=", 1)[1]).read_bytes()
        serial = transcript(case) + self.serial_extra
        raw_serial = serial.encode("utf-8") + self.raw_serial_extra
        return (33, raw_serial.decode("utf-8", "replace"), "", False, False, False,
                raw_serial, self.raw_stderr_extra)

    def run_silently(self, case="normal", parent=None):
        with contextlib.redirect_stdout(io.StringIO()):
            return runner.run_case(case, parent, self.handoff)

    def test_exact_pair_reaches_the_cold_vm_only_through_the_private_handoff(self):
        parent = self.run_silently()
        self.assertIs(parent["passed"], True, parent.get("error"))
        binding = parent["replay_handoff"]
        self.assertEqual(binding["payload_sha256"], runner.hashlib.sha256(PAYLOAD).hexdigest())
        self.assertNotIn(PAYLOAD.hex(), runner.json.dumps(parent))
        self.assertFalse((self.handoff / "normal-capture.bin").exists())
        cold = self.run_silently("recover", parent)
        self.assertIs(cold["passed"], True, cold.get("error"))
        self.assertEqual(self.cold_input_seen, PAYLOAD)
        self.assertEqual(cold["replay_input"]["payload_sha256"], binding["payload_sha256"])
        self.assertNotIn(PAYLOAD.hex(), runner.json.dumps(cold))
        for path in self.handoff.iterdir():
            self.assertEqual(path.stat().st_mode & 0o077, 0, path.name)

    def test_missing_short_or_unframed_capture_fails_the_normal_run(self):
        for capture in (b"", runner.REPLAY_MAGIC + PAYLOAD[:-1], b"NOSRPL02" + PAYLOAD,
                        runner.REPLAY_MAGIC + bytes(40) + PAYLOAD[40:],
                        runner.REPLAY_MAGIC + PAYLOAD + b"x"):
            with self.subTest(capture=capture[:12]):
                for path in list(self.handoff.iterdir()):
                    path.unlink()
                self.capture = capture
                self.assertIs(self.run_silently()["passed"], False)

    def test_token_bytes_in_archived_serial_fail_the_run(self):
        leaks = (PAYLOAD.hex().encode(), PAYLOAD[:40].hex().encode(),
                 PAYLOAD[40:].hex().encode(), PAYLOAD)
        for leak in leaks:
            with self.subTest(leak=leak[:12]):
                self.serial_extra = ""
                self.raw_serial_extra = b""
                self.raw_stderr_extra = b""
                if leak == PAYLOAD:
                    self.raw_serial_extra = leak
                else:
                    self.serial_extra = leak.decode("ascii") + "\n"
                result = self.run_silently()
                self.assertIs(result["passed"], False)
                run = Path(result["run_directory"])
                artifacts = b"".join(path.read_bytes() for path in run.iterdir() if path.is_file())
                self.assertNotIn(leak, artifacts)
                self.assertNotIn(PAYLOAD.hex().encode(), artifacts)
                self.assertNotIn(PAYLOAD[:40].hex().encode(), artifacts)
                self.assertNotIn(PAYLOAD[40:].hex().encode(), artifacts)
                self.assertIn(b"NAS:OUTPUT:REDACTED", (run / "serial.txt").read_bytes())

    def test_token_bytes_on_stderr_are_not_archived(self):
        self.raw_stderr_extra = PAYLOAD
        result = self.run_silently()
        self.assertIs(result["passed"], False)
        run = Path(result["run_directory"])
        artifacts = b"".join(path.read_bytes() for path in run.iterdir() if path.is_file())
        self.assertNotIn(PAYLOAD, artifacts)
        self.assertIn(b"withheld", (run / "stderr.txt").read_bytes())

    def test_cold_run_refuses_a_changed_or_unbound_handoff(self):
        parent = self.run_silently()
        self.assertIs(parent["passed"], True, parent.get("error"))
        payload = self.handoff / "payload.bin"
        original = payload.read_bytes()
        # Both refusals must happen before any QEMU start.
        before = self.executions
        payload.write_bytes(bytes(80))
        with self.assertRaises(ValueError):
            self.run_silently("recover", parent)
        payload.write_bytes(original)
        other = dict(parent, replay_handoff=dict(parent["replay_handoff"], origin_run_id="other-run"))
        with self.assertRaises(ValueError):
            self.run_silently("recover", other)
        self.assertEqual(self.executions, before)

    def test_cold_input_changed_during_the_run_is_rejected(self):
        parent = self.run_silently()
        self.assertIs(parent["passed"], True, parent.get("error"))

        def mutate(run):
            cold = self.handoff / "cold-input.bin"
            cold.chmod(0o600)
            cold.write_bytes(bytes(80))
            cold.chmod(0o400)

        self.mutation = mutate
        self.assertIs(self.run_silently("recover", parent)["passed"], False)

    def test_runs_without_a_private_handoff_directory_are_refused(self):
        with self.assertRaisesRegex(ValueError, "handoff"):
            with contextlib.redirect_stdout(io.StringIO()):
                runner.run_case("normal", None, None)
        self.assertEqual(self.executions, 0)

    def test_artifact_mutation_after_launch_cannot_pass(self):
        for relative in ("esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI", "OVMF_CODE.fd"):
            with self.subTest(relative=relative):
                self.mutation = lambda run: (run / relative).write_bytes(b"changed")
                self.assertIs(self.run_silently()["passed"], False)

    def test_platform_drift_is_rejected_before_cold_launch(self):
        parent = self.run_silently()
        self.assertIs(parent["passed"], True)
        for change in ("firmware", "vars-template", "qemu", "machine"):
            with self.subTest(change=change):
                old_code, old_qemu = self.code.read_bytes(), self.qemu.read_bytes()
                old_vars = self.variables.read_bytes()
                count = self.executions
                # A real drift: always a machine other than the parent's,
                # independent of NOS_QEMU_MACHINE in the test environment.
                other = next(m for m in ("pc-q35-7.2", "pc-q35-10.0")
                             if m != parent["platform"]["machine"])
                context = patch.object(runner, "qemu_machine", return_value=other)
                if change == "firmware":
                    self.code.write_bytes(b"different firmware")
                if change == "qemu":
                    self.qemu.write_bytes(b"\x7fELF different implementation")
                if change == "vars-template":
                    self.variables.write_bytes(b"different original vars")
                with context if change == "machine" else contextlib.nullcontext():
                    with self.assertRaisesRegex(ValueError, "platform"):
                        self.run_silently("recover", parent)
                self.assertEqual(self.executions, count)
                self.code.write_bytes(old_code)
                self.qemu.write_bytes(old_qemu)
                self.variables.write_bytes(old_vars)

    def test_mutable_guest_vars_are_not_treated_as_immutable_template(self):
        self.mutation = lambda run: (run / "OVMF_VARS.fd").write_bytes(b"guest updated vars")
        self.assertIs(self.run_silently()["passed"], True)

    def test_qemu_and_original_firmware_cannot_change_during_execution(self):
        for path in (self.qemu, self.code, self.variables):
            with self.subTest(path=path.name):
                original = path.read_bytes()
                self.mutation = lambda _run: path.write_bytes(b"changed")
                self.assertIs(self.run_silently()["passed"], False)
                path.write_bytes(original)

    def test_unchanged_platform_accepts_cold_pair(self):
        parent = self.run_silently()
        self.assertIs(parent["passed"], True)
        cold = self.run_silently("recover", parent)
        self.assertIs(cold["passed"], True)
        self.assertEqual(parent["platform"], cold["platform"])
        self.assertEqual(self.executions, 2)

    def test_wrapper_is_setup_failure_before_execute(self):
        self.qemu.write_bytes(b"#!/bin/sh\nexec other-qemu\n")
        with self.assertRaisesRegex(RuntimeError, "actual QEMU ELF"):
            self.run_silently()
        self.assertEqual(self.executions, 0)

    def test_version_probe_cannot_change_executable_before_vm(self):
        def version_probe(command, **_kwargs):
            self.qemu.write_bytes(b"changed by version probe")
            return "QEMU test"

        with patch.object(runner.subprocess, "check_output", version_probe):
            with self.assertRaisesRegex(ValueError, "immutable"):
                self.run_silently()
        self.assertEqual(self.executions, 0)


if __name__ == "__main__":
    unittest.main()
