"""Host-only constructor and failure-report regressions; no native VM evidence."""
import io
import json
import os
from pathlib import Path
import sys
import time
import unittest.mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import run_graphics
import run_net_system


class DeferredSocket(run_graphics.Qmp):
    """Overrides depend on state initialized only after the parent constructor."""
    def __init__(self, path, deadline, process):
        super().__init__(path, deadline, process)
        self.ready = True
        self.dispatched = []

    def read(self):
        self.dispatched.append("read")
        return super().read()

    def send(self, data):
        self.dispatched.append("send")
        super().send(data)

    def command(self, name, arguments=None):
        self.dispatched.append("command")
        return super().command(name, arguments)

    def close(self):
        self.dispatched.append("close")
        super().close()


class QmpConstruction(unittest.TestCase):
    def socket_fixture(self, *responses):
        stream = io.BytesIO(b"".join(json.dumps(item).encode() + b"\n" for item in responses))
        transport = unittest.mock.Mock()
        transport.makefile.return_value = stream
        return transport, stream

    def test_bootstrap_does_not_dispatch_before_subclass_state_exists(self):
        transport, stream = self.socket_fixture(
            {"QMP": {}}, {"return": {}, "id": 1}, {"return": {"status": "running"}, "id": 2})
        with unittest.mock.patch.object(run_graphics.socket, "socket", return_value=transport):
            control = DeferredSocket("synthetic-qmp", time.monotonic() + 10, unittest.mock.Mock())
        try:
            self.assertTrue(control.ready)
            self.assertEqual(control.dispatched, [])
            self.assertEqual(control.sequence, 1)
            self.assertEqual(control.trace[0]["request"], {"execute": "qmp_capabilities", "id": 1})
            self.assertEqual(control.command("query-status"), {"status": "running"})
            self.assertEqual(control.dispatched, ["command", "send", "read"])
        finally:
            control.close()
        self.assertTrue(stream.closed)
        transport.close.assert_called_once()

    def test_failed_bootstrap_closes_own_transport_without_subclass_dispatch(self):
        for replies, message in [([{}], "Missing QMP greeting"),
                                 ([{"QMP": {}}, {"error": {"class": "GenericError"}, "id": 1}],
                                  "QMP command failed")]:
            with self.subTest(message=message):
                transport, stream = self.socket_fixture(*replies)
                with unittest.mock.patch.object(run_graphics.socket, "socket", return_value=transport):
                    with self.assertRaisesRegex((ValueError, RuntimeError), message):
                        DeferredSocket("synthetic-qmp", time.monotonic() + 10, unittest.mock.Mock())
                self.assertTrue(stream.closed)
                transport.close.assert_called_once()

    def test_pipe_initializes_protocol_without_opening_a_socket(self):
        read_fd, write_fd = os.pipe()
        command_read, command_write = os.pipe()
        reader = os.fdopen(read_fd, "rb", buffering=0)
        writer = os.fdopen(command_write, "wb", buffering=0)
        try:
            os.write(write_fd, b'{"QMP":{}}\n{"return":{},"id":1}\n')
            with unittest.mock.patch.object(run_graphics.socket, "socket", side_effect=AssertionError("socket opened")):
                control = run_graphics.QmpPipe(unittest.mock.Mock(stdout=reader, stdin=writer), time.monotonic() + 10)
            try:
                self.assertEqual(control.sequence, 1)
                self.assertEqual(json.loads(os.read(command_read, 4096)),
                                 {"execute": "qmp_capabilities", "id": 1})
            finally:
                control.close()
            self.assertTrue(reader.closed)
            self.assertTrue(writer.closed)
        finally:
            reader.close()
            writer.close()
            os.close(write_fd)
            os.close(command_read)


class NetworkFailureEvidence(unittest.TestCase):
    def test_peer_shutdown_and_evidence_failures_preserve_failed_report(self):
        for failure_stage in ("__exit__", "evidence"):
            with self.subTest(failure_stage=failure_stage):
                peer = unittest.mock.MagicMock()
                peer.__enter__.return_value = peer
                peer.__exit__.return_value = False
                getattr(peer, failure_stage).side_effect = RuntimeError("synthetic peer failure")
                report = {"passed": True}
                directory = Path("synthetic-volume")
                with unittest.mock.patch.object(run_net_system, "EthernetPeer", return_value=peer), \
                     unittest.mock.patch.object(run_net_system, "run_system", return_value=(report, directory)), \
                     unittest.mock.patch.object(run_net_system, "store_report") as store:
                    result, actual_directory = run_net_system.run_net_system("normal")
                self.assertIs(result, report)
                self.assertEqual(actual_directory, directory)
                self.assertFalse(result["passed"])
                self.assertFalse(result["network_gate"])
                self.assertIn("synthetic peer failure", result["network_error"])
                store.assert_called_once_with(report, directory)


if __name__ == "__main__":
    unittest.main()
