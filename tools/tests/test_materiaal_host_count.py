"""Host log parsing checks; no native execution evidence."""
import importlib.util
from pathlib import Path
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
