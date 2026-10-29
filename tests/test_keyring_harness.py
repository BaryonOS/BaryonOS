"""Host-only parser tests for the B01 runner. Synthetic transcripts here are
never emulator evidence; they pin that the verdict rejects what it must."""
from __future__ import annotations
import sys
import socket
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import run_keyring as rs  # noqa: E402
import test_keyring as gate  # noqa: E402


def transcript(case: str, values: dict[str, int], extra: str = "") -> str:
    lines = list(rs.BOOT) + ["KEYRING:MANIFEST:PASS", "KEYRING:READY"]
    lines += [f"KEYRING:{key}:{value:020d}" for key, value in values.items()]
    lines += [f"KEYRING:CASE:{rs.CASE_MARKER[case]}:PASS", "KEYRING:RESOURCES:PASS",
              "KEYRING:PASS", "M1:PASS"]
    return "\n".join(lines) + ("\n" + extra if extra else "") + "\n"


def good(case: str) -> dict[str, int]:
    values = dict(rs.EXPECT[case])
    values.update(rs.AT_LEAST[case])
    values.setdefault("TICKS", 1234)
    return values


class Numbers(unittest.TestCase):
    def test_numbers_require_twenty_digits_and_reject_duplicates(self):
        self.assertEqual(rs.numbers("KEYRING:REQUESTS:00000000000000000003\n"), {"REQUESTS": 3})
        self.assertEqual(rs.numbers("KEYRING:REQUESTS:3\n"), {})
        with self.assertRaises(ValueError):
            rs.numbers("KEYRING:REQUESTS:00000000000000000003\nKEYRING:REQUESTS:00000000000000000004\n")

    def test_every_case_has_a_marker_and_expectations(self):
        for case in rs.FEATURE:
            self.assertIn(case, rs.CASE_MARKER)
            self.assertIn(case, rs.EXPECT)
            self.assertIn(case, rs.AT_LEAST)
            self.assertEqual(rs.expected_markers(case)[-1], "M1:PASS")

    def test_gate_requires_every_native_case(self):
        self.assertEqual(set(gate.PLAN), set(rs.FEATURE))


class EntropyFixture(unittest.TestCase):
    def test_silent_backend_observes_requests_without_answering(self):
        with rs.SilentEgd() as fixture:
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
                client.settimeout(0.1)
                client.connect(str(fixture.path))
                client.sendall(bytes([2, 255, 2, 1]))
                self.assertTrue(fixture.observed.wait(timeout=2))
                with self.assertRaises(socket.timeout):
                    client.recv(1)
        self.assertEqual(fixture.evidence(), {"backend": "silent-local-egd",
                         "requested_lengths": [255, 1], "firmware_requested_lengths": [],
                         "firmware_response_bytes": 0, "response_bytes": 0, "error": None})
        self.assertFalse(fixture.path.exists())

    def test_firmware_works_but_manifest_stops_entropy_before_next_request(self):
        import tempfile
        with tempfile.TemporaryDirectory() as temporary:
            serial = Path(temporary) / "serial.txt"
            with rs.SilentEgd(serial) as fixture:
                with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
                    client.settimeout(0.1)
                    client.connect(str(fixture.path))
                    client.sendall(bytes([2, 32]))
                    # Real host entropy can bootstrap OVMF. No fixed seed is
                    # supplied to the device or the cryptographic service.
                    answer = b""
                    while len(answer) < 32:
                        answer += client.recv(32 - len(answer))
                    self.assertEqual(len(answer), 32)
                    serial.write_text("KEYRING:MANIFEST:PASS\n", encoding="utf-8")
                    client.sendall(bytes([2, 255, 2, 1]))
                    self.assertTrue(fixture.observed.wait(timeout=2))
                    with self.assertRaises(socket.timeout):
                        client.recv(1)
            self.assertEqual(fixture.requests, [255, 1])
            self.assertEqual(fixture.firmware_requests, [32])
            self.assertIsNone(fixture.error)

    def test_timeout_arguments_cannot_use_a_responding_backend(self):
        with self.assertRaisesRegex(ValueError, "silent EGD"):
            rs.qemu_arguments("timeout", "qemu", "fixture")
        arguments = rs.qemu_arguments("timeout", "qemu", "fixture", Path("/tmp/egd.sock"))
        self.assertIn("rng-egd,id=rng0,chardev=rng-channel", arguments)
        self.assertFalse(any("urandom" in argument for argument in arguments))
        self.assertIn("none", arguments)  # no guest network


class Verdict(unittest.TestCase):
    def test_complete_transcripts_pass_for_every_case(self):
        for case in rs.FEATURE:
            found = rs.evaluate(case, transcript(case, good(case)), 33, False)
            self.assertEqual(found["TICKS"], 1234)

    def test_failure_marker_rejects_even_with_all_pass_markers(self):
        text = transcript("normal", good("normal"), "KEYRING:FAIL:LATE")
        with self.assertRaisesRegex(ValueError, "failure marker"):
            rs.evaluate("normal", text, 33, False)
        text = transcript("normal", good("normal"), "M2:PANIC_LINE:0000000012")
        with self.assertRaisesRegex(ValueError, "failure marker"):
            rs.evaluate("normal", text, 33, False)

    def test_wrong_exit_timeout_and_marker_order_reject(self):
        text = transcript("no-device", good("no-device"))
        with self.assertRaisesRegex(ValueError, "raw exit"):
            rs.evaluate("no-device", text, 35, False)
        with self.assertRaisesRegex(ValueError, "raw exit"):
            rs.evaluate("no-device", text, 33, True)
        reordered = text.replace("KEYRING:READY\n", "").replace("KEYRING:PASS\n",
                                                                   "KEYRING:PASS\nKEYRING:READY\n")
        with self.assertRaisesRegex(ValueError, "marker sequence"):
            rs.evaluate("no-device", reordered, 33, False)
        with self.assertRaisesRegex(ValueError, "marker sequence"):
            rs.evaluate("stuck", transcript("normal", good("normal")), 33, False)

    def test_each_expected_number_is_load_bearing(self):
        for case in rs.FEATURE:
            for key in rs.EXPECT[case]:
                values = good(case)
                values[key] += 1
                with self.assertRaisesRegex(ValueError, key):
                    rs.evaluate(case, transcript(case, values), 33, False)
            for key, minimum in rs.AT_LEAST[case].items():
                values = good(case)
                values[key] = minimum - 1
                with self.assertRaisesRegex(ValueError, key):
                    rs.evaluate(case, transcript(case, values), 33, False)
            values = good(case)
            del values["TICKS"]
            with self.assertRaisesRegex(ValueError, "tick budget"):
                rs.evaluate(case, transcript(case, values), 33, False)

    def test_missing_number_rejects(self):
        values = good("stuck")
        del values["SERVICE_A_STATE"]
        with self.assertRaisesRegex(ValueError, "SERVICE_A_STATE"):
            rs.evaluate("stuck", transcript("stuck", values), 33, False)


if __name__ == "__main__":
    unittest.main()
