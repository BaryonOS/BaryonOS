import json
import importlib.util
from pathlib import Path
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
            "cases": [{"case": case, "passed": True} for _ in range(repeat)],
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
        self.assertEqual(
            jobs["smp-matrix-aggregate"]["needs"], ["smp-case-50", "smp-soak-50"]
        )
        self.assertEqual(jobs["smp-final-gate"]["needs"], "smp-matrix-aggregate")

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
