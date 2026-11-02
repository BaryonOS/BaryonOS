#!/usr/bin/env python3
"""Host adversarial checks of N01 peer/harness; no native success claim."""
from __future__ import annotations
import struct
import unittest
from network_service_peer import (ACK, FIN, PSH, RST, SYN, CLIENT_IP, CLIENT_MAC,
                                  PEER_IP, PEER_MAC, REQUESTS, RESPONSES, UDP_PAYLOADS,
                                  FrameDecoder, Machine, Peer, ProtocolError, checksum,
                                  framed, validate_transcript)
from run_network_service import BOOT, OFFSETS, ROLES, sequence


def client_ip(payload, protocol=6):
    header = struct.pack("!BBHHHBBH4s4s", 0x45, 0, 20 + len(payload), 1, 0x4000, 64, protocol, 0, CLIENT_IP, PEER_IP)
    header = header[:10] + struct.pack("!H", checksum(header)) + header[12:]
    return (PEER_MAC + CLIENT_MAC + b"\x08\x00" + header + payload).ljust(60, b"\0")


def client_tcp(port, source, seq, ack=0, flags=SYN, payload=b""):
    segment = struct.pack("!HHIIBBHHH", source, port, seq, ack, 5 << 4, flags, 4096, 0, 0) + payload
    value = checksum(CLIENT_IP + PEER_IP + struct.pack("!BBH", 0, 6, len(segment)) + segment)
    return client_ip(segment[:16] + struct.pack("!H", value) + segment[18:])


def client_udp(slot):
    payload = UDP_PAYLOADS[slot]
    udp = struct.pack("!HHHH", 45000 + slot, 9000 + slot, 8 + len(payload), 0) + payload
    value = checksum(CLIENT_IP + PEER_IP + struct.pack("!BBH", 0, 17, len(udp)) + udp) or 0xffff
    return client_ip(udp[:6] + struct.pack("!H", value) + udp[8:], 17)


def transcript():
    """Construct legal synthetic client packets only for host parser checks."""
    machine, actions, now = Machine(), [], 0.0
    def feed(packet):
        nonlocal now
        now += 0.001
        output = machine.handle(packet, now)
        actions.append(dict(kind="input", at=now, input=packet.hex(), output=[p.hex() for p in output]))
        return output
    def finish(port):
        nonlocal now
        c = machine.connections[port]
        slot, case = (0, port - 8080) if port < 8090 else (1, port - 8090)
        if case in (1, 4):
            return
        if case == 3:
            feed(client_tcp(port, c.source, c.isn))
        feed(client_tcp(port, c.source, c.client_next, c.server_start, ACK))
        feed(client_tcp(port, c.source, c.client_next, c.server_start, ACK | PSH, REQUESTS[slot]))
        if case == 2:
            return
        if case == 0:
            # Explicit ACK of the unadvanced sequence is legal. Accepting
            # the checksum-broken payload would be caught by Machine.handle.
            feed(client_tcp(port, c.source, c.client_next, c.server_start, ACK))
            now += 0.251
            output = machine.poll(now)
            actions.append(dict(kind="poll", at=now, output=[p.hex() for p in output]))
        feed(client_tcp(port, c.source, c.client_next, c.data_end, ACK))
        feed(client_tcp(port, c.source, c.client_next, c.data_end + 1, ACK))
        feed(client_tcp(port, c.source, c.client_next, c.data_end + 1, FIN | ACK))
    # Two owners open before either HTTP transaction completes.
    feed(client_tcp(8080, 40000, 100))
    feed(client_tcp(8090, 40001, 200))
    finish(8080)
    finish(8090)
    source = 40002
    for offset in range(1, 5):
        for base in (8080, 8090):
            port = base + offset
            feed(client_tcp(port, source, 1000 + source))
            source += 1
            finish(port)
    feed(client_udp(0))
    feed(client_udp(1))
    result = machine.summary()
    result.update(actions=actions, wire_frames_sent=result["counts"]["server_frames"],
                  connected=True, error=None, rejected_frame=None, passed=True)
    return result


def native_serial():
    service = dict.fromkeys(OFFSETS, 0)
    service.update({400: 1, 408: 10, 416: 66, 424: 65, 432: 100, 440: 22,
                    448: 2, 456: 2, 464: 10, 472: 3 * sum(map(len, REQUESTS)),
                    480: 2 * sum(map(len, RESPONSES)), 488: 4, 496: 3, 512: 4, 520: 2, 528: 1})
    app = dict.fromkeys(OFFSETS, 0)
    app.update({400: 1, 408: 2, 416: 1, 424: 1, 432: 9, 440: 1, 448: 3,
                456: len(RESPONSES[0]), 464: 2, 472: 4, 480: 1, 488: 1,
                496: 1, 504: 1, 512: 1, 528: 1})
    network = {"NET_TX_SUBMITTED": 66, "NET_TX_COMPLETED": 66, "NET_RX_COMPLETED": 65,
               "NET_RX_DELIVERED": 65, "NET_DENIED": 2, "NET_TX_TIMEOUTS": 0,
               "NET_RESETS": 1, "NET_RESET_ACKS": 1}
    lines = [*BOOT, "NETSVC:MANIFEST:PASS"]
    for role in ROLES:
        values = service if role == "SERVICE" else app
        lines.extend(f"NETSVC:{role}:{offset}:{values[offset]:020d}" for offset in OFFSETS)
    lines.extend(f"SYSTEM:{name}:{value:020d}" for name, value in network.items())
    lines.extend(["NETSVC:ENTROPY_BYTES:00000000000000000008", "NETSVC:CLEANUP:PASS", "NETSVC:PASS", "M1:PASS"])
    return "\n".join(lines) + "\n"


class PeerChecks(unittest.TestCase):
    def test_full_two_owner_wire_transcript_replays(self):
        value = transcript()
        summary = validate_transcript(value)
        self.assertTrue(summary["complete"])
        self.assertEqual(summary["counts"]["udp_requests"], 2)
        for name in ("udp_corrupt_sent", "udp_wrong_scope_sent", "udp_echo_sent", "udp_overflow_sent"):
            self.assertEqual(summary["counts"][name], 2)
        self.assertEqual(summary["counts"]["client_fins_acked"], 4)
        self.assertNotEqual(summary["request_sha256"][0], summary["request_sha256"][1])

    def test_raw_input_reply_and_summary_tampering_is_rejected(self):
        for kind in ("input", "output", "events", "counts", "time", "wire", "missing"):
            value = transcript()
            if kind == "input":
                value["actions"][0]["input"] = "00" * 60
            elif kind == "output":
                value["actions"][0]["output"][0] = "00" * 60
            elif kind == "events":
                value["events"][0]["frame_sha256"] = "0" * 64
            elif kind == "counts":
                value["counts"]["udp_requests"] = 0
            elif kind == "time":
                value["actions"][1]["at"] = float("nan")
            elif kind == "wire":
                value["wire_frames_sent"] -= 1
            else:
                value["actions"].pop()
            with self.subTest(kind=kind), self.assertRaises(ProtocolError):
                validate_transcript(value)

    def test_malformed_tuple_checksum_lengths_and_order_are_rejected(self):
        for case in ("short", "mac", "checksum", "port", "local", "order"):
            packet = client_tcp(8081 if case == "order" else 8080, 40010 if case == "local" else 40000, 10)
            if case == "short":
                packet = packet[:20]
            elif case == "mac":
                packet = packet[:6] + b"\xff" * 6 + packet[12:]
            elif case == "checksum":
                packet = packet[:50] + bytes([packet[50] ^ 1]) + packet[51:]
            elif case == "port":
                packet = client_tcp(9000, 40000, 10)
            with self.subTest(case=case), self.assertRaises(ProtocolError):
                Machine().handle(packet)

    def test_other_owner_payload_cannot_be_counted_as_http_success(self):
        machine = Machine()
        machine.handle(client_tcp(8080, 40000, 10))
        c = machine.connections[8080]
        machine.handle(client_tcp(8080, 40000, c.client_next, c.server_start, ACK))
        with self.assertRaises(ProtocolError):
            machine.handle(client_tcp(8080, 40000, c.client_next, c.server_start, ACK | PSH, REQUESTS[1]))

    def test_corrupt_tcp_data_ack_is_rejected(self):
        machine = Machine()
        machine.handle(client_tcp(8080, 40000, 10))
        c = machine.connections[8080]
        machine.handle(client_tcp(8080, 40000, c.client_next, c.server_start, ACK))
        machine.handle(client_tcp(8080, 40000, c.client_next, c.server_start, ACK | PSH, REQUESTS[0]))
        with self.assertRaises(ProtocolError):
            machine.handle(client_tcp(8080, 40000, c.client_next, c.data_end, ACK))

    def test_socket_framing_rejects_untrusted_lengths_and_truncation(self):
        decoder = FrameDecoder()
        packet = client_tcp(8080, 40000, 10)
        stream = framed(packet)
        self.assertEqual(decoder.feed(stream[:3]), [])
        self.assertEqual(decoder.feed(stream[3:]), [packet])
        decoder.finish()
        for length in (0, 13, 1515, 0xffffffff):
            with self.subTest(length=length), self.assertRaises(ProtocolError):
                FrameDecoder().feed(struct.pack("!I", length))
        decoder = FrameDecoder()
        decoder.feed(stream[:-1])
        with self.assertRaises(ProtocolError):
            decoder.finish()

    def test_native_serial_rejects_missing_duplicate_fault_and_backpressure(self):
        original = native_serial()
        self.assertEqual(sequence(original)["network"]["NET_DENIED"], 2)
        cases = [original.replace("NETSVC:PASS\n", ""), original + "NETSVC:PASS\n",
                 original.replace("NETSVC:SERVICE:520:00000000000000000002", "NETSVC:SERVICE:520:00000000000000000000"),
                 original.replace("NETSVC:APP_A:240:00000000000000000000", "NETSVC:APP_A:240:00000000000000000001"),
                 original.replace("SYSTEM:NET_DENIED:00000000000000000002", "SYSTEM:NET_DENIED:00000000000000000001"),
                 original.replace("NETSVC:MANIFEST:PASS", "NETSVC:MANIFEST:PASS\nUNKNOWN")]
        for data in cases:
            with self.subTest(data=data[-60:]), self.assertRaises(ValueError):
                sequence(data)

    def test_lifetimes_and_unbounded_transcripts_are_rejected(self):
        for timeout in (0, -1, float("nan"), 181):
            with self.assertRaises(ValueError):
                Peer(timeout)
        for actions in (None, [], [{}] * 1025):
            with self.assertRaises(ProtocolError):
                validate_transcript({"actions": actions})


if __name__ == "__main__":
    unittest.main()
