"""Host tests for the M6e runner's serial evidence parsing (tools/run_volume.py)."""
from __future__ import annotations
import ast
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import run_volume as rv  # noqa: E402

LATENCY = "VOLUME:BROKER:MAX_COMPLETION_TICKS:"
DIAGNOSTICS = (LATENCY, "VOLUME:BROKER:TIMEOUTS:", "VOLUME:BROKER:OUTSTANDING_TICKS:")


class VolumeEvidenceParsing(unittest.TestCase):
    def test_broker_latency_is_recorded_as_evidence_only(self):
        transcript = "\n".join(rv.expected_markers("normal")) + "\n"
        block = "".join(f"{key}{value:020d}\n" for key, value in zip(DIAGNOSTICS, (7, 1, 34)))
        with_latency = transcript.replace("VOLUME:RESOURCES:PASS\n", block + "VOLUME:RESOURCES:PASS\n")
        self.assertEqual([rv.numbers(with_latency)[key] for key in DIAGNOSTICS], [7, 1, 34])
        # The diagnostic never enters the controlled marker sequence.
        self.assertEqual(rv.controlled(with_latency), list(rv.expected_markers("normal")))
        self.assertEqual(rv.controlled(with_latency), rv.controlled(transcript))

    def test_numbers_requires_twenty_digits_and_rejects_duplicates(self):
        self.assertEqual(rv.numbers(f"{LATENCY}7\n"), {})
        with self.assertRaises(ValueError):
            rv.numbers(f"{LATENCY}{1:020d}\n{LATENCY}{2:020d}\n")

    def test_failure_markers_do_not_match_the_diagnostic(self):
        for key in DIAGNOSTICS:
            line = f"{key}{34:020d}"
            self.assertFalse(any(marker in line for marker in rv.FAILURES))


class VolumeStageTables(unittest.TestCase):
    """Every gate stage has a feature, a marker, a built case and an allowed parent."""

    def test_plan_matches_the_runner_tables(self):
        tools = Path(__file__).resolve().parents[1] / "tools"
        tree = ast.parse((tools / "test_volume.py").read_text(encoding="utf-8"))
        plan = next(ast.literal_eval(node.value) for node in tree.body
                    if isinstance(node, ast.Assign) and getattr(node.targets[0], "id", None) == "PLAN")
        cases = {case for case, _ in plan}
        self.assertTrue({"install", "install-recover", "install-cut"} <= cases)
        for case, parent in plan:
            self.assertIn(case, rv.FEATURE)
            self.assertTrue(case in rv.CUT_MARKER or case in rv.CASE_MARKER, case)
            if parent is None:
                self.assertIn(case, rv.FRESH)
            else:
                self.assertNotIn(case, rv.FRESH)
                self.assertIn((case, parent), rv.ALLOWED_PARENTS)
        import build
        for case, feature in rv.FEATURE.items():
            self.assertIn(feature, build.VOLUME_FEATURES, case)
        # The install stages link the plain app, never the archive portal.
        self.assertFalse(set(rv.INSTALL_CASES) & set(build.VOLUME_ARCHIVE_CASES))
        self.assertEqual(rv.expected_markers("install-cut")[-1], "VOLUME:CUT:INSTALL")
        self.assertIn("install-recover", rv.READ_ONLY)


if __name__ == "__main__":
    unittest.main()
