import hashlib
import os
import re
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_tpm as probe
import tpm_seed
from tools.tests import test_ipc_wait


class NativeTpmTranscriptTests(unittest.TestCase):
    LOADER = ("LOADER-TPM:TCG2:PRESENT\nLOADER-TPM:BANKS:000000000000000f\n"
              "LOADER-TPM:PERMANENT:0000000000000400\nLOADER-TPM:PCR13:ZERO\n"
              "LOADER-TPM:PASS\nLOADER-TPM:RNG:ABSENT\n")

    HAUTH = ("LOADER-HAUTH:BEGIN\nLOADER-HAUTH:PERMANENT:{permanent:016x}\n{mode}\n"
             "LOADER-HAUTH:BOUND\n{lockout}\nLOADER-HAUTH:PCR13:CLOSED\n"
             + "".join(f"{line}\n" for line in probe.closed_banks(0xF) + probe.closed_events(0xF)) +
             "LOADER-HAUTH:EVENTLOG:CLOSED\n"
             "LOADER-HAUTH:READ:REFUSED\nLOADER-HAUTH:OWNER:REFUSED\n"
             "LOADER-HAUTH:DA:{da:016x}:{da:016x}\n"
             "LOADER-HAUTH:RESIDUE:NONE\nLOADER-HAUTH:PASS\n")

    def transcript(self, case, boot=0):
        if (case, boot) in probe.HALT:
            bits, steps = probe.HALT[case, boot]
            return (self.LOADER.replace("RNG:ABSENT", f"RNG:{probe.LOADER_RNG[case]}")
                    + f"LOADER-HAUTH:BEGIN\nLOADER-HAUTH:PERMANENT:{0x400 | bits:016x}\n"
                    + "".join(f"{step}\n" for step in steps))
        if case in probe.PROVISIONED:
            mode, lockout, bits = probe.hauth_mode(case, boot)
            hauth = self.HAUTH.format(permanent=0x400 | bits, mode=mode, lockout=lockout,
                                      da=int(case in probe.DA_SEEDED))
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
        for case in (case for case in probe.FEATURES if (case, 0) not in probe.HALT):
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
        events = probe.closed_events(0xF)
        for bad, boot in (
            (good, 1),
            (self.transcript("loader-hauth", 1), 0),
            (good + "LOADER-HAUTH:MODE:EXISTING\n", 0),
            (good.replace("LOADER-HAUTH:RESIDUE:NONE\n", ""), 0),
            (good.replace("READ:REFUSED", "READ:ALLOWED"), 0),
            (good.replace("LOADER-HAUTH:PCR13:CLOSED\n", ""), 0),
            (good.replace("HAUTH:PERMANENT:0000000000000400", "HAUTH:PERMANENT:0000000000000401"), 0),
            (good.replace("RNG:PASS", "RNG:ABSENT"), 0),
            (good.replace("DA:0000000000000000:0000000000000000", "DA:0000000000000000:0000000000000001"), 0),
            (good.replace("LOADER-HAUTH:DA:0000000000000000:0000000000000000\n", ""), 0),
            (good + "LOADER-HAUTH:HALT:PCR13:CLOSED\n", 0),
            (good, 2),
            # Every active bank must hold H(0^n || H(CLOSED)), once and in order.
            (good.replace(":f6e96e2d", ":f6e96e2e"), 0),
            (good.replace(probe.closed_banks(0xF)[0] + "\n", ""), 0),
            (good.replace(probe.closed_banks(0xF)[3], probe.closed_banks(0x7)[3]), 0),
            (good.replace(probe.closed_banks(0xF)[1] + "\n", "") + probe.closed_banks(0xF)[1] + "\n", 0),
            (good + probe.closed_banks(0xF)[2] + "\n", 0),
            (good.replace("BANKS:000000000000000f", "BANKS:000000000000000b"), 0),
            # The log's last entry must hold H(CLOSED) for exactly the active banks, in order.
            (good.replace("LOADER-HAUTH:EVENTLOG:CLOSED\n", ""), 0),
            (good.replace(events[0] + "\n", ""), 0),
            (good.replace(events[1], events[1][:-4] + "0000"), 0),
            (good.replace(events[2], probe.closed_banks(0xF)[2].replace("PCR13:BANK", "EVENTLOG:DIGEST")), 0),
            (good.replace(events[0] + "\n" + events[1], events[1] + "\n" + events[0]), 0),
            (good.replace("EVENTLOG:CLOSED\n", f"EVENTLOG:CLOSED\n{events[3]}\n"), 0),
        ):
            with self.subTest(bad=bad, boot=boot), self.assertRaises(ValueError):
                probe.evaluate(bad, "loader-hauth", boot)
        for case in ("loader-rng", "normal"):
            with self.subTest(case=case), self.assertRaises(ValueError):
                probe.evaluate(good, case)
        # An inactive bank is reported, not read; the expected values follow trial_tpm.rs.
        inactive = good.replace("BANKS:000000000000000f", "BANKS:000000000000000b").replace(
            probe.closed_banks(0xF)[2], probe.closed_banks(0xB)[2])
        with self.assertRaises(ValueError):  # the log must not name the inactive bank
            probe.evaluate(inactive, "loader-hauth")
        inactive = inactive.replace(events[2] + "\n", "")
        self.assertEqual(probe.closed_events(0xB), [events[0], events[1], events[3]])
        self.assertEqual(probe.evaluate(inactive, "loader-hauth")["LOADER_BANKS"], 0xB)
        self.assertTrue(probe.closed_banks(0xB)[2].endswith(":INACTIVE"))
        source = (Path(__file__).resolve().parents[2] / "crates/update-core/src/trial_tpm.rs").read_text()
        self.assertIn(f'EVENT_CLOSED: &[u8] = b"{probe.EVENT_CLOSED.decode()}";', source)
        closed = re.search(r'P_CLOSED: \[u8; 32\] =\s*h\("([0-9a-f]{64})"\)', source)
        self.assertEqual(probe.closed_banks(0x2)[1].rsplit(":", 1)[1], closed.group(1))
        digest = re.search(r'DIGEST_CLOSED: \[u8; 32\] =\s*h\("([0-9a-f]{64})"\)', source)
        self.assertEqual(probe.closed_events(0x2), [f"LOADER-HAUTH:EVENTLOG:DIGEST:{0xB:016x}:{digest.group(1)}"])

    def test_loader_halt_stops_after_the_closed_extend_and_before_the_kernel(self):
        good = self.transcript("loader-hauth-no-rng")
        result = probe.evaluate(good, "loader-hauth-no-rng")
        self.assertEqual((result["HAUTH_PERMANENT"], result["LOADER_RNG"]), (0x400, "ABSENT"))
        self.assertEqual(probe.markers("loader-hauth-no-rng")[-1],
                         "LOADER-HAUTH:FAIL:RNG_ABSENT:0000000000000000")
        self.assertEqual((probe.HALT_EXIT, probe.LOADER_RNG["loader-hauth-no-rng"]), (35, "ABSENT"))
        for bad, boot in (
            (good + self.transcript("normal"), 0),
            (good + "KERNEL:ENTRY\n", 0),
            (good + "M1:PASS\n", 0),
            (good.replace("LOADER-HAUTH:MODE:PROVISION\n", "LOADER-HAUTH:MODE:PROVISION\nLOADER-HAUTH:BOUND\n"), 0),
            (good.replace("HALT:PCR13:CLOSED", "HALT:PCR13:OTHER"), 0),
            (good.replace("LOADER-HAUTH:HALT:PCR13:CLOSED\n", ""), 0),
            (good.replace("FAIL:RNG_ABSENT", "FAIL:RNG_ZERO"), 0),
            (good + "LOADER-HAUTH:PASS\n", 0),
            (good.replace("RNG:ABSENT", "RNG:PASS"), 0),
            (good.replace("HAUTH:PERMANENT:0000000000000400", "HAUTH:PERMANENT:0000000000000405"), 0),
            (good, 1),
        ):
            with self.subTest(bad=bad, boot=boot), self.assertRaises(ValueError):
                probe.evaluate(bad, "loader-hauth-no-rng", boot)
        for case in ("loader-hauth", "loader-probe", "normal"):
            with self.subTest(case=case), self.assertRaises(ValueError):
                probe.evaluate(good, case)
        with self.assertRaises(ValueError):
            probe.evaluate(self.transcript("loader-hauth"), "loader-hauth-no-rng")

    def test_pre_seeded_cases_pin_the_boot_that_halts_and_its_exact_failure(self):
        # Boots before the seed, the repaired cut and the replaced empty-ownerAuth squat behave
        # exactly like loader-hauth; the DA case differs only in its nonzero counter.
        for case, boot in (("loader-hauth-unbound", 0), *((cut, boot) for cut in probe.CUTS for boot in (0, 1)),
                           ("loader-hauth-owner-squat-empty", 0), ("loader-hauth-owner-squat-empty", 1),
                           ("loader-hauth-da-counted", 0)):
            result = probe.evaluate(self.transcript(case, boot), case, boot)
            self.assertEqual(result["HAUTH_PERMANENT"] & 0x5, 5 if boot else 0)
            self.assertEqual(result["HAUTH_DA_COUNTER"], int(case in probe.DA_SEEDED))
            self.assertEqual(probe.markers(case, boot), probe.markers("loader-hauth", boot))
        moved = self.transcript("loader-hauth-da-counted")
        for bad, case in ((self.transcript("loader-hauth"), "loader-hauth-da-counted"),
                          (moved, "loader-hauth"), (moved, "loader-hauth-owner-squat-empty"),
                          (moved.replace("0000000000000001\n", "0000000000000002\n"), "loader-hauth-da-counted")):
            with self.subTest(case=case, bad=bad), self.assertRaises(ValueError):
                probe.evaluate(bad, case)
        for case in ("loader-hauth-squat", "loader-hauth-foreign-owner", "loader-hauth-owner-squat",
                     "loader-hauth-da-counted"):
            with self.subTest(case=case), self.assertRaises(ValueError):
                probe.evaluate(self.transcript(case), case, 1)
        failures = {steps[-1] for _, steps in probe.HALT.values()}
        for (case, boot), (bits, steps) in probe.HALT.items():
            good = self.transcript(case, boot)
            self.assertEqual(probe.evaluate(good, case, boot)["HAUTH_PERMANENT"], 0x400 | bits)
            mode = probe.EXISTING if steps[0] == probe.PROVISION else probe.PROVISION
            for bad in (
                good.replace(steps[0], mode),
                good.replace(f"HAUTH:PERMANENT:{0x400 | bits:016x}", f"HAUTH:PERMANENT:{0x400 | bits ^ 1:016x}"),
                good.replace(f"{probe.CLOSED}\n", ""),
                good + "KERNEL:ENTRY\n",
                *(good.replace(steps[-1], other) for other in failures - {steps[-1]}),
            ):
                with self.subTest(case=case, bad=bad), self.assertRaises(ValueError):
                    probe.evaluate(bad, case, boot)
            for other in set(probe.HALT) - {(case, boot)}:
                with self.subTest(case=case, other=other), self.assertRaises(ValueError):
                    probe.evaluate(good, *other)
        self.assertEqual(probe.markers("loader-hauth-unbound", 1)[-1], "LOADER-HAUTH:FAIL:UNBOUND:0000000000000000")
        self.assertEqual(probe.markers("loader-hauth-unbound", 0), probe.markers("loader-hauth", 0))

    def test_lockout_cut_only_sets_lockout_auth_then_verifies_like_loader_hauth(self):
        case = "loader-hauth-lockout-cut"
        repaired = self.transcript(case)
        result = probe.evaluate(repaired, case)
        self.assertEqual((result["HAUTH_PERMANENT"], result["HAUTH_DA_COUNTER"]), (0x401, 0))
        self.assertEqual(probe.evaluate(self.transcript(case, 1), case, 1)["HAUTH_PERMANENT"], 0x405)
        self.assertEqual(probe.markers(case, 1), probe.markers("loader-hauth", 1))
        self.assertEqual(set(probe.markers(case)) - set(probe.markers("loader-hauth")), {probe.EXISTING})
        for bad, other, boot in (
            (self.transcript("loader-hauth"), case, 0),
            (self.transcript("loader-hauth", 1), case, 0),
            (repaired, "loader-hauth", 0), (repaired, "loader-hauth", 1), (repaired, case, 1),
            (repaired.replace(probe.SET, probe.KEPT), case, 0),
            (repaired + probe.KEPT + "\n", case, 0),
            *((repaired.replace("HAUTH:PERMANENT:0000000000000401", f"HAUTH:PERMANENT:{value:016x}"), case, 0)
              for value in (0x400, 0x405)),
        ):
            with self.subTest(other=other, boot=boot, bad=bad), self.assertRaises(ValueError):
                probe.evaluate(bad, other, boot)

    def test_public_state_around_seeded_boots(self):
        public = {"hauth": {"attributes": 1, "name": "aa"}, "lrec": {"attributes": 2, "name": "bb", "sha256": "11"}}
        cut = {"permanent": 0x401, "lockout_counter": 0, **public}
        repaired = cut | {"permanent": 0x405}
        probe.check_state("loader-hauth-lockout-cut", 0, {"seeded": cut, "after": repaired})
        stale = cut | {"permanent": 0x400}
        fresh = stale | {"permanent": 0x405, "lrec": public["lrec"] | {"sha256": "22"}}
        probe.check_state("loader-hauth-cut-lrec-written", 0, {"seeded": stale, "after": fresh})
        probe.check_state("loader-hauth-cut-hauth-defined", 0, {"seeded": stale | {"lrec": {"rc": 0x18B}}, "after": fresh})
        probe.check_state("loader-hauth-squat", 0, {"seeded": stale, "after": stale})
        probe.check_state("loader-hauth-cut-lrec-written", 1, {"after": fresh})
        for case, boot, seeded, after in (
            ("loader-hauth-lockout-cut", 0, cut, cut),
            ("loader-hauth-lockout-cut", 0, cut, repaired | {"lockout_counter": 1}),
            ("loader-hauth-lockout-cut", 0, cut, fresh | {"permanent": 0x405}),
            ("loader-hauth-cut-lrec-written", 0, stale, stale | {"permanent": 0x405}),
            ("loader-hauth-cut-hauth-written", 0, stale, fresh | {"lrec": {"rc": 0x18B}}),
            ("loader-hauth-squat", 0, stale, fresh),
            ("loader-hauth-unbound", 1, repaired, fresh),
        ):
            with self.subTest(case=case, after=after), self.assertRaises(ValueError):
                probe.check_state(case, boot, {"seeded": seeded, "after": after})

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


@unittest.skipUnless(shutil.which("swtpm"), "NOT RUN: swtpm absent")
class TpmSeedTests(unittest.TestCase):
    """The seeds build the frozen trial_tpm templates and read back only public state."""

    def setUp(self):
        source = (Path(__file__).resolve().parents[2] / "crates/update-core/src/trial_tpm.rs").read_text()
        self.names = {name: "000b" + digest for name, digest in re.findall(
            r'pub const (NAME_\w+): \[u8; 34\] = name\(h\(\s*"([0-9a-f]{64})"', source)}
        self.assertEqual(set(self.names), {"NAME_HAUTH", "NAME_HAUTH_WRITTEN", "NAME_LREC_WRITTEN"})
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name)
        (self.directory / "state").mkdir()
        self.tool = (shutil.which("swtpm"), dict(os.environ))

    def test_each_seed_produces_the_documented_public_state(self):
        cut = tpm_seed.seed(*self.tool, self.directory, "cut")
        self.assertEqual((cut["permanent"] & 0x5, cut["hauth"]["name"]), (0, self.names["NAME_HAUTH_WRITTEN"]))
        self.assertFalse(cut["lrec"]["attributes"] & tpm_seed.WRITTEN)
        self.assertNotIn("sha256", cut["lrec"])
        with tpm_seed.server(*self.tool, self.directory) as tpm:
            tpm.write_lrec(b"NOSLRC01" + bytes(40) + bytes(range(16)))
            before = tpm.lrec()
        flipped = tpm_seed.seed(*self.tool, self.directory, "unbound")
        self.assertEqual(flipped["lrec"]["name"], self.names["NAME_LREC_WRITTEN"])
        with tpm_seed.server(*self.tool, self.directory) as tpm:
            after = tpm.lrec()
        self.assertEqual([at for at in range(64) if before[at] != after[at]], [48])
        self.assertEqual(tpm_seed.snapshot(*self.tool, self.directory), flipped)
        fresh = self.directory / "fresh"
        squat = tpm_seed.AUTHWRITE | tpm_seed.AUTHREAD | tpm_seed.NO_DA
        for name, permanent, attributes in (("foreign-owner", 1, None),
                                            ("squat", 0, squat | tpm_seed.PLATFORMCREATE),
                                            ("owner-squat", 1, squat), ("owner-squat-empty", 0, squat),
                                            ("da-counted", 0, None)):
            with self.subTest(name=name):
                shutil.rmtree(fresh, ignore_errors=True)
                (fresh / "state").mkdir(parents=True)
                state = tpm_seed.seed(*self.tool, fresh, name)
                self.assertEqual((state["permanent"] & 0x5, state["lrec"]), (permanent, {"rc": 0x18B}))
                # Read on the seeding server itself: only the DA seed moves the counter, by one.
                self.assertEqual(state["lockout_counter"], int(name == "da-counted"))
                self.assertEqual(state["hauth"].get("attributes"), attributes)
                if attributes is None:
                    self.assertEqual(state["hauth"], {"rc": 0x18B})
                else:
                    self.assertNotIn(state["hauth"]["name"], self.names.values())

    def test_cut_points_leave_the_documented_public_state(self):
        source = (Path(__file__).resolve().parents[2] / "crates/update-core/src/trial.rs").read_text()
        self.assertIn(f'LREC_MAGIC: [u8; 8] = *b"{tpm_seed.LREC_MAGIC.decode()}";', source)
        self.assertIn(f"LREC_BINDING_LEN: usize = {tpm_seed.LREC_BINDING_LEN};", source)
        self.assertIn("sha256(hauth)[..LREC_BINDING_LEN]", source)
        for name, permanent, hauth, lrec in (("hauth-defined", 0, "NAME_HAUTH", None),
                                             ("hauth-written", 0, "NAME_HAUTH_WRITTEN", None),
                                             ("lrec-written", 0, "NAME_HAUTH_WRITTEN", "NAME_LREC_WRITTEN"),
                                             ("lockout-cut", 1, "NAME_HAUTH_WRITTEN", "NAME_LREC_WRITTEN")):
            with self.subTest(name=name):
                directory = self.directory / name
                (directory / "state").mkdir(parents=True)
                state = tpm_seed.seed(*self.tool, directory, name)
                self.assertEqual((state["permanent"] & 0x5, state["lockout_counter"], state["hauth"]["name"]),
                                 (permanent, 0, self.names[hauth]))
                if lrec is None:
                    self.assertEqual(state["lrec"], {"rc": 0x18B})
                    continue
                self.assertEqual(state["lrec"]["name"], self.names[lrec])
                with tpm_seed.server(*self.tool, directory) as tpm:
                    record = tpm.lrec()
                self.assertEqual(record[:48], tpm_seed.LREC_MAGIC + bytes(40))
                self.assertEqual(state["lrec"]["sha256"], hashlib.sha256(record).hexdigest())

if __name__ == "__main__":
    unittest.main()
