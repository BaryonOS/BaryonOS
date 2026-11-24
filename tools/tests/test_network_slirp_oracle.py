#!/usr/bin/env python3
"""Host tests for the N02-T6 libslirp oracle, pcap parser and runner binding; no QEMU or native evidence."""
from __future__ import annotations
import socket
import struct
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import network_slirp_oracle as slirp
import run_network_dhcp as runner
from network_service_peer import REQUESTS, RESPONSES

SLIRP_MAC = bytes.fromhex("5255c0000201")
LEASE = socket.inet_aton(slirp.DHCP_START)


def ip(source, target, protocol, body):
    header = struct.pack("!BBHHHBBH4s4s", 0x45, 0, 20 + len(body), 0, 0, 64, protocol, 0,
                         socket.inet_aton(source), socket.inet_aton(target))
    return header + body


def frame(destination, source, payload, ethertype=0x0800):
    return destination + source + struct.pack("!H", ethertype) + payload


def udp(sport, dport, payload):
    return struct.pack("!HHHH", sport, dport, 8 + len(payload), 0) + payload


def ack(router=slirp.HOST, dns=slirp.DNS, address=LEASE):
    bootp = (struct.pack("!BBBBIHH", 2, 1, 6, 0, 1, 0, 0) + b"\0" * 4 + address + b"\0" * 8
             + slirp.GUEST_MAC + b"\0" * 202 + b"\x63\x82\x53\x63")
    options = (bytes((53, 1, 5, 1, 4)) + socket.inet_aton("255.255.255.0") + bytes((3, 4))
               + socket.inet_aton(router) + bytes((6, 4)) + socket.inet_aton(dns)
               + bytes((51, 4)) + struct.pack("!I", 86400) + bytes((54, 4)) + socket.inet_aton(slirp.HOST) + b"\xff")
    return frame(slirp.GUEST_MAC, SLIRP_MAC, ip(slirp.HOST, slirp.DHCP_START, 17, udp(67, 68, bootp + options)))


def qname(name=slirp.NAME):
    return b"".join(bytes((len(p),)) + p.encode() for p in name.split(".")) + b"\0"


def query(ident=7, port=50000):
    message = struct.pack("!HHHHHH", ident, 0x0100, 1, 0, 0, 0) + qname() + struct.pack("!HH", 1, 1)
    return frame(bytes.fromhex("5255c0000203"), slirp.GUEST_MAC,
                 ip(slirp.DHCP_START, slirp.DNS, 17, udp(port, 53, message)))


def response(addresses, name=slirp.NAME, rcode=0):
    message = struct.pack("!HHHHHH", 7, 0x8180 | rcode, 1, len(addresses), 0, 0) + qname(name) + struct.pack("!HH", 1, 1)
    for address in addresses:
        message += b"\xc0\x0c" + struct.pack("!HHIH", 1, 1, 60, 4) + socket.inet_aton(address)
    return frame(slirp.GUEST_MAC, bytes.fromhex("5255c0000203"),
                 ip(slirp.DNS, slirp.DHCP_START, 17, udp(53, 50000, message)))


def syn(target=slirp.HOST, port=slirp.HTTP_PORT):
    segment = struct.pack("!HHIIBBHHH", 40000, port, 1, 0, 5 << 4, 0x02, 4096, 0, 0)
    return frame(SLIRP_MAC, slirp.GUEST_MAC, ip(slirp.DHCP_START, target, 6, segment))


def pcap(frames, *, magic=b"\xd4\xc3\xb2\xa1", linktype=1, slice_last=False):
    data = magic + struct.pack("<HHiIII", 2, 4, 0, 0, 65535, linktype)
    for i, f in enumerate(frames):
        wire = len(f) + (1 if slice_last and i == len(frames) - 1 else 0)
        data += struct.pack("<IIII", 0, 0, len(f), wire) + f
    return data


class PcapTest(unittest.TestCase):
    def parse(self, data):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "slirp.pcap"
            path.write_bytes(data)
            return slirp.parse_pcap(path)

    def test_summary_counts_and_extracts_lease_dns_and_syn(self):
        summary = self.parse(pcap([syn(), ack(), query(), response(["93.184.215.14", "96.7.128.198"])]))
        self.assertEqual((summary["guest_frames"], summary["slirp_frames"], summary["other_frames"]), (2, 2, 0))
        self.assertEqual(summary["other_ip"], 0)
        self.assertEqual(summary["dhcp_acks"], [dict(address=slirp.DHCP_START, router=slirp.HOST, dns=slirp.DNS,
                                                     mask="255.255.255.0", lease_s=86400)])
        self.assertEqual(summary["dns_responses"], [dict(rcode=0, addresses=["93.184.215.14", "96.7.128.198"])])
        self.assertEqual((summary["http_syns"], summary["foreign"]), (1, []))

    def test_other_questions_and_off_net_guest_traffic_are_not_hidden(self):
        summary = self.parse(pcap([query(), response(["1.2.3.4"], name="other.example"), syn(target="198.51.100.9")]))
        self.assertEqual(summary["dns_responses"], [])
        self.assertEqual(summary["foreign"], [f"{slirp.DHCP_START}->198.51.100.9"])

    def test_answers_without_a_matching_guest_query_are_ignored(self):
        for name, frames in (("no query", [response(["1.2.3.4"])]), ("other id", [query(ident=8), response(["1.2.3.4"])]),
                             ("other port", [query(port=50001), response(["1.2.3.4"])])):
            with self.subTest(name):
                self.assertEqual(self.parse(pcap(frames))["dns_responses"], [])

    def test_bad_ipv4_headers_and_fragments_are_counted_not_parsed(self):
        good = syn()
        for name, patch_at, value in (("ihl", 14, 0x44), ("version", 14, 0x65), ("fragment", 20, 0x20)):
            bad = bytearray(good)
            bad[patch_at] = value
            with self.subTest(name):
                summary = self.parse(pcap([bytes(bad)]))
                self.assertEqual((summary["other_ip"], summary["http_syns"]), (1, 0))
        for name, bad in (("short ipv4", good[:30]), ("ipv6", good[:12] + b"\x86\xdd" + good[14:])):
            with self.subTest(name):
                self.assertEqual(self.parse(pcap([bad]))["other_ip"], 1)

    def test_dns_answers_follow_the_guest_resolver_rules(self):
        def message(records):
            body = struct.pack("!HHHHHH", 7, 0x8180, 1, len(records), 0, 0) + qname() + struct.pack("!HH", 1, 1)
            for owner, rtype, rclass, data in records:
                body += owner + struct.pack("!HHIH", rtype, rclass, 60, len(data)) + data
            return body
        a = lambda ip, owner=b"\xc0\x0c", rclass=1: (owner, 1, rclass, socket.inet_aton(ip))
        alias = qname("cdn.example")
        cases = {
            "duplicate": ([a("1.2.3.4"), a("1.2.3.4"), a("5.6.7.8")], ["1.2.3.4", "5.6.7.8"]),
            "class": ([a("1.2.3.4", rclass=3), a("5.6.7.8")], ["5.6.7.8"]),
            "cname": ([(b"\xc0\x0c", 5, 1, alias), a("9.9.9.9", owner=alias), a("1.1.1.1", owner=qname("x.example"))],
                      ["9.9.9.9"]),
        }
        for name, (records, expected) in cases.items():
            with self.subTest(name):
                self.assertEqual(slirp.dns_answers(message(records), slirp.NAME), (0, expected))
        with self.subTest("cname loop"):
            loop = [(b"\xc0\x0c", 5, 1, alias), (alias, 5, 1, qname())]
            self.assertIsNone(slirp.dns_answers(message(loop), slirp.NAME))

    def test_malformed_captures_are_refused(self):
        for name, data in (("magic", pcap([ack()], magic=b"\xa1\xb2\xc3\xd4")), ("linktype", pcap([ack()], linktype=0)),
                           ("sliced", pcap([ack()], slice_last=True)), ("truncated", pcap([ack()])[:-3]),
                           ("empty", b"")):
            with self.subTest(name), self.assertRaises(ValueError):
                self.parse(data)

    def test_truncated_dns_answers_are_ignored(self):
        payload = response(["1.2.3.4"])[14 + 20 + 8:]
        self.assertIsNone(slirp.dns_answers(payload[:-2], slirp.NAME))
        self.assertEqual(slirp.dns_answers(payload, slirp.NAME), (0, ["1.2.3.4"]))


class OracleTest(unittest.TestCase):
    def test_exactly_app_b_request_gets_app_b_response(self):
        with patch.object(slirp, "HTTP_PORT", 0):
            with slirp.SlirpOracle(2) as oracle:
                port = oracle.listener.getsockname()[1]
                with socket.create_connection(("127.0.0.1", port)) as client:
                    client.sendall(REQUESTS[1])
                    data = b""
                    while chunk := client.recv(4096):
                        data += chunk
        evidence = oracle.finish()
        self.assertEqual(data, RESPONSES[1])
        self.assertEqual((evidence["connections"], evidence["request_ok"], evidence["error"]), (1, True, None))

    def test_bytes_after_the_request_are_refused(self):
        with patch.object(slirp, "HTTP_PORT", 0):
            with slirp.SlirpOracle(2) as oracle:
                with socket.create_connection(("127.0.0.1", oracle.listener.getsockname()[1])) as client:
                    client.sendall(REQUESTS[1] + b"x")
                    while client.recv(4096):
                        pass
        self.assertEqual(oracle.finish()["error"], "bytes after app B's request")

    def test_wrong_request_is_refused(self):
        with patch.object(slirp, "HTTP_PORT", 0):
            with slirp.SlirpOracle(2) as oracle:
                with socket.create_connection(("127.0.0.1", oracle.listener.getsockname()[1])) as client:
                    client.sendall(REQUESTS[0])
                    self.assertEqual(client.recv(4096), b"")
        evidence = oracle.finish()
        self.assertEqual((evidence["request_ok"], evidence["error"]), (False, "request differs from app B's request"))


class SlirpRunnerTest(unittest.TestCase):
    """evaluate_slirp() in isolation; sequence() and the pcap parser are covered elsewhere."""
    RUN = runner.ROOT / "build/run/network-dhcp-x"

    def report(self, **wire):
        pcap_rel = self.RUN.relative_to(runner.ROOT).as_posix() + "/slirp.pcap"
        good = dict(guest_frames=20, slirp_frames=18, other_frames=0, other_ip=0, foreign=[], http_syns=1,
                    dhcp_acks=[dict(address=slirp.DHCP_START, router=slirp.HOST, dns=slirp.DNS,
                                    mask="255.255.255.0", lease_s=86400)],
                    dns_responses=[dict(rcode=0, addresses=["93.184.215.14", "96.7.128.198"])])
        command = ["qemu", "-nic", "none", "-object", "rng-random,id=rng0"] + slirp.arguments(pcap_rel)
        return dict(command=command, run_directory=str(self.RUN), wire=dict(good, **wire))

    def evaluate(self, report, oracle=None, a656=None, a664=2, address=None):
        lease = int.from_bytes(LEASE, "big") if address is None else address
        observed = dict(network=dict(NET_TX_COMPLETED=20, NET_RX_DELIVERED=18),
                        processes=dict(SERVICE={"576": lease, "584": lease, "448": 1},
                                       APP_A={"656": int.from_bytes(socket.inet_aton("93.184.215.14"), "big")
                                              if a656 is None else a656, "664": a664}))
        oracle = dict(connections=1, request_ok=True, response_sent=True, error=None) if oracle is None else oracle
        return runner.evaluate_slirp(report, oracle, observed)

    def test_good_report_passes_and_case_is_registered(self):
        self.assertEqual(runner.CASES["slirp"], "test-net-service-slirp")
        self.evaluate(self.report())

    def test_mutants_are_refused(self):
        first_ack = self.report()["wire"]["dhcp_acks"][0]
        mutants = {
            "tx count": (self.report(guest_frames=21), {}),
            "rx count": (self.report(slirp_frames=17), {}),
            "other frame": (self.report(other_frames=1), {}),
            "other ip": (self.report(other_ip=1), {}),
            "foreign": (self.report(foreign=["192.0.2.15->198.51.100.9"]), {}),
            "no http": (self.report(http_syns=0), {}),
            "two acks": (self.report(dhcp_acks=[first_ack, first_ack]), {}),
            "router": (self.report(dhcp_acks=[dict(first_ack, router="192.0.2.2")]), {}),
            "lease": (self.report(), dict(address=0xc0000210)),
            "unseen answer": (self.report(), dict(a656=0x01020304)),
            "count": (self.report(), dict(a664=1)),
            "second answer": (self.report(dns_responses=[dict(rcode=0, addresses=["93.184.215.14", "96.7.128.198"])] * 2), {}),
            "nxdomain": (self.report(dns_responses=[dict(rcode=3, addresses=["93.184.215.14", "96.7.128.198"])]), {}),
            "private answer": (self.report(dns_responses=[dict(rcode=0, addresses=["10.0.0.1"])]),
                               dict(a656=0x0a000001, a664=1)),
            "multicast answer": (self.report(dns_responses=[dict(rcode=0, addresses=["224.0.0.1"])]),
                                 dict(a656=0xe0000001, a664=1)),
            "oracle": (self.report(), dict(oracle=dict(connections=2, request_ok=True, response_sent=True,
                                                       error="more than one connection reached the oracle"))),
        }
        for name, (report, change) in mutants.items():
            with self.subTest(name), self.assertRaises(ValueError):
                self.evaluate(report, **change)

    def test_extra_network_forwarding_is_refused(self):
        for extra in (["-netdev", "socket,id=n1,connect=127.0.0.1:9"], ["-object", "filter-dump,id=d,netdev=net0,file=x"]):
            report = self.report()
            report["command"] += extra
            with self.subTest(extra), self.assertRaisesRegex(ValueError, "QEMU network"):
                self.evaluate(report)
        report = self.report()
        report["command"] = [item.replace("ipv6=off", "ipv6=off,hostfwd=tcp::1-:22") for item in report["command"]]
        with self.assertRaisesRegex(ValueError, "QEMU network"):
            self.evaluate(report)


if __name__ == "__main__":
    unittest.main()
