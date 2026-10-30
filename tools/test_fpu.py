#!/usr/bin/env python3
"""Host rejection tests for FPU evidence; these are not native FP evidence."""
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import run_fpu as f


def serial(*, wait=False):
    values = (3, 49, 52, 18, 17, 21, 38, 124, 163, 163, 1, 2, 1, 49263, 8192)
    fields = f.FIELDS
    end = f.END
    if wait:
        fields += f.WAIT_FIELDS
        values += (49, 47, 47, 2, 31)
        end = ("FPU:WAIT:PASS",) + end
    return "\n".join(f.BOOT + f.CASES + tuple(f"FPU:{name}:{value:020d}" for name, value in zip(fields, values)) + end) + "\n"


class EvidenceTests(unittest.TestCase):
    def test_valid_numeric_evidence(self):
        self.assertEqual(f.validate_sequence("UEFI banner\n" + serial())["SAVES"], 163)

    def test_wait_profile_requires_real_blocking_and_exact_counters(self):
        self.assertEqual(f.validate_sequence(serial(wait=True), wait=True)["WAIT_BLOCKED"], 47)
        for label, value in (("WAIT_CALLS", 48), ("WAIT_BLOCKED", 2),
                             ("WAIT_TIMEOUTS", 46), ("WAIT_IMMEDIATE", 0), ("WAIT_IDLE_IRQS", 0)):
            text = serial(wait=True)
            line = next(line for line in text.splitlines() if line.startswith(f"FPU:{label}:"))
            with self.subTest(label=label), self.assertRaises(ValueError):
                f.validate_sequence(text.replace(line, f"FPU:{label}:{value:020d}"), wait=True)
        for text, wait in ((serial(), True), (serial(wait=True), False),
                           (serial(wait=True).replace("FPU:WAIT:PASS\n", ""), True)):
            with self.subTest(wait=wait, text=text), self.assertRaises(ValueError):
                f.validate_sequence(text, wait=wait)

    def test_unknown_missing_duplicate_or_reordered_markers(self):
        text = serial()
        for changed in (text + "UNKNOWN\n", text.replace("FPU:PASS\n", ""), text + "M1:PASS\n",
                        text.replace(f.CASES[1], "TEMP").replace(f.CASES[2], f.CASES[1]).replace("TEMP", f.CASES[2]),
                        "FPU:PASS\n" + text, text.replace("M2:PASS", "M2:FAIL")):
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                f.validate_sequence(changed)

    def test_bad_numeric_fields(self):
        for label, value in (("SAVES", 164), ("RESTORES", 162), ("CLOCK_DENIED", 17), ("TIMER_B", 0),
                             ("CLOCK_READS", 51), ("SYSCALL_CALLS", 123), ("CLEAN_STARTS", 2),
                             ("CONTEXT_BYTES_WIPED", 512), ("FREE_FRAMES", 0), ("TIMER_IRQS", 39)):
            line = next(line for line in serial().splitlines() if line.startswith(f"FPU:{label}:"))
            with self.subTest(label=label), self.assertRaises(ValueError):
                f.validate_sequence(serial().replace(line, f"FPU:{label}:{value:020d}"))
        for value in ("18446744073709551616", "0000000000000000016x", "163", "-0000000000000000163"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                f.validate_sequence(serial().replace("FPU:SAVES:00000000000000000163", "FPU:SAVES:" + value))

    def test_duplicate_or_oversized_json(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "input.json"
            for raw in ('{"passed":false,"passed":true}', ' ' * (1024 * 1024 + 1)):
                path.write_text(raw)
                with self.assertRaises(ValueError):
                    f.load_json(path)

    def test_wrong_build_profile_or_source(self):
        for change in ({"kernel_feature": "milestone-m3"}, {"source_sha256": "old"},
                       {"loader_feature": "graphics-g0"}, {"sdk_runtime": True}):
            manifest = {"kernel_feature": "native-fpu", "loader_feature": None, "source_fingerprint_version": 3,
                        "source_sha256": "new", "sdk_runtime": False, "document_session": False} | change
            with patch.object(f, "load_json", return_value=manifest), self.assertRaises(ValueError):
                f.validate_build(Path("unused"), "new")

    def test_each_exact_elf_must_be_embedded(self):
        with tempfile.TemporaryDirectory() as temp, patch.object(f, "ROOT", Path(temp)):
            build = Path(temp)
            (build / "users/fpu").mkdir(parents=True)
            (build / "esp").mkdir()
            apps = []
            for name in f.NAMES:
                path = build / f"users/fpu/{name}.elf"
                path.write_bytes(name.encode())
                apps.append({"name": name, "sha256": f.sha256(path), "elf": {}})
            (build / "esp/kernel.elf").write_bytes(b"kernel+fpu-a")
            (build / "manifest.json").write_text("{}")
            (build / "users/fpu/build-report.json").write_text("{}")
            artifact = {"passed": True, "native_runtime_performed": False, "apps": apps}
            manifest = {"kernel_feature": "native-fpu", "loader_feature": None, "source_fingerprint_version": 3,
                        "source_sha256": "new", "sdk_runtime": False, "document_session": False, "fpu": artifact}
            with patch.object(f, "load_json", side_effect=[manifest, artifact]), patch.object(f, "inspect_user", return_value={}), self.assertRaises(ValueError):
                f.validate_build(build, "new")

    def test_symlinks_and_special_files_rejected_before_reads(self):
        import os
        with tempfile.TemporaryDirectory() as temp, patch.object(f, "ROOT", Path(temp)):
            root = Path(temp)
            (root / "real").write_bytes(b"image")
            (root / "link").symlink_to(root / "real")
            os.mkfifo(root / "fifo")
            for path in (root / "link", root / "fifo", root.parent / "outside"):
                with self.subTest(path=path), self.assertRaises(ValueError):
                    f.private_path(path, regular=True)

    def test_actual_builder_refuses_hardlinked_output_before_cargo(self):
        import os
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            root = base / "checkout"
            output = root / "build/users/fpu"
            output.mkdir(parents=True)
            sentinel = base / "outside"
            sentinel.write_bytes(b"preserve-outside")
            os.link(sentinel, output / "fpu-a.elf")
            with patch.object(f, "ROOT", root), patch.object(f, "BUILD", root / "build"), patch.object(f, "TARGET", root / "target"), patch.object(f, "run_logged") as cargo:
                with self.assertRaises(ValueError):
                    f.build_users()
                cargo.assert_not_called()
            self.assertEqual(sentinel.read_bytes(), b"preserve-outside")

    def test_atomic_publication_does_not_mutate_prior_inode(self):
        with tempfile.TemporaryDirectory() as temp, patch.object(f, "ROOT", Path(temp)):
            path = Path(temp) / "output"
            path.write_bytes(b"old")
            with path.open("rb") as prior_inode:
                f.publish(path, b"new")
                self.assertEqual(prior_inode.read(), b"old")
            self.assertEqual(path.read_bytes(), b"new")

    def fixture(self, root):
        build = root / "build"
        directory = build / "run/test"
        directory.mkdir(parents=True)
        expected = {"esp/kernel.elf": "", "esp/EFI/BOOT/BOOTX64.EFI": ""}
        hashes = {}
        for name in (*expected, "OVMF_CODE.fd", "OVMF_VARS.template.fd"):
            path = directory / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(name.encode())
            hashes[name] = f.sha256(path)
            if name in expected:
                expected[name] = hashes[name]
        (directory / "serial.txt").write_text(serial())
        prefix = "build/run/test"
        command = ["/qemu", "-machine", "pc-q35-8.2", "-accel", "tcg,thread=single", "-cpu", "qemu64", "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none", "-serial", "stdio", "-nic", "none", "-no-reboot",
                   "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={prefix}/OVMF_CODE.fd",
                   "-drive", f"if=pflash,unit=1,format=raw,file={prefix}/OVMF_VARS.fd",
                   "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{prefix}/esp",
                   "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
        report = {"raw_qemu_exit": 33, "timed_out": False, "passed": True, "expected_exit": 33,
                  "artifact_binding_passed": True, "required_markers": list(f.REQUIRED), "forbidden_markers": list(f.FORBIDDEN),
                  "source_fingerprint_version": 3, "source_sha256": "new", "run_directory": str(directory), "command": command,
                  "profile": "development-package; pc-q35-8.2/qemu64/TCG/1CPU/256MiB", "qemu_version": "QEMU emulator version 8.2.2",
                  "artifact_sha256": hashes, "expected_artifact_sha256": expected, "stdout": serial()}
        return build, directory, {"expected": expected}, report

    def test_guest_artifact_profile_source_and_serial_binding(self):
        with tempfile.TemporaryDirectory() as temp, patch.object(f, "ROOT", Path(temp)):
            build, directory, bound, report = self.fixture(Path(temp))
            self.assertEqual(f.validate_guest(report, bound, "new", "pc-q35-8.2", "/qemu", build)["FAULTS"], 1)
            for change in ({"raw_qemu_exit": True}, {"timed_out": True}, {"source_sha256": "old"},
                           {"passed": False}, {"profile": "8GiB"}, {"stdout": serial() + "garbage"},
                           {"artifact_binding_passed": False}, {"command": report["command"] + ["-net", "user"]}):
                with self.subTest(change=change), self.assertRaises(ValueError):
                    f.validate_guest(report | change, bound, "new", "pc-q35-8.2", "/qemu", build)
            (directory / "esp/kernel.elf").write_bytes(b"changed")
            with self.assertRaises(ValueError):
                f.validate_guest(report, bound, "new", "pc-q35-8.2", "/qemu", build)

    def test_wait_profile_cannot_reuse_normal_guest_evidence(self):
        with tempfile.TemporaryDirectory() as temp, patch.object(f, "ROOT", Path(temp)):
            build, _, bound, report = self.fixture(Path(temp))
            bound["manifest"] = {"kernel_feature": "test-fpu-wait"}
            with self.assertRaises(ValueError):
                f.validate_guest(report, bound, "new", "pc-q35-8.2", "/qemu", build)


if __name__ == "__main__":
    unittest.main()
