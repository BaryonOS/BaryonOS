"""Host log parsing checks; no native execution evidence."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

path = Path(__file__).resolve().parents[2] / "sdk/examples/materiaal/verify.py"
spec = importlib.util.spec_from_file_location("materiaal_verify", path)
verify = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify)


class HostCount(unittest.TestCase):
    def test_recorded_chain_count(self):
        log = (path.parent / "verification/f12-host.stdout.txt").read_text()
        # The historical full host run has multiple target summaries.
        chain = next(line for line in log.splitlines() if line.startswith("test result: ok. 20 passed"))
        self.assertEqual(verify.chain_scenario_count(chain), 20)

    def test_missing_ambiguous_and_incomplete_results_rejected(self):
        good = "test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;"
        for log in ("", good + "\n" + good, good + "\ntest result: FAILED.", good.replace("20 passed", "0 passed"),
                    good.replace("0 ignored", "1 ignored"), good.replace("0 failed", "1 failed")):
            with self.subTest(log=log), self.assertRaises(RuntimeError):
                verify.chain_scenario_count(log)


class SdkBuildInput(unittest.TestCase):
    def test_diff_file_tracks_current_diff_and_clears_stale_output(self):
        with tempfile.TemporaryDirectory() as name:
            output = Path(name)
            file = output / "sdk-build-input.json"
            verify.write_sdk_build_input(output, b"")
            self.assertFalse(file.exists())
            diff = b"diff --git a/sdk/runtime b/sdk/runtime\n"
            verify.write_sdk_build_input(output, diff)
            self.assertEqual(json.loads(file.read_text()), {
                "patch_utf8": diff.decode(), "sha256": hashlib.sha256(diff).hexdigest()})
            verify.write_sdk_build_input(output, b"")
            self.assertFalse(file.exists())
            verify.write_sdk_build_input(output, b"")
            self.assertFalse(file.exists())


class HostScenarioCountTests(unittest.TestCase):
    def count(self, text):
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / "host.stdout.txt"
            log.write_text(text)
            return verify.host_scenario_count(log)

    def test_counts_all_successful_suites(self):
        self.assertEqual(self.count(
            "running 2 tests\n"
            "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
            "running 18 tests\n"
            "test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.25s\n"
        ), 20)

    def test_rejects_missing_wrong_or_invalid_counts(self):
        valid = "running 20 tests\n" "test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
        for text in ("", valid.replace("20 passed", "19 passed"),
                     valid.replace("20 passed", "xx passed"),
                     valid + "test result: FAILED. 1 passed; 1 failed\n",
                     valid.replace("0 ignored", "1 ignored")):
            with self.subTest(text=text), self.assertRaises(RuntimeError):
                self.count(text)
