"""Host-only checks of the portable PR133 runner; no native execution evidence."""
from __future__ import annotations

import ast
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SOURCE / "tools"))
import run_volume_admission as admission  # noqa: E402

ARCHIVED = SOURCE / "docs/verification/volume-format-admission/native/native_admission.py"


def functions(path):
    return {node.name: node for node in ast.parse(path.read_text(encoding="utf-8")).body
            if isinstance(node, ast.FunctionDef)}


class ArchivedMachine(ast.NodeTransformer):
    def visit_Constant(self, node):
        if node.value == "pc-q35-10.0":
            return ast.Call(func=ast.Name(id="qemu_machine", ctx=ast.Load()), args=[], keywords=[])
        return node


class AdmissionRunnerTests(unittest.TestCase):
    def test_native_verdicts_and_oracles_are_not_replaced_or_weakened(self):
        old, new = functions(ARCHIVED), functions(Path(admission.__file__))
        for name in ("run", "matrix", "prepare", "source_state", "frozen", "regression",
                     "regression_child", "sha", "save", "local", "main", "build_profiles"):
            with self.subTest(function=name):
                original = ArchivedMachine().visit(old[name])
                self.assertEqual(ast.dump(original), ast.dump(new[name]))

    def test_archived_runner_bytes_remain_the_original_evidence(self):
        self.assertEqual(
            hashlib.sha256(ARCHIVED.read_bytes()).hexdigest(),
            "dc5707cc6972b6b666d2e9d6925a275850d5afa9b48fa9bd68ce69d2c26175a0")

    def test_foreign_fixtures_keep_both_original_full_image_hashes(self):
        self.assertEqual({name: digest for name, (_, digest) in admission.CANDIDATES.items()}, {
            "btrfs": "0f3e5d9631ac9e8747abd7d6744538772620c0d2d9c6c5f40f2ed7e864edac39",
            "openzfs-leaf": "23a409ac5e9a844056d9096ead0453112f1d1e6e45d4b7d5db084a397baea4ea",
        })

    def test_source_is_checkout_and_outputs_default_to_build(self):
        self.assertEqual(admission.SOURCE, SOURCE)
        self.assertEqual(admission.ROOT, admission.BUILD / "volume-admission")
        for path, _ in admission.CANDIDATES.values():
            path.relative_to(admission.FIXTURES)

    def test_build_environment_reuses_toolchain_and_cache_without_installing(self):
        with patch.object(admission, "environment", return_value={"CARGO_HOME": "existing-cache"}), \
                patch.object(admission, "firmware", return_value=("code.fd", "vars.fd")):
            env = admission.build_environment()
        self.assertEqual(env["CARGO_HOME"], "existing-cache")
        self.assertEqual(env["CARGO_NET_OFFLINE"], "true")
        self.assertEqual(env["RUSTUP_AUTO_INSTALL"], "0")
        self.assertNotIn("LOCALAPPDATA", env)

    def test_machine_selection_is_explicit_and_rejects_unknown_profiles(self):
        with patch.dict(os.environ, {"NOS_QEMU_MACHINE": "pc-q35-8.2"}):
            self.assertEqual(admission.qemu_machine(), "pc-q35-8.2")
        with patch.dict(os.environ, {"NOS_QEMU_MACHINE": "unrecorded-machine"}):
            with self.assertRaises(ValueError):
                admission.qemu_machine()

    def test_freeze_rejects_source_test_or_runner_drift(self):
        scratch = SOURCE / "build/volume-admission-host-tests"
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=scratch) as directory:
            state = {"head": "a" * 40, "source_sha256": "b" * 64,
                     "volume_test_inputs": {"tests/test_volume_runner.py": "c" * 64},
                     "runner_sha256": "d" * 64}
            with patch.object(admission, "ROOT", Path(directory)), \
                    patch.object(admission, "source_state", return_value=state) as source:
                self.assertEqual(admission.frozen(state["head"]), state)
                self.assertEqual(json.loads((Path(directory) / "freeze.json").read_text()), state)
                for key in ("head", "source_sha256", "volume_test_inputs", "runner_sha256"):
                    source.return_value = dict(state, **{key: "changed"})
                    with self.subTest(key=key), self.assertRaises(RuntimeError):
                        admission.frozen(state["head"])

    def test_output_confinement_rejects_checkout_or_parent_paths(self):
        with self.assertRaises(ValueError):
            admission.local(SOURCE / "services/volume/Cargo.toml")
        with self.assertRaises(ValueError):
            admission.local(admission.ROOT / "../escape")

    def test_wrong_expected_source_head_is_a_hard_failure(self):
        with patch.object(admission.subprocess, "check_output", return_value="b" * 40 + "\n"):
            with self.assertRaises(RuntimeError):
                admission.source_state("a" * 40)

    def test_pr_and_main_push_schedule_both_existing_native_gates(self):
        workflow = (SOURCE / ".github/workflows/volume-acceptance.yml").read_text()
        events = workflow.split("permissions:", 1)[0]
        self.assertIn("  pull_request:\n", events)
        self.assertIn("  push:\n    branches: [main]\n", events)
        self.assertIn("'tests/test_volume_*.py'", events)
        self.assertIn("run: python3 -B -u tools/test_volume.py", workflow)
        for action in ("build", "matrix"):
            self.assertIn(f'tools/run_volume_admission.py {action} --expected-head "$expected_head"', workflow)
        self.assertIn("  actions: read\n", workflow)

    def test_absent_or_malformed_fixture_run_is_never_a_skipped_green_gate(self):
        workflow = (SOURCE / ".github/workflows/volume-acceptance.yml").read_text()
        step = workflow.split("      - name: Obtain the immutable foreign-image fixtures\n", 1)[1]
        step = step.split("      - name:", 1)[0].split("        run: |\n", 1)[1]
        script = "\n".join(line[10:] for line in step.splitlines())
        for value in ("", "latest", "0", "12; exit 0"):
            with self.subTest(value=value):
                result = subprocess.run(["bash", "-c", script],
                                        env=dict(os.environ, FIXTURE_RUN_ID=value),
                                        capture_output=True, text=True, timeout=5)
                self.assertEqual(result.returncode, 1)
                self.assertIn("absence is not a passed native matrix", result.stdout)


if __name__ == "__main__":
    unittest.main()
