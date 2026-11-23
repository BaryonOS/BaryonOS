#!/usr/bin/env python3
"""Host tests for the strict N02 DHCP peer, runner summary and build fingerprint; no QEMU or native evidence."""
from __future__ import annotations
import json
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from network_dhcp_peer import (BROADCAST_IP, BROADCAST_MAC, CLIENT_MAC, LEASES, PEER_IP, PEER_MAC,
                               SPOOF_IP, ZERO_IP, DhcpMachine, ProtocolError, checksum, ipv4_header,
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


def tcp(address, source, target, seq, ack, flags, payload=b"", tcp_opts=b""):
    words = 5 + len(tcp_opts) // 4
    segment = (struct.pack("!HHIIBBHHH", source, target, seq & MASK, ack & MASK, words << 4, flags, 4096, 0, 0)
               + tcp_opts + payload)
    value = checksum(address + PEER_IP + struct.pack("!BBH", 0, 6, len(segment)) + segment)
    segment = segment[:16] + struct.pack("!H", value) + segment[18:]
    ip = ipv4_header(address, PEER_IP, 6, len(segment), 7)
    return (PEER_MAC + CLIENT_MAC + b"\x08\x00" + ip + segment).ljust(60, b"\0")


def query(address, port, ident, name, *, flags=0x0100, kind=1, valid_checksum=True, labels=None):
    """Guest A query in the exact layout of services/net-protocol dns.rs."""
    labels = name.split(".") if labels is None else labels
    qname = b"".join(bytes((len(label),)) + label.encode() for label in labels) + b"\0"
    message = struct.pack("!HHHHHH", ident, flags, 1, 0, 0, 0) + qname + struct.pack("!HH", kind, 1)
    udp = struct.pack("!HHHH", port, 53, 8 + len(message), 0) + message
    value = checksum(address + PEER_IP + struct.pack("!BBH", 0, 17, len(udp)) + udp) or 0xffff
    if not valid_checksum:
        value ^= 0x0101
    udp = udp[:6] + struct.pack("!H", value) + udp[8:]
    return (PEER_MAC + CLIENT_MAC + b"\x08\x00" + ipv4_header(address, PEER_IP, 17, len(udp), 3) + udp).ljust(60, b"\0")


def answer(reply):
    """(destination port, source port, DNS message) with both checksums verified."""
    assert checksum(reply[14:34]) == 0
    total = struct.unpack_from("!H", reply, 16)[0]
    udp = reply[34:14 + total]
    assert checksum(reply[26:30] + reply[30:34] + struct.pack("!BBH", 0, 17, len(udp)) + udp) == 0
    source, target = struct.unpack_from("!HH", udp)
    return target, source, udp[8:]


def resolve_all(step, lease):
    """App A's queries on `lease`; returns the reply frames per name."""
    names = ("service.test", "missing.test") if lease == 0 else ("service.test",)
    return {name: step(query(LEASES[lease], 50000 + 2 * i, 0x1200 + i + 16 * lease, name))
            for i, name in enumerate(names)}


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
    resolve_all(step, 0)
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
            self.assertEqual((found[1], found[3], found[6], found[54]),
                             (bytes((255, 255, 255, 0)), PEER_IP, PEER_IP, PEER_IP))
        replies = resolve_all(step, 0)
        spoof, real = replies["service.test"]
        self.assertEqual(answer(spoof)[:2], (50000, 53))
        self.assertEqual(struct.unpack_from("!H", answer(spoof)[2])[0], 0x1200 ^ 1)
        self.assertEqual(answer(spoof)[2][-4:], SPOOF_IP)
        port, source, message = answer(real)
        self.assertEqual((port, source), (50000, 53))
        self.assertEqual(struct.unpack_from("!HHHHHH", message), (0x1200, 0x8180, 1, 1, 0, 0))
        self.assertEqual(message[-4:], PEER_IP)
        self.assertEqual(message[12:12 + 18], query(LEASES[0], 50000, 0x1200, "service.test")[54:72])
        spoof, real = replies["missing.test"]
        self.assertEqual(answer(spoof)[:2], (50002, 54))
        self.assertEqual(struct.unpack_from("!HH", answer(spoof)[2]), (0x1201, 0x8180))
        self.assertEqual(answer(spoof)[2][-4:], SPOOF_IP)
        self.assertEqual(struct.unpack_from("!HHHHHH", answer(real)[2]), (0x1201, 0x8183, 1, 0, 1, 0))
        exchange(step, LEASES[0], 40000, 8080, 0)
        exchange(step, LEASES[0], 40001, 8090, 1)
        server = (0x6e303200 + 8081 * 0x100 + 1) & MASK
        step(tcp(LEASES[0], 40002, 8081, 5, 0, SYN))
        step(tcp(LEASES[0], 40002, 8081, 6, server, ACK))
        nak, = step(dhcp(3, 0x22222222, ciaddr=LEASES[0]))
        self.assertEqual((nak[:6], nak[30:34], nak[42 + 16:42 + 20]), (BROADCAST_MAC, BROADCAST_IP, ZERO_IP))
        self.assertEqual(options(nak)[53], bytes((6,)))
        acquire(step, 1, 0x33333333)
        real, = resolve_all(step, 1)["service.test"]
        self.assertEqual(answer(real)[2][-4:], PEER_IP)
        exchange(step, LEASES[1], 40004, 8082, 0)
        evidence = step.evidence()
        self.assertEqual(evidence["counts"]["dns_queries"], 3)
        self.assertEqual(evidence["counts"]["dns_spoofs"], 2)
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
        resolve_all(step, 0)
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
        acquire(step, 1, 0x33333333)
        resolve_all(step, 1)
        exchange(step, LEASES[1], 40004, 8082, 0)
        # An ACKed early renewal means the NAK was not tested on the first renewal.
        self.assertEqual(step.machine.counts["early_renewal_acks"], 1)
        self.assertFalse(step.evidence()["complete"])

    def test_illegal_tcp_option_lengths_are_refused(self):
        for bad, reason in ((bytes((2, 0, 0, 0)), "option length"), (bytes((2, 1, 0, 0)), "option length"),
                            (bytes((2, 3, 5, 0)), "MSS"), (bytes((2, 4, 0, 0)), "MSS"),
                            (bytes((3, 5, 1, 1)), "option length"), (bytes((0, 0, 0, 1)), "padding")):
            step = Recorder()
            acquire(step, 0, 1, announcements=2)
            with self.assertRaisesRegex(ProtocolError, reason):
                step(tcp(LEASES[0], 40000, 8080, 1000 * 8080, 0, SYN, tcp_opts=bad))
        for good in (bytes((2, 4, 5, 0xb4)), bytes((1, 1, 1, 0)), b""):
            step = Recorder()
            acquire(step, 0, 1, announcements=2)
            self.assertEqual(len(step(tcp(LEASES[0], 40000, 8080, 1000 * 8080, 0, SYN, tcp_opts=good))), 1)

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

    def test_dns_queries_are_strict_and_never_repeated(self):
        cases = {
            "repeat": query(LEASES[0], 50004, 9, "missing.test"),
            "unknown name": query(LEASES[0], 50004, 9, "other.test"),
            "app B port": query(LEASES[0], 50005, 9, "service.test"),
            "fixed port": query(LEASES[0], 45000, 9, "service.test"),
            "checksum": query(LEASES[0], 50004, 9, "service.test", valid_checksum=False),
            "flags": query(LEASES[0], 50004, 9, "service.test", flags=0),
            "AAAA": query(LEASES[0], 50004, 9, "service.test", kind=28),
            "wrong source": query(LEASES[1], 50004, 9, "service.test"),
        }
        for name, frame in cases.items():
            with self.subTest(name):
                step = Recorder()
                acquire(step, 0, 1)
                step(query(LEASES[0], 50002, 8, "missing.test"))
                with self.assertRaises(ProtocolError):
                    step(frame)
        step = Recorder()
        with self.assertRaisesRegex(ProtocolError, "DNS outside a bound lease"):
            step(query(LEASES[0], 50000, 1, "service.test"))
        step = Recorder()
        first_lease(step)
        step(dhcp(3, 0x22222222, ciaddr=LEASES[0]))
        with self.assertRaisesRegex(ProtocolError, "withdrawn"):
            step(query(LEASES[0], 50000, 1, "service.test"))

    def test_single_label_with_a_dot_is_not_the_fixture_name(self):
        step = Recorder()
        acquire(step, 0, 1)
        with self.assertRaisesRegex(ProtocolError, "unexpected or repeated DNS query"):
            step(query(LEASES[0], 50000, 1, "service.test", labels=["service.test"]))
        self.assertEqual(step.machine.dns_seen, [])
        self.assertEqual(len(step(query(LEASES[0], 50000, 1, "service.test"))), 2)

    def test_dns_queries_out_of_order_leave_the_run_incomplete(self):
        def run(names):
            step = Recorder()
            acquire(step, 0, 0x11111111, announcements=2)
            for i, name in enumerate(names):
                step(query(LEASES[0], 50000 + 2 * i, 0x1200 + i, name))
            exchange(step, LEASES[0], 40000, 8080, 0)
            exchange(step, LEASES[0], 40001, 8090, 1)
            server = (0x6e303200 + 8081 * 0x100 + 1) & MASK
            step(tcp(LEASES[0], 40002, 8081, 5, 0, SYN))
            step(tcp(LEASES[0], 40002, 8081, 6, server, ACK))
            step(dhcp(3, 0x22222222, ciaddr=LEASES[0]))
            acquire(step, 1, 0x33333333)
            resolve_all(step, 1)
            exchange(step, LEASES[1], 40004, 8082, 0)
            return step.evidence()
        self.assertTrue(run(("service.test", "missing.test"))["complete"])
        reversed_run = run(("missing.test", "service.test"))
        self.assertEqual(reversed_run["counts"]["dns_queries"], 3)
        self.assertEqual(reversed_run["counts"]["dns_spoofs"], 2)
        self.assertFalse(reversed_run["complete"])

    def test_incomplete_run_is_not_complete(self):
        step = Recorder()
        first_lease(step)
        self.assertFalse(step.evidence()["complete"])
        with self.assertRaises(ProtocolError):
            validate_transcript(dict(step.evidence(), case="dhcp"))


class DhcpRunnerSummaryTest(unittest.TestCase):
    """F8/F11: a newer attempt always replaces an earlier passing stable summary."""
    def setUp(self):
        import build as builder
        import run_network_dhcp as runner
        self.builder, self.runner = builder, runner
        (runner.ROOT / "build").mkdir(exist_ok=True)
        # Under the checkout (private_path) but never the real BUILD that holds native evidence.
        self.build = Path(tempfile.mkdtemp(prefix="test-dhcp-summary-", dir=runner.ROOT / "build"))
        self.addCleanup(shutil.rmtree, self.build)
        self.summary = self.build / "network-dhcp-report.json"
        self.summary.write_text(json.dumps({"passed": True, "network_dhcp_gate": True}))

    def summary_json(self):
        return json.loads(self.summary.read_text())

    def test_stale_no_build_failure_replaces_earlier_pass(self):
        with patch.object(self.runner, "BUILD", self.build), \
                patch.object(self.runner, "source_hash", return_value="a" * 64):
            with self.assertRaises(ValueError):
                self.runner.main(["--no-build"])  # no source-matching manifest in this BUILD
        summary = self.summary_json()
        self.assertIs(summary["passed"], False)
        self.assertIs(summary["network_dhcp_gate"], False)
        self.assertIn("ValueError", summary["error"])

    def test_build_failure_is_published_and_summary_invalidated_first(self):
        seen = []
        def failing_build(*_):
            seen.append(self.summary_json()["passed"])
            raise subprocess.CalledProcessError(101, ["cargo", "build"])
        with patch.object(self.runner, "BUILD", self.build), \
                patch.object(self.runner, "source_hash", return_value="a" * 64), \
                patch.object(self.builder, "build", side_effect=failing_build), \
                patch.object(self.runner, "run_network_dhcp") as run:
            with self.assertRaises(subprocess.CalledProcessError):
                self.runner.main([])
        self.assertEqual(seen, [False])
        run.assert_not_called()
        summary = self.summary_json()
        self.assertIs(summary["passed"], False)
        self.assertIn("CalledProcessError", summary["error"])

    def test_source_is_frozen_before_build_and_success_publishes_normally(self):
        order = []
        def fingerprint():
            order.append("hash")
            return "f" * 64
        def run(*, timeout, frozen):
            order.append(("run", frozen))
            self.runner.publish_json(self.summary, {"passed": True})
            return {"passed": True}
        with patch.object(self.runner, "BUILD", self.build), \
                patch.object(self.runner, "source_hash", side_effect=fingerprint), \
                patch.object(self.builder, "ensure_lockfile", side_effect=lambda: order.append("lock")), \
                patch.object(self.builder, "build", side_effect=lambda *_: order.append("build")), \
                patch.object(self.runner, "run_network_dhcp", side_effect=run):
            self.assertEqual(self.runner.main([]), 0)
        # Cargo.lock exists before the freeze, so a first build cannot change the fingerprint.
        self.assertEqual(order, ["lock", "hash", "build", ("run", "f" * 64)])
        self.assertIs(self.summary_json()["passed"], True)


class BuildSourceFingerprintTest(unittest.TestCase):
    """F11: build() refuses a source that changed between its first and last fingerprint."""
    def setUp(self):
        import build as builder
        self.builder, self.calls = builder, []
        (builder.ROOT / "build").mkdir(exist_ok=True)

    def assert_fingerprint_brackets_compiles(self):
        # First fingerprint before any compile, second after the last one.
        self.assertEqual((self.calls[0], self.calls[-1], self.calls.count("hash")), ("hash", "hash", 2))
        self.assertGreaterEqual(self.calls.count("compile"), 2)

    def fingerprints(self, *values):
        values = iter(values)
        def fingerprint():
            self.calls.append("hash")
            return next(values)
        return fingerprint

    def test_source_change_during_build_fails_before_manifest(self):
        with patch.object(self.builder, "source_hash", side_effect=self.fingerprints("a" * 64, "b" * 64)), \
                patch.object(self.builder, "run_logged", side_effect=lambda *_, **__: self.calls.append("compile")), \
                patch.object(self.builder, "inspect_pe") as pe, \
                patch.object(self.builder.shutil, "copy2") as copy, \
                patch.object(self.builder, "save_json") as save:
            with self.assertRaisesRegex(ValueError, "Source changed during the build"):
                self.builder.build()
        self.assert_fingerprint_brackets_compiles()
        pe.assert_not_called()
        copy.assert_not_called()
        save.assert_not_called()

    def test_unchanged_source_records_build_start_fingerprint(self):
        build = Path(tempfile.mkdtemp(prefix="test-build-fingerprint-", dir=self.builder.ROOT / "build"))
        self.addCleanup(shutil.rmtree, build)
        with patch.object(self.builder, "source_hash", side_effect=self.fingerprints("a" * 64, "a" * 64)), \
                patch.object(self.builder, "run_logged", side_effect=lambda *_, **__: self.calls.append("compile")), \
                patch.object(self.builder, "BUILD", build), \
                patch.object(self.builder, "sha256", return_value="0" * 64), \
                patch.object(self.builder, "inspect_pe", return_value={}), \
                patch.object(self.builder, "inspect_elf", return_value={"segments": []}), \
                patch.object(self.builder.shutil, "copy2"), \
                patch.object(self.builder, "save_json") as save:
            manifest = self.builder.build()
        self.assert_fingerprint_brackets_compiles()
        self.assertEqual(manifest["source_sha256"], "a" * 64)
        self.assertEqual(save.call_args_list[0].args[1]["source_sha256"], "a" * 64)


    def test_missing_lockfile_is_generated_before_the_first_fingerprint(self):
        lockfile, exists = self.builder.ROOT / "Cargo.lock", Path.exists
        def step(args, name, **_):
            self.calls.append("lockfile" if name == "lockfile" else "compile")
        with patch.object(Path, "exists", lambda path: False if path == lockfile else exists(path)), \
                patch.object(self.builder, "source_hash", side_effect=self.fingerprints("a" * 64, "b" * 64)), \
                patch.object(self.builder, "run_logged", side_effect=step), \
                patch.object(self.builder, "inspect_pe"), \
                patch.object(self.builder.shutil, "copy2"), \
                patch.object(self.builder, "save_json") as save:
            with self.assertRaisesRegex(ValueError, "Source changed during the build"):
                self.builder.build()
        self.assertEqual(self.calls[:2], ["lockfile", "hash"])
        save.assert_not_called()

if __name__ == "__main__":
    unittest.main()
