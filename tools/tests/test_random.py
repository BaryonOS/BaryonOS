"""Host counterexamples only. Synthetic fixtures are never VM/RNG evidence."""
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_random as r
import run_fpu as guarded
import volume_disk as disk


def transcript(case="raw", changes=None):
    base = dict(APP_READY=2, APP_DOCS=1, APP_REVISION=1, APP_BYTES=4096, APP_DENIED=3,
                APP_CONTENT_SUM=disk.fnv(disk.fixture(0, 1, 4096)), WORKER_SPAWNS=2,
                WORKER_FAULTS=1, WORKER_RESTARTS=1, WORKER_KILLS=1, LIVE_PROCESSES_AFTER=0,
                FRAMES_RECOVERED=1, DISK_READS=304, DISK_WRITES=42, DISK_FLUSHES=3,
                CRYPTO_SEAL=2, CRYPTO_UNSEAL=2, CRYPTO_TAMPER_DENIED=9, RECORD_SEAL=1,
                RECORD_UNSEAL=1, RECORD_CONTEXT_DENIED=10, RECORD_REVOKED_DENIED=2)
    values = dict.fromkeys(r.FIELDS, 0)
    values.update(LEGACY_STATUS=1, TERMINAL_TICKS=410, STATUS_BEGIN_TICKS=411, STATUS_END_TICKS=415,
                  LEGACY_POLLS_AFTER=8, PROBE_PID=47, KEY_PID=12, APP_PID=33,
                  EXPECTED_CALLER_PID=33 if case == "wrong-caller" else 47, CASE=r.CASES.index(case),
                  CLOCK_READS=1000, START_TICKS=4, END_TICKS=430)
    if case == "raw":
        values.update(SUCCESSES=89, BYTES=4737, VARIABLES=5, LEGACY_DENIED=9, INVALID_DENIED=9,
                      QUOTA_DENIED=1, ENQUEUED=110, REBINDS=1, STALE_DENIED=1, REPLAY_DENIED=1,
                      REVOKED_DENIED=1, REPLIES=109, CAP_DENIED=6)
        service = [90, 4769, 3, 110, 0, 9, 1, 0, 9, 109, 1, 0]
    elif case == "sdk":
        values.update(ENQUEUED=5, SDK_BYTES=225, SDK_SENDS=5, SDK_CLOSES=2, CANCELLED=1,
                      NO_RETRY=1, SDK_POSITIVES=4)
        service = [5, 257, 1, 5, 0, 0, 1, 0, 0, 4, 1, 0]
    elif case == "unknown":
        values.update(ENQUEUED=1, SDK_SENDS=1, SDK_CLOSES=2, UNKNOWN=1, NO_RETRY=1)
        service = [1, 64, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0]
    else:
        values.update(ENQUEUED=1, PID_DENIED=1)
        service = [0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0]
    values.update(zip(r.SERVICE_FIELDS, service))
    values.update(changes or {})
    base["ENTROPY_BYTES"] = 1024 + 256 * values["SEEDS"]
    lines = ["BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "SYSTEM:MANIFEST:PASS"]
    lines += [f"SYSTEM:RANDOM_{name}:{values[name]:020d}" for name in r.FIELDS]
    lines += ["SYSTEM:RANDOM:PASS", "SYSTEM:BROKERS:PASS", "SYSTEM:CLEANUP:PASS",
              "SYSTEM:CASE:NORMAL:PASS", "M1:PASS"]
    lines += [f"SYSTEM:{name}:{value:020d}" for name, value in base.items()]
    return "\n".join(lines) + "\n"


class SequenceTests(unittest.TestCase):
    def test_four_distinct_profiles_and_both_legal_cancel_races(self):
        for case in r.CASES:
            self.assertEqual(r.validate_sequence(transcript(case), case, 33, False)["RANDOM_CASE"], r.CASES.index(case))
        for case, calls in (("sdk", 5), ("unknown", 1)):
            text = transcript(case, dict(RETIRED=0, REPLY_CLOSED=0, REPLY_ENQUEUED=calls))
            r.validate_sequence(text, case, 33, False)

    def test_timeout_or_silence_is_not_actual_wrong_caller_rejection(self):
        for raw, timeout in ((33, True), (None, True), (0, False), (True, False)):
            with self.subTest(raw=raw, timeout=timeout), self.assertRaises(ValueError):
                r.validate_sequence(transcript("wrong-caller"), "wrong-caller", raw, timeout)
        for change in (dict(INGRESS_SEEN=0), dict(BAD_CALLER=0), dict(SERVICE_CALLS=1),
                       dict(SERVICE_BYTES=32), dict(SEEDS=1), dict(REPLY_ENQUEUED=1)):
            with self.subTest(change=change), self.assertRaises(ValueError):
                r.validate_sequence(transcript("wrong-caller", change), "wrong-caller", 33, False)

    def test_actual_pid_binding_cannot_mix_callers_or_reused_slots(self):
        for case in r.CASES:
            for change in (dict(PROBE_PID=0), dict(PROBE_PID=12), dict(KEY_PID=33),
                           dict(EXPECTED_CALLER_PID=47 if case == "wrong-caller" else 33)):
                with self.subTest(case=case, change=change), self.assertRaises(ValueError):
                    r.validate_sequence(transcript(case, change), case, 33, False)

    def test_legacy_status_must_follow_terminal_rng_and_precede_probe_exit(self):
        for change in (dict(LEGACY_STATUS=0), dict(STATUS_BEGIN_TICKS=409), dict(STATUS_END_TICKS=410),
                       dict(END_TICKS=414), dict(LEGACY_POLLS_AFTER=0), dict(START_TICKS=412),
                       dict(CLOCK_READS=0), dict(END_TICKS=99999)):
            with self.subTest(change=change), self.assertRaises(ValueError):
                r.validate_sequence(transcript(changes=change), "raw", 33, False)
        with self.assertRaises(ValueError):
            r.validate_sequence(transcript("wrong-caller", dict(END_TICKS=420)), "wrong-caller", 33, False)

    def test_missing_duplicate_reordered_unknown_and_overflow_counters_fail(self):
        good = transcript()
        line = "SYSTEM:RANDOM_SEEDS:00000000000000000003"
        for text in (good.replace(line+"\n", ""), good+line+"\n", good.replace(line, "SYSTEM:RANDOM_SEEDS:3"),
                     good.replace(line, "SYSTEM:RANDOM_SEEDS:18446744073709551616"),
                     good.replace(line, "SYSTEM:RANDOM_UNKNOWN_FIELD:00000000000000000003"),
                     good.replace("SYSTEM:RANDOM:PASS", "SYSTEM:RANDOM:FAIL"),
                     good.replace("SYSTEM:MANIFEST:PASS", "SYSTEM:BROKERS:PASS")):
            with self.subTest(text=text[:80]), self.assertRaises(ValueError):
                r.validate_sequence(text, "raw", 33, False)
        with self.assertRaises(ValueError):
            r.validate_sequence(good, "sdk", 33, False)

    def test_any_delivery_output_reseed_or_denial_cannot_be_omitted(self):
        for name in r.APP_FIELDS + r.SERVICE_FIELDS:
            original = r.validate_sequence(transcript(), "raw", 33, False)["RANDOM_" + name]
            with self.subTest(name=name), self.assertRaises(ValueError):
                r.validate_sequence(transcript(changes={name: original + 1}), "raw", 33, False)
        good = transcript()
        with self.assertRaises(ValueError):
            r.validate_sequence(good.replace("SYSTEM:ENTROPY_BYTES:00000000000000001792",
                                            "SYSTEM:ENTROPY_BYTES:00000000000000001024"), "raw", 33, False)

    def test_uncertain_output_or_automatic_retry_never_counts_as_cancellation(self):
        for case in ("sdk", "unknown"):
            for change in (dict(SDK_CLOSES=1), dict(SDK_SENDS=9), dict(NO_RETRY=0),
                           dict(PENDING=1), dict(REPLY_TIMEOUT=1), dict(SDK_BYTES=999)):
                with self.subTest(case=case, change=change), self.assertRaises(ValueError):
                    r.validate_sequence(transcript(case, change), case, 33, False)


def elf(label: str) -> bytes:
    data = bytearray(8193)
    data[:7] = b"\x7fELF\x02\x01\x01"
    struct.pack_into("<HHIQQQIHHH", data, 16, 2, 62, 1, 0x40000000, 64, 0, 0, 64, 56, 2)
    struct.pack_into("<IIQQQQQQ", data, 64, 1, 6, 4096, 0x40010000, 0x40010000, 1024, 1024, 4096)
    struct.pack_into("<IIQQQQQQ", data, 120, 1, 5, 8192, 0x40000000, 0x40000000, 1, 1, 4096)
    data[8192] = 0xc3
    return bytes(data) + label.encode()


class ArtifactTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name) / "checkout"
        self.build, self.target = self.root / "build", self.root / "target"
        self.run = self.build / "run" / ("system-normal-" + "1" * 32)
        self.volume = self.build / "run" / ("system-image-" + "2" * 32)
        self.run.mkdir(parents=True)
        self.volume.mkdir()
        for module, name, value in ((r,"ROOT",self.root),(r,"BUILD",self.build),(r,"TARGET",self.target),
                                    (guarded,"ROOT",self.root)):
            change=patch.object(module,name,value);change.start();self.addCleanup(change.stop)
        change=patch.object(r,"qemu_machine",return_value="pc-q35-8.2");change.start();self.addCleanup(change.stop)
        programs={};kernel=b"synthetic host fixture, NOT a native result\n"
        for name,(directory,filename) in r.PROGRAMS.items():
            path=self.target/directory/"x86_64-unknown-none/release"/filename
            path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(elf(name))
            programs[name]={"sha256":r.sha256(path),"elf":r.inspect_user(path)};kernel+=path.read_bytes()
        self.manifest=dict(schema_version=1,source_fingerprint_version=3,source_sha256="a"*64,
                           loader_feature=None,kernel_feature=r.FEATURE["raw"],system=programs,
                           random_probe=copy.deepcopy(programs["random_probe"]),
                           kernel={"sha256":r.digest(kernel)},loader={"sha256":r.digest(b"fixture-loader")})
        for esp in (self.build/"esp",self.run/"esp"):
            (esp/"EFI/BOOT").mkdir(parents=True);(esp/"kernel.elf").write_bytes(kernel)
            (esp/"EFI/BOOT/BOOTX64.EFI").write_bytes(b"fixture-loader")
        self.json(self.build/"manifest.json",self.manifest)
        (self.run/"OVMF_CODE.fd").write_bytes(b"fixture-code")
        (self.run/"OVMF_VARS.fd").write_bytes(b"fixture-vars-after-run")
        initial=disk.create_image(self.volume/"volume.img");data=bytearray((self.volume/"volume.img").read_bytes())
        content=disk.fixture(0,1,4096);start=(disk.FIRST+disk.DATA_START)*disk.SECTOR;data[start:start+4096]=content
        entries=[disk.empty_entry()for _ in range(disk.OBJECTS)]
        entries[0]=dict(disk.empty_entry(),revision=1,request_id=3,created=1,modified=1,present=True,
                        current=dict(revision=1,length=4096,name=b"systeemproef",content_sum=disk.fnv(content),
                                     chunks=[0],sums=[disk.fnv(content)]))
        for i,sector in enumerate(disk.encode_table(1,3,entries)):
            start=(disk.FIRST+disk.TABLE_SECTORS+i)*disk.SECTOR;data[start:start+disk.SECTOR]=sector
        (self.volume/"volume.img").write_bytes(data);(self.run/"volume.after.img").write_bytes(data)
        serial=transcript();(self.run/"serial.txt").write_text(serial)
        self.firmware={"code":r.digest(b"fixture-code"),"vars_template":"d"*64}
        relative=self.run.relative_to(self.root).as_posix();image=(self.volume/"volume.img").relative_to(self.root).as_posix()
        self.command=["/fixture/qemu-system-x86_64","-machine","pc-q35-8.2","-accel","tcg,thread=single",
                      "-cpu","qemu64","-smp","1","-m","256M","-nodefaults","-display","none",
                      "-monitor","none","-serial","stdio","-nic","none","-no-reboot",
                      "-drive",f"if=pflash,unit=0,format=raw,readonly=on,file={relative}/OVMF_CODE.fd",
                      "-drive",f"if=pflash,unit=1,format=raw,file={relative}/OVMF_VARS.fd",
                      "-drive",f"if=virtio,index=0,format=raw,readonly=on,file=fat:{relative}/esp",
                      "-device","isa-debug-exit,iobase=0xf4,iosize=0x04",
                      "-drive",f"if=none,id=volume,format=raw,cache=writeback,readonly=off,file={image}",
                      "-device","virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-SYSTEM-TEST",
                      "-object","rng-random,id=rng0,filename=/dev/urandom",
                      "-device","virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000"]
        self.report=dict(schema_version=1,case="normal",random_case="raw",random_gate=False,random_qemu_sha256="e"*64,
                         passed=True,kernel_feature=r.FEATURE["raw"],source_fingerprint_version=3,
                         source_sha256="a"*64,source_commit="b"*40,timeout_s=180,duration_s=1.0,
                         manifest=self.manifest,run_directory=str(self.run),volume_directory=str(self.volume),
                         command=self.command,qemu_version="QEMU emulator version fixture",firmware_sha256=self.firmware,
                         esp_sha256={"esp/kernel.elf":r.digest(kernel),"esp/EFI/BOOT/BOOTX64.EFI":r.digest(b"fixture-loader")},
                         before_sha256=initial["sha256"],after_sha256=r.digest(data),oracle=disk.analyse(bytes(data)),
                         raw_qemu_exit=33,timed_out=False,evidence=r.validate_sequence(serial,"raw",33,False))

    @staticmethod
    def json(path,value):path.write_text(json.dumps(value))
    def validate(self,report=None):
        return r.validate_artifacts(report or self.report,"a"*64,"raw",qemu=self.command[0],
                                    qemu_sha256="e"*64,firmware_hashes=self.firmware)

    def test_complete_synthetic_artifacts_bind_serial_elf_and_disk(self):
        self.validate()

    def test_copied_pass_flag_cannot_hide_false_capture_or_wrong_profile(self):
        for change in (dict(source_sha256="c"*64),dict(source_fingerprint_version=2),dict(kernel_feature="native-system"),
                       dict(case="recover"),dict(passed=1),dict(schema_version=True),dict(timeout_s=True),dict(duration_s=float("nan")),
                       dict(evidence={}),dict(timed_out=True),dict(raw_qemu_exit=0),
                       dict(random_case="sdk"),dict(random_gate=1),dict(random_qemu_sha256="f"*64)):
            with self.subTest(change=change),self.assertRaises(ValueError):self.validate(self.report|change)

    def test_command_snapshot_network_or_wrong_disk_is_rejected(self):
        for command in ([*self.command,"-loadvm","old"],[*self.command,"-netdev","user,id=public"],
                        ["/bin/true",*self.command[1:]],
                        [arg.replace("readonly=off","readonly=on")for arg in self.command]):
            with self.subTest(command=command),self.assertRaises(ValueError):self.validate(self.report|dict(command=command))

    def test_stale_serial_or_wrong_initial_snapshot_never_uses_saved_counters(self):
        serial=self.run/"serial.txt";original=serial.read_bytes();serial.write_bytes(original+b"SYSTEM:RANDOM:PASS\n")
        with self.assertRaises(ValueError):self.validate()
        serial.write_bytes(original)
        with self.assertRaises(ValueError):self.validate(self.report|dict(before_sha256=self.report["after_sha256"]))

    def test_changed_esp_loader_or_embedded_program_is_rejected(self):
        for path in (self.run/"esp/kernel.elf",self.build/"esp/EFI/BOOT/BOOTX64.EFI",
                     self.target/"random-probe-app/x86_64-unknown-none/release/random-probe"):
            original=path.read_bytes();path.write_bytes(original+b"changed")
            with self.subTest(path=path),self.assertRaises(ValueError):self.validate()
            path.write_bytes(original)
        self.report["manifest"]["system"]["random_probe"]["elf"]["entry"]+=1
        self.report["manifest"]["random_probe"]=copy.deepcopy(self.report["manifest"]["system"]["random_probe"])
        self.json(self.build/"manifest.json",self.report["manifest"])
        with self.assertRaises(ValueError):self.validate()

    def test_manifest_alias_and_actual_six_program_set_cannot_contradict(self):
        for change in (dict(random_probe={}),dict(system={}),dict(schema_version=True),dict(loader_feature="framebuffer")):
            with self.subTest(change=change),self.assertRaises(ValueError):
                r.validate_manifest(self.manifest|change,"a"*64,"raw")

    def test_true_disk_bytes_override_ui_or_oracle_success(self):
        image=self.volume/"volume.img";raw=bytearray(image.read_bytes());raw[(disk.FIRST+disk.DATA_START)*disk.SECTOR]^=1
        image.write_bytes(raw)
        with self.assertRaises(ValueError):self.validate()
        self.report["after_sha256"]=r.digest(raw);(self.run/"volume.after.img").write_bytes(raw)
        with self.assertRaises(ValueError):self.validate()

    def test_linked_evidence_rejected_without_touching_external_content(self):
        for kind in ("symlink","hardlink"):
            path=self.run/"serial.txt";original=path.read_bytes();outside=self.root.parent/"outside"
            outside.write_bytes(original);path.unlink()
            if kind=="symlink":path.symlink_to(outside)
            else:os.link(outside,path)
            with self.subTest(kind=kind),self.assertRaises(ValueError):self.validate()
            self.assertEqual(outside.read_bytes(),original);path.unlink();path.write_bytes(original);outside.unlink()

    def test_bad_existing_report_is_rejected_before_vm_or_external_write(self):
        destination=self.build/"random-report.json";outside=self.root.parent/"outside";outside.write_bytes(b"preserve")
        os.link(outside,destination)
        with patch.object(r,"source_hash")as source,patch.object(r,"run_system")as guest,self.assertRaises(ValueError):
            r.run_random("raw")
        source.assert_not_called();guest.assert_not_called();self.assertEqual(outside.read_bytes(),b"preserve")

    def test_generic_pass_cannot_publish_rng_pass_after_capture_error_or_source_drift(self):
        qemu=self.root/"qemu-system-x86_64";qemu.write_bytes(b"synthetic executable identity")
        code=self.root/"code.fd";code.write_bytes(b"fixture-code")
        variables=self.root/"vars.fd";variables.write_bytes(b"fixture-vars-template")
        for mode in ("bad-capture","source-drift"):
            report=copy.deepcopy(self.report)
            with self.subTest(mode=mode),patch.object(r,"environment",return_value={"PATH":""}), \
                    patch.object(r.shutil,"which",return_value=str(qemu)), \
                    patch.object(r,"firmware",return_value=(code,variables)), \
                    patch.object(r,"source_hash",side_effect=["a"*64,"c"*64])as sources, \
                    patch.object(r,"run_system",return_value=(report,self.volume))as guest, \
                    patch.object(r,"validate_artifacts",side_effect=ValueError("changed serial")if mode=="bad-capture"else None):
                result,_=r.run_random("raw")
                self.assertIs(result["passed"],False)
                self.assertIs(result["random_gate"],False)
                guest.assert_called_once_with("normal",timeout=180,kernel_feature=r.FEATURE["raw"])
                self.assertEqual(sources.call_count,1 if mode=="bad-capture"else 2)
                stored=json.loads((self.build/"random-report.json").read_text())
                self.assertIs(stored["random_gate"],False)

    def test_source_esp_refuses_extra_file_hardlink_and_directory_link_before_copy(self):
        extra=self.build/"esp/extra.bin";extra.write_bytes(b"unexpected file")
        with self.assertRaises(ValueError):r.preflight_esp(self.manifest)
        extra.unlink()
        kernel=self.build/"esp/kernel.elf";original=kernel.read_bytes()
        outside=self.root.parent/"same-kernel-marker";outside.write_bytes(original)
        kernel.unlink();os.link(outside,kernel)
        with self.assertRaises(ValueError):r.preflight_esp(self.manifest)
        kernel.unlink();outside.unlink();kernel.write_bytes(original)
        outside_dir=self.root.parent/"private-dir";outside_dir.mkdir()
        (outside_dir/"keep-marker").write_bytes(b"do not copy")
        linked=self.build/"esp/linked-dir";linked.symlink_to(outside_dir,target_is_directory=True)
        with self.assertRaises(ValueError):r.preflight_esp(self.manifest)
        self.assertEqual((outside_dir/"keep-marker").read_bytes(),b"do not copy")
        linked.unlink()
        r.preflight_esp(self.manifest)

    def test_preflight_failure_replaces_old_green_index_with_failed_attempt(self):
        index=self.build/"random-report.json"
        index.write_text(json.dumps(dict(passed=True,random_gate=True)))
        self.json(self.build/"manifest.json",self.manifest|dict(loader_feature="wrong-profile"))
        with patch.object(r,"source_hash",return_value="a"*64),patch.object(r,"run_system")as guest:
            with self.assertRaises(ValueError):r.run_random("raw")
            guest.assert_not_called()
        failed=json.loads(index.read_text())
        self.assertIs(failed["passed"],False);self.assertIs(failed["random_gate"],False)
        self.assertEqual(failed["status"],"failed")

    def test_delegate_exception_replaces_old_green_index_and_propagates_failure(self):
        index=self.build/"random-report.json"
        index.write_text(json.dumps(dict(passed=True,random_gate=True)))
        qemu=self.root/"qemu-system-x86_64";qemu.write_bytes(b"synthetic emulator marker")
        code=self.root/"code.fd";code.write_bytes(b"fixture-code")
        variables=self.root/"vars.fd";variables.write_bytes(b"fixture-vars-template")
        with patch.object(r,"source_hash",return_value="a"*64),patch.object(r,"environment",return_value={"PATH":""}),                 patch.object(r.shutil,"which",return_value=str(qemu)),patch.object(r,"firmware",return_value=(code,variables)),                 patch.object(r,"run_system",side_effect=RuntimeError("controlled launch failure"))as guest:
            with self.assertRaisesRegex(RuntimeError,"controlled launch failure"):r.run_random("raw")
            self.assertEqual(guest.call_count,1)
        failed=json.loads(index.read_text())
        self.assertIs(failed["passed"],False);self.assertIs(failed["random_gate"],False)
        self.assertEqual(failed["status"],"failed")

    def test_source_esp_external_link_is_refused_before_delegate_or_marker_exposure(self):
        outside=self.root.parent/"keep-marker";outside.write_bytes(b"private synthetic sentinel")
        (self.build/"esp/unknown.bin").symlink_to(outside)
        qemu=self.root/"qemu-system-x86_64";qemu.write_bytes(b"synthetic emulator marker")
        code=self.root/"code.fd";code.write_bytes(b"fixture-code")
        variables=self.root/"vars.fd";variables.write_bytes(b"fixture-vars-template")
        with patch.object(r,"source_hash",return_value="a"*64),patch.object(r,"environment",return_value={"PATH":""}),                 patch.object(r.shutil,"which",return_value=str(qemu)),patch.object(r,"firmware",return_value=(code,variables)),                 patch.object(r,"run_system",side_effect=RuntimeError("delegate reached too early"))as guest:
            with self.assertRaises(ValueError):r.run_random("raw")
            guest.assert_not_called()
        self.assertEqual(outside.read_bytes(),b"private synthetic sentinel")


if __name__ == "__main__":unittest.main()
