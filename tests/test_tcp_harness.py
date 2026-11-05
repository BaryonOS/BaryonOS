"""Host-only counterexamples for the TCP proof boundary; never native evidence."""
import copy
import importlib.util
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import run_tcp_system as runner
from run_system import check_parent

sys.path.insert(0, str(ROOT / "sdk/examples/tcp-probe"))
_spec = importlib.util.spec_from_file_location("nieuw_os_tcp_fixture_tests", ROOT / "sdk/examples/tcp-probe/test_peer.py")
fixtures = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(fixtures)


def fixture():
    machine = runner.peer_module.PeerMachine()
    machine.handle(fixtures.ARP)
    fixtures.first_three(machine)
    base = fixtures.connect(machine, 8083)
    fixtures.finish_http(machine, 8083, base, fixtures.request(machine, 8083, base))
    fixtures.connect(machine, 8084)
    fixtures.connect(machine, 8085)  # baseline: the opt-in case must be actually refused
    peer = machine.summary()
    peer.update(passed=True, connected=True, error=None, rejected_frame=None,
                wire_frames_sent=peer["counts"]["server_frames"])
    tx, rx = peer["counts"]["client_frames"], peer["counts"]["server_frames"]
    values = dict(runner.EXACT, TCP_TX_COMPLETED=tx, TCP_RX_DELIVERED=rx,
                  NET_TX_SUBMITTED=tx, NET_TX_COMPLETED=tx, NET_RX_DELIVERED=rx,
                  NET_RX_COMPLETED=rx, TCP_START_TICKS=10, TCP_END_TICKS=810,
                  TCP_TIMEOUT_TICKS=300, TCP_CLOCK_READS=2000)
    report = dict(passed=True, kernel_feature="native-system-tcp", evidence=values,
                  manifest=dict(system=dict(net_probe=dict(sha256="a" * 64))))
    return report, peer


class TcpGateTests(unittest.TestCase):
    def test_complete_guest_and_real_server_fixture_match(self):
        report, peer = fixture()
        runner.evaluate_tcp(report, peer)

    def test_every_exact_native_or_server_counter_is_required(self):
        report, peer = fixture()
        for name in runner.EXACT:
            changed = copy.deepcopy(report)
            changed["evidence"][name] += 1
            with self.subTest(name=name), self.assertRaises(ValueError):
                runner.evaluate_tcp(changed, peer)
        for name in runner.PEER_EXACT:
            changed = copy.deepcopy(peer)
            changed["counts"][name] += 1
            with self.subTest(name=name), self.assertRaises(ValueError):
                runner.evaluate_tcp(report, changed)

    def test_missing_dynamic_broker_counts_time_or_boolean_counters_fail(self):
        report, peer = fixture()
        for name, value in (("TCP_HTTP", True), ("NET_RX_COMPLETED", 0),
                            ("TCP_TX_COMPLETED", 0), ("TCP_TIMEOUT_TICKS", 299),
                            ("TCP_START_TICKS", 1000), ("TCP_CLOCK_READS", 0)):
            changed = copy.deepcopy(report)
            changed["evidence"][name] = value
            with self.subTest(name=name), self.assertRaises(ValueError):
                runner.evaluate_tcp(changed, peer)

    def test_partial_shutdown_wrong_profile_and_missing_native_elf_fail(self):
        report, peer = fixture()
        for key, value in (("passed", False), ("complete", False), ("connected", False),
                           ("error", "failure"), ("rejected_frame", dict(length=60))):
            with self.subTest(key=key), self.assertRaises(ValueError):
                runner.evaluate_tcp(report, {**peer, key: value})
        for change in (dict(passed=False), dict(kernel_feature="native-system-net"), dict(manifest={})):
            with self.assertRaises(ValueError):
                runner.evaluate_tcp({**report, **change}, peer)

    def test_http_without_closed_fin_or_ack_and_forged_transcript_fail(self):
        report, peer = fixture()
        for problem in ("fin", "ack", "close", "events", "duplicate", "hash", "wire", "response", "corrupt", "request"):
            changed = copy.deepcopy(peer)
            if problem in ("fin", "ack", "close"):
                field = {"fin": "fin_sent", "ack": "data_acked", "close": "closed"}[problem]
                changed["cases"]["8080"][field] = False
            elif problem == "events":
                changed["events"] = []
            elif problem == "duplicate":
                changed["events"].append(changed["events"][-1])
            elif problem == "hash":
                changed["events"][0]["frame_sha256"] = "z" * 64
            elif problem == "wire":
                changed["wire_frames_sent"] -= 1
            elif problem == "corrupt":
                changed["corrupt_response_sha256"] = runner.RESPONSE_SHA256
            elif problem == "request":
                changed["cases"]["8080"]["request_sha256"] = "f" * 64
            else:
                changed["response_sha256"] = "f" * 64
            with self.subTest(problem=problem), self.assertRaises(ValueError):
                runner.evaluate_tcp(report, changed)

    def test_tcp_cold_chain_cannot_use_other_profile_or_partial_peer(self):
        previous = dict(case="normal", passed=True, kernel_feature="native-system-tcp",
                        source_sha256="source", after_sha256="disk", network_gate=True, tcp_gate=True)
        check_parent(previous, "source", "disk", network=True, tcp=True)
        for change in (dict(kernel_feature="native-system-net"), dict(network_gate=False), dict(tcp_gate=False),
                       dict(passed=False), dict(source_sha256="different"), dict(after_sha256="different")):
            with self.assertRaises(ValueError):
                check_parent({**previous, **change}, "source", "disk", network=True, tcp=True)
        with self.assertRaises(ValueError):
            check_parent(previous, "source", "disk", network=False, tcp=True)
        with self.assertRaises(ValueError):
            check_parent(previous, "source", "disk", network=True)


if __name__ == "__main__":
    unittest.main()
