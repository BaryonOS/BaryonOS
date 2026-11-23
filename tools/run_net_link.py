#!/usr/bin/env python3
"""H04-T3: native virtio-net link loss and reconnect in the joint system profile.

The independent kernel broker samples the NIC link once per PIT tick, numbers
transitions with its own epochs, refuses TX/RX (-23) while the link is down
and destroys undelivered RX bytes at the down edge. The ring-3 net probe reads
only (epoch, up) through the test-only syscall 0x74. The link is changed only
through a private QMP unix socket inside the run directory, with exactly
`qmp_capabilities` and `set_link net0`; each change reacts to a kernel serial
marker. The synthetic peer answers packets and is never the OS client.

QEMU/TCG evidence only: no physical NIC. QEMU 8.2 has no 802.11 or Bluetooth
device model, so Wi-Fi and Bluetooth are outside this harness.
"""
from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
import stat
import time

from common import BUILD, save_json, source_hash
from net_test_peer import EthernetPeer
from run_net_system import EVENTS as BASE_EVENTS, PEER_EXACT as BASE_PEER
from run_system import MAX_TIMEOUT, run_system

# case -> (kernel feature, NIC offers VIRTIO_NET_F_STATUS, QMP link control)
CASES = {
    "normal": ("test-system-net-link", True, True),
    "nostatus": ("test-system-net-link", False, False),
    "storm": ("test-system-net-link-storm", True, True),
}
NETDEV = "net0"
ALLOWED_QMP = ("qmp_capabilities", "set_link")
# The base schedule delivers 7 frames; the 8th RX completion is the reply to
# the pre-down echo, held undelivered in the kernel when the link drops.
PREDOWN_RX = 8
STORM_TOGGLES = 9  # 8 budgeted transitions after the first sample, then exhaustion
LATER_TOGGLES = (True, False, True)  # after the latch: must produce no event
MIN_HOLD_GAP_S = 0.5
NUMBER = re.compile(r"[0-9]{20}")


def marker(label: str, value: int) -> str:
    return f"{label}{value:020d}"


class LinkControl:
    """Serial-marker driven QMP link controller (the run_system qmp_hook).

    It connects lazily to the run directory's private socket, sends only
    allowlisted commands and records every request/response and action.
    """

    def __init__(self, case: str, deadline: float, connect=None):
        if case not in ("normal", "storm"):
            raise ValueError("only the normal and storm cases toggle the link")
        self.case = case
        self.deadline = deadline
        self.connect = connect or self._connect
        self.qmp = None
        self.directory = None
        self.phase = "wait-held"
        self.actions: list[dict] = []
        self.later_done_at: float | None = None
        self.brokers_at: float | None = None
        self.exhausted_seen = False
        self.started = time.monotonic()
        self.timeline: list[tuple[float, str]] = []

    @staticmethod
    def _connect(run_dir: Path, process, deadline: float, client=None):
        from run_graphics import Qmp  # existing bounded QMP client
        Qmp = client or Qmp
        path = run_dir / "qmp.sock"
        directory = os.open(run_dir, os.O_PATH | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            start = time.monotonic()
            while not path.exists() and time.monotonic() - start < 5:
                time.sleep(0.02)
            details = path.lstat()
            if not stat.S_ISSOCK(details.st_mode) or details.st_uid != os.getuid():
                raise ValueError("QMP endpoint is not this run's private socket")
            # An absolute AF_UNIX path can exceed 107 bytes; resolve through
            # the already-open run directory instead.
            return Qmp(f"/proc/self/fd/{directory}/qmp.sock", deadline, process), directory
        except Exception:
            os.close(directory)
            raise

    def command(self, name: str, arguments: dict | None = None):
        if name not in ALLOWED_QMP:
            raise ValueError(f"QMP command outside the allowlist: {name}")
        return self.qmp.command(name, arguments)

    def toggle(self, up: bool, trigger: str, process, run_dir: Path) -> None:
        if self.qmp is None:
            self.qmp, self.directory = self.connect(run_dir, process, self.deadline)
        if len(self.actions) >= STORM_TOGGLES + len(LATER_TOGGLES):
            raise RuntimeError("link toggle budget of the harness exceeded")
        reply = self.command("set_link", {"name": NETDEV, "up": up})
        if reply != {}:
            raise RuntimeError(f"unexpected set_link reply {reply!r}")
        now = time.monotonic()
        self.actions.append(dict(up=up, trigger=trigger, phase=self.phase,
                                 completed_s=round(now - self.started, 3)))
        self.timeline.append((round(now - self.started, 3), f"QMP:set_link:{NETDEV}:{'on' if up else 'off'}"))

    def __call__(self, line: str, process, run_dir: Path) -> None:
        if line.startswith("NETLINK:") or line == "SYSTEM:BROKERS:PASS":
            if len(self.timeline) < 256:
                self.timeline.append((round(time.monotonic() - self.started, 3), line))
        if line == "SYSTEM:BROKERS:PASS":
            self.brokers_at = time.monotonic()
        if self.phase == "wait-held":
            if line == marker("NETLINK:RX_HELD:", PREDOWN_RX):
                self.toggle(False, line, process, run_dir)
                self.phase = "wait-refused" if self.case == "normal" else "storm"
        elif self.phase == "wait-refused":
            if line == marker("NETLINK:DOWN_REFUSED:", 2):
                self.toggle(True, line, process, run_dir)
                self.phase = "done"
        elif self.phase == "storm":
            if line.startswith("NETLINK:EXHAUSTED:"):
                self.exhausted_seen = True
                self.phase = "later"
                for up in LATER_TOGGLES:
                    self.toggle(up, line, process, run_dir)
                self.later_done_at = time.monotonic()
                self.phase = "done"
            elif line.startswith("NETLINK:EVENT:DOWN:"):
                self.toggle(True, line, process, run_dir)
            elif line.startswith("NETLINK:EVENT:UP:"):
                self.toggle(False, line, process, run_dir)

    def close(self) -> None:
        if self.qmp is not None:
            self.qmp.close()
        if self.directory is not None:
            os.close(self.directory)
            self.directory = None

    def evidence(self) -> dict:
        trace = list(self.qmp.trace) if self.qmp is not None else []
        gap = (None if self.later_done_at is None or self.brokers_at is None
               else self.brokers_at - self.later_done_at)
        return dict(kind="QMP set_link driver; private unix socket in the run directory",
                    phase=self.phase, trace=trace, actions=list(self.actions),
                    later_toggles_before_finish_s=gap,
                    timeline_note="host monotonic seconds when the harness read each line",
                    timeline=[list(entry) for entry in self.timeline])


def link_lines(stdout: str) -> list[str]:
    return [line for line in stdout.splitlines() if line.startswith("NETLINK:")]


def numbered(lines: list[str], prefix: str) -> list[int]:
    values = []
    for line in lines:
        if line.startswith(prefix):
            value = line[len(prefix):]
            if not NUMBER.fullmatch(value):
                raise ValueError(f"malformed link marker {line!r}")
            values.append(int(value))
    return values


def expected(case: str) -> dict:
    """Exact per-case expectations; values come from the independent design."""
    base_net = {"NET_ARP_REPLIES": 1, "NET_UDP_ECHOES": 1, "NET_DNS_ANSWERS": 1, "NET_APP_DENIED": 7,
                "NET_ARP_REJECTS": 2, "NET_DNS_REJECTS": 2, "NET_RX_TIMEOUTS": 1,
                "NET_CACHE_INTACT": 1, "NET_STALE_CAP": 1, "NET_TX_TIMEOUTS": 0}
    zero_probe = dict.fromkeys(("SUPPORTED", "FIRST_EPOCH", "DOWN_EPOCH", "UP_EPOCH", "TX_REFUSED",
                                "RX_REFUSED", "STALE_ABSENT", "ARP_REPLIES", "UDP_ECHOES", "OFFLINE",
                                "PREDOWN", "LAST_EPOCH"), 0) | {"DENIED": 3}
    if case == "normal":
        tx, rx_done, rx_delivered, denied, resets = 11, 10, 9, 12, 1
        kernel = dict(SUPPORTED=1, EPOCH=3, EVENTS=3, UPS=2, DOWNS=1, SYNTHETIC=0, TX_REFUSED=1,
                      RX_REFUSED=1, RX_DROPPED=1, WINDOWS=1, WINDOW_DELTA=0, EXHAUSTED=0)
        probe = zero_probe | dict(SUPPORTED=1, FIRST_EPOCH=1, DOWN_EPOCH=2, UP_EPOCH=3, TX_REFUSED=1,
                                  RX_REFUSED=1, STALE_ABSENT=1, ARP_REPLIES=1, UDP_ECHOES=1,
                                  PREDOWN=1, LAST_EPOCH=3)
        events = [("EVENT:UP", 1), ("EVENT:DOWN", 2), ("EVENT:UP", 3)]
        queries = [("UP", 1), ("DOWN", 2), ("UP", 3)]
        held, dropped, windows, refused = list(range(1, 11)), [1], 1, [2]
        peer_events = [*BASE_EVENTS, "udp_predown_replies", "arp_reconnect_sent", "udp_reconnect_replies"]
        peer = BASE_PEER | dict(accepted_frames=11, arp_requests=4, arp_reconnect_sent=1,
                                udp_echo_requests=3, udp_predown_replies=1, udp_reconnect_replies=1,
                                response_frames=10)
        toggles = [False, True]
    elif case == "storm":
        tx, rx_done, rx_delivered, denied, resets = 9, 8, 7, 13, 2
        kernel = dict(SUPPORTED=1, EPOCH=9, EVENTS=9, UPS=5, DOWNS=4, SYNTHETIC=0, TX_REFUSED=0,
                      RX_REFUSED=0, RX_DROPPED=1, WINDOWS=4, WINDOW_DELTA=0, EXHAUSTED=1)
        probe = zero_probe | dict(SUPPORTED=1, FIRST_EPOCH=1, OFFLINE=2, PREDOWN=1, LAST_EPOCH=None)
        events = [("EVENT:UP" if epoch % 2 else "EVENT:DOWN", epoch) for epoch in range(1, 10)]
        queries = None  # the probe samples a subset of the epochs
        held, dropped, windows, refused = list(range(1, 9)), [1], 4, []
        peer_events = [*BASE_EVENTS, "udp_predown_replies"]
        peer = BASE_PEER | dict(accepted_frames=9, udp_echo_requests=2, udp_predown_replies=1,
                                response_frames=8)
        toggles = [False, True] * 4 + [False, *LATER_TOGGLES]
    elif case == "nostatus":
        tx, rx_done, rx_delivered, denied, resets = 8, 7, 7, 11, 1
        kernel = dict.fromkeys(("SUPPORTED", "EPOCH", "EVENTS", "UPS", "DOWNS", "SYNTHETIC", "TX_REFUSED",
                                "RX_REFUSED", "RX_DROPPED", "WINDOWS", "WINDOW_DELTA", "EXHAUSTED",
                                "OBSERVATIONS", "BUSY", "QUERIES"), 0)
        probe = zero_probe
        events, queries, held, dropped, windows, refused = [], [], [], [], 0, []
        peer_events, peer, toggles = list(BASE_EVENTS), dict(BASE_PEER), None
    else:
        raise ValueError("unknown link case")
    net = base_net | {"NET_TX_SUBMITTED": tx, "NET_TX_COMPLETED": tx, "NET_TX_COMPLETIONS": tx,
                      "NET_RX_COMPLETED": rx_done, "NET_RX_DELIVERED": rx_delivered,
                      "NET_RX_FRAMES": rx_delivered, "NET_DENIED": denied,
                      "NET_RESETS": resets, "NET_RESET_ACKS": resets}
    return dict(net=net, kernel=kernel, probe=probe, events=events, queries=queries, held=held,
                dropped=dropped, windows=windows, refused=refused, peer_events=peer_events,
                peer=peer, toggles=toggles)


def check_markers(case: str, stdout: str, want: dict) -> dict:
    lines = link_lines(stdout)
    attach = [line for line in lines if line.startswith("NETLINK:ATTACH:")]
    if attach != ["NETLINK:ATTACH:NO_STATUS" if case == "nostatus" else "NETLINK:ATTACH:STATUS"]:
        raise ValueError(f"link attach markers {attach}")
    events = []
    for line in lines:
        match = re.fullmatch(r"NETLINK:((?:EVENT|SYNTHETIC):(?:UP|DOWN)):([0-9]{20})", line)
        if match:
            events.append((match.group(1), int(match.group(2))))
        elif line.startswith(("NETLINK:EVENT:", "NETLINK:SYNTHETIC:")):
            raise ValueError(f"malformed link event {line!r}")
    if events != want["events"]:
        raise ValueError(f"link events {events} != {want['events']}")
    exhausted = numbered(lines, "NETLINK:EXHAUSTED:")
    if exhausted != ([9] if case == "storm" else []):
        raise ValueError(f"exhaustion markers {exhausted}")
    if case == "storm":
        after = lines[next(index for index, line in enumerate(lines) if line.startswith("NETLINK:EXHAUSTED:")):]
        if any(line.startswith(("NETLINK:EVENT:", "NETLINK:SYNTHETIC:", "NETLINK:WINDOW_", "NETLINK:QUERY:"))
               for line in after):
            raise ValueError("link event or state after the exhaustion latch")
    queries = []
    for line in lines:
        match = re.fullmatch(r"NETLINK:QUERY:(UP|DOWN):([0-9]{20})", line)
        if match:
            queries.append((match.group(1), int(match.group(2))))
    if want["queries"] is not None and queries != want["queries"]:
        raise ValueError(f"probe-visible link states {queries} != {want['queries']}")
    if want["queries"] is None and (not queries or queries[0] != ("UP", 1) or any(
            epoch > 9 or (kind == "UP") != (epoch % 2 == 1) for kind, epoch in queries)):
        raise ValueError(f"storm probe-visible link states {queries}")
    for prefix, key in (("NETLINK:RX_HELD:", "held"), ("NETLINK:RX_DROPPED:", "dropped"),
                        ("NETLINK:DOWN_REFUSED:", "refused")):
        if numbered(lines, prefix) != want[key]:
            raise ValueError(f"{prefix} markers {numbered(lines, prefix)} != {want[key]}")
    windows = []
    for kind in ("TX", "PUBLISHED"):
        opened = numbered(lines, f"NETLINK:WINDOW_OPEN_{kind}:")
        closed = numbered(lines, f"NETLINK:WINDOW_CLOSE_{kind}:")
        if len(opened) != want["windows"] or opened != closed:
            raise ValueError(f"down-window {kind} counters changed: {opened} -> {closed}")
        windows.append(opened)
    if case == "normal":
        order = [lines.index(item) for item in (
            marker("NETLINK:RX_HELD:", PREDOWN_RX), marker("NETLINK:RX_DROPPED:", 1),
            marker("NETLINK:EVENT:DOWN:", 2), marker("NETLINK:QUERY:DOWN:", 2),
            marker("NETLINK:DOWN_REFUSED:", 2), marker("NETLINK:EVENT:UP:", 3),
            marker("NETLINK:QUERY:UP:", 3), marker("NETLINK:RX_HELD:", PREDOWN_RX + 1))]
        if order != sorted(order):
            raise ValueError("link-loss/reconnect markers out of order")
    return dict(events=len(events), queries=queries, windows=windows)


def check_evidence(report: dict, want: dict) -> None:
    found = report.get("evidence", {})
    for name, value in want["net"].items():
        if found.get(name) != value:
            raise ValueError(f"{name}: expected {value}, found {found.get(name)}")
    for name, value in want["kernel"].items():
        if found.get(f"NETLINK_{name}") != value:
            raise ValueError(f"NETLINK_{name}: expected {value}, found {found.get(f'NETLINK_{name}')}")
    if want["kernel"]["SUPPORTED"]:
        if found.get("NETLINK_OBSERVATIONS", 0) < want["kernel"]["EVENTS"] or found.get("NETLINK_QUERIES", 0) < 1:
            raise ValueError("kernel link observations/queries missing")
    for name, value in want["probe"].items():
        actual = found.get(f"NETLINK_PROBE_{name}")
        if value is None:
            if type(actual) is not int or not 1 <= actual <= want["kernel"]["EPOCH"]:
                raise ValueError(f"NETLINK_PROBE_{name}: {actual} outside the kernel epochs")
        elif actual != value:
            raise ValueError(f"NETLINK_PROBE_{name}: expected {value}, found {actual}")


def check_peer(peer: dict, want: dict) -> None:
    if peer.get("connected") is not True or peer.get("error") is not None:
        raise ValueError(f"testpeer connection/protocol failed: {peer.get('error')}")
    if peer.get("counts") != want["peer"]:
        raise ValueError(f"peer counts {peer.get('counts')} != {want['peer']}")
    events = peer.get("events", [])
    if ([event.get("event") for event in events] != want["peer_events"]
            or [event.get("sequence") for event in events] != list(range(1, len(want["peer_events"]) + 1))):
        raise ValueError("wrong/missing/duplicate packet transcript")
    hashes = [event.get("payload_sha256") for event in events if event["event"].startswith("udp_")]
    if len(hashes) != len(set(hashes)):
        raise ValueError("an echo payload repeated across link epochs")


def check_qmp(case: str, control: dict | None, want: dict) -> None:
    if want["toggles"] is None:
        if control is not None:
            raise ValueError("the STATUS-less case must not use QMP")
        return
    trace = control.get("trace", [])
    requests = [entry["request"] for entry in trace]
    if not requests or requests[0].get("execute") != "qmp_capabilities":
        raise ValueError("QMP trace does not start with capability negotiation")
    if any(request.get("execute") not in ALLOWED_QMP for request in requests):
        raise ValueError("QMP command outside the allowlist")
    toggles = []
    for request, entry in zip(requests[1:], trace[1:]):
        arguments = request.get("arguments")
        if (request.get("execute") != "set_link" or set(arguments or {}) != {"name", "up"}
                or arguments["name"] != NETDEV or type(arguments["up"]) is not bool
                or entry["response"].get("return") != {}):
            raise ValueError(f"unexpected QMP exchange {entry}")
        toggles.append(arguments["up"])
    if toggles != want["toggles"] or control.get("phase") != "done":
        raise ValueError(f"QMP set_link sequence {toggles} (phase {control.get('phase')})")
    if case == "storm":
        gap = control.get("later_toggles_before_finish_s")
        if type(gap) is not float or gap < MIN_HOLD_GAP_S:
            raise ValueError(f"later toggles not applied while the latched guest still ran ({gap})")


def evaluate_link(case: str, report: dict, stdout: str, peer: dict, control: dict | None) -> dict:
    feature, status, _ = CASES[case]
    if report.get("passed") is not True:
        raise ValueError("the joint-system/disk gate failed")
    if (report.get("case") != "normal" or report.get("kernel_feature") != feature
            or report.get("nic_status") is not status):
        raise ValueError("case/profile/NIC differs from the requested link run")
    digest = report.get("manifest", {}).get("system", {}).get("net_probe", {}).get("sha256")
    if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
        raise ValueError("missing inspected native network ELF hash")
    command = report.get("command", [])
    device = [item for item in command if item.startswith("virtio-net-pci,")]
    if len(device) != 1 or device[0].endswith(",status=off") == status:
        raise ValueError("QEMU NIC STATUS feature differs from the case")
    netdevs = [command[index + 1] for index, item in enumerate(command[:-1]) if item == "-netdev"]
    if len(netdevs) != 1 or not netdevs[0].startswith("socket,id=net0,connect=127.0.0.1:"):
        raise ValueError("unexpected QEMU network backend (no TAP, bridge or user network)")
    monitors = [(item, command[index + 1]) for index, item in enumerate(command[:-1])
                if item in ("-monitor", "-qmp", "-serial")]
    private = f"/{Path(report['run_directory']).name}/qmp.sock,server=on,wait=off"
    qmp = [value for item, value in monitors if item == "-qmp"]
    if (len(monitors) != 2 + (control is not None)
            or monitors.count(("-monitor", "none")) != 1 or monitors.count(("-serial", "stdio")) != 1
            or len(qmp) != (control is not None)
            or any(not (value.startswith("unix:") and value.endswith(private)) for value in qmp)):
        raise ValueError("unexpected QEMU monitor: only a private QMP socket is allowed")
    want = expected(case)
    markers = check_markers(case, stdout, want)
    check_evidence(report, want)
    check_peer(peer, want)
    check_qmp(case, control, want)
    return markers


def store(report: dict, volume_dir: Path, extra: dict[str, dict]) -> None:
    run_dir = Path(report["run_directory"]).resolve(strict=True)
    run_dir.relative_to((BUILD / "run").resolve(strict=True))
    volume_dir = volume_dir.resolve(strict=True)
    volume_dir.relative_to((BUILD / "run").resolve(strict=True))
    for path in [run_dir / "result.json", volume_dir / "last-system-stage.json",
                 *(run_dir / name for name in extra)]:
        try:
            details = path.lstat()
        except FileNotFoundError:
            continue
        if not stat.S_ISREG(details.st_mode) or details.st_nlink != 1:
            raise ValueError("link report must be a regular single-link file")
    for name, value in extra.items():
        save_json(run_dir / name, value)
    save_json(run_dir / "result.json", report)
    # A link run is never a recovery predecessor (check_parent needs native-system-net).
    save_json(volume_dir / "last-system-stage.json", report)


def run_net_link(case: str, *, timeout: int = MAX_TIMEOUT):
    if case not in CASES or not 0 < timeout <= MAX_TIMEOUT:
        raise ValueError("unknown case or unbounded timeout")
    feature, status, toggles = CASES[case]
    control = LinkControl(case, time.monotonic() + timeout) if toggles else None
    report = None
    try:
        with EthernetPeer(timeout=timeout, predown=status, reconnect=case == "normal") as peer:
            try:
                report, volume_dir = run_system("normal", timeout=timeout, kernel_feature=feature,
                                                network_port=peer.port, nic_status=status,
                                                qmp_hook=control)
            finally:
                if control is not None:
                    control.close()
        evidence = peer.evidence()
    except Exception as error:
        if report is None:
            raise
        report["passed"] = False
        report["link_gate"] = False
        report["link_error"] = f"{type(error).__name__}: {error}"
        store(report, volume_dir, {})
        return report
    qmp = control.evidence() if control is not None else None
    report["network_peer"] = evidence
    report["link_control"] = qmp
    report["link_gate"] = False
    stdout = (Path(report["run_directory"]) / "serial.txt").read_text()
    try:
        report["link_markers"] = evaluate_link(case, report, stdout, evidence, qmp)
        if source_hash() != report["source_sha256"]:
            raise ValueError("source changed during link evidence collection")
        report["link_gate"] = True
    except Exception as error:
        report["passed"] = False
        report["link_error"] = f"{type(error).__name__}: {error}"
    report["link_case"] = case
    store(report, volume_dir, {"network-peer.json": evidence, "link-control.json": qmp or {}})
    print(f"NATIVE NET LINK {case}: passed={report['passed']} link_gate={report['link_gate']} "
          f"{report['run_directory']}")
    if "link_error" in report:
        print(report["link_error"])
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=tuple(CASES), required=True)
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    options = parser.parse_args()
    raise SystemExit(0 if run_net_link(options.case, timeout=options.timeout)["passed"] else 1)
