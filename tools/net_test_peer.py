#!/usr/bin/env python3
"""Bounded synthetic Ethernet counterparty for a real Nieuw OS QEMU client.

Only listens on ephemeral 127.0.0.1. It replies to received ARP/UDP/DNS packets;
it is not a host implementation of the OS client and emits no guest markers.
"""
from __future__ import annotations

import hashlib
import socket
import struct
import threading
import time

MAX_FRAME = 1514
MAX_FRAMES = 64
MAX_PAYLOAD = 256
CLIENT_MAC = bytes.fromhex("020000000007")
PEER_MAC = bytes.fromhex("020000000001")
CLIENT_IP = bytes((192, 0, 2, 2))
PEER_IP = bytes((192, 0, 2, 1))
DNS_IP = bytes((192, 0, 2, 53))
ANSWER_IP = bytes((198, 51, 100, 42))
DNS_NAME = "native.nieuw.test"
TIMEOUT_NAME = "timeout.nieuw.test"


class ProtocolError(ValueError):
    """A malformed or out-of-contract synthetic protocol input."""


def checksum(data: bytes) -> int:
    if len(data) & 1:
        data += b"\0"
    total = sum(struct.unpack(f"!{len(data) // 2}H", data)) if data else 0
    while total >> 16:
        total = (total & 0xffff) + (total >> 16)
    return (~total) & 0xffff


def framed(frame: bytes) -> bytes:
    if not 14 <= len(frame) <= MAX_FRAME:
        raise ProtocolError("Ethernet length outside fixture bound")
    return struct.pack("!I", len(frame)) + frame


class FrameDecoder:
    """Streaming QEMU socketnetdev framing; reject length before allocation."""

    def __init__(self):
        self.header = bytearray()
        self.payload = bytearray()
        self.expected: int | None = None
        self.frames = 0
        self.failed = False

    def feed(self, data: bytes) -> list[bytes]:
        if self.failed:
            raise ProtocolError("framing already failed")
        output = []
        position = 0
        try:
            while position < len(data):
                if self.expected is None:
                    count = min(4 - len(self.header), len(data) - position)
                    self.header.extend(data[position:position + count])
                    position += count
                    if len(self.header) != 4:
                        continue
                    self.expected = struct.unpack("!I", self.header)[0]
                    self.header.clear()
                    if not 14 <= self.expected <= MAX_FRAME:
                        raise ProtocolError("untrusted socket frame length")
                count = min(self.expected - len(self.payload), len(data) - position)
                self.payload.extend(data[position:position + count])
                position += count
                if len(self.payload) == self.expected:
                    self.frames += 1
                    if self.frames > MAX_FRAMES:
                        raise ProtocolError("connection frame budget exceeded")
                    output.append(bytes(self.payload))
                    self.payload.clear()
                    self.expected = None
        except ProtocolError:
            self.failed = True
            self.header.clear()
            self.payload.clear()
            raise
        return output

    def finish(self) -> None:
        if self.failed or self.header or self.payload or self.expected is not None:
            raise ProtocolError("connection closed with incomplete/bad frame")


def arp_reply(*, bad_sender: bool = False, wrong_target: bool = False) -> bytes:
    source = bytes.fromhex("020000000009") if bad_sender else PEER_MAC
    target = bytes((192, 0, 2, 99)) if wrong_target else CLIENT_IP
    packet = (CLIENT_MAC + source + b"\x08\x06" +
              struct.pack("!HHBBH", 1, 0x0800, 6, 4, 2) +
              PEER_MAC + PEER_IP + CLIENT_MAC + target)
    return packet.ljust(60, b"\0")


def udp_frame(payload: bytes, source_ip: bytes, source_port: int, target_port: int) -> bytes:
    if len(payload) > MAX_PAYLOAD:
        raise ProtocolError("fixture UDP payload too large")
    length = 8 + len(payload)
    udp = struct.pack("!HHHH", source_port, target_port, length, 0) + payload
    pseudo = source_ip + CLIENT_IP + struct.pack("!BBH", 0, 17, length)
    value = checksum(pseudo + udp) or 0xffff
    udp = udp[:6] + struct.pack("!H", value) + udp[8:]
    ip = struct.pack("!BBHHHBBH4s4s", 0x45, 0, 20 + length, 0x4310,
                     0x4000, 64, 17, 0, source_ip, CLIENT_IP)
    ip = ip[:10] + struct.pack("!H", checksum(ip)) + ip[12:]
    return (CLIENT_MAC + PEER_MAC + b"\x08\x00" + ip + udp).ljust(60, b"\0")


def parse_udp(frame: bytes) -> dict:
    if not 42 <= len(frame) <= MAX_FRAME or frame[12:14] != b"\x08\x00":
        raise ProtocolError("not a bounded IPv4 UDP frame")
    ip = frame[14:]
    if ip[0] != 0x45 or checksum(ip[:20]) != 0 or ip[8] == 0 or ip[9] != 17:
        raise ProtocolError("unsupported or corrupt IPv4 header")
    total = struct.unpack_from("!H", ip, 2)[0]
    flags = struct.unpack_from("!H", ip, 6)[0]
    if total < 28 or total > len(ip) or flags & 0xbfff:
        raise ProtocolError("invalid IP length/fragment flags")
    end = 14 + total
    if len(frame) not in (end, max(60, end)):
        raise ProtocolError("unexpected Ethernet padding/trailer")
    udp = ip[20:total]
    source_port, target_port, length, transmitted = struct.unpack_from("!HHHH", udp)
    if length != len(udp) or transmitted == 0:
        raise ProtocolError("invalid or unchecked UDP length")
    pseudo = ip[12:20] + struct.pack("!BBH", 0, 17, length)
    if checksum(pseudo + udp) != 0:
        raise ProtocolError("corrupt UDP payload or pseudoheader")
    if len(udp) - 8 > MAX_PAYLOAD:
        raise ProtocolError("UDP fixture payload budget exceeded")
    return dict(source_mac=frame[6:12], target_mac=frame[:6], source_ip=ip[12:16],
                target_ip=ip[16:20], source_port=source_port, target_port=target_port,
                payload=udp[8:])


def parse_query(message: bytes) -> tuple[int, str, bytes]:
    if not 17 <= len(message) <= MAX_PAYLOAD:
        raise ProtocolError("DNS query length")
    identifier, flags, questions, answers, authority, extra = struct.unpack_from("!6H", message)
    if flags != 0x0100 or (questions, answers, authority, extra) != (1, 0, 0, 0):
        raise ProtocolError("DNS query header outside fixture")
    cursor = 12
    labels = []
    wire_size = 0
    while True:
        if cursor >= len(message):
            raise ProtocolError("truncated query name")
        length = message[cursor]
        cursor += 1
        wire_size += length + 1
        if length == 0:
            break
        if length > 63 or wire_size > 255 or cursor + length > len(message):
            raise ProtocolError("bad/compressed query label")
        label = message[cursor:cursor + length]
        if not all(97 <= byte <= 122 or 48 <= byte <= 57 or byte == 45 for byte in label):
            raise ProtocolError("noncanonical fixture DNS label")
        labels.append(label.decode("ascii"))
        cursor += length
    if message[cursor:] != b"\0\x01\0\x01":
        raise ProtocolError("wrong DNS query type/class/trailer")
    return identifier, ".".join(labels), message[12:]


def dns_reply(query: bytes, mode: str) -> bytes:
    identifier, name, question = parse_query(query)
    if name != DNS_NAME or mode not in ("wrong-id", "bad-name", "answer"):
        raise ProtocolError("unexpected DNS response fixture")
    if mode == "wrong-id":
        identifier ^= 1
    header = struct.pack("!6H", identifier, 0x8180, 1, 1, 0, 0)
    owner_start = len(header) + len(question)
    owner = (b"\x01x" + struct.pack("!H", 0xc000 | owner_start)
             if mode == "bad-name" else b"\xc0\x0c")
    return header + question + owner + struct.pack("!HHIH", 1, 1, 60, 4) + ANSWER_IP


class PeerMachine:
    """Deterministic server fault schedule driven solely by actual requests."""

    def __init__(self):
        self.counts = dict(accepted_frames=0, arp_requests=0, arp_bad_sender_sent=0,
                           arp_wrong_target_sent=0, arp_valid_sent=0,
                           udp_echo_requests=0, udp_echo_replies=0, dns_requests=0,
                           dns_wrong_id_sent=0, dns_bad_name_sent=0, dns_answers_sent=0,
                           dns_timeouts=0, response_frames=0)
        self.events: list[dict] = []

    def handle(self, frame: bytes) -> list[bytes]:
        if not 14 <= len(frame) <= MAX_FRAME or self.counts["accepted_frames"] >= MAX_FRAMES:
            raise ProtocolError("peer input length/frame budget")
        if frame[6:12] != CLIENT_MAC:
            raise ProtocolError("unexpected client MAC")
        responses = []
        details = {}
        if frame[12:14] == b"\x08\x06":
            if (len(frame) not in (42, 60) or frame[:6] != b"\xff" * 6 or
                    frame[14:22] != struct.pack("!HHBBH", 1, 0x0800, 6, 4, 1) or
                    frame[22:28] != CLIENT_MAC or frame[28:32] != CLIENT_IP or
                    frame[32:38] != b"\0" * 6 or frame[38:42] != PEER_IP):
                raise ProtocolError("invalid ARP fixture request")
            sequence = self.counts["arp_requests"]
            if sequence >= 3:
                raise ProtocolError("unexpected extra ARP request")
            event = ("arp_bad_sender_sent", "arp_wrong_target_sent", "arp_valid_sent")[sequence]
            details = dict(requested_ipv4="192.0.2.1")
            responses = [arp_reply(bad_sender=sequence == 0, wrong_target=sequence == 1)]
            self.counts["arp_requests"] += 1
        elif frame[12:14] == b"\x08\x00":
            packet = parse_udp(frame)
            if packet["source_ip"] != CLIENT_IP or packet["target_mac"] != PEER_MAC:
                raise ProtocolError("unexpected UDP client/next-hop")
            if packet["target_ip"] == PEER_IP and packet["target_port"] == 9000:
                if self.counts["udp_echo_requests"] != 0 or not packet["payload"]:
                    raise ProtocolError("invalid/extra echo")
                event = "udp_echo_replies"
                details = dict(payload_sha256=hashlib.sha256(packet["payload"]).hexdigest())
                self.counts["udp_echo_requests"] += 1
                responses = [udp_frame(packet["payload"], PEER_IP, 9000, packet["source_port"])]
            elif packet["target_ip"] == DNS_IP and packet["target_port"] == 53:
                identifier, name, _ = parse_query(packet["payload"])
                details = dict(question_name=name, transaction_id=identifier)
                if name == TIMEOUT_NAME:
                    if self.counts["dns_timeouts"] != 0:
                        raise ProtocolError("extra timeout query")
                    event = "dns_timeouts"
                elif name == DNS_NAME:
                    sequence = self.counts["dns_requests"]
                    if sequence >= 3:
                        raise ProtocolError("extra answered query")
                    mode = ("wrong-id", "bad-name", "answer")[sequence]
                    event = ("dns_wrong_id_sent", "dns_bad_name_sent", "dns_answers_sent")[sequence]
                    responses = [udp_frame(dns_reply(packet["payload"], mode), DNS_IP, 53, packet["source_port"])]
                else:
                    raise ProtocolError("unexpected query name")
                self.counts["dns_requests"] += 1
            else:
                raise ProtocolError("unsupported fixture UDP destination")
        else:
            raise ProtocolError("unsupported fixture EtherType")
        self.counts[event] += 1
        self.counts["accepted_frames"] += 1
        self.counts["response_frames"] += len(responses)
        self.events.append(dict(sequence=len(self.events) + 1, event=event,
                                request_sha256=hashlib.sha256(frame).hexdigest(),
                                response_sha256=[hashlib.sha256(value).hexdigest() for value in responses],
                                **details))
        return responses


class EthernetPeer:
    """One localhost QEMU connection; bounded framing, bytes and lifetime."""

    def __init__(self, timeout: float = 180):
        if not 0 < timeout <= 180:
            raise ValueError("peer lifetime outside bound")
        self.timeout = timeout
        self.machine = PeerMachine()
        self.error: str | None = None
        self.rejected_frame: dict | None = None
        self.connected = False
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.bind(("127.0.0.1", 0))
        self.socket.listen(1)
        self.socket.settimeout(0.1)
        self.port = self.socket.getsockname()[1]
        self.stop_event = threading.Event()
        self.thread = threading.Thread(target=self._run, name="nieuw-os-test-peer", daemon=True)
        self.thread.start()

    def _run(self) -> None:
        deadline = time.monotonic() + self.timeout
        connection = None
        decoder = FrameDecoder()
        try:
            while not self.stop_event.is_set() and time.monotonic() < deadline:
                if connection is None:
                    try:
                        connection, address = self.socket.accept()
                    except socket.timeout:
                        continue
                    if address[0] != "127.0.0.1":
                        raise ProtocolError("nonlocal counterparty")
                    connection.settimeout(0.1)
                    self.connected = True
                try:
                    data = connection.recv(4096)
                except socket.timeout:
                    continue
                if not data:
                    decoder.finish()
                    break
                for frame in decoder.feed(data):
                    try:
                        responses = self.machine.handle(frame)
                    except ProtocolError:
                        # Synthetic test traffic only. Preserve bounded diagnostics;
                        # an unknown frame remains a failing protocol observation.
                        self.rejected_frame = dict(length=len(frame), ethertype=frame[12:14].hex(),
                                                   prefix_hex=frame[:64].hex(),
                                                   sha256=hashlib.sha256(frame).hexdigest())
                        raise
                    for response in responses:
                        connection.sendall(framed(response))
            decoder.finish()
            if time.monotonic() >= deadline:
                raise TimeoutError("peer lifetime exhausted")
        except Exception as error:
            if not (self.stop_event.is_set() and connection is None and isinstance(error, OSError)):
                self.error = f"{type(error).__name__}: {error}"
        finally:
            if connection is not None:
                connection.close()

    def close(self) -> None:
        self.stop_event.set()
        self.socket.close()
        self.thread.join(2)
        if self.thread.is_alive():
            raise RuntimeError("bounded testpeer shutdown failed")

    def evidence(self) -> dict:
        if self.thread.is_alive():
            raise RuntimeError("stop the testpeer before reading final evidence")
        return dict(kind="synthetic protocol counterparty; not OS client", connected=self.connected,
                    error=self.error, rejected_frame=self.rejected_frame,
                    counts=dict(self.machine.counts), events=list(self.machine.events))

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()
