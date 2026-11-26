"""The larger runner (baryon-heavy-16) is reachable only from one guarded, default-off expression."""
import pathlib
import unittest

import yaml

ROOT = pathlib.Path(__file__).resolve().parents[1]
HEAVY = "baryon-heavy-16"
EXPECTED = ("${{ github.event_name == 'workflow_dispatch' && github.ref == 'refs/heads/main' "
            "&& matrix.leg == 'native-scale2' && vars.BARYON_HEAVY_ENABLED == 'true' "
            "&& 'baryon-heavy-16' || 'ubuntu-24.04' }}")


class HeavyRunnerRoutingTests(unittest.TestCase):
    def test_only_the_guarded_window_session_leg_names_the_larger_runner(self):
        users = []
        for path in sorted((ROOT / ".github" / "workflows").glob("*.yml")):
            text = path.read_text(encoding="utf-8")
            jobs = (yaml.safe_load(text) or {}).get("jobs") or {}
            users += [(path.name, job_id, job.get("runs-on")) for job_id, job in jobs.items()
                      if HEAVY in str(job.get("runs-on"))]
            # Any other mention (a label list, a group, an env value) would bypass this pin.
            self.assertEqual(text.count(HEAVY), 1 if path.name == "native-window-session.yml" else 0, path.name)
        self.assertEqual(users, [("native-window-session.yml", "native-window-session", EXPECTED)])

    def test_the_expression_falls_back_to_the_standard_runner(self):
        # A missing, empty or any value other than 'true' selects ubuntu-24.04 before scheduling;
        # pull_request, merge_group and push never reach the larger runner.
        self.assertTrue(EXPECTED.endswith("|| 'ubuntu-24.04' }}"))
        for guard in ("github.event_name == 'workflow_dispatch'", "github.ref == 'refs/heads/main'",
                      "vars.BARYON_HEAVY_ENABLED == 'true'"):
            self.assertIn(guard, EXPECTED)
        self.assertNotIn("inputs.", EXPECTED)


if __name__ == "__main__":
    unittest.main()
