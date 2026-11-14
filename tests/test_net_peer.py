"""Host protocol-counterparty/harness tests, never native OS acceptance."""
from __future__ import annotations

import copy
from contextlib import redirect_stdout, suppress
import io
import json
from pathlib import Path
import random
import socket
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import net_test_peer as peer
import run_net_system as runner
from run_system import network_arguments

ARP = bytes.fromhex("ffffffffffff02000000000708060001080006040001020000000007c0000202000000000000c0000201").ljust(60, b"\0")


def reference_checksum(data):
    total = 0
    for at in range(0, len(data), 2):
        total += data[at] << 8
        if at + 1 < len(data):
            total += data[at + 1]
        total = (total & 0xffff) + (total >> 16)
    return 0xffff ^ total


def query(name=peer.DNS_NAME, identifier=0x1234):
    wire = b"".join(bytes((len(label),)) + label.encode() for label in name.split(".")) + b"\0"
    return struct.pack("!6H", identifier, 0x0100, 1, 0, 0, 0) + wire + b"\0\x01\0\x01"


def client_udp(payload, target=peer.DNS_IP, port=53, source_port=49152):
    length = len(payload) + 8
    udp = struct.pack("!HHHH", source_port, port, length, 0) + payload
    pseudo = peer.CLIENT_IP + target + b"\0\x11" + length.to_bytes(2, "big")
    value = reference_checksum(pseudo + udp) or 0xffff
    udp = udp[:6] + value.to_bytes(2, "big") + udp[8:]
    ip = bytes.fromhex("450000001632400040110000") + peer.CLIENT_IP + target
    ip = ip[:2] + (20 + length).to_bytes(2, "big") + ip[4:]
    ip = ip[:10] + reference_checksum(ip).to_bytes(2, "big") + ip[12:]
    return (peer.PEER_MAC + peer.CLIENT_MAC + b"\x08\0" + ip + udp).ljust(60, b"\0")


def completed_fixture():
    machine = peer.PeerMachine()
    for _ in range(3):
        machine.handle(ARP)
    machine.handle(client_udp(b"dynamic-native-pid:\x05\0\0\0", peer.PEER_IP, 9000, 49153))
    for _ in range(3):
        machine.handle(client_udp(query()))
    machine.handle(client_udp(query(peer.TIMEOUT_NAME, 0x5678)))
    return machine


def native_report(**values):
    return dict(passed=True, evidence=dict(runner.NET_EXACT), kernel_feature="native-system-net",
                manifest=dict(system=dict(net_probe=dict(sha256="a" * 64))), **values)


class FramingTests(unittest.TestCase):
    def test_every_split_and_one_byte_stream_preserve_packet(self):
        encoded = peer.framed(ARP)
        for split in range(len(encoded) + 1):
            decoder = peer.FrameDecoder()
            self.assertEqual(decoder.feed(encoded[:split]) + decoder.feed(encoded[split:]), [ARP])
            decoder.finish()
        decoder = peer.FrameDecoder()
        result = []
        for byte in encoded:
            result.extend(decoder.feed(bytes((byte,))))
        self.assertEqual(result, [ARP])

    def test_lengths_rejected_before_payload_and_error_latches(self):
        for size in (0, 13, 1515, 65535, 0xffffffff):
            decoder = peer.FrameDecoder()
            with self.assertRaises(peer.ProtocolError):
                decoder.feed(struct.pack("!I", size))
            self.assertEqual(len(decoder.payload), 0)
            with self.assertRaises(peer.ProtocolError):
                decoder.feed(peer.framed(ARP))
        decoder = peer.FrameDecoder()
        decoder.feed(peer.framed(ARP)[:-1])
        with self.assertRaises(peer.ProtocolError):
            decoder.finish()

    def test_coalesced_frame_budget_and_unfinished_header_are_bounded(self):
        decoder = peer.FrameDecoder()
        self.assertEqual(len(decoder.feed(peer.framed(ARP) * peer.MAX_FRAMES)), peer.MAX_FRAMES)
        with self.assertRaises(peer.ProtocolError):
            decoder.feed(peer.framed(ARP))
        for length in (1, 2, 3):
            decoder = peer.FrameDecoder()
            decoder.feed(b"\0" * length)
            with self.assertRaises(peer.ProtocolError):
                decoder.finish()


class PacketTests(unittest.TestCase):
    def test_rfc_checksum_and_udp_payload_pseudoheader_corruption(self):
        self.assertEqual(peer.checksum(bytes.fromhex("0001f203f4f5f6f7")), 0x220d)
        data = client_udp(query())
        self.assertEqual(peer.parse_udp(data)["payload"], query())
        for at in range(14, len(data)):
            if at >= 14 + int.from_bytes(data[16:18], "big"):
                continue
            changed = bytearray(data)
            changed[at] ^= 1
            with self.assertRaises(peer.ProtocolError, msg=f"offset {at}"):
                peer.parse_udp(bytes(changed))

    def test_truncation_and_length_trailers_are_refused(self):
        data = client_udp(query())
        for cut in range(len(data)):
            with self.assertRaises(peer.ProtocolError):
                peer.parse_udp(data[:cut])
        with self.assertRaises(peer.ProtocolError):
            peer.parse_udp(data + b"\0")
        absent = bytearray(data)
        absent[40:42] = b"\0\0"
        with self.assertRaises(peer.ProtocolError):
            peer.parse_udp(bytes(absent))

    def test_dns_query_header_name_type_and_bounds(self):
        self.assertEqual(peer.parse_query(query())[:2], (0x1234, peer.DNS_NAME))
        for cut in range(len(query())):
            with self.assertRaises(peer.ProtocolError):
                peer.parse_query(query()[:cut])
        for at in (2, 5, 7, 9, 11, 12, len(query()) - 1):
            data = bytearray(query())
            data[at] ^= 0x80
            with self.assertRaises(peer.ProtocolError):
                peer.parse_query(bytes(data))
        with self.assertRaises(peer.ProtocolError):
            peer.parse_query(query() + b"\0")

    def test_server_schedule_is_driven_by_received_packets_and_echoes_input(self):
        machine = peer.PeerMachine()
        replies = [machine.handle(ARP)[0] for _ in range(3)]
        self.assertNotEqual(replies[0][6:12], replies[0][22:28])
        self.assertNotEqual(replies[1][38:42], peer.CLIENT_IP)
        self.assertEqual(replies[2], peer.arp_reply())
        payload = b"chosen-by-native-client:\x13\x00\x81"
        echo = machine.handle(client_udp(payload, peer.PEER_IP, 9000, 49153))[0]
        self.assertEqual(peer.parse_udp(echo)["payload"], payload)
        replies = [peer.parse_udp(machine.handle(client_udp(query()))[0])["payload"] for _ in range(3)]
        self.assertEqual(int.from_bytes(replies[0][:2], "big"), 0x1235)
        self.assertEqual(replies[1][len(query()):len(query()) + 4], b"\x01x" + struct.pack("!H", 0xc000 | len(query())))
        self.assertEqual(replies[2][-4:], peer.ANSWER_IP)
        self.assertEqual(machine.handle(client_udp(query(peer.TIMEOUT_NAME, 0x5678))), [])
        self.assertEqual(machine.counts, runner.PEER_EXACT)
        self.assertEqual([event["event"] for event in machine.events], list(runner.EVENTS))
        with self.assertRaises(peer.ProtocolError):
            machine.handle(ARP)

    def test_wrong_identity_target_arp_shape_and_unknown_destination_are_refused(self):
        for at in (0, 6, 14, 20, 22, 28, 32, 38):
            data = bytearray(ARP)
            data[at] ^= 1
            with self.assertRaises(peer.ProtocolError):
                peer.PeerMachine().handle(bytes(data))
        with self.assertRaises(peer.ProtocolError):
            peer.PeerMachine().handle(client_udp(b"no", peer.PEER_IP, 1234))

    def test_deterministic_hostile_bytes_do_not_escape_declared_parser_errors(self):
        randomizer = random.Random(0x4e49455557)
        for length in range(0, 1600):
            data = randomizer.randbytes(length)
            for parser in (peer.parse_udp, peer.parse_query, peer.PeerMachine().handle):
                with suppress(peer.ProtocolError):
                    parser(data)
        # A valid DNS header forces the hostile name/length parser paths too.
        for length in range(0, 245):
            data = query()[:12] + randomizer.randbytes(length)
            with suppress(peer.ProtocolError):
                peer.parse_query(data)


class GateTests(unittest.TestCase):
    def test_firmware_network_is_disabled_while_native_nic_remains_present(self):
        arguments = network_arguments(31337)
        for driver in ("VirtioNetSupport", "IPv4Support", "IPv6Support"):
            self.assertIn(f"name=opt/org.tianocore/{driver},string=no", arguments)
        self.assertIn("socket,id=net0,connect=127.0.0.1:31337", arguments)
        self.assertIn("virtio-net-pci,netdev=net0,id=net,addr=0x7,disable-legacy=on,mac=02:00:00:00:00:07,romfile=", arguments)
        for value in (True, 0, 1023, 65536, "31337"):
            with self.assertRaises(ValueError):
                network_arguments(value)

    def test_rejected_frame_diagnostics_preserve_failure_and_are_bounded(self):
        unknown = ARP[:12] + b"\x86\xdd" + b"\xab" * (peer.MAX_FRAME - 14)
        with peer.EthernetPeer(timeout=5) as server:
            with socket.create_connection(("127.0.0.1", server.port), timeout=2) as connection:
                connection.sendall(peer.framed(unknown))
                self.assertEqual(connection.recv(1), b"")
        evidence = server.evidence()
        self.assertEqual(evidence["error"], "ProtocolError: unsupported fixture EtherType")
        self.assertEqual(evidence["counts"]["accepted_frames"], 0)
        self.assertEqual(evidence["rejected_frame"]["ethertype"], "86dd")
        self.assertEqual(evidence["rejected_frame"]["length"], peer.MAX_FRAME)
        self.assertEqual(evidence["rejected_frame"]["prefix_hex"], unknown[:64].hex())
        self.assertEqual(len(evidence["rejected_frame"]["sha256"]), 64)

    def test_exact_native_and_independent_peer_evidence_required(self):
        machine = completed_fixture()
        report = native_report()
        evidence = dict(connected=True, error=None, counts=machine.counts, events=machine.events)
        runner.evaluate_network(report, evidence)
        for name in runner.NET_EXACT:
            changed = copy.deepcopy(report)
            changed["evidence"][name] += 1
            with self.assertRaises(ValueError):
                runner.evaluate_network(changed, evidence)
        for name in runner.PEER_EXACT:
            changed = copy.deepcopy(evidence)
            changed["counts"][name] -= 1
            with self.assertRaises(ValueError):
                runner.evaluate_network(report, changed)
        for key, value in (("connected", False), ("error", "bad frame"), ("events", [])):
            changed = {**evidence, key: value}
            with self.assertRaises(ValueError):
                runner.evaluate_network(report, changed)
        with self.assertRaises(ValueError):
            runner.evaluate_network({**report, "passed": False}, evidence)

    def test_duplicate_transcript_and_boolean_counter_cannot_pass(self):
        machine = completed_fixture()
        report = native_report()
        evidence = dict(connected=True, error=None, counts=machine.counts, events=machine.events)
        report["evidence"]["NET_ARP_REPLIES"] = True
        with self.assertRaises(ValueError):
            runner.evaluate_network(report, evidence)
        report["evidence"] = dict(runner.NET_EXACT)
        evidence["events"] = machine.events + machine.events[-1:]
        with self.assertRaises(ValueError):
            runner.evaluate_network(report, evidence)

    def test_localhost_socket_counterparty_receives_actual_framed_request(self):
        def receive_exact(connection, count):
            output = b""
            while len(output) < count:
                data = connection.recv(count - len(output))
                self.assertTrue(data)
                output += data
            return output
        with peer.EthernetPeer(timeout=5) as server:
            with socket.create_connection(("127.0.0.1", server.port), timeout=2) as connection:
                connection.sendall(peer.framed(ARP))
                length = struct.unpack("!I", receive_exact(connection, 4))[0]
                self.assertEqual(receive_exact(connection, length), peer.arp_reply(bad_sender=True))
        evidence = server.evidence()
        self.assertTrue(evidence["connected"])
        self.assertIsNone(evidence["error"])
        self.assertEqual(evidence["counts"]["accepted_frames"], 1)

    def test_failed_network_or_shutdown_cannot_leave_passing_cold_stage(self):
        machine = completed_fixture()

        class FakePeer:
            port = 31337
            shutdown_error = False
            def __enter__(self): return self
            def __exit__(self, *_):
                if self.shutdown_error:
                    raise RuntimeError("peer shutdown fixture")
            def evidence(self):
                return dict(connected=True, error=None, counts=copy.deepcopy(machine.counts),
                            events=copy.deepcopy(machine.events))

        for problem in ("none", "missing-counter", "source-drift", "shutdown"):
            with self.subTest(problem=problem), tempfile.TemporaryDirectory(prefix="nieuw-os-net-harness-") as temporary:
                build = Path(temporary) / "build"
                run_dir = build / "run" / "system-normal-test"
                volume_dir = build / "run" / "volume-test"
                run_dir.mkdir(parents=True)
                volume_dir.mkdir()
                report = native_report(case="normal", source_sha256="frozen", run_directory=str(run_dir))
                if problem == "missing-counter":
                    del report["evidence"]["NET_DNS_ANSWERS"]
                fake = FakePeer()
                fake.shutdown_error = problem == "shutdown"
                with patch.object(runner, "BUILD", build), patch.object(runner, "EthernetPeer", return_value=fake), \
                        patch.object(runner, "source_hash", return_value="drift" if problem == "source-drift" else "frozen"), \
                        patch.object(runner, "run_system", return_value=(report, volume_dir)) as generic:
                    # These are host-only wrapper mocks, never guest evidence.
                    with redirect_stdout(io.StringIO()):
                        result, _ = runner.run_net_system("normal", timeout=5)
                self.assertIs(result["passed"], problem == "none")
                self.assertIs(result["network_gate"], problem == "none")
                saved = json.loads((volume_dir / "last-system-stage.json").read_text())
                self.assertIs(saved["passed"], problem == "none")
                generic.assert_called_once_with("normal", None, timeout=5,
                                                kernel_feature="native-system-net", network_port=31337)

    def test_wrong_profile_manifest_dns_transcript_and_unsafe_paths_fail(self):
        machine = completed_fixture()
        evidence = dict(connected=True, error=None, counts=machine.counts, events=machine.events)
        for change in (dict(kernel_feature="native-system"), dict(manifest={})):
            with self.assertRaises(ValueError):
                runner.evaluate_network({**native_report(), **change}, evidence)
        changed = copy.deepcopy(evidence)
        changed["events"][4]["question_name"] = "other.test"
        with self.assertRaises(ValueError):
            runner.evaluate_network(native_report(), changed)
        with tempfile.TemporaryDirectory(prefix="nieuw-os-net-path-") as temporary:
            base = Path(temporary)
            build = base / "build"
            (build / "run").mkdir(parents=True)
            outside = base / "outside"
            outside.mkdir()
            with patch.object(runner, "BUILD", build), self.assertRaises(ValueError):
                runner.store_report(dict(run_directory=str(outside)), outside)
            self.assertEqual(list(outside.iterdir()), [])
            run_dir = build / "run" / "generated"
            volume_dir = build / "run" / "volume"
            run_dir.mkdir()
            volume_dir.mkdir()
            target = outside / "untouched"
            target.write_text("preserve")
            (run_dir / "result.json").symlink_to(target)
            with patch.object(runner, "BUILD", build), self.assertRaises(ValueError):
                runner.store_report(dict(run_directory=str(run_dir)), volume_dir)
            self.assertEqual(target.read_text(), "preserve")
        with patch.object(runner, "EthernetPeer") as factory:
            for case, timeout in (("other", 1), ("normal", 0), ("normal", 181)):
                with self.assertRaises(ValueError):
                    runner.run_net_system(case, timeout=timeout)
            factory.assert_not_called()

    def test_partial_socket_frame_is_recorded_as_protocol_failure(self):
        def exact(connection, count):
            output = b""
            while len(output) < count:
                data = connection.recv(count - len(output))
                self.assertTrue(data)
                output += data
            return output
        with peer.EthernetPeer(timeout=5) as server:
            with socket.create_connection(("127.0.0.1", server.port), timeout=2) as connection:
                connection.sendall(peer.framed(ARP))
                length = struct.unpack("!I", exact(connection, 4))[0]
                exact(connection, length)
                connection.sendall(b"\0\0")
            server.thread.join(1)
            self.assertFalse(server.thread.is_alive())
        self.assertIn("incomplete/bad frame", server.evidence()["error"])


if __name__ == "__main__":
    unittest.main()
