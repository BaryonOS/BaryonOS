import copy
from contextlib import ExitStack
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import materiaal_cold as cold
import run_materiaal_cold as runner
import run_materiaal_preview as preview
import run_fpu
from tools.tests.test_materiaal_preview import reference_frames


def parent_trace(directory):
    trace = []
    stages = []

    def add(request):
        index = len(trace) + 1
        trace.append({"request": {**request, "id": index},
                      "response": {"return": {}, "id": index}})

    add({"execute": "qmp_capabilities"})
    for phase, group in preview.interactive_groups(directory):
        for request in group:
            add(request)
            if request["execute"] == "screendump":
                add(request)
        stages.append({"phase": phase, "qmp_index": len(trace)})
    add({"execute": "quit"})
    return trace, stages


class ColdTraceTests(unittest.TestCase):
    def test_exact_input_with_repeated_observations_is_accepted(self):
        directory = Path("/owned/parent")
        trace, stages = parent_trace(directory)
        cold.replay_input_trace(trace, stages, directory)

    def test_rejects_altered_input_failed_response_and_missing_quit(self):
        directory = Path("/owned/parent")
        trace, stages = parent_trace(directory)
        bad_input = copy.deepcopy(trace)
        bad_input[3]["request"]["arguments"]["events"][0]["data"]["value"] += 1
        bad_response = copy.deepcopy(trace)
        bad_response[1]["response"]["id"] = True
        extra_input = copy.deepcopy(trace)
        extra_input[-1]["request"]["execute"] = "send-key"
        for bad in (bad_input, bad_response, extra_input, trace[:-1]):
            with self.subTest(bad=bad[-1]), self.assertRaises(ValueError):
                cold.replay_input_trace(bad, stages, directory)

    def test_rejects_stale_stage_and_early_quit(self):
        directory = Path("/owned/parent")
        trace, stages = parent_trace(directory)
        bad_stages = copy.deepcopy(stages)
        bad_stages[0]["qmp_index"] -= 1
        with self.assertRaises(ValueError):
            cold.replay_input_trace(trace, bad_stages, directory)
        with self.assertRaises(ValueError):
            cold.replay_input_trace(trace[:1] + trace[-1:], stages, directory)

    def test_full_frame_composition_does_not_ignore_any_actual_pixel(self):
        frames = reference_frames()
        header = len(b"P6\n800 600\n255\n")
        ready = bytearray(frames["ready"])
        listed = bytearray(frames["list"])
        status_at = header + (540 * 800 + 42) * 3
        outside = header + (100 * 800 + 42) * 3
        ready[status_at] = 123
        listed[status_at] = 124
        listed[outside] = 125
        frames.update(ready=bytes(ready), list=bytes(listed))
        composed = cold.cold_list_frame(frames)
        self.assertEqual(composed[status_at], 123)
        self.assertEqual(composed[outside], 125)
        for offset in (0 + header, status_at, outside, len(composed) - 1):
            changed = bytearray(composed)
            changed[offset] ^= 1
            self.assertFalse(preview.matching_reference_frame(changed, composed))


class ColdParentTests(unittest.TestCase):
    def test_complete_replay_and_refusal_of_source_disk_and_frame_tampering(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            (root / "esp/EFI/BOOT").mkdir(parents=True)
            kernel, loader = b"kernel", b"loader"
            (root / "esp/kernel.elf").write_bytes(kernel)
            (root / "esp/EFI/BOOT/BOOTX64.EFI").write_bytes(loader)
            serial = "\n".join((*preview.BOOT, *preview.READY)).encode()
            (root / "serial.txt").write_bytes(serial)
            frames = reference_frames()
            for phase, frame in frames.items():
                (root / (phase + ".ppm")).write_bytes(frame)
            preview.volume_disk.create_image(root / "volume.before.img", "empty")
            before = (root / "volume.before.img").read_bytes()
            after = b"independently-decoded-fixture"
            for file in ("volume.img", "volume.after.img"):
                (root / file).write_bytes(after)
            manifest = {
                "kernel": {"sha256": hashlib.sha256(kernel).hexdigest()},
                "loader": {"sha256": hashlib.sha256(loader).hexdigest()},
            }
            reference = {
                "report_sha256": "a" * 64, "apps": [], "frames": frames,
                "qemu_implementation_sha256": "b" * 64,
            }
            firmware = {}
            for file, field in (
                ("OVMF_CODE.fd", "ovmf_code_sha256"),
                ("OVMF_VARS.template.fd", "ovmf_vars_template_sha256"),
                ("bios-256k.bin", "seabios_bios_sha256"),
                ("VGA.rom", "seabios_vga_sha256"),
            ):
                (root / file).write_bytes(file.encode())
                firmware[field] = preview.sha256(root / file)
            (root / "OVMF_VARS.fd").write_bytes(b"mutable guest firmware state")
            trace, stages = parent_trace(root)
            for stage in stages:
                stage["screenshot_sha256"] = hashlib.sha256(frames[stage["phase"]]).hexdigest()
            report = {
                "passed": True, "raw_qemu_exit": 0, "source_sha256": "c" * 64,
                "source_fingerprint_version": 3, "kernel_feature": "native-system-materiaal",
                "kernel_sha256": manifest["kernel"]["sha256"],
                "loader_sha256": manifest["loader"]["sha256"], "machine": "pc-q35-10.0",
                "firmware": firmware, "qemu_sha256": "d" * 64,
                "qemu_implementation_sha256": "b" * 64,
                "interactive_reference": {
                    "report_sha256": "a" * 64, "apps": [], "source_sha256": "c" * 64,
                },
                "interactive": {
                    "pixel_masks": [], "stages": stages, "disk": {"decoded": True},
                    "before_sha256": hashlib.sha256(before).hexdigest(),
                    "after_sha256": hashlib.sha256(after).hexdigest(),
                },
                "qmp_trace": trace, "serial_sha256": hashlib.sha256(serial).hexdigest(),
            }
            path = root / "result.json"
            path.write_text(json.dumps(report))
            with (
                patch.object(run_fpu, "ROOT", root),
                patch.object(preview.probe, "disk_oracle", return_value={"decoded": True}),
            ):
                def replay():
                    return cold.replay_parent(
                        root, manifest, reference, "c" * 64, "pc-q35-10.0", firmware, "d" * 64
                    )

                self.assertEqual(replay()["disk"], after)
                for file in ("OVMF_CODE.fd", "OVMF_VARS.template.fd", "bios-256k.bin", "VGA.rom"):
                    original = (root / file).read_bytes()
                    (root / file).write_bytes(original + b"tampered")
                    with self.subTest(firmware=file), self.assertRaisesRegex(ValueError, "firmware"):
                        replay()
                    (root / file).write_bytes(original)
                (root / "OVMF_VARS.fd").write_bytes(b"changed mutable state")
                self.assertEqual(replay()["disk"], after)
                for changes in (
                    {"source_sha256": "e" * 64}, {"raw_qemu_exit": False},
                    {"kernel_sha256": "f" * 64}, {"machine": "pc-q35-8.2"},
                ):
                    path.write_text(json.dumps({**report, **changes}))
                    with self.subTest(changes=changes), self.assertRaises(ValueError):
                        replay()
                path.write_text(json.dumps(report))
                (root / "volume.img").write_bytes(after + b"x")
                with self.assertRaisesRegex(ValueError, "disk"):
                    replay()
                (root / "volume.img").write_bytes(after)
                (root / "ready.ppm").write_bytes(frames["editor"])
                with self.assertRaisesRegex(ValueError, "frame"):
                    replay()

    def test_preparation_failure_removes_old_cold_success_without_vm(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            report = root / "cold-result.json"
            report.write_text('{"passed":true}')
            with (
                patch.object(run_fpu, "ROOT", root),
                patch.object(preview, "validate_build", side_effect=ValueError("Invalid build")),
                patch.object(runner.subprocess, "Popen") as start,
            ):
                with self.assertRaisesRegex(ValueError, "Invalid build"):
                    runner.run_cold(
                        build_directory=root, parent_run=root, reference_run=root,
                        bios_rom=root / "bios", vga_rom=root / "vga", machine="pc-q35-10.0",
                    )
                self.assertFalse(report.exists())
                start.assert_not_called()


class ColdLifecycleTests(unittest.TestCase):
    """Host mocks only; never native/cold execution evidence."""

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.build = self.root / "build"
        (self.build / "esp/EFI/BOOT").mkdir(parents=True)
        (self.build / "esp/kernel.elf").write_bytes(b"kernel")
        (self.build / "esp/EFI/BOOT/BOOTX64.EFI").write_bytes(b"loader")
        self.files = {}
        for name in ("code", "vars", "bios", "vga", "qemu"):
            self.files[name] = self.root / name
            self.files[name].write_bytes(name.encode())
        self.manifest = {
            "source_sha256": "a" * 64,
            "kernel": {"sha256": preview.sha256(self.build / "esp/kernel.elf")},
            "loader": {"sha256": preview.sha256(self.build / "esp/EFI/BOOT/BOOTX64.EFI")},
        }
        self.frames = reference_frames()
        self.expected = {"list": cold.cold_list_frame(self.frames),
                         "reopened": self.frames["reopened"]}
        self.reference = {
            "frames": self.frames, "report_sha256": "b" * 64,
            "apps": [],
            "qemu_implementation_sha256": preview.sha256(self.files["qemu"]),
        }
        self.parent = {
            "directory": self.root / "parent", "report_sha256": "c" * 64,
            "disk": b"HOST_ONLY_MOCK_DISK",
        }
        self.process = Mock()
        self.process.returncode = None
        self.process.poll.side_effect = lambda: self.process.returncode
        self.process.wait.side_effect = self.wait
        self.process.terminate.side_effect = lambda: setattr(self.process, "returncode", -15)
        self.exit_code = 0
        self.quit = False
        self.control = Mock()
        self.control.trace = []
        self.control.command.side_effect = self.command
        self.oracle_calls = 0
        self.directory = None
        stack = ExitStack()
        self.addCleanup(stack.close)
        for target, name, value in (
            (run_fpu, "ROOT", self.root),
            (preview, "validate_build", Mock(return_value=(self.build, self.manifest))),
            (preview, "source_hash", Mock(return_value="a" * 64)),
            (preview, "environment", Mock(return_value={"PATH": ""})),
            (runner.shutil, "which", Mock(return_value=str(self.files["qemu"]))),
            (preview, "firmware", Mock(return_value=(self.files["code"], self.files["vars"]))),
            (runner.subprocess, "check_output", Mock(return_value="HOST_ONLY")),
            (runner.subprocess, "Popen", Mock(side_effect=self.start)),
            (preview, "QmpPipe", Mock(return_value=self.control)),
            (runner.os, "readlink", Mock(return_value=str(self.files["qemu"]))),
            (preview, "load_reference", Mock(return_value=self.reference)),
            (cold, "replay_parent", Mock(return_value=self.parent)),
            (preview, "inspect_stopped_disk", Mock(side_effect=self.inspect)),
            (runner.time, "sleep", Mock()),
        ):
            stack.enter_context(patch.object(target, name, value))

    def start(self, command, **kwargs):
        self.directory = kwargs["cwd"]
        (self.directory / "serial.txt").write_bytes("\n".join((*preview.BOOT, *preview.READY)).encode())
        self.assertEqual((self.directory / "OVMF_VARS.fd").read_bytes(), b"vars")
        self.assertEqual((self.directory / "volume.img").read_bytes(), self.parent["disk"])
        return self.process

    def command(self, execute, arguments=None):
        index = len(self.control.trace) + 1
        request = {"execute": execute, "id": index}
        if arguments is not None:
            request["arguments"] = arguments
        self.control.trace.append({"request": request, "response": {"return": {}, "id": index}})
        if execute == "screendump":
            path = Path(arguments["filename"])
            path.write_bytes(self.expected[path.stem])
        elif execute == "quit":
            self.quit = True

    def wait(self, **kwargs):
        if self.process.returncode == -15:
            return -15
        self.assertTrue(self.quit)
        self.process.returncode = self.exit_code
        return self.exit_code

    def inspect(self, process, directory):
        self.assertTrue(self.quit)
        self.assertEqual(process.poll(), 0)
        self.oracle_calls += 1
        (directory / "volume.after.img").write_bytes((directory / "volume.img").read_bytes())
        return {"disk": {"HOST_ONLY": True}, "before_sha256": "d" * 64, "after_sha256": "d" * 64}

    def run_cold(self):
        self.control.trace = [
            {"request": {"execute": "qmp_capabilities", "id": 1},
             "response": {"return": {}, "id": 1}}
        ]
        return runner.run_cold(
            build_directory=self.build, parent_run=self.parent["directory"],
            reference_run=self.root / "reference", bios_rom=self.files["bios"],
            vga_rom=self.files["vga"], machine="pc-q35-10.0",
        )

    def test_exact_cold_frames_and_disk_only_after_quit_zero(self):
        result = self.run_cold()
        self.assertTrue(result["passed"])
        self.assertEqual(self.oracle_calls, 1)
        self.assertEqual([stage["phase"] for stage in result["stages"]], ["list", "reopened"])
        self.assertTrue(result["volume_unchanged"])
        self.assertEqual(cold.replay_parent.call_count, 2)
        self.assertEqual(preview.load_reference.call_count, 2)
        self.process.terminate.assert_not_called()

    def test_invalid_parent_does_not_start_vm(self):
        cold.replay_parent.side_effect = ValueError("Invalid parent")
        with self.assertRaisesRegex(ValueError, "Invalid parent"):
            self.run_cold()
        runner.subprocess.Popen.assert_not_called()
        self.assertFalse(json.loads((self.build / "cold-result.json").read_text())["passed"])

    def test_nonzero_exit_never_inspects_disk(self):
        self.exit_code = 33
        with self.assertRaisesRegex(ValueError, "raw0"):
            self.run_cold()
        self.assertEqual(self.oracle_calls, 0)
        self.assertEqual(json.loads((self.build / "cold-result.json").read_text())["raw_qemu_exit"], 33)

    def check_late_guest_failure(self, marker):
        original_command = self.command

        def late_failure(execute, arguments=None):
            original_command(execute, arguments)
            if execute == "screendump" and Path(arguments["filename"]).stem == "reopened":
                with (self.directory / "serial.txt").open("ab") as serial:
                    serial.write(("\n" + marker + "\n").encode())

        self.control.command.side_effect = late_failure
        with self.assertRaises(ValueError):
            self.run_cold()
        report = json.loads((self.build / "cold-result.json").read_text())
        self.assertFalse(report["passed"])
        self.assertEqual(report["raw_qemu_exit"], 0)
        self.assertEqual(self.oracle_calls, 0)

    def test_late_guest_panic_after_final_frame_cannot_pass(self):
        self.check_late_guest_failure("PANIC:late failure")

    def test_late_guest_fail_after_final_frame_cannot_pass(self):
        self.check_late_guest_failure("MATERIAAL:FAIL:late failure")

    def test_changed_disk_after_quit_is_rejected(self):
        original = self.inspect

        def changed(process, directory):
            result = original(process, directory)
            (directory / "volume.after.img").write_bytes(b"changed")
            return result

        preview.inspect_stopped_disk.side_effect = changed
        with self.assertRaisesRegex(ValueError, "changed writable"):
            self.run_cold()
        self.assertFalse(json.loads((self.build / "cold-result.json").read_text())["passed"])

    def test_changed_source_after_exit_is_rejected(self):
        preview.source_hash.side_effect = ["a" * 64, "e" * 64]
        with self.assertRaisesRegex(ValueError, "source, parent, reference"):
            self.run_cold()
        self.assertFalse(json.loads((self.build / "cold-result.json").read_text())["passed"])

    def test_wrong_emulator_identity_cannot_send_input(self):
        self.reference["qemu_implementation_sha256"] = "e" * 64
        with self.assertRaisesRegex(ValueError, "implementation differs"):
            self.run_cold()
        self.control.command.assert_not_called()
        self.process.terminate.assert_called_once()

    def test_mutated_frozen_executable_is_rejected(self):
        original = self.inspect

        def mutate(process, directory):
            result = original(process, directory)
            (directory / "esp/kernel.elf").write_bytes(b"modified")
            return result

        preview.inspect_stopped_disk.side_effect = mutate
        with self.assertRaisesRegex(ValueError, "frozen executable"):
            self.run_cold()
        self.assertFalse(json.loads((self.build / "cold-result.json").read_text())["passed"])

    def test_unmatched_list_stops_before_reopen_and_terminates_own_vm(self):
        ticks = iter([0, 0, 0, 0, 0, 0, 1000, 1000, 1000, 1000])
        with (
            patch.object(runner.time, "monotonic", side_effect=lambda: next(ticks, 1000)),
            patch.object(preview, "matching_reference_frame", return_value=False),
        ):
            with self.assertRaises(TimeoutError):
                self.run_cold()
        self.assertFalse(self.quit)
        self.assertFalse(any(item["request"]["execute"] == "send-key" for item in self.control.trace))
        self.assertEqual(self.oracle_calls, 0)
        self.process.terminate.assert_called_once()


if __name__ == "__main__":
    unittest.main()
