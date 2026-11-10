"""Host-only anti-false-pass fixtures. No fixture is native execution evidence."""
from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import capture_system as capture
import test_system as system_gate
from test_system_harness import system_image, transcript
from test_net_peer import completed_fixture
from test_tcp_harness import fixture as tcp_fixture
from run_net_system import NET_EXACT
from run_system import network_arguments
import volume_disk as disk


def put_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


class SystemCaptureTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.build = self.root / "build"
        self.run_root = self.build / "run"
        self.run_root.mkdir(parents=True)
        (self.build / "logs").mkdir()
        (self.root / "docs/verification").mkdir(parents=True)
        self.destination = self.root / "docs/verification/full-release-host-fixture"
        self.source = "a" * 64
        self.test_inputs = {"tests/test_system_harness.py": "c" * 64}
        for name, value in (("ROOT", self.root), ("BUILD", self.build)):
            p = patch.object(capture, name, value)
            p.start()
            self.addCleanup(p.stop)
        for name, value in (("source_hash", self.source), ("inputs", self.test_inputs), ("_git_head", "b" * 40)):
            p = patch.object(capture, name, return_value=value)
            p.start()
            self.addCleanup(p.stop)
        self.volume = self.run_root / ("system-image-" + "3" * 32)
        self.volume.mkdir()
        image = self.volume / "volume.img"
        empty_hash = disk.create_image(image)["sha256"]
        raw = bytearray(system_image())
        image.write_bytes(raw)
        after = capture.digest(raw)
        oracle = disk.analyse(raw, capture.OPAQUE_OBJECTS)
        capture.check_disk(oracle)
        self.reports = []
        self.runs = []
        self.manifests = []
        for index, case in enumerate(("normal", "recover")):
            run = self.run_root / (f"system-{case}-" + str(index + 1) * 32)
            run.mkdir()
            (run / "volume.after.img").write_bytes(raw)
            (run / "OVMF_CODE.fd").write_bytes(b"host fixture firmware code")
            (run / "OVMF_VARS.fd").write_bytes(b"host fixture firmware variables")
            esp, manifest = self.make_esp(run / "esp", case)
            self.manifests.append(manifest)
            serial = transcript(case)
            (run / "serial.txt").write_bytes(serial.encode())
            (run / "stderr.txt").write_bytes(b"host-only fixture\r\n")
            report = dict(schema_version=1, source_fingerprint_version=3, source_commit="b" * 40,
                          case=case, passed=True, source_sha256=self.source,
                          kernel_feature=capture.FEATURE[case], manifest=manifest,
                          run_directory=str(run), volume_directory=str(self.volume),
                          raw_qemu_exit=33, timed_out=False, duration_s=1.0, timeout_s=180,
                          before_sha256=empty_hash if case == "normal" else after,
                          after_sha256=after, evidence=capture.evaluate(case, serial, 33, False),
                          oracle=oracle, command=self.command(case, run),
                          firmware_sha256=dict(code=capture.digest((run / "OVMF_CODE.fd").read_bytes()),
                                               vars_template="d" * 64),
                          esp_sha256=esp)
            self.reports.append(report)
            self.runs.append(run)
            put_json(run / "result.json", report)
        put_json(self.volume / "last-system-stage.json", self.reports[-1])
        self.make_esp(self.build / "esp", "normal")
        put_json(self.build / "manifest.json", self.manifests[0])
        hosts = []
        host_commands = [
            [sys.executable, "-B", "-m", "unittest", "discover", "-s", "tests", "-p", "test_system_*.py"],
            ["cargo", "+1.90.0", "test", "--locked", "--manifest-path", "sdk/Cargo.toml", "-p", "nieuw-os-sdk-runtime"],
        ]
        for index in range(2):
            host = dict(command=host_commands[index], exit=0)
            for stream in ("stdout", "stderr"):
                path = self.build / "logs" / f"system-host-{index}.{stream}.txt"
                path.write_bytes(f"host-only {stream} fixture\r\n".encode())
                host[stream] = str(path)
            hosts.append(host)
        self.gate = dict(schema_version=1, source_fingerprint_version=3, status="completed", passed=True,
                         source_sha256=self.source, test_inputs=self.test_inputs, host=hosts,
                         stages=[{key: report[key] for key in ("case", "passed", "run_directory",
                                  "raw_qemu_exit", "before_sha256", "after_sha256", "duration_s")}
                                 for report in self.reports],
                         final_manifest=str(self.build / "manifest.json"),
                         final_kernel_sha256=self.manifests[0]["kernel"]["sha256"])
        self.save_gate()

    def make_esp(self, esp, case):
        (esp / "EFI/BOOT").mkdir(parents=True)
        (esp / "kernel.elf").write_bytes(f"host fixture kernel {case}".encode())
        (esp / "EFI/BOOT/BOOTX64.EFI").write_bytes(b"host fixture loader")
        hashes = {"esp/" + name: capture.digest((esp / name).read_bytes())
                  for name in ("kernel.elf", "EFI/BOOT/BOOTX64.EFI")}
        manifest = dict(schema_version=1, source_fingerprint_version=3, source_sha256=self.source,
                        kernel_feature=capture.FEATURE[case],
                        kernel=dict(sha256=hashes["esp/kernel.elf"]),
                        loader=dict(sha256=hashes["esp/EFI/BOOT/BOOTX64.EFI"]),
                        system={key: dict(sha256="e" * 64) for key in
                                ("system_init", "system_app", "system_worker", "keyring_service", "volume_service")})
        return hashes, manifest

    def command(self, case, run):
        rel = run.relative_to(self.root).as_posix()
        image = (self.volume / "volume.img").relative_to(self.root).as_posix()
        mode = "on" if case == "recover" else "off"
        return ["/fixture/qemu-system-x86_64", "-machine", "pc-q35-8.2", "-accel", "tcg,thread=single",
                "-cpu", "qemu64", "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none",
                "-monitor", "none", "-serial", "stdio", "-nic", "none", "-no-reboot",
                "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
                "-drive", f"if=pflash,unit=1,format=raw,file={rel}/OVMF_VARS.fd",
                "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
                "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
                "-drive", f"if=none,id=volume,format=raw,cache=writeback,readonly={mode},file={image}",
                "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-SYSTEM-TEST",
                "-object", "rng-random,id=rng0,filename=/dev/urandom",
                "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000"]

    def save_gate(self):
        put_json(self.build / "system-gate.json", self.gate)

    def save_report(self, index):
        put_json(self.runs[index] / "result.json", self.reports[index])

    def rejected(self):
        with self.assertRaises((ValueError, FileExistsError)):
            capture.capture(self.destination)
        self.assertFalse(self.destination.exists())

    def test_valid_snapshot_preserves_exact_raw_bytes_and_hashes_without_binaries(self):
        original = (self.runs[0] / "serial.txt").read_bytes()
        archive = capture.capture(self.destination)
        self.assertEqual((archive / "0-normal/serial.txt").read_bytes(), original)
        self.assertEqual((archive / "system-gate.json").read_bytes(), (self.build / "system-gate.json").read_bytes())
        self.assertEqual((archive / "1-recover/stderr.txt").read_bytes(), b"host-only fixture\r\n")
        self.assertFalse(any(p.suffix in (".img", ".elf", ".fd", ".efi") for p in archive.rglob("*")))
        for line in (archive / "SHA256SUMS").read_text().splitlines():
            value, name = line.split("  ", 1)
            self.assertEqual(hashlib.sha256((archive / name).read_bytes()).hexdigest(), value)

    def test_exact_kvm_acceleration_is_accepted_and_other_values_are_rejected(self):
        command = self.command("normal", self.runs[0])
        command[command.index("-accel") + 1] = "kvm"
        capture._command(command, "normal", self.runs[0], self.volume / "volume.img", self.root)
        command[command.index("-accel") + 1] = "kvm,dirty-ring-size=4096"
        with self.assertRaisesRegex(ValueError, "unsupported QEMU profile"):
            capture._command(command, "normal", self.runs[0], self.volume / "volume.img", self.root)

    def test_incomplete_false_or_stale_gate_is_rejected_before_writes(self):
        good = copy.deepcopy(self.gate)
        for key, value in (("status", "running"), ("passed", False), ("source_sha256", "f" * 64),
                           ("test_inputs", {}), ("stages", []), ("host", [])):
            with self.subTest(key=key):
                self.gate = {**good, key: value}
                self.save_gate()
                self.rejected()

    def test_stage_missing_duplicate_out_of_order_or_failed_is_rejected(self):
        good = copy.deepcopy(self.gate["stages"])
        for stages in ([good[0]], list(reversed(good)), [good[0], good[0]],
                       [{**good[0], "passed": False}, good[1]]):
            self.gate["stages"] = stages
            self.save_gate()
            self.rejected()

    def test_pass_boolean_cannot_hide_wrong_raw_exit_missing_markers_or_timeout(self):
        for key, value in (("raw_qemu_exit", 0), ("timed_out", True), ("evidence", {})):
            with self.subTest(key=key):
                original = copy.deepcopy(self.reports[0])
                self.reports[0][key] = value
                self.save_report(0)
                self.rejected()
                self.reports[0] = original
                self.save_report(0)
        (self.runs[0] / "serial.txt").write_text("M1:PASS\n")
        self.rejected()

    def test_stale_esp_kernel_loader_or_final_artifact_is_rejected(self):
        paths = [self.runs[0] / "esp/kernel.elf", self.runs[1] / "esp/EFI/BOOT/BOOTX64.EFI",
                 self.build / "esp/kernel.elf"]
        for path in paths:
            with self.subTest(path=path):
                original = path.read_bytes()
                path.write_bytes(b"different untested binary")
                self.rejected()
                path.write_bytes(original)

    def test_recovery_readonly_cannot_be_replaced_by_snapshot_or_writable_disk(self):
        original = self.reports[1]["command"]
        for command in ([arg.replace("readonly=on,file=build/run/system-image-", "readonly=off,file=build/run/system-image-")
                         for arg in original], original + ["-snapshot"], ["/bin/true"]):
            self.reports[1]["command"] = command
            self.save_report(1)
            self.rejected()

    def test_real_disk_hash_or_contents_cannot_be_replaced_by_stored_oracle(self):
        path = self.runs[0] / "volume.after.img"
        raw = path.read_bytes()
        damaged = bytearray(raw)
        damaged[(disk.FIRST + disk.DATA_START) * disk.SECTOR] ^= 1
        path.write_bytes(damaged)
        # Even updating the report hash cannot turn a corrupt actual document
        # into a pass via the old, apparently successful oracle JSON.
        self.reports[0]["after_sha256"] = capture.digest(damaged)
        self.gate["stages"][0]["after_sha256"] = capture.digest(damaged)
        self.save_report(0)
        self.save_gate()
        self.rejected()

    def test_current_image_change_breaks_cold_chain(self):
        (self.volume / "volume.img").write_bytes(bytes(disk.IMAGE_SECTORS * disk.SECTOR))
        self.rejected()

    def test_individually_valid_recovery_image_still_cannot_change_unused_disk_bytes(self):
        path = self.runs[1] / "volume.after.img"
        raw = bytearray(path.read_bytes())
        # An unallocated chunk is invisible in the document oracle. The exact
        # readonly cold-chain hash must still detect this write.
        raw[(disk.FIRST + disk.DATA_START + 3 * disk.CHUNK_SECTORS) * disk.SECTOR] ^= 1
        oracle = disk.analyse(raw, capture.OPAQUE_OBJECTS)
        capture.check_disk(oracle)
        path.write_bytes(raw)
        self.reports[1]["after_sha256"] = capture.digest(raw)
        self.reports[1]["oracle"] = oracle
        self.gate["stages"][1]["after_sha256"] = capture.digest(raw)
        self.save_report(1)
        self.save_gate()
        self.rejected()

    def test_fresh_final_manifest_cannot_substitute_an_untested_normal_kernel(self):
        replacement = b"an untested but freshly hashed kernel"
        (self.build / "esp/kernel.elf").write_bytes(replacement)
        manifest = copy.deepcopy(self.manifests[0])
        manifest["kernel"]["sha256"] = capture.digest(replacement)
        put_json(self.build / "manifest.json", manifest)
        self.gate["final_kernel_sha256"] = capture.digest(replacement)
        self.save_gate()
        self.rejected()

    def test_stale_run_source_or_replaced_firmware_is_rejected(self):
        self.reports[0]["source_sha256"] = "f" * 64
        self.save_report(0)
        self.rejected()
        self.reports[0]["source_sha256"] = self.source
        self.save_report(0)
        (self.runs[1] / "OVMF_CODE.fd").write_bytes(b"untested firmware")
        self.rejected()

    def test_failed_host_check_is_not_saved_as_completed_success(self):
        self.gate["host"][0]["exit"] = 1
        self.save_gate()
        self.rejected()

    def test_success_exit_cannot_replace_the_required_host_commands(self):
        original = copy.deepcopy(self.gate["host"])
        choices = ((0, ["/bin/true"]), (1, ["/bin/true"]),
                   (0, original[1]["command"]), (1, original[0]["command"]))
        for number, (index, command) in enumerate(choices):
            with self.subTest(index=index, command=command):
                self.destination = self.root / f"docs/verification/full-release-host-command-{number}"
                self.gate["host"] = copy.deepcopy(original)
                self.gate["host"][index]["command"] = command
                self.save_gate()
                self.rejected()

    def test_input_file_change_after_initial_validation_cannot_escape_snapshot_check(self):
        original = capture.Snapshot.stable
        calls = 0
        def change(snapshot):
            nonlocal calls
            original(snapshot)
            if calls == 0:
                (self.runs[0] / "serial.txt").write_bytes(b"changed after validation")
            calls += 1
        with patch.object(capture.Snapshot, "stable", change):
            self.rejected()

    def test_source_change_during_archive_removes_only_new_output(self):
        with patch.object(capture, "source_hash", side_effect=[self.source, self.source, "f" * 64]):
            self.rejected()
        self.assertTrue((self.build / "system-gate.json").exists())

    def test_test_input_change_during_validation_is_rejected(self):
        with patch.object(capture, "inputs", side_effect=[self.test_inputs, {}]):
            self.rejected()

    def test_overwrite_existing_directory_file_and_broken_symlink_is_rejected(self):
        for kind in ("directory", "file", "symlink"):
            with self.subTest(kind=kind):
                if kind == "directory":
                    self.destination.mkdir()
                elif kind == "file":
                    self.destination.write_bytes(b"keep this user content")
                else:
                    self.destination.symlink_to(self.root / "missing")
                with self.assertRaises(FileExistsError):
                    capture.capture(self.destination)
                if kind == "directory":
                    self.destination.rmdir()
                else:
                    self.destination.unlink()

    def test_path_escape_symlink_hardlink_or_external_logs_are_rejected(self):
        with self.assertRaises(ValueError):
            capture.capture(self.root / "elsewhere/full-release-test")
        original = self.gate["host"][0]["stdout"]
        self.gate["host"][0]["stdout"] = str(self.root / "personal-file")
        self.save_gate()
        self.rejected()
        self.gate["host"][0]["stdout"] = original
        self.save_gate()
        path = self.runs[0] / "serial.txt"
        text = path.read_bytes()
        other = self.root / "other-serial.txt"
        other.write_bytes(text)
        path.unlink()
        path.symlink_to(other)
        self.rejected()
        path.unlink()
        path.write_bytes(text)
        import os
        os.link(path, self.root / "hard-serial.txt")
        self.rejected()

    def test_duplicate_json_keys_are_not_silently_overwritten(self):
        path = self.build / "system-gate.json"
        raw = path.read_bytes()
        path.write_bytes(raw.replace(b'"passed": true', b'"passed": false, "passed": true', 1))
        self.rejected()

    def prepare_network(self):
        p = patch.object(capture, "network_inputs", return_value=self.test_inputs)
        p.start()
        self.addCleanup(p.stop)
        machine = completed_fixture()
        peer = dict(connected=True, error=None, rejected_frame=None,
                    counts=machine.counts, events=machine.events)
        for index, report in enumerate(self.reports):
            case = report["case"]
            report["kernel_feature"] = capture.NETWORK_FEATURE[case]
            report["manifest"]["kernel_feature"] = report["kernel_feature"]
            report["manifest"]["system"]["net_probe"] = dict(sha256="f" * 64)
            report["command"] += network_arguments(31337)
            report["network_gate"] = True
            report["network_peer"] = copy.deepcopy(peer)
            serial = (self.runs[index] / "serial.txt").read_text()
            serial += "".join(f"SYSTEM:{name}:{value:020d}\n" for name, value in NET_EXACT.items())
            (self.runs[index] / "serial.txt").write_text(serial)
            report["evidence"] = capture.evaluate(case, serial, 33, False)
            self.gate["stages"][index]["network_gate"] = True
            put_json(self.runs[index] / "network-peer.json", peer)
            self.save_report(index)
        put_json(self.volume / "last-system-stage.json", self.reports[-1])
        put_json(self.build / "manifest.json", self.manifests[0])
        commands = [
            [sys.executable, "-B", "-m", "unittest", "discover", "-s", "tests", "-p", "test_net_*.py"],
            ["cargo", "+1.90.0", "test", "--locked", "-p", "capability-core"],
            ["cargo", "+1.90.0", "test", "--locked", "--manifest-path", "sdk/Cargo.toml", "-p", "nieuw-os-sdk-runtime"],
        ]
        self.gate["host"] = []
        for index, command in enumerate(commands):
            host = dict(command=command, exit=0)
            for stream in ("stdout", "stderr"):
                path = self.build / "logs" / f"net-host-{index}.{stream}.txt"
                path.write_text("host-only network fixture\n")
                host[stream] = str(path)
            self.gate["host"].append(host)
        self.save_network_gate()

    def save_network_gate(self):
        put_json(self.build / "net-system-gate.json", self.gate)

    def network_rejected(self):
        with self.assertRaises((ValueError, FileNotFoundError)):
            capture.capture(self.destination, network=True)
        self.assertFalse(self.destination.exists())

    def test_network_capture_rechecks_raw_guest_peer_and_exact_firmware_policy(self):
        self.prepare_network()
        archive = capture.capture(self.destination, network=True)
        self.assertTrue((archive / "net-system-gate.json").is_file())
        self.assertFalse((archive / "system-gate.json").exists())
        self.assertEqual((archive / "0-normal/network-peer.json").read_bytes(),
                         (self.runs[0] / "network-peer.json").read_bytes())
        for line in (archive / "SHA256SUMS").read_text().splitlines():
            value, name = line.split("  ", 1)
            self.assertEqual(hashlib.sha256((archive / name).read_bytes()).hexdigest(), value)

    def test_network_capture_rejects_partial_wrapper_or_peer_failure(self):
        self.prepare_network()
        report = self.reports[0]
        good = copy.deepcopy(report)
        for problem in ("stage", "report", "peer-error", "peer-rejected", "peer-differs"):
            self.reports[0] = report = copy.deepcopy(good)
            self.gate["stages"][0]["network_gate"] = problem != "stage"
            if problem == "report":
                report["network_gate"] = False
            if problem == "peer-error":
                report["network_peer"]["error"] = "ProtocolError: unsupported fixture EtherType"
            if problem == "peer-rejected":
                report["network_peer"]["rejected_frame"] = dict(ethertype="86dd")
            peer = copy.deepcopy(report["network_peer"])
            if problem == "peer-differs":
                peer["events"] = []
            put_json(self.runs[0] / "network-peer.json", peer)
            self.save_report(0)
            self.save_network_gate()
            self.network_rejected()

    def test_network_capture_requires_native_counters_and_independent_peer_counts(self):
        self.prepare_network()
        path = self.runs[0] / "serial.txt"
        before = path.read_text()
        changed = before.replace("SYSTEM:NET_ARP_REPLIES:00000000000000000001", "SYSTEM:NET_ARP_REPLIES:00000000000000000000")
        path.write_text(changed)
        self.reports[0]["evidence"] = capture.evaluate("normal", changed, 33, False)
        self.save_report(0)
        self.network_rejected()
        path.write_text(before)
        self.reports[0]["evidence"] = capture.evaluate("normal", before, 33, False)
        self.reports[0]["network_peer"]["counts"]["accepted_frames"] = 7
        put_json(self.runs[0] / "network-peer.json", self.reports[0]["network_peer"])
        self.save_report(0)
        self.network_rejected()

    def test_network_capture_rejects_enabled_firmware_or_public_peer(self):
        self.prepare_network()
        original = self.reports[0]["command"]
        for key, replacement in (("VirtioNetSupport,string=no", "VirtioNetSupport,string=yes"),
                                 ("127.0.0.1:31337", "0.0.0.0:31337"),
                                 ("IPv6Support,string=no", "IPv6Support,string=yes"),
                                 (",romfile=", "")):
            self.reports[0]["command"] = [arg.replace(key, replacement) for arg in original]
            self.save_report(0)
            self.network_rejected()

    def test_network_capture_requires_its_own_host_checks_and_test_inputs(self):
        self.prepare_network()
        commands = copy.deepcopy(self.gate["host"])
        self.gate["host"][1]["command"] = ["/bin/true"]
        self.save_network_gate()
        self.network_rejected()
        self.gate["host"] = commands
        self.gate["test_inputs"] = {}
        self.save_network_gate()
        self.network_rejected()

    def prepare_tcp(self):
        p = patch.object(capture, "tcp_inputs", return_value=self.test_inputs)
        p.start()
        self.addCleanup(p.stop)
        fixture_report, peer = tcp_fixture()
        for index, report in enumerate(self.reports):
            case = report["case"]
            report["kernel_feature"] = capture.TCP_FEATURE[case]
            report["manifest"]["kernel_feature"] = report["kernel_feature"]
            report["manifest"]["system"]["net_probe"] = copy.deepcopy(fixture_report["manifest"]["system"]["net_probe"])
            report["command"] += network_arguments(31337)
            report.update(network_gate=True, tcp_gate=True, network_peer=copy.deepcopy(peer))
            serial = (self.runs[index] / "serial.txt").read_text()
            serial += "".join(f"SYSTEM:{name}:{value:020d}\n" for name, value in fixture_report["evidence"].items())
            (self.runs[index] / "serial.txt").write_text(serial)
            report["evidence"] = capture.evaluate(case, serial, 33, False)
            self.gate["stages"][index].update(network_gate=True, tcp_gate=True)
            put_json(self.runs[index] / "network-peer.json", peer)
            self.save_report(index)
        put_json(self.volume / "last-system-stage.json", self.reports[-1])
        put_json(self.build / "manifest.json", self.manifests[0])
        self.gate["host"] = []
        for index, command in enumerate(capture.tcp_host_commands()):
            host = dict(command=command, exit=0)
            for stream in ("stdout", "stderr"):
                path = self.build / "logs" / f"tcp-host-{index}.{stream}.txt"
                path.write_text("host-only TCP fixture\n")
                host[stream] = str(path)
            self.gate["host"].append(host)
        self.save_tcp_gate()

    def save_tcp_gate(self):
        put_json(self.build / "tcp-system-gate.json", self.gate)

    def tcp_rejected(self):
        with self.assertRaises((ValueError, FileNotFoundError)):
            capture.capture(self.destination, tcp=True)
        self.assertFalse(self.destination.exists())

    def test_tcp_capture_revalidates_its_own_pair_and_preserves_exact_peer_bytes(self):
        self.prepare_tcp()
        archive = capture.capture(self.destination, tcp=True)
        self.assertTrue((archive / "tcp-system-gate.json").is_file())
        self.assertFalse((archive / "net-system-gate.json").exists())
        self.assertFalse((archive / "system-gate.json").exists())
        validation = json.loads((archive / "capture-validation.json").read_text())
        self.assertIn("native TCP", validation["boundary"])
        self.assertIn("not full release acceptance", validation["boundary"])
        for index, case in enumerate(("normal", "recover")):
            self.assertEqual((archive / f"{index}-{case}/network-peer.json").read_bytes(),
                             (self.runs[index] / "network-peer.json").read_bytes())
        for line in (archive / "SHA256SUMS").read_text().splitlines():
            value, name = line.split("  ", 1)
            self.assertEqual(hashlib.sha256((archive / name).read_bytes()).hexdigest(), value)

    def test_tcp_capture_requires_both_completion_flags_in_both_stages_and_reports(self):
        self.prepare_tcp()
        for index in range(2):
            for scope in ("stage", "report"):
                target = self.gate["stages"][index] if scope == "stage" else self.reports[index]
                for flag in ("network_gate", "tcp_gate"):
                    for value in (None, False, 1):
                        with self.subTest(index=index, scope=scope, flag=flag, value=value):
                            if value is None:
                                target.pop(flag, None)
                            else:
                                target[flag] = value
                            self.save_report(index)
                            self.save_tcp_gate()
                            self.tcp_rejected()
                            target[flag] = True
                self.save_report(index)
                self.save_tcp_gate()

    def test_tcp_capture_requires_raw_tcp_evidence_beyond_generic_system_success(self):
        self.prepare_tcp()
        path = self.runs[0] / "serial.txt"
        before = path.read_text()
        for name, value in (("TCP_HTTP", 0), ("TCP_TIMEOUT_TICKS", 299), ("NET_RX_COMPLETED", 0)):
            with self.subTest(name=name):
                old = self.reports[0]["evidence"][name]
                changed = before.replace(f"SYSTEM:{name}:{old:020d}", f"SYSTEM:{name}:{value:020d}")
                self.assertNotEqual(changed, before)
                path.write_text(changed)
                # Even a matching stored counter map does not satisfy TCP.
                self.reports[0]["evidence"] = capture.evaluate("normal", changed, 33, False)
                self.save_report(0)
                self.tcp_rejected()
        generic = transcript("normal")
        path.write_text(generic)
        self.reports[0]["evidence"] = capture.evaluate("normal", generic, 33, False)
        self.save_report(0)
        self.tcp_rejected()

    def test_tcp_capture_binds_the_independent_peer_response_and_close_evidence(self):
        self.prepare_tcp()
        good = copy.deepcopy(self.reports[0])
        for problem in ("mismatch", "response", "partial", "fin", "counts", "event"):
            with self.subTest(problem=problem):
                report = self.reports[0] = copy.deepcopy(good)
                peer = report["network_peer"]
                if problem == "response":
                    peer["response_sha256"] = "f" * 64
                elif problem == "partial":
                    peer["complete"] = False
                elif problem == "fin":
                    peer["cases"]["8080"]["closed"] = False
                elif problem == "counts":
                    peer["counts"]["valid_data_sent"] -= 1
                elif problem == "event":
                    peer["events"][0]["frame_sha256"] = "invalid"
                stored = copy.deepcopy(peer)
                if problem == "mismatch":
                    stored["response_sha256"] = "f" * 64
                put_json(self.runs[0] / "network-peer.json", stored)
                self.save_report(0)
                self.tcp_rejected()

    def test_tcp_capture_rejects_mixed_profiles_and_untested_normal_artifacts(self):
        self.prepare_tcp()
        original = copy.deepcopy(self.reports[0])
        for feature in (capture.FEATURE["normal"], capture.NETWORK_FEATURE["normal"], capture.TCP_FEATURE["recover"]):
            self.reports[0] = copy.deepcopy(original)
            self.reports[0]["kernel_feature"] = feature
            self.reports[0]["manifest"]["kernel_feature"] = feature
            self.save_report(0)
            self.tcp_rejected()
        self.reports[0] = original
        self.save_report(0)
        final = copy.deepcopy(self.manifests[0])
        final["kernel_feature"] = capture.NETWORK_FEATURE["normal"]
        put_json(self.build / "manifest.json", final)
        self.tcp_rejected()

    def test_tcp_capture_requires_its_five_exact_host_commands_slots_and_inputs(self):
        self.prepare_tcp()
        good = copy.deepcopy(self.gate)
        for problem in ("missing", "command", "network-command", "log-slot", "inputs", "exit"):
            self.gate = copy.deepcopy(good)
            if problem == "missing":
                self.gate["host"].pop()
            elif problem == "command":
                self.gate["host"][2]["command"] = ["/bin/true"]
            elif problem == "network-command":
                self.gate["host"][0]["command"][-1] = "test_net_*.py"
            elif problem == "log-slot":
                self.gate["host"][0]["stdout"] = str(self.build / "logs/system-host-0.stdout.txt")
            elif problem == "inputs":
                self.gate["test_inputs"] = {}
            else:
                self.gate["host"][4]["exit"] = True
            with self.subTest(problem=problem):
                self.save_tcp_gate()
                self.tcp_rejected()

    def test_tcp_capture_preserves_readonly_and_firmware_isolation_guards(self):
        self.prepare_tcp()
        original = copy.deepcopy(self.reports[1])
        for needle, replacement in (("readonly=on,file=build/run/system-image-", "readonly=off,file=build/run/system-image-"),
                                    ("IPv6Support,string=no", "IPv6Support,string=yes"),
                                    ("127.0.0.1:31337", "0.0.0.0:31337")):
            self.reports[1] = copy.deepcopy(original)
            self.reports[1]["command"] = [arg.replace(needle, replacement) for arg in original["command"]]
            self.save_report(1)
            with self.subTest(needle=needle):
                self.tcp_rejected()

    def test_tcp_and_network_capture_flags_are_mutually_exclusive(self):
        with self.assertRaises(ValueError):
            capture.capture(self.destination, network=True, tcp=True)
        with patch("sys.stderr"):
            with self.assertRaises(SystemExit) as error:
                capture.main(["--destination", str(self.destination), "--network", "--tcp"])
        self.assertEqual(error.exception.code, 2)
        self.assertFalse(self.destination.exists())

    def test_base_gate_freezes_the_imported_tcp_wire_fixture_sources(self):
        paths = ("tests/test_system_capture.py", "tests/test_net_peer.py", "tests/test_tcp_harness.py",
                 "sdk/examples/tcp-probe/test_peer.py")
        for relative in paths:
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("host-only imported fixture\n")
        with patch.object(system_gate, "ROOT", self.root):
            original = system_gate.inputs()
            self.assertEqual(set(original), set(paths))
            (self.root / paths[2]).write_text("changed TCP validator fixture\n")
            self.assertNotEqual(system_gate.inputs(), original)
            original = system_gate.inputs()
            (self.root / paths[3]).write_text("changed TCP wire fixture\n")
            self.assertNotEqual(system_gate.inputs(), original)


if __name__ == "__main__":
    unittest.main()
