"""HOST_ONLY RTC verifier counterexamples; fixtures are not native evidence."""
from datetime import datetime, timezone
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_rtc as gate
from run import run_guest


def fixture(base):
    date = datetime.strptime(base, "%Y-%m-%dT%H:%M:%S").replace(tzinfo=timezone.utc)
    fields = ",".join(str(n) for n in (date.year, date.month, date.day,
                                      date.hour, date.minute, date.second))
    return "\n".join((*gate.BOOT, f"RTC:UTC:{int(date.timestamp())}",
                      "RTC:DATE:" + fields, *gate.END)) + "\n"


class RtcVerifier(unittest.TestCase):
    def test_calendar_cases_have_independent_epoch_oracles(self):
        for case, base in gate.CASES.items():
            with self.subTest(case=case):
                result = gate.evaluate(fixture(base), 33, False, base)
                self.assertEqual(result["quality"], "UnauthenticatedRtc")
        self.assertEqual(gate.evaluate(fixture(gate.CASES["epoch"]), 33, False,
                                       gate.CASES["epoch"])["unix_utc"], 0)

    def test_exit_timeout_missing_and_duplicate_markers_are_rejected(self):
        base = gate.CASES["current"]
        text = fixture(base)
        for raw, timeout in [(35, False), (0, False), (None, False), (33, True)]:
            with self.subTest(raw=raw, timeout=timeout), self.assertRaises(ValueError):
                gate.evaluate(text, raw, timeout, base)
        for marker in gate.REQUIRED:
            with self.subTest(marker=marker), self.assertRaises(ValueError):
                gate.evaluate(text.replace(marker + "\n", ""), 33, False, base)
        for altered in [text + "RTC:PASS\n", text + "M1:FAIL:RTC\n",
                        "RTC:PASS\n" + text, "BOOT:EBS_OK\n" + text,
                        text.replace("RTC:RESTORE:PASS", "RTC:PASS")]:
            with self.assertRaises(ValueError):
                gate.evaluate(altered, 33, False, base)

    def test_wrong_date_timestamp_quality_and_host_time_do_not_pass(self):
        base = gate.CASES["epoch"]
        text = fixture(base)
        for old, new in [("RTC:UTC:0", "RTC:UTC:1"),
                         ("1970,1,1", "1970,2,30"),
                         ("UNAUTHENTICATED", "AUTHENTICATED"),
                         ("RTC:UTC:0", "RTC:UTC:-1"),
                         ("RTC:DATE:", "RTC:DATE:bad")]:
            with self.subTest(old=old), self.assertRaises(ValueError):
                gate.evaluate(text.replace(old, new), 33, False, base)
        with self.assertRaises(ValueError):
            gate.evaluate(fixture(gate.CASES["current"]), 33, False, base)

    def test_vm_elapsed_calendar_window_has_exact_boundaries(self):
        base = gate.CASES["epoch"]
        for seconds, accepted in [(0, True), (60, True), (61, False)]:
            date = datetime.fromtimestamp(seconds, timezone.utc).strftime("%Y-%m-%dT%H:%M:%S")
            if accepted:
                gate.evaluate(fixture(date), 33, False, base)
            else:
                with self.assertRaises(ValueError):
                    gate.evaluate(fixture(date), 33, False, base)

    def test_bad_rtc_option_is_refused_before_any_guest_or_firmware_access(self):
        for base in ["2026-1-01T00:00:00", "2026-02-30T00:00:00",
                     "2026-01-01T00:00:00,clock=host", "now", "2026-01-01"]:
            with self.subTest(base=base), patch("run.firmware") as firmware:
                with self.assertRaises(ValueError):
                    run_guest(rtc_base=base)
                firmware.assert_not_called()

    def test_commandline_only_adds_vm_clock_for_explicit_calendar(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            build = root / "build"
            esp = build / "esp"
            esp.mkdir(parents=True)
            (esp / "kernel.elf").write_bytes(b"HOST_ONLY fake kernel")
            code, variables = root / "code.fd", root / "vars.fd"
            code.write_bytes(b"HOST_ONLY firmware")
            variables.write_bytes(b"HOST_ONLY vars")
            for base in (None, gate.CASES["current"]):
                guest = subprocess.CompletedProcess([], 33, fixture(gate.CASES["current"]).encode(), b"")
                git = subprocess.CompletedProcess([], 0, "fake-commit", "")
                with patch("run.ROOT", root), patch("run.BUILD", build), \
                     patch("run.firmware", return_value=(code, variables)), \
                     patch("run.environment", return_value={"PATH": ""}), \
                     patch("run.shutil.which", return_value=str(root / "qemu")), \
                     patch("run.source_hash", return_value="f" * 64), \
                     patch("run.subprocess.check_output", return_value="HOST_ONLY QEMU"), \
                     patch("run.subprocess.run", side_effect=[guest, git]) as execute:
                    report = run_guest(rtc_base=base, required_markers=gate.REQUIRED)
                    command = execute.call_args_list[0].args[0]
                    self.assertTrue(report["passed"])
                    if base is None:
                        self.assertNotIn("-rtc", command)
                    else:
                        self.assertEqual(command[command.index("-rtc") + 1],
                                         f"base={base},clock=vm,driftfix=none")


class RtcProvenance(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.build = Path(self.temporary.name)
        self.source = "f" * 64
        self.manifest = {"kernel_feature": "native-rtc", "loader_feature": None,
                         "source_fingerprint_version": 3, "source_sha256": self.source,
                         "kernel": {"sha256": "a" * 64}, "loader": {"sha256": "b" * 64}}
        self.write_manifest(self.manifest)
        self.addCleanup(patch.stopall)
        patch.object(gate, "BUILD", self.build).start()
        self.hash_source = patch.object(gate, "source_hash", return_value=self.source).start()

    def write_manifest(self, manifest):
        (self.build / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")

    def guest(self):
        return {"stdout": fixture(gate.CASES["current"]), "raw_qemu_exit": 33,
                "timed_out": False, "passed": True, "source_sha256": self.source,
                "artifact_binding_passed": True}

    def test_stale_success_is_removed_before_manifest_source_or_profile_rejection(self):
        for field, value in [("kernel_feature", "milestone-m2"), ("loader_feature", "test-ebs"),
                             ("source_fingerprint_version", 2), ("source_sha256", "stale")]:
            with self.subTest(field=field), patch.object(gate, "run_guest") as boot:
                self.write_manifest({**self.manifest, field: value})
                (self.build / "rtc-report.json").write_text('{"passed":true}', encoding="utf-8")
                with self.assertRaises(ValueError):
                    gate.run(("current",))
                self.assertFalse((self.build / "rtc-report.json").exists())
                boot.assert_not_called()

    def test_each_boot_binds_both_artifacts_and_the_explicit_vm_base(self):
        with patch.object(gate, "run_guest", return_value=self.guest()) as boot:
            result = gate.run(("current",))
            self.assertTrue(result["passed"])
            self.assertEqual(boot.call_args.kwargs["expected_kernel_sha256"], "a" * 64)
            self.assertEqual(boot.call_args.kwargs["expected_loader_sha256"], "b" * 64)
            self.assertEqual(boot.call_args.kwargs["rtc_base"], gate.CASES["current"])
            self.assertEqual(result["source_sha256"], self.source)
            self.assertEqual(result["manifest_sha256"], gate.sha256(self.build / "manifest.json"))

    def test_failure_drift_and_bad_artifact_binding_never_publish_success(self):
        for change in ({"passed": False}, {"source_sha256": "other"},
                       {"artifact_binding_passed": False}, {"raw_qemu_exit": 35}):
            with self.subTest(change=change), \
                 patch.object(gate, "run_guest", return_value={**self.guest(), **change}):
                with self.assertRaises(ValueError):
                    gate.run(("current",))
                report = json.loads((self.build / "rtc-report.json").read_text())
                self.assertFalse(report["passed"])
                self.assertFalse(report["cases"][0]["passed"])

        def mutate_manifest(*args, **kwargs):
            self.write_manifest({**self.manifest, "changed": True})
            return self.guest()

        with patch.object(gate, "run_guest", side_effect=mutate_manifest):
            with self.assertRaises(ValueError):
                gate.run(("current",))
        self.write_manifest(self.manifest)
        self.hash_source.side_effect = [self.source, "changed"]
        with patch.object(gate, "run_guest") as boot:
            with self.assertRaises(ValueError):
                gate.run(("current",))
            boot.assert_not_called()


if __name__ == "__main__":
    unittest.main()
