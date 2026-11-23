"""HOST_ONLY H04-T3 link harness counterexamples; fixtures are not native evidence.

They exercise the peer schedule extensions, the serial-marker driven QMP
controller (with a fake transport) and the independent verifier of a native
link-loss/reconnect run. Only tools/run_net_link.py against QEMU is native.
"""
import copy
import hashlib
from pathlib import Path
import struct
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import net_test_peer as peer  # noqa: E402
import run_net_link as link  # noqa: E402
import run_system  # noqa: E402
from run_volume import execute  # noqa: E402

ARP = bytes.fromhex(
    "ffffffffffff02000000000708060001080006040001020000000007c0000202000000000000c0000201").ljust(60, b"\0")


def query(name=peer.DNS_NAME, identifier=0x1234):
    wire = b"".join(bytes((len(label),)) + label.encode() for label in name.split(".")) + b"\0"
    return struct.pack("!6H", identifier, 0x0100, 1, 0, 0, 0) + wire + b"\0\x01\0\x01"


def client_udp(payload, target=peer.DNS_IP, port=53, source_port=49152):
    length = len(payload) + 8
    udp = struct.pack("!HHHH", source_port, port, length, 0) + payload
    pseudo = peer.CLIENT_IP + target + b"\0\x11" + length.to_bytes(2, "big")
    udp = udp[:6] + (peer.checksum(pseudo + udp) or 0xffff).to_bytes(2, "big") + udp[8:]
    ip = bytes.fromhex("450000001632400040110000") + peer.CLIENT_IP + target
    ip = ip[:2] + (20 + length).to_bytes(2, "big") + ip[4:]
    ip = ip[:10] + peer.checksum(ip).to_bytes(2, "big") + ip[12:]
    return (peer.PEER_MAC + peer.CLIENT_MAC + b"\x08\0" + ip + udp).ljust(60, b"\0")


def echo(tag):
    return client_udp(tag + b"\x05\0\0\0\0\0\0\0", peer.PEER_IP, 9000, 49153)


def base_schedule(machine):
    for _ in range(3):
        machine.handle(ARP)
    machine.handle(echo(b"nieuw-os-native/1:"))
    for _ in range(3):
        machine.handle(client_udp(query()))
    machine.handle(client_udp(query(peer.TIMEOUT_NAME, 0x5678)))


class PeerScheduleTests(unittest.TestCase):
    def test_default_schedule_is_unchanged_and_rejects_link_extras(self):
        machine = peer.PeerMachine()
        base_schedule(machine)
        self.assertNotIn("udp_predown_replies", machine.counts)
        for frame in (ARP, echo(b"nieuw-os-link/old:")):
            with self.assertRaises(peer.ProtocolError):
                copy.deepcopy(machine).handle(frame)

    def test_reconnect_requires_predown_and_strict_kwarg_types(self):
        for kwargs in (dict(reconnect=True), dict(predown=1), dict(predown=True, reconnect="yes")):
            with self.assertRaises(ValueError):
                peer.PeerMachine(**kwargs)

    def test_predown_echo_only_after_the_base_schedule(self):
        machine = peer.PeerMachine(predown=True)
        for _ in range(3):
            machine.handle(ARP)
        machine.handle(echo(b"nieuw-os-native/1:"))
        with self.assertRaises(peer.ProtocolError):
            copy.deepcopy(machine).handle(echo(b"nieuw-os-link/old:"))
        for _ in range(3):
            machine.handle(client_udp(query()))
        machine.handle(client_udp(query(peer.TIMEOUT_NAME, 0x5678)))
        self.assertEqual(len(machine.handle(echo(b"nieuw-os-link/old:"))), 1)
        with self.assertRaises(peer.ProtocolError):
            machine.handle(ARP)  # no reconnect schedule

    def test_reconnect_needs_fresh_arp_before_the_new_echo(self):
        machine = peer.PeerMachine(predown=True, reconnect=True)
        base_schedule(machine)
        with self.assertRaises(peer.ProtocolError):
            copy.deepcopy(machine).handle(ARP)  # before the pre-down echo
        machine.handle(echo(b"nieuw-os-link/old:"))
        with self.assertRaises(peer.ProtocolError):
            copy.deepcopy(machine).handle(echo(b"nieuw-os-link/new:"))
        reply = machine.handle(ARP)
        self.assertEqual(reply, [peer.arp_reply()])
        machine.handle(echo(b"nieuw-os-link/new:"))
        self.assertEqual([event["event"] for event in machine.events][-3:],
                         ["udp_predown_replies", "arp_reconnect_sent", "udp_reconnect_replies"])
        for frame in (ARP, echo(b"nieuw-os-link/third:")):
            with self.assertRaises(peer.ProtocolError):
                copy.deepcopy(machine).handle(frame)


class FakeQmp:
    def __init__(self, reply=None):
        self.trace = [dict(request=dict(execute="qmp_capabilities", id=1), response=dict(**{"return": {}}, id=1))]
        self.reply = {} if reply is None else reply
        self.closed = False

    def command(self, name, arguments=None):
        request = dict(execute=name, id=len(self.trace) + 1)
        if arguments is not None:
            request["arguments"] = arguments
        self.trace.append(dict(request=request, response={"return": self.reply, "id": request["id"]}))
        return self.reply

    def close(self):
        self.closed = True


def controller(case, reply=None):
    connections = []

    def connect(run_dir, process, deadline):
        connections.append(run_dir)
        return FakeQmp(reply), None
    control = link.LinkControl(case, deadline=float("inf"), connect=connect)
    return control, connections


def held(value):
    return link.marker("NETLINK:RX_HELD:", value)


def feed(control, lines):
    for line in lines:
        control(line, None, Path("run"))


class LinkControlTests(unittest.TestCase):
    def test_normal_reacts_only_to_kernel_markers_and_connects_lazily(self):
        control, connections = controller("normal")
        feed(control, ["NETLINK:ATTACH:STATUS", link.marker("NETLINK:EVENT:UP:", 1),
                       link.marker("NETLINK:DOWN_REFUSED:", 2), *(held(i) for i in range(1, 8))])
        self.assertEqual((connections, control.actions), ([], []))
        feed(control, [held(8), link.marker("NETLINK:EVENT:DOWN:", 2), held(8)])
        self.assertEqual([action["up"] for action in control.actions], [False])
        feed(control, [link.marker("NETLINK:DOWN_REFUSED:", 3), link.marker("NETLINK:DOWN_REFUSED:", 2),
                       link.marker("NETLINK:EVENT:UP:", 3), link.marker("NETLINK:DOWN_REFUSED:", 2)])
        self.assertEqual([action["up"] for action in control.actions], [False, True])
        self.assertEqual((len(connections), control.phase), (1, "done"))
        requests = [entry["request"] for entry in control.evidence()["trace"]]
        self.assertEqual([request.get("arguments") for request in requests],
                         [None, {"name": "net0", "up": False}, {"name": "net0", "up": True}])

    def test_storm_toggles_on_each_event_then_three_later_after_exhaustion(self):
        control, _ = controller("storm")
        feed(control, [link.marker("NETLINK:EVENT:UP:", 1), held(8)])
        for epoch in range(2, 10):
            feed(control, [link.marker("NETLINK:EVENT:DOWN:" if epoch % 2 == 0 else "NETLINK:EVENT:UP:", epoch)])
        feed(control, [link.marker("NETLINK:EXHAUSTED:", 9), link.marker("NETLINK:EVENT:UP:", 10)])
        self.assertEqual([action["up"] for action in control.actions],
                         [False, True] * 4 + [False, True, False, True])
        self.assertEqual(control.phase, "done")

    def test_allowlist_reply_and_toggle_budget_fail_closed(self):
        control, _ = controller("normal")
        feed(control, [held(8)])
        for name in ("human-monitor-command", "quit", "system_reset", "netdev_add"):
            with self.assertRaises(ValueError):
                control.command(name)
        bad, _ = controller("normal", reply={"unexpected": 1})
        with self.assertRaises(RuntimeError):
            feed(bad, [held(8)])
        storm, _ = controller("storm")
        feed(storm, [held(8)])
        with self.assertRaises(RuntimeError):
            for epoch in range(2, 40):
                feed(storm, [link.marker("NETLINK:EVENT:DOWN:" if epoch % 2 == 0 else "NETLINK:EVENT:UP:", epoch)])
        with self.assertRaises(ValueError):
            link.LinkControl("nostatus", 0.0)


def lines_for(case):
    """A self-consistent serial tail, written out by hand for each case."""
    m = link.marker
    out = ["BOOT:EBS_OK"]
    if case == "nostatus":
        return out + ["NETLINK:ATTACH:NO_STATUS"]
    out += ["NETLINK:ATTACH:STATUS", m("NETLINK:EVENT:UP:", 1), m("NETLINK:QUERY:UP:", 1)]
    out += [held(i) for i in range(1, 9)]
    window = [m("NETLINK:WINDOW_OPEN_TX:", 9), m("NETLINK:WINDOW_OPEN_PUBLISHED:", 9 << 16 | 9)]
    closing = [m("NETLINK:WINDOW_CLOSE_TX:", 9), m("NETLINK:WINDOW_CLOSE_PUBLISHED:", 9 << 16 | 9)]
    if case == "normal":
        return out + [m("NETLINK:RX_DROPPED:", 1), *window, m("NETLINK:EVENT:DOWN:", 2),
                      m("NETLINK:QUERY:DOWN:", 2), m("NETLINK:DOWN_REFUSED:", 2), *closing,
                      m("NETLINK:EVENT:UP:", 3), m("NETLINK:QUERY:UP:", 3), held(9), held(10)]
    out += [m("NETLINK:RX_DROPPED:", 1)]
    for epoch in range(2, 10, 2):
        out += [*window, m("NETLINK:EVENT:DOWN:", epoch), m("NETLINK:QUERY:DOWN:", epoch), *closing,
                m("NETLINK:EVENT:UP:", epoch + 1)]
    return out + [m("NETLINK:QUERY:UP:", 9), m("NETLINK:EXHAUSTED:", 9)]


def fixture(case):
    feature, status, toggles = link.CASES[case]
    want = link.expected(case)
    evidence = dict(want["net"])
    evidence.update({f"NETLINK_{key}": value for key, value in want["kernel"].items()})
    evidence.update({f"NETLINK_PROBE_{key}": (7 if value is None else value) for key, value in want["probe"].items()})
    if status:
        evidence.update(NETLINK_OBSERVATIONS=6405, NETLINK_BUSY=0, NETLINK_QUERIES=21)
    device = "virtio-net-pci,netdev=net0,id=net,addr=0x7" + ("" if status else ",status=off")
    command = ["qemu", "-monitor", "none", "-serial", "stdio", "-netdev",
               "socket,id=net0,connect=127.0.0.1:40000", "-device", device]
    if toggles:
        command += ["-qmp", "unix:build/run/system-normal-x/qmp.sock,server=on,wait=off"]
    report = dict(passed=True, case="normal", kernel_feature=feature, nic_status=status, command=command,
                  run_directory="/checkout/build/run/system-normal-x", evidence=evidence,
                  manifest=dict(system=dict(net_probe=dict(sha256="b" * 64))))
    events = list(want["peer_events"])
    peer_evidence = dict(connected=True, error=None, counts=dict(want["peer"]),
                         events=[dict(sequence=index + 1, event=event,
                                      payload_sha256=hashlib.sha256(event.encode()).hexdigest())
                                 for index, event in enumerate(events)])
    control = None
    if toggles:
        trace = [dict(request=dict(execute="qmp_capabilities", id=1), response={"return": {}, "id": 1})]
        for index, up in enumerate(want["toggles"]):
            trace.append(dict(request=dict(execute="set_link", id=index + 2, arguments=dict(name="net0", up=up)),
                              response={"return": {}, "id": index + 2}))
        control = dict(phase="done", trace=trace, later_toggles_before_finish_s=3.0 if case == "storm" else None)
    return report, "\n".join(lines_for(case)) + "\n", peer_evidence, control


class VerifierTests(unittest.TestCase):
    def test_consistent_transcripts_pass(self):
        for case in link.CASES:
            with self.subTest(case=case):
                link.evaluate_link(case, *fixture(case))

    def test_handwritten_normal_expectations(self):
        want = link.expected("normal")
        self.assertEqual((want["net"]["NET_TX_SUBMITTED"], want["net"]["NET_RX_COMPLETED"],
                          want["net"]["NET_RX_DELIVERED"], want["net"]["NET_DENIED"]), (11, 10, 9, 12))
        self.assertEqual(link.expected("storm")["toggles"], [False, True, False, True, False, True, False,
                                                             True, False, True, False, True])

    def assert_rejected(self, case, mutate, message=None):
        report, stdout, peer_evidence, control = copy.deepcopy(fixture(case))
        result = mutate(report, stdout, peer_evidence, control)
        if isinstance(result, str):
            self.assertNotEqual(result, stdout, "mutation must change the serial transcript")
            stdout = result
        with self.assertRaises(ValueError, msg=message):
            link.evaluate_link(case, report, stdout, peer_evidence, control)

    def test_kernel_marker_counterexamples(self):
        m = link.marker
        self.assert_rejected("storm", lambda *a: a[1] + m("NETLINK:EVENT:DOWN:", 10) + "\n", "event after latch")
        self.assert_rejected("storm", lambda *a: a[1] + m("NETLINK:QUERY:UP:", 9) + "\n", "state after latch")
        self.assert_rejected("normal", lambda *a: a[1].replace(
            m("NETLINK:WINDOW_CLOSE_TX:", 9), m("NETLINK:WINDOW_CLOSE_TX:", 10)), "TX published while down")
        self.assert_rejected("normal", lambda *a: a[1].replace(
            m("NETLINK:WINDOW_CLOSE_PUBLISHED:", 9 << 16 | 9), m("NETLINK:WINDOW_CLOSE_PUBLISHED:", 9 << 16 | 10)),
            "doorbell while down")
        self.assert_rejected("normal", lambda *a: a[1].replace(
            m("NETLINK:EVENT:UP:", 3), m("NETLINK:SYNTHETIC:DOWN:", 3) + "\n" + m("NETLINK:EVENT:UP:", 3)))
        self.assert_rejected("normal", lambda *a: a[1].replace(m("NETLINK:RX_DROPPED:", 1) + "\n", ""),
                             "stale RX copy survived the down edge")
        swapped = lambda *a: a[1].replace(  # noqa: E731
            m("NETLINK:QUERY:DOWN:", 2) + "\n" + m("NETLINK:DOWN_REFUSED:", 2),
            m("NETLINK:DOWN_REFUSED:", 2) + "\n" + m("NETLINK:QUERY:DOWN:", 2))
        self.assert_rejected("normal", swapped, "refusal before the probe saw the down state")
        self.assert_rejected("nostatus", lambda *a: a[1] + m("NETLINK:EVENT:UP:", 1) + "\n",
                             "event without STATUS")
        self.assert_rejected("normal", lambda *a: a[1].replace("NETLINK:ATTACH:STATUS", "NETLINK:ATTACH:NO_STATUS"))

    def test_counter_windows_must_bracket_their_down_interval(self):
        """Equal window counters prove nothing unless the window spans the link-down interval."""
        m = link.marker
        down = m("NETLINK:EVENT:DOWN:", 2) + "\n"
        window = [m("NETLINK:WINDOW_OPEN_TX:", 9), m("NETLINK:WINDOW_OPEN_PUBLISHED:", 9 << 16 | 9)]
        closing = [m("NETLINK:WINDOW_CLOSE_TX:", 9), m("NETLINK:WINDOW_CLOSE_PUBLISHED:", 9 << 16 | 9)]
        block = "".join(line + "\n" for line in window)
        close = "".join(line + "\n" for line in closing)

        def before_down(_report, stdout, *_):  # both markers moved before EVENT:DOWN, values unchanged
            return stdout.replace(close, "").replace(block + down, block + close + down)
        with self.subTest(case="before_down"):
            self.assert_rejected("normal", before_down, "window closed while the link was still up")

        def after_up(_report, stdout, *_):  # window opened only after the reconnect
            up = m("NETLINK:EVENT:UP:", 3) + "\n"
            return stdout.replace(block, "").replace(close, "").replace(up, up + block + close)
        with self.subTest(case="after_up"):
            self.assert_rejected("normal", after_up, "window outside the down interval")

        def tx_only_late(_report, stdout, *_):  # one kind closes after the reconnect
            up = m("NETLINK:EVENT:UP:", 3) + "\n"
            tx = closing[0] + "\n"
            return stdout.replace(tx, "").replace(up, up + tx)
        with self.subTest(case="tx_only_late"):
            self.assert_rejected("normal", tx_only_late, "TX window closed after the link came back")

        def storm_last_late(_report, stdout, *_):  # the last storm window closes after EVENT:UP:9
            up = m("NETLINK:EVENT:UP:", 9) + "\n"
            head, tail = stdout.rsplit(close + up, 1)
            return head + up + close + tail
        with self.subTest(case="storm_last_late"):
            self.assert_rejected("storm", storm_last_late, "last storm window closed after the up edge")

    def test_final_evidence_counterexamples(self):
        for case, key, value in (("normal", "NETLINK_WINDOW_DELTA", 1), ("normal", "NETLINK_TX_REFUSED", 0),
                                 ("normal", "NET_TX_SUBMITTED", 12), ("normal", "NETLINK_PROBE_STALE_ABSENT", 0),
                                 ("storm", "NET_RESET_ACKS", 1), ("storm", "NETLINK_EXHAUSTED", 0),
                                 ("storm", "NETLINK_PROBE_LAST_EPOCH", 10), ("nostatus", "NETLINK_OBSERVATIONS", 1),
                                 ("nostatus", "NET_DENIED", 10)):
            with self.subTest(key=key):
                self.assert_rejected(case, lambda report, *_: report["evidence"].__setitem__(key, value))
        self.assert_rejected("normal", lambda report, *_: report.__setitem__("passed", False))
        self.assert_rejected("normal", lambda report, *_: report.__setitem__("nic_status", False))

    def test_qemu_command_counterexamples(self):
        def replace(item, new):
            return lambda report, *_: report.__setitem__(
                "command", [new if part == item else part for part in report["command"]])
        self.assert_rejected("normal", replace("socket,id=net0,connect=127.0.0.1:40000", "tap,id=net0"))
        self.assert_rejected("normal", replace("none", "stdio"))
        self.assert_rejected("nostatus", replace("virtio-net-pci,netdev=net0,id=net,addr=0x7,status=off",
                                                 "virtio-net-pci,netdev=net0,id=net,addr=0x7"))
        self.assert_rejected("normal", lambda report, *_: report["command"].extend(["-qmp", "tcp:0.0.0.0:4444"]))
        self.assert_rejected("normal", replace("unix:build/run/system-normal-x/qmp.sock,server=on,wait=off",
                                               "unix:/tmp/qmp.sock,server=on,wait=off"))

    def test_qmp_and_peer_counterexamples(self):
        def request(index, **changes):
            return lambda *a: a[3]["trace"][index]["request"].update(changes)
        self.assert_rejected("normal", request(1, execute="human-monitor-command"))
        self.assert_rejected("normal", request(1, arguments=dict(name="net", up=False)))
        self.assert_rejected("normal", request(2, arguments=dict(name="net0", up=1)))
        self.assert_rejected("normal", lambda *a: a[3]["trace"].pop(0))
        self.assert_rejected("storm", lambda *a: a[3]["trace"].pop())
        self.assert_rejected("storm", lambda *a: a[3].__setitem__("later_toggles_before_finish_s", 0.01),
                             "later toggles raced the guest finish")
        report, stdout, peer_evidence, _ = fixture("nostatus")
        with self.assertRaises(ValueError):
            link.evaluate_link("nostatus", report, stdout, peer_evidence, dict(phase="done", trace=[]))
        self.assert_rejected("normal", lambda *a: a[2]["events"][-1].__setitem__(
            "payload_sha256", a[2]["events"][3]["payload_sha256"]), "echo bytes reused across epochs")
        self.assert_rejected("normal", lambda *a: a[2]["counts"].__setitem__("arp_reconnect_sent", 0))
        self.assert_rejected("storm", lambda *a: a[2].__setitem__("error", "ProtocolError: x"))


class SystemHookTests(unittest.TestCase):
    def test_status_off_argument_and_link_only_controls(self):
        self.assertTrue(run_system.network_arguments(40000, status=False)[-1].endswith(",romfile=,status=off"))
        self.assertTrue(run_system.network_arguments(40000)[-1].endswith(",romfile="))
        with self.assertRaises(ValueError):
            run_system.network_arguments(40000, status="off")
        for kwargs in (dict(nic_status=False), dict(qmp_hook=lambda *_: None)):
            with self.assertRaises(ValueError, msg=kwargs):
                run_system.run_system("normal", kernel_feature="native-system-net", network_port=40000, **kwargs)
            with self.assertRaises(ValueError, msg=kwargs):
                run_system.run_system("normal", **kwargs)

    def test_execute_hook_sees_ordered_lines_and_a_hook_error_kills_the_child(self):
        script = "import sys,time\nfor i in range(3): print(f'L{i}', flush=True)\ntime.sleep(60)\n"
        seen = []

        def hook(line, process):
            seen.append(line)
            if line == "L2":
                raise RuntimeError("stop")
        with self.assertRaises(RuntimeError) as caught:
            execute([sys.executable, "-c", script], None, timeout=30, on_line=hook)
        self.assertEqual(seen, ["L0", "L1", "L2"])
        self.assertEqual(caught.exception.partial_stdout.splitlines(), ["L0", "L1", "L2"])
        completed = execute([sys.executable, "-c", "print('A'); print('B')"], None, timeout=30,
                            on_line=lambda line, process: seen.append(line))
        self.assertEqual((completed[0], seen[-2:]), (0, ["A", "B"]))


if __name__ == "__main__":
    unittest.main()
