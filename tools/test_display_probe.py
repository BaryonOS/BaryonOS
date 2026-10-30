#!/usr/bin/env python3
"""Host counterexamples for the evidence gate, never native display/input proof."""
from contextlib import ExitStack
import copy
import json
import os
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch
from unittest.mock import Mock

import run_display_probe as d
import run_fpu as f


def serial():
    lines = list(d.BOOT) + ["DISPINPUT:MANIFEST:PASS"]
    for generation in (1, 2):
        lines += [f"DISPINPUT:DISPLAY_READY:{generation:020d}", f"DISPINPUT:INPUT_READY:{generation:020d}"]
        for index, byte in enumerate(d.INPUT_BYTES):
            status = 0x21 if index >= 2 else 0x01
            lines += [f"DISPINPUT:SEQ:{index + 1:020d}", f"DISPINPUT:RAW:{status * 256 + byte:020d}"]
        if generation == 1:
            lines += ["DISPINPUT:REUSE:PASS"]
    values = dict(BLITS=2, PIXELS=512, DISPLAY_DENIED=22, CAP_DENIED=44, POINTER_DENIED=72,
                  OUTPUT_UNCHANGED=99, RESETS=2, READY=2, SAMPLES=16, RESET_PENDING=8,
                  RESET_DISCARDED=0, CLOSED=4, OLD_DENIED=2, EXITS=6, SLOT_REUSES=3,
                  DISPLAY_EPOCH=2, INPUT_EPOCH=2, INPUT_TICKET=2, TIMER_IRQS=41,
                  LIVE_PROCESSES=0, FRAMES_RECOVERED=1)
    lines += [f"DISPINPUT:{name}:{values[name]:020d}" for name in d.FIELDS] + list(d.END)
    return "\n".join(lines) + "\n"


def frames():
    pixels = bytearray(800 * 600 * 3)
    result = []
    for generation in (1, 2):
        left, top = (64, 64) if generation == 1 else (112, 80)
        for y in range(8):
            for x in range(32):
                at = ((top + y) * 800 + left + x) * 3
                pixels[at:at + 3] = bytes((0x20 + generation * 0x20 + x * 3, 0x30 + y * 0x13, 0x70 + x + y))
        result.append(b"P6\n800 600\n255\n" + pixels)
    return result


def user_elf(index):
    """Minimal structurally valid ELF for host hash/embedding counterexamples."""
    data = bytearray(8192 + 1024)
    data[:16] = b"\x7fELF\x02\x01\x01" + bytes(9)
    struct.pack_into("<HHIQQQIHHH", data, 16, 2, 62, 1, 0x40000000, 64, 0, 0, 64, 56, 2)
    for i, (offset, address, flags, size) in enumerate(((4096, 0x40000000, 5, 8), (8192, 0x40010000, 6, 1024))):
        struct.pack_into("<IIQQQQQQ", data, 64 + i * 56, 1, flags, offset, address, address, size, size, 4096)
    data[4096:4104] = bytes([index + 1]) * 8
    return bytes(data)


class ProbeTests(unittest.TestCase):
    def test_serial_log_is_qemu_owned_for_both_qmp_transports(self):
        for transport, qmp in (("pipe", "stdio"), ("unix", "unix:qmp.sock,server=on,wait=off")):
            with self.subTest(transport=transport):
                args = d.qemu_arguments("/qemu", "pc-q35-8.2", transport)
                self.assertEqual(args[args.index("-serial") + 1], "file:serial.txt")
                self.assertEqual(args[args.index("-qmp") + 1], qmp)

    def test_screendump_uses_cwd_relative_artifacts(self):
        requests = d.qmp_requests(Path("/tmp/display-run"))
        filenames = [request["arguments"]["filename"] for request in requests if request["execute"] == "screendump"]
        self.assertEqual(filenames, ["generation-1.ppm", "generation-2.ppm"])

    def checkout(self, root):
        stack = ExitStack()
        self.addCleanup(stack.close)
        stack.enter_context(patch.object(d, "ROOT", root))
        stack.enter_context(patch.object(f, "ROOT", root))

    def build_fixture(self, root):
        (root / "boot/loader/src").mkdir(parents=True)
        (root / "boot/loader/src/graphics.rs").write_text("host-only loader identity fixture")
        build = root / "build"
        (build / "users/display-probe").mkdir(parents=True)
        (build / "esp/EFI/BOOT").mkdir(parents=True)
        apps = []
        binaries = []
        for index, name in enumerate(d.NAMES):
            path = build / f"users/display-probe/{name}.elf"
            data = user_elf(index)
            path.write_bytes(data)
            binaries.append(data)
            apps.append({"name": name, "sha256": d.sha256(path), "elf": d.inspect_user(path)})
        artifact = {"passed": True, "native_runtime_performed": False, "apps": apps}
        (build / "esp/kernel.elf").write_bytes(b"kernel" + b"".join(binaries))
        (build / "esp/EFI/BOOT/BOOTX64.EFI").write_bytes(b"loader")
        manifest = {"kernel_feature": "native-display-probe", "loader_feature": "framebuffer",
                    "source_fingerprint_version": 3, "source_sha256": "a" * 64,
                    "sdk_runtime": False, "document_session": False, "display_probe": artifact,
                    "kernel": {"sha256": d.sha256(build / "esp/kernel.elf")},
                    "loader": {"sha256": d.sha256(build / "esp/EFI/BOOT/BOOTX64.EFI")}}
        (build / "manifest.json").write_text(json.dumps(manifest))
        (build / "users/display-probe/build-report.json").write_text(json.dumps(artifact))
        return build, d.validate_build(build, "a" * 64)

    def execution_fixture(self, build, bound):
        directory = build / "run/actual"
        directory.mkdir(parents=True)
        frozen = {}
        for name in bound["expected"]:
            path = directory / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes((build / name).read_bytes())
            frozen[name] = d.sha256(path)
        for name in ("OVMF_CODE.fd", "OVMF_VARS.template.fd", "VGA.rom"):
            (directory / name).write_bytes(name.encode())
            frozen[name] = d.sha256(directory / name)
        (directory / "inputs").mkdir()
        for name in d.NAMES:
            path = directory / f"inputs/{name}.elf"
            path.write_bytes((build / f"users/display-probe/{name}.elf").read_bytes())
            frozen[f"inputs/{name}.elf"] = d.sha256(path)
        for name, value in (("build-manifest.json", bound["manifest"]), ("build-report.json", bound["artifact"])):
            path = directory / "inputs" / name
            path.write_text(json.dumps(value))
            frozen["inputs/" + name] = d.sha256(path)
        (directory / "serial.txt").write_text(serial())
        trace = [{"request": request, "response": {"return": {}, "id": request["id"]}}
                 for request in d.qmp_requests(directory)]
        (directory / "qmp-trace.json").write_text(json.dumps(trace))
        for generation, raw in enumerate(frames(), 1):
            (directory / f"generation-{generation}.ppm").write_bytes(raw)
        report = {"raw_qemu_exit": 33, "timed_out": False, "stdout": serial(), "cwd": str(directory),
                  "profile": "pc-q35-8.2/qemu64/TCG/1CPU/256MiB/VGA/PS2", "development_profile": d.DEV_PROFILE, "frozen_artifacts": frozen,
                  "loader_geometry_contract": d.geometry_contract(),
                  "command": d.qemu_arguments("/qemu", "pc-q35-8.2", "pipe")}
        return directory, report

    def test_complete_transcript_and_live_raw_sequences(self):
        result = d.validate_sequence("UEFI banner\n" + serial())
        self.assertEqual(result["counters"]["SAMPLES"], 16)
        self.assertEqual(result["raw_samples"][1][-1], {"sequence": 8, "status": 0x21, "byte": 0})

    def test_forged_missing_duplicate_extra_or_replayed_markers_fail(self):
        text = serial()
        bad = (text.replace("DISPINPUT:REUSE:PASS\n", ""), text + "M1:PASS\n", text + "UNKNOWN\n",
               "DISPINPUT:INPUT:PASS\n" + text, text.replace("M2:PASS", "M2:FAIL"),
               text.replace("DISPINPUT:INPUT_READY:00000000000000000002", "DISPINPUT:INPUT_READY:00000000000000000001"),
               text.replace("DISPINPUT:SEQ:00000000000000000008", "DISPINPUT:SEQ:00000000000000000007", 1),
               text.replace("DISPINPUT:RAW:00000000000000000286", "DISPINPUT:RAW:00000000000000008478", 1),
               text.replace("DISPINPUT:SEQ:00000000000000000001", "DISPINPUT:SEQ:1", 1))
        for changed in bad:
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                d.validate_sequence(changed)

    def test_incomplete_native_counters_and_unsigned_overflow_fail(self):
        for name, value in (("EXITS", "00000000000000000005"), ("OUTPUT_UNCHANGED", "00000000000000000098"),
                            ("INPUT_TICKET", "00000000000000000001"), ("TIMER_IRQS", "00000000000000000000"),
                            ("POINTER_DENIED", "18446744073709551616"), ("READY", "-0000000000000000002")):
            line = next(line for line in serial().splitlines() if line.startswith(f"DISPINPUT:{name}:"))
            with self.subTest(name=name), self.assertRaises(ValueError):
                d.validate_sequence(serial().replace(line, f"DISPINPUT:{name}:{value}"))

    def test_actual_pixels_reject_partial_store_stride_swap_geometry_and_hidden_damage(self):
        first, second = frames()
        self.assertTrue(d.validate_frames(first, second)["outside_second_tile_unchanged"])
        width, height, pixels = d.ppm(second)
        header = f"P6\n{width} {height}\n255\n".encode()
        for at, color in ((((80 * 800 + 112) * 3), b"\xee\xee\xee"),
                          (((64 * 800 + 64) * 3), b"\x01\x02\x03"), (0, b"\x01\x02\x03"),
                          (((81 * 800 + 112) * 3), pixels[(80 * 800 + 112) * 3:(80 * 800 + 112) * 3 + 3])):
            changed = bytearray(pixels)
            changed[at:at + 3] = color
            with self.subTest(at=at), self.assertRaises(ValueError):
                d.validate_frames(first, header + changed)
        for changed in (second[:-1], second.replace(b"800 600", b"799 600", 1), b"P6\n1921 600\n255\n" + pixels):
            with self.assertRaises(ValueError):
                d.validate_frames(first, changed)
        # A complete PPM at the old mistaken geometry is still forbidden;
        # this development gate never falls back to any available resolution.
        with self.assertRaises(ValueError):
            d.validate_frames(first, b"P6\n1280 720\n255\n" + bytes(1280 * 720 * 3))

    def test_qmp_cannot_substitute_other_keys_input_or_success_responses(self):
        directory = Path("/private/run")
        trace = [{"request": request, "response": {"return": {}, "id": request["id"]}} for request in d.qmp_requests(directory)]
        d.validate_qmp_trace(trace, directory)
        for index, field, value in ((2, "request", {"execute": "send-key", "id": 3, "arguments": {"keys": [{"type": "qcode", "data": "b"}]}}),
                                    (0, "response", {"return": {}, "id": True}),
                                    (3, "response", {"error": {}, "id": 4})):
            changed = copy.deepcopy(trace)
            changed[index][field] = value
            with self.subTest(index=index), self.assertRaises(ValueError):
                d.validate_qmp_trace(changed, directory)
        for changed in (trace[:-1], trace + [trace[-1]]):
            with self.assertRaises(ValueError):
                d.validate_qmp_trace(changed, directory)

    def test_build_requires_current_profile_paired_loader_three_exact_embedded_elfs_and_two_esp_files(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); self.checkout(root)
            build, bound = self.build_fixture(root)
            for change in ({"loader_feature": "graphics-g0"}, {"source_sha256": "b" * 64},
                           {"kernel_feature": "native-system"}, {"sdk_runtime": True}, {"source_fingerprint_version": True}):
                (build / "manifest.json").write_text(json.dumps(bound["manifest"] | change))
                with self.subTest(change=change), self.assertRaises(ValueError):
                    d.validate_build(build, "a" * 64)
            (build / "manifest.json").write_text(json.dumps(bound["manifest"]))
            (build / "esp/hidden").write_bytes(b"extra")
            with self.assertRaises(ValueError): d.validate_build(build, "a" * 64)
            (build / "esp/hidden").unlink()
            app = build / "users/display-probe/input-probe.elf"
            app.write_bytes(user_elf(7))
            with self.assertRaises(ValueError): d.validate_build(build, "a" * 64)

    def test_changed_embedded_program_cannot_pass_even_with_rehashed_kernel(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); self.checkout(root)
            build, bound = self.build_fixture(root)
            (build / "esp/kernel.elf").write_bytes(b"kernel" + user_elf(0) + user_elf(1))
            manifest = copy.deepcopy(bound["manifest"])
            manifest["kernel"]["sha256"] = d.sha256(build / "esp/kernel.elf")
            (build / "manifest.json").write_text(json.dumps(manifest))
            with self.assertRaises(ValueError): d.validate_build(build, "a" * 64)

    def test_execution_false_pass_wrong_ram_network_timeout_serial_and_snapshot_are_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); self.checkout(root)
            build, bound = self.build_fixture(root)
            directory, report = self.execution_fixture(build, bound)
            self.assertEqual(d.validate_execution(report, bound, directory, "/qemu", "pc-q35-8.2", "pipe")["counters"]["EXITS"], 6)
            wrong_ram = list(report["command"]); wrong_ram[wrong_ram.index("256M")] = "8G"
            for change in ({"raw_qemu_exit": True}, {"timed_out": True}, {"raw_qemu_exit": 0},
                           {"stdout": serial() + "M1:PASS\n"}, {"command": wrong_ram},
                           {"development_profile": "display-input-any-geometry"},
                           {"loader_geometry_contract": d.geometry_contract() | {"width": 1280}},
                           {"command": report["command"] + ["-net", "user"]}):
                with self.subTest(change=change), self.assertRaises(ValueError):
                    d.validate_execution(report | {"passed": True} | change, bound, directory, "/qemu", "pc-q35-8.2", "pipe")
            (directory / "inputs/display-probe.elf").write_bytes(b"changed")
            with self.assertRaises(ValueError):
                d.validate_execution(report, bound, directory, "/qemu", "pc-q35-8.2", "pipe")

    def test_linked_reports_and_linked_build_outputs_refuse_before_tools_or_external_writes(self):
        for kind in ("symlink", "hardlink"):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as temp:
                base = Path(temp); root = base / "checkout"; root.mkdir(); self.checkout(root)
                build = root / "build"; build.mkdir()
                outside = base / "keep"; outside.write_bytes(b"preserve")
                path = build / "display-probe-report.json"
                if kind == "symlink": path.symlink_to(outside)
                else: os.link(outside, path)
                with patch.object(d, "source_hash") as source, patch.object(d, "environment") as env, self.assertRaises(ValueError):
                    d.run_display_probe(build_directory=build, qmp_transport="pipe", vga_rom=outside)
                source.assert_not_called(); env.assert_not_called()
                self.assertEqual(outside.read_bytes(), b"preserve")
                path.unlink()
                output = build / "users/display-probe/display-probe.elf"; output.parent.mkdir(parents=True)
                if kind == "symlink": output.symlink_to(outside)
                else: os.link(outside, output)
                with patch.object(d, "BUILD", build), patch.object(d, "TARGET", root / "target"), patch.object(d, "run_logged") as tool, self.assertRaises(ValueError):
                    d.build_users()
                tool.assert_not_called()
                self.assertEqual(outside.read_bytes(), b"preserve")

    def test_timeout_with_raw_exit33_still_publishes_failed_report(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); self.checkout(root)
            build, _ = self.build_fixture(root)
            qemu = root / "test-tool"; qemu.write_bytes(b"not executed")
            code = root / "code.fd"; code.write_bytes(b"code")
            variables = root / "vars.fd"; variables.write_bytes(b"vars")
            rom = root / "vga.rom"; rom.write_bytes(b"rom")
            process = Mock(returncode=33)
            process.poll.side_effect = [None, 33]
            with patch.object(d, "source_hash", return_value="a" * 64), patch.object(d, "environment", return_value={"PATH": "/test"}), \
                    patch.object(d.shutil, "which", return_value=str(qemu)), patch.object(d, "firmware", return_value=(code, variables)), \
                    patch.object(d.subprocess, "check_output", side_effect=["QEMU emulator version test", "c" * 40]), \
                    patch.object(d.subprocess, "Popen", return_value=process), \
                    patch.object(d, "QmpPipe", side_effect=TimeoutError("deliberately absent QMP greeting")):
                result = d.run_display_probe(build_directory=build, qmp_transport="pipe", vga_rom=rom)
            self.assertIs(result["passed"], False)
            self.assertIs(result["timed_out"], True)
            self.assertEqual(result["raw_qemu_exit"], 33)
            self.assertIs(json.loads((build / "display-probe-report.json").read_text())["passed"], False)
            process.kill.assert_called_once()

    def test_stale_prior_pass_is_retired_before_a_bad_new_manifest(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); self.checkout(root)
            build, bound = self.build_fixture(root)
            (build / "display-probe-report.json").write_text('{"passed":true}')
            (build / "manifest.json").write_text(json.dumps(bound["manifest"] | {"loader_feature": "graphics-g0"}))
            with patch.object(d, "source_hash", return_value="a" * 64), patch.object(d, "environment") as env, self.assertRaises(ValueError):
                d.run_display_probe(build_directory=build, qmp_transport="pipe", vga_rom=root / "unopened-rom")
            env.assert_not_called()
            self.assertIs(json.loads((build / "display-probe-report.json").read_text())["passed"], False)


if __name__ == "__main__":
    unittest.main()
