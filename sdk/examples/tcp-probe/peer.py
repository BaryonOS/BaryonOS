#!/usr/bin/env python3
"""Bounded Ethernet TCP/HTTP TEST SERVER for the actual Nieuw OS client.

The server receives real guest ARP/TCP packets over localhost QEMU socketnetdev.
It never runs an OS client, supplies guest markers, or proves native acceptance
by itself. There is no Internet, TAP, bridge, allocator-sized wire length, or
application data other than the synthetic request/response below.
"""
from __future__ import annotations

from dataclasses import dataclass, field
import hashlib
import socket
import struct
import threading
import time

CLIENT_MAC = bytes.fromhex("020000000007")
PEER_MAC = bytes.fromhex("020000000001")
CLIENT_IP = bytes((192, 0, 2, 2))
PEER_IP = bytes((192, 0, 2, 1))
PORTS = (8080, 8081, 8082, 8083, 8084)
REQUEST = b"GET /nieuw-os-tcp HTTP/1.1\r\nHost: fixture.invalid\r\nConnection: close\r\n\r\n"
BODY = b"Nieuw OS native TCP\n"
RESPONSE = b"HTTP/1.1 200 OK\r\nContent-Length: 20\r\nConnection: close\r\n\r\n" + BODY
# Distinct bytes make a client checksum bug observable even when its premature
# ACK reaches this server after the valid retransmission's deadline.
CORRUPT_RESPONSE = b"X" + RESPONSE[1:]
MAX_FRAME = 1514
MAX_FRAMES = 256
MAX_PAYLOAD = 1024
MAX_EVENTS = 1024
CORRUPT_DELAY = 0.25
FIN, SYN, RST, PSH, ACK = 1, 2, 4, 8, 16
MASK = 0xffffffff


class ProtocolError(ValueError):
    """A malformed packet or a violated synthetic server contract."""


def checksum(data: bytes) -> int:
    if len(data) & 1:
        data += b"\0"
    value = sum(struct.unpack(f"!{len(data) // 2}H", data)) if data else 0
    while value >> 16:
        value = (value & 0xffff) + (value >> 16)
    return value ^ 0xffff


def framed(packet: bytes) -> bytes:
    if not 14 <= len(packet) <= MAX_FRAME:
        raise ProtocolError("Ethernet frame bound")
    return struct.pack("!I", len(packet)) + packet


class FrameDecoder:
    def __init__(self):
        self.header = bytearray()
        self.payload = bytearray()
        self.length = None
        self.count = 0
        self.failed = False

    def feed(self, data: bytes) -> list[bytes]:
        if self.failed:
            raise ProtocolError("framing already failed")
        output = []
        at = 0
        try:
            while at < len(data):
                if self.length is None:
                    size = min(4 - len(self.header), len(data) - at)
                    self.header.extend(data[at:at + size])
                    at += size
                    if len(self.header) < 4:
                        continue
                    self.length = struct.unpack("!I", self.header)[0]
                    self.header.clear()
                    if not 14 <= self.length <= MAX_FRAME:
                        raise ProtocolError("untrusted socket frame length")
                size = min(self.length - len(self.payload), len(data) - at)
                self.payload.extend(data[at:at + size])
                at += size
                if len(self.payload) == self.length:
                    self.count += 1
                    if self.count > MAX_FRAMES:
                        raise ProtocolError("frame budget exceeded")
                    output.append(bytes(self.payload))
                    self.payload.clear()
                    self.length = None
        except ProtocolError:
            self.failed = True
            self.header.clear()
            self.payload.clear()
            raise
        return output

    def finish(self):
        if self.failed or self.header or self.payload or self.length is not None:
            raise ProtocolError("incomplete or failed frame stream")


@dataclass(frozen=True)
class Segment:
    source: int
    target: int
    seq: int
    ack: int
    flags: int
    window: int
    options: bytes
    payload: bytes


def parse_tcp(frame: bytes) -> Segment:
    if not 54 <= len(frame) <= MAX_FRAME or frame[12:14] != b"\x08\x00":
        raise ProtocolError("not bounded IPv4 TCP")
    if frame[:6] != PEER_MAC or frame[6:12] != CLIENT_MAC:
        raise ProtocolError("wrong Ethernet identity")
    ip = frame[14:]
    total = struct.unpack_from("!H", ip, 2)[0]
    if (ip[0] != 0x45 or ip[8] == 0 or ip[9] != 6 or checksum(ip[:20]) != 0
            or total < 40 or total > len(ip)
            or struct.unpack_from("!H", ip, 6)[0] & 0xbfff
            or ip[12:16] != CLIENT_IP or ip[16:20] != PEER_IP
            or len(frame) not in (14 + total, max(60, 14 + total))):
        raise ProtocolError("invalid IP identity, checksum, fragmentation or length")
    tcp = ip[20:total]
    source, target, seq, ack, offset, flags, window, _, urgent = struct.unpack_from("!HHIIBBHHH", tcp)
    size = (offset >> 4) * 4
    if (offset & 15 or not 20 <= size <= min(60, len(tcp)) or flags & ~31 or urgent
            or len(tcp) - size > MAX_PAYLOAD
            or checksum(CLIENT_IP + PEER_IP + struct.pack("!BBH", 0, 6, len(tcp)) + tcp) != 0):
        raise ProtocolError("invalid TCP flags, checksum or bounds")
    if target not in PORTS or source != 40000 + PORTS.index(target):
        raise ProtocolError("wrong TCP fixture tuple")
    options = tcp[20:size]
    at = 0
    while at < len(options):
        kind = options[at]
        if kind == 0:
            if any(options[at + 1:]):
                raise ProtocolError("noncanonical TCP option padding")
            break
        if kind == 1:
            at += 1
            continue
        if at + 2 > len(options) or options[at + 1] < 2 or at + options[at + 1] > len(options):
            raise ProtocolError("truncated TCP option")
        if kind == 2 and (options[at + 1] != 4 or options[at + 2:at + 4] == b"\0\0"):
            raise ProtocolError("invalid TCP MSS option")
        at += options[at + 1]
    return Segment(source, target, seq, ack, flags, window, options, tcp[size:])


@dataclass
class Connection:
    port: int
    source: int
    isn: int
    options: bytes
    syn_flags: int
    server_isn: int
    phase: str
    syns: int = 1
    request: bytearray = field(default_factory=bytearray)
    data_sent: bool = False
    data_acked: bool = False
    fin_sent: bool = False
    closed: bool = False
    corrupt_due: float | None = None

    @property
    def client_next(self):
        return (self.isn + 1 + len(self.request)) & MASK

    @property
    def server_start(self):
        return (self.server_isn + 1) & MASK

    @property
    def data_end(self):
        return (self.server_start + len(RESPONSE)) & MASK


class PeerMachine:
    """Small protocol-conforming TEST SERVER, driven by observed guest frames."""
    def __init__(self):
        self.connections: dict[int, Connection] = {}
        self.events: list[dict] = []
        names = ("client_frames", "server_frames", "arp_requests", "arp_replies", "syns",
                 "syn_acks", "refusals", "resets_after_get", "first_syn_dropped", "retry_syns",
                 "blackhole_syns", "blackhole_rsts", "corrupt_data_sent", "valid_data_sent",
                 "http_requests", "valid_data_acks", "fins_sent", "client_fins_acked")
        self.counts = dict.fromkeys(names, 0)
        self.ip_id = 1

    def event(self, direction: str, label: str, packet: bytes, **detail):
        if len(self.events) >= MAX_EVENTS:
            raise ProtocolError("transcript budget exceeded")
        self.events.append(dict(sequence=len(self.events) + 1, direction=direction,
                                event=label, frame_sha256=hashlib.sha256(packet).hexdigest(),
                                length=len(packet), **detail))
        self.counts["server_frames" if direction == "tx" else "client_frames"] += 1

    def tcp(self, c: Connection, flags: int, *, seq: int, ack: int, label: str,
            payload: bytes = b"", options: bytes = b"", corrupt: bool = False) -> bytes:
        tcp = struct.pack("!HHIIBBHHH", c.port, c.source, seq & MASK, ack & MASK,
                          ((20 + len(options)) // 4) << 4, flags, 4096, 0, 0) + options + payload
        value = checksum(PEER_IP + CLIENT_IP + struct.pack("!BBH", 0, 6, len(tcp)) + tcp)
        tcp = tcp[:16] + struct.pack("!H", value ^ (1 if corrupt else 0)) + tcp[18:]
        ip = struct.pack("!BBHHHBBH4s4s", 0x45, 0, 20 + len(tcp), self.ip_id,
                         0x4000, 64, 6, 0, PEER_IP, CLIENT_IP)
        self.ip_id = (self.ip_id + 1) & 0xffff
        ip = ip[:10] + struct.pack("!H", checksum(ip)) + ip[12:]
        packet = (CLIENT_MAC + PEER_MAC + b"\x08\x00" + ip + tcp).ljust(60, b"\0")
        self.event("tx", label, packet, port=c.port, seq=seq & MASK, ack=ack & MASK,
                   flags=flags, payload_length=len(payload))
        return packet

    def syn_ack(self, c: Connection) -> bytes:
        self.counts["syn_acks"] += 1
        return self.tcp(c, SYN | ACK, seq=c.server_isn, ack=c.client_next,
                        options=b"\x02\x04\x05\xb4", label="syn-ack")

    def valid_data(self, c: Connection) -> bytes:
        c.corrupt_due = None
        c.data_sent = True
        c.phase = "data-ack"
        self.counts["valid_data_sent"] += 1
        return self.tcp(c, PSH | ACK, seq=c.server_start, ack=c.client_next,
                        payload=RESPONSE, label="http-data")

    def handle(self, frame: bytes, now: float = 0) -> list[bytes]:
        if not 14 <= len(frame) <= MAX_FRAME or self.counts["client_frames"] >= MAX_FRAMES:
            raise ProtocolError("input frame budget")
        if frame[6:12] != CLIENT_MAC:
            raise ProtocolError("wrong client MAC")
        if frame[12:14] == b"\x08\x06":
            if (len(frame) not in (42, 60) or frame[:6] not in (b"\xff" * 6, PEER_MAC)
                    or frame[14:22] != struct.pack("!HHBBH", 1, 0x800, 6, 4, 1)
                    or frame[22:28] != CLIENT_MAC or frame[28:32] != CLIENT_IP
                    or frame[32:38] not in (b"\0" * 6, PEER_MAC) or frame[38:42] != PEER_IP):
                raise ProtocolError("invalid ARP request")
            self.event("rx", "arp-request", frame)
            packet = (CLIENT_MAC + PEER_MAC + b"\x08\x06" + struct.pack("!HHBBH", 1, 0x800, 6, 4, 2)
                      + PEER_MAC + PEER_IP + CLIENT_MAC + CLIENT_IP).ljust(60, b"\0")
            self.counts["arp_requests"] += 1
            self.counts["arp_replies"] += 1
            self.event("tx", "arp-reply", packet)
            return [packet]
        s = parse_tcp(frame)
        self.event("rx", "tcp", frame, port=s.target, seq=s.seq, ack=s.ack,
                   flags=s.flags, payload_length=len(s.payload))
        c = self.connections.get(s.target)
        if s.flags & SYN:
            if s.flags != SYN or s.payload or s.ack:
                raise ProtocolError("SYN outside initial handshake")
            self.counts["syns"] += 1
            if c is None:
                index = PORTS.index(s.target)
                if len(self.connections) != index:
                    raise ProtocolError("fixture cases must run in order")
                if index and not self.done(self.connections[PORTS[index - 1]]):
                    raise ProtocolError("previous connection not finished")
                phase = "refused" if s.target == 8081 else "blackhole" if s.target == 8084 else "syn"
                c = Connection(s.target, s.source, s.seq, s.options, s.flags,
                               0x6e6f7300 + index * 0x1000, phase)
                self.connections[s.target] = c
            else:
                if (s.source, s.seq, s.options, s.flags) != (c.source, c.isn, c.options, c.syn_flags):
                    raise ProtocolError("retransmitted SYN changed transaction identity")
                if c.phase not in ("syn", "refused", "blackhole"):
                    raise ProtocolError("SYN after established connection")
                c.syns += 1
            if c.port == 8081:
                self.counts["refusals"] += 1
                return [self.tcp(c, RST | ACK, seq=0, ack=c.client_next, label="connection-refused")]
            if c.port == 8084:
                self.counts["blackhole_syns"] += 1
                return []
            if c.port == 8083:
                if c.syns == 1:
                    self.counts["first_syn_dropped"] += 1
                    return []
                self.counts["retry_syns"] += 1
            return [self.syn_ack(c)]
        if c is None:
            raise ProtocolError("TCP data without a SYN")
        if s.flags & RST:
            if c.port not in (8081, 8082, 8084) or s.payload:
                raise ProtocolError("unexpected client reset")
            if c.port == 8084:
                self.counts["blackhole_rsts"] += 1
            return []
        if not s.flags & ACK or s.flags & ~(ACK | PSH | FIN):
            raise ProtocolError("established segment needs ACK")
        if c.phase in ("refused", "reset", "blackhole"):
            raise ProtocolError("data on refused/reset/blackhole connection")
        if c.phase == "syn":
            if s.seq != c.client_next or s.ack != c.server_start:
                raise ProtocolError("bad handshake acknowledgment")
            c.phase = "request"
        if c.closed and s.flags == ACK and s.seq == (c.client_next + 1) & MASK:
            if s.ack != (c.data_end + 1) & MASK or s.payload:
                raise ProtocolError("wrong final ACK")
            return []
        if s.seq != c.client_next:
            raise ProtocolError("unexpected or duplicate client sequence")
        if c.phase == "request":
            if s.ack != c.server_start or s.flags & FIN:
                raise ProtocolError("unexpected HTTP request acknowledgment/FIN")
            if not s.payload:
                return []
            c.request.extend(s.payload)
            if len(c.request) > len(REQUEST) or not REQUEST.startswith(c.request):
                raise ProtocolError("wrong HTTP request")
            if len(c.request) < len(REQUEST):
                return [self.tcp(c, ACK, seq=c.server_start, ack=c.client_next, label="request-ack")]
            self.counts["http_requests"] += 1
            if c.port == 8082:
                c.phase = "reset"
                self.counts["resets_after_get"] += 1
                return [self.tcp(c, RST | ACK, seq=c.server_start, ack=c.client_next, label="reset-after-get")]
            if c.port == 8080:
                c.phase = "corrupt-delay"
                c.corrupt_due = now + CORRUPT_DELAY
                self.counts["corrupt_data_sent"] += 1
                return [self.tcp(c, PSH | ACK, seq=c.server_start, ack=c.client_next,
                                 payload=CORRUPT_RESPONSE, corrupt=True, label="bad-tcp-checksum")]
            return [self.valid_data(c)]
        if s.payload:
            raise ProtocolError("unexpected HTTP request tail")
        if c.phase == "corrupt-delay":
            if s.ack != c.server_start:
                raise ProtocolError("client acknowledged corrupt TCP data")
            return []
        if c.phase == "data-ack":
            if s.ack == c.server_start and not s.flags & FIN:
                return []  # A delayed duplicate ACK has not acknowledged data.
            if s.ack != c.data_end or s.flags & FIN:
                raise ProtocolError("wrong data ACK or premature client FIN")
            c.data_acked = True
            c.fin_sent = True
            c.phase = "fin-ack"
            self.counts["valid_data_acks"] += 1
            self.counts["fins_sent"] += 1
            return [self.tcp(c, FIN | ACK, seq=c.data_end, ack=c.client_next, label="server-fin")]
        final_ack = (c.data_end + 1) & MASK
        if c.phase in ("fin-ack", "closed"):
            if s.ack != final_ack:
                raise ProtocolError("wrong FIN acknowledgment")
            if s.flags & FIN:
                if c.closed:
                    return [self.tcp(c, ACK, seq=final_ack, ack=(c.client_next + 1) & MASK,
                                     label="duplicate-client-fin-ack")]
                c.closed = True
                c.phase = "closed"
                self.counts["client_fins_acked"] += 1
                return [self.tcp(c, ACK, seq=final_ack, ack=(c.client_next + 1) & MASK,
                                 label="client-fin-ack")]
            return []
        raise ProtocolError("unhandled server state")

    def poll(self, now: float) -> list[bytes]:
        output = []
        for c in self.connections.values():
            if c.corrupt_due is not None and now >= c.corrupt_due:
                output.append(self.valid_data(c))
        return output

    @staticmethod
    def done(c: Connection) -> bool:
        return c.closed or c.phase in ("refused", "reset", "blackhole")

    def summary(self) -> dict:
        cases = {str(port): dict(phase=c.phase, syns=c.syns, request_bytes=len(c.request),
                                request_sha256=hashlib.sha256(c.request).hexdigest(),
                                data_acked=c.data_acked, fin_sent=c.fin_sent, closed=c.closed)
                 for port, c in self.connections.items()}
        complete = (tuple(self.connections) == PORTS and all(self.done(c) for c in self.connections.values())
                    and self.connections[8080].data_acked and self.connections[8083].data_acked
                    and self.counts["first_syn_dropped"] == 1 and self.counts["retry_syns"] >= 1
                    and self.counts["corrupt_data_sent"] == 1 and self.counts["valid_data_sent"] == 2
                    and self.counts["client_fins_acked"] == 2)
        return dict(complete=complete, counts=dict(self.counts), cases=cases,
                    response_sha256=hashlib.sha256(RESPONSE).hexdigest(),
                    corrupt_response_sha256=hashlib.sha256(CORRUPT_RESPONSE).hexdigest(),
                    response_body_bytes=len(BODY), events=list(self.events))


class Peer:
    """One ephemeral localhost QEMU connection; no external listener or client."""
    def __init__(self, timeout: float = 180):
        if not 0 < timeout <= 180:
            raise ValueError("peer timeout outside 0..180 seconds")
        self.timeout = timeout
        self.machine = PeerMachine()
        self.port = 0
        self.socket = None
        self.thread = None
        self.stop = threading.Event()
        self.error = None
        self.connected = False
        self.wire_frames_sent = 0
        self.rejected_frame = None

    def start(self):
        if self.thread is not None or self.stop.is_set():
            raise RuntimeError("peer can start only once")
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.bind(("127.0.0.1", 0))
        self.socket.listen(1)
        self.socket.settimeout(0.05)
        self.port = self.socket.getsockname()[1]
        self.thread = threading.Thread(target=self._run, name="nieuw-os-tcp-test-server", daemon=True)
        self.thread.start()
        return self

    def _run(self):
        deadline = time.monotonic() + self.timeout
        connection = None
        decoder = FrameDecoder()
        try:
            while not self.stop.is_set() and time.monotonic() < deadline:
                if connection is None:
                    try:
                        connection, address = self.socket.accept()
                    except socket.timeout:
                        continue
                    if address[0] != "127.0.0.1":
                        raise ProtocolError("nonlocal connection")
                    connection.settimeout(0.02)
                    self.connected = True
                try:
                    data = connection.recv(4096)
                except socket.timeout:
                    # Drain input before the delayed valid segment. Otherwise
                    # an already queued ACK of the corrupt segment could be
                    # misattributed to the valid segment sent just before recv.
                    for packet in self.machine.poll(time.monotonic()):
                        connection.sendall(framed(packet))
                        self.wire_frames_sent += 1
                    continue
                if not data:
                    decoder.finish()
                    break
                for frame in decoder.feed(data):
                    try:
                        packets = self.machine.handle(frame, time.monotonic())
                    except ProtocolError:
                        self.rejected_frame = dict(length=len(frame), prefix_hex=frame[:128].hex())
                        raise
                    for packet in packets:
                        connection.sendall(framed(packet))
                        self.wire_frames_sent += 1
            decoder.finish()
            if time.monotonic() >= deadline:
                raise TimeoutError("test server lifetime exhausted")
        except Exception as error:
            if not (self.stop.is_set() and connection is None and isinstance(error, OSError)):
                self.error = f"{type(error).__name__}: {error}"
        finally:
            if connection is not None:
                connection.close()

    def close(self):
        self.stop.set()
        if self.socket is not None:
            self.socket.close()
        if self.thread is not None:
            self.thread.join(2)
            if self.thread.is_alive():
                raise RuntimeError("test server shutdown exceeded bound")

    def finish(self) -> dict:
        self.close()
        result = self.machine.summary()
        result.update(kind="synthetic TCP server only; not OS client or native proof",
                      connected=self.connected, error=self.error, rejected_frame=self.rejected_frame,
                      wire_frames_sent=self.wire_frames_sent)
        result["passed"] = (self.connected and self.error is None and result["complete"]
                            and self.wire_frames_sent == result["counts"]["server_frames"])
        return result

    def __enter__(self):
        return self.start()

    def __exit__(self, *_):
        self.close()
