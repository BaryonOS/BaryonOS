"""The pre-native report must reflect a complete host test run."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("materiaal_verify", ROOT / "sdk/examples/materiaal/verify.py")
verify = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify)


def result(passed, failed=0, ignored=0, filtered=0):
    return (f"test result: ok. {passed} passed; {failed} failed; {ignored} ignored; "
            f"0 measured; {filtered} filtered out; finished in 0.00s\n")


class HostScenariosTests(unittest.TestCase):
    def parse(self, output):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "host.stdout.txt"
            path.write_text(output)
            return verify.host_scenario_count(path)

    def test_counts_multiple_complete_harnesses(self):
        output = "running 0 tests\n" + result(0) + "running 20 tests\n" + result(20)
        self.assertEqual(self.parse(output), 20)

    def test_rejects_missing_malformed_filtered_and_wrong_counts(self):
        for output in ("", "running 20 tests\n", "running 20 tests\n" + result(19),
                       "running 19 tests\n" + result(19),
                       "running 20 tests\n" + result(20, failed=1),
                       "running 20 tests\n" + result(20, ignored=1),
                       "running 20 tests\n" + result(20, filtered=1)):
            with self.subTest(output=output), self.assertRaises(RuntimeError):
                self.parse(output)


if __name__ == "__main__":
    unittest.main()
