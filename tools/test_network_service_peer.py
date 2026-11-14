#!/usr/bin/env python3
"""Host adversarial checks of N01 peer/harness; no native success claim."""
from __future__ import annotations
import os
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from network_service_peer import (ACK, FIN, PSH, RST, SYN, CLIENT_IP, CLIENT_MAC, PROBES, PROBE_ISN,
                                  PEER_IP, PEER_MAC, REQUESTS, RESPONSES, RESET_PROBE, UDP_PAYLOADS,
                                  FrameDecoder, Machine, Peer, ProtocolError, checksum,
                                  framed, validate_transcript)
from run_network_service import BOOT, OFFSETS, REVOKE_ROLES, ROLES, ROOT, execute_native, record_streams, sequence


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


def transcript(case="normal"):
    """Construct legal synthetic client packets only for host parser checks."""
    machine, actions, now = Machine(case), [], 0.0
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
    if case == "normal":
        # Two owners open before either HTTP transaction completes.
        feed(client_tcp(8080, 40000, 100))
        feed(client_tcp(8090, 40001, 200))
        finish(8080)
        finish(8090)
        for offset in range(1, 5):
            for slot, base in enumerate((8080, 8090)):
                port, source = base + offset, 40000 + slot + offset * 2
                feed(client_tcp(port, source, 1000 + source))
                finish(port)
        feed(client_udp(0))
    else:
        if case == "revoke":
            # A's live request is held; the guest answers both probes with
            # RST, then the revocation aborts A (plus the bare RST race).
            feed(client_tcp(8080, 40000, 100))
            for guest, peer in ((g, PROBES[g]) for g in (22, 443)):
                feed(client_tcp(peer, guest, 0, PROBE_ISN + guest + 1, RST | ACK))
            c = machine.connections[8080]
            feed(client_tcp(8080, 40000, c.client_next, c.server_start, ACK))
            feed(client_tcp(8080, 40000, c.client_next, c.server_start, ACK | PSH, REQUESTS[0]))
            feed(client_tcp(8080, 40000, c.client_next, c.server_start, RST | ACK))
            feed(client_tcp(50002, 40000, 0, PROBE_ISN + 40000 + 1, RST | ACK))
            feed(client_tcp(8080, 40000, c.client_next, 0, RST))
            feed(client_tcp(8090, 40001, 200))
        else:
            # A abandons a SYN-SENT connection. B continues after that event.
            feed(client_tcp(8084, 40000, 100))
            feed(client_tcp(8090, 40001, 200))
            feed(client_tcp(8084, 40000, 101, flags=RST | ACK))
        finish(8090)
        for offset in range(1, 5):
            port, source = 8090 + offset, 40001 + offset * 2
            feed(client_tcp(port, source, 1000 + source))
            finish(port)
    feed(client_udp(1))
    result = machine.summary()
    result.update(actions=actions, wire_frames_sent=result["counts"]["server_frames"],
                  connected=True, error=None, rejected_frame=None, passed=True)
    return result


def reset_transcript(published=True):
    """Synthetic parser fixture; no native reset or DMA evidence."""
    machine, actions = Machine("device-reset"), []
    def feed(packet):
        now = (len(actions) + 1) * 0.001
        output = machine.handle(packet, now)
        actions.append(dict(kind="input", at=now, input=packet.hex(), output=[p.hex() for p in output]))
    arp = (b"\xff" * 6 + CLIENT_MAC + b"\x08\x06" + struct.pack("!HHBBH", 1, 0x800, 6, 4, 1)
           + CLIENT_MAC + CLIENT_IP + b"\0" * 6 + PEER_IP).ljust(60, b"\0")
    feed(arp)
    feed(client_tcp(8080, 40000, 100))
    c = machine.connections[8080]
    feed(client_tcp(8080, 40000, c.client_next, c.server_start, ACK))
    feed(client_tcp(8080, 40000, c.client_next, c.server_start, ACK | PSH, REQUESTS[0]))
    if published:
        feed(client_tcp(8080, 40000, c.client_next, c.server_start, ACK | PSH, RESET_PROBE))
    result = machine.summary()
    result.update(actions=actions, wire_frames_sent=machine.counts["server_frames"])
    return result


def native_serial(case="normal"):
    service = dict.fromkeys(OFFSETS, 0)
    service.update({400: 1, 408: 10, 416: 66, 424: 65, 432: 100, 440: 22,
                    448: 2, 456: 2, 464: 10, 472: 3 * sum(map(len, REQUESTS)),
                    480: 2 * sum(map(len, RESPONSES)), 488: 4, 496: 3, 512: 8, 520: 2, 528: 1})
    app = dict.fromkeys(OFFSETS, 0)
    app.update({400: 1, 408: 2, 416: 1, 424: 1, 432: 9, 440: 1, 448: 3,
                456: len(RESPONSES[0]), 464: 2, 472: 4, 480: 1, 488: 1,
                496: 1, 504: 1, 512: 1, 520: 2, 528: 1})
    if case == "client-exit":
        service.update({440: 13, 448: 1, 456: 1, 464: 6, 472: 3 * len(REQUESTS[1]),
                        480: 2 * len(RESPONSES[1]), 488: 2, 512: 4, 520: 1, 536: 1})
    if case == "revoke":
        service.update({440: 19, 448: 1, 456: 1, 464: 6, 472: 3 * len(REQUESTS[1]) + len(REQUESTS[0]),
                        480: 2 * len(RESPONSES[1]), 488: 3, 512: 4, 520: 1, 544: 1, 552: 10})
    network = {"NET_TX_SUBMITTED": 66, "NET_TX_COMPLETED": 66, "NET_RX_COMPLETED": 65,
               "NET_RX_DELIVERED": 65, "NET_DENIED": 3 if case == "revoke" else 2, "NET_TX_TIMEOUTS": 0,
               "NET_RESETS": 1, "NET_RESET_ACKS": 1}
    if case == "device-reset":
        service = dict.fromkeys(OFFSETS, 0)
        service.update({240: 12, 400: 1, 408: 10, 416: 4, 424: 3, 432: 20, 440: 4,
                        464: 1, 472: len(REQUESTS[0]) + len(RESET_PROBE), 504: 12, 536: 52,
                        544: 12, 552: 5})
        app = dict.fromkeys(OFFSETS, 0)
        app.update({400: 1, 424: 1, 440: 1, 480: 1, 504: 53, 528: 1, 536: 9})
        network.update({"NET_TX_SUBMITTED": 5, "NET_TX_COMPLETED": 4, "NET_RX_COMPLETED": 3,
                        "NET_RX_DELIVERED": 3, "NET_FAULT_TICK": 50, "NET_FAULT_TICKET": 5,
                        "NET_FAULT_TX": 5, "NET_FAULT_RX": 3, "NET_FAULT_STATUS_BEFORE": 15,
                        "NET_FAULT_STATUS_AFTER": 0, "NET_FAULT_BME_BEFORE": 1, "NET_FAULT_BME_AFTER": 1,
                        "NET_TERMINAL_READS": 1, "NET_TERMINAL_ERROR": 12, "NET_DENIED": 3,
                        "NET_FAULT_RX_ERRORS": 0, "NET_FAULT_RX_ERROR_CODE": 0})
    lines = [*BOOT, "NETSVC:MANIFEST:PASS"]
    if case == "revoke":
        lines.append("NETSVC:REVOKE_TICKS:00000000000000000008")
    for role in (REVOKE_ROLES if case == "revoke" else ROLES):
        values = service if role == "SERVICE" else app
        if role == "APP_A" and case == "device-reset":
            values = dict(app)
            values.update({448: 2, 544: 1, 552: 49})
        if role == "APP_A" and case == "client-exit":
            values = dict.fromkeys(OFFSETS, 0)
            values.update({400: 1, 424: 1, 440: 1, 480: 1, 528: 1, 536: 1})
        if role == "APP_A" and case == "revoke":
            values = dict.fromkeys(OFFSETS, 0)
            values.update({400: 1, 424: 2, 432: 5, 440: 1, 448: 1, 480: 1, 504: 3, 528: 1, 544: 2, 552: 12})
        if role == "POLICY":
            values = dict.fromkeys(OFFSETS, 0)
            values.update({400: 1, 528: 1, 544: 1, 552: 9})
        lines.extend(f"NETSVC:{role}:{offset}:{values[offset]:020d}" for offset in OFFSETS)
    lines.extend(f"SYSTEM:{name}:{value:020d}" for name, value in network.items())
    app_a_exit = 60 if case == "device-reset" else 30
    lines.extend([f"NETSVC:APP_A_EXIT_TICKS:{app_a_exit:020d}", "NETSVC:APP_B_EXIT_TICKS:00000000000000000500", "NETSVC:ENTROPY_BYTES:00000000000000000008", "NETSVC:CLEANUP:PASS", "NETSVC:PASS", "M1:PASS"])
    return "\n".join(lines) + "\n"


class PeerChecks(unittest.TestCase):
    def test_device_reset_wire_allows_one_uncertain_publication_without_replay(self):
        for published in (False, True):
            evidence = reset_transcript(published)
            self.assertTrue(validate_transcript(evidence, case="device-reset")["complete"])
            self.assertEqual(evidence["counts"]["device_reset_probes"], int(published))
            with self.assertRaises(ProtocolError):
                validate_transcript(evidence, case="normal")
        for payload in (b"wrong", RESET_PROBE + b"x"):
            m = Machine("device-reset")
            for action in reset_transcript(False)["actions"]:
                m.handle(bytes.fromhex(action["input"]), action["at"])
            c = m.connections[8080]
            with self.assertRaises(ProtocolError):
                m.handle(client_tcp(8080, 40000, c.client_next, c.server_start, ACK | PSH, payload))
        m = Machine("device-reset")
        value = reset_transcript(True)
        for action in value["actions"]:
            m.handle(bytes.fromhex(action["input"]), action["at"])
        with self.assertRaises(ProtocolError):
            m.handle(bytes.fromhex(value["actions"][-1]["input"]), 1.0)
        m = Machine("device-reset")
        for action in value["actions"][:2]:
            m.handle(bytes.fromhex(action["input"]), action["at"])
        self.assertFalse(m.summary()["complete"])

    def test_device_reset_serial_requires_live_ticket_error_and_containment(self):
        good = native_serial("device-reset")
        self.assertEqual(sequence(good, "device-reset")["reset_tick"], 50)
        rx_first = good
        for label, old, new in (
            ("NETSVC:SERVICE:240", 12, 15), ("NETSVC:SERVICE:504", 12, 15),
            ("NETSVC:SERVICE:520", 0, 12), ("SYSTEM:NET_FAULT_RX_ERRORS", 0, 1),
            ("SYSTEM:NET_FAULT_RX_ERROR_CODE", 0, 12), ("SYSTEM:NET_DENIED", 3, 4),
        ):
            rx_first = rx_first.replace(f"{label}:{old:020d}", f"{label}:{new:020d}")
        self.assertEqual(sequence(rx_first, "device-reset")["reset_tick"], 50)
        with self.assertRaises(ValueError):
            sequence(rx_first.replace("SYSTEM:NET_DENIED:00000000000000000004",
                                      "SYSTEM:NET_DENIED:00000000000000000003"), "device-reset")
        for label, old, new in (
            ("SYSTEM:NET_FAULT_STATUS_BEFORE", 15, 0),
            ("SYSTEM:NET_FAULT_STATUS_AFTER", 0, 15),
            ("SYSTEM:NET_FAULT_BME_AFTER", 1, 0),
            ("SYSTEM:NET_TERMINAL_READS", 1, 0),
            ("SYSTEM:NET_TERMINAL_ERROR", 12, 14),
            ("SYSTEM:NET_TX_SUBMITTED", 5, 6),
            ("SYSTEM:NET_RESET_ACKS", 1, 0),
            ("NETSVC:SERVICE:552", 5, 6),
            ("NETSVC:SERVICE:240", 12, 0),
            ("NETSVC:APP_A:552", 49, 51),
            ("NETSVC:APP_A:536", 9, 0),
            ("NETSVC:APP_B:504", 53, 51),
        ):
            bad = good.replace(f"{label}:{old:020d}", f"{label}:{new:020d}")
            self.assertNotEqual(bad, good)
            with self.subTest(label=label), self.assertRaises(ValueError):
                sequence(bad, "device-reset")
        with self.assertRaises(ValueError):
            sequence(good, "normal")

    def test_revoke_case_requires_reset_probes_and_no_reuse(self):
        value = transcript("revoke")
        summary = validate_transcript(value, case="revoke")
        self.assertTrue(summary["complete"])
        self.assertEqual(summary["cases"]["8080"]["phase"], "revoked")
        self.assertEqual((summary["counts"]["probe_rsts"], summary["counts"]["revoke_rsts"]), (3, 2))
        for wrong in ("normal", "client-exit"):
            with self.assertRaises(ProtocolError):
                validate_transcript(value, case=wrong)
        # Without the abort's RST|ACK, or without a probe RST, nothing is proven.
        for drop in ("abort", "probe"):
            machine = Machine("revoke")
            machine.handle(client_tcp(8080, 40000, 100))
            if drop != "probe":
                for guest, peer in ((g, PROBES[g]) for g in (22, 443)):
                    machine.handle(client_tcp(peer, guest, 0, PROBE_ISN + guest + 1, RST | ACK))
            c = machine.connections[8080]
            machine.handle(client_tcp(8080, 40000, c.client_next, c.server_start, ACK | PSH, REQUESTS[0]))
            if drop != "abort":
                machine.handle(client_tcp(8080, 40000, c.client_next, c.server_start, RST | ACK))
            self.assertFalse(machine.summary()["complete"], drop)

    def test_revoke_case_rejects_listening_port_reuse_and_late_traffic(self):
        def held():
            machine = Machine("revoke")
            machine.handle(client_tcp(8080, 40000, 100))
            c = machine.connections[8080]
            machine.handle(client_tcp(8080, 40000, c.client_next, c.server_start, ACK | PSH, REQUESTS[0]))
            return machine, c
        attempts = {
            "probe-syn-ack": lambda m, c: m.handle(client_tcp(50000, 22, 0, PROBE_ISN + 23, SYN | ACK)),
            "probe-wrong-ack": lambda m, c: m.handle(client_tcp(50000, 22, 0, PROBE_ISN, RST | ACK)),
            "probe-twice": lambda m, c: [m.handle(client_tcp(50000, 22, 0, PROBE_ISN + 23, RST | ACK))
                                         for _ in range(2)],
            "reset-twice": lambda m, c: [m.handle(client_tcp(8080, 40000, c.client_next, c.server_start, RST | ACK))
                                         for _ in range(2)],
            "reset-wrong-seq": lambda m, c: m.handle(client_tcp(8080, 40000, c.client_next + 1, c.server_start, RST | ACK)),
            "data-after-reset": lambda m, c: [m.handle(client_tcp(8080, 40000, c.client_next, c.server_start, RST | ACK)),
                                              m.handle(client_tcp(8080, 40000, c.client_next, c.server_start, ACK | PSH, b"x"))],
            "new-syn": lambda m, c: m.handle(client_tcp(8081, 40002, 1)),
            "unsolicited-40000-probe-reply": lambda m, c: m.handle(
                client_tcp(50002, 40000, 0, PROBE_ISN + 40001, RST | ACK)),
            "udp": lambda m, c: m.handle(client_udp(0)),
            "fin-while-held": lambda m, c: m.handle(client_tcp(8080, 40000, c.client_next, c.server_start, FIN | ACK)),
        }
        for name, attempt in attempts.items():
            machine, c = held()
            with self.subTest(name=name), self.assertRaises(ProtocolError):
                attempt(machine, c)

    def test_revoke_native_serial_binds_policy_latency_and_denials(self):
        good = native_serial("revoke")
        observed = sequence(good, "revoke")
        self.assertEqual(observed["revoke_ticks"], 8)
        self.assertEqual(observed["processes"]["POLICY"]["544"], 1)
        with self.assertRaises(ValueError):
            sequence(good, "normal")
        for old, new in (
            ("NETSVC:REVOKE_TICKS:00000000000000000008\n", ""),
            ("NETSVC:APP_A:504:00000000000000000003", "NETSVC:APP_A:504:00000000000000000500"),
            ("NETSVC:APP_A:432:00000000000000000005", "NETSVC:APP_A:432:00000000000000000004"),
            ("NETSVC:APP_A:456:00000000000000000000", "NETSVC:APP_A:456:00000000000000000001"),
            ("NETSVC:POLICY:448:00000000000000000000", "NETSVC:POLICY:448:00000000000000000001"),
            ("NETSVC:SERVICE:544:00000000000000000001", "NETSVC:SERVICE:544:00000000000000000000"),
            ("SYSTEM:NET_DENIED:00000000000000000003", "SYSTEM:NET_DENIED:00000000000000000002"),
            # Causal order: service revocation after app A saw Closed, or policy send before the request.
            ("NETSVC:SERVICE:552:00000000000000000010", "NETSVC:SERVICE:552:00000000000000000013"),
            ("NETSVC:POLICY:552:00000000000000000009", "NETSVC:POLICY:552:00000000000000000007"),
        ):
            bad = good.replace(old, new)
            self.assertNotEqual(bad, good)
            with self.subTest(old=old), self.assertRaises(ValueError):
                sequence(bad, "revoke")

    def test_client_exit_case_requires_survivor_packets_and_correct_binding(self):
        value = transcript("client-exit")
        self.assertTrue(validate_transcript(value, case="client-exit")["complete"])
        self.assertEqual(value["counts"]["udp_requests"], 1)
        self.assertEqual(set(value["cases"]), {"8084", *map(str, range(8090, 8095))})
        self.assertEqual(sequence(native_serial("client-exit"), "client-exit")["processes"]["SERVICE"]["536"], 1)
        for wrong in ("normal", "unknown"):
            with self.assertRaises(ValueError):
                validate_transcript(value, case=wrong)
        with self.assertRaises(ValueError):
            sequence(native_serial("client-exit"), "normal")
        bad = native_serial("client-exit").replace("NETSVC:APP_B_EXIT_TICKS:00000000000000000500", "NETSVC:APP_B_EXIT_TICKS:00000000000000000029")
        with self.assertRaises(ValueError):
            sequence(bad, "client-exit")
        machine = Machine("client-exit")
        with self.assertRaises(ProtocolError):
            machine.handle(client_tcp(8080, 40000, 1))
        with self.assertRaises(ProtocolError):
            machine.handle(client_tcp(8091, 40002, 1))

    def test_raw_binary_pipes_are_verbatim_and_utf8_failure_is_terminal(self):
        raw = b"\xff\n" + native_serial().encode()
        script = "import sys; sys.stdout.buffer.write(bytes.fromhex(sys.argv[1])); sys.stderr.buffer.write(b'err\\r\\n')"
        rc, stdout, stderr, timed_out, overflow = execute_native([sys.executable, "-c", script, raw.hex()], os.environ, timeout=2)
        self.assertEqual((rc, stdout, stderr, timed_out, overflow), (0, raw, b"err\r\n", False, False))
        base = ROOT / "build"
        base.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=base) as name:
            directory = Path(name)
            with self.assertRaises(UnicodeDecodeError):
                record_streams(directory, stdout, stderr)
            self.assertEqual((directory / "serial.txt").read_bytes(), raw)
            self.assertEqual((directory / "stderr.txt").read_bytes(), b"err\r\n")
            text = native_serial().encode()
            self.assertEqual(record_streams(directory, text, b"err\r\n"), (text.decode(), "err\r\n"))
            with self.assertRaises(UnicodeDecodeError):
                record_streams(directory, text, b"\xff")

    def test_process_output_and_lifetime_have_hard_bounds(self):
        rc, stdout, stderr, timed_out, overflow = execute_native(
            [sys.executable, "-c", "import sys;sys.stdout.buffer.write(b'x'*200000);sys.stdout.flush()"], os.environ, timeout=2, limit=100)
        self.assertLessEqual(len(stdout), 100)
        self.assertTrue(overflow)
        self.assertFalse(timed_out)
        rc, stdout, stderr, timed_out, overflow = execute_native(
            [sys.executable, "-c", "import time;time.sleep(5)"], os.environ, timeout=0.05)
        self.assertTrue(timed_out)
        self.assertNotEqual(rc, 0)
        self.assertFalse(overflow)

    def test_full_two_owner_wire_transcript_replays(self):
        value = transcript()
        summary = validate_transcript(value)
        self.assertTrue(summary["complete"])
        self.assertEqual(summary["counts"]["udp_requests"], 2)
        for name in ("udp_corrupt_sent", "udp_wrong_scope_sent", "udp_echo_sent", "udp_overflow_sent"):
            self.assertEqual(summary["counts"][name], 2)
        self.assertEqual(summary["counts"]["client_fins_acked"], 4)
        self.assertNotEqual(summary["request_sha256"][0], summary["request_sha256"][1])

    def test_fragment_fixtures_have_valid_ipv4_headers_and_real_offsets(self):
        machine = Machine()
        for kind, flags, size in (("first", 0x2000, 16), ("tail", 2, 8 + len(UDP_PAYLOADS[0]) - 16)):
            packet = machine.udp_packet(0, "udp-fragment-" + kind, UDP_PAYLOADS[0], fragment=kind)
            ip = packet[14:]
            self.assertEqual(checksum(ip[:20]), 0)
            self.assertEqual(struct.unpack_from("!H", ip, 6)[0], flags)
            self.assertEqual(struct.unpack_from("!H", ip, 2)[0], 20 + size)
            self.assertEqual(ip[9], 17)
            self.assertEqual(ip[12:20], PEER_IP + CLIENT_IP)
        machine = Machine()
        syn = client_tcp(8080, 40000, 100)
        self.assertEqual(len(machine.handle(syn)), 3)
        self.assertEqual(len(machine.handle(syn)), 1)
        self.assertEqual(machine.counts["udp_fragment_first_sent"], 1)
        self.assertEqual(machine.counts["udp_fragment_tail_sent"], 1)
        for case, owners in (("normal", 2), ("client-exit", 1)):
            summary = validate_transcript(transcript(case), case=case)
            for kind in ("first", "tail"):
                self.assertEqual(summary["counts"]["udp_fragment_" + kind + "_sent"], owners)

    def test_native_serial_requires_fragment_rejections_not_only_old_udp_cases(self):
        for case, old_count in (("normal", 4), ("client-exit", 2)):
            original = native_serial(case)
            lines = [
                f"NETSVC:SERVICE:512:{old_count:020d}" if line.startswith("NETSVC:SERVICE:512:") else line
                for line in original.splitlines()
            ]
            with self.subTest(case=case), self.assertRaisesRegex(ValueError, "fragment"):
                sequence("\n".join(lines) + "\n", case)

    def test_native_serial_requires_actual_terminal_receive_after_timeout(self):
        for case in ("normal", "client-exit"):
            original = native_serial(case)
            for role in (("APP_A", "APP_B") if case == "normal" else ("APP_B",)):
                bad = original.replace(f"NETSVC:{role}:520:00000000000000000002",
                                       f"NETSVC:{role}:520:00000000000000000000")
                with self.subTest(case=case, role=role), self.assertRaises(ValueError):
                    sequence(bad, case)

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
