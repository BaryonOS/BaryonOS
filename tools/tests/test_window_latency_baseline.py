"""Host negatives for the descriptive latency baseline; no VM evidence."""
import json
import sys
from pathlib import Path
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import window_latency_baseline as baseline


def report(run, values, **changes):
    data = {"passed": True, "raw_qemu_exit": 33, "source_commit": "a" * 40,
            "source_sha256": "b" * 64, "kernel_feature": "native-window-session-probe",
            "scale": 1, "viewport": [1280, 800], "qemu_sha256": "c" * 64,
            "clipboard_requested": True, "workspace_requested": True,
            "switcher_requested": False, "library_requested": False,
            "session_route_requested": False, "run_directory": f"/run/{run}",
            "duration_s": 100.0 + run,
            "phases": {"boot": {"seconds": 1.0},
                       **{name: {"seconds": 2.0, baseline.FIELD: value, "last_input_kind": "key"}
                          for name, value in values.items()}}}
    data.update(changes)
    return data


class BaselineTests(unittest.TestCase):
    def write(self, *reports):
        directory = Path(self.enterContext(tempfile.TemporaryDirectory()))
        paths = []
        for index, data in enumerate(reports):
            path = directory / f"report-{index}.json"
            path.write_text(json.dumps(data), encoding="utf-8")
            paths.append(path)
        return paths

    def test_groups_phases_and_never_claims_acceptance(self):
        paths = self.write(*(report(run, {"editor": 0.1 * (run + 1), "dirty": 0.5}) for run in range(3)))
        summary = baseline.summarise(paths)
        self.assertEqual((summary["kind"], summary["accepted"], summary["budget"]), ("baseline", None, None))
        editor = summary["phases"]["editor"]
        self.assertEqual(editor["count"], 3)
        self.assertAlmostEqual(editor["min_s"], 0.1)
        self.assertAlmostEqual(editor["p50_s"], 0.2)
        self.assertAlmostEqual(editor["p95_s"], 0.3)
        self.assertAlmostEqual(editor["max_s"], 0.3)
        self.assertEqual(editor["last_input_kinds"], ["key"])
        self.assertNotIn("boot", summary["phases"])  # No interval recorded.
        self.assertEqual(len({run["sha256"] for run in summary["runs"]}), 3)
        self.assertIn("not U02 acceptance", baseline.text(summary))

    def test_rejects_failed_or_unexited_runs(self):
        for bad in ({"passed": False}, {"raw_qemu_exit": 1}, {"error": "Native phase did not complete"},
                    {"phases": {}}):
            with self.subTest(bad=bad), self.assertRaisesRegex(ValueError, "did not pass|no verified"):
                baseline.summarise(self.write(report(0, {"editor": 0.1}), report(1, {"editor": 0.1}, **bad)))

    def test_rejects_mixed_source_profile_or_feature(self):
        for key, value in (("source_commit", "d" * 40), ("source_sha256", "e" * 64),
                           ("kernel_feature", "native-window-session-hidpi-probe"), ("scale", 2),
                           ("workspace_requested", False), ("qemu_sha256", "f" * 64)):
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, key):
                baseline.summarise(self.write(report(0, {"editor": 0.1}), report(1, {"editor": 0.1}, **{key: value})))

    def test_rejects_reused_run_or_duplicate_report(self):
        with self.assertRaisesRegex(ValueError, "Duplicate"):
            baseline.summarise(self.write(report(0, {"editor": 0.1}), report(0, {"editor": 0.2})))
        same = report(0, {"editor": 0.1})
        with self.assertRaisesRegex(ValueError, "Duplicate"):
            baseline.summarise(self.write(same, same))

    def test_rejects_invalid_or_missing_intervals(self):
        for value in (-1, "0.1", None, float("inf")):
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, "invalid"):
                baseline.summarise(self.write(report(0, {"editor": value})))
        with self.assertRaisesRegex(ValueError, "No phase"):
            baseline.summarise(self.write(report(0, {})))

    def test_rejects_unbound_source_and_uneven_phases(self):
        for key in ("source_commit", "source_sha256"):
            with self.subTest(key=key), self.assertRaises(ValueError):
                baseline.summarise(self.write(*(report(run, {"editor": 0.1}, **{key: None}) for run in range(2))))
        with self.assertRaisesRegex(ValueError, "present in 1 of 2"):
            baseline.summarise(self.write(report(0, {"editor": 0.1, "dirty": 0.2}), report(1, {"editor": 0.1})))

    def test_records_report_base_names_only(self):
        summary = baseline.summarise(self.write(*(report(run, {"editor": 0.1}) for run in range(2))))
        self.assertEqual([run["report"] for run in summary["runs"]], ["report-0.json", "report-1.json"])

    def test_cli_fails_closed_and_writes_json(self):
        good = self.write(report(0, {"editor": 0.1}), report(1, {"editor": 0.2}))
        output = good[0].parent / "baseline.json"
        self.assertEqual(baseline.main([*map(str, good), "--output", str(output)]), 0)
        self.assertIsNone(json.loads(output.read_text())["accepted"])
        bad = self.write(report(0, {"editor": 0.1}, passed=False))
        self.assertEqual(baseline.main([str(bad[0])]), 1)


if __name__ == "__main__":
    unittest.main()
