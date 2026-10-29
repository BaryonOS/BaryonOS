"""Host tests for the M6e runner's serial evidence parsing (tools/run_volume.py)."""
from __future__ import annotations
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import run_volume as rv  # noqa: E402

LATENCY = "VOLUME:BROKER:MAX_COMPLETION_TICKS:"


class VolumeEvidenceParsing(unittest.TestCase):
    def test_broker_latency_is_recorded_as_evidence_only(self):
        transcript = "\n".join(rv.expected_markers("normal")) + "\n"
        with_latency = transcript.replace(
            "VOLUME:RESOURCES:PASS\n", f"{LATENCY}{7:020d}\nVOLUME:RESOURCES:PASS\n")
        self.assertEqual(rv.numbers(with_latency)[LATENCY], 7)
        # The diagnostic never enters the controlled marker sequence.
        self.assertEqual(rv.controlled(with_latency), list(rv.expected_markers("normal")))
        self.assertEqual(rv.controlled(with_latency), rv.controlled(transcript))

    def test_numbers_requires_twenty_digits_and_rejects_duplicates(self):
        self.assertEqual(rv.numbers(f"{LATENCY}7\n"), {})
        with self.assertRaises(ValueError):
            rv.numbers(f"{LATENCY}{1:020d}\n{LATENCY}{2:020d}\n")

    def test_failure_markers_do_not_match_the_diagnostic(self):
        line = f"{LATENCY}{34:020d}"
        self.assertFalse(any(marker in line for marker in rv.FAILURES))


if __name__ == "__main__":
    unittest.main()
