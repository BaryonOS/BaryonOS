import ast
import json
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from aggregate_smp_ci import aggregate, REPETITIONS, SOAK_CASE, SOAK_REPETITIONS, SOAK_SHARDS
from run_smp import CASES

HAS_YAML = importlib.util.find_spec("yaml") is not None


class SmpCiAggregationTests(unittest.TestCase):
    def report(self, case, repeat):
        return {
            "schema_version": 1,
            "passed": True,
            "source_sha256": "a" * 64,
            "source_fingerprint_version": 3,
            "kernel_feature": CASES[case][0],
            "repeat": repeat,
            "repetitions": {case: {"requested": repeat, "attempted": repeat,
                                    "passed": repeat, "failed": 0, "complete": True,
                                    "flaky": False}},
            "cases": [{"case": case, "passed": True,
                       "observed": {"guest_waits": {"sync_wait": [run, 2000]}},
                       "host_watchdog": {"measured_s": 10.0 + run, "deadline_s": 60}}
                      for run in range(repeat)],
        }

    def fixtures(self, root):
        for case in CASES:
            repeat = SOAK_REPETITIONS if case == SOAK_CASE else REPETITIONS
            shards = SOAK_SHARDS if case == SOAK_CASE else 1
            for shard in range(shards):
                path = root / f"{case}-{shard}" / "smp-report.json"
                path.parent.mkdir()
                path.write_text(json.dumps(self.report(case, repeat)), encoding="utf-8")

    def test_accepts_exact_complete_matrix(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixtures(root)
            with patch("aggregate_smp_ci.source_hash", return_value="a" * 64):
                result = aggregate(root, root / "matrix-report.json")
            self.assertTrue(result["passed"])
            self.assertEqual(result["case_count"], len(CASES))
            self.assertEqual(result["guest_run_count"], len(CASES) * REPETITIONS)
            self.assertTrue(all(item["passed"] == REPETITIONS for item in result["cases"].values()))
            # AC-3: the largest margin over every boot of a case is summarised.
            self.assertEqual(result["cases"]["four"]["margins"],
                             {"host_watchdog_s": [10.0 + REPETITIONS - 1, 60],
                              "sync_wait": [REPETITIONS - 1, 2000]})

    def test_rejects_missing_soak_shard(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixtures(root)
            (root / f"{SOAK_CASE}-9" / "smp-report.json").unlink()
            with patch("aggregate_smp_ci.source_hash", return_value="a" * 64):
                with self.assertRaisesRegex(ValueError, "expected 10 reports"):
                    aggregate(root, root / "matrix-report.json")

    @unittest.skipUnless(HAS_YAML, "PyYAML unavailable")
    def test_workflow_partitions_every_case_and_gates_after_aggregate(self):
        import yaml

        workflow = yaml.safe_load(
            (Path(__file__).resolve().parents[2] / ".github/workflows/native-smp.yml")
            .read_text(encoding="utf-8")
        )
        jobs = workflow["jobs"]
        normal = jobs["smp-case-50"]["strategy"]["matrix"]["case"]
        soak = jobs["smp-soak-50"]["strategy"]["matrix"]["shard"]
        self.assertEqual(set(normal), set(CASES) - {SOAK_CASE})
        self.assertEqual(len(normal), len(set(normal)))
        self.assertEqual(soak, list(range(SOAK_SHARDS)))
        self.assertEqual(jobs["smp-case-50"]["needs"], "smp-environment-probe")
        self.assertEqual(jobs["smp-soak-50"]["needs"], "smp-environment-probe")
        self.assertEqual(jobs["smp-case-50"]["runs-on"], "ubuntu-26.04")
        self.assertEqual(jobs["smp-soak-50"]["runs-on"], "ubuntu-26.04")
        self.assertEqual(jobs["smp-environment-probe"]["env"]["NOS_QEMU_MACHINE"], "pc-q35-10.0")
        self.assertNotIn("with", jobs["native-smp"]["steps"][1])
        for job in ("smp-environment-probe", "smp-case-50", "smp-soak-50", "smp-final-gate"):
            steps = jobs[job]["steps"]
            names = [step["name"] for step in steps]
            self.assertLess(names.index(next(name for name in names if name.startswith("Pinned QEMU"))),
                            names.index("Pinned Python"))
            python = next(step for step in steps if step["name"] == "Pinned Python")
            self.assertEqual(python["with"]["system-python"], "true")
        self.assertEqual(
            jobs["smp-matrix-aggregate"]["needs"], ["smp-case-50", "smp-soak-50"]
        )
        self.assertEqual(jobs["smp-final-gate"]["needs"], "smp-matrix-aggregate")

    @unittest.skipUnless(HAS_YAML, "PyYAML unavailable")
    def test_clock_diagnostics_cannot_launch_acceptance_jobs(self):
        import yaml

        workflow = yaml.load(
            (Path(__file__).resolve().parents[2] / ".github/workflows/native-smp.yml")
            .read_text(encoding="utf-8"), Loader=yaml.BaseLoader,
        )
        self.assertEqual(workflow["on"]["workflow_dispatch"]["inputs"]
                         ["clock_diagnostic"]["default"], "false")
        jobs = workflow["jobs"]

        def eligible(expression, event, full, diagnostic, margin=False):
            expression = expression[3:-2].strip().replace("&&", " and ").replace("||", " or ")
            expression = expression.replace("!=", " NE ").replace("!", " not ").replace(" NE ", "!=")
            values = {("github", "event_name"): event, ("inputs", "full_matrix"): full,
                      ("inputs", "clock_diagnostic"): diagnostic,
                      ("inputs", "margin_diagnostic"): margin}

            def visit(node):
                if isinstance(node, ast.Constant):
                    return node.value
                if isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name):
                    return values[(node.value.id, node.attr)]
                if isinstance(node, ast.BoolOp):
                    if isinstance(node.op, ast.And):
                        return all(visit(value) for value in node.values)
                    if isinstance(node.op, ast.Or):
                        return any(visit(value) for value in node.values)
                if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.Not):
                    return not visit(node.operand)
                if isinstance(node, ast.Compare) and len(node.ops) == 1:
                    left, right = visit(node.left), visit(node.comparators[0])
                    if isinstance(node.ops[0], ast.Eq):
                        return left == right
                    if isinstance(node.ops[0], ast.NotEq):
                        return left != right
                if (isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
                        and node.func.id == "always" and not node.args and not node.keywords):
                    return True
                self.fail(f"Unsupported workflow condition: {ast.dump(node)}")

            return visit(ast.parse(expression.strip(), mode="eval").body)

        full_jobs = {"smp-environment-probe", "smp-case-50", "smp-soak-50",
                     "smp-matrix-aggregate", "smp-final-gate"}
        for event, full, diagnostic, expected in (
            ("pull_request", False, False, {"native-smp", "experimental-m3-m4-smp"}),
            ("workflow_dispatch", False, False, {"native-smp", "experimental-m3-m4-smp"}),
            ("workflow_dispatch", True, False, full_jobs),
            ("workflow_dispatch", False, True, {"diagnostic-clock"}),
            ("workflow_dispatch", True, True, {"diagnostic-clock"}),
        ):
            with self.subTest(event=event, full=full, diagnostic=diagnostic):
                actual = {name for name, job in jobs.items()
                          if eligible(job["if"], event, full, diagnostic)}
                self.assertEqual(actual, expected)
        # A margin diagnostic excludes every acceptance and clock job, whatever else is set.
        for full in (False, True):
            for diagnostic in (False, True):
                with self.subTest(margin=True, full=full, diagnostic=diagnostic):
                    self.assertEqual({name for name, job in jobs.items()
                                      if eligible(job["if"], "workflow_dispatch", full, diagnostic, True)},
                                     {"diagnostic-margin", "diagnostic-gate-rehearsal"})
        margin = jobs["diagnostic-margin"]
        for key in ("runs-on", "timeout-minutes"):
            self.assertEqual(margin[key], jobs["smp-case-50"][key])
        self.assertEqual(margin["env"]["NOS_QEMU_MACHINE"], jobs["smp-case-50"]["env"]["NOS_QEMU_MACHINE"])
        refuse = next(step for step in margin["steps"] if "OTHER_MODES" in step.get("env", {}))
        for modes, exit_code in (("false/false", 0), ("true/false", 2), ("false/true", 2), ("/", 2)):
            result = subprocess.run(["bash", "-c", refuse["run"]], env={"OTHER_MODES": modes},
                                    text=True, capture_output=True)
            self.assertEqual(result.returncode, exit_code)
        rehearsal = jobs["diagnostic-gate-rehearsal"]
        self.assertEqual(rehearsal["env"] | {"NOS_BUILD_DIR": 0, "NOS_TARGET_DIR": 0, "CARGO_TARGET_DIR": 0},
                         jobs["smp-final-gate"]["env"] | {"NOS_BUILD_DIR": 0, "NOS_TARGET_DIR": 0, "CARGO_TARGET_DIR": 0})
        self.assertTrue(rehearsal["steps"][-1]["with"]["name"].startswith("DIAGNOSTIC-NOT-ACCEPTANCE-"))
        evidence = margin["steps"][-1]["with"]
        self.assertTrue(evidence["name"].startswith("DIAGNOSTIC-NOT-ACCEPTANCE-"))
        self.assertNotIn("smp-report.json", evidence["path"] + evidence["required-paths"])
        steps = jobs["diagnostic-clock"]["steps"]
        guard = next(i for i, step in enumerate(steps)
                     if "FULL_MATRIX_REQUESTED" in step.get("env", {}))
        setup = next(i for i, step in enumerate(steps)
                     if step.get("uses") == "./.github/actions/setup-qemu")
        self.assertLess(guard, setup)
        for requested, exit_code in (("false", 0), ("true", 2), ("", 2)):
            result = subprocess.run(["bash", "-c", steps[guard]["run"]],
                                    env={"FULL_MATRIX_REQUESTED": requested},
                                    text=True, capture_output=True)
            self.assertEqual(result.returncode, exit_code)

    @unittest.skipUnless(HAS_YAML, "PyYAML unavailable")
    def test_qemu_action_keeps_old_profile_and_pins_ci_matrix_profile(self):
        import yaml

        root = Path(__file__).resolve().parents[2]
        action = yaml.safe_load(
            (root / ".github/actions/setup-qemu/action.yml").read_text(encoding="utf-8")
        )
        script = action["runs"]["steps"][0]["run"]
        subprocess.run(["bash", "-n"], input=script, text=True, check=True)
        for required in (
            "qemu-system-x86=1:8.2.2+ds-0ubuntu1.18",
            "qemu-system-x86=1:10.2.1+ds-1ubuntu3.3",
            "python3.14=3.14.4-1ubuntu0.2",
            "pc-q35-8.2",
            "pc-q35-10.0",
            "874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7",
        ):
            self.assertIn(required, script)

    def test_rejects_failed_run_hidden_by_summary(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixtures(root)
            path = root / "one-0" / "smp-report.json"
            report = json.loads(path.read_text())
            report["cases"][17]["passed"] = False
            path.write_text(json.dumps(report), encoding="utf-8")
            with patch("aggregate_smp_ci.source_hash", return_value="a" * 64):
                with self.assertRaisesRegex(ValueError, "malformed SMP run evidence"):
                    aggregate(root, root / "matrix-report.json")


if __name__ == "__main__":
    unittest.main()
