from contextlib import ExitStack
import copy
import hashlib
import io
import json
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_materiaal_preview as preview
import run_session_preview as session_preview
import run_fpu
from run_materiaal_preview import BOOT, READY, verify_frame, verify_ready_transcript


def sent_qcodes(trace):
    """Inspect keyboard operations, never UUIDs or screenshot filenames."""
    return [key["data"] for item in trace
            if item["request"]["execute"] == "send-key"
            for key in item["request"]["arguments"]["keys"]
            if key["type"] == "qcode"]


class MateriaalPreviewTests(unittest.TestCase):
    def test_f12_in_screenshot_path_is_not_a_shutdown_key(self):
        trace = [{"request": {"execute": "screendump", "arguments": {
            "filename": "/owned/materiaal-18f5542f12/ready.ppm"}}}]
        self.assertNotIn("f12", sent_qcodes(trace))

    def test_actual_f12_qcode_is_still_detected(self):
        trace = [{"request": {"execute": "send-key", "arguments": {
            "keys": [{"type": "qcode", "data": "f12"}]}}}]
        self.assertIn("f12", sent_qcodes(trace))

    def test_invalid_build_removes_previous_success_before_validation(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            report = root / "preview-result.json"
            report.write_text('{"passed": true}')
            with (
                patch.object(run_fpu, "ROOT", root),
                patch.object(preview, "validate_build", side_effect=ValueError("Invalid build")),
                patch.object(preview.subprocess, "Popen") as start,
            ):
                with self.assertRaisesRegex(ValueError, "Invalid build"):
                    preview.run_preview(
                        build_directory=root, bios_rom=root / "bios", vga_rom=root / "vga"
                    )
                self.assertFalse(report.exists())
                start.assert_not_called()

    def test_invalid_firmware_removes_previous_success_before_preparation(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            report = root / "preview-result.json"
            report.write_text('{"passed": true}')
            with (
                patch.object(run_fpu, "ROOT", root),
                patch.object(preview, "validate_build", return_value=(root, {"source_sha256": "a" * 64})),
                patch.object(preview, "source_hash", return_value="a" * 64),
                patch.object(preview, "environment", return_value={"PATH": ""}),
                patch.object(preview.shutil, "which", return_value="/unused/qemu"),
                patch.object(preview.subprocess, "check_output", return_value="QEMU"),
                patch.object(preview, "firmware", side_effect=ValueError("Invalid firmware")),
                patch.object(preview.subprocess, "Popen") as start,
            ):
                with self.assertRaisesRegex(ValueError, "Invalid firmware"):
                    preview.run_preview(
                        build_directory=root, bios_rom=root / "bios", vga_rom=root / "vga"
                    )
                self.assertFalse(report.exists())
                start.assert_not_called()

    def test_report_symlink_is_rejected_without_touching_target(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            target = root / "other.json"
            target.write_text('{"passed": true}')
            (root / "preview-result.json").symlink_to(target)
            with patch.object(run_fpu, "ROOT", root):
                with self.assertRaisesRegex(ValueError, "Symlink"):
                    preview.run_preview(
                        build_directory=root, bios_rom=root / "bios", vga_rom=root / "vga"
                    )
            self.assertEqual(target.read_text(), '{"passed": true}')

    def test_requires_boot_before_manifest_and_each_native_ready_marker(self):
        transcript = "\n".join(("KS-01:PASS", *BOOT, *READY)).encode()
        self.assertEqual(verify_ready_transcript(transcript), list((*BOOT, *READY)))

    def test_accepts_independent_service_readiness_order(self):
        transcript = "\n".join((*BOOT, READY[0], READY[2], READY[1], *READY[3:])).encode()
        self.assertEqual(
            verify_ready_transcript(transcript),
            list((*BOOT, READY[0], READY[2], READY[1], *READY[3:])),
        )

    def test_rejects_missing_or_repeated_ready_markers(self):
        missing = "\n".join((*BOOT, *READY[:-1])).encode()
        duplicate = "\n".join((*BOOT, *READY, READY[2])).encode()
        with self.assertRaises(ValueError):
            verify_ready_transcript(missing)
        with self.assertRaises(ValueError):
            verify_ready_transcript(duplicate)

    def test_rejects_readiness_before_manifest_or_reordered_boot(self):
        for markers in (
            (*BOOT, READY[1], READY[0], *READY[2:]),
            (BOOT[1], BOOT[0], BOOT[2], *READY),
        ):
            with self.assertRaises(ValueError):
                verify_ready_transcript("\n".join(markers).encode())

    def test_rejects_guest_failure_even_after_readiness(self):
        transcript = "\n".join((*BOOT, *READY, "MATERIAAL:FAIL:TEST")).encode()
        with self.assertRaises(ValueError):
            verify_ready_transcript(transcript)

    def test_requires_a_nonblank_native_frame(self):
        pixels = bytearray(800 * 600 * 3)
        for index in range(0, len(pixels), 3):
            color = (index // 3) % 16
            pixels[index:index + 3] = bytes((color, color * 2, color * 3))
        frame = b"P6\n800 600\n255\n" + pixels
        self.assertEqual(verify_frame(frame)["distinct_colors"], 16)
        blank = b"P6\n800 600\n255\n" + bytes(800 * 600 * 3)
        with self.assertRaises(ValueError):
            verify_frame(blank)


def reference_frames():
    """Synthetic host pixels only; never native execution evidence."""
    pixels = bytearray(800 * 600 * 3)
    x, y, _, _ = preview.probe.ASSIGNED
    for index in range(12):
        at = ((y + 5) * 800 + x + 5 + index) * 3
        pixels[at:at + 3] = bytes((10 + index, 20 + index, 30 + index))
    frames = {}
    for index, name in enumerate(preview.probe.PHASES["normal"], 1):
        at = ((y + 10) * 800 + x + 10) * 3
        pixels[at:at + 3] = bytes((index * 20, 30, 40))
        frames[name] = b"P6\n800 600\n255\n" + pixels
    return frames


class InteractiveBindingTests(unittest.TestCase):
    def setUp(self):
        self.frozen = "a" * 64
        self.machine = "pc-q35-8.2"
        self.apps = [
            {"name": name, "sha256": hashlib.sha256(name.encode()).hexdigest(), "elf": {}}
            for name in preview.APP_NAMES
        ]
        self.manifest = {
            "source_sha256": self.frozen,
            "kernel_feature": "native-system-materiaal",
            "loader": {"sha256": "b" * 64},
            "materiaal": {"apps": copy.deepcopy(self.apps)},
        }
        self.reference_manifest = copy.deepcopy(self.manifest)
        self.reference_manifest["kernel_feature"] = "native-system-materiaal-probe"
        self.reference_manifest["materiaal"]["apps"].append(
            {"name": "desktop-controller", "sha256": "c" * 64, "elf": {}}
        )
        self.report = {
            "passed": True, "case": "normal", "machine": self.machine,
            "source_fingerprint_version": 3, "source_sha256": self.frozen,
        }

    def bind(self, report=None, reference_manifest=None):
        preview.reference_binding(
            self.report if report is None else report,
            self.reference_manifest if reference_manifest is None else reference_manifest,
            self.manifest, self.frozen, self.machine,
        )

    def test_requires_matching_source_normal_profile_and_all_seven_apps(self):
        self.bind()
        for changed in (
            {"source_sha256": "d" * 64}, {"passed": False}, {"case": "recover"},
            {"source_fingerprint_version": True}, {"machine": "pc-q35-10.0"},
        ):
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                self.bind(report={**self.report, **changed})
        bad = copy.deepcopy(self.reference_manifest)
        bad["source_sha256"] = "e" * 64
        with self.assertRaises(ValueError):
            self.bind(reference_manifest=bad)

    def test_rejects_changed_or_reordered_app_hash_and_loader(self):
        for index in range(len(self.apps)):
            bad = copy.deepcopy(self.reference_manifest)
            bad["materiaal"]["apps"][index]["sha256"] = "f" * 64
            with self.subTest(index=index), self.assertRaises(ValueError):
                self.bind(reference_manifest=bad)
        bad = copy.deepcopy(self.reference_manifest)
        bad["materiaal"]["apps"][0:2] = reversed(bad["materiaal"]["apps"][0:2])
        with self.assertRaises(ValueError):
            self.bind(reference_manifest=bad)
        bad = copy.deepcopy(self.reference_manifest)
        bad["loader"]["sha256"] = "f" * 64
        with self.assertRaises(ValueError):
            self.bind(reference_manifest=bad)

    def test_reference_loader_replays_native_proof_and_rejects_missing_phase(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            (root / "inputs").mkdir()
            firmware = {
                "ovmf_code_sha256": "1" * 64,
                "ovmf_vars_template_sha256": "2" * 64,
                "seabios_vga_sha256": "3" * 64,
            }
            report = {
                **self.report, "qemu_entry_sha256": "4" * 64,
                "qemu_implementation_sha256": "5" * 64,
                "frozen_artifacts": {
                    "OVMF_CODE.fd": "1" * 64, "OVMF_VARS.template.fd": "2" * 64,
                    "VGA.rom": "3" * 64,
                },
            }
            (root / "result.json").write_text(json.dumps(report))
            (root / "inputs/manifest.json").write_text(json.dumps(self.reference_manifest))
            frames = reference_frames()
            with (
                patch.object(preview, "private_path", side_effect=lambda path, **kwargs: path),
                patch.object(preview.probe, "replay") as replay,
                patch.object(preview.probe, "frame_bytes", side_effect=lambda directory, phase: frames[phase]),
            ):
                reference = preview.load_reference(
                    root, self.manifest, self.frozen, self.machine, firmware, "4" * 64
                )
                replay.assert_called_once_with(report, root, self.frozen)
                self.assertEqual(list(reference["frames"]), list(preview.probe.PHASES["normal"]))
                replay.side_effect = ValueError("Native artifact replay failed")
                with self.assertRaisesRegex(ValueError, "replay failed"):
                    preview.load_reference(
                        root, self.manifest, self.frozen, self.machine, firmware, "4" * 64
                    )
                replay.side_effect = None
                frames.pop("saved")
                with self.assertRaises(KeyError):
                    preview.load_reference(
                        root, self.manifest, self.frozen, self.machine, firmware, "4" * 64
                    )


class InteractiveInputTests(unittest.TestCase):
    def test_uses_exact_six_groups_without_f12(self):
        groups = preview.interactive_groups(Path("/owned/run"))
        self.assertEqual(tuple(name for name, _ in groups), preview.probe.PHASES["normal"])
        requests = [request for _, group in groups for request in group]
        self.assertTrue(any(request["execute"] == "input-send-event" for request in requests))
        keys = [key["data"] for request in requests if request["execute"] == "send-key"
                for key in request["arguments"]["keys"]]
        self.assertNotIn("f12", keys)
        self.assertIn("s", keys)
        self.assertIn("esc", keys)
        self.assertIn("ret", keys)

    def test_rejects_missing_phase_or_changed_diagnostic_close_group(self):
        groups = preview.probe.qmp_groups(Path("/owned/run"), "normal")
        early_close = copy.deepcopy(groups)
        early_close[2][1].insert(0, copy.deepcopy(groups[-1][1][0]))
        for changed in (groups[:-2] + groups[-1:], groups[:-1] + [("close", [])], early_close):
            with patch.object(preview.probe, "qmp_groups", return_value=changed):
                with self.assertRaises(ValueError):
                    preview.interactive_groups(Path("/owned/run"))

    def test_exact_pixel_comparison_has_no_cursor_caret_or_chrome_mask(self):
        frame = reference_frames()["ready"]
        self.assertTrue(preview.matching_reference_frame(frame, frame))
        header = len(b"P6\n800 600\n255\n")
        for x, y in ((0, 0), (30, 200), (799, 599)):
            changed = bytearray(frame)
            changed[header + (y * 800 + x) * 3] ^= 1
            self.assertFalse(preview.matching_reference_frame(bytes(changed), frame))
        with self.assertRaises(ValueError):
            preview.matching_reference_frame(b"P6\n1 1\n255\nabc", frame)

    def test_drives_input_only_after_matching_ready_and_records_each_phase(self):
        frames = reference_frames()
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            (directory / "serial.txt").write_bytes("\n".join((*BOOT, *READY)).encode())
            process = Mock()
            process.poll.return_value = None
            control = Mock()
            control.trace = []

            def command(execute, arguments):
                control.trace.append({"request": {"execute": execute, "arguments": arguments}})

            control.command.side_effect = command
            with (
                patch.object(preview.probe, "frame_bytes", side_effect=lambda path, phase: frames[phase]),
                patch.object(preview.time, "sleep"),
            ):
                result = preview.run_interactive(
                    control, process, directory, {"frames": frames}, time.monotonic() + 10
                )
            self.assertEqual([stage["phase"] for stage in result["stages"]],
                             list(preview.probe.PHASES["normal"]))
            self.assertEqual(result["pixel_masks"], [])
            self.assertEqual(control.trace[0]["request"]["execute"], "screendump")
            self.assertNotIn("f12", json.dumps(control.trace))
            self.assertNotIn("quit", json.dumps(control.trace))

    def test_missing_reference_stage_is_rejected_before_hardware_input(self):
        frames = reference_frames()
        frames.pop("saved")
        control = Mock()
        with self.assertRaises(ValueError):
            preview.run_interactive(
                control, Mock(), Path("/owned/run"), {"frames": frames}, time.monotonic() + 10
            )
        control.command.assert_not_called()

    def test_expired_deadline_does_not_send_hardware_input(self):
        control = Mock()
        process = Mock()
        process.poll.return_value = None
        with self.assertRaises(TimeoutError):
            preview.run_interactive(
                control, process, Path("/owned/run"),
                {"frames": reference_frames()}, time.monotonic() - 1,
            )
        control.command.assert_not_called()

    def test_unmatched_ready_pixels_never_allow_mouse_or_text_fallback(self):
        frames = reference_frames()
        changed = bytearray(frames["ready"])
        changed[-1] ^= 1
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            (directory / "serial.txt").write_bytes("\n".join((*BOOT, *READY)).encode())
            process = Mock()
            process.poll.return_value = None
            control = Mock()
            control.trace = []
            with (
                patch.object(preview.probe, "frame_bytes", return_value=bytes(changed)),
                patch.object(preview.time, "monotonic", side_effect=[0, 0, 2]),
                patch.object(preview.time, "sleep"),
            ):
                with self.assertRaisesRegex(TimeoutError, "ready"):
                    preview.run_interactive(
                        control, process, directory, {"frames": frames}, 1
                    )
            control.command.assert_called_once_with(
                "screendump", {"filename": str(directory / "ready.ppm")}
            )

    def test_live_or_nonzero_qemu_never_reaches_disk_oracle(self):
        for raw_exit in (None, 33, -9, False):
            process = Mock()
            process.poll.return_value = raw_exit
            with patch.object(preview.probe, "disk_oracle") as oracle:
                with self.subTest(raw_exit=raw_exit), self.assertRaises(ValueError):
                    preview.inspect_stopped_disk(process, Path("/owned/run"))
                oracle.assert_not_called()

    def test_exited_qemu_with_unsaved_empty_disk_is_not_accepted(self):
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            preview.volume_disk.create_image(directory / "volume.img", "empty")
            (directory / "volume.before.img").write_bytes(
                (directory / "volume.img").read_bytes()
            )
            process = Mock()
            process.poll.return_value = 0
            with self.assertRaises(ValueError):
                preview.inspect_stopped_disk(process, directory)
            self.assertFalse((directory / "volume.after.img").exists())


class PreviewLifecycleTests(unittest.TestCase):
    """Mock host orchestration only, including the post-QMP exit boundary."""

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.build = self.root / "build"
        (self.build / "esp/EFI/BOOT").mkdir(parents=True)
        kernel = self.build / "esp/kernel.elf"
        loader = self.build / "esp/EFI/BOOT/BOOTX64.EFI"
        kernel.write_bytes(b"HOST_ONLY_KERNEL")
        loader.write_bytes(b"HOST_ONLY_LOADER")
        self.files = {}
        for name in ("code", "vars", "bios", "vga", "qemu"):
            self.files[name] = self.root / name
            self.files[name].write_bytes(name.encode())
        self.frames = reference_frames()
        self.manifest = {
            "source_sha256": "a" * 64, "kernel_feature": "native-system-materiaal",
            "kernel": {"sha256": preview.sha256(kernel)},
            "loader": {"sha256": preview.sha256(loader)},
            "materiaal": {"apps": []},
        }
        self.reference = {
            "directory": self.root / "reference", "report_sha256": "b" * 64,
            "source_sha256": "a" * 64, "apps": [], "frames": self.frames,
            "qemu_implementation_sha256": preview.sha256(self.files["qemu"]),
        }
        self.process = Mock()
        self.process.returncode = None
        self.process.poll.side_effect = lambda: self.process.returncode
        self.exit_code = 0
        self.control = Mock()
        self.control.trace = []
        self.control.command.side_effect = self.command
        self.handshake = False
        self.quit = False
        self.process.wait.side_effect = self.wait
        stack = ExitStack()
        self.addCleanup(stack.close)
        for target, name, value in (
            (run_fpu, "ROOT", self.root),
            (preview, "validate_build", Mock(return_value=(self.build, self.manifest))),
            (preview, "source_hash", Mock(return_value="a" * 64)),
            (preview, "environment", Mock(return_value={"PATH": ""})),
            (preview.shutil, "which", Mock(return_value=str(self.files["qemu"]))),
            (preview, "firmware", Mock(return_value=(self.files["code"], self.files["vars"]))),
            (preview.subprocess, "check_output", Mock(return_value="HOST_ONLY")),
            (preview.subprocess, "Popen", Mock(side_effect=self.start)),
            (preview, "QmpPipe", Mock(side_effect=self.connect)),
            (preview.os, "readlink", Mock(side_effect=self.implementation)),
            (preview, "load_reference", Mock(return_value=self.reference)),
            (preview.time, "sleep", Mock()),
        ):
            stack.enter_context(patch.object(target, name, value))
        stack.enter_context(patch("sys.stdout", new=io.StringIO()))

    def start(self, command, **kwargs):
        (kwargs["cwd"] / "serial.txt").write_bytes("\n".join((*BOOT, *READY)).encode())
        return self.process

    def connect(self, process, deadline):
        self.handshake = True
        return self.control

    def implementation(self, path):
        self.assertTrue(self.handshake, "Wrapper must finish exec/QMP before /proc identity check")
        return str(self.files["qemu"])

    def command(self, execute, arguments=None):
        self.control.trace.append({"request": {"execute": execute, "arguments": arguments}})
        if execute == "screendump":
            path = Path(arguments["filename"])
            path.write_bytes(self.frames.get(path.stem, self.frames["ready"]))
        if execute == "quit":
            self.quit = True

    def wait(self, **kwargs):
        self.assertTrue(self.quit)
        self.process.returncode = self.exit_code
        return self.exit_code

    def run_preview(self, interactive=False):
        return preview.run_preview(
            build_directory=self.build, bios_rom=self.files["bios"], vga_rom=self.files["vga"],
            machine="pc-q35-8.2",
            interactive_reference_run=self.root / "reference" if interactive else None,
        )

    def test_default_preview_does_not_load_reference_or_decode_interactive_disk(self):
        with patch.object(preview.probe, "disk_oracle") as oracle:
            result = self.run_preview()
        self.assertTrue(result["passed"])
        self.assertEqual(result["raw_qemu_exit"], 0)
        self.assertNotIn("interactive", result)
        preview.load_reference.assert_not_called()
        oracle.assert_not_called()
        self.assertEqual([item["request"]["execute"] for item in self.control.trace],
                         ["screendump", "quit"])

    def test_interactive_disk_decoder_runs_after_qmp_quit_and_raw_zero(self):
        def oracle(raw):
            self.assertTrue(self.quit)
            self.assertEqual(self.process.poll(), 0)
            return {"HOST_ONLY_MOCK": True}

        with patch.object(preview.probe, "disk_oracle", side_effect=oracle) as decode:
            result = self.run_preview(interactive=True)
        self.assertTrue(result["passed"])
        decode.assert_called_once()
        self.assertEqual(preview.load_reference.call_count, 2)
        self.assertEqual([stage["phase"] for stage in result["interactive"]["stages"]],
                         list(preview.probe.PHASES["normal"]))
        self.assertEqual(self.control.trace[-1]["request"]["execute"], "quit")
        self.assertNotIn("f12", sent_qcodes(self.control.trace))

    def test_nonzero_exit_cannot_pass_even_with_matching_frames(self):
        self.exit_code = 33
        with patch.object(preview.probe, "disk_oracle") as oracle:
            with self.assertRaises(RuntimeError):
                self.run_preview(interactive=True)
        oracle.assert_not_called()
        report = json.loads((self.build / "preview-result.json").read_text())
        self.assertFalse(report["passed"])
        self.assertEqual(report["raw_qemu_exit"], 33)

    def test_invalid_reference_is_recorded_without_starting_vm(self):
        preview.load_reference.side_effect = ValueError("Invalid reference")
        with self.assertRaisesRegex(ValueError, "Invalid reference"):
            self.run_preview(interactive=True)
        preview.subprocess.Popen.assert_not_called()
        report = json.loads((self.build / "preview-result.json").read_text())
        self.assertFalse(report["passed"])
        self.assertIn("Invalid reference", report["error"])

    def test_two_walkthroughs_are_rejected_before_vm_start(self):
        with self.assertRaisesRegex(ValueError, "one native walkthrough"):
            preview.run_preview(
                build_directory=self.build, bios_rom=self.files["bios"], vga_rom=self.files["vga"],
                interactive_reference_run=self.root / "reference",
                session_fixture_directory=self.root / "session",
            )
        preview.subprocess.Popen.assert_not_called()

    def test_session_hook_revalidates_host_oracle_after_native_exit(self):
        fixture = {
            "directory": self.root / "session", "metadata": {"kind": session_preview.ORACLE_KIND},
            "metadata_sha256": "c" * 64, "frames": {"host-only": b"host-oracle"},
        }

        def walkthrough(control, process, directory, reference, deadline):
            self.assertTrue(self.handshake)
            self.assertIsNone(process.poll())
            (directory / "session-material.ppm").write_bytes(self.frames["ready"])
            return {"stages": ["HOST_ONLY_HOOK_FIXTURE"]}

        with (
            patch.object(session_preview, "load_fixture", return_value=fixture) as load,
            patch.object(session_preview, "run_walkthrough", side_effect=walkthrough) as walk,
            patch.object(preview.probe, "disk_oracle") as disk,
        ):
            report = preview.run_preview(
                build_directory=self.build, bios_rom=self.files["bios"], vga_rom=self.files["vga"],
                session_fixture_directory=self.root / "session", timeout=240,
            )
        self.assertTrue(report["passed"])
        self.assertEqual(load.call_count, 2)
        walk.assert_called_once()
        preview.load_reference.assert_not_called()
        disk.assert_not_called()
        self.assertIn("host-rendered", report["scope"])
        self.assertEqual(report["session_walkthrough"]["stages"], ["HOST_ONLY_HOOK_FIXTURE"])


def session_frames():
    """Synthetic host-test pixels only; no native-execution claim."""
    base = reference_frames()["ready"]
    header = len(b"P6\n800 600\n255\n")
    full, cropped = {}, {}
    for index, name in enumerate(session_preview.PHASES):
        raw = bytearray(base)
        raw[header + ((48 + 10) * 800 + 16 + 10) * 3] = (
            0 if name == "accounts-return" else index
        )
        full[name] = bytes(raw)
        cropped[name] = b"P6\n768 536\n255\n" + session_preview.app_pixels(raw)
    return cropped, full


class SessionWalkthroughTests(unittest.TestCase):
    def test_compares_every_app_pixel_and_explicitly_excludes_only_external_chrome(self):
        cropped, full = session_frames()
        raw, expected = full["accounts"], cropped["accounts"]
        self.assertTrue(session_preview.matching_app_frame(raw, expected))
        header = len(b"P6\n800 600\n255\n")
        for x, y in ((16, 48), (783, 583), (160, 250)):
            changed = bytearray(raw)
            changed[header + (y * 800 + x) * 3] ^= 1
            self.assertFalse(session_preview.matching_app_frame(bytes(changed), expected))
        changed = bytearray(raw)
        changed[header] ^= 1
        self.assertTrue(session_preview.matching_app_frame(bytes(changed), expected))
        with self.assertRaises(ValueError):
            session_preview.matching_app_frame(b"P6\n1 1\n255\nabc", expected)
        with self.assertRaises(ValueError):
            session_preview.matching_app_frame(raw, b"P6\n1 1\n255\nabc")

    def test_exact_keyboard_route_has_no_diagnostic_key_or_guest_marker(self):
        groups = session_preview.input_groups()
        self.assertEqual(tuple(name for name, _ in groups), session_preview.PHASES)
        keys = [key["data"] for _, requests in groups for request in requests
                for key in request["arguments"]["keys"]]
        self.assertEqual(keys, [
            "tab", "ret", "t", "e", "s", "t", "ret", "esc", "ret", "ret",
            "tab", "tab", "ret", "m", "a", "t", "ret",
        ])
        self.assertNotIn("f12", keys)
        self.assertEqual(groups[0][1], [])

    def test_missing_or_wrong_geometry_oracle_never_sends_input(self):
        frames, _ = session_frames()
        for bad in ({}, {**frames, "search": b"P6\n1 1\n255\nabc"}):
            control = Mock()
            with self.assertRaises(ValueError):
                session_preview.run_walkthrough(
                    control, Mock(), Path("/owned/run"), {"frames": bad}, time.monotonic() + 5,
                )
            control.command.assert_not_called()

    def test_all_eight_frames_must_match_before_advancing(self):
        cropped, full = session_frames()
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            (directory / "serial.txt").write_bytes("\n".join((*BOOT, *READY)).encode())
            process, control = Mock(), Mock()
            process.poll.return_value = None
            control.trace = []

            def command(execute, arguments):
                control.trace.append({"execute": execute, "arguments": arguments})
                if execute == "screendump":
                    path = Path(arguments["filename"])
                    path.write_bytes(full[path.stem.removeprefix("session-")])

            control.command.side_effect = command
            with (
                patch.object(session_preview, "private_path", side_effect=lambda path, **kwargs: path),
                patch.object(session_preview.time, "sleep"),
            ):
                report = session_preview.run_walkthrough(
                    control, process, directory, {"frames": cropped}, time.monotonic() + 10,
                )
            self.assertEqual([stage["phase"] for stage in report["stages"]], list(session_preview.PHASES))
            self.assertEqual(report["pixel_masks_inside_app"], [])
            self.assertFalse(report["host_oracle_is_native_evidence"])
            self.assertEqual(control.trace[0]["execute"], "screendump")

    def test_unmatched_accounts_frame_blocks_all_hardware_input(self):
        cropped, full = session_frames()
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            (directory / "serial.txt").write_bytes("\n".join((*BOOT, *READY)).encode())
            process, control = Mock(), Mock()
            process.poll.return_value = None
            control.trace = []

            def command(execute, arguments):
                self.assertEqual(execute, "screendump")
                Path(arguments["filename"]).write_bytes(full["personal-login"])

            control.command.side_effect = command
            with (
                patch.object(session_preview, "private_path", side_effect=lambda path, **kwargs: path),
                patch.object(session_preview.time, "monotonic", side_effect=[0, 0, 0, 2]),
                patch.object(session_preview.time, "sleep"),
            ):
                with self.assertRaisesRegex(TimeoutError, "accounts"):
                    session_preview.run_walkthrough(control, process, directory, {"frames": cropped}, 1)
            control.command.assert_called_once()

    def test_source_profile_hash_and_changed_oracle_are_rejected(self):
        cropped, _ = session_frames()
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            frozen = "a" * 64
            manifest = {"source_sha256": frozen, "kernel_feature": session_preview.FEATURE}
            metadata = {
                "schema_version": 1, "kind": session_preview.ORACLE_KIND,
                "kernel_feature": session_preview.FEATURE,
                "source_fingerprint_version": 3, "source_sha256": frozen,
                "app_rectangle": list(session_preview.APP_RECT), "phases": list(session_preview.PHASES),
                "generator_exit": 0,
                "frames_sha256": {phase: hashlib.sha256(raw).hexdigest() for phase, raw in cropped.items()},
            }
            for phase, raw in cropped.items():
                (directory / f"{phase}.ppm").write_bytes(raw)
            path = directory / "fixture.json"
            path.write_text(json.dumps(metadata))
            with patch.object(session_preview, "private_path", side_effect=lambda path, **kwargs: path):
                self.assertEqual(len(session_preview.load_fixture(directory, manifest, frozen)["frames"]), 8)
                for changed in (
                    {"source_sha256": "b" * 64}, {"schema_version": True},
                    {"generator_exit": False}, {"kind": "native proof"},
                    {"kernel_feature": "native-system-materiaal"}, {"frames_sha256": []},
                ):
                    path.write_text(json.dumps({**metadata, **changed}))
                    with self.subTest(changed=changed), self.assertRaises(ValueError):
                        session_preview.load_fixture(directory, manifest, frozen)
                path.write_text(json.dumps(metadata))
                (directory / "accounts.ppm").write_bytes(cropped["personal-login"])
                with self.assertRaisesRegex(ValueError, "changed"):
                    session_preview.load_fixture(directory, manifest, frozen)

    def test_nondiscriminating_rejection_or_account_return_is_rejected(self):
        frames, _ = session_frames()
        session_preview.validate_discriminating_frames(frames)
        for changed in (
            {"denied": frames["personal-login"]},
            {"denied": frames["desktop"]},
            {"accounts-return": frames["personal-login"]},
        ):
            with self.subTest(changed=tuple(changed)), self.assertRaises(ValueError):
                session_preview.validate_discriminating_frames({**frames, **changed})


if __name__ == "__main__":
    unittest.main()
