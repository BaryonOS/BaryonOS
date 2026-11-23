#!/usr/bin/env python3
"""Bounded synthetic DHCPv4 + DNS + HTTP server for the N02 native DHCP case.

The peer is the sole Ethernet neighbour of the guest NIC. It leases
192.0.2.66/24 (router and DNS server 192.0.2.1), answers app A's A queries
strictly (service.test -> 192.0.2.1, missing.test -> NXDOMAIN with SOA, each at
most once per lease, every real reply preceded by one spoof the guest must
drop: wrong DNS ID or wrong source port), serves one HTTP exchange per
app on that lease, answers the first renewal REQUEST after app A holds an idle
connection with a DHCPNAK, then refuses (fails the run on) any frame still
sourced from the withdrawn address. A fresh DISCOVER receives 192.0.2.67 and
app A resolves service.test again (a new query: its cache was flushed) and
repeats its exchange there. RFC 5227 probes and announcements are
accepted only in their exact form and count. Every recorded reply is replayed
against the actual triggering guest bytes. This is a test server, never an
OS client, and it is not a general DHCP implementation.
"""
from __future__ import annotations
from dataclasses import dataclass, field
import hashlib
import re
import struct

import network_service_peer as base
from network_service_peer import (ACK, CLIENT_MAC, FIN, MASK, MAX_FRAME, PEER_IP, PEER_MAC, PSH,
                                  REQUESTS, RESPONSES, RST, SYN, ProtocolError, checksum)

BROADCAST_MAC = b"\xff" * 6
BROADCAST_IP = b"\xff" * 4
ZERO_IP = b"\0" * 4
SUBNET_MASK = bytes((255, 255, 255, 0))
LEASES = (bytes((192, 0, 2, 66)), bytes((192, 0, 2, 67)))
LEASE_SECONDS = 3600
# (T1, T2) per lease: the first renews soon enough to deliver the NAK inside
# the native deadline; the second does not renew during the run.
TIMES = ((10, 3000), (1800, 3000))
COOKIE = bytes((99, 130, 83, 99))
PARAMETERS = bytes((1, 3, 6, 51, 54, 58, 59))
BOOTP_BYTES = 300
DHCP_FRAME = 14 + 20 + 8 + BOOTP_BYTES
DISCOVER, OFFER, REQUEST, ACK_TYPE, NAK = 1, 2, 3, 5, 6
# port -> (owner slot, lease index, guest source port); 8081 is held idle.
EXCHANGES = {8080: (0, 0, 40000), 8090: (1, 0, 40001), 8082: (0, 1, 40004)}
HELD = {8081: (0, 0, 40002)}
MAX_ACTIONS = 512
# (lease index, name) -> answer; any other query, or a repeat, fails the run.
DNS_QUERIES = {(0, "service.test"): "a", (0, "missing.test"): "nxdomain", (1, "service.test"): "a"}
DNS_TTL, NEGATIVE_TTL = 300, 60
DNS_PORT_FIRST = 49152
# Spoofs carry a different positive answer, so accepting one would change
# the app's result instead of hiding behind the identical real reply.
SPOOF_IP = bytes((192, 0, 2, 99))


def ipv4_header(source, target, protocol, payload_length, identification):
    header = struct.pack("!BBHHHBBH4s4s", 0x45, 0, 20 + payload_length, identification, 0x4000,
                         64, protocol, 0, source, target)
    return header[:10] + struct.pack("!H", checksum(header)) + header[12:]


def dhcp_options(raw):
    """Exact option list in order; padding after END must be zero."""
    options, at = [], 0
    while True:
        if at >= len(raw):
            raise ProtocolError("DHCP options without END")
        code = raw[at]
        if code == 255:
            if any(raw[at + 1:]):
                raise ProtocolError("DHCP nonzero padding after END")
            return options
        if code == 0 or at + 2 > len(raw) or at + 2 + raw[at + 1] > len(raw):
            raise ProtocolError("truncated or padded DHCP option")
        options.append((code, raw[at + 2:at + 2 + raw[at + 1]]))
        at += 2 + raw[at + 1]


def parse_dhcp(frame):
    """Strict DHCP client message as built by services/net-protocol dhcp.rs."""
    if len(frame) != DHCP_FRAME or frame[6:12] != CLIENT_MAC or frame[12:14] != b"\x08\x00":
        raise ProtocolError("DHCP frame length or Ethernet identity")
    ip, udp, bootp = frame[14:34], frame[34:42], frame[42:]
    source, target = ip[12:16], ip[16:20]
    if (ip[:2] != b"\x45\x00" or struct.unpack_from("!H", ip, 2)[0] != 20 + 8 + BOOTP_BYTES
            or ip[4:6] != b"\0\0" or ip[6:8] != b"\x40\x00" or ip[8] != 64 or ip[9] != 17
            or checksum(ip)):
        raise ProtocolError("DHCP IPv4 header")
    sport, dport, size, value = struct.unpack("!HHHH", udp)
    if (sport, dport, size) != (68, 67, 8 + BOOTP_BYTES) or not value or checksum(
            source + target + struct.pack("!BBH", 0, 17, size) + udp + bootp):
        raise ProtocolError("DHCP UDP tuple or checksum")
    op, htype, hlen, hops, xid, secs, flags = struct.unpack_from("!BBBBIHH", bootp)
    ciaddr, yiaddr, siaddr, giaddr = (bootp[at:at + 4] for at in (12, 16, 20, 24))
    if (op, htype, hlen, hops) != (1, 1, 6, 0) or yiaddr != ZERO_IP or siaddr != ZERO_IP or giaddr != ZERO_IP:
        raise ProtocolError("DHCP BOOTP header")
    if bootp[28:34] != CLIENT_MAC:
        raise ProtocolError("DHCP chaddr is not the guest NIC")
    if any(bootp[34:236]) or bootp[236:240] != COOKIE:
        raise ProtocolError("DHCP chaddr padding, sname/file or cookie")
    options = dhcp_options(bootp[240:])
    codes = [code for code, _ in options]
    values = dict(options)
    if len(values) != len(options):
        raise ProtocolError("duplicate DHCP option")
    selecting = codes == [53, 61, 50, 54, 55, 57]
    if codes != [53, 61, 55, 57] and not selecting:
        raise ProtocolError("unexpected DHCP option set")
    if (values[61] != b"\x01" + CLIENT_MAC or values[55] != PARAMETERS
            or values[57] != struct.pack("!H", 576) or len(values[53]) != 1):
        raise ProtocolError("DHCP client identifier, parameter list or size")
    kind = values[53][0]
    message = dict(xid=xid, secs=secs, ciaddr=ciaddr, source=source, target=target)
    if kind == DISCOVER and not selecting:
        message["kind"] = "discover"
        layer = (BROADCAST_MAC, ZERO_IP, BROADCAST_IP, 0x8000, ZERO_IP)
    elif kind == REQUEST and selecting:
        if len(values[50]) != 4 or len(values[54]) != 4:
            raise ProtocolError("DHCP requested address or server identifier length")
        message.update(kind="select", requested=values[50], server=values[54])
        layer = (BROADCAST_MAC, ZERO_IP, BROADCAST_IP, 0x8000, ZERO_IP)
    elif kind == REQUEST:
        message["kind"] = "renew"
        layer = (PEER_MAC, ciaddr, PEER_IP, 0, ciaddr)
    else:
        raise ProtocolError("unexpected DHCP message type")
    if (frame[:6], source, target, flags, ciaddr) != layer or (kind == REQUEST and not selecting and ciaddr == ZERO_IP):
        raise ProtocolError("DHCP addressing does not match message kind")
    return message


def dns_query(frame, client_ip):
    """Strict guest A query as built by services/net-protocol dns.rs build_query,
    from slot 0's random DNS port (low bit 0) on the current lease to port 53."""
    if not 14 + 20 + 8 + 12 <= len(frame) <= MAX_FRAME or frame[:6] != PEER_MAC or frame[6:12] != CLIENT_MAC:
        raise ProtocolError("DNS frame bound or Ethernet identity")
    ip = frame[14:]
    total = struct.unpack_from("!H", ip, 2)[0]
    if (ip[0] != 0x45 or not ip[8] or ip[9] != 17 or checksum(ip[:20]) or not 20 + 8 + 12 <= total <= len(ip)
            or struct.unpack_from("!H", ip, 6)[0] & 0xbfff or ip[12:16] != client_ip
            or ip[16:20] != PEER_IP or len(frame) not in (14 + total, max(60, 14 + total))):
        raise ProtocolError("DNS IPv4 tuple, checksum, fragmentation or length")
    udp = ip[20:total]
    source, target, size, value = struct.unpack_from("!HHHH", udp)
    if (target != 53 or source < DNS_PORT_FIRST or source & 1 or size != len(udp) or not value
            or checksum(client_ip + PEER_IP + struct.pack("!BBH", 0, 17, size) + udp)):
        raise ProtocolError("DNS UDP ports, length or checksum")
    message = udp[8:]
    ident, flags, qd, an, ns, ar = struct.unpack_from("!HHHHHH", message)
    if (flags, qd, an, ns, ar) != (0x0100, 1, 0, 0, 0):
        raise ProtocolError("DNS query header")
    labels, at = [], 12
    while True:
        if at >= len(message) or message[at] > 63:
            raise ProtocolError("DNS query name")
        length = message[at]
        if not length:
            break
        labels.append(message[at + 1:at + 1 + length].decode("ascii", "replace"))
        at += 1 + length
    if message[at + 1:] != b"\0\x01\0\x01":
        raise ProtocolError("DNS query is not exactly one IN A question")
    return source, ident, ".".join(labels), message[12:]


def tcp_segment(frame, client_ip):
    """Strict guest TCP segment from the current lease to the peer."""
    if not 54 <= len(frame) <= MAX_FRAME or frame[:6] != PEER_MAC or frame[6:12] != CLIENT_MAC:
        raise ProtocolError("TCP frame bound or Ethernet identity")
    ip = frame[14:]
    total = struct.unpack_from("!H", ip, 2)[0]
    if (ip[0] != 0x45 or not ip[8] or ip[9] != 6 or checksum(ip[:20]) or not 40 <= total <= len(ip)
            or struct.unpack_from("!H", ip, 6)[0] & 0xbfff or ip[12:16] != client_ip
            or ip[16:20] != PEER_IP or len(frame) not in (14 + total, max(60, 14 + total))):
        raise ProtocolError("TCP IPv4 tuple, checksum, fragmentation or length")
    tcp = ip[20:total]
    source, target, seq, ack, offset, flags, window, _, urgent = struct.unpack_from("!HHIIBBHHH", tcp)
    size = (offset >> 4) * 4
    if (offset & 15 or not 20 <= size <= min(60, len(tcp)) or flags & ~31 or urgent
            or len(tcp) - size > 1024 or checksum(client_ip + PEER_IP + struct.pack("!BBH", 0, 6, len(tcp)) + tcp)):
        raise ProtocolError("TCP checksum, flags or bounds")
    return base._wire.Segment(source, target, seq, ack, flags, window, tcp[20:size], tcp[size:])


@dataclass
class Connection:
    port: int
    source: int
    address: bytes
    isn: int
    server_isn: int
    phase: str = "syn"
    request: bytearray = field(default_factory=bytearray)
    data_acked: bool = False
    closed: bool = False
    @property
    def client_next(self):
        return (self.isn + 1 + len(self.request)) & MASK
    @property
    def server_start(self):
        return (self.server_isn + 1) & MASK
    @property
    def data_end(self):
        return (self.server_start + len(RESPONSES[EXCHANGES[self.port][0]])) & MASK


class DhcpMachine:
    def __init__(self):
        self.phase = "idle"
        self.lease = -1
        self.offer_xid = None
        self.xids = []
        self.leases = []
        self.withdrawn = set()
        self.probes, self.announcements = [0, 0], [0, 0]
        self.connections = {}
        self.events = []
        self.ip_id = 1
        self.dns_seen = []
        names = ("client_frames", "server_frames", "discovers", "offers", "selects", "acks",
                 "renewals", "renewal_acks", "naks", "retransmissions", "arp_probes",
                 "arp_announcements", "arp_requests", "arp_replies", "syns", "syn_acks",
                 "http_requests", "valid_data_sent", "fins_sent", "client_fins_acked", "held",
                 "dns_queries", "dns_answers", "dns_nxdomains", "dns_spoofs")
        self.counts = dict.fromkeys(names, 0)

    @property
    def address(self):
        return LEASES[self.lease] if self.lease >= 0 else None

    def event(self, direction, label, frame, **fields):
        if len(self.events) >= MAX_ACTIONS:
            raise ProtocolError("event budget exhausted")
        self.events.append(dict(sequence=len(self.events) + 1, direction=direction, event=label,
                                length=len(frame), frame_sha256=hashlib.sha256(frame).hexdigest(), **fields))
        self.counts["client_frames" if direction == "rx" else "server_frames"] += 1

    def ipv4(self, target_mac, target_ip, protocol, payload):
        header = ipv4_header(PEER_IP, target_ip, protocol, len(payload), self.ip_id)
        self.ip_id = (self.ip_id + 1) & 0xffff
        return (target_mac + PEER_MAC + b"\x08\x00" + header + payload).ljust(60, b"\0")

    def dhcp_reply(self, kind, message, *, broadcast):
        lease = self.lease
        nak = kind == NAK
        yiaddr = ZERO_IP if nak else LEASES[lease]
        ciaddr = message["ciaddr"] if kind == ACK_TYPE and message["kind"] == "renew" else ZERO_IP
        flags = 0x8000 if message["kind"] != "renew" else 0
        bootp = (struct.pack("!BBBBIHH", 2, 1, 6, 0, message["xid"], 0, flags) + ciaddr + yiaddr
                 + ZERO_IP + ZERO_IP + CLIENT_MAC + b"\0" * 10 + b"\0" * 192 + COOKIE)
        options = bytes((53, 1, kind, 54, 4)) + PEER_IP
        if not nak:
            t1, t2 = TIMES[lease]
            options += (bytes((51, 4)) + struct.pack("!I", LEASE_SECONDS) + bytes((58, 4)) + struct.pack("!I", t1)
                        + bytes((59, 4)) + struct.pack("!I", t2) + bytes((1, 4)) + SUBNET_MASK + bytes((3, 4)) + PEER_IP
                        # Every OFFER/ACK (renewals too) names the peer as DNS server.
                        + bytes((6, 4)) + PEER_IP)
        bootp = (bootp + options + b"\xff").ljust(BOOTP_BYTES, b"\0")
        target_ip = BROADCAST_IP if broadcast else LEASES[lease]
        udp = struct.pack("!HHHH", 67, 68, 8 + len(bootp), 0) + bootp
        value = checksum(PEER_IP + target_ip + struct.pack("!BBH", 0, 17, len(udp)) + udp) or 0xffff
        udp = udp[:6] + struct.pack("!H", value) + udp[8:]
        packet = self.ipv4(BROADCAST_MAC if broadcast else CLIENT_MAC, target_ip, 17, udp)
        label = {OFFER: "dhcp-offer", ACK_TYPE: "dhcp-ack", NAK: "dhcp-nak"}[kind]
        self.event("tx", label, packet, xid=message["xid"], lease=lease)
        return packet

    def ready_for_nak(self):
        done = all(self.connections.get(p) is not None and self.connections[p].closed
                   and self.connections[p].data_acked for p in (8080, 8090))
        held = self.connections.get(8081)
        return done and held is not None and held.phase == "held"

    def dhcp(self, frame):
        m = parse_dhcp(frame)
        retransmit = m["xid"] in self.xids
        if m["kind"] == "discover":
            if retransmit and self.phase == "offered" and m["xid"] == self.offer_xid:
                self.counts["retransmissions"] += 1
            elif retransmit or self.phase not in ("idle", "withdrawn") or self.lease >= 1:
                raise ProtocolError("stale XID or DISCOVER outside acquisition")
            else:
                self.xids.append(m["xid"])
                self.lease += 1
                self.offer_xid, self.phase = m["xid"], "offered"
                self.counts["discovers"] += 1
            self.event("rx", "dhcp-discover", frame, xid=m["xid"])
            self.counts["offers"] += 1
            return [self.dhcp_reply(OFFER, m, broadcast=True)]
        if m["kind"] == "select":
            if (m["xid"] != self.offer_xid or self.phase not in ("offered", "checking")
                    or (self.phase == "checking" and self.probes[self.lease])
                    or m["requested"] != LEASES[self.lease] or m["server"] != PEER_IP):
                raise ProtocolError("REQUEST does not select the outstanding OFFER")
            if self.phase == "checking":
                self.counts["retransmissions"] += 1
            self.phase = "checking"
            self.event("rx", "dhcp-select", frame, xid=m["xid"])
            self.counts["selects"] += 1
            self.counts["acks"] += 1
            return [self.dhcp_reply(ACK_TYPE, m, broadcast=True)]
        if self.phase != "bound" or m["ciaddr"] != self.address or retransmit:
            raise ProtocolError("renewal outside a bound lease or with a stale XID")
        self.xids.append(m["xid"])
        self.event("rx", "dhcp-renew", frame, xid=m["xid"])
        self.counts["renewals"] += 1
        if self.lease == 0 and self.ready_for_nak():
            self.withdrawn.add(self.address)
            self.phase = "withdrawn"
            self.counts["naks"] += 1
            return [self.dhcp_reply(NAK, m, broadcast=True)]
        self.counts["renewal_acks"] += 1
        self.counts["acks"] += 1
        return [self.dhcp_reply(ACK_TYPE, m, broadcast=False)]

    def arp(self, frame):
        if len(frame) not in (42, 60) or any(frame[42:]) or frame[14:22] != struct.pack("!HHBBH", 1, 0x800, 6, 4, 1) or frame[22:28] != CLIENT_MAC:
            raise ProtocolError("invalid ARP request")
        spa, tha, tpa = frame[28:32], frame[32:38], frame[38:42]
        candidate = LEASES[self.lease] if self.lease >= 0 else None
        if spa == ZERO_IP or tpa == candidate:
            # RFC 5227: broadcast, unknown target MAC, sender 0 (probe) or
            # the candidate itself (announcement); never another address.
            if (frame[:6] != BROADCAST_MAC or tha != b"\0" * 6 or candidate is None
                    or tpa != candidate or spa not in (ZERO_IP, candidate)):
                raise ProtocolError("RFC 5227 frame outside the candidate lease")
            if spa == ZERO_IP:
                if self.phase != "checking" or self.probes[self.lease] >= 3:
                    raise ProtocolError("unexpected ARP probe")
                self.probes[self.lease] += 1
                self.counts["arp_probes"] += 1
                self.event("rx", "arp-probe", frame, lease=self.lease)
                return []
            if self.probes[self.lease] != 3 or self.announcements[self.lease] >= 2 or self.phase not in ("checking", "bound"):
                raise ProtocolError("unexpected ARP announcement")
            self.announcements[self.lease] += 1
            self.phase = "bound"
            self.counts["arp_announcements"] += 1
            self.event("rx", "arp-announcement", frame, lease=self.lease)
            return []
        if (self.phase != "bound" or spa != self.address or tpa != PEER_IP
                or frame[:6] not in (BROADCAST_MAC, PEER_MAC) or tha not in (b"\0" * 6, PEER_MAC, BROADCAST_MAC)):
            raise ProtocolError("ARP request outside the bound lease")
        self.event("rx", "arp-request", frame, lease=self.lease)
        packet = (CLIENT_MAC + PEER_MAC + b"\x08\x06" + struct.pack("!HHBBH", 1, 0x800, 6, 4, 2)
                  + PEER_MAC + PEER_IP + CLIENT_MAC + spa).ljust(60, b"\0")
        self.counts["arp_requests"] += 1
        self.counts["arp_replies"] += 1
        self.event("tx", "arp-reply", packet)
        return [packet]

    def dns_reply(self, port, message, label, *, source=53):
        udp = struct.pack("!HHHH", source, port, 8 + len(message), 0) + message
        value = checksum(PEER_IP + self.address + struct.pack("!BBH", 0, 17, len(udp)) + udp) or 0xffff
        udp = udp[:6] + struct.pack("!H", value) + udp[8:]
        packet = self.ipv4(CLIENT_MAC, self.address, 17, udp)
        self.event("tx", label, packet, port=port, source_port=source, dns_id=struct.unpack_from("!H", message)[0])
        return packet

    def dns(self, frame):
        if self.phase != "bound":
            raise ProtocolError("DNS outside a bound lease")
        port, ident, name, question = dns_query(frame, self.address)
        key = (self.lease, name)
        if key not in DNS_QUERIES or key in self.dns_seen:
            raise ProtocolError("unexpected or repeated DNS query")
        self.dns_seen.append(key)
        self.counts["dns_queries"] += 1
        self.event("rx", "dns-query", frame, port=port, dns_id=ident, name=name, lease=self.lease)
        def positive(dns_id, address):
            return (struct.pack("!HHHHHH", dns_id, 0x8180, 1, 1, 0, 0) + question
                    + b"\xc0\x0c" + struct.pack("!HHIH", 1, 1, DNS_TTL, 4) + address)
        if DNS_QUERIES[key] == "a":
            reply = positive(ident, PEER_IP)
            self.counts["dns_answers"] += 1
            label = "dns-answer"
        else:
            soa = b"\xc0\x0c\xc0\x0c" + struct.pack("!IIIII", 1, 3600, 600, 86400, NEGATIVE_TTL)
            reply = (struct.pack("!HHHHHH", ident, 0x8183, 1, 0, 1, 0) + question
                     + b"\xc0\x0c" + struct.pack("!HHIH", 6, 1, 600, len(soa)) + soa)
            self.counts["dns_nxdomains"] += 1
            label = "dns-nxdomain"
        packets = []
        if self.lease == 0:
            # One deterministic poisoned spoof first: wrong ID for the answer,
            # wrong source port for the NXDOMAIN, each claiming SPOOF_IP. The
            # guest must drop and count it.
            if DNS_QUERIES[key] == "a":
                packets.append(self.dns_reply(port, positive(ident ^ 1, SPOOF_IP), "dns-spoof-id"))
            else:
                packets.append(self.dns_reply(port, positive(ident, SPOOF_IP), "dns-spoof-port", source=54))
            self.counts["dns_spoofs"] += 1
        packets.append(self.dns_reply(port, reply, label))
        return packets

    def tcp(self, c, flags, *, seq, ack, label, payload=b"", options=b""):
        tcp = struct.pack("!HHIIBBHHH", c.port, c.source, seq & MASK, ack & MASK,
                          ((20 + len(options)) // 4) << 4, flags, 4096, 0, 0) + options + payload
        value = checksum(PEER_IP + c.address + struct.pack("!BBH", 0, 6, len(tcp)) + tcp)
        tcp = tcp[:16] + struct.pack("!H", value) + tcp[18:]
        packet = self.ipv4(CLIENT_MAC, c.address, 6, tcp)
        self.event("tx", label, packet, port=c.port, seq=seq & MASK, ack=ack & MASK, flags=flags,
                   payload_length=len(payload))
        return packet

    def segment(self, frame):
        if self.phase != "bound":
            raise ProtocolError("TCP outside a bound lease")
        s = tcp_segment(frame, self.address)
        owner, lease, source = {**EXCHANGES, **HELD}.get(s.target, (None, None, None))
        if owner is None or lease != self.lease or s.source != source:
            raise ProtocolError("TCP tuple outside the selected lease")
        self.event("rx", "tcp", frame, port=s.target, seq=s.seq, ack=s.ack, flags=s.flags,
                   payload_length=len(s.payload))
        c = self.connections.get(s.target)
        if s.flags & RST:
            raise ProtocolError("unexpected guest TCP reset")
        if s.flags & SYN:
            if s.flags != SYN or s.payload or s.ack:
                raise ProtocolError("noninitial SYN")
            if c is None:
                c = Connection(s.target, s.source, self.address, s.seq, 0x6e303200 + s.target * 0x100)
                self.connections[s.target] = c
            elif c.phase != "syn" or s.seq != c.isn:
                raise ProtocolError("changed or late retransmitted SYN")
            self.counts["syns"] += 1
            self.counts["syn_acks"] += 1
            return [self.tcp(c, SYN | ACK, seq=c.server_isn, ack=c.client_next, label="syn-ack",
                             options=b"\x02\x04\x05\xb4")]
        if c is None or not s.flags & ACK or s.flags & ~(ACK | PSH | FIN):
            raise ProtocolError("segment outside an established tuple")
        if c.phase == "syn":
            if s.seq != c.client_next or s.ack != c.server_start:
                raise ProtocolError("bad handshake acknowledgment")
            c.phase = "held" if s.target in HELD else "request"
            if c.phase == "held":
                self.counts["held"] += 1
        if c.phase == "held":
            if s.payload or s.flags & FIN or s.seq != c.client_next or s.ack != c.server_start:
                raise ProtocolError("held connection carried data or closed")
            return []
        if c.closed and s.flags == ACK and s.seq == (c.client_next + 1) & MASK:
            if s.ack != (c.data_end + 1) & MASK or s.payload:
                raise ProtocolError("wrong final ACK")
            return []
        if s.seq != c.client_next:
            raise ProtocolError("unexpected client sequence")
        slot = EXCHANGES[s.target][0]
        if c.phase == "request":
            if s.ack != c.server_start or s.flags & FIN:
                raise ProtocolError("premature FIN or HTTP acknowledgment")
            if not s.payload:
                return []
            c.request.extend(s.payload)
            if len(c.request) > len(REQUESTS[slot]) or not REQUESTS[slot].startswith(c.request):
                raise ProtocolError("wrong owner HTTP request")
            if len(c.request) < len(REQUESTS[slot]):
                return [self.tcp(c, ACK, seq=c.server_start, ack=c.client_next, label="request-ack")]
            self.counts["http_requests"] += 1
            self.counts["valid_data_sent"] += 1
            c.phase = "data-ack"
            return [self.tcp(c, PSH | ACK, seq=c.server_start, ack=c.client_next,
                             payload=RESPONSES[slot], label="http-data")]
        if s.payload:
            raise ProtocolError("extra HTTP request bytes")
        if c.phase == "data-ack":
            if s.ack == c.server_start and not s.flags & FIN:
                return []
            if s.ack != c.data_end or s.flags & FIN:
                raise ProtocolError("wrong data ACK or premature FIN")
            c.data_acked, c.phase = True, "fin-ack"
            self.counts["fins_sent"] += 1
            return [self.tcp(c, FIN | ACK, seq=c.data_end, ack=c.client_next, label="server-fin")]
        if c.phase in ("fin-ack", "closed"):
            final = (c.data_end + 1) & MASK
            if s.ack != final:
                raise ProtocolError("wrong FIN acknowledgment")
            if s.flags & FIN:
                if not c.closed:
                    self.counts["client_fins_acked"] += 1
                c.closed, c.phase = True, "closed"
                return [self.tcp(c, ACK, seq=final, ack=(c.client_next + 1) & MASK,
                                 label="client-fin-ack")]
            return []
        raise ProtocolError("unhandled TCP phase")

    def handle(self, frame, now=0.0):
        if not 14 <= len(frame) <= MAX_FRAME or self.counts["client_frames"] >= base.MAX_FRAMES:
            raise ProtocolError("input frame budget")
        if frame[6:12] != CLIENT_MAC:
            raise ProtocolError("wrong client MAC")
        kind = frame[12:14]
        if kind == b"\x08\x06":
            if len(frame) >= 32 and frame[28:32] in self.withdrawn:
                raise ProtocolError("ARP from the withdrawn address")
            return self.arp(frame)
        if kind != b"\x08\x00" or len(frame) < 34:
            raise ProtocolError("unexpected EtherType")
        if frame[26:30] in self.withdrawn:
            raise ProtocolError("IPv4 frame from the withdrawn address")
        if frame[23] == 17 and frame[14] == 0x45 and frame[36:38] == b"\0\x35":
            return self.dns(frame)
        if frame[23] == 17:
            return self.dhcp(frame)
        if frame[23] == 6:
            return self.segment(frame)
        raise ProtocolError("unexpected IPv4 protocol")

    def poll(self, now):
        return []

    def summary(self):
        cases = {str(p): dict(phase=c.phase, source=c.source, address=".".join(map(str, c.address)),
                             request_bytes=len(c.request), data_acked=c.data_acked, closed=c.closed)
                 for p, c in self.connections.items()}
        c = self.connections
        exchanged = all(p in c and c[p].closed and c[p].data_acked and bytes(c[p].request) == REQUESTS[EXCHANGES[p][0]]
                        and c[p].address == LEASES[EXCHANGES[p][1]] for p in EXCHANGES)
        complete = (exchanged and set(c) == set(EXCHANGES) | set(HELD) and c[8081].phase == "held"
                    and self.lease == 1 and self.phase == "bound" and self.withdrawn == {LEASES[0]}
                    and self.counts["discovers"] == 2 and self.counts["selects"] >= 2
                    and self.counts["naks"] == 1 and self.probes == [3, 3] and self.announcements[0] == 2
                    # The service may retire and exit before the second
                    # announcement of the final lease; the first makes it usable.
                    and self.announcements[1] in (1, 2)
                    and self.counts["client_fins_acked"] == 3 and self.counts["held"] == 1
                    and sorted(self.dns_seen) == sorted(DNS_QUERIES) and self.counts["dns_spoofs"] == 2)
        return dict(case="dhcp", complete=complete, counts=dict(self.counts), cases=cases,
                    dns_queries=[[lease, name] for lease, name in self.dns_seen],
                    leases=[".".join(map(str, lease)) for lease in LEASES[:self.lease + 1]],
                    withdrawn=sorted(".".join(map(str, a)) for a in self.withdrawn),
                    xids=len(self.xids), events=list(self.events))


def validate_transcript(evidence):
    """Replay every recorded guest frame; the replies must be reproduced exactly."""
    if evidence.get("case") != "dhcp":
        raise ProtocolError("peer transcript case binding changed")
    actions = evidence.get("actions")
    if not isinstance(actions, list) or not 1 <= len(actions) <= MAX_ACTIONS:
        raise ProtocolError("missing/bounded raw transcript")
    machine, last = DhcpMachine(), 0.0
    for action in actions:
        if not isinstance(action, dict) or set(action) != {"kind", "at", "input", "output"} or action["kind"] != "input":
            raise ProtocolError("malformed transcript action")
        now = action["at"]
        if type(now) not in (int, float) or not last <= now <= 180:
            raise ProtocolError("changed/nonfinite transcript time")
        last = now
        def packet(value):
            if not isinstance(value, str) or not 28 <= len(value) <= MAX_FRAME * 2 or len(value) % 2 or re.fullmatch("[0-9a-f]+", value) is None:
                raise ProtocolError("malformed recorded packet")
            return bytes.fromhex(value)
        expected = action["output"]
        if not isinstance(expected, list) or len(expected) > 2:
            raise ProtocolError("unbounded response list")
        if machine.handle(packet(action["input"]), now) != [packet(p) for p in expected]:
            raise ProtocolError("server reply differs from actual triggering guest bytes")
    summary = machine.summary()
    if not summary["complete"] or any(evidence.get(k) != v for k, v in summary.items()):
        raise ProtocolError("transcript contradicts summary/counters or incomplete DHCP case")
    if evidence.get("wire_frames_sent") != summary["counts"]["server_frames"]:
        raise ProtocolError("not all causal replies reached wire")
    return summary


class Peer(base.Peer):
    """The N01 socket loop with the DHCP machine and DHCP transcript replay."""
    def __init__(self, timeout=180):
        super().__init__(timeout, case="normal")
        self.machine = DhcpMachine()

    def finish(self):
        self.close()
        result = self.machine.summary()
        result.update(kind="synthetic DHCP/Ethernet server; native client evidence required separately",
                      connected=self.connected, error=self.error, rejected_frame=self.rejected_frame,
                      wire_frames_sent=self.wire_frames_sent, actions=list(self.actions), passed=False)
        if self.connected and self.error is None and result["complete"]:
            try:
                validate_transcript(result)
                result["passed"] = True
            except ProtocolError as error:
                result["error"] = str(error)
        return result
