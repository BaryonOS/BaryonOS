#!/usr/bin/env python3
"""Bounded synthetic Ethernet server for N01; never an OS client.

Only an ephemeral 127.0.0.1 QEMU socket is opened. All recorded bytes are fixed
public test data. Replaying the bounded raw packet transcript checks every
server reply against its actual triggering guest packet, including both owners.
"""
from __future__ import annotations
from dataclasses import dataclass, field
import hashlib
import importlib.util
from pathlib import Path
import re
import socket
import struct
import sys
import threading
import time

_spec = importlib.util.spec_from_file_location("nieuw_os_n01_tcp_wire", Path(__file__).resolve().parents[1] / "sdk/examples/tcp-probe/peer.py")
_wire = importlib.util.module_from_spec(_spec)
sys.modules[_spec.name] = _wire
_spec.loader.exec_module(_wire)
checksum, framed, FrameDecoder = _wire.checksum, _wire.framed, _wire.FrameDecoder
ProtocolError = _wire.ProtocolError
CLIENT_MAC, PEER_MAC = _wire.CLIENT_MAC, _wire.PEER_MAC
CLIENT_IP, PEER_IP = _wire.CLIENT_IP, _wire.PEER_IP
FIN, SYN, RST, PSH, ACK, MASK = 1, 2, 4, 8, 16, 0xffffffff
MAX_FRAME, MAX_FRAMES, MAX_EVENTS = 1514, 256, 1024
PORTS = tuple(range(8080, 8085)) + tuple(range(8090, 8095))
REQUESTS = tuple(b"GET /app-" + s + b" HTTP/1.1\r\nHost: fixture.invalid\r\nConnection: close\r\n\r\n" for s in (b"a", b"b"))
BODIES = (b"Nieuw OS service A\n", b"Nieuw OS service B\n")
# Existing native fixture deliberately checks this exact wire representation.
RESPONSES = tuple(b"HTTP/1.1 200 OK\r\nContent-Length: 19\r\nConnection: close\r\n\r\n" + b for b in BODIES)
UDP_PAYLOADS = (b"Nieuw OS UDP app A", b"Nieuw OS UDP app B")
CORRUPT_DELAY = 0.25
CASES = ("normal", "client-exit")

def owner(port: int) -> tuple[int, int]:
    if port not in PORTS:
        raise ProtocolError("port outside frozen two-owner fixture")
    return (0, port - 8080) if port < 8090 else (1, port - 8090)


def tcp_segment(frame: bytes):
    if not 54 <= len(frame) <= MAX_FRAME or frame[12:14] != b"\x08\x00":
        raise ProtocolError("not bounded IPv4 TCP")
    if frame[:6] != PEER_MAC or frame[6:12] != CLIENT_MAC:
        raise ProtocolError("wrong Ethernet identity")
    ip = frame[14:]
    total = struct.unpack_from("!H", ip, 2)[0]
    if (ip[0] != 0x45 or not ip[8] or ip[9] != 6 or checksum(ip[:20])
            or total < 40 or total > len(ip) or struct.unpack_from("!H", ip, 6)[0] & 0xbfff
            or ip[12:16] != CLIENT_IP or ip[16:20] != PEER_IP
            or len(frame) not in (14 + total, max(60, 14 + total))):
        raise ProtocolError("invalid IPv4 tuple, checksum, fragmentation or length")
    tcp = ip[20:total]
    source, target, seq, ack, offset, flags, window, _, urgent = struct.unpack_from("!HHIIBBHHH", tcp)
    size = (offset >> 4) * 4
    if (offset & 15 or not 20 <= size <= min(60, len(tcp)) or flags & ~31 or urgent
            or len(tcp) - size > 1024
            or checksum(CLIENT_IP + PEER_IP + struct.pack("!BBH", 0, 6, len(tcp)) + tcp)):
        raise ProtocolError("invalid TCP checksum, flags or bounds")
    owner(target)
    if not 40000 <= source < 40010:
        raise ProtocolError("unexpected local TCP port")
    options, at = tcp[20:size], 0
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
            raise ProtocolError("invalid MSS option")
        at += options[at + 1]
    return _wire.Segment(source, target, seq, ack, flags, window, options, tcp[size:])


@dataclass
class Connection:
    port: int
    source: int
    isn: int
    options: bytes
    server_isn: int
    phase: str
    syns: int = 1
    request: bytearray = field(default_factory=bytearray)
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
        return (self.server_start + len(RESPONSES[owner(self.port)[0]])) & MASK


class Machine:
    def __init__(self, case="normal"):
        if case not in CASES:
            raise ValueError("unknown native service peer case")
        self.case = case
        self.ports = PORTS if case == "normal" else (8084, *range(8090, 8095))
        self.slots = (0, 1) if case == "normal" else (1,)
        self.connections = {}
        self.events = []
        self.udp = set()
        self.ip_id = 1
        names = ("client_frames", "server_frames", "arp_requests", "arp_replies", "syns", "syn_acks",
                 "refusals", "resets_after_get", "first_syn_dropped", "retry_syns", "blackhole_syns",
                 "blackhole_rsts", "corrupt_data_sent", "valid_data_sent", "http_requests",
                 "valid_data_acks", "fins_sent", "client_fins_acked", "udp_requests",
                 "udp_corrupt_sent", "udp_wrong_scope_sent", "udp_echo_sent", "udp_overflow_sent")
        self.counts = dict.fromkeys(names, 0)
    def event(self, direction, label, frame, **fields):
        if len(self.events) >= MAX_EVENTS:
            raise ProtocolError("event budget exhausted")
        self.events.append(dict(sequence=len(self.events) + 1, direction=direction, event=label,
                                length=len(frame), frame_sha256=hashlib.sha256(frame).hexdigest(), **fields))
        self.counts["client_frames" if direction == "rx" else "server_frames"] += 1
    def ip(self, payload, protocol):
        ip = struct.pack("!BBHHHBBH4s4s", 0x45, 0, 20 + len(payload), self.ip_id, 0x4000, 64, protocol, 0, PEER_IP, CLIENT_IP)
        self.ip_id = (self.ip_id + 1) & 0xffff
        ip = ip[:10] + struct.pack("!H", checksum(ip)) + ip[12:]
        return (CLIENT_MAC + PEER_MAC + b"\x08\x00" + ip + payload).ljust(60, b"\0")
    def tcp(self, c, flags, *, seq, ack, label, payload=b"", options=b"", corrupt=False):
        tcp = struct.pack("!HHIIBBHHH", c.port, c.source, seq & MASK, ack & MASK,
                          ((20 + len(options)) // 4) << 4, flags, 4096, 0, 0) + options + payload
        value = checksum(PEER_IP + CLIENT_IP + struct.pack("!BBH", 0, 6, len(tcp)) + tcp)
        tcp = tcp[:16] + struct.pack("!H", value ^ int(corrupt)) + tcp[18:]
        packet = self.ip(tcp, 6)
        self.event("tx", label, packet, port=c.port, seq=seq & MASK, ack=ack & MASK,
                   flags=flags, payload_length=len(payload))
        return packet
    def syn_ack(self, c):
        self.counts["syn_acks"] += 1
        return self.tcp(c, SYN | ACK, seq=c.server_isn, ack=c.client_next, label="syn-ack", options=b"\x02\x04\x05\xb4")
    def valid_data(self, c):
        c.corrupt_due = None
        c.phase = "data-ack"
        self.counts["valid_data_sent"] += 1
        return self.tcp(c, PSH | ACK, seq=c.server_start, ack=c.client_next,
                        payload=RESPONSES[owner(c.port)[0]], label="http-data")
    @staticmethod
    def done(c):
        return c.closed or c.phase in ("refused", "reset", "blackhole")
    def arp(self, frame):
        broadcast = frame[:6] == b"\xff" * 6 and frame[32:38] == b"\xff" * 6
        if (len(frame) not in (42, 60) or frame[:6] not in (b"\xff" * 6, PEER_MAC)
                or frame[14:22] != struct.pack("!HHBBH", 1, 0x800, 6, 4, 1)
                or frame[22:28] != CLIENT_MAC or frame[28:32] != CLIENT_IP
                or (frame[32:38] not in (b"\0" * 6, PEER_MAC) and not broadcast)
                or frame[38:42] != PEER_IP):
            raise ProtocolError("invalid ARP request")
        self.event("rx", "arp-request", frame)
        packet = (CLIENT_MAC + PEER_MAC + b"\x08\x06" + struct.pack("!HHBBH", 1, 0x800, 6, 4, 2)
                  + PEER_MAC + PEER_IP + CLIENT_MAC + CLIENT_IP).ljust(60, b"\0")
        self.counts["arp_requests"] += 1
        self.counts["arp_replies"] += 1
        self.event("tx", "arp-reply", packet)
        return [packet]
    def udp_packet(self, slot, label, payload, *, wrong=False, corrupt=False):
        source, target = 9000 + slot + int(wrong) * 100, 45000 + slot
        udp = struct.pack("!HHHH", source, target, 8 + len(payload), 0) + payload
        value = checksum(PEER_IP + CLIENT_IP + struct.pack("!BBH", 0, 17, len(udp)) + udp) or 0xffff
        udp = udp[:6] + struct.pack("!H", value ^ int(corrupt)) + udp[8:]
        packet = self.ip(udp, 17)
        self.event("tx", label, packet, slot=slot, payload_length=len(payload))
        self.counts[label.replace("-", "_") + "_sent"] += 1
        return packet
    def udp_request(self, frame):
        ip = frame[14:]
        if len(ip) < 28:
            raise ProtocolError("short UDP frame")
        total = struct.unpack_from("!H", ip, 2)[0]
        if (frame[:6] != PEER_MAC or ip[0] != 0x45 or not ip[8] or ip[9] != 17
                or checksum(ip[:20]) or not 28 <= total <= len(ip)
                or struct.unpack_from("!H", ip, 6)[0] & 0xbfff
                or ip[12:16] != CLIENT_IP or ip[16:20] != PEER_IP
                or len(frame) not in (14 + total, max(60, 14 + total))):
            raise ProtocolError("invalid UDP IPv4 framing")
        udp = ip[20:total]
        source, target, size, check = struct.unpack_from("!HHHH", udp)
        if (source not in (45000, 45001) or target != 9000 + source - 45000
                or size != len(udp) or not check
                or checksum(CLIENT_IP + PEER_IP + struct.pack("!BBH", 0, 17, size) + udp)):
            raise ProtocolError("UDP tuple, size or checksum")
        slot = source - 45000
        if slot not in self.slots:
            raise ProtocolError("UDP from retired fault-case owner")
        if slot in self.udp or udp[8:] != UDP_PAYLOADS[slot]:
            raise ProtocolError("UDP duplicate or wrong synthetic payload")
        if 8084 + slot * 10 not in self.connections:
            raise ProtocolError("UDP before owner TCP scenarios")
        self.udp.add(slot)
        self.event("rx", "udp", frame, slot=slot, payload_length=len(udp) - 8)
        self.counts["udp_requests"] += 1
        payload = udp[8:]
        return [self.udp_packet(slot, "udp-corrupt", payload, corrupt=True),
                self.udp_packet(slot, "udp-wrong-scope", payload, wrong=True),
                self.udp_packet(slot, "udp-echo", payload),
                self.udp_packet(slot, "udp-overflow", b"overflow must not replace echo")]
    def handle(self, frame, now=0.0):
        if not 14 <= len(frame) <= MAX_FRAME or self.counts["client_frames"] >= MAX_FRAMES:
            raise ProtocolError("input frame budget")
        if frame[6:12] != CLIENT_MAC:
            raise ProtocolError("wrong client MAC")
        if frame[12:14] == b"\x08\x06":
            return self.arp(frame)
        if len(frame) > 23 and frame[12:14] == b"\x08\x00" and frame[23] == 17:
            return self.udp_request(frame)
        s = tcp_segment(frame)
        slot, relative = owner(s.target)
        expected_source = 40000 if self.case == "client-exit" and s.target == 8084 else 40000 + slot + 2 * relative
        if s.target not in self.ports or s.source != expected_source:
            raise ProtocolError("TCP tuple outside selected native peer case")
        self.event("rx", "tcp", frame, port=s.target, seq=s.seq, ack=s.ack,
                   flags=s.flags, payload_length=len(s.payload))
        c = self.connections.get(s.target)
        slot, case = owner(s.target)
        if s.flags & SYN:
            if s.flags != SYN or s.payload or s.ack:
                raise ProtocolError("noninitial SYN")
            self.counts["syns"] += 1
            if c is None:
                if any(previous.source == s.source for previous in self.connections.values()):
                    raise ProtocolError("local TCP port reused by another tuple")
                if case and not (self.case == "client-exit" and s.target == 8084) and (s.target - 1 not in self.connections or not self.done(self.connections[s.target - 1])):
                    raise ProtocolError("owner cases out of order")
                phase = "refused" if case == 1 else "blackhole" if case == 4 else "syn"
                c = Connection(s.target, s.source, s.seq, s.options, 0x6e6f7300 + PORTS.index(s.target) * 0x1000, phase)
                self.connections[s.target] = c
            else:
                if (s.source, s.seq, s.options) != (c.source, c.isn, c.options) or c.phase not in ("syn", "refused", "blackhole"):
                    raise ProtocolError("changed or late retransmitted SYN")
                c.syns += 1
            if case == 1:
                self.counts["refusals"] += 1
                return [self.tcp(c, RST | ACK, seq=0, ack=c.client_next, label="connection-refused")]
            if case == 4:
                self.counts["blackhole_syns"] += 1
                return []
            if case == 3:
                if c.syns == 1:
                    self.counts["first_syn_dropped"] += 1
                    return []
                self.counts["retry_syns"] += 1
            return [self.syn_ack(c)]
        if c is None or s.source != c.source:
            raise ProtocolError("TCP without bound tuple")
        if s.flags & RST:
            if case not in (1, 2, 4) or s.payload or s.flags not in (RST, RST | ACK) or s.seq != c.client_next:
                raise ProtocolError("unexpected client RST")
            expected_ack = c.server_start if case == 2 and s.flags & ACK else 0
            if s.ack != expected_ack:
                raise ProtocolError("wrong client RST acknowledgment")
            if case == 4:
                self.counts["blackhole_rsts"] += 1
            return []
        if not s.flags & ACK or s.flags & ~(ACK | PSH | FIN) or c.phase in ("refused", "reset", "blackhole"):
            raise ProtocolError("data outside established connection")
        if c.phase == "syn":
            if s.seq != c.client_next or s.ack != c.server_start:
                raise ProtocolError("bad handshake acknowledgment")
            c.phase = "request"
        if c.closed and s.flags == ACK and s.seq == (c.client_next + 1) & MASK:
            if s.ack != (c.data_end + 1) & MASK or s.payload:
                raise ProtocolError("wrong final ACK")
            return []
        if s.seq != c.client_next:
            raise ProtocolError("unexpected client sequence")
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
            if case == 2:
                c.phase = "reset"
                self.counts["resets_after_get"] += 1
                return [self.tcp(c, RST | ACK, seq=c.server_start, ack=c.client_next, label="reset-after-get")]
            if case == 0:
                c.phase = "corrupt-delay"
                c.corrupt_due = now + CORRUPT_DELAY
                self.counts["corrupt_data_sent"] += 1
                return [self.tcp(c, PSH | ACK, seq=c.server_start, ack=c.client_next,
                                 payload=b"X" + RESPONSES[slot][1:], corrupt=True, label="bad-tcp-checksum")]
            return [self.valid_data(c)]
        if s.payload:
            raise ProtocolError("extra HTTP request bytes")
        if c.phase == "corrupt-delay":
            if s.ack != c.server_start or s.flags & FIN:
                raise ProtocolError("client acknowledged corrupt TCP bytes")
            return []
        if c.phase == "data-ack":
            if s.ack == c.server_start and not s.flags & FIN:
                return []
            if s.ack != c.data_end or s.flags & FIN:
                raise ProtocolError("wrong data ACK or premature FIN")
            c.data_acked, c.fin_sent, c.phase = True, True, "fin-ack"
            self.counts["valid_data_acks"] += 1
            self.counts["fins_sent"] += 1
            return [self.tcp(c, FIN | ACK, seq=c.data_end, ack=c.client_next, label="server-fin")]
        if c.phase in ("fin-ack", "closed"):
            final = (c.data_end + 1) & MASK
            if s.ack != final:
                raise ProtocolError("wrong FIN acknowledgment")
            if s.flags & FIN:
                label = "duplicate-client-fin-ack" if c.closed else "client-fin-ack"
                if not c.closed:
                    self.counts["client_fins_acked"] += 1
                c.closed, c.phase = True, "closed"
                return [self.tcp(c, ACK, seq=final, ack=(c.client_next + 1) & MASK, label=label)]
            return []
        raise ProtocolError("unhandled TCP phase")
    def poll(self, now):
        return [self.valid_data(c) for c in self.connections.values()
                if c.corrupt_due is not None and now >= c.corrupt_due]
    def summary(self):
        cases = {str(p): dict(phase=c.phase, source=c.source, syns=c.syns, request_bytes=len(c.request),
                             request_sha256=hashlib.sha256(c.request).hexdigest(), data_acked=c.data_acked,
                             fin_sent=c.fin_sent, closed=c.closed) for p, c in self.connections.items()}
        success_ports = (8080, 8083, 8090, 8093) if self.case == "normal" else (8090, 8093)
        owners = len(self.slots)
        complete = (set(self.connections) == set(self.ports) and self.udp == set(self.slots)
                    and {c.source for c in self.connections.values()} == (set(range(40000, 40010)) if self.case == "normal" else {40000, 40001, 40003, 40005, 40007, 40009})
                    and all(self.done(c) for c in self.connections.values())
                    and all(self.connections[p].closed and self.connections[p].data_acked
                            for p in success_ports)
                    and self.counts["first_syn_dropped"] == owners and self.counts["retry_syns"] >= owners
                    and self.counts["corrupt_data_sent"] == owners and self.counts["valid_data_sent"] == 2 * owners
                    and self.counts["client_fins_acked"] == 2 * owners)
        return dict(case=self.case, complete=complete, counts=dict(self.counts), cases=cases, events=list(self.events),
                    request_sha256=[hashlib.sha256(r).hexdigest() for r in REQUESTS],
                    response_sha256=[hashlib.sha256(r).hexdigest() for r in RESPONSES],
                    udp_sha256=[hashlib.sha256(r).hexdigest() for r in UDP_PAYLOADS])


def validate_transcript(evidence, *, case=None):
    selected = evidence.get("case") if case is None else case
    if selected not in CASES or evidence.get("case") != selected:
        raise ProtocolError("peer transcript case binding changed")
    actions = evidence.get("actions")
    if not isinstance(actions, list) or not 1 <= len(actions) <= MAX_EVENTS:
        raise ProtocolError("missing/bounded raw transcript")
    machine, last = Machine(selected), 0.0
    for action in actions:
        if not isinstance(action, dict) or set(action) not in ({"kind", "at", "input", "output"}, {"kind", "at", "output"}):
            raise ProtocolError("malformed transcript action")
        now = action["at"]
        if type(now) not in (int, float) or not last <= now <= 180:
            raise ProtocolError("changed/nonfinite transcript time")
        last = now
        expected = action["output"]
        if not isinstance(expected, list) or len(expected) > 4:
            raise ProtocolError("unbounded response list")
        def packet(value):
            if not isinstance(value, str) or not 28 <= len(value) <= MAX_FRAME * 2 or len(value) % 2 or re.fullmatch("[0-9a-f]+", value) is None:
                raise ProtocolError("malformed recorded packet")
            return bytes.fromhex(value)
        outputs = [packet(p) for p in expected]
        if action["kind"] == "input" and "input" in action:
            actual = machine.handle(packet(action["input"]), now)
        elif action["kind"] == "poll" and "input" not in action and outputs:
            actual = machine.poll(now)
        else:
            raise ProtocolError("unknown/empty transcript action")
        if actual != outputs:
            raise ProtocolError("server reply differs from actual triggering guest bytes")
    summary = machine.summary()
    if not summary["complete"] or any(evidence.get(k) != v for k, v in summary.items()):
        raise ProtocolError("transcript contradicts summary/counters or incomplete cases")
    if evidence.get("wire_frames_sent") != summary["counts"]["server_frames"]:
        raise ProtocolError("not all causal replies reached wire")
    return summary


class Peer:
    def __init__(self, timeout=180, *, case="normal"):
        if type(timeout) not in (int, float) or not 0 < timeout <= 180:
            raise ValueError("bounded peer lifetime required")
        self.timeout, self.port = timeout, 0
        self.machine, self.actions = Machine(case), []
        self.socket = self.thread = None
        self.stop = threading.Event()
        self.connected, self.error, self.rejected_frame = False, None, None
        self.wire_frames_sent = 0
    def start(self):
        if self.thread is not None or self.stop.is_set():
            raise RuntimeError("peer is single use")
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.bind(("127.0.0.1", 0))
        self.socket.listen(1)
        self.socket.settimeout(0.05)
        self.port = self.socket.getsockname()[1]
        self.thread = threading.Thread(target=self._run, name="nieuw-os-network-service-peer", daemon=True)
        self.thread.start()
        return self
    def _run(self):
        started, connection, decoder = time.monotonic(), None, FrameDecoder()
        def send(packets):
            for p in packets:
                connection.sendall(framed(p))
                self.wire_frames_sent += 1
        try:
            while not self.stop.is_set() and time.monotonic() - started < self.timeout:
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
                    now = time.monotonic() - started
                    packets = self.machine.poll(now)
                    if packets:
                        self.actions.append(dict(kind="poll", at=now, output=[p.hex() for p in packets]))
                        send(packets)
                    continue
                if not data:
                    decoder.finish()
                    break
                for frame in decoder.feed(data):
                    now = time.monotonic() - started
                    try:
                        packets = self.machine.handle(frame, now)
                    except ProtocolError:
                        self.rejected_frame = dict(length=len(frame), prefix_hex=frame[:128].hex())
                        raise
                    self.actions.append(dict(kind="input", at=now, input=frame.hex(), output=[p.hex() for p in packets]))
                    if len(self.actions) > MAX_EVENTS:
                        raise ProtocolError("action budget exhausted")
                    send(packets)
            decoder.finish()
            if time.monotonic() - started >= self.timeout:
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
                raise RuntimeError("test peer shutdown exceeded bound")
    def finish(self):
        self.close()
        result = self.machine.summary()
        result.update(kind="synthetic Ethernet server; native client evidence required separately",
                      connected=self.connected, error=self.error, rejected_frame=self.rejected_frame,
                      wire_frames_sent=self.wire_frames_sent, actions=list(self.actions), passed=False)
        if self.connected and self.error is None and result["complete"]:
            try:
                validate_transcript(result)
                result["passed"] = True
            except ProtocolError as error:
                result["error"] = str(error)
        return result
    def __enter__(self):
        return self.start()
    def __exit__(self, *_):
        self.close()
