"""Host-only verdict tests for the B01-T2 requester-crash and no-reset-ack cases.
Synthetic transcripts and fixture records here are never emulator evidence;
they pin that the runner rejects what it must."""
from __future__ import annotations
from pathlib import Path
import re
import sys
import unittest

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))
import build  # noqa: E402
import run_keyring as rs  # noqa: E402
import test_keyring as gate  # noqa: E402

NEW = ("requester-crash", "no-reset-ack")


def transcript(case: str, values: dict[str, int], marker: str | None = None, extra: str = "") -> str:
    lines = list(rs.BOOT) + ["KEYRING:MANIFEST:PASS", "KEYRING:READY"]
    lines += [f"KEYRING:{key}:{value:020d}" for key, value in values.items()]
    lines += [f"KEYRING:CASE:{marker or rs.CASE_MARKER[case]}:PASS", "KEYRING:RESOURCES:PASS",
              "KEYRING:PASS", "M1:PASS"]
    return "\n".join(lines) + ("\n" + extra if extra else "") + "\n"


def good(case: str) -> dict[str, int]:
    values = dict(rs.EXPECT[case])
    values.update(rs.AT_LEAST[case])
    values.setdefault("TICKS", 75)
    return values


def fixture(lengths, error=None, response_bytes=0) -> dict:
    return {"backend": "silent-local-egd", "requested_lengths": list(lengths),
            "firmware_requested_lengths": [4, 4, 4, 4], "firmware_response_bytes": 16,
            "response_bytes": response_bytes, "error": error}


class Arguments(unittest.TestCase):
    def test_new_cases_require_the_silent_backend(self):
        for case in NEW:
            with self.subTest(case=case):
                with self.assertRaisesRegex(ValueError, "silent EGD"):
                    rs.qemu_arguments(case, "qemu", "fixture")
                arguments = rs.qemu_arguments(case, "qemu", "fixture", Path("/tmp/egd.sock"))
                self.assertIn("rng-egd,id=rng0,chardev=rng-channel", arguments)
                self.assertFalse(any("urandom" in argument for argument in arguments))


class Verdict(unittest.TestCase):
    def test_complete_transcripts_pass(self):
        for case in NEW:
            found = rs.evaluate(case, transcript(case, good(case)), 33, False)
            self.assertEqual(found["REPEAT_REFUSED"], 2)
            self.assertEqual(found["DEVICE_SUBMISSIONS"], 1)
            self.assertEqual(found["QUARANTINED"], 1)
            self.assertEqual(found["RESET_ACKS"], 0)
            self.assertEqual(found["DELIVERED_BYTES"], 0)

    def test_success_ack_replay_or_reset_ack_reject_by_name(self):
        bad = [(case, "DELIVERED_BYTES", 256) for case in NEW]
        bad += [(case, "QUARANTINED", 0) for case in NEW]
        bad += [(case, "RESET_ACKS", 1) for case in NEW]
        bad += [(case, "REPEAT_REFUSED", 1) for case in NEW]
        bad += [(case, "DEVICE_SUBMISSIONS", 2) for case in NEW]
        bad += [("requester-crash", "TIMEOUTS", 1), ("requester-crash", "REQUESTER_CRASHES", 0),
                ("requester-crash", "REQUESTER_CRASHES", 2),
                ("requester-crash", "CRASH_TICKET_REFUSED", 0),
                ("no-reset-ack", "TIMEOUTS", 0)]
        for case, key, value in bad:
            values = good(case)
            values[key] = value
            with self.subTest(case=case, key=key, value=value), self.assertRaisesRegex(ValueError, key):
                rs.evaluate(case, transcript(case, values), 33, False)

    def test_crash_evidence_is_required(self):
        for key in ("REQUESTER_CRASHES", "CRASH_TICKET_REFUSED", "REPEAT_REFUSED", "DEVICE_SUBMISSIONS"):
            values = good("requester-crash")
            del values[key]
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, key):
                rs.evaluate("requester-crash", transcript("requester-crash", values), 33, False)

    def test_duplicate_crash_line_rejects(self):
        text = transcript("requester-crash", good("requester-crash"),
                          extra="KEYRING:REQUESTER_CRASHES:00000000000000000001")
        with self.assertRaisesRegex(ValueError, "duplicate evidence REQUESTER_CRASHES"):
            rs.evaluate("requester-crash", text, 33, False)

    def test_wrong_marker_failure_line_exit_or_timeout_reject(self):
        values = good("requester-crash")
        with self.assertRaisesRegex(ValueError, "marker sequence"):
            rs.evaluate("requester-crash", transcript("requester-crash", values, "TIMEOUT"), 33, False)
        with self.assertRaisesRegex(ValueError, "marker sequence"):
            rs.evaluate("no-reset-ack", transcript("no-reset-ack", good("no-reset-ack"), "TIMEOUT"),
                        33, False)
        text = transcript("requester-crash", values, extra="KEYRING:FAIL:X")
        with self.assertRaisesRegex(ValueError, "failure marker"):
            rs.evaluate("requester-crash", text, 33, False)
        text = transcript("requester-crash", values)
        with self.assertRaisesRegex(ValueError, "raw exit"):
            rs.evaluate("requester-crash", text, 35, False)
        with self.assertRaisesRegex(ValueError, "raw exit"):
            rs.evaluate("requester-crash", text, 33, True)


class Fixture(unittest.TestCase):
    def test_only_whole_unanswered_request_pairs_pass(self):
        for case in NEW + ("timeout",):
            rs.check_fixture(case, fixture([255, 1]))
            rs.check_fixture(case, fixture([255, 1, 255, 1]))
            for lengths in ([], [32], [255], [1, 255], [255, 1, 32], [255, 1, 255]):
                with self.subTest(case=case, lengths=lengths), self.assertRaises(ValueError):
                    rs.check_fixture(case, fixture(lengths))
            with self.assertRaisesRegex(ValueError, "pending request"):
                rs.check_fixture(case, fixture([255, 1], error="ValueError: x"))
            with self.assertRaisesRegex(ValueError, "answered"):
                rs.check_fixture(case, fixture([255, 1], response_bytes=256))


class SilentBackend(unittest.TestCase):
    """Real AF_UNIX socket, no emulator: QEMU writes 255 and 1 separately."""

    def test_pair_split_across_stop_is_drained_not_cut(self):
        import socket
        import time
        with rs.SilentEgd() as egd:
            peer = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            peer.connect(str(egd.path))
            peer.sendall(bytes([2, 255]))
            self.assertTrue(egd.observed.wait(2))
            egd._stop.set()
            time.sleep(0.3)  # the worker has seen stop before the second command
            try:
                peer.sendall(bytes([2, 1]))
            except BrokenPipeError:
                pass  # a worker that stops on stop has already closed: the pair is cut
            peer.close()
        self.assertEqual(egd.requests, [255, 1])
        self.assertIsNone(egd.error)
        rs.check_fixture("timeout", egd.evidence())

    def test_drain_is_bounded_and_an_odd_tail_still_fails_closed(self):
        import socket
        self.assertLess(rs.DRAIN_SECONDS, 2)
        with rs.SilentEgd() as egd:
            peer = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            peer.connect(str(egd.path))
            peer.sendall(bytes([2, 255, 2]))
            self.assertTrue(egd.observed.wait(2))
            peer.close()
        self.assertIn("truncated", egd.error or "")
        with self.assertRaises(ValueError):
            rs.check_fixture("timeout", egd.evidence())


class Consistency(unittest.TestCase):
    def test_cases_are_registered_everywhere(self):
        for case in NEW:
            self.assertIn(case, build.KEYRING_CASES)
            self.assertIn(case, gate.PLAN)
            self.assertIn(case, rs.SILENT)
        for feature in rs.FEATURE.values():
            self.assertIn(feature, build.KEYRING_FEATURES)
        self.assertIn(["-s", "tools/tests"], [command[5:7] for command in gate.HOST])

    def test_new_features_keep_the_three_process_slot_count(self):
        # m3.rs boots three keyring processes only for the listed test features.
        cargo = (TOOLS.parent / "kernel/Cargo.toml").read_text(encoding="utf-8")
        for case in NEW:
            self.assertRegex(cargo, rf'(?m)^test-keyring-{case} = \["native-keyring", '
                                    r'"test-keyring-timeout"\]$')
        source = (TOOLS.parent / "kernel/src/m2/m3/keyring.rs").read_text(encoding="utf-8")
        body = re.search(r"fn case\(\) -> u64 \{(.*?)\n\}", source, re.S).group(1)
        order = [body.index(f'"test-keyring-{name}"') for name in
                 ("requester-crash", "no-reset-ack", "timeout")]
        self.assertEqual(order, sorted(order))


if __name__ == "__main__":
    unittest.main()
