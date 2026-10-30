import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

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
            trace, stages = parent_trace(root)
            for stage in stages:
                stage["screenshot_sha256"] = hashlib.sha256(frames[stage["phase"]]).hexdigest()
            report = {
                "passed": True, "raw_qemu_exit": 0, "source_sha256": "c" * 64,
                "source_fingerprint_version": 3, "kernel_feature": "native-system-materiaal",
                "kernel_sha256": manifest["kernel"]["sha256"],
                "loader_sha256": manifest["loader"]["sha256"], "machine": "pc-q35-10.0",
                "firmware": {}, "qemu_sha256": "d" * 64,
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
                        root, manifest, reference, "c" * 64, "pc-q35-10.0", {}, "d" * 64
                    )

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


if __name__ == "__main__":
    unittest.main()
