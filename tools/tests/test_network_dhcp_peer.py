#!/usr/bin/env python3
"""Host tests for the strict N02 DHCP peer; no QEMU or native evidence."""
from __future__ import annotations
import struct
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from network_dhcp_peer import (BROADCAST_IP, BROADCAST_MAC, CLIENT_MAC, LEASES, PEER_IP, PEER_MAC,
                               ZERO_IP, DhcpMachine, ProtocolError, checksum, ipv4_header,
                               validate_transcript)
from network_service_peer import ACK, FIN, PSH, REQUESTS, RESPONSES, SYN, MASK


def dhcp(kind, xid, *, ciaddr=ZERO_IP, requested=None, server=None, chaddr=CLIENT_MAC, extra=b"",
         cookie=bytes((99, 130, 83, 99))):
    """Client message in the exact layout of services/net-protocol dhcp.rs."""
    renew = kind == 3 and requested is None
    flags = 0 if renew else 0x8000
    bootp = (struct.pack("!BBBBIHH", 1, 1, 6, 0, xid, 0, flags) + ciaddr + ZERO_IP * 3
             + chaddr + b"\0" * 10 + b"\0" * 192 + cookie)
    options = bytes((53, 1, kind, 61, 7, 1)) + CLIENT_MAC
    if requested is not None:
        options += bytes((50, 4)) + requested + bytes((54, 4)) + server
    options += bytes((55, 7, 1, 3, 6, 51, 54, 58, 59, 57, 2, 2, 64)) + extra + b"\xff"
    bootp = (bootp + options).ljust(300, b"\0")
    source, target = (ciaddr, PEER_IP) if renew else (ZERO_IP, BROADCAST_IP)
    udp = struct.pack("!HHHH", 68, 67, 8 + len(bootp), 0) + bootp
    value = checksum(source + target + struct.pack("!BBH", 0, 17, len(udp)) + udp) or 0xffff
    udp = udp[:6] + struct.pack("!H", value) + udp[8:]
    return ((PEER_MAC if renew else BROADCAST_MAC) + CLIENT_MAC + b"\x08\x00"
            + ipv4_header(source, target, 17, len(udp), 0) + udp)


def arp(sender_ip, target_ip, destination=BROADCAST_MAC):
    return (destination + CLIENT_MAC + b"\x08\x06" + struct.pack("!HHBBH", 1, 0x800, 6, 4, 1)
            + CLIENT_MAC + sender_ip + b"\0" * 6 + target_ip).ljust(60, b"\0")


def tcp(address, source, target, seq, ack, flags, payload=b""):
    segment = struct.pack("!HHIIBBHHH", source, target, seq & MASK, ack & MASK, 5 << 4, flags, 4096, 0, 0) + payload
    value = checksum(address + PEER_IP + struct.pack("!BBH", 0, 6, len(segment)) + segment)
    segment = segment[:16] + struct.pack("!H", value) + segment[18:]
    ip = ipv4_header(address, PEER_IP, 6, len(segment), 7)
    return (PEER_MAC + CLIENT_MAC + b"\x08\x00" + ip + segment).ljust(60, b"\0")


def options(reply):
    found, at = {}, 14 + 20 + 8 + 240
    while reply[at] != 255:
        found[reply[at]] = reply[at + 2:at + 2 + reply[at + 1]]
        at += 2 + reply[at + 1]
    return found


class Recorder:
    """Drives the machine like Peer._run and keeps the replayable transcript."""
    def __init__(self):
        self.machine, self.actions, self.sent = DhcpMachine(), [], 0

    def __call__(self, frame):
        output = self.machine.handle(frame, float(len(self.actions)))
        self.actions.append(dict(kind="input", at=float(len(self.actions)), input=frame.hex(),
                                 output=[p.hex() for p in output]))
        self.sent += len(output)
        return output

    def evidence(self):
        result = self.machine.summary()
        result.update(actions=list(self.actions), wire_frames_sent=self.sent)
        return result


def acquire(step, lease, xid, announcements=1):
    offer, = step(dhcp(1, xid))
    assert offer[30:34] == BROADCAST_IP and offer[42 + 16:42 + 20] == LEASES[lease]
    ack, = step(dhcp(3, xid, requested=LEASES[lease], server=PEER_IP))
    for _ in range(3):
        assert step(arp(ZERO_IP, LEASES[lease])) == []
    for _ in range(announcements):
        assert step(arp(LEASES[lease], LEASES[lease])) == []
    reply, = step(arp(LEASES[lease], PEER_IP))
    assert reply[38:42] == LEASES[lease]
    return offer, ack


def exchange(step, address, source, port, slot):
    server = (0x6e303200 + port * 0x100 + 1) & MASK
    isn = 1000 * port
    syn_ack, = step(tcp(address, source, port, isn, 0, SYN))
    assert struct.unpack_from("!I", syn_ack, 38)[0] == server - 1
    assert step(tcp(address, source, port, isn + 1, server, ACK)) == []
    data, = step(tcp(address, source, port, isn + 1, server, ACK | PSH, REQUESTS[slot]))
    assert data[54:54 + len(RESPONSES[slot])] == RESPONSES[slot]
    client = isn + 1 + len(REQUESTS[slot])
    end = server + len(RESPONSES[slot])
    fin, = step(tcp(address, source, port, client, end, ACK))
    assert fin[47] == FIN | ACK
    final, = step(tcp(address, source, port, client, end + 1, FIN | ACK))
    assert final[47] == ACK


def first_lease(step):
    acquire(step, 0, 0x11111111, announcements=2)
    exchange(step, LEASES[0], 40000, 8080, 0)
    exchange(step, LEASES[0], 40001, 8090, 1)
    server = (0x6e303200 + 8081 * 0x100 + 1) & MASK
    step(tcp(LEASES[0], 40002, 8081, 5, 0, SYN))
    assert step(tcp(LEASES[0], 40002, 8081, 6, server, ACK)) == []


class DhcpPeerTest(unittest.TestCase):
    def test_full_acquire_nak_reacquire_replays(self):
        step = Recorder()
        offer, ack = acquire(step, 0, 0x11111111, announcements=2)
        for reply in (offer, ack):
            found = options(reply)
            self.assertNotIn(6, found)  # no DNS option is ever offered
            self.assertEqual((found[1], found[3], found[54]), (bytes((255, 255, 255, 0)), PEER_IP, PEER_IP))
        exchange(step, LEASES[0], 40000, 8080, 0)
        exchange(step, LEASES[0], 40001, 8090, 1)
        server = (0x6e303200 + 8081 * 0x100 + 1) & MASK
        step(tcp(LEASES[0], 40002, 8081, 5, 0, SYN))
        step(tcp(LEASES[0], 40002, 8081, 6, server, ACK))
        nak, = step(dhcp(3, 0x22222222, ciaddr=LEASES[0]))
        self.assertEqual((nak[:6], nak[30:34], nak[42 + 16:42 + 20]), (BROADCAST_MAC, BROADCAST_IP, ZERO_IP))
        self.assertEqual(options(nak)[53], bytes((6,)))
        acquire(step, 1, 0x33333333)
        exchange(step, LEASES[1], 40004, 8082, 0)
        evidence = step.evidence()
        self.assertTrue(evidence["complete"])
        self.assertEqual(evidence["withdrawn"], ["192.0.2.66"])
        self.assertEqual(validate_transcript(dict(evidence, case="dhcp"))["counts"]["naks"], 1)
        tampered = dict(evidence, case="dhcp", actions=[dict(a) for a in evidence["actions"]])
        tampered["actions"][1]["output"] = list(tampered["actions"][0]["output"])
        with self.assertRaises(ProtocolError):
            validate_transcript(tampered)

    def test_renewal_before_held_connection_is_acked_not_nakked(self):
        step = Recorder()
        acquire(step, 0, 1)
        ack, = step(dhcp(3, 2, ciaddr=LEASES[0]))
        self.assertEqual(options(ack)[53], bytes((5,)))
        self.assertEqual((ack[:6], ack[30:34]), (CLIENT_MAC, LEASES[0]))
        self.assertEqual(step.machine.counts["naks"], 0)

    def test_renewal_before_second_announcement_is_acked_not_nakked(self):
        step = Recorder()
        acquire(step, 0, 0x11111111)
        exchange(step, LEASES[0], 40000, 8080, 0)
        exchange(step, LEASES[0], 40001, 8090, 1)
        server = (0x6e303200 + 8081 * 0x100 + 1) & MASK
        step(tcp(LEASES[0], 40002, 8081, 5, 0, SYN))
        step(tcp(LEASES[0], 40002, 8081, 6, server, ACK))
        ack, = step(dhcp(3, 0x22222222, ciaddr=LEASES[0]))
        self.assertEqual(options(ack)[53], bytes((5,)))
        self.assertEqual(step.machine.counts["naks"], 0)
        self.assertEqual(step(arp(LEASES[0], LEASES[0])), [])  # second announcement still accepted
        nak, = step(dhcp(3, 0x22222223, ciaddr=LEASES[0]))
        self.assertEqual(options(nak)[53], bytes((6,)))
        self.assertEqual(step.machine.counts["naks"], 1)

    def test_frames_from_withdrawn_address_are_refused(self):
        for late in (tcp(LEASES[0], 40002, 8081, 6, 0, ACK), arp(LEASES[0], PEER_IP),
                     arp(LEASES[0], LEASES[0]), dhcp(3, 0x44444444, ciaddr=LEASES[0])):
            step = Recorder()
            first_lease(step)
            step(dhcp(3, 0x22222222, ciaddr=LEASES[0]))
            with self.assertRaisesRegex(ProtocolError, "withdrawn"):
                step(late)

    def test_stale_xids_are_refused(self):
        step = Recorder()
        first_lease(step)
        with self.assertRaisesRegex(ProtocolError, "stale XID"):
            step(dhcp(3, 0x11111111, ciaddr=LEASES[0]))
        step(dhcp(3, 0x22222222, ciaddr=LEASES[0]))
        with self.assertRaisesRegex(ProtocolError, "stale XID"):
            step(dhcp(1, 0x11111111))
        step = Recorder()
        step(dhcp(1, 5))
        with self.assertRaisesRegex(ProtocolError, "outstanding OFFER"):
            step(dhcp(3, 6, requested=LEASES[0], server=PEER_IP))

    def test_malformed_or_truncated_dhcp_is_refused(self):
        valid = dhcp(1, 9)
        broken_udp = bytearray(valid)
        broken_udp[41] ^= 1
        broken_ip = bytearray(valid)
        broken_ip[22] = 1
        cases = {
            "truncated": valid[:-1],
            "truncated option area": valid[:14 + 20 + 8 + 241],
            "cookie": dhcp(1, 9, cookie=b"\0\0\0\0"),
            "udp checksum": bytes(broken_udp),
            "ttl": bytes(broken_ip),
            "wrong chaddr": dhcp(1, 9, chaddr=bytes.fromhex("020000000008")),
            "unexpected option": dhcp(1, 9, extra=bytes((12, 3)) + b"abc"),
            "renewal without lease": dhcp(3, 9, ciaddr=LEASES[0]),
        }
        for name, frame in cases.items():
            with self.subTest(name), self.assertRaises(ProtocolError):
                DhcpMachine().handle(frame)

    def test_rfc5227_frames_and_tcp_need_the_current_lease(self):
        step = Recorder()
        with self.assertRaisesRegex(ProtocolError, "RFC 5227"):
            step(arp(ZERO_IP, LEASES[0]))
        step = Recorder()
        step(dhcp(1, 1))
        step(dhcp(3, 1, requested=LEASES[0], server=PEER_IP))
        with self.assertRaisesRegex(ProtocolError, "outside a bound lease"):
            step(tcp(LEASES[0], 40000, 8080, 1, 0, SYN))
        with self.assertRaisesRegex(ProtocolError, "announcement"):
            step(arp(LEASES[0], LEASES[0]))
        with self.assertRaisesRegex(ProtocolError, "RFC 5227"):
            step(arp(ZERO_IP, LEASES[1]))
        for _ in range(3):
            step(arp(ZERO_IP, LEASES[0]))
        with self.assertRaisesRegex(ProtocolError, "probe"):
            step(arp(ZERO_IP, LEASES[0]))
        step(arp(LEASES[0], LEASES[0]))
        with self.assertRaisesRegex(ProtocolError, "selected lease"):
            step(tcp(LEASES[0], 40004, 8082, 1, 0, SYN))

    def test_incomplete_run_is_not_complete(self):
        step = Recorder()
        first_lease(step)
        self.assertFalse(step.evidence()["complete"])
        with self.assertRaises(ProtocolError):
            validate_transcript(dict(step.evidence(), case="dhcp"))


if __name__ == "__main__":
    unittest.main()
