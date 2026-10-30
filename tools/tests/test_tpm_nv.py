"""Synthetic harness checks only; these never count as a native TPM run."""
import copy
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import test_tpm_nv as probe
from tools.tests import test_tpm


def transcript(case):
    base = test_tpm.NativeTpmTranscriptTests().transcript("normal")
    extra = ["TPMNV:PUBLIC:PASS", f"TPMNV:PASS:{case}", f"TPMNV:BEFORE:{15:020d}"]
    if case != "read":
        extra.append(f"TPMNV:AFTER:{16 if case == 'provision' else 15:020d}")
    if case in ("auth", "missing"):
        extra.append(f"TPMNV:EXPECTED_RC:{0x98e if case == 'auth' else 0x18b:020d}")
    return base + "\n".join(extra) + "\n"


class NvVerdicts(unittest.TestCase):
    def test_exact_counter_relations_and_canonical_rejections(self):
        for case in probe.CASES:
            good = transcript(case)
            self.assertEqual(probe.evaluate(good, case)["BEFORE"], 15)
            for bad in (
                good.replace("TPMNV:PUBLIC:PASS\n", ""),
                good + f"TPMNV:BEFORE:{15:020d}\n",
                good.replace(f"TPMNV:BEFORE:{15:020d}", "TPMNV:BEFORE:15"),
                good.replace(f"TPMNV:PASS:{case}", "TPMNV:PASS:other"),
                good.replace(f"TPMNV:BEFORE:{15:020d}", f"TPMNV:BEFORE:{2**64:020d}"),
            ):
                with self.subTest(case=case), self.assertRaises(ValueError):
                    probe.evaluate(bad, case)
        for case in ("provision", "auth", "missing"):
            bad = transcript(case).replace("TPMNV:AFTER:", "TPMNV:BEFORE:")
            with self.assertRaises(ValueError):
                probe.evaluate(bad, case)
        for case in ("auth", "missing"):
            good = transcript(case)
            rc = 0x98e if case == "auth" else 0x18b
            with self.assertRaises(ValueError):
                probe.evaluate(good.replace(f"{rc:020d}", f"{0:020d}"), case)

    def test_cold_stages_require_distinct_children_same_state_and_counter(self):
        stages = [{"case": case, "guest": {"passed": True},
                   "backend": {"exit_status": 0, "state_directory": "private/state",
                               "directory": f"private/child-{case}"},
                   "observed": {"BEFORE": 15 if case == "provision" else 16, "AFTER": 16}}
                  for case in probe.CASES]
        probe.reconcile(stages)
        for change in (
            lambda s: s.pop(),
            lambda s: s[1]["backend"].update(directory=s[0]["backend"]["directory"]),
            lambda s: s[1]["backend"].update(state_directory="other"),
            lambda s: s[1]["backend"].update(exit_status=1),
            lambda s: s[1]["guest"].update(passed=False),
            lambda s: s[1]["observed"].update(BEFORE=15),
        ):
            broken = copy.deepcopy(stages)
            change(broken)
            with self.assertRaises(ValueError):
                probe.reconcile(broken)

    def test_explicit_state_reuse_preserves_private_original_state(self):
        backend = probe.run_tpm
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / "local/swtpm-native/usr/bin/swtpm"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"synthetic executable identity")
            state = root / "build/tpm/original/state"
            state.mkdir(parents=True, mode=0o700)
            (state / "persistent").write_bytes(b"synthetic state, not a TPM")
            with patch.object(backend, "ROOT", root), \
                    patch.object(backend, "LOCAL", root / "local"), \
                    patch.object(backend, "BUILD", root / "build"), \
                    patch.object(backend, "environment", return_value={"PATH": ""}), \
                    patch.object(backend.subprocess, "check_output", return_value="swtpm mock"), \
                    patch.object(backend.subprocess, "Popen") as start, \
                    patch.object(Path, "is_socket", return_value=True):
                child = start.return_value
                child.poll.return_value = None
                child.wait.return_value = 0
                report = {}
                with backend.backend(report, state_directory=state) as control:
                    command = start.call_args.args[0]
                    self.assertEqual(command[command.index("--tpmstate") + 1],
                                     "dir=build/tpm/original/state")
                    self.assertNotEqual(control.parent, state.parent)
                    self.assertEqual(report["backend"]["state_directory"], str(state))
                child.terminate.assert_called_once_with()
                self.assertEqual((state / "persistent").read_bytes(),
                                 b"synthetic state, not a TPM")
                start.reset_mock()
                for invalid in (root, root / "outside", state / "persistent"):
                    with self.assertRaises((ValueError, FileNotFoundError)):
                        with backend.backend({}, state_directory=invalid):
                            self.fail("Invalid state accepted")
                state.chmod(0o755)
                with self.assertRaisesRegex(ValueError, "private"):
                    with backend.backend({}, state_directory=state):
                        self.fail("Public state accepted")
                state.chmod(0o700)
                link = root / "build/tpm/link"
                link.symlink_to(state, target_is_directory=True)
                with self.assertRaisesRegex(ValueError, "symlink"):
                    with backend.backend({}, state_directory=link):
                        self.fail("Symlink accepted")
                (state / "persistent").unlink()
                with self.assertRaisesRegex(ValueError, "nonempty"):
                    with backend.backend({}, state_directory=state):
                        self.fail("Empty state accepted")
                start.assert_not_called()


if __name__ == "__main__":
    unittest.main()
