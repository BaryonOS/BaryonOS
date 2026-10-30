"""Semantic anti-false-pass tests; all traffic here is a host fixture."""
from __future__ import annotations

import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from tcp_transcript import validate_transcript
from test_tcp_harness import fixture, fixtures, runner


def validate(peer):
    validate_transcript(peer, request_length=len(fixtures.peer.REQUEST),
                        response_length=len(fixtures.peer.RESPONSE))


class TranscriptTests(unittest.TestCase):
    def setUp(self):
        _, self.peer = fixture()

    def changed(self, change):
        peer = copy.deepcopy(self.peer)
        change(peer)
        with self.assertRaises(ValueError):
            validate(peer)

    def test_complete_actual_peer_model_and_wrapped_sequences(self):
        validate(self.peer)
        # The independently built wire fixtures wrap the client sequence at 2^32.
        self.assertTrue(any(e.get("seq") == 57 for e in self.peer["events"]))

    def test_partial_request_duplicate_fin_and_blackhole_retry_remain_valid(self):
        machine = runner.peer_module.PeerMachine()
        machine.handle(fixtures.ARP)
        base = fixtures.connect(machine, 8080)
        request = fixtures.peer.REQUEST
        machine.handle(fixtures.client(seq=0xfffffff1, ack=base, flags=fixtures.peer.ACK,
                                       payload=request[:10]), 2)
        packets = machine.handle(fixtures.client(seq=0xfffffff1 + 10, ack=base,
                                                 flags=fixtures.peer.ACK, payload=request[10:]), 2)
        fixtures.finish_http(machine, 8080, base, packets)
        machine.handle(fixtures.client(seq=(0xfffffff1 + len(request)) & fixtures.peer.MASK,
                                       ack=base + len(fixtures.peer.RESPONSE) + 1,
                                       flags=fixtures.peer.ACK | fixtures.peer.FIN))
        fixtures.connect(machine, 8081)
        base = fixtures.connect(machine, 8082)
        fixtures.request(machine, 8082, base)
        base = fixtures.connect(machine, 8083)
        fixtures.finish_http(machine, 8083, base, fixtures.request(machine, 8083, base))
        fixtures.connect(machine, 8084)
        machine.handle(fixtures.client(8084, options=b"\x02\x04\x05\xb4"))
        machine.handle(fixtures.client(8084, seq=0xfffffff1, flags=fixtures.peer.RST))
        validate(machine.summary())

    def test_original_forged_labels_and_missing_tcp_fields_are_rejected(self):
        def forge(peer):
            for event in peer["events"]:
                event["event"] = "forged-no-protocol"
                for name in ("port", "seq", "ack", "flags", "payload_length"):
                    event.pop(name, None)
        self.changed(forge)
        for name in ("port", "seq", "ack", "flags", "payload_length"):
            with self.subTest(name=name):
                self.changed(lambda p: p["events"][2].pop(name))

    def test_unknown_or_wrong_direction_labels_and_bad_numeric_types(self):
        for index, field, value in ((2, "event", []), (2, "event", "unknown"),
                                    (2, "event", "syn-ack"), (3, "direction", "rx"),
                                    (2, "seq", True), (2, "seq", 1 << 32),
                                    (2, "payload_length", -1), (2, "port", 9000),
                                    (2, "length", 53), (2, "sequence", True)):
            with self.subTest(index=index, field=field, value=value):
                self.changed(lambda p: p["events"][index].__setitem__(field, value))

    def test_server_ack_sequence_flags_payload_and_length_are_causal(self):
        for label, field in (("syn-ack", "ack"), ("syn-ack", "seq"),
                             ("http-data", "seq"), ("http-data", "ack"),
                             ("http-data", "payload_length"), ("server-fin", "seq"),
                             ("server-fin", "flags"), ("client-fin-ack", "ack")):
            def mutate(peer):
                event = next(e for e in peer["events"] if e["event"] == label)
                event[field] += 1
            with self.subTest(label=label, field=field):
                self.changed(mutate)

    def test_changed_syn_retry_identity_or_answered_first_syn_is_rejected(self):
        def retry(peer):
            events = [e for e in peer["events"] if e.get("port") == 8083 and e.get("flags") == 2]
            events[1]["seq"] += 1
        self.changed(retry)

        def first_answered(peer):
            events = peer["events"]
            syns = [i for i, e in enumerate(events) if e.get("port") == 8083 and e.get("flags") == 2]
            # Remove the first SYN and renumber: the surviving sole SYN must
            # still be dropped, so its following SYN/ACK cannot be accepted.
            events.pop(syns[0])
            for index, event in enumerate(events, 1):
                event["sequence"] = index
        self.changed(first_answered)

    def test_corrupt_data_ack_before_valid_data_and_equal_frame_hashes_fail(self):
        def premature(peer):
            events = peer["events"]
            valid = next(i for i, e in enumerate(events) if e["event"] == "http-data")
            advanced_ack = events.pop(valid + 1)
            events.insert(valid, advanced_ack)
            for index, event in enumerate(events, 1):
                event["sequence"] = index
        self.changed(premature)

        def same_hash(peer):
            bad = next(e for e in peer["events"] if e["event"] == "bad-tcp-checksum")
            good = next(e for e in peer["events"] if e["event"] == "http-data")
            good["frame_sha256"] = bad["frame_sha256"]
        self.changed(same_hash)

    def test_missing_fin_or_its_reply_cannot_be_replaced_by_summary_flags(self):
        for label in ("server-fin", "client-fin-ack"):
            def remove(peer):
                peer["events"] = [e for e in peer["events"] if e["event"] != label]
                for index, event in enumerate(peer["events"], 1):
                    event["sequence"] = index
            with self.subTest(label=label):
                self.changed(remove)

    def test_every_packet_derived_counter_and_connection_summary_is_required(self):
        for name in self.peer["counts"]:
            with self.subTest(counter=name):
                self.changed(lambda p: p["counts"].__setitem__(name, p["counts"][name] + 1))
        for field, value in (("closed", False), ("data_acked", False), ("fin_sent", False),
                             ("phase", "request"), ("syns", 2), ("request_bytes", 71),
                             ("request_sha256", "a" * 64)):
            with self.subTest(field=field):
                self.changed(lambda p: p["cases"]["8080"].__setitem__(field, value))
        self.changed(lambda p: p["counts"].__setitem__("first_syn_dropped", True))

    def test_unexpected_server_traffic_to_blackhole_and_fixture_substitution_fail(self):
        def blackhole(peer):
            event = copy.deepcopy(peer["events"][3])
            event.update(sequence=len(peer["events"]) + 1, port=8084)
            peer["events"].append(event)
        self.changed(blackhole)
        for field in ("response_sha256", "corrupt_response_sha256"):
            with self.subTest(field=field):
                self.changed(lambda p: p.__setitem__(field, "a" * 64))
        with self.assertRaises(ValueError):
            validate_transcript(self.peer, request_length=True, response_length=78)
        self.changed(lambda p: p.__setitem__("events", []))


if __name__ == "__main__":
    unittest.main()
