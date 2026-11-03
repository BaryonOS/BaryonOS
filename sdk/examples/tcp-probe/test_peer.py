"""Host tests of the synthetic TCP SERVER, never native client acceptance."""
from __future__ import annotations

import hashlib
from pathlib import Path
import random
import socket
import struct
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import peer


def reference_checksum(data):
    value = 0
    for at in range(0, len(data), 2):
        value += data[at] << 8
        if at + 1 < len(data):
            value += data[at + 1]
        value = (value & 0xffff) + (value >> 16)
    return value ^ 0xffff


def client(port=8080, *, seq=0xfffffff0, ack=0, flags=peer.SYN, payload=b"",
           options=b"", window=4096, identification=1, source=None):
    """Independent wire fixture, not an OS implementation or test result."""
    if source is None:
        source = 40000 + peer.PORTS.index(port)
    tcp = struct.pack("!HHIIBBHHH", source, port, seq & peer.MASK, ack & peer.MASK,
                      ((20 + len(options)) // 4) << 4, flags, window, 0, 0) + options + payload
    pseudo = peer.CLIENT_IP + peer.PEER_IP + bytes((0, 6)) + len(tcp).to_bytes(2, "big")
    tcp = tcp[:16] + reference_checksum(pseudo + tcp).to_bytes(2, "big") + tcp[18:]
    ip = bytes((0x45, 0)) + (20 + len(tcp)).to_bytes(2, "big")
    ip += identification.to_bytes(2, "big") + bytes.fromhex("400040060000") + peer.CLIENT_IP + peer.PEER_IP
    ip = ip[:10] + reference_checksum(ip).to_bytes(2, "big") + ip[12:]
    return (peer.PEER_MAC + peer.CLIENT_MAC + b"\x08\0" + ip + tcp).ljust(60, b"\0")


def server(packet, *, valid=True):
    assert packet[:6] == peer.CLIENT_MAC and packet[6:12] == peer.PEER_MAC
    assert packet[12:14] == b"\x08\0"
    ip = packet[14:]
    assert reference_checksum(ip[:20]) == 0
    total = int.from_bytes(ip[2:4], "big")
    tcp = ip[20:total]
    pseudo = peer.PEER_IP + peer.CLIENT_IP + b"\0\x06" + len(tcp).to_bytes(2, "big")
    assert (reference_checksum(pseudo + tcp) == 0) is valid
    fields = struct.unpack_from("!HHIIBBHHH", tcp)
    return dict(source=fields[0], target=fields[1], seq=fields[2], ack=fields[3],
                flags=fields[5], payload=tcp[(fields[4] >> 4) * 4:])


ARP = bytes.fromhex("ffffffffffff02000000000708060001080006040001020000000007c0000202000000000000c0000201").ljust(60, b"\0")


def connect(machine, port, now=0):
    frame = client(port, options=b"\x02\x04\x05\xb4")
    result = machine.handle(frame, now)
    if port == 8083:
        assert result == []
        result = machine.handle(client(port, options=b"\x02\x04\x05\xb4", identification=2), now + 1)
    if port in (8081, 8084) or (port == 8085 and machine.scenario == "baseline"):
        return result
    packet = server(result[0])
    assert packet["flags"] == peer.SYN | peer.ACK and packet["ack"] == 0xfffffff1
    machine.handle(client(port, seq=0xfffffff1, ack=packet["seq"] + 1, flags=peer.ACK), now + 1)
    return packet["seq"] + 1


def request(machine, port, base, now=2):
    return machine.handle(client(port, seq=0xfffffff1, ack=base, flags=peer.ACK | peer.PSH,
                                 payload=peer.REQUEST), now)


def finish_http(machine, port, base, packets, now=2, request_length=None):
    if port == 8080:
        malformed = server(packets[0], valid=False)
        assert malformed["payload"] == peer.CORRUPT_RESPONSE
        assert malformed["payload"] != peer.RESPONSE
        assert len(malformed["payload"]) == len(peer.RESPONSE)
        assert machine.poll(now + peer.CORRUPT_DELAY / 2) == []
        packets = machine.poll(now + peer.CORRUPT_DELAY)
    data = server(packets[0])
    assert data["payload"] == peer.RESPONSE and data["seq"] == base
    if request_length is None:
        request_length = len(peer.BACKPRESSURE_REQUEST) if port == 8085 else len(peer.REQUEST)
    next_client = (0xfffffff1 + request_length) & peer.MASK
    end = (base + len(peer.RESPONSE)) & peer.MASK
    fin = machine.handle(client(port, seq=next_client, ack=end, flags=peer.ACK), now + 1)
    assert server(fin[0])["flags"] == peer.FIN | peer.ACK
    last = machine.handle(client(port, seq=next_client, ack=end + 1, flags=peer.FIN | peer.ACK), now + 2)
    assert server(last[0])["ack"] == (next_client + 1) & peer.MASK
    assert server(last[0])["flags"] == peer.ACK


def backpressure(machine, base):
    sequence = 0xfffffff1
    first = peer.BACKPRESSURE_REQUEST[:512]
    original = client(8085, seq=sequence, ack=base, flags=peer.ACK | peer.PSH, payload=first)
    assert machine.handle(original, 10) == []
    buffered = peer.BACKPRESSURE_REQUEST[512:1460]
    assert machine.handle(client(8085, seq=sequence + 512, ack=base,
                                 flags=peer.ACK | peer.PSH, payload=buffered,
                                 identification=2), 11) == []
    retransmit = client(8085, seq=sequence, ack=base, flags=peer.ACK,
                        payload=peer.BACKPRESSURE_REQUEST[:1460], identification=3)
    release = machine.handle(retransmit, 11)
    assert len(release) == 1
    assert server(release[0])["ack"] == (sequence + 1460) & peer.MASK
    at = 1460
    while at < len(peer.BACKPRESSURE_REQUEST):
        data = peer.BACKPRESSURE_REQUEST[at:at + 1024]
        packets = machine.handle(client(8085, seq=sequence + at, ack=base,
                                        flags=peer.ACK | peer.PSH, payload=data,
                                        identification=4 + at // 1024), 12 + at)
        at += len(data)
        if at < len(peer.BACKPRESSURE_REQUEST):
            assert len(packets) == 1 and server(packets[0])["ack"] == (sequence + at) & peer.MASK
        else:
            assert len(packets) == 1 and server(packets[0])["payload"] == peer.RESPONSE
            finish_http(machine, 8085, base, packets, now=30, request_length=len(peer.BACKPRESSURE_REQUEST))


def first_three(machine):
    base = connect(machine, 8080)
    finish_http(machine, 8080, base, request(machine, 8080, base))
    refused = server(connect(machine, 8081)[0])
    assert refused["flags"] == peer.RST | peer.ACK
    base = connect(machine, 8082)
    reset = server(request(machine, 8082, base)[0])
    assert reset["flags"] == peer.RST | peer.ACK and not reset["payload"]


class FramingTests(unittest.TestCase):
    def test_splits_single_bytes_coalescing_and_payload_bounds(self):
        data = peer.framed(ARP)
        for at in range(len(data) + 1):
            decoder = peer.FrameDecoder()
            self.assertEqual(decoder.feed(data[:at]) + decoder.feed(data[at:]), [ARP])
            decoder.finish()
        decoder = peer.FrameDecoder()
        result = []
        for byte in data:
            result.extend(decoder.feed(bytes((byte,))))
        self.assertEqual(result, [ARP])
        for size in (0, 13, peer.MAX_FRAME + 1, 0xffffffff):
            decoder = peer.FrameDecoder()
            with self.assertRaises(peer.ProtocolError):
                decoder.feed(size.to_bytes(4, "big"))
            self.assertEqual(len(decoder.payload), 0)
            with self.assertRaises(peer.ProtocolError):
                decoder.feed(data)

    def test_budget_and_incomplete_stream_are_failures(self):
        decoder = peer.FrameDecoder()
        self.assertEqual(len(decoder.feed(peer.framed(ARP) * peer.MAX_FRAMES)), peer.MAX_FRAMES)
        with self.assertRaises(peer.ProtocolError):
            decoder.feed(peer.framed(ARP))
        for cut in (1, 2, 3, 8, 20):
            decoder = peer.FrameDecoder()
            decoder.feed(peer.framed(ARP)[:cut])
            with self.assertRaises(peer.ProtocolError):
                decoder.finish()


class ParserTests(unittest.TestCase):
    def test_checksum_known_vector_and_correct_tuple(self):
        self.assertEqual(peer.checksum(bytes.fromhex("0001f203f4f5f6f7")), 0x220d)
        segment = peer.parse_tcp(client(options=b"\x02\x04\x05\xb4"))
        self.assertEqual((segment.source, segment.target, segment.seq, segment.flags),
                         (40000, 8080, 0xfffffff0, peer.SYN))

    def test_truncations_extra_trailer_corruption_and_identity(self):
        frame = client(payload=b"test")
        for cut in range(len(frame)):
            if cut == 14 + int.from_bytes(frame[16:18], "big"):
                continue  # Stripped Ethernet padding is a valid frame.
            with self.assertRaises(peer.ProtocolError):
                peer.parse_tcp(frame[:cut])
        for at in range(14, 14 + int.from_bytes(frame[16:18], "big")):
            bad = bytearray(frame)
            bad[at] ^= 1
            with self.assertRaises(peer.ProtocolError, msg=str(at)):
                peer.parse_tcp(bytes(bad))
        for at in (0, 6, 12):
            bad = bytearray(frame)
            bad[at] ^= 1
            with self.assertRaises(peer.ProtocolError):
                peer.parse_tcp(bytes(bad))
        with self.assertRaises(peer.ProtocolError):
            peer.parse_tcp(frame + b"\0")
        with self.assertRaises(peer.ProtocolError):
            peer.parse_tcp(client(source=40001))

    def test_options_and_data_budget_are_checked(self):
        for options in (b"\x02\x01\0\0", b"\x02\x08\0\0", b"\0\x01\0\0", b"\x02\x04\0\0"):
            with self.assertRaises(peer.ProtocolError):
                peer.parse_tcp(client(options=options))
        with self.assertRaises(peer.ProtocolError):
            peer.parse_tcp(client(payload=b"x" * (peer.MAX_PAYLOAD + 1)))

    def test_hostile_bytes_never_escape_declared_errors(self):
        generator = random.Random(0x544350)
        for size in range(0, 1600):
            packet = generator.randbytes(size)
            try:
                peer.parse_tcp(packet)
            except peer.ProtocolError:
                pass


class ConversationTests(unittest.TestCase):
    def test_native_smoltcp_broadcast_target_arp_preserves_all_identity_checks(self):
        # Exact 60-byte first frame from the failed native run 0f59050c...
        # RFC 826 Packet Generation leaves the unknown target hardware address
        # unspecified: https://www.rfc-editor.org/rfc/rfc826.html
        native = bytes.fromhex("ffffffffffff02000000000708060001080006040001020000000007c0000202"
                               "ffffffffffffc0000201000000000000000000000000000000000000")
        self.assertEqual(len(native), 60)
        self.assertEqual(native, ARP[:32] + b"\xff" * 6 + ARP[38:])
        for request_frame in (ARP, ARP[:32] + peer.PEER_MAC + ARP[38:], native):
            with self.subTest(target_hardware=request_frame[32:38].hex()):
                machine = peer.PeerMachine()
                replies = machine.handle(request_frame)
                expected = (peer.CLIENT_MAC + peer.PEER_MAC + b"\x08\x06"
                            + struct.pack("!HHBBH", 1, 0x800, 6, 4, 2)
                            + peer.PEER_MAC + peer.PEER_IP + peer.CLIENT_MAC + peer.CLIENT_IP).ljust(60, b"\0")
                self.assertEqual(replies, [expected])
                self.assertEqual(machine.counts["arp_requests"], 1)
                self.assertEqual(machine.counts["arp_replies"], 1)
                self.assertEqual(machine.events[0]["frame_sha256"], hashlib.sha256(request_frame).hexdigest())
                self.assertFalse(machine.summary()["complete"])
        for offset in (6, 21, 22, 28, 32, 41):
            damaged = bytearray(native)
            damaged[offset] ^= 1
            machine = peer.PeerMachine()
            with self.subTest(offset=offset), self.assertRaises(peer.ProtocolError):
                machine.handle(bytes(damaged))
            self.assertEqual(machine.events, [])
            self.assertTrue(all(value == 0 for value in machine.counts.values()))
        # The new broadcast-THA variant applies only to L2 broadcast requests.
        with self.assertRaises(peer.ProtocolError):
            peer.PeerMachine().handle(peer.PEER_MAC + native[6:])

    def test_all_five_server_cases_require_real_causal_packets(self):
        machine = peer.PeerMachine()
        replies = machine.handle(ARP)
        self.assertEqual(replies[0][38:42], peer.CLIENT_IP)
        first_three(machine)
        base = connect(machine, 8083)
        finish_http(machine, 8083, base, request(machine, 8083, base))
        self.assertEqual(connect(machine, 8084), [])
        self.assertEqual(machine.handle(client(8084, options=b"\x02\x04\x05\xb4"), 4), [])
        self.assertEqual(machine.handle(client(8084, flags=peer.RST, seq=0xfffffff1), 5), [])
        disabled = connect(machine, 8085)
        self.assertEqual(server(disabled[0])["flags"], peer.RST | peer.ACK)
        result = machine.summary()
        self.assertTrue(result["complete"])
        self.assertEqual(result["counts"]["first_syn_dropped"], 1)
        self.assertEqual(result["counts"]["valid_data_sent"], 2)
        self.assertEqual(result["counts"]["corrupt_data_sent"], 1)
        self.assertEqual(result["counts"]["client_fins_acked"], 2)
        self.assertEqual(result["counts"]["backpressure_disabled"], 1)
        self.assertEqual(result["response_body_bytes"], 20)
        self.assertEqual(result["response_sha256"], "d128fa1924be70f9e1f42e2a466e1e94123182b23b11110d273ced90cefda90b")
        self.assertEqual([event["sequence"] for event in result["events"]],
                         list(range(1, len(result["events"]) + 1)))
        self.assertFalse(peer.PeerMachine().summary()["complete"])

    def test_opt_in_data_ack_loss_requires_matching_overlap_and_request_recovery(self):
        machine = peer.PeerMachine("loss-backpressure")
        machine.handle(ARP)
        first_three(machine)
        base = connect(machine, 8083)
        finish_http(machine, 8083, base, request(machine, 8083, base))
        self.assertEqual(connect(machine, 8084), [])
        machine.handle(client(8084, options=b"\x02\x04\x05\xb4"), 4)
        machine.handle(client(8084, flags=peer.RST, seq=0xfffffff1), 5)
        base = connect(machine, 8085)
        backpressure(machine, base)
        result = machine.summary()
        self.assertTrue(result["complete"])
        self.assertEqual(result["counts"]["backpressure_acks_withheld"], 1)
        self.assertEqual(result["counts"]["backpressure_retransmissions"], 1)
        self.assertEqual(result["counts"]["backpressure_acks_released"], 1)
        self.assertGreater(result["counts"]["backpressure_recovery_segments"], 0)
        self.assertEqual(result["cases"]["8085"]["request_bytes"], 8192)
        machine = peer.PeerMachine("loss-backpressure")
        connection = peer.Connection(8085, 40005, 0xfffffff0, b"", peer.SYN,
                                     0x6e6f7300 + 5 * 0x1000, "backpressure-request")
        first = peer.BACKPRESSURE_REQUEST[:512]
        machine.backpressure_request(connection, peer.Segment(
            40005, 8085, 0xfffffff1, connection.server_start, peer.ACK | peer.PSH,
            4096, b"", first))
        changed = first[:-1] + b"Y"
        with self.assertRaisesRegex(peer.ProtocolError, "changed or repeated"):
            machine.backpressure_request(connection, peer.Segment(
                40005, 8085, 0xfffffff1, connection.server_start, peer.ACK | peer.PSH,
                4096, b"", changed))
        machine.backpressure_request(connection, peer.Segment(
            40005, 8085, (0xfffffff1 + 512) & peer.MASK,
            connection.server_start, peer.ACK | peer.PSH,
            4096, b"", peer.BACKPRESSURE_REQUEST[512:1460]))
        release = machine.backpressure_request(connection, peer.Segment(
            40005, 8085, 0xfffffff1, connection.server_start, peer.ACK,
            4096, b"", peer.BACKPRESSURE_REQUEST[:1460]))
        self.assertEqual(server(release[0])["ack"], (0xfffffff1 + 1460) & peer.MASK)

    def test_client_cannot_ack_corrupt_segment_before_valid_data(self):
        machine = peer.PeerMachine()
        base = connect(machine, 8080)
        request(machine, 8080, base)
        with self.assertRaisesRegex(peer.ProtocolError, "corrupt"):
            machine.handle(client(seq=0xfffffff1 + len(peer.REQUEST), ack=base + len(peer.RESPONSE), flags=peer.ACK), 2.1)
        self.assertEqual(machine.counts["valid_data_sent"], 0)

    def test_fin_waits_for_valid_data_ack_and_acknowledges_client_fin(self):
        machine = peer.PeerMachine()
        base = connect(machine, 8080)
        packets = request(machine, 8080, base)
        self.assertEqual(machine.counts["fins_sent"], 0)
        finish_http(machine, 8080, base, packets)
        self.assertTrue(machine.connections[8080].closed)
        next_client = (0xfffffff1 + len(peer.REQUEST)) & peer.MASK
        duplicate = machine.handle(client(seq=next_client, ack=base + len(peer.RESPONSE) + 1,
                                          flags=peer.ACK | peer.FIN), 6)
        self.assertEqual(server(duplicate[0])["ack"], (next_client + 1) & peer.MASK)
        self.assertEqual(machine.counts["client_fins_acked"], 1)

    def test_wrong_handshake_get_sequence_and_order_cannot_pass(self):
        with self.assertRaises(peer.ProtocolError):
            peer.PeerMachine().handle(client(8081))
        machine = peer.PeerMachine()
        response = server(machine.handle(client())[0])
        with self.assertRaises(peer.ProtocolError):
            machine.handle(client(seq=0xfffffff1, ack=response["seq"] + 2, flags=peer.ACK))
        machine = peer.PeerMachine()
        base = connect(machine, 8080)
        with self.assertRaises(peer.ProtocolError):
            machine.handle(client(seq=0xfffffff1, ack=base, flags=peer.ACK, payload=b"not HTTP"))
        self.assertFalse(machine.summary()["complete"])

    def test_retransmission_must_match_original_syn_identity(self):
        machine = peer.PeerMachine()
        first_three(machine)
        self.assertEqual(machine.handle(client(8083, options=b"\x02\x04\x05\xb4")), [])
        for change in (dict(seq=0xfffffff1), dict(options=b"\x02\x04\x04\xb4")):
            with self.assertRaises(peer.ProtocolError):
                machine.handle(client(8083, **change))
        response = machine.handle(client(8083, options=b"\x02\x04\x05\xb4", identification=99, window=2048))
        self.assertEqual(server(response[0])["flags"], peer.SYN | peer.ACK)

    def test_partial_http_request_is_acked_without_fabricating_response(self):
        machine = peer.PeerMachine()
        base = connect(machine, 8080)
        response = machine.handle(client(seq=0xfffffff1, ack=base, flags=peer.ACK, payload=peer.REQUEST[:10]))
        self.assertEqual(server(response[0])["flags"], peer.ACK)
        self.assertEqual(machine.counts["http_requests"], 0)
        self.assertEqual(machine.counts["corrupt_data_sent"], 0)
        machine.handle(client(seq=0xfffffff1 + 10, ack=base, flags=peer.ACK, payload=peer.REQUEST[10:]))
        self.assertEqual(machine.counts["http_requests"], 1)


class SocketTests(unittest.TestCase):
    @staticmethod
    def exact(connection, amount):
        output = b""
        while len(output) < amount:
            data = connection.recv(amount - len(output))
            if not data:
                raise AssertionError("short host test frame")
            output += data
        return output

    def test_localhost_server_sends_only_after_actual_request(self):
        with peer.Peer(timeout=3) as server_peer:
            with socket.create_connection(("127.0.0.1", server_peer.port), timeout=1) as connection:
                connection.sendall(peer.framed(ARP))
                length = int.from_bytes(self.exact(connection, 4), "big")
                packet = self.exact(connection, length)
                self.assertEqual(packet[38:42], peer.CLIENT_IP)
        result = server_peer.finish()
        self.assertTrue(result["connected"])
        self.assertIsNone(result["error"])
        self.assertEqual(result["wire_frames_sent"], 1)
        self.assertFalse(result["passed"], "one ARP exchange cannot pass the TCP matrix")

    def test_partial_stream_and_invalid_timeouts_fail_closed(self):
        with peer.Peer(timeout=3) as server_peer:
            with socket.create_connection(("127.0.0.1", server_peer.port), timeout=1) as connection:
                connection.sendall(b"\0\0")
            server_peer.thread.join(1)
        result = server_peer.finish()
        self.assertIn("incomplete", result["error"])
        self.assertFalse(result["passed"])
        for value in (0, -1, 181):
            with self.assertRaises(ValueError):
                peer.Peer(timeout=value)
        server_peer = peer.Peer(timeout=3)
        server_peer.close()
        with self.assertRaises(RuntimeError):
            server_peer.start()

    def test_socket_ack_of_corrupt_data_prevents_later_valid_response(self):
        def packet(connection):
            size = int.from_bytes(self.exact(connection, 4), "big")
            return self.exact(connection, size)

        with peer.Peer(timeout=3) as server_peer:
            with socket.create_connection(("127.0.0.1", server_peer.port), timeout=1) as connection:
                connection.sendall(peer.framed(client()))
                syn_ack = server(packet(connection))
                base = syn_ack["seq"] + 1
                connection.sendall(peer.framed(client(seq=0xfffffff1, ack=base, flags=peer.ACK)))
                connection.sendall(peer.framed(client(seq=0xfffffff1, ack=base, flags=peer.ACK | peer.PSH,
                                                       payload=peer.REQUEST)))
                self.assertEqual(server(packet(connection), valid=False)["payload"], peer.CORRUPT_RESPONSE)
                connection.sendall(peer.framed(client(seq=0xfffffff1 + len(peer.REQUEST),
                                                       ack=base + len(peer.RESPONSE), flags=peer.ACK)))
                server_peer.thread.join(1)
                self.assertFalse(server_peer.thread.is_alive())
        result = server_peer.finish()
        self.assertIn("acknowledged corrupt", result["error"])
        self.assertEqual(result["counts"]["valid_data_sent"], 0)
        self.assertFalse(result["passed"])


if __name__ == "__main__":
    unittest.main()
