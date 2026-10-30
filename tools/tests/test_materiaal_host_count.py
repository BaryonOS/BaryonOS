import importlib.util
from pathlib import Path
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location(
    "materiaal_verify", ROOT / "sdk/examples/materiaal/verify.py"
)
verify = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify)


class HostScenarioCountTests(unittest.TestCase):
    def count(self, text):
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / "host.stdout.txt"
            log.write_text(text)
            return verify.host_scenario_count(log)

    def test_counts_all_successful_suites(self):
        self.assertEqual(self.count(
            "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
            "test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.25s\n"
        ), 20)

    def test_rejects_missing_wrong_or_invalid_counts(self):
        valid = "test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
        for text in ("", valid.replace("20 passed", "19 passed"),
                     valid.replace("20 passed", "xx passed"),
                     valid + "test result: FAILED. 1 passed; 1 failed\n",
                     valid.replace("0 ignored", "1 ignored")):
            with self.subTest(text=text), self.assertRaises(RuntimeError):
                self.count(text)


if __name__ == "__main__":
    unittest.main()
