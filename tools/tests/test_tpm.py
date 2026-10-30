import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_tpm as probe
from tools.tests import test_ipc_wait


class NativeTpmTranscriptTests(unittest.TestCase):
    def transcript(self, case):
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

    def test_case_substitution_and_malformed_evidence_are_rejected(self):
        for case in probe.FEATURES:
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


if __name__ == "__main__":
    unittest.main()
