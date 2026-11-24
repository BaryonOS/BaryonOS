"""CI-EVIDENCE: preservation must fail closed when its baseline is incomplete.

These fixtures execute the verifier and its bounded reporting simulations in
temporary Git repositories. They do not execute workflow jobs or upload data.
"""

import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
VERIFIER_PATH = ROOT / "docs/verification/ci-artifact-storage-20261108/verify_workflows.py"
SPEC = importlib.util.spec_from_file_location("ci_evidence_workflow_verifier", VERIFIER_PATH)
VERIFIER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFIER)


class WorkflowVerifierTests(unittest.TestCase):
    """Require real comparisons and reject workflow/collector regressions."""

    def setUp(self):
        """Copy the real workflow inputs into an isolated, disposable Git history."""
        self.temp = tempfile.TemporaryDirectory(prefix="ci-workflow-baseline-test-")
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        shutil.copytree(ROOT / ".github", self.repo / ".github")
        self.git("init", "--quiet")
        self.git("add", ".github")
        self.git("-c", "user.name=CI fixture", "-c", "user.email=fixture@example.invalid",
                 "commit", "--quiet", "-m", "Synthetic baseline for verifier host tests")
        self.baseline = self.git("rev-parse", "HEAD").strip()

    def git(self, *args):
        """Run fixed Git fixture operations without a shell or remote access."""
        return subprocess.check_output(["git", *args], cwd=self.repo, text=True,
                                       stderr=subprocess.STDOUT)

    def report(self):
        """Inspect the isolated workflow bytes against their committed baseline."""
        return VERIFIER.run(self.repo, self.baseline)

    def manual_compact(self, name):
        """Observe the specific policy check independently of baseline edits."""
        key = "manual-default-compact:.github/workflows/" + name
        return next(check["passed"] for check in self.report()["checks"]
                    if check["name"] == key)

    def test_integrated_workflows_preserve_all_reporting_cases(self):
        """A valid baseline retains 21 workflow files and 41 preservation calls."""
        report = self.report()
        self.assertTrue(report["passed"], report["failed_checks"])
        self.assertEqual(report["archive_workflow_count"], 21)
        self.assertEqual(len(report["archive_workflows"]), 21)
        for name in ("native-xhci.yml", "native-hid-input.yml", "native-hw-inventory.yml"):
            self.assertIn(".github/workflows/" + name, report["archive_workflows"])
        self.assertEqual(report["local_snapshot_baseline"], self.baseline)
        self.assertEqual(report["repository_path"], ".")
        self.assertTrue(all(comparison["status"] == "compared"
                            and comparison["remaining_configuration_preserved"]
                            for comparison in report["baseline_comparisons"].values()))
        for prefix in ("preservation-invocation:", "existing-preservation-step-unchanged:"):
            # Experimental SMP adds one call. KVM adds one workflow/call, while
            # H05 retains one workflow with distinct S5 and POWER-system calls.
            # The K06 section 9 matrix adds eight (probe, case, soak, aggregate,
            # final gate, clock diagnostic, margin diagnostic, gate rehearsal).
            # Code coverage reporting adds three calls in ci.yml (Rust, Python,
            # JavaScript), each counted by both prefixes.
            # H02 adds one xHCI and one HID workflow/preservation invocation.
            # Native network extensions add their own DHCP/link/route/sweep reports;
            # N02-T6 adds one libslirp evidence call.
            # Window gates retain six report/serial proof artifacts from six
            # static calls: host, leg proof, library cold, leg compact,
            # read-only and packaging collections.
            # H06 adds one HW-INV inventory workflow/preservation invocation.
            # The ci-impact-shadow job adds one plan call in ci.yml.
            self.assertEqual(sum(check["name"].startswith(prefix)
                                 for check in report["checks"]), 41)
        self.assertEqual(sum(check["name"].startswith("report-shell:")
                             for check in report["checks"]), 28)
        self.assertEqual(sum(check["name"].startswith("package-check:")
                             for check in report["checks"]), 6)

    def test_only_the_library_cold_proof_may_be_leg_conditional(self):
        """The exact exception cannot suppress base, final, other legs' or another job's evidence."""
        workflow = ".github/workflows/native-window-session.yml"
        step = {"id": "window-library-cold-proof", "name": "Preserve complete library cold proof",
                "if": "${{ always() && matrix.mode == 'library' }}"}
        self.assertTrue(VERIFIER.preservation_condition(workflow, "native-window-session", step))
        for path, job, changed in (
            (".github/workflows/native-display-input.yml", "native-window-session", step),
            (workflow, "another-job", step),
            (workflow, "normal-and-package", step),
            (workflow, "native-window-session", dict(step, name="Different step")),
            (workflow, "native-window-session", dict(step, id="window-native-proof")),
            (workflow, "native-window-session", dict(step, id="window-library-write-proof")),
            (workflow, "native-window-session", dict(step, id="window-switcher-scale2-proof")),
            (workflow, "native-window-session", dict(step, id="final")),
            (workflow, "native-window-session", dict(step, **{"if": "${{ success() && matrix.mode == 'library' }}"})),
            (workflow, "native-window-session", dict(step, **{"if": "${{ always() && matrix.mode == 'native' }}"})),
            (workflow, "native-window-session", dict(step, **{"if": "${{ always() && matrix.scale == 1 }}"})),
        ):
            with self.subTest(path=path, job=job, step=changed):
                self.assertFalse(VERIFIER.preservation_condition(path, job, changed))

    def test_unconditional_window_collections_cannot_be_restricted(self):
        """Run the actual verifier on attempts to restrict either unconditional caller."""
        path = self.repo / ".github/workflows/native-window-session.yml"
        original = path.read_text()
        for name, occurrence in (("Preserve complete base native proof", 1),
                                 # Host, native-leg, read-only and packaging collections.
                                 ("Preserve compact evidence and report archive availability", 1),
                                 ("Preserve compact evidence and report archive availability", 2),
                                 ("Preserve compact evidence and report archive availability", 3),
                                 ("Preserve compact evidence and report archive availability", 4)):
            with self.subTest(name=name, occurrence=occurrence):
                parts = original.split("      - name: " + name)
                self.assertEqual(len(parts), 2 if "base" in name else 5)
                before = ("      - name: " + name).join(parts[:occurrence])
                block = ("      - name: " + name).join(parts[occurrence:])
                # Mutate only this step's own condition, never a later step's.
                step, marker, rest = block.partition("\n      - name: ")
                self.assertIn("if: ${{ always() }}", step)
                step = step.replace("if: ${{ always() }}", "if: ${{ always() && matrix.mode == 'library' }}", 1)
                path.write_text(before + "      - name: " + name + step + marker + rest)
                report = self.report()
                self.assertFalse(report["passed"])
                self.assertIn("preservation-invocation:.github/workflows/native-window-session.yml",
                              report["failed_checks"])
        path.write_text(original)

    def test_compact_only_accepts_empty_or_absent_manual_trigger(self):
        """No full-evidence selector is needed when every call fixes it false."""
        name = "native-hid-input.yml"
        path = self.repo / ".github/workflows" / name
        original = path.read_text()
        for trigger in ("  workflow_dispatch: {}\n", "  workflow_dispatch:\n", ""):
            with self.subTest(trigger=trigger):
                path.write_text(original.replace("  workflow_dispatch: {}\n", trigger, 1))
                self.assertTrue(self.manual_compact(name))

    def test_compact_only_rejects_implicit_dynamic_or_true_full_evidence(self):
        """Action defaults and expressions cannot stand in for fixed false."""
        name = "native-hid-input.yml"
        path = self.repo / ".github/workflows" / name
        original = path.read_text()
        for value in ("", "          full-evidence: 'true'\n",
                      "          full-evidence: ${{ false }}\n"):
            with self.subTest(value=value):
                path.write_text(original.replace("          full-evidence: 'false'\n", value, 1))
                self.assertFalse(self.manual_compact(name))

    def test_compact_only_requires_every_preservation_invocation_to_be_fixed(self):
        """A second call cannot silently enable the full archive."""
        name = "native-hid-input.yml"
        path = self.repo / ".github/workflows" / name
        original = path.read_text()
        extra = original[original.index("      - name: Preserve compact source-bound reports only\n"):]
        path.write_text(original + extra)
        self.assertTrue(self.manual_compact(name))
        path.write_text(original + extra.replace("full-evidence: 'false'", "full-evidence: 'true'", 1))
        self.assertFalse(self.manual_compact(name))

    def test_explicit_selector_still_requires_false_default_even_for_fixed_calls(self):
        """Fixed calls must not bypass an explicitly unsafe or missing default."""
        name = "native-hid-input.yml"
        path = self.repo / ".github/workflows" / name
        original = path.read_text()
        header = ("  workflow_dispatch:\n    inputs:\n      full_evidence:\n"
                  "        description: Full artifact selector\n        type: boolean\n")
        for default, expected in (("        default: false\n", True),
                                  ("        default: true\n", False), ("", False)):
            with self.subTest(default=default):
                path.write_text(original.replace("  workflow_dispatch: {}\n", header + default, 1))
                self.assertEqual(self.manual_compact(name), expected)

    def test_new_workflow_without_baseline_is_not_reported_as_preserved(self):
        """A newly added workflow cannot inherit an absent comparison as a pass."""
        path = ".github/workflows/new-workflow.yml"
        shutil.copyfile(self.repo / ".github/workflows/copilot-setup-steps.yml", self.repo / path)
        report = self.report()
        self.assertFalse(report["passed"])
        self.assertIn("workflow-inventory-preserved", report["failed_checks"])
        self.assertIn("original-tests-runners-gates-preserved:" + path, report["failed_checks"])
        self.assertEqual(report["baseline_comparisons"][path]["status"], "not-compared")

    def test_deleted_workflow_is_not_silently_ignored(self):
        """Comparing only surviving files must not hide deletion of a CI job."""
        (self.repo / ".github/workflows/copilot-setup-steps.yml").unlink()
        report = self.report()
        self.assertFalse(report["passed"])
        inventory = next(check for check in report["checks"]
                         if check["name"] == "workflow-inventory-preserved")
        self.assertIn(".github/workflows/copilot-setup-steps.yml", inventory["removed"])
        self.assertEqual(report["baseline_comparisons"][".github/workflows/copilot-setup-steps.yml"]["status"],
                         "missing-from-checkout")

    def test_runner_change_is_not_hidden_by_evidence_normalization(self):
        """The artifact comparison does not authorize a change back to self-hosting."""
        path = self.repo / ".github/workflows/ci.yml"
        text = path.read_text()
        self.assertIn("runs-on: ubuntu-24.04", text)
        path.write_text(text.replace("runs-on: ubuntu-24.04", "runs-on: self-hosted", 1))
        report = self.report()
        self.assertFalse(report["passed"])
        self.assertIn("original-tests-runners-gates-preserved:.github/workflows/ci.yml",
                      report["failed_checks"])

    def test_collector_error_cannot_become_optional(self):
        """Only upload failures may be tolerated; collection remains required."""
        path = self.repo / ".github/actions/preserve-evidence/action.yml"
        text = path.read_text()
        self.assertIn("      id: collect\n", text)
        path.write_text(text.replace("      id: collect\n",
                                     "      id: collect\n      continue-on-error: true\n", 1))
        report = self.report()
        self.assertFalse(report["passed"])
        self.assertIn("only-uploads-continue-on-error", report["failed_checks"])

    def test_default_missing_evidence_policy_cannot_be_weakened(self):
        """Already migrated calls still enforce the action's default error policy."""
        path = self.repo / ".github/actions/preserve-evidence/action.yml"
        text = path.read_text()
        self.assertIn("    default: 'error'\n", text)
        path.write_text(text.replace("    default: 'error'\n", "    default: 'warn'\n", 1))
        report = self.report()
        self.assertFalse(report["passed"])
        self.assertIn("default-required-evidence-policy", report["failed_checks"])

    def test_migrated_evidence_selection_must_remain_unchanged(self):
        """Removing already migrated action steps must not hide changed selections."""
        path = self.repo / ".github/workflows/ci.yml"
        text = path.read_text()
        self.assertIn("            build/ci-toolchain/emulator.txt\n", text)
        path.write_text(text.replace("            build/ci-toolchain/emulator.txt\n", "", 1))
        report = self.report()
        self.assertFalse(report["passed"])
        self.assertIn("preservation-calls-unchanged:.github/workflows/ci.yml",
                      report["failed_checks"])

    def test_invalid_baseline_writes_failed_report_and_exits_nonzero(self):
        """A missing ref yields inspectable failure evidence instead of a traceback."""
        output = self.repo / "failed-report.json"
        result = subprocess.run(
            [sys.executable, "-B", str(VERIFIER_PATH), "--repo", str(self.repo),
             "--baseline", "missing-baseline-ref", "--output", str(output)],
            text=True, capture_output=True, check=False,
        )
        self.assertEqual(result.returncode, 1, result.stderr)
        report = json.loads(output.read_text())
        self.assertFalse(report["passed"])
        self.assertIsNone(report["local_snapshot_baseline"])
        self.assertIn("baseline-resolves-to-commit", report["failed_checks"])
        self.assertNotIn("Traceback", result.stderr)


if __name__ == "__main__":
    unittest.main()
