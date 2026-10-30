"""Recheck the semantics recorded by the bounded synthetic TCP server.

This is a trace validator, not another packet parser or proof of packet bytes.
The peer parses/checks actual frames; source/artifact binding and native results
are checked by the caller. Here fields, causal order and independently derived
totals must agree. Only the five explicitly fixed fixture conversations apply.
"""
from __future__ import annotations

from dataclasses import dataclass
import hashlib
import re

PORTS = (8080, 8081, 8082, 8083, 8084)
FIN, SYN, RST, PSH, ACK = 1, 2, 4, 8, 16
MASK = (1 << 32) - 1
REQUEST = b"GET /nieuw-os-tcp HTTP/1.1\r\nHost: fixture.invalid\r\nConnection: close\r\n\r\n"
RESPONSE = b"HTTP/1.1 200 OK\r\nContent-Length: 20\r\nConnection: close\r\n\r\nNieuw OS native TCP\n"
COUNTERS = ("client_frames", "server_frames", "arp_requests", "arp_replies", "syns",
            "syn_acks", "refusals", "resets_after_get", "first_syn_dropped", "retry_syns",
            "blackhole_syns", "blackhole_rsts", "corrupt_data_sent", "valid_data_sent",
            "http_requests", "valid_data_acks", "fins_sent", "client_fins_acked")
TX_COUNTERS = {"arp-reply": "arp_replies", "syn-ack": "syn_acks",
               "connection-refused": "refusals", "reset-after-get": "resets_after_get",
               "bad-tcp-checksum": "corrupt_data_sent", "http-data": "valid_data_sent",
               "server-fin": "fins_sent", "client-fin-ack": "client_fins_acked"}
TX_LABELS = set(TX_COUNTERS) | {"request-ack", "duplicate-client-fin-ack"}


def _integer(value, low: int, high: int) -> int:
    if type(value) is not int or not low <= value <= high:
        raise ValueError("Missing or out-of-range TCP transcript integer")
    return value


def _require(condition: bool, reason: str) -> None:
    if not condition:
        raise ValueError(reason)


@dataclass
class _Connection:
    port: int
    isn: int
    syn_length: int
    syns: int = 0
    request_bytes: int = 0
    phase: str = "syn"
    data_acked: bool = False
    fin_sent: bool = False
    closed: bool = False
    corrupt_hash: str | None = None

    @property
    def client_next(self) -> int:
        return (self.isn + 1 + self.request_bytes) & MASK

    @property
    def server_start(self) -> int:
        return (0x6e6f7300 + PORTS.index(self.port) * 0x1000 + 1) & MASK

    def terminal(self) -> bool:
        return self.closed or self.phase in ("refused", "reset", "blackhole")


def validate_transcript(peer: dict, *, request_length: int, response_length: int) -> None:
    """Reject inconsistent/missing handshake, HTTP, retry, CRC or FIN evidence.

    SYN identity covers the recorded port/sequence/ACK/flags/payload/length.
    TCP option bytes and checksums were checked by the source-bound peer, and
    cannot be reconstructed from an opaque frame hash in this compact trace.
    """
    _require(type(peer) is dict, "Missing TCP peer object")
    _require(type(request_length) is int and request_length == len(REQUEST)
             and type(response_length) is int and response_length == len(RESPONSE),
             "Unexpected synthetic HTTP fixture lengths")
    events = peer.get("events")
    _require(isinstance(events, list) and 1 <= len(events) <= 1024, "Missing/big TCP transcript")
    counts = dict.fromkeys(COUNTERS, 0)
    connections: dict[int, _Connection] = {}
    pending: tuple | None = None

    def answer(label: str, c: _Connection, seq: int, ack: int, flags: int, payload=0):
        return label, c.port, seq & MASK, ack & MASK, flags, payload

    for index, event in enumerate(events, 1):
        _require(isinstance(event, dict), "Malformed transcript event")
        _require(_integer(event.get("sequence"), 1, 1024) == index, "Transcript sequence changed")
        direction, label = event.get("direction"), event.get("event")
        _require(direction in ("rx", "tx"), "Unknown transcript direction")
        _require(isinstance(label, str), "Missing transcript label")
        digest = event.get("frame_sha256")
        _require(isinstance(digest, str) and re.fullmatch(r"[0-9a-f]{64}", digest) is not None,
                 "Malformed packet hash")
        length = _integer(event.get("length"), 14, 1514)
        counts["client_frames" if direction == "rx" else "server_frames"] += 1
        if label in ("arp-request", "arp-reply"):
            _require(length in (42, 60), "Bad recorded ARP frame size")
            _require(not any(k in event for k in ("port", "seq", "ack", "flags", "payload_length")),
                     "ARP event carries unrelated TCP fields")
            if direction == "rx" and label == "arp-request":
                _require(pending is None, "Client frame before required server reply")
                counts["arp_requests"] += 1
                pending = ("arp-reply",)
            else:
                _require(direction == "tx" and label == "arp-reply" and pending == ("arp-reply",)
                         and length == 60, "Unsolicited/missing ARP reply")
                counts["arp_replies"] += 1
                pending = None
            continue

        _require(label == "tcp" if direction == "rx" else label in TX_LABELS - {"arp-reply"},
                 "Unknown or wrongly directed TCP event")
        port = _integer(event.get("port"), PORTS[0], PORTS[-1])
        seq = _integer(event.get("seq"), 0, MASK)
        ack = _integer(event.get("ack"), 0, MASK)
        flags = _integer(event.get("flags"), 0, 31)
        payload = _integer(event.get("payload_length"), 0, 1024)
        _require(54 + payload <= length <= max(60, 94 + payload), "TCP length disagrees with payload")
        c = connections.get(port)
        if direction == "tx":
            _require(c is not None and port != 8084, "Server transmitted without SYN/to blackhole")
            actual = label, port, seq, ack, flags, payload
            if pending is None:
                _require(label == "http-data" and port == 8080 and c.phase == "corrupt",
                         "Unsolicited server event")
                expected = answer("http-data", c, c.server_start, c.client_next, PSH | ACK, response_length)
            else:
                expected = pending
            _require(actual == expected, "Server fields/order disagree with triggering client segment")
            _require(length == max(60, 54 + payload), "Server packet length changed")
            pending = None
            if label in TX_COUNTERS:
                counts[TX_COUNTERS[label]] += 1
            if label == "bad-tcp-checksum":
                c.corrupt_hash = digest
                c.phase = "corrupt"
            elif label == "http-data":
                if port == 8080:
                    _require(c.corrupt_hash is not None and digest != c.corrupt_hash,
                             "Corrupt and valid response have the same packet hash")
                c.phase = "data-ack"
            elif label == "server-fin":
                c.phase = "fin-ack"
                c.fin_sent = True
            continue

        _require(pending is None, "Client event precedes required server response")
        if flags == SYN:
            _require(ack == 0 and payload == 0, "Changed SYN fields")
            if c is None:
                ordinal = PORTS.index(port)
                _require(len(connections) == ordinal and
                         (ordinal == 0 or connections[PORTS[ordinal - 1]].terminal()),
                         "TCP scenarios started out of order/before prior completion")
                c = _Connection(port, seq, length)
                if port == 8081:
                    c.phase = "refused"
                elif port == 8084:
                    c.phase = "blackhole"
                connections[port] = c
            _require(seq == c.isn and length == c.syn_length
                     and c.phase in ("syn", "refused", "blackhole"), "Changed/late SYN transaction")
            c.syns += 1
            counts["syns"] += 1
            if port == 8081:
                pending = answer("connection-refused", c, 0, c.client_next, RST | ACK)
            elif port == 8084:
                counts["blackhole_syns"] += 1
            elif port == 8083 and c.syns == 1:
                counts["first_syn_dropped"] += 1
            else:
                if port == 8083:
                    counts["retry_syns"] += 1
                pending = answer("syn-ack", c, c.server_start - 1, c.client_next, SYN | ACK)
            continue

        _require(c is not None, "Client TCP data without SYN")
        if flags & RST:
            _require(port in (8081, 8082, 8084) and c.terminal() and payload == 0
                     and flags in (RST, RST | ACK) and seq == c.client_next,
                     "Unexpected client reset/sequence")
            expected_ack = c.server_start if port == 8082 and flags & ACK else 0
            _require(ack == expected_ack, "Unexpected client reset acknowledgment")
            if port == 8084:
                counts["blackhole_rsts"] += 1
            continue
        _require(flags in (ACK, ACK | PSH, ACK | FIN, ACK | FIN | PSH), "Unexpected client flags")
        _require(c.phase not in ("refused", "reset", "blackhole"), "Data on terminated connection")
        if c.phase == "syn":
            _require(c.syns >= (2 if port == 8083 else 1)
                     and seq == c.client_next and ack == c.server_start,
                     "Handshake ACK missing/wrong or first retry SYN answered")
            c.phase = "request"
        end = (c.server_start + response_length) & MASK
        if c.closed and flags == ACK and seq == (c.client_next + 1) & MASK:
            _require(ack == (end + 1) & MASK and payload == 0, "Wrong post-close ACK")
            continue
        _require(seq == c.client_next, "Unexpected client sequence")
        if c.phase == "request":
            _require(ack == c.server_start and not flags & FIN, "Premature request FIN/wrong ACK")
            c.request_bytes += payload
            _require(c.request_bytes <= request_length, "HTTP request byte count overflow")
            if payload == 0:
                continue
            if c.request_bytes < request_length:
                pending = answer("request-ack", c, c.server_start, c.client_next, ACK)
            else:
                counts["http_requests"] += 1
                if port == 8082:
                    c.phase = "reset"
                    pending = answer("reset-after-get", c, c.server_start, c.client_next, RST | ACK)
                elif port == 8080:
                    c.phase = "await-corrupt"
                    pending = answer("bad-tcp-checksum", c, c.server_start, c.client_next, PSH | ACK, response_length)
                else:
                    c.phase = "await-data"
                    pending = answer("http-data", c, c.server_start, c.client_next, PSH | ACK, response_length)
        else:
            _require(payload == 0, "Request payload after completion")
            if c.phase == "corrupt":
                _require(ack == c.server_start and not flags & FIN, "Client acknowledged corrupt-only data")
            elif c.phase == "data-ack":
                _require(not flags & FIN, "Client FIN before server EOF")
                if ack == c.server_start:
                    continue
                _require(ack == end, "Wrong HTTP data acknowledgment")
                c.data_acked = True
                counts["valid_data_acks"] += 1
                pending = answer("server-fin", c, end, c.client_next, FIN | ACK)
            elif c.phase in ("fin-ack", "closed"):
                _require(ack == (end + 1) & MASK, "Wrong server FIN acknowledgment")
                if flags & FIN:
                    label = "duplicate-client-fin-ack" if c.closed else "client-fin-ack"
                    c.closed = True
                    c.phase = "closed"
                    pending = answer(label, c, end + 1, c.client_next + 1, ACK)
            else:
                raise ValueError("Incomplete server phase")

    _require(pending is None, "Transcript ends before required server response")
    _require(tuple(connections) == PORTS and all(c.terminal() for c in connections.values()),
             "Incomplete TCP scenarios")
    _require(peer.get("complete") is True, "Missing complete server result")
    actual_counts = peer.get("counts")
    _require(isinstance(actual_counts, dict) and set(actual_counts) == set(counts), "Counter fields changed")
    for name, count in counts.items():
        _require(_integer(actual_counts[name], 0, 1024) == count, "Packet-derived counter differs: " + name)
    cases = peer.get("cases")
    _require(isinstance(cases, dict) and set(cases) == {str(p) for p in PORTS}, "Missing connection summaries")
    for port, c in connections.items():
        expected_bytes = request_length if port in (8080, 8082, 8083) else 0
        _require(c.request_bytes == expected_bytes, "Incomplete request")
        if port in (8080, 8083):
            _require(c.closed and c.data_acked and c.fin_sent, "HTTP lacked data ACK/FIN exchange")
        case = cases[str(port)]
        _require(isinstance(case, dict), "Malformed connection summary")
        for name, value in (("phase", c.phase), ("syns", c.syns), ("request_bytes", c.request_bytes),
                            ("data_acked", c.data_acked), ("fin_sent", c.fin_sent), ("closed", c.closed)):
            _require(type(case.get(name)) is type(value) and case[name] == value,
                     "Connection summary contradicts packets: " + name)
        expected_hash = hashlib.sha256(REQUEST if expected_bytes else b"").hexdigest()
        _require(case.get("request_sha256") == expected_hash, "Different HTTP request fixture")
    _require(peer.get("response_sha256") == hashlib.sha256(RESPONSE).hexdigest()
             and peer.get("corrupt_response_sha256") == hashlib.sha256(b"X" + RESPONSE[1:]).hexdigest()
             and type(peer.get("response_body_bytes")) is int and peer["response_body_bytes"] == 20,
             "Different valid/corrupt response fixture")
