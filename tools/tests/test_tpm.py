import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_tpm as probe
from tools.tests import test_ipc_wait


class NativeTpmTranscriptTests(unittest.TestCase):
    LOADER = ("LOADER-TPM:TCG2:PRESENT\nLOADER-TPM:BANKS:000000000000000f\n"
              "LOADER-TPM:PERMANENT:0000000000000400\nLOADER-TPM:PCR13:ZERO\n"
              "LOADER-TPM:PASS\nLOADER-TPM:RNG:ABSENT\n")

    HAUTH = ("LOADER-HAUTH:BEGIN\nLOADER-HAUTH:PERMANENT:{permanent:016x}\n{mode}\n"
             "LOADER-HAUTH:BOUND\n{lockout}\nLOADER-HAUTH:PCR13:CLOSED\n"
             "LOADER-HAUTH:READ:REFUSED\nLOADER-HAUTH:OWNER:REFUSED\n"
             "LOADER-HAUTH:RESIDUE:NONE\nLOADER-HAUTH:PASS\n")

    def transcript(self, case, boot=0):
        if case == "loader-hauth":
            mode, lockout = probe.HAUTH_MODE[boot]
            hauth = self.HAUTH.format(permanent=0x405 if boot else 0x400, mode=mode, lockout=lockout)
            return self.LOADER.replace("RNG:ABSENT", "RNG:PASS") + hauth + self.transcript("normal")
        if case == "loader-probe":
            return self.LOADER + self.transcript("normal")
        if case == "loader-rng":
            return self.LOADER.replace("RNG:ABSENT", "RNG:PASS") + self.transcript("normal")
        text = test_ipc_wait.NativeWaitTranscriptTests().transcript()
        values = {"WRITES": 12000 if case == "normal" else 0,
                  "BOUNDS_REJECTED": 7, "QUARANTINED": int(case == "absent")}
        if case == "normal":
            values["FAMILY"] = 0x322E3000
        extra = ["TPMCRB:BOUNDS:PASS", "TPMCRB:MAPPING:PASS"]
        extra += (["TPMCRB:RELEASED:PASS", "TPMCRB:PASS"] if case == "normal"
                  else ["TPMCRB:ABSENT:PASS"])
        return text + "\n".join(extra + [f"TPMCRB:{k}:{v:020d}" for k, v in values.items()]) + "\n"

    def test_completed_and_missing_device_cases(self):
        for case in probe.FEATURES:
            self.assertEqual(probe.evaluate(self.transcript(case), case)["BOUNDS_REJECTED"], 7)

    def test_loader_probe_records_firmware_facts_and_rejects_gaps(self):
        good = self.transcript("loader-probe")
        result = probe.evaluate(good, "loader-probe")
        self.assertEqual((result["LOADER_BANKS"], result["LOADER_PERMANENT"], result["LOADER_RNG"]),
                         (0xF, 0x400, "ABSENT"))
        self.assertEqual(probe.evaluate(self.transcript("loader-rng"), "loader-rng")["LOADER_RNG"],
                         "PASS")
        # U25 is pinned per case: no RNG without the device, a working RNG with it.
        with self.assertRaisesRegex(ValueError, "must be ABSENT"):
            probe.evaluate(good.replace("RNG:ABSENT", "RNG:PASS"), "loader-probe")
        for bad in (good, self.transcript("loader-rng").replace("RNG:PASS", "RNG:FAIL")):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                probe.evaluate(bad, "loader-rng")
        for bad in (
            self.transcript("normal"),
            good.replace("LOADER-TPM:TCG2:PRESENT", "LOADER-TPM:TCG2:ABSENT"),
            good.replace("PCR13:ZERO", "PCR13:NONZERO"),
            good.replace("LOADER-TPM:PASS\n", ""),
            good.replace("RNG:ABSENT", "RNG:FAIL"),
            good + "LOADER-TPM:RNG:PASS\n",
            good.replace("BANKS:000000000000000f", "BANKS:000000000000000d"),
            good.replace("PERMANENT:0000000000000400", "PERMANENT:400"),
            good.replace("TPMCRB:PASS\n", ""),
        ):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                probe.evaluate(bad, "loader-probe")
        with self.assertRaises(ValueError):
            probe.evaluate(good, "normal")

    def test_loader_hauth_pins_the_boot_mode_and_every_check(self):
        for boot, permanent in ((0, 0x400), (1, 0x405)):
            result = probe.evaluate(self.transcript("loader-hauth", boot), "loader-hauth", boot)
            self.assertEqual((result["HAUTH_PERMANENT"], result["LOADER_RNG"]), (permanent, "PASS"))
        good = self.transcript("loader-hauth")
        for bad, boot in (
            (good, 1),
            (self.transcript("loader-hauth", 1), 0),
            (good + "LOADER-HAUTH:MODE:EXISTING\n", 0),
            (good.replace("LOADER-HAUTH:RESIDUE:NONE\n", ""), 0),
            (good.replace("READ:REFUSED", "READ:ALLOWED"), 0),
            (good.replace("LOADER-HAUTH:PCR13:CLOSED\n", ""), 0),
            (good.replace("HAUTH:PERMANENT:0000000000000400", "HAUTH:PERMANENT:0000000000000401"), 0),
            (good.replace("RNG:PASS", "RNG:ABSENT"), 0),
            (good, 2),
        ):
            with self.subTest(bad=bad, boot=boot), self.assertRaises(ValueError):
                probe.evaluate(bad, "loader-hauth", boot)
        for case in ("loader-rng", "normal"):
            with self.subTest(case=case), self.assertRaises(ValueError):
                probe.evaluate(good, case)

    def test_virtio_rng_is_a_boolean_opt_in_of_the_tpm_fixture_only(self):
        import run
        for kwargs in ({"virtio_rng": True}, {"virtio_rng": 1, "tpm_socket": "x"},
                       {"virtio_rng": True, "tpm_socket": "x", "m5_case": "read"}):
            with self.subTest(kwargs=kwargs), self.assertRaisesRegex(ValueError, "virtio-rng"):
                run.run_guest(**kwargs)

    def test_probe_loader_pairs_only_with_the_native_tpm_kernel(self):
        import build
        with patch.object(build, "ensure_lockfile"), \
                patch.object(build, "source_hash", return_value="s"):
            for loader in ("tpm-loader-probe-test", "tpm-hauth-provision-test"):
                for kernel in (None, "milestone-m2", "test-tpm-crb-absent"):
                    with self.subTest(loader=loader, kernel=kernel), \
                            self.assertRaisesRegex(ValueError, loader):
                        build.build(loader, kernel)

    def test_case_substitution_and_malformed_evidence_are_rejected(self):
        for case in ("normal", "absent"):
            other = "absent" if case == "normal" else "normal"
            good = self.transcript(case)
            for bad in (
                self.transcript(other),
                good.replace("TPMCRB:MAPPING:PASS\n", ""),
                good + "TPMCRB:WRITES:00000000000000000000\n",
                good.replace("TPMCRB:BOUNDS_REJECTED:00000000000000000007",
                             "TPMCRB:BOUNDS_REJECTED:7"),
                good.replace("TPMCRB:BOUNDS_REJECTED:00000000000000000007",
                             "TPMCRB:BOUNDS_REJECTED:00000000000000000006"),
                good.replace(f"TPMCRB:WRITES:{12000 if case == 'normal' else 0:020d}",
                             f"TPMCRB:WRITES:{0 if case == 'normal' else 1:020d}"),
                good.replace("TPMCRB:QUARANTINED:00000000000000000001",
                             "TPMCRB:QUARANTINED:00000000000000000000") if case == "absent"
                else good.replace(f"TPMCRB:FAMILY:{0x322E3000:020d}", f"TPMCRB:FAMILY:{0:020d}"),
            ):
                with self.subTest(case=case, bad=bad), self.assertRaises(ValueError):
                    probe.evaluate(bad, case)


class NativeTpmBackendTests(unittest.TestCase):
    def test_private_state_uses_portable_options_and_terminates_its_child(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / "local/swtpm-native/usr/bin/swtpm"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"test executable identity")
            report = {}
            with patch.object(probe, "ROOT", root), \
                    patch.object(probe, "LOCAL", root / "local"), \
                    patch.object(probe, "BUILD", root / "build"), \
                    patch.object(probe, "environment", return_value={"PATH": ""}), \
                    patch.object(probe.subprocess, "check_output", return_value="swtpm 0.7.3"), \
                    patch.object(probe.subprocess, "Popen") as start, \
                    patch.object(Path, "is_socket", return_value=True):
                child = start.return_value
                child.poll.return_value = None
                child.wait.return_value = 0
                with probe.backend(report) as control:
                    command = start.call_args.args[0]
                    state_option = command[command.index("--tpmstate") + 1]
                    self.assertNotIn(",lock", state_option)
                    state = root / state_option.removeprefix("dir=")
                    self.assertEqual(state.stat().st_mode & 0o777, 0o700)
                    self.assertEqual(control.parent, state.parent)
                    self.assertIn(f"type=unixio,path={control.relative_to(root)},mode=0600", command)
                    self.assertEqual(start.call_args.kwargs["cwd"], root)
                child.terminate.assert_called_once_with()
                child.wait.assert_called_once_with(timeout=5)
                self.assertEqual(report["backend"]["exit_status"], 0)
                # A second boot reuses exactly that state; nothing else is accepted.
                with probe.backend(report, state.parent):
                    command = start.call_args.args[0]
                    self.assertEqual(command[command.index("--tpmstate") + 1], state_option)
                for other in (root / "build/tpm/missing", root):
                    with self.subTest(other=other), self.assertRaisesRegex(ValueError, "Reused TPM state"):
                        with probe.backend(report, other):
                            pass


if __name__ == "__main__":
    unittest.main()
