"""Host counterexamples for the runner; these tests are not native IPC evidence."""
from contextlib import ExitStack
import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_hid_input as hid
from test_display_probe import user_elf


SOURCE = "a" * 64
LOADER = "c" * 64
VOLUME = "d" * 64


def serial():
    # Independent fixture contract, rather than constructing positive evidence
    # from the implementation's expected-marker constants. The M3 diagnostic
    # also exercises the shared helper's distinct M1-only profile inference.
    return """firmware banner
BOOT:EBS_OK
KERNEL:ENTRY
BOOTINFO:OK
KS-01:PASS
KS-02:PASS
KS-03:PASS
M3:IPC:READY
HID:SYNTHETIC_REPORTS:ACTUAL_IPC
HID:BOOT:PASS
HID:KEYBOARD:PASS
HID:MOUSE:PASS
HID:FOCUS:PASS
HID:RESET:PASS
HID:DETACH:PASS
HID:REATTACH:PASS
HID:REJECT:PASS
HID:SOURCE_PUBLICATIONS:00000000000000000011
HID:SOURCE_REJECTIONS:00000000000000000006
HID:CLIENT_EVENTS:00000000000000000010
HID:CLIENT_CANCELS:00000000000000000004
HID:STALE_ACK:00000000000000000001
HID:ROLE_DENIAL:00000000000000000001
HID:POLICY_REJECTIONS:00000000000000000002
HID:POLICY_RECEIVE_RIGHTS:00000000000000000015
HID:CLEANUP:PASS
HID:CASE:PASS
M1:PASS
"""


class StageOracleTests(unittest.TestCase):
    def test_one_complete_sequence_has_explicit_synthetic_scope(self):
        result = hid.evaluate(serial(), 33, False)
        self.assertTrue(result["synthetic_reports"])
        self.assertTrue(result["native_ipc_assertions"])
        self.assertFalse(result["usb_enumeration"])
        self.assertFalse(result["rendering"])

    def test_every_boot_stage_and_terminal_marker_is_required_once(self):
        for marker in hid.BOOT + hid.STAGES + (hid.END,):
            for replacement in ("", marker + "\n" + marker + "\n"):
                with self.subTest(marker=marker, replacement=replacement), self.assertRaises(ValueError):
                    hid.evaluate(serial().replace(marker + "\n", replacement), 33, False)

    def test_actual_cancellation_stages_cannot_be_omitted(self):
        for reason in ("FOCUS", "RESET", "DETACH"):
            with self.subTest(reason=reason), self.assertRaises(ValueError):
                hid.evaluate(serial().replace(f"HID:{reason}:PASS\n", ""), 33, False)

    def test_exact_counter_values_and_canonical_decimal_encoding_are_required(self):
        for name, value in hid.COUNTERS:
            original = f"HID:{name}:{value:020d}"
            for wrong in (f"{value - 1:020d}", str(value), "-1", "0x1"):
                with self.subTest(name=name, wrong=wrong), self.assertRaises(ValueError):
                    hid.evaluate(serial().replace(original, f"HID:{name}:{wrong}"), 33, False)

    def test_unknown_padded_failed_and_reordered_stages_are_rejected(self):
        variants = [
            serial().replace("HID:MOUSE:PASS", "HID:UNRECOGNIZED:PASS"),
            serial().replace("HID:MOUSE:PASS", " HID:MOUSE:PASS"),
            serial().replace("HID:MOUSE:PASS", "HID:MOUSE:FAIL"),
            serial().replace("HID:KEYBOARD:PASS\nHID:MOUSE:PASS", "HID:MOUSE:PASS\nHID:KEYBOARD:PASS"),
            serial().replace("HID:CASE:PASS", "HID:EXTRA:PASS\nHID:CASE:PASS"),
            serial().replace("HID:CASE:PASS", "PANIC\nHID:CASE:PASS"),
            serial() + "unexpected output after completion\n",
        ]
        for value in variants:
            with self.subTest(value=value), self.assertRaises(ValueError):
                hid.evaluate(value, 33, False)

    def test_timeout_and_wrong_raw_exit_cannot_certify_even_complete_markers(self):
        for raw_exit, timed_out in ((None, True), (33, True), (0, False), (35, False), (33.0, False), (33, 0)):
            with self.subTest(raw_exit=raw_exit, timed_out=timed_out), self.assertRaises(ValueError):
                hid.evaluate(serial(), raw_exit, timed_out)

    def test_oversized_transcript_is_rejected(self):
        with self.assertRaises(ValueError):
            hid.evaluate("x" * (hid.SERIAL_LIMIT + 1) + serial(), 33, False)


class RunnerBindingTests(unittest.TestCase):
    def setUp(self):
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.directory = Path(self.stack.enter_context(tempfile.TemporaryDirectory()))
        self.stack.enter_context(patch.object(hid, "BUILD", self.directory))
        self.source_hash = self.stack.enter_context(patch.object(hid, "source_hash", return_value=SOURCE))
        self.app_dir = self.directory / "users/materiaal"
        self.app_dir.mkdir(parents=True)
        applications = []
        for index, name in enumerate(sorted(hid.APP_NAMES)):
            path = self.app_dir / (name + ".elf")
            path.write_bytes(user_elf(index))
            applications.append({"name": name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                                 "elf": hid.inspect_user(path)})
        self.kernel = self.directory / hid.KERNEL
        self.kernel.parent.mkdir(parents=True)
        self.kernel.write_bytes(b"host embedding counterexample fixture\n" + b"".join(
            (self.app_dir / (name + ".elf")).read_bytes() for name in hid.INPUT_APPS))
        self.kernel_hash = hashlib.sha256(self.kernel.read_bytes()).hexdigest()
        self.manifest = {
            "kernel_feature": hid.FEATURE,
            "loader_feature": hid.LOADER_FEATURE,
            "source_fingerprint_version": 3,
            "source_sha256": SOURCE,
            "kernel": {"sha256": self.kernel_hash},
            "loader": {"sha256": LOADER},
            "materiaal": {"passed": True, "native_runtime_performed": False,
                          "probe_controller": False, "hid_synthetic_probe": True,
                          "apps": applications},
        }
        self.manifest_path = self.directory / "manifest.json"
        self.write_manifest()
        self.rom = self.directory / "rom.bin"
        self.rom.write_bytes(b"isolated test ROM")
        self.guest = {
            "stdout": serial(), "raw_qemu_exit": 33, "timed_out": False,
            "passed": True, "hid_input_probe": True, "artifact_binding_passed": True,
            "source_fingerprint_version": 3, "source_sha256": SOURCE,
            "artifact_sha256": {hid.KERNEL: self.kernel_hash, hid.LOADER: LOADER,
                                "VGA.rom": hashlib.sha256(self.rom.read_bytes()).hexdigest()},
            "volume_initial_sha256": VOLUME, "volume_final_sha256": VOLUME,
        }
        self.run_guest = self.stack.enter_context(patch.object(hid, "run_guest", return_value=self.guest))

    def write_manifest(self):
        self.manifest_path.write_text(json.dumps(self.manifest), encoding="utf-8")

    def saved_report(self):
        return json.loads((self.directory / "hid-input-report.json").read_text(encoding="utf-8"))

    def test_current_manifest_passes_exact_profile_and_hashes_to_shared_runner(self):
        result = hid.run_probe(vga_rom=self.rom)
        self.assertTrue(result["passed"])
        self.assertTrue(self.saved_report()["passed"])
        args, kwargs = self.run_guest.call_args
        self.assertEqual(args, ("hid-input",))
        self.assertTrue(kwargs["hid_input_probe"])
        self.assertEqual(kwargs["vga_rom"], self.rom)
        self.assertEqual(kwargs["expected_kernel_sha256"], self.kernel_hash)
        self.assertEqual(kwargs["expected_loader_sha256"], LOADER)
        self.assertEqual(kwargs["timeout"], 60)
        self.assertNotIn(hid.END, kwargs["required_markers"])
        self.assertEqual(set(result["input_app_sha256"]), set(hid.INPUT_APPS))

    def test_missing_duplicate_or_wrong_profile_apps_are_rejected_before_launch(self):
        original = copy.deepcopy(self.manifest["materiaal"])
        variants = [
            {**original, "hid_synthetic_probe": False},
            {**original, "apps": original["apps"][:-1]},
            {**original, "apps": original["apps"][:-1] + [original["apps"][0]]},
        ]
        for value in variants:
            self.manifest["materiaal"] = value
            self.write_manifest()
            with self.subTest(value=value), self.assertRaises(ValueError):
                hid.run_probe(vga_rom=self.rom)
        self.run_guest.assert_not_called()

    def test_fixture_hash_metadata_and_kernel_embedding_are_independently_checked(self):
        path = self.app_dir / "input-hid-source.elf"
        original = copy.deepcopy(self.manifest)
        original_bytes = path.read_bytes()
        for kind in ("hash", "metadata", "embedding"):
            self.manifest = copy.deepcopy(original)
            path.write_bytes(original_bytes)
            app = next(app for app in self.manifest["materiaal"]["apps"] if app["name"] == "input-hid-source")
            if kind == "metadata":
                app["elf"] = {}
            else:
                path.write_bytes(user_elf(20))
                if kind == "embedding":
                    # Even a valid replacement with matching manifest metadata
                    # must be rejected when the actual kernel embeds old bytes.
                    app["sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
                    app["elf"] = hid.inspect_user(path)
            self.write_manifest()
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                hid.run_probe(vga_rom=self.rom)
            self.assertFalse(self.saved_report()["passed"])
        self.run_guest.assert_not_called()

    def test_app_replacement_during_execution_cannot_leave_success(self):
        def changed(*args, **kwargs):
            (self.app_dir / "input-policy.elf").write_bytes(user_elf(20))
            return self.guest

        self.run_guest.side_effect = changed
        with self.assertRaises(ValueError):
            hid.run_probe(vga_rom=self.rom)
        self.assertFalse(self.saved_report()["passed"])

    def test_wrong_build_or_missing_hash_is_rejected_before_launch(self):
        original = copy.deepcopy(self.manifest)
        changes = (("kernel_feature", "native-window-session-probe"), ("loader_feature", None),
                   ("source_fingerprint_version", 2), ("source_sha256", "e" * 64),
                   ("kernel", {}), ("loader", {"sha256": "missing"}))
        for key, value in changes:
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.manifest = {**original, key: value}
                self.write_manifest()
                hid.run_probe(vga_rom=self.rom)
            self.assertFalse(self.saved_report()["passed"])
        self.run_guest.assert_not_called()

    def test_duplicate_manifest_keys_cannot_override_a_profile(self):
        self.manifest_path.write_text(json.dumps(self.manifest)[:-1] + ',"kernel_feature":"other"}', encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "Duplicate manifest key"):
            hid.run_probe(vga_rom=self.rom)
        self.run_guest.assert_not_called()

    def test_a_failed_run_cannot_leave_a_previous_successful_receipt(self):
        (self.directory / "hid-input-report.json").write_text('{"passed": true}', encoding="utf-8")
        self.guest["stdout"] = self.guest["stdout"].replace("HID:CLEANUP:PASS\n", "")
        with self.assertRaises(ValueError):
            hid.run_probe(vga_rom=self.rom)
        self.assertFalse(self.saved_report()["passed"])
        self.assertIn("guest", self.saved_report())

    def test_guest_success_does_not_override_identity_or_artifact_mismatches(self):
        original = copy.deepcopy(self.guest)
        variants = [
            {**original, "hid_input_probe": False},
            {**original, "artifact_binding_passed": False},
            {**original, "source_sha256": "e" * 64},
            {**original, "artifact_sha256": {**original["artifact_sha256"], hid.KERNEL: "e" * 64}},
            {**original, "artifact_sha256": {**original["artifact_sha256"], "VGA.rom": "e" * 64}},
            {**original, "volume_initial_sha256": None},
        ]
        for value in variants:
            self.run_guest.return_value = value
            with self.subTest(guest=value), self.assertRaises(ValueError):
                hid.run_probe(vga_rom=self.rom)
            self.assertFalse(self.saved_report()["passed"])

    def test_source_change_during_execution_is_rejected(self):
        self.source_hash.side_effect = [SOURCE, "e" * 64]
        with self.assertRaises(ValueError):
            hid.run_probe(vga_rom=self.rom)
        self.assertFalse(self.saved_report()["passed"])

    def test_manifest_or_rom_change_during_execution_is_rejected(self):
        for path in (self.manifest_path, self.rom):
            original = path.read_bytes()

            def changed(*args, **kwargs):
                path.write_bytes(original + b" ")
                return self.guest

            self.run_guest.side_effect = changed
            with self.subTest(path=path), self.assertRaises(ValueError):
                hid.run_probe(vga_rom=self.rom)
            self.assertFalse(self.saved_report()["passed"])
            path.write_bytes(original)

    def test_symlink_rom_and_invalid_timeout_do_not_launch(self):
        link = self.directory / "linked-rom.bin"
        link.symlink_to(self.rom)
        with self.assertRaises(ValueError):
            hid.run_probe(vga_rom=link)
        for timeout in (0, hid.MAX_TIMEOUT + 1, True, float("nan")):
            with self.subTest(timeout=timeout), self.assertRaises(ValueError):
                hid.run_probe(vga_rom=self.rom, timeout=timeout)
        self.run_guest.assert_not_called()


if __name__ == "__main__":
    unittest.main()
