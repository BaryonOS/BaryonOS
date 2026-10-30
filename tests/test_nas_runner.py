"""Synthetic runner-verifier negatives, not evidence that the guest ran."""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import test_nas as runner


def transcript(case):
    numbers = [("ADMIN_PID", 1), ("GUEST_PID", 4)]
    counts = [(6, 1), (12, 6), (6, 5)] if case == "normal" else [(4, 0), (4, 4), (0, 4)]
    for slot, (ok, errors) in zip((0, 1, 3), counts):
        numbers.extend([("ROLE_SLOT", slot), ("RPC_OK", ok), ("RPC_ERRORS", errors),
                        ("RIGHTS_DENIED", int(slot == 3)), ("STACK_TOUCHED", 4096)])
    numbers.extend([("ENTROPY_BYTES", 256), ("DISK_READS", 1),
                    ("DISK_WRITES", int(case == "normal")),
                    ("DISK_FLUSHES", int(case == "normal"))])
    return "\n".join([*runner.BOOT, "NAS:MANIFEST:PASS",
                      *[f"NAS:{name}:{value:020d}" for name, value in numbers],
                      "NAS:BROKERS:PASS", "NAS:CLEANUP:PASS",
                      f"NAS:CASE:{case.upper()}:PASS", "M1:PASS"]) + "\n"


class NasRunnerTests(unittest.TestCase):
    def test_exact_normal_and_cold_evidence(self):
        for case in runner.FEATURES:
            result = runner.evaluate(case, transcript(case), 33, False)
            self.assertEqual((result["admin_pid"], result["guest_pid"]), (1, 4))

    def test_bad_exit_timeout_and_late_failure_rejected(self):
        serial = transcript("normal")
        for raw, timeout, text in [(0, False, serial), (True, False, serial),
                                   (33, True, serial), (33, False, serial + "PANIC\n"),
                                   (33, False, serial + "NAS:FAIL:late\n")]:
            with self.subTest(raw=raw, timeout=timeout, text=text[-20:]):
                with self.assertRaises(ValueError):
                    runner.evaluate("normal", text, raw, timeout)

    def test_missing_duplicated_or_changed_role_assertions_rejected(self):
        serial = transcript("normal")
        line = "NAS:RIGHTS_DENIED:00000000000000000001"
        mutations = [
            serial.replace(line, "NAS:RIGHTS_DENIED:00000000000000000000"),
            serial.replace(line, ""),
            serial.replace(line, line + "\n" + line),
            serial.replace("NAS:STACK_TOUCHED:00000000000000004096",
                           "NAS:STACK_TOUCHED:00000000000000065537"),
            serial.replace("NAS:GUEST_PID:00000000000000000004",
                           "NAS:GUEST_PID:00000000000000000001"),
            serial.replace("NAS:RPC_ERRORS:00000000000000000006",
                           "NAS:RPC_ERRORS:00000000000000000005"),
        ]
        for text in mutations:
            with self.assertRaises(ValueError):
                runner.evaluate("normal", text, 33, False)

    def test_cold_write_and_missing_entropy_or_read_rejected(self):
        serial = transcript("recover")
        for name, old, new in [("DISK_WRITES", 0, 1), ("DISK_FLUSHES", 0, 1),
                                ("DISK_READS", 1, 0), ("ENTROPY_BYTES", 256, 0)]:
            with self.subTest(name=name):
                text = serial.replace(f"NAS:{name}:{old:020d}", f"NAS:{name}:{new:020d}")
                with self.assertRaises(ValueError):
                    runner.evaluate("recover", text, 33, False)


if __name__ == "__main__":
    unittest.main()
