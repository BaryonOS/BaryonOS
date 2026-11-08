"""CI-HOSTED: guard public-runner isolation and mandatory workflow coverage.

These are checks of the checked-in configuration, not execution of GitHub's
expression engine, the test commands, or native QEMU acceptance.
"""

from pathlib import Path
import re
import unittest

import yaml


ROOT = Path(__file__).resolve().parents[1]
EVIDENCE_ACTION = ".github/actions/preserve-evidence/action.yml"


class UniqueKeyLoader(yaml.BaseLoader):
    """Keep `on` and boolean scalars as text and reject overwritten YAML keys."""

    def construct_mapping(self, node, deep=False):
        """Fail instead of silently accepting a duplicate workflow setting."""
        result = {}
        for key_node, value_node in node.value:
            key = self.construct_object(key_node, deep=deep)
            if key in result:
                raise yaml.constructor.ConstructorError(
                    None, None, f"duplicate YAML key: {key}", key_node.start_mark,
                )
            result[key] = self.construct_object(value_node, deep=deep)
        return result


class WorkflowContractTests(unittest.TestCase):
    """Detect runner, credential, gate and deduplication regressions."""

    @classmethod
    def setUpClass(cls):
        """Read the actual workflows and local composite actions once per run."""
        cls.workflows = {}
        cls.actions = {}
        for directory, pattern, destination in [
            (ROOT / ".github/workflows", "*.y*ml", cls.workflows),
            (ROOT / ".github/actions", "**/action.y*ml", cls.actions),
        ]:
            for path in sorted(directory.glob(pattern)):
                destination[path.relative_to(ROOT).as_posix()] = yaml.load(
                    path.read_text(encoding="utf-8"), Loader=UniqueKeyLoader,
                )

    def jobs(self):
        """Yield every job, including manual and private-only maintenance jobs."""
        for path, workflow in self.workflows.items():
            for job_id, job in workflow["jobs"].items():
                yield path, job_id, job

    def steps(self):
        """Include nested local-action steps in supply-chain and error checks."""
        for path, job_id, job in self.jobs():
            for step in job.get("steps", []):
                yield path, job_id, step
        for path, action in self.actions.items():
            for step in action.get("runs", {}).get("steps", []):
                yield path, "composite", step

    def test_all_jobs_use_standard_hosted_ubuntu(self):
        """No custom label, dynamic selector or persistent runner can return."""
        self.assertTrue(self.workflows)
        for path, job_id, job in self.jobs():
            with self.subTest(path=path, job=job_id):
                self.assertEqual(job.get("runs-on"), "ubuntu-24.04")

    def test_public_prs_keep_read_only_tokens(self):
        """Untrusted PR code must not run under a privileged target event/token."""
        for path, workflow in self.workflows.items():
            with self.subTest(path=path):
                self.assertNotIn("pull_request_target", workflow["on"])
                permissions = workflow.get("permissions")
                self.assertIsInstance(permissions, dict)
                self.assertEqual(permissions.get("contents"), "read")
                self.assertTrue(set(permissions.values()) <= {"read", "none"})
        for path, job_id, job in self.jobs():
            with self.subTest(path=path, job=job_id):
                permissions = job.get("permissions", {})
                self.assertIsInstance(permissions, dict)
                self.assertTrue(set(permissions.values()) <= {"read", "none"})

    def test_external_actions_are_pinned_to_full_commits(self):
        """Moving tags cannot silently change executable Actions code."""
        for path, job_id, step in self.steps():
            reference = step.get("uses")
            if reference and not reference.startswith("./"):
                with self.subTest(path=path, job=job_id, uses=reference):
                    self.assertRegex(reference, r"^[^\s@]+/[^\s@]+@[0-9a-f]{40}$")

    def test_checkout_does_not_persist_credentials(self):
        """Builds and public PR scripts do not inherit checkout credentials."""
        for path, job_id, step in self.steps():
            if step.get("uses", "").startswith("actions/checkout@"):
                with self.subTest(path=path, job=job_id):
                    self.assertEqual(step.get("with", {}).get("persist-credentials"), "false")

    def test_required_gates_are_unfiltered_and_independent(self):
        """Required check names always exist and run without a serial dependency."""
        workflow = self.workflows[".github/workflows/ci.yml"]
        for event in ("pull_request", "push"):
            trigger = workflow["on"][event]
            with self.subTest(event=event):
                self.assertNotIn("paths", trigger)
                self.assertNotIn("paths-ignore", trigger)
        self.assertIn("main", workflow["on"]["push"]["branches"])
        for job_id in ("host-quality", "qemu-smoke"):
            job = workflow["jobs"][job_id]
            with self.subTest(job=job_id):
                self.assertEqual(job.get("name", job_id), job_id)
                self.assertNotIn("if", job)
                self.assertFalse(job.get("needs"))
                self.assertEqual(job.get("continue-on-error", "false"), "false")
                self.assertTrue(any("run" in step for step in job["steps"]))

    def test_only_artifact_uploads_can_tolerate_errors(self):
        """Storage failure may be optional; failed builds, tests and collectors are not."""
        for path, job_id, job in self.jobs():
            with self.subTest(path=path, job=job_id):
                self.assertEqual(job.get("continue-on-error", "false"), "false")
        for path, job_id, step in self.steps():
            if step.get("continue-on-error", "false") != "false":
                with self.subTest(path=path, job=job_id, step=step.get("name")):
                    self.assertEqual(step["continue-on-error"], "true")
                    self.assertEqual(path, EVIDENCE_ACTION)
                    self.assertTrue(step.get("uses", "").startswith("actions/upload-artifact@"))

    def test_pr_lifecycle_runs_once_without_skipping_drafts(self):
        """Draft changes are tested already, so becoming ready need not repeat them."""
        for path, workflow in self.workflows.items():
            if "pull_request" not in workflow["on"]:
                continue
            trigger = workflow["on"]["pull_request"] or {}
            types = trigger.get("types", ["opened", "synchronize", "reopened"])
            with self.subTest(path=path):
                self.assertTrue({"opened", "synchronize", "reopened"} <= set(types))
                self.assertNotIn("ready_for_review", types)
                for job in workflow["jobs"].values():
                    self.assertNotIn("draft", job.get("if", ""))

    def test_deduplication_keeps_host_and_scalar_component_coverage(self):
        """Both removed matrix entries retain tests, lint and scalar target builds."""
        matrix = self.workflows[".github/workflows/component-leaves.yml"]["jobs"][
            "component-quality"
        ]["strategy"]["matrix"]["include"]
        self.assertTrue({"artifact-store", "materiaal-ui"}.isdisjoint(
            entry["component"] for entry in matrix
        ))
        for filename, job_id, component in [
            ("artifact-store-quality.yml", "artifact-store-core", "artifact-store-core"),
            ("native-display-input.yml", "native-display-input", "ui/native/Cargo.toml"),
        ]:
            job = self.workflows[f".github/workflows/{filename}"]["jobs"][job_id]
            relevant = [step for step in job["steps"] if component in step.get("run", "")]
            commands = "\n".join(step["run"] for step in relevant)
            with self.subTest(component=component):
                for operation in ("test", "clippy", "fmt"):
                    self.assertRegex(commands, rf"\bcargo\b[^\n]*\b{operation}\b")
                scalar = [step for step in relevant if re.search(
                    r"\bcargo\b[^\n]*\bbuild\b", step["run"],
                ) and "x86_64-unknown-none" in step["run"]]
                self.assertTrue(scalar, "missing freestanding component build")
                for step in scalar:
                    flags = {**job.get("env", {}), **step.get("env", {})}.get("RUSTFLAGS", "")
                    self.assertIn("no-redzone=yes", flags)
                    self.assertIn("-sse2", flags)
                    self.assertIn("-avx2", flags)

    def test_artifact_component_filter_includes_shared_build_inputs(self):
        """The dedicated gate replaces its broad matrix entry for shared changes too."""
        workflow = self.workflows[".github/workflows/artifact-store-quality.yml"]
        for event in ("pull_request", "push"):
            with self.subTest(event=event):
                self.assertTrue({
                    "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo/**",
                    ".github/actions/**", "crates/artifact-store-core/**",
                } <= set(workflow["on"][event]["paths"]))


if __name__ == "__main__":
    unittest.main()
