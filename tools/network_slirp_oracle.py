#!/usr/bin/env python3
"""N02-T6 host oracle for the libslirp case of tools/run_network_dhcp.py.

QEMU's built-in libslirp user network (`-netdev user`) is the guest's only L2
neighbour: its own DHCP server leases the address, it answers ARP for the
router 192.0.2.1 and the DNS address 192.0.2.3, forwards DNS to the host's
real resolver and translates TCP to 192.0.2.1:<port> to host 127.0.0.1:<port>.
`restrict=off` is required for that translation and for the DNS option; the
project owner approved this harness trust boundary on 24 November 2026 for a
separate, non-required job.

The oracle is (1) one HTTP server on 127.0.0.1:8090 that accepts exactly one
connection, checks app B's exact request and answers app B's exact response,
and (2) a bounded parser of the netdev's filter-dump pcap, which is the wire
record the runner binds to the broker counters and to app A's DNS answers
(the positive one for NAME and the NXDOMAIN for NEGATIVE).
"""
from __future__ import annotations
import ipaddress
import socket
import struct
import threading
from pathlib import Path

from network_service_peer import REQUESTS, RESPONSES

NET, HOST, DNS, DHCP_START = "192.0.2.0/24", "192.0.2.1", "192.0.2.3", "192.0.2.15"
GUEST_MAC = bytes.fromhex("020000000007")
HTTP_PORT = 8090
NAME = "example.com"
# RFC 6761 reserves .invalid: a conforming resolver answers NXDOMAIN without an address.
NEGATIVE = "missing.invalid"
MAX_PCAP_BYTES = 4 * 1024 * 1024
MAX_FRAMES = 4096


def arguments(pcap: str) -> list[str]:
    """QEMU NIC arguments: run_system.network_arguments with the socket netdev
    replaced by libslirp and a filter-dump of every frame on that netdev."""
    return ["-fw_cfg", "name=opt/org.tianocore/VirtioNetSupport,string=no",
            "-fw_cfg", "name=opt/org.tianocore/IPv4Support,string=no",
            "-fw_cfg", "name=opt/org.tianocore/IPv6Support,string=no",
            "-netdev", f"user,id=net0,net={NET},host={HOST},dhcpstart={DHCP_START},dns={DNS},"
                       "restrict=off,ipv6=off",
            "-object", f"filter-dump,id=dump0,netdev=net0,file={pcap}",
            "-device", "virtio-net-pci,netdev=net0,id=net,addr=0x7,disable-legacy=on,mac=02:00:00:00:00:07,romfile="]


class SlirpOracle:
    """Context manager: listen before QEMU starts; finish() returns evidence."""

    def __init__(self, timeout: float):
        self.timeout = timeout
        self.listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.listener.bind(("127.0.0.1", HTTP_PORT))
        self.listener.listen(4)
        self.listener.settimeout(0.2)
        self.stop = threading.Event()
        self.result = dict(connections=0, request_ok=False, response_sent=False, error=None)
        self.thread = threading.Thread(target=self._serve, daemon=True)

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *_):
        self.stop.set()
        self.thread.join(5)
        self.listener.close()

    def _serve(self):
        while not self.stop.is_set():
            try:
                connection, _ = self.listener.accept()
            except socket.timeout:
                continue
            except OSError as error:
                self.result["error"] = f"accept: {error}"
                return
            with connection:
                self.result["connections"] += 1
                if self.result["connections"] != 1:
                    self.result["error"] = "more than one connection reached the oracle"
                    continue
                try:
                    connection.settimeout(self.timeout)
                    data = b""
                    while len(data) < len(REQUESTS[1]):
                        chunk = connection.recv(len(REQUESTS[1]) - len(data))
                        if not chunk:
                            break
                        data += chunk
                    if data != REQUESTS[1]:
                        self.result["error"] = "request differs from app B's request"
                        continue
                    self.result["request_ok"] = True
                    connection.sendall(RESPONSES[1])
                    self.result["response_sent"] = True
                    # Any byte after the exact request is a duplicate or corrupt exchange.
                    connection.shutdown(socket.SHUT_WR)
                    connection.settimeout(2)
                    try:
                        if connection.recv(1):
                            self.result["error"] = "bytes after app B's request"
                    except socket.timeout:
                        pass  # no byte and no EOF within the deadline: nothing extra was sent
                except OSError as error:
                    self.result["error"] = f"exchange: {error}"

    def finish(self) -> dict:
        return dict(self.result, http_port=HTTP_PORT, name=NAME)


def dns_answers(payload: bytes, question: str):
    """(rcode, A addresses) of a well-formed DNS response to `question`, else None.

    Mirrors net-protocol dns::resolve_path: only class-IN answer records owned
    by the current name count, CNAMEs are followed (at most 8 hops) and
    duplicate addresses are dropped, so the result matches the guest's view.
    """
    def name(at, depth=0):
        out = []
        while True:
            if at >= len(payload) or depth > 16:
                raise ValueError("truncated DNS name")
            length = payload[at]
            if length & 0xc0 == 0xc0:
                pointer = struct.unpack("!H", payload[at:at + 2])[0] & 0x3fff
                return out + name(pointer, depth + 1)[0], at + 2
            if length == 0:
                return out, at + 1
            out.append(payload[at + 1:at + 1 + length].decode("ascii", "replace").lower())
            at += 1 + length
    try:
        flags, qd, an = struct.unpack("!2xHHH", payload[:8])
        if not flags & 0x8000 or qd != 1:
            return None
        labels, at = name(12)
        at += 4
        if ".".join(labels) != question:
            return None
        records = []
        for _ in range(an):
            owner, at = name(at)
            rtype, rclass, _, length = struct.unpack("!HHIH", payload[at:at + 10])
            at += 10
            if at + length > len(payload):
                return None
            if rclass == 1 and rtype == 1 and length == 4:
                records.append((".".join(owner), "A", socket.inet_ntoa(payload[at:at + 4])))
            elif rclass == 1 and rtype == 5:
                records.append((".".join(owner), "CNAME", ".".join(name(at)[0])))
            at += length
        current, seen = question, set()
        for _ in range(9):
            if current in seen:
                return None
            seen.add(current)
            addresses = list(dict.fromkeys(v for o, t, v in records if o == current and t == "A"))
            aliases = {v for o, t, v in records if o == current and t == "CNAME"}
            if not aliases:
                return flags & 0xf, addresses
            if addresses or len(aliases) != 1:
                return None
            current = aliases.pop()
        return None
    except (ValueError, struct.error):
        return None


def _qname(payload: bytes):
    """Lower-case question name of an uncompressed one-question DNS query, else None."""
    labels, at = [], 12
    while at < len(payload) and payload[at] and payload[at] < 64:
        labels.append(payload[at + 1:at + 1 + payload[at]].decode("ascii", "replace").lower())
        at += 1 + payload[at]
    if at >= len(payload) or payload[at] or len(payload) < 12 or struct.unpack("!H", payload[4:6])[0] != 1:
        return None
    return ".".join(labels)


def _options(payload: bytes) -> dict:
    options, i = {}, 240
    while i + 1 < len(payload) and payload[i] != 255:
        if payload[i] == 0:
            i += 1
            continue
        options[payload[i]] = payload[i + 2:i + 2 + payload[i + 1]]
        i += 2 + payload[i + 1]
    return options


def parse_pcap(path: Path) -> dict:
    """Bounded summary of the filter-dump capture of the libslirp netdev."""
    with path.open("rb") as capture:
        data = capture.read(MAX_PCAP_BYTES + 1)
    if len(data) > MAX_PCAP_BYTES or len(data) < 24 or data[:4] != b"\xd4\xc3\xb2\xa1":
        raise ValueError("Missing, oversized or non-little-endian pcap")
    if struct.unpack("<I", data[20:24])[0] != 1:
        raise ValueError("pcap link type is not Ethernet")
    at, frames, guest, slirp, other = 24, 0, 0, 0, 0
    acks, dns, http_syns, foreign = [], [], 0, []
    queries, other_ip = {}, 0
    network = ipaddress.ip_network(NET)
    while at < len(data):
        if at + 16 > len(data):
            raise ValueError("Truncated pcap record header")
        caplen, wirelen = struct.unpack("<II", data[at + 8:at + 16])
        frame = data[at + 16:at + 16 + caplen]
        at += 16 + caplen
        frames += 1
        if frames > MAX_FRAMES or caplen != wirelen or len(frame) != caplen or caplen < 14:
            raise ValueError("Truncated, sliced or excess pcap frame")
        destination, source, ethertype = frame[:6], frame[6:12], struct.unpack("!H", frame[12:14])[0]
        if source == GUEST_MAC:
            guest += 1
        elif destination in (GUEST_MAC, b"\xff" * 6):
            slirp += 1
        else:
            other += 1
        if ethertype == 0x0806:
            continue
        # Only ARP and unfragmented IPv4 with a sane header are expected; count anything else.
        if (ethertype != 0x0800 or len(frame) < 34 or frame[14] >> 4 != 4 or (frame[14] & 0xf) < 5
                or len(frame) < 14 + (frame[14] & 0xf) * 4 or struct.unpack("!H", frame[20:22])[0] & 0x3fff):
            other_ip += 1
            continue
        ihl = (frame[14] & 0xf) * 4
        protocol, src, dst = frame[23], socket.inet_ntoa(frame[26:30]), socket.inet_ntoa(frame[30:34])
        body = frame[14 + ihl:]
        if source == GUEST_MAC and src != "0.0.0.0" and (ipaddress.ip_address(src) not in network
                                                        or ipaddress.ip_address(dst) not in network
                                                        and dst != "255.255.255.255"):
            foreign.append(f"{src}->{dst}")
        if protocol == 6 and len(body) >= 14 and source == GUEST_MAC and dst == HOST:
            if body[13] & 0x12 == 0x02 and struct.unpack("!H", body[2:4])[0] == HTTP_PORT:
                http_syns += 1
        if protocol != 17 or len(body) < 8:
            continue
        sport, dport = struct.unpack("!HH", body[:4])
        payload = body[8:]
        if (sport, dport) == (67, 68) and len(payload) >= 240 and payload[236:240] == b"\x63\x82\x53\x63":
            options = _options(payload)
            if options.get(53) == b"\x05":
                four = lambda code: socket.inet_ntoa(options[code][:4]) if len(options.get(code, b"")) >= 4 else None
                acks.append(dict(address=socket.inet_ntoa(payload[16:20]), router=four(3), dns=four(6),
                                 mask=four(1), lease_s=struct.unpack("!I", options[51])[0]
                                 if len(options.get(51, b"")) == 4 else None))
        elif dport == 53 and dst == DNS and source == GUEST_MAC and len(payload) >= 2:
            queries[(payload[:2], sport)] = _qname(payload)
        elif sport == 53 and src == DNS and destination == GUEST_MAC:
            # Bind each answer to a guest query seen earlier: same ID, port and name.
            question = queries.get((payload[:2], dport))
            answer = dns_answers(payload, question) if question in (NAME, NEGATIVE) else None
            if answer is not None:
                dns.append(dict(name=question, rcode=answer[0], addresses=answer[1]))
    return dict(frames=frames, guest_frames=guest, slirp_frames=slirp, other_frames=other,
                dhcp_acks=acks, dns_responses=dns, http_syns=http_syns, foreign=foreign,
                other_ip=other_ip)
