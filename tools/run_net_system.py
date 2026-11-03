#!/usr/bin/env python3
"""Real native ARP/UDP/DNS plus the existing joint-system/cold-image gate."""
from __future__ import annotations

import argparse
from pathlib import Path
import stat

from common import BUILD, save_json, source_hash
from net_test_peer import DNS_NAME, TIMEOUT_NAME, EthernetPeer
from run_system import MAX_TIMEOUT, run_system

FEATURE = {"normal": "native-system-net", "recover": "test-system-net-recover"}
NET_EXACT = {
    "NET_ARP_REPLIES": 1, "NET_UDP_ECHOES": 1, "NET_DNS_ANSWERS": 1,
    "NET_DENIED": 7, "NET_TX_COMPLETIONS": 8, "NET_RX_FRAMES": 7,
    "NET_ARP_REJECTS": 2, "NET_DNS_REJECTS": 2, "NET_RX_TIMEOUTS": 1,
    "NET_CACHE_INTACT": 1, "NET_STALE_CAP": 1,
    "NET_APP_DENIED": 7, "NET_TX_SUBMITTED": 8, "NET_TX_COMPLETED": 8,
    "NET_RX_COMPLETED": 7, "NET_RX_DELIVERED": 7, "NET_TX_TIMEOUTS": 0,
    "NET_RESETS": 1, "NET_RESET_ACKS": 1,
}
PEER_EXACT = {
    "accepted_frames": 8, "arp_requests": 3, "arp_bad_sender_sent": 1,
    "arp_wrong_target_sent": 1, "arp_valid_sent": 1, "udp_echo_requests": 1,
    "udp_echo_replies": 1, "dns_requests": 4, "dns_wrong_id_sent": 1,
    "dns_bad_name_sent": 1, "dns_answers_sent": 1, "dns_timeouts": 1,
    "response_frames": 7,
}
EVENTS = ("arp_bad_sender_sent", "arp_wrong_target_sent", "arp_valid_sent",
          "udp_echo_replies", "dns_wrong_id_sent", "dns_bad_name_sent",
          "dns_answers_sent", "dns_timeouts")


def evaluate_network(report: dict, peer: dict) -> None:
    if report.get("passed") is not True:
        raise ValueError("the joint-system/disk gate failed")
    if report.get("kernel_feature") not in FEATURE.values():
        raise ValueError("network evidence requires the native network kernel profile")
    digest = report.get("manifest", {}).get("system", {}).get("net_probe", {}).get("sha256")
    if not isinstance(digest, str) or len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest):
        raise ValueError("missing inspected native network ELF hash")
    if peer.get("connected") is not True or peer.get("error") is not None:
        raise ValueError(f"testpeer connection/protocol failed: {peer.get('error')}")
    for name, expected in NET_EXACT.items():
        actual = report.get("evidence", {}).get(name)
        if type(actual) is not int or actual != expected:
            raise ValueError(f"{name}: expected {expected}, found {actual}")
    for name, expected in PEER_EXACT.items():
        actual = peer.get("counts", {}).get(name)
        if type(actual) is not int or actual != expected:
            raise ValueError(f"peer {name}: expected {expected}, found {actual}")
    events = peer.get("events", [])
    if ([event.get("event") for event in events] != list(EVENTS) or
            [event.get("sequence") for event in events] != list(range(1, 9))):
        raise ValueError("wrong/missing/duplicate packet transcript")
    for event in events:
        digests = [event.get("request_sha256"), *event.get("response_sha256", [])]
        if (len(event.get("response_sha256", [])) != (0 if event["event"] == "dns_timeouts" else 1)
                or any(not isinstance(digest, str) or len(digest) != 64 or
                       any(char not in "0123456789abcdef" for char in digest) for digest in digests)):
            raise ValueError("missing or malformed packet hashes")
    if ([event.get("question_name") for event in events[4:]] != [DNS_NAME] * 3 + [TIMEOUT_NAME]
            or [event.get("transaction_id") for event in events[4:]] != [0x1234] * 3 + [0x5678]):
        raise ValueError("peer observed different DNS questions or transaction IDs")


def store_report(report: dict, volume_dir: Path, evidence: dict | None = None) -> None:
    run_dir = Path(report["run_directory"]).resolve(strict=True)
    run_dir.relative_to((BUILD / "run").resolve(strict=True))
    volume_dir = volume_dir.resolve(strict=True)
    volume_dir.relative_to((BUILD / "run").resolve(strict=True))
    outputs = [run_dir / "result.json", volume_dir / "last-system-stage.json"]
    if evidence is not None:
        outputs.append(run_dir / "network-peer.json")
    for path in outputs:
        try:
            details = path.lstat()
        except FileNotFoundError:
            continue
        if not stat.S_ISREG(details.st_mode) or details.st_nlink != 1:
            raise ValueError("network report must be a regular single-link file")
    if evidence is not None:
        save_json(run_dir / "network-peer.json", evidence)
    save_json(run_dir / "result.json", report)
    # A failed network normal run must not authorize the cold successor.
    save_json(volume_dir / "last-system-stage.json", report)


def run_net_system(case: str, volume_dir: Path | None = None, *, timeout: int = MAX_TIMEOUT):
    if case not in FEATURE or not 0 < timeout <= MAX_TIMEOUT:
        raise ValueError("unknown case or unbounded timeout")
    report = None
    try:
        with EthernetPeer(timeout=timeout) as peer:
            report, volume_dir = run_system(case, volume_dir, timeout=timeout,
                                            kernel_feature=FEATURE[case], network_port=peer.port)
    except Exception as error:
        if report is None:
            raise
        report["passed"] = False
        report["network_gate"] = False
        report["network_error"] = f"{type(error).__name__}: {error}"
        store_report(report, volume_dir)
        return report, volume_dir
    evidence = peer.evidence()
    report["network_peer"] = evidence
    report["network_gate"] = False
    try:
        evaluate_network(report, evidence)
        if report.get("case") != case or report.get("kernel_feature") != FEATURE[case]:
            raise ValueError("case/profile differs from the requested network run")
        if source_hash() != report["source_sha256"]:
            raise ValueError("source changed during network evidence collection")
        report["network_gate"] = True
    except Exception as error:
        report["passed"] = False
        report["network_error"] = f"{type(error).__name__}: {error}"
    store_report(report, volume_dir, evidence)
    print(f"NATIVE NET {case}: passed={report['passed']} {report['run_directory']}")
    if "network_error" in report:
        print(report["network_error"])
    return report, volume_dir


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=tuple(FEATURE), required=True)
    parser.add_argument("--volume", type=Path)
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    options = parser.parse_args()
    raise SystemExit(0 if run_net_system(options.case, options.volume, timeout=options.timeout)[0]["passed"] else 1)
