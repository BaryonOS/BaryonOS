"""CI-EVIDENCE: mandatory gate reports must survive compact collection.

Execute the checked-in Bash checks with isolated fixtures, without running
native gates or GitHub's expression engine or artifact service.
"""

from pathlib import Path
import subprocess
import tempfile
import unittest

import yaml


ROOT = Path(__file__).resolve().parents[1]


class RequiredGateEvidenceTests(unittest.TestCase):
    """Fail closed locally while keeping artifact upload outages best effort."""

    def test_mandatory_report_contract(self):
        """Reject absent, empty, and linked reports even with auxiliary logs."""
        for workflow, job, report in [
            ("storage-resilience", "s08", "build/ci-s08/s08-storage-resilience.json"),
            ("volume-acceptance", "m6e", "build/ci-m6e/volume-gate.json"),
        ]:
            config = yaml.load(
                (ROOT / f".github/workflows/{workflow}.yml").read_text(),
                Loader=yaml.BaseLoader,
            )
            steps = config["jobs"][job]["steps"]
            check = next(s for s in steps if s.get("name") == "Require mandatory gate report")
            preserve = next(s for s in steps if s.get("uses") == "./.github/actions/preserve-evidence")
            self.assertEqual(check["if"], "${{ always() }}")
            self.assertNotIn("continue-on-error", check)
            self.assertLess(steps.index(check), steps.index(preserve))
            self.assertEqual(preserve["if"], "${{ always() }}")
            self.assertEqual(preserve["with"]["if-no-files-found"], "error")
            self.assertEqual(preserve["with"]["required-paths"], report)
            self.assertIn(report, preserve["with"]["path"].splitlines())
            for case in ("absent", "diagnostics-only", "empty", "symlink", "present"):
                with self.subTest(workflow=workflow, case=case), tempfile.TemporaryDirectory() as directory:
                    root = Path(directory)
                    target = root / report
                    target.parent.mkdir(parents=True)
                    if case not in ("absent", "present"):
                        diagnostic = target.parent / "run/volume-normal"
                        diagnostic.mkdir(parents=True)
                        (diagnostic / "result.json").write_text('{}\n')
                        (diagnostic / "serial.txt").write_text('diagnostic fixture\n')
                    if case == "empty":
                        target.touch()
                    elif case == "symlink":
                        target.symlink_to("run/volume-normal/result.json")
                    elif case == "present":
                        target.write_text('{"passed":true}\n')
                    result = subprocess.run(
                        ["bash", "-e", "-o", "pipefail", "-c", check["run"]],
                        cwd=root, capture_output=True, text=True, check=False,
                    )
                    self.assertEqual(result.returncode, 0 if case == "present" else 1,
                                     result.stdout + result.stderr)
                    if case != "present":
                        self.assertIn("::error::Required gate report", result.stdout)

    def test_composite_action_passes_exact_required_paths_to_collector(self):
        """Keep required-path validation inside the bounded collector boundary."""
        action = yaml.load(
            (ROOT / ".github/actions/preserve-evidence/action.yml").read_text(),
            Loader=yaml.BaseLoader,
        )
        self.assertEqual(action["inputs"]["required-paths"]["default"], "")
        collect = next(step for step in action["runs"]["steps"]
                       if step.get("id") == "collect")
        self.assertEqual(
            collect["env"]["EVIDENCE_REQUIRED_PATHS"],
            "${{ inputs.required-paths }}",
        )
        self.assertIn(
            '--required-paths "$EVIDENCE_REQUIRED_PATHS"',
            collect["run"],
        )

    def test_upload_failures_remain_separate(self):
        """Only remote upload steps tolerate errors in evidence preservation."""
        action = yaml.load(
            (ROOT / ".github/actions/preserve-evidence/action.yml").read_text(),
            Loader=yaml.BaseLoader,
        )
        uploads = [s for s in action["runs"]["steps"]
                   if s.get("uses", "").startswith("actions/upload-artifact@")]
        self.assertTrue(uploads)
        for step in uploads:
            self.assertEqual(step["continue-on-error"], "true")


if __name__ == "__main__":
    unittest.main()
