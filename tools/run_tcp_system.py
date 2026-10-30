#!/usr/bin/env python3
"""Native TCP through the own kernel, alongside services and cold disk recovery."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
from pathlib import Path
import re
import sys

from common import ROOT, source_hash
from run_net_system import store_report
from run_system import MAX_TIMEOUT, TCP_FEATURE as FEATURE, run_system
from tcp_transcript import validate_transcript

_spec = importlib.util.spec_from_file_location("nieuw_os_tcp_peer", ROOT / "sdk/examples/tcp-probe/peer.py")
peer_module = importlib.util.module_from_spec(_spec)
sys.modules[_spec.name] = peer_module
_spec.loader.exec_module(peer_module)

EXACT = {"TCP_HTTP": 2, "TCP_REFUSED": 1, "TCP_RESET": 1, "TCP_RETRANSMIT": 1,
         "TCP_TIMEOUT": 1, "TCP_BODY_BYTES": 40, "TCP_STALE_CAP": 1,
         "NET_DENIED": 1, "NET_TX_TIMEOUTS": 0, "NET_RESETS": 1, "NET_RESET_ACKS": 1}
PEER_EXACT = {"first_syn_dropped": 1, "corrupt_data_sent": 1, "valid_data_sent": 2,
              "valid_data_acks": 2, "fins_sent": 2, "client_fins_acked": 2,
              "http_requests": 3, "resets_after_get": 1, "refusals": 1}
RESPONSE_SHA256 = "d128fa1924be70f9e1f42e2a466e1e94123182b23b11110d273ced90cefda90b"
CORRUPT_SHA256 = hashlib.sha256(peer_module.CORRUPT_RESPONSE).hexdigest()
REQUEST_SHA256 = hashlib.sha256(peer_module.REQUEST).hexdigest()


def integer(values: dict, name: str, low=0, high=(1 << 64) - 1) -> int:
    value = values.get(name)
    if type(value) is not int or not low <= value <= high:
        raise ValueError(f"missing or out-of-bound TCP evidence {name}")
    return value


def evaluate_tcp(report: dict, peer: dict) -> None:
    if report.get("passed") is not True or report.get("kernel_feature") not in FEATURE.values():
        raise ValueError("native joint-system/TCP profile did not pass")
    digest = report.get("manifest", {}).get("system", {}).get("net_probe", {}).get("sha256")
    if not isinstance(digest, str) or re.fullmatch("[0-9a-f]{64}", digest) is None:
        raise ValueError("missing inspected native TCP ELF hash")
    if (peer.get("passed") is not True or peer.get("complete") is not True
            or peer.get("connected") is not True or peer.get("error") is not None
            or peer.get("rejected_frame") is not None):
        raise ValueError(f"TCP test server failed: {peer.get('error')}")
    values, counts = report.get("evidence", {}), peer.get("counts", {})
    for name, expected in EXACT.items():
        if integer(values, name) != expected:
            raise ValueError(f"wrong native TCP evidence {name}")
    for name, expected in PEER_EXACT.items():
        if integer(counts, name) != expected:
            raise ValueError(f"wrong causal TCP server evidence {name}")
    tx = integer(values, "TCP_TX_COMPLETED", 1, peer_module.MAX_FRAMES)
    rx = integer(values, "TCP_RX_DELIVERED", 1, peer_module.MAX_FRAMES)
    if (tx != integer(values, "NET_TX_SUBMITTED") or tx != integer(values, "NET_TX_COMPLETED")
            or tx != integer(counts, "client_frames") or rx != integer(values, "NET_RX_DELIVERED")
            or rx != integer(values, "NET_RX_COMPLETED") or rx != integer(counts, "server_frames")
            or rx != integer(peer, "wire_frames_sent")):
        raise ValueError("TCP app, broker and actual server frame counts disagree")
    start = integer(values, "TCP_START_TICKS")
    end = integer(values, "TCP_END_TICKS", start + 300)
    if (end - start > 20_000 or integer(values, "TCP_TIMEOUT_TICKS", 300, 500) > end - start
            or integer(values, "TCP_CLOCK_READS", 5) == 0):
        raise ValueError("TCP lacks actual bounded monotonic timeout evidence")
    for name in ("retry_syns", "blackhole_syns", "arp_requests"):
        integer(counts, name, 1, peer_module.MAX_FRAMES)
    if peer.get("response_sha256") != RESPONSE_SHA256 or peer.get("response_body_bytes") != 20:
        raise ValueError("different HTTP response fixture")
    if peer.get("corrupt_response_sha256") != CORRUPT_SHA256 or CORRUPT_SHA256 == RESPONSE_SHA256:
        raise ValueError("missing distinct corrupt-checksum response fixture")
    cases = peer.get("cases", {})
    if set(cases) != {str(port) for port in peer_module.PORTS}:
        raise ValueError("missing TCP scenarios")
    for port in ("8080", "8083"):
        case = cases[port]
        if (case.get("closed") is not True or case.get("data_acked") is not True
                or case.get("fin_sent") is not True or case.get("phase") != "closed"
                or case.get("request_bytes") != len(peer_module.REQUEST)
                or case.get("request_sha256") != REQUEST_SHA256):
            raise ValueError("HTTP response was not acknowledged and cleanly closed")
    if (cases["8082"].get("request_sha256") != REQUEST_SHA256
            or cases["8082"].get("request_bytes") != len(peer_module.REQUEST)):
        raise ValueError("reset did not follow the expected actual HTTP request")
    if any(cases[port].get("phase") != phase for port, phase in
           (("8081", "refused"), ("8082", "reset"), ("8084", "blackhole"))):
        raise ValueError("wrong refused/reset/blackhole scenario")
    events = peer.get("events", [])
    if not 1 <= len(events) <= peer_module.MAX_EVENTS:
        raise ValueError("missing or unbounded actual packet transcript")
    directions = {"rx": 0, "tx": 0}
    for index, event in enumerate(events, 1):
        if (type(event.get("sequence")) is not int or event["sequence"] != index
                or event.get("direction") not in directions
                or not isinstance(event.get("frame_sha256"), str)
                or re.fullmatch("[0-9a-f]{64}", event["frame_sha256"]) is None):
            raise ValueError("invalid or duplicated TCP packet transcript")
        integer(event, "length", 14, peer_module.MAX_FRAME)
        directions[event["direction"]] += 1
    if directions != {"rx": tx, "tx": rx}:
        raise ValueError("packet transcript does not match server/broker counts")
    validate_transcript(peer, request_length=len(peer_module.REQUEST), response_length=len(peer_module.RESPONSE))


def run_tcp_system(case: str, volume_dir: Path | None = None, *, timeout=MAX_TIMEOUT):
    if case not in FEATURE or not 0 < timeout <= MAX_TIMEOUT:
        raise ValueError("unknown TCP case or unbounded timeout")
    report = None
    try:
        with peer_module.Peer(timeout=timeout) as peer:
            report, volume_dir = run_system(case, volume_dir, timeout=timeout,
                                            kernel_feature=FEATURE[case], network_port=peer.port)
        evidence = peer.finish()
    except Exception as error:
        if report is None:
            raise
        report.update(passed=False, network_gate=False, tcp_gate=False,
                      network_error=f"{type(error).__name__}: {error}")
        store_report(report, volume_dir)
        return report, volume_dir
    report.update(network_peer=evidence, network_gate=False, tcp_gate=False)
    try:
        evaluate_tcp(report, evidence)
        if (report.get("case") != case or report.get("kernel_feature") != FEATURE[case]
                or source_hash() != report["source_sha256"]):
            raise ValueError("TCP source, case or profile changed")
        report.update(network_gate=True, tcp_gate=True)
    except Exception as error:
        report.update(passed=False, network_error=f"{type(error).__name__}: {error}")
    store_report(report, volume_dir, evidence)
    print(f"NATIVE TCP {case}: passed={report['passed']} {report['run_directory']}")
    if "network_error" in report:
        print(report["network_error"])
    return report, volume_dir


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=tuple(FEATURE), required=True)
    parser.add_argument("--volume", type=Path)
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    options = parser.parse_args()
    raise SystemExit(0 if run_tcp_system(options.case, options.volume, timeout=options.timeout)[0]["passed"] else 1)
