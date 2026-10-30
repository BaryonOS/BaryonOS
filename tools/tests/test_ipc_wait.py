import re
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_ipc_wait as probe


class NativeWaitTranscriptTests(unittest.TestCase):
    def transcript(self):
        counts = {"BLOCKED": 4, "WAKE_READY": 2, "WAKE_TIMEOUT": 2,
                  "WAKE_ERROR": 1, "IDLE_IRQS": 3}
        return "\n".join(
            [f"IPCWAIT:{key}:{value:020d}" for key, value in counts.items()]
            + [f"IPCWAIT:PHASE_MASK:{mask:020d}" for mask in (0x3f, 0x3c)]
            + list(probe.MARKERS)
        ) + "\n"

    def test_requires_real_wait_and_idle_counters(self):
        self.assertEqual(probe.evaluate(self.transcript())["IDLE_IRQS"], 3)
        for key in ("BLOCKED", "WAKE_READY", "WAKE_TIMEOUT", "WAKE_ERROR", "IDLE_IRQS"):
            with self.subTest(key=key):
                text = self.transcript()
                text = re.sub(
                    rf"(IPCWAIT:{key}:)[0-9]{{20}}", rf"\g<1>{0:020d}", text)
                with self.assertRaises(ValueError):
                    probe.evaluate(text)

    def test_rejects_missing_duplicate_or_malformed_observations(self):
        good = self.transcript()
        for bad in (
            good.replace("IPCWAIT:WAKE_READY:00000000000000000002\n", ""),
            good + "IPCWAIT:WAKE_READY:00000000000000000002\n",
            good.replace("IPCWAIT:BLOCKED:00000000000000000004", "IPCWAIT:BLOCKED:4"),
            good.replace("IPCWAIT:CLEANUP:PASS\n", ""),
            good + "IPCWAIT:PASS\n",
            good.replace(f"IPCWAIT:PHASE_MASK:{0x3f:020d}", f"IPCWAIT:PHASE_MASK:{0x3e:020d}"),
        ):
            with self.subTest(text=bad), self.assertRaises(ValueError):
                probe.evaluate(bad)


if __name__ == "__main__":
    unittest.main()
