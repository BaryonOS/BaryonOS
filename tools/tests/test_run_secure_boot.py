"""Host-only checks for the B04 Secure Boot TEST runner; no QEMU, no signing."""
import inspect
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import unittest.mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_secure_boot as sb

DENIED = ('BdsDxe: failed to load Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x1,0x0): '
          'Access Denied -- rejected probably by Secure Boot\nBdsDxe: No bootable option or device was found.\n')


def pe(size=0x200, header=0x80):
    data = bytearray(size)
    data[:2] = b"MZ"
    data[0x3C:0x40] = header.to_bytes(4, "little")
    data[header:header + 4] = b"PE\0\0"
    return bytes(data)


class RefusalTests(unittest.TestCase):
    def test_denial_with_timeout_is_refusal(self):
        self.assertEqual(sb.classify_refusal(DENIED, -9, True), [])

    def test_guest_marker_is_not_refusal(self):
        self.assertTrue(sb.classify_refusal(DENIED + "BOOT:FLOOR:0\n", -9, True))
        self.assertTrue(sb.classify_refusal(DENIED + "xxBOOT:EBS_OK\n", -9, True))

    def test_silence_without_denial_is_not_refusal(self):
        # A hang or crash must not count as a firmware refusal.
        self.assertTrue(sb.classify_refusal("", -9, True))
        self.assertTrue(sb.classify_refusal("Access Denied\n", -9, True))

    def test_denial_of_another_device_is_not_refusal(self):
        other = DENIED.replace("Boot0002", "Boot0003").replace("Pci(0x1,0x0)", "Pci(0x5,0x0)")
        self.assertTrue(sb.classify_refusal(other, -9, True))
        split = ('BdsDxe: failed to load Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x1,0x0): Not Found\n'
                 "Access Denied -- rejected probably by Secure Boot\n")
        self.assertTrue(sb.classify_refusal(split, -9, True))

    def test_smoke_exit_is_not_refusal(self):
        self.assertTrue(sb.classify_refusal(DENIED, 33, False))


class ArgumentTests(unittest.TestCase):
    BASE = ["qemu", "-machine", "pc-q35-10.0", "-accel", "tcg,thread=single", "-drive", "x"]

    def test_smm_and_secure_pflash(self):
        args = sb.secure_boot_args(self.BASE, "pc-q35-10.0")
        self.assertEqual(args[2], "pc-q35-10.0,smm=on")
        self.assertIn("driver=cfi.pflash01,property=secure,value=on", args)
        self.assertEqual(self.BASE[2], "pc-q35-10.0")

    def test_machine_mismatch_refused(self):
        with self.assertRaises(ValueError):
            sb.secure_boot_args(self.BASE, "pc-q35-8.2")


class DestinationTests(unittest.TestCase):
    def test_system_vars_refused(self):
        for path in (sb.SNAKEOIL_VARS, Path("/usr/share/OVMF/x.fd"), Path("/etc/x")):
            with self.assertRaises(ValueError):
                sb.writable_destination(path)

    def test_outside_run_refused(self):
        with self.assertRaises(ValueError):
            sb.writable_destination(sb.BUILD / "esp" / "OVMF_VARS.fd")

    def test_run_dir_allowed(self):
        path = sb.BUILD / "run" / "secure-boot-x" / "OVMF_VARS.fd"
        self.assertEqual(sb.writable_destination(path), path.resolve())


class TamperTests(unittest.TestCase):
    def test_flips_exactly_the_timestamp_byte(self):
        image = pe()
        changed = sb.tamper(image)
        diff = [i for i, (a, b) in enumerate(zip(image, changed)) if a != b]
        self.assertEqual(diff, [0x80 + 8])

    def test_rejects_non_pe(self):
        with self.assertRaises(ValueError):
            sb.timestamp_offset(b"\0" * 0x200)
        bad = bytearray(pe())
        bad[0x80:0x84] = b"XXXX"
        with self.assertRaises(ValueError):
            sb.timestamp_offset(bytes(bad))


class CaseTableTests(unittest.TestCase):
    def test_mandatory_cases_and_expectations(self):
        expected = {"unsigned": "refused", "signed": "boot", "wrong-key": "refused", "tampered": "refused"}
        for case, outcome in expected.items():
            self.assertEqual(sb.CASES[case][3], outcome)
        self.assertEqual(set(sb.ORDER), set(sb.CASES))



class KeyCleanupTests(unittest.TestCase):
    def test_key_removed_when_signing_fails(self):
        for case in ("signed", "wrong-key"):
            with tempfile.TemporaryDirectory() as temp:
                work = Path(temp)
                loader = work / "loader.efi"
                loader.write_bytes(b"MZ")

                def fake_run(command, **kwargs):
                    if command[0] == "openssl":
                        flag = "-out" if command[1] == "pkey" else "-keyout"
                        Path(command[command.index(flag) + 1]).write_text("TEST KEY")
                        if "-x509" in command:
                            Path(command[command.index("-out") + 1]).write_text("TEST CERT")
                        return subprocess.CompletedProcess(command, 0)
                    raise subprocess.CalledProcessError(1, command)

                with unittest.mock.patch.object(sb.subprocess, "run", side_effect=fake_run) as run:
                    with self.assertRaises(subprocess.CalledProcessError):
                        sb.make_loader(case, loader, work, {})
                self.assertEqual(run.call_args.args[0][0], "sbsign")
                self.assertFalse((work / "signing-key.pem").exists(), case)


def parameters(function):
    return list(inspect.signature(function).parameters)


class InterfaceTests(unittest.TestCase):
    """The runner composes B03 internals; fail here if their shape changes."""

    def test_run_update_interfaces(self):
        import run_update
        self.assertEqual(parameters(run_update.qemu_command), ["qemu", "run_dir", "state_dir"])
        self.assertEqual(parameters(run_update.execute), ["args", "env", "cut_marker", "timeout"])
        self.assertEqual(parameters(run_update.pack_store), ["stage", "images", "env", "path"])
        self.assertEqual(parameters(run_update.mutate), ["store", "steps"])
        self.assertEqual(parameters(run_update.evaluate),
                         ["stage", "stdout", "rc", "timed_out", "killed", "running"])
        self.assertEqual(parameters(run_update.check_store), ["stage", "before", "after", "images"])
        self.assertEqual(parameters(run_update.load_images), [])
        for stage in ("install", "bad-digest"):
            self.assertLessEqual({"pack", "mutate", "loader", "outcome"}, set(run_update.STAGES[stage]))
        self.assertTrue(callable(run_update.disk.sha256))
        self.assertIsInstance(run_update.MAX_TIMEOUT, int)

    def test_qemu_command_shape(self):
        import run_update
        run = sb.BUILD / "run" / "x"
        args = run_update.qemu_command("qemu", run, run)
        self.assertEqual(args.count("-machine"), 1)
        self.assertEqual(sb.secure_boot_args(args, args[args.index("-machine") + 1])[-1],
                         "driver=cfi.pflash01,property=secure,value=on")

    def test_test_update_interface(self):
        import test_update
        self.assertEqual(parameters(test_update.prepare_images), ["frozen"])


def report(case, expected, passed, code=sb.EXPECTED_CODE_SHA256, vars_=sb.EXPECTED_VARS_SHA256, run=None):
    return {"case": case, "expected": expected, "passed": passed, "problems": [],
            "firmware": {"code_sha256": code, "vars_template_sha256": vars_,
                         "vars_run_before_sha256": run or vars_}}


class SecureBootProofTests(unittest.TestCase):
    def test_boot_alone_is_unproven(self):
        reports = [report("signed", "boot", True)]
        sb.require_refusal_proof(reports)
        self.assertFalse(reports[0]["passed"])
        self.assertIn("unproven", reports[0]["problems"][0])

    def test_failed_refusal_is_no_proof(self):
        reports = [report("unsigned", "refused", False), report("signed", "boot", True)]
        sb.require_refusal_proof(reports)
        self.assertFalse(reports[1]["passed"])

    def test_refusal_on_other_firmware_is_no_proof(self):
        for kwargs in ({"code": "0" * 64}, {"vars_": "1" * 64}, {"run": "2" * 64}):
            reports = [report("unsigned", "refused", True, **kwargs), report("signed", "boot", True)]
            sb.require_refusal_proof(reports)
            self.assertFalse(reports[1]["passed"], kwargs)

    def test_matching_refusal_proves_boot(self):
        reports = [report("unsigned", "refused", True), report("signed", "boot", True),
                   report("kernel-digest", "boot", True)]
        sb.require_refusal_proof(reports)
        self.assertTrue(reports[1]["passed"] and reports[2]["passed"])
        self.assertEqual(reports[1]["secure_boot_proof"], ["unsigned"])

    def test_failed_boot_stays_failed(self):
        reports = [report("unsigned", "refused", True), report("signed", "boot", False)]
        sb.require_refusal_proof(reports)
        self.assertFalse(reports[1]["passed"])

    def test_firmware_pins_fail_closed(self):
        self.assertEqual(sb.firmware_pin_problems(sb.EXPECTED_CODE_SHA256, sb.EXPECTED_VARS_SHA256), [])
        self.assertTrue(sb.firmware_pin_problems("0" * 64, sb.EXPECTED_VARS_SHA256))
        self.assertTrue(sb.firmware_pin_problems(sb.EXPECTED_CODE_SHA256, "0" * 64))

    def test_main_refuses_unpinned_firmware_before_any_run(self):
        with unittest.mock.patch.object(sb, "sha256", return_value="0" * 64), \
                unittest.mock.patch.object(sb, "prepare", side_effect=AssertionError("must not build")):
            self.assertEqual(sb.main(["--case", "signed"]), 1)


if __name__ == "__main__":
    unittest.main()
