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
SESSION, UPLOAD = PAYLOAD[:40], PAYLOAD[40:]
# Independent of the runner: every 16-byte window of each 40-byte token.
WINDOWS = [token[start:start + 16] for token in (SESSION, UPLOAD) for start in range(25)]


class FakeQemu:
    """Stand-in at the subprocess.Popen boundary: exact stdout/stderr bytes, exit 33.

    It mocks only the QEMU process; it proves how the runner handles raw
    output, not that a guest ran."""

    def __init__(self, stdout, stderr):
        self.stdout, self.stderr, self.returncode = io.BytesIO(stdout), io.BytesIO(stderr), 33

    def communicate(self, timeout=None):
        return self.stdout.read(), self.stderr.read()

    def poll(self):
        return self.returncode

    def wait(self, timeout=None):
        return self.returncode

    def kill(self):
        pass


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
    def test_safe_serial_keeps_only_allowlisted_failure_diagnostics(self):
        serial = "\n".join([
            "NAS:FAIL:ROLE_RESULT",
            "NAS:ERROR:SLOT:00000000000000000003",
            "NAS:ERROR:STEP:00000000000000000017",
            "NAS:TIMEOUT:SLOT:00000000000000000002",
            "NAS:TIMEOUT:STEP:00000000000000000123",
            "NAS:FAIL:" + PAYLOAD.hex().upper(),
            "NAS:ERROR:STEP:000000000000000000017",
            "NAS:FAIL:UNEXPECTED SECRET",
        ]) + "\n"

        archived = runner.safe_serial_evidence(serial)

        self.assertIn("NAS:FAIL:ROLE_RESULT\n", archived)
        self.assertIn("NAS:ERROR:SLOT:00000000000000000003\n", archived)
        self.assertIn("NAS:ERROR:STEP:00000000000000000017\n", archived)
        self.assertIn("NAS:TIMEOUT:SLOT:00000000000000000002\n", archived)
        self.assertIn("NAS:TIMEOUT:STEP:00000000000000000123\n", archived)
        self.assertNotIn(PAYLOAD.hex().upper(), archived)
        self.assertNotIn("UNEXPECTED SECRET", archived)
        self.assertIn("NAS:OUTPUT:REDACTED\n", archived)

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
        self.capture_mode = None
        self.serial_extra, self.stderr_extra, self.outputs = {}, {}, {}
        self.executions = 0
        self.handoff = runner.private_handoff_dir()
        self.addCleanup(runner.shutil.rmtree, self.handoff, True)
        for target, name, value in [
            (runner, "BUILD", self.build),
            (runner, "source_hash", lambda: "fixed source"),
            (runner, "environment", lambda: {"PATH": ""}),
            (runner, "build", self.fake_build),
            (runner, "firmware", lambda: (self.code, self.variables)),
            (runner.subprocess, "Popen", self.fake_popen),
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

    def fake_popen(self, command, **_kwargs):
        self.executions += 1
        first_drive = command[command.index("-drive") + 1]
        run = (runner.ROOT / first_drive.split("file=", 1)[1]).parent
        if self.mutation:
            self.mutation(run)
        case = "recover" if "nas-recover-" in str(run) else "normal"
        if case == "normal":
            chardev = command[command.index("-chardev") + 1]
            capture_path = Path(chardev.split("path=", 1)[1])
            capture_path.write_bytes(self.capture)
            if self.capture_mode is not None:
                capture_path.chmod(self.capture_mode)
        else:
            fw_cfg = command[command.index("-fw_cfg") + 1]
            self.cold_input_seen = Path(fw_cfg.split("file=", 1)[1]).read_bytes()
        stdout = transcript(case).encode() + self.serial_extra.get(case, b"")
        stderr = self.stderr_extra.get(case, b"")
        self.outputs[case] = (stdout, stderr)
        return FakeQemu(stdout, stderr)

    def run_silently(self, case="normal", parent=None):
        with contextlib.redirect_stdout(io.StringIO()):
            return runner.run_case(case, parent, self.handoff)

    def assert_leak_fails(self, case="normal", parent=None, *labels):
        report = self.run_silently(case, parent)
        self.assertIs(report["passed"], False)
        for label in labels:
            self.assertIn(label, report.get("error", ""))
        run = Path(report["run_directory"])
        for name, original in zip(("serial", "stderr"), self.outputs[case]):
            archived = (run / f"{name}.txt").read_text()
            self.assertTrue(archived.startswith("REDACTED"), name)
            self.assertIn(runner.hashlib.sha256(original).hexdigest(), archived)
            self.assertNotIn("NAS:CASE:", archived)
        return report

    def assert_no_token_material(self, *folders):
        for folder in (self.base, *folders):
            for path in folder.rglob("*"):
                if not path.is_file():
                    continue
                data = path.read_bytes()
                for window in WINDOWS:
                    self.assertNotIn(window, data, path)
                    self.assertNotIn(window.hex().encode(), data.lower(), path)

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
        for report in (parent, cold):
            run = Path(report["run_directory"])
            self.assertEqual((run / "serial.txt").read_text(), transcript(report["case"]))
            self.assertEqual((run / "stderr.txt").read_text(), "")
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

    def test_archive_filters_unlisted_lines_after_raw_validation(self):
        self.serial_extra["normal"] = b"unlisted debug output\n"
        report = self.run_silently()
        self.assertIs(report["passed"], True, report.get("error"))
        archived = (Path(report["run_directory"]) / "serial.txt").read_text()
        self.assertNotIn("unlisted debug output", archived)
        self.assertEqual(archived, transcript("normal") + "NAS:OUTPUT:REDACTED\n")

    def test_misframed_capture_is_searched_whole_and_redacted(self):
        # A second exported pair and a junk-prefixed frame both fail framing;
        # the leak check must still see every captured byte (fail closed).
        second = bytes(range(160, 240))
        cases = (
            ("double-frame", runner.REPLAY_MAGIC + PAYLOAD + runner.REPLAY_MAGIC + second, second[:40]),
            ("junk-prefix", bytes(64) + runner.REPLAY_MAGIC + PAYLOAD, SESSION),
            ("wrong-magic", b"NOSRPL02" + PAYLOAD, SESSION[8:32]),
        )
        for name, capture, leaked in cases:
            with self.subTest(case=name):
                for path in list(self.handoff.iterdir()):
                    path.unlink()
                self.capture = capture
                self.serial_extra["normal"] = leaked.hex().encode() + b"\n"
                self.assert_leak_fails("normal", None, "capture:serial:hex")
                for run in (self.base / "build").rglob("nas-normal-*"):
                    if run.is_dir() and (run / "serial.txt").exists():
                        self.assertNotIn(leaked.hex(), (run / "serial.txt").read_text().lower())

    def test_unreadable_or_oversize_capture_redacts_output_without_a_reference(self):
        for name, capture, mode in (("oversize", runner.REPLAY_MAGIC + PAYLOAD + bytes(runner.CAPTURE_LIMIT), None),
                                    ("group-readable", runner.REPLAY_MAGIC + PAYLOAD, 0o644)):
            with self.subTest(case=name):
                for path in list(self.handoff.iterdir()):
                    path.unlink()
                self.capture, self.capture_mode = capture, mode
                self.assert_leak_fails("normal", None, "reference-unavailable")
        self.capture_mode = None

    def test_realistic_shared_prefix_tokens_are_detected_without_false_positives(self):
        # services/nas/native-core encode_token: boot(32 random, shared) || le64(serial).
        boot = bytes(range(200, 232))
        session, upload = boot + (1).to_bytes(8, "little"), boot + (2).to_bytes(8, "little")
        references = runner.pair_references(session + upload)
        self.assertEqual(runner.token_leaks({"serial": transcript("normal").encode()}, references), [])
        found = runner.token_leaks({"serial": b"x " + upload.hex().encode()}, references)
        self.assertIn("upload:serial:hex", found)

    def test_token_leaks_checks_every_window_of_both_tokens(self):
        references = runner.pair_references(PAYLOAD)
        for window in WINDOWS:
            for form, data in (("raw", window), ("hex", window.hex().encode())):
                with self.subTest(window=window[:2].hex(), form=form):
                    self.assertTrue(runner.token_leaks({"serial": b"<" + data + b">"}, references))
        self.assertEqual(runner.token_leaks({"serial": SESSION[:15].hex().encode()}, references), [])

    def test_empty_capture_is_not_redacted_and_fails_on_framing(self):
        self.capture = b""
        report = self.run_silently()
        self.assertIs(report["passed"], False)
        archived = (Path(report["run_directory"]) / "serial.txt").read_text()
        self.assertFalse(archived.startswith("REDACTED"))

    def test_token_bytes_in_archived_serial_fail_the_run(self):
        self.serial_extra["normal"] = PAYLOAD.hex().encode() + b"\n"
        self.assertIs(self.run_silently()["passed"], False)

    def test_session_token_alone_as_hex_fails_the_run(self):
        self.serial_extra["normal"] = b"debug " + SESSION.hex().encode() + b"\n"
        self.assert_leak_fails("normal", None, "session:serial:hex")

    def test_upload_token_alone_fails_the_run(self):
        self.serial_extra["normal"] = UPLOAD.hex().encode() + b"\n"
        report = self.assert_leak_fails("normal", None, "upload:serial:hex")
        # Holds only for this synthetic disjoint fixture; real tokens share a
        # 32-byte prefix (see test_realistic_shared_prefix_tokens_...).
        self.assertNotIn("session:", report["error"])

    def test_uppercase_hex_token_fails_the_run(self):
        self.serial_extra["normal"] = UPLOAD.hex().upper().encode() + b"\n"
        self.assert_leak_fails("normal", None, "upload:serial:hex")

    def test_raw_token_bytes_fail_the_run_before_text_decoding(self):
        # UPLOAD contains 0x80..0x8c: not valid UTF-8, so a decoded-text
        # search can never see these bytes.
        for token, label in ((UPLOAD, "upload:serial:raw"), (SESSION, "session:serial:raw")):
            with self.subTest(label=label):
                self.serial_extra["normal"] = b"\x00" + token + b"\n"
                self.assert_leak_fails("normal", None, label)

    def test_every_16_byte_token_window_fails_the_run(self):
        for token, start, form, label in ((SESSION, 0, "hex", "session:serial:hex"),
                                          (SESSION, 12, "hex", "session:serial:hex"),
                                          (SESSION, 24, "raw", "session:serial:raw"),
                                          (UPLOAD, 7, "raw", "upload:serial:raw"),
                                          (UPLOAD, 24, "hex", "upload:serial:hex")):
            with self.subTest(label=label, start=start):
                window = token[start:start + 16]
                self.serial_extra["normal"] = (window.hex().encode() if form == "hex" else window) + b"\n"
                self.assert_leak_fails("normal", None, label)
        # Documented threshold (LEAK_WINDOW): shorter fragments are not matched.
        self.serial_extra["normal"] = SESSION[:15].hex().encode() + b"\n"
        self.assertIs(self.run_silently()["passed"], True)

    def test_token_leak_in_qemu_stderr_fails_the_run(self):
        self.stderr_extra["normal"] = b"qemu: " + SESSION.hex().encode() + b"\n"
        self.assert_leak_fails("normal", None, "session:stderr:hex")

    def test_cold_run_token_leak_is_detected_before_archiving(self):
        parent = self.run_silently()
        self.assertIs(parent["passed"], True, parent.get("error"))
        self.serial_extra["recover"] = SESSION.hex().encode() + b"\n"
        self.assert_leak_fails("recover", parent, "session:serial:hex")
        self.assert_no_token_material()

    def test_leaking_run_leaves_no_original_output_on_disk(self):
        self.serial_extra["normal"] = b"debug " + PAYLOAD.hex().encode() + b"\n"
        self.stderr_extra["normal"] = b"qemu: " + UPLOAD + b"\n"
        report = self.assert_leak_fails("normal", None, "session:serial:hex", "upload:stderr:raw")
        self.assertEqual(sorted(path.name for path in self.handoff.iterdir()), [])
        self.assert_no_token_material(self.handoff)
        self.assertNotIn(PAYLOAD.hex(), runner.json.dumps(report))

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

    def test_refused_handoff_directory_inside_the_checkout_is_removed(self):
        inside = self.base / "handoff-inside-checkout"
        inside.mkdir()
        with patch.object(runner.tempfile, "mkdtemp", lambda **_kwargs: str(inside)):
            with self.assertRaisesRegex(ValueError, "outside the checkout"):
                runner.private_handoff_dir()
        self.assertFalse(inside.exists())

    def test_gate_removes_the_handoff_directory_on_leak_and_unexpected_error(self):
        made, original = [], runner.private_handoff_dir

        def handoff_dir():
            made.append(original())
            return made[-1]

        self.serial_extra["normal"] = SESSION.hex().encode() + b"\n"
        with patch.object(runner, "private_handoff_dir", handoff_dir):
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(runner.main(), 1)
            with patch.object(runner, "run_case", side_effect=TypeError("unexpected")):
                with self.assertRaises(TypeError), contextlib.redirect_stdout(io.StringIO()):
                    runner.main()
        self.assertEqual(len(made), 2)
        for directory in made:
            self.assertFalse(directory.exists(), directory)
        self.assert_no_token_material()

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
