"""HOST_ONLY verifier counterexamples; generated text is no native evidence."""
from pathlib import Path
from contextlib import ExitStack, contextmanager
import hashlib
import json
import os
import struct
import subprocess
import tempfile
from unittest.mock import patch
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_cap_bench as gate
import run_fpu as safety
from build_users import inspect_user


def fixture():
    values = dict(zip(gate.HEADER, (1, 4, 21, 29, 9, 4, 9, 16)))
    rounds = {name: list(range(10 + index, 19 + index)) for index, name in enumerate(gate.VARIANTS)}
    for name, numbers in rounds.items():
        values.update({name + "_COUNT": 144, name + "_TSC_SUM": sum(numbers),
                       name + "_TSC_MIN": min(numbers), name + "_TSC_MAX": max(numbers)})
    boot = list(gate.BOOT)
    # Native serial carries the ACPI status pair between KS-02 and KS-03.
    at = boot.index("KS-02:PASS") + 1
    boot[at:at] = [gate.EXPECTED_ACPI_MARKER, gate.DEVICE_EXECUTION_MARKER]
    lines = boot + [f"CAPBENCH:{name}:{values[name]:020d}" for name in gate.FIELDS]
    for index in range(values["ROUNDS"]):
        lines += [f"CAPBENCH:ROUND_VARIANT:{(index + position) % 3:020d}" for position in range(3)]
        lines += [f"CAPBENCH:ROUND_{name}_TSC:{rounds[name][index]:020d}" for name in gate.VARIANTS]
    return "\n".join(lines + list(gate.END)) + "\n"


class DiagnosticVerifier(unittest.TestCase):
    def test_complete_fixture_only_tests_the_parser(self):
        observed = gate.evaluate(fixture(), 33, False)
        self.assertEqual(observed["values"]["DIRECT_COUNT"], 144)
        self.assertEqual(observed["execution_order"],
                         [[gate.VARIANTS[(r + p) % 3] for p in range(3)] for r in range(9)])

    def test_exit_timeout_and_truncation_cannot_pass(self):
        for raw, timeout in [(35, False), (0, False), (33, True), (None, False)]:
            with self.subTest(raw=raw, timeout=timeout), self.assertRaises(ValueError):
                gate.evaluate(fixture(), raw, timeout)
        with self.assertRaises(ValueError):
            gate.evaluate(fixture().removesuffix("M1:PASS\n"), 33, False)

    def test_altered_totals_rounds_or_workload_are_rejected(self):
        for old, new in [("REF_COUNT:00000000000000000144", "REF_COUNT:00000000000000000143"),
                         ("REF_TSC_SUM:00000000000000000126", "REF_TSC_SUM:00000000000000000127"),
                         ("ROUND_REF_TSC:00000000000000000010", "ROUND_REF_TSC:00000000000000000000"),
                         ("PROCESSES:00000000000000000004", "PROCESSES:00000000000000000005"),
                         ("ROUND_VARIANT:00000000000000000000", "ROUND_VARIANT:00000000000000000002")]:
            value = fixture().replace(old, new)
            self.assertNotEqual(value, fixture())
            with self.subTest(old=old), self.assertRaises(ValueError):
                gate.evaluate(value, 33, False)


# Deliberately independent fixture list. These files model code provenance;
# the patched source_hash response is not a native source/build assertion.
SOURCE_FIXTURES = (
    "tools/run_cap_bench.py", "tools/tests/test_cap_bench.py", "tools/build.py",
    "kernel/src/m2/m3/cap_bench.rs", "kernel/src/m2/m3.rs", "kernel/Cargo.toml", "kernel/build.rs",
    "crates/capability-core/src/lib.rs", "crates/capability-core/Cargo.toml",
    "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo/config.toml",
)
PROGRAMS = {"system_init": ("system-system", "system-init"),
            "system_app": ("system-system", "system-app"),
            "system_worker": ("system-system", "system-worker"),
            "keyring_service": ("system-keyring", "keyring-service"),
            "volume_service": ("system-volume", "volume-service")}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def user_elf(name):
    data = bytearray(8193)
    data[:7] = b"\x7fELF\x02\x01\x01"
    struct.pack_into("<HHIQQQIHHH", data, 16, 2, 62, 1, 0x40000000, 64, 0, 0, 64, 56, 2)
    struct.pack_into("<IIQQQQQQ", data, 64, 1, 6, 4096, 0x40010000, 0x40010000, 1024, 1024, 4096)
    struct.pack_into("<IIQQQQQQ", data, 120, 1, 5, 8192, 0x40000000, 0x40000000, 1, 1, 4096)
    data[8192] = 0xc3
    return bytes(data) + name.encode()


class EntryAndFrozenInputs(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="capbench-HOST_ONLY-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name) / "checkout"
        self.build, self.target = self.root / "build", self.root / "target"
        self.build.mkdir(parents=True)
        self.target.mkdir()
        for name in SOURCE_FIXTURES:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(("HOST_ONLY source fixture: " + name).encode())
        programs = {}
        kernel = b"HOST_ONLY recording image, never executed\n"
        for name, (folder, binary) in PROGRAMS.items():
            path = self.target / folder / "x86_64-unknown-none/release" / binary
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(user_elf(name))
            programs[name] = {"sha256": digest(path.read_bytes()), "elf": inspect_user(path)}
            kernel += path.read_bytes()
        (self.build / "esp/EFI/BOOT").mkdir(parents=True)
        (self.build / "esp/kernel.elf").write_bytes(kernel)
        (self.build / "esp/EFI/BOOT/BOOTX64.EFI").write_bytes(b"HOST_ONLY loader")
        self.manifest = {"schema_version": 1, "source_fingerprint_version": 3, "source_sha256": "a"*64,
                         "kernel_feature": gate.FEATURE, "loader_feature": None, "system": programs,
                         "kernel": {"sha256": digest(kernel)}, "loader": {"sha256": digest(b"HOST_ONLY loader")}}
        (self.build / "manifest.json").write_text(json.dumps(self.manifest))
        self.code, self.variables = self.root / "source-code.fd", self.root / "source-vars.fd"
        self.code.write_bytes(b"HOST_ONLY code bytes")
        self.variables.write_bytes(b"HOST_ONLY UEFI variable template")
        self.qemu = self.root / "qemu-system-x86_64"
        self.qemu.write_bytes(b"HOST_ONLY response-spy identity; not executed")
        self.index = self.build / "cap-bench-report.json"
        self.index.write_text(json.dumps({"passed": True, "status": "completed", "historical": "preserve no stale latest pass"}))
        self.effect = None
        self.calls = 0
        self.executed_directory = None

    @contextmanager
    def controlled(self):
        def metadata(command, **_kwargs):
            return "b"*40+"\n" if command[0] == "git" else "QEMU emulator version HOST_ONLY response spy\n"
        def response(command, *, cwd, **_kwargs):
            self.calls += 1
            self.executed_directory = Path(cwd)
            if self.effect:
                self.effect(self.executed_directory)
            return subprocess.CompletedProcess(command, 33, fixture(), "")
        with ExitStack() as stack:
            for module, name, value in ((gate,"ROOT",self.root),(gate,"BUILD",self.build),(gate,"TARGET",self.target),(safety,"ROOT",self.root)):
                stack.enter_context(patch.object(module,name,value))
            stack.enter_context(patch.object(gate,"source_hash",return_value="a"*64))
            stack.enter_context(patch.object(gate,"environment",return_value={"PATH":""}))
            stack.enter_context(patch.object(gate.shutil,"which",return_value=str(self.qemu)))
            stack.enter_context(patch.object(gate,"firmware",return_value=(self.code,self.variables)))
            stack.enter_context(patch.object(gate,"qemu_machine",return_value="pc-q35-8.2"))
            stack.enter_context(patch.object(gate.subprocess,"check_output",side_effect=metadata))
            stack.enter_context(patch.object(gate.subprocess,"run",side_effect=response))
            yield

    @staticmethod
    def change(path):
        data = bytearray(path.read_bytes()); data[-1] ^= 1; path.write_bytes(data)

    def test_invalid_timeout_invalidates_old_latest_pass_before_any_launch(self):
        with self.controlled(), self.assertRaises(ValueError):
            gate.run(0)
        self.assertEqual(self.calls,0)
        failed = json.loads(self.index.read_text())
        self.assertIs(failed["passed"],False)
        self.assertEqual(failed["status"],"failed")

    def test_bad_target_invalidates_old_latest_pass_without_touching_outside(self):
        outside = self.root.parent / "outside"
        outside.mkdir()
        (outside / "keep").write_bytes(b"unchanged sentinel")
        with self.controlled(), patch.object(gate,"TARGET",outside), self.assertRaises(ValueError):
            gate.run()
        self.assertIs(json.loads(self.index.read_text())["passed"],False)
        self.assertEqual(self.calls,0)
        self.assertEqual((outside/"keep").read_bytes(),b"unchanged sentinel")

    def test_changed_copied_firmware_cannot_pass_by_learning_its_new_hash(self):
        self.effect = lambda run: self.change(run/"OVMF_CODE.fd")
        with self.controlled(): report = gate.run()
        self.assertIs(report["passed"],False)
        self.assertIs(json.loads(self.index.read_text())["passed"],False)

    def test_changed_archived_elf_cannot_pass_with_contradictory_manifest(self):
        self.effect = lambda run: self.change(run/"inputs/system_app.elf")
        with self.controlled(): report = gate.run()
        self.assertIs(report["passed"],False)

    def test_unchanged_host_model_binds_sources_and_all_immutable_copies(self):
        with self.controlled(): report = gate.run()
        self.assertIs(report["passed"],True)  # Parser/binding layer ONLY, no VM.
        self.assertEqual(report["status"],"completed")
        self.assertEqual(set(report["source_files_sha256"]),set(SOURCE_FIXTURES))
        for name, expected in report["expected_artifacts"].items():
            actual = self.executed_directory/name
            self.assertEqual(expected,{"sha256":digest(actual.read_bytes()),"bytes":actual.stat().st_size})
            self.assertEqual(report["artifact_sha256"][name],expected["sha256"])
        for item in report["frozen_sources"].values():
            self.assertEqual(item["sha256"],digest(Path(item["path"]).read_bytes()))
        self.assertEqual(report["initial_mutable_artifacts"],
                         {"OVMF_VARS.fd":{"sha256":digest(self.variables.read_bytes()),"bytes":self.variables.stat().st_size}})

    def test_source_fw_template_target_manifest_and_direct_code_drift_are_rejected(self):
        paths = [self.code,self.variables,self.qemu,self.build/"manifest.json",
                 self.build/"esp/kernel.elf",self.build/"esp/EFI/BOOT/BOOTX64.EFI",
                 self.target/"system-system/x86_64-unknown-none/release/system-app",
                 self.root/"kernel/src/m2/m3/cap_bench.rs"]
        originals = {p:p.read_bytes()for p in paths}
        for path in paths:
            with self.subTest(path=path):
                self.effect = lambda _run, path=path: self.change(path)
                with self.controlled(): report = gate.run()
                self.assertIs(report["passed"],False)
                path.write_bytes(originals[path])

    def test_source_change_during_snapshot_publication_is_rejected_before_launch(self):
        original_publish = gate.publish
        changed = False
        def mutate_after_copy(path, data):
            nonlocal changed
            original_publish(path, data)
            if not changed:
                self.change(self.code)
                changed = True
        with self.controlled(), patch.object(gate,"publish",side_effect=mutate_after_copy), self.assertRaises(ValueError):
            gate.run()
        self.assertTrue(changed)
        self.assertEqual(self.calls,0)
        self.assertIs(json.loads(self.index.read_text())["passed"],False)

    def test_template_archive_and_input_manifest_snapshot_are_immutable(self):
        for name in ("OVMF_VARS.template.fd","inputs/manifest.json","inputs/frozen-inputs.json","volume.img","volume.json"):
            with self.subTest(name=name):
                self.effect = lambda run,name=name:self.change(run/name)
                with self.controlled(): report=gate.run()
                self.assertIs(report["passed"],False)

    def test_legitimate_same_geometry_writable_vars_change_is_not_falsely_refused(self):
        self.effect=lambda run:self.change(run/"OVMF_VARS.fd")
        with self.controlled():report=gate.run()
        self.assertIs(report["passed"],True)
        self.assertNotEqual(report["artifact_sha256"]["OVMF_VARS.fd"],report["firmware_sha256"]["vars_template"])

    def test_initial_writable_vars_must_match_frozen_template_before_launch(self):
        original_publish = gate.publish
        def mutate_initial_vars(path, data):
            original_publish(path, data)
            if path.name == "OVMF_VARS.fd":
                self.change(path)
        with self.controlled(), patch.object(gate,"publish",side_effect=mutate_initial_vars), self.assertRaises(ValueError):
            gate.run()
        self.assertEqual(self.calls,0)
        self.assertIs(json.loads(self.index.read_text())["passed"],False)

    def test_initial_writable_vars_reject_size_and_links_before_launch(self):
        outside = self.root.parent / "vars-sentinel"
        outside.write_bytes(self.variables.read_bytes())
        original_publish = gate.publish
        for kind in ("short", "hardlink", "symlink"):
            with self.subTest(kind=kind):
                def replace_initial_vars(path, data):
                    original_publish(path, data)
                    if path.name == "OVMF_VARS.fd":
                        path.unlink()
                        if kind == "short": path.write_bytes(data[:-1])
                        elif kind == "hardlink": os.link(outside,path)
                        else: path.symlink_to(outside)
                with self.controlled(), patch.object(gate,"publish",side_effect=replace_initial_vars), self.assertRaises(ValueError):
                    gate.run()
                self.assertEqual(self.calls,0)
                self.assertIs(json.loads(self.index.read_text())["passed"],False)
                self.assertEqual(outside.read_bytes(),self.variables.read_bytes())

    def test_launch_exception_keeps_failed_latest_and_propagates(self):
        def fail(_run):raise RuntimeError("controlled launch failure")
        self.effect=fail
        with self.controlled(),self.assertRaisesRegex(RuntimeError,"controlled launch failure"):
            gate.run()
        failed=json.loads(self.index.read_text())
        self.assertIs(failed["passed"],False)
        self.assertEqual(failed["status"],"failed")

    def test_report_link_is_rejected_before_publication_without_external_write(self):
        outside=self.root.parent/"keep";outside.write_bytes(b"controlled sentinel")
        self.index.unlink();os.link(outside,self.index)
        with self.controlled(),self.assertRaises(ValueError):gate.run()
        self.assertEqual(outside.read_bytes(),b"controlled sentinel")
        self.assertEqual(self.calls,0)

    def test_source_esp_extra_link_is_rejected_before_delegation(self):
        outside=self.root.parent/"keep";outside.write_bytes(b"controlled sentinel")
        (self.build/"esp/extra").symlink_to(outside)
        with self.controlled(),self.assertRaises(ValueError):gate.run()
        self.assertEqual(outside.read_bytes(),b"controlled sentinel")
        self.assertEqual(self.calls,0)

    def test_postexecute_disk_link_is_rejected_before_hashing_its_target(self):
        outside=self.root.parent/"disk-sentinel"
        outside.write_bytes(b"controlled external marker, never guest input")
        reads=[]
        original_hash=gate.sha256
        def observed_hash(path):
            if Path(path).resolve()==outside: reads.append(str(path))
            return original_hash(path)
        def replace_disk(run):
            (run/"volume.img").unlink()
            (run/"volume.img").symlink_to(outside)
        self.effect=replace_disk
        with self.controlled(), patch.object(gate,"sha256",side_effect=observed_hash):
            report=gate.run()
        self.assertIs(report["passed"],False)
        self.assertEqual(reads,[])
        self.assertEqual(outside.read_bytes(),b"controlled external marker, never guest input")


if __name__ == "__main__":
    unittest.main()
