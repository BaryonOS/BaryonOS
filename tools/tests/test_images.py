"""Host verifier counterexamples only; none of these fixtures is a VM log."""
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
import run_images as r
import run_fpu as guarded
import volume_disk as disk


def transcript(linked_elf=b"host fixture ELF bytes", changes=None):
    base = dict(APP_READY=2, APP_DOCS=1, APP_REVISION=1, APP_BYTES=4096, APP_DENIED=3,
                APP_CONTENT_SUM=disk.fnv(disk.fixture(0, 1, 4096)), WORKER_SPAWNS=2,
                WORKER_FAULTS=1, WORKER_RESTARTS=1, WORKER_KILLS=1, LIVE_PROCESSES_AFTER=0,
                FRAMES_RECOVERED=1, DISK_READS=304, DISK_WRITES=42, DISK_FLUSHES=3,
                CRYPTO_SEAL=2, CRYPTO_UNSEAL=2, CRYPTO_TAMPER_DENIED=9, RECORD_SEAL=1,
                RECORD_UNSEAL=1, RECORD_CONTEXT_DENIED=10, RECORD_REVOKED_DENIED=2,
                ENTROPY_BYTES=1024)
    values = dict(r.EXACT, PID=47, ELF_BYTES=len(linked_elf), ELF_FNV64=r.fnv64(linked_elf),
                  ARENA_PEAK=1234567, START_TICKS=4, END_TICKS=17, EXIT_TICKS=18)
    values.update(changes or {})
    lines = ["BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "SYSTEM:IMAGE_MANIFEST_PID:00000000000000000047",
             "SYSTEM:MANIFEST:PASS"]
    lines += [f"SYSTEM:IMAGE_{name}:{values[name]:020d}" for name in r.FIELDS]
    lines += ["SYSTEM:IMAGES:PASS", "SYSTEM:BROKERS:PASS", "SYSTEM:CLEANUP:PASS",
              "SYSTEM:CASE:NORMAL:PASS", "M1:PASS"]
    lines += [f"SYSTEM:{name}:{value:020d}" for name, value in base.items()]
    return "\n".join(lines) + "\n"


class SequenceTests(unittest.TestCase):
    def validate(self,text=None):
        return r.validate_sequence(text or transcript(),33,False,linked_elf=b"host fixture ELF bytes")

    def test_complete_image_scope_and_known_noncryptographic_fnv_vectors(self):
        self.assertEqual(self.validate()["PIXELS"],149647)
        self.assertEqual(r.fnv64(b""),0xcbf29ce484222325)
        self.assertEqual(r.fnv64(b"a"),0xaf63dc4c8601ec8c)
        self.assertEqual(r.fnv64(b"foobar"),0x85944171f73967e8)

    def test_actual_exit_fault_or_timeout_cannot_be_replaced_with_pass(self):
        for raw,timeout in ((33,True),(None,True),(0,False),(35,False),(True,False)):
            with self.subTest(raw=raw,timeout=timeout),self.assertRaises(ValueError):
                r.validate_sequence(transcript(),raw,timeout,linked_elf=b"host fixture ELF bytes")
        for marker in ("SYSTEM:IMAGE_FAIL:FAULT", "SYSTEM:IMAGE_FAULT_PID:00000000000000000047", "PANIC"):
            with self.subTest(marker=marker),self.assertRaises(ValueError):self.validate(transcript()+marker+"\n")

    def test_wrong_or_reused_pid_cannot_reuse_a_bootstrap_identity(self):
        for value in (0,1,48,(1<<64)-1):
            with self.subTest(value=value),self.assertRaises(ValueError):self.validate(transcript(changes=dict(PID=value)))
        for anchor in ("", "SYSTEM:IMAGE_MANIFEST_PID:00000000000000000000",
                       "SYSTEM:IMAGE_MANIFEST_PID:47", "SYSTEM:IMAGE_MANIFEST_PID:18446744073709551616"):
            bad=transcript().replace("SYSTEM:IMAGE_MANIFEST_PID:00000000000000000047",anchor)
            with self.subTest(anchor=anchor),self.assertRaises(ValueError):self.validate(bad)

    def test_exact_format_quality_pixel_negative_and_heap_stack_facts_remain_required(self):
        for name,value in r.EXACT.items():
            with self.subTest(name=name),self.assertRaises(ValueError):self.validate(transcript(changes={name:value+1}))
        for peak in (0,4194305):
            with self.subTest(peak=peak),self.assertRaises(ValueError):self.validate(transcript(changes=dict(ARENA_PEAK=peak)))

    def test_missing_duplicate_reordered_unknown_or_overflow_evidence_is_rejected(self):
        good=transcript();line="SYSTEM:IMAGE_PNG:00000000000000000008"
        bad=(good.replace(line+"\n",""),good+line+"\n",good.replace(line,"SYSTEM:IMAGE_PNG:8"),
             good.replace(line,"SYSTEM:IMAGE_PNG:18446744073709551616"),
             good.replace(line,"SYSTEM:IMAGE_UNKNOWN:00000000000000000008"),
             good.replace("SYSTEM:IMAGES:PASS","SYSTEM:IMAGE_FAIL:PIXELS"),
             good.replace("SYSTEM:IMAGE_MANIFEST_PID:00000000000000000047\nSYSTEM:MANIFEST:PASS",
                          "SYSTEM:MANIFEST:PASS\nSYSTEM:IMAGE_MANIFEST_PID:00000000000000000047"),
             good.replace("SYSTEM:IMAGE_MANIFEST_PID:00000000000000000047\n","")
                 +"SYSTEM:IMAGE_MANIFEST_PID:00000000000000000047\n")
        for text in bad:
            with self.subTest(text=text[:80]),self.assertRaises(ValueError):self.validate(text)

    def test_ticks_and_real_linked_elf_drift_cannot_reuse_a_prior_pixel_pass(self):
        for change in (dict(START_TICKS=18),dict(END_TICKS=19),dict(EXIT_TICKS=20004),
                       dict(ELF_BYTES=1),dict(ELF_FNV64=0)):
            with self.subTest(change=change),self.assertRaises(ValueError):self.validate(transcript(changes=change))
        with self.assertRaises(ValueError):
            r.validate_sequence(transcript(),33,False,linked_elf=b"different actual ELF bytes")

    def test_original_system_crypto_storage_privilege_and_cleanup_cannot_disappear(self):
        good=transcript()
        for old,new in (("SYSTEM:CRYPTO_SEAL:00000000000000000002","SYSTEM:CRYPTO_SEAL:00000000000000000000"),
                        ("SYSTEM:APP_BYTES:00000000000000004096","SYSTEM:APP_BYTES:00000000000000000000"),
                        ("SYSTEM:FRAMES_RECOVERED:00000000000000000001","SYSTEM:FRAMES_RECOVERED:00000000000000000000"),
                        ("SYSTEM:ENTROPY_BYTES:00000000000000001024","SYSTEM:ENTROPY_BYTES:00000000000000001280")):
            with self.subTest(old=old),self.assertRaises(ValueError):self.validate(good.replace(old,new))


def elf(label: str) -> bytes:
    data=bytearray(8193);data[:7]=b"\x7fELF\x02\x01\x01"
    struct.pack_into("<HHIQQQIHHH",data,16,2,62,1,0x40000000,64,0,0,64,56,2)
    struct.pack_into("<IIQQQQQQ",data,64,1,6,4096,0x40010000,0x40010000,1024,1024,4096)
    struct.pack_into("<IIQQQQQQ",data,120,1,5,8192,0x40000000,0x40000000,1,1,4096)
    data[8192]=0xc3
    return bytes(data)+label.encode()


class ArtifactTests(unittest.TestCase):
    def setUp(self):
        temporary=tempfile.TemporaryDirectory();self.addCleanup(temporary.cleanup)
        self.root=Path(temporary.name)/"checkout";self.build,self.target=self.root/"build",self.root/"target"
        self.run=self.build/"run"/("system-normal-"+"1"*32)
        self.volume=self.build/"run"/("system-image-"+"2"*32)
        self.run.mkdir(parents=True);self.volume.mkdir()
        for module,name,value in ((r,"ROOT",self.root),(r,"BUILD",self.build),(r,"TARGET",self.target),(guarded,"ROOT",self.root)):
            change=patch.object(module,name,value);change.start();self.addCleanup(change.stop)
        change=patch.object(r,"qemu_machine",return_value="pc-q35-8.2");change.start();self.addCleanup(change.stop)
        programs={};kernel=b"synthetic host fixture, NOT a native result\n"
        for name,(directory,filename) in r.PROGRAMS.items():
            path=self.target/directory/"x86_64-unknown-none/release"/filename
            path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(elf(name))
            programs[name]={"sha256":r.sha256(path),"elf":r.inspect_user(path)};kernel+=path.read_bytes()
            if name=="image_probe":self.probe=path.read_bytes()
        self.manifest=dict(schema_version=1,source_fingerprint_version=3,source_sha256="a"*64,
                           loader_feature=None,kernel_feature=r.FEATURE,system=programs,
                           image_probe=copy.deepcopy(programs["image_probe"]),
                           kernel={"sha256":r.digest(kernel)},loader={"sha256":r.digest(b"fixture-loader")})
        for esp in (self.build/"esp",self.run/"esp"):
            (esp/"EFI/BOOT").mkdir(parents=True);(esp/"kernel.elf").write_bytes(kernel)
            (esp/"EFI/BOOT/BOOTX64.EFI").write_bytes(b"fixture-loader")
        self.json(self.build/"manifest.json",self.manifest)
        (self.run/"OVMF_CODE.fd").write_bytes(b"fixture-code");(self.run/"OVMF_VARS.fd").write_bytes(b"fixture-vars-after-run")
        initial=disk.create_image(self.volume/"volume.img");data=bytearray((self.volume/"volume.img").read_bytes())
        content=disk.fixture(0,1,4096);start=(disk.FIRST+disk.DATA_START)*disk.SECTOR;data[start:start+4096]=content
        entries=[disk.empty_entry()for _ in range(disk.OBJECTS)]
        entries[0]=dict(disk.empty_entry(),revision=1,request_id=3,created=1,modified=1,present=True,
                        current=dict(revision=1,length=4096,name=b"systeemproef",content_sum=disk.fnv(content),chunks=[0],sums=[disk.fnv(content)]))
        for i,sector in enumerate(disk.encode_table(1,3,entries)):
            start=(disk.FIRST+disk.TABLE_SECTORS+i)*disk.SECTOR;data[start:start+disk.SECTOR]=sector
        (self.volume/"volume.img").write_bytes(data);(self.run/"volume.after.img").write_bytes(data)
        serial=transcript(self.probe);(self.run/"serial.txt").write_text(serial)
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
        self.report=dict(schema_version=1,case="normal",image_case="normal",image_gate=False,image_qemu_sha256="e"*64,
                         passed=True,kernel_feature=r.FEATURE,source_fingerprint_version=3,source_sha256="a"*64,
                         source_commit="b"*40,timeout_s=180,duration_s=1.0,manifest=self.manifest,
                         run_directory=str(self.run),volume_directory=str(self.volume),command=self.command,
                         qemu_version="QEMU emulator version fixture",firmware_sha256=self.firmware,
                         esp_sha256={"esp/kernel.elf":r.digest(kernel),"esp/EFI/BOOT/BOOTX64.EFI":r.digest(b"fixture-loader")},
                         before_sha256=initial["sha256"],after_sha256=r.digest(data),oracle=disk.analyse(bytes(data)),
                         raw_qemu_exit=33,timed_out=False,evidence=r.evaluate("normal",serial,33,False))

    @staticmethod
    def json(path,value):path.write_text(json.dumps(value))
    def validate(self,report=None):
        return r.validate_artifacts(report or self.report,"a"*64,qemu=self.command[0],qemu_sha256="e"*64,firmware_hashes=self.firmware)

    def test_synthetic_model_binds_all_six_elfs_disk_and_actual_serial(self):
        self.assertEqual(self.validate()["ELF_FNV64"],r.fnv64(self.probe))

    def test_copied_success_or_stale_profile_source_and_counters_are_rejected(self):
        for change in (dict(source_sha256="c"*64),dict(source_fingerprint_version=2),dict(kernel_feature="native-system"),
                       dict(case="recover"),dict(passed=1),dict(schema_version=True),dict(timeout_s=True),
                       dict(duration_s=float("nan")),dict(evidence={}),dict(timed_out=True),dict(raw_qemu_exit=35),
                       dict(image_case="other"),dict(image_gate=1),dict(image_qemu_sha256="f"*64),dict(image_evidence={})):
            with self.subTest(change=change),self.assertRaises(ValueError):self.validate(self.report|change)

    def test_changed_elf_esp_or_stored_image_inspection_fails(self):
        for path in (self.run/"esp/kernel.elf",self.build/"esp/EFI/BOOT/BOOTX64.EFI",
                     self.target/"system-image-probe/x86_64-unknown-none/release/image-probe"):
            original=path.read_bytes();path.write_bytes(original+b"changed")
            with self.subTest(path=path),self.assertRaises(ValueError):self.validate()
            path.write_bytes(original)
        self.manifest["system"]["image_probe"]["elf"]["entry"]+=1
        self.manifest["image_probe"]=copy.deepcopy(self.manifest["system"]["image_probe"])
        self.json(self.build/"manifest.json",self.manifest)
        with self.assertRaises(ValueError):self.validate()

    def test_manifest_alias_and_actual_six_program_set_cannot_contradict(self):
        for change in (dict(image_probe={}),dict(system={}),dict(schema_version=True),dict(loader_feature="framebuffer")):
            with self.subTest(change=change),self.assertRaises(ValueError):r.validate_manifest(self.manifest|change,"a"*64)

    def test_snapshot_network_wrong_disk_and_host_success_command_are_rejected(self):
        for command in ([*self.command,"-loadvm","old"],[*self.command,"-netdev","user,id=public"],
                        ["/bin/true",*self.command[1:]],
                        [arg.replace("readonly=off","readonly=on")for arg in self.command]):
            with self.subTest(command=command),self.assertRaises(ValueError):self.validate(self.report|dict(command=command))

    def test_stale_pixel_result_or_oracle_cannot_hide_changed_serial_or_disk(self):
        serial=self.run/"serial.txt";original=serial.read_bytes();serial.write_bytes(original+b"SYSTEM:IMAGES:PASS\n")
        with self.assertRaises(ValueError):self.validate()
        serial.write_bytes(original)
        with self.assertRaises(ValueError):self.validate(self.report|dict(before_sha256=self.report["after_sha256"]))
        image=self.volume/"volume.img";data=bytearray(image.read_bytes());data[(disk.FIRST+disk.DATA_START)*disk.SECTOR]^=1
        image.write_bytes(data)
        with self.assertRaises(ValueError):self.validate()
        self.report["after_sha256"]=r.digest(data);(self.run/"volume.after.img").write_bytes(data)
        with self.assertRaises(ValueError):self.validate()

    def test_links_are_rejected_without_mutating_external_content(self):
        for kind in ("symlink","hardlink"):
            path=self.run/"serial.txt";original=path.read_bytes();outside=self.root.parent/"outside"
            outside.write_bytes(original);path.unlink()
            if kind=="symlink":path.symlink_to(outside)
            else:os.link(outside,path)
            with self.subTest(kind=kind),self.assertRaises(ValueError):self.validate()
            self.assertEqual(outside.read_bytes(),original);path.unlink();path.write_bytes(original);outside.unlink()

    def test_bad_existing_output_fails_before_any_vm_or_external_write(self):
        destination=self.build/"image-report.json";outside=self.root.parent/"outside";outside.write_bytes(b"preserve")
        os.link(outside,destination)
        with patch.object(r,"source_hash")as source,patch.object(r,"run_system")as guest,self.assertRaises(ValueError):r.run_images()
        source.assert_not_called();guest.assert_not_called();self.assertEqual(outside.read_bytes(),b"preserve")

    def test_generic_pass_cannot_authorize_after_capture_error_or_source_drift(self):
        qemu=self.root/"qemu-system-x86_64";qemu.write_bytes(b"synthetic executable identity")
        code=self.root/"code.fd";code.write_bytes(b"fixture-code")
        variables=self.root/"vars.fd";variables.write_bytes(b"fixture-vars-template")
        for mode in ("bad-capture","source-drift"):
            report=copy.deepcopy(self.report)
            with self.subTest(mode=mode),patch.object(r,"environment",return_value={"PATH":""}), \
                    patch.object(r.shutil,"which",return_value=str(qemu)),patch.object(r,"firmware",return_value=(code,variables)), \
                    patch.object(r,"source_hash",side_effect=["a"*64,"c"*64])as sources, \
                    patch.object(r,"run_system",return_value=(report,self.volume))as guest, \
                    patch.object(r,"validate_artifacts",return_value={},side_effect=ValueError("changed serial")if mode=="bad-capture"else None):
                result,_=r.run_images()
                self.assertIs(result["passed"],False);self.assertIs(result["image_gate"],False)
                guest.assert_called_once_with("normal",timeout=180,kernel_feature=r.FEATURE)
                self.assertEqual(sources.call_count,1 if mode=="bad-capture"else 2)
                self.assertIs(json.loads((self.build/"image-report.json").read_text())["image_gate"],False)

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
        index=self.build/"image-report.json"
        index.write_text(json.dumps(dict(passed=True,image_gate=True)))
        self.json(self.build/"manifest.json",self.manifest|dict(loader_feature="wrong-profile"))
        with patch.object(r,"source_hash",return_value="a"*64),patch.object(r,"run_system")as guest:
            with self.assertRaises(ValueError):r.run_images()
            guest.assert_not_called()
        failed=json.loads(index.read_text())
        self.assertIs(failed["passed"],False);self.assertIs(failed["image_gate"],False)
        self.assertEqual(failed["status"],"failed")

    def test_delegate_exception_replaces_old_green_index_and_propagates_failure(self):
        index=self.build/"image-report.json"
        index.write_text(json.dumps(dict(passed=True,image_gate=True)))
        qemu=self.root/"qemu-system-x86_64";qemu.write_bytes(b"synthetic emulator marker")
        code=self.root/"code.fd";code.write_bytes(b"fixture-code")
        variables=self.root/"vars.fd";variables.write_bytes(b"fixture-vars-template")
        with patch.object(r,"source_hash",return_value="a"*64),patch.object(r,"environment",return_value={"PATH":""}),                 patch.object(r.shutil,"which",return_value=str(qemu)),patch.object(r,"firmware",return_value=(code,variables)),                 patch.object(r,"run_system",side_effect=RuntimeError("controlled launch failure"))as guest:
            with self.assertRaisesRegex(RuntimeError,"controlled launch failure"):r.run_images()
            self.assertEqual(guest.call_count,1)
        failed=json.loads(index.read_text())
        self.assertIs(failed["passed"],False);self.assertIs(failed["image_gate"],False)
        self.assertEqual(failed["status"],"failed")

    def test_source_esp_external_link_is_refused_before_delegate_or_marker_exposure(self):
        outside=self.root.parent/"keep-marker";outside.write_bytes(b"private synthetic sentinel")
        (self.build/"esp/unknown.bin").symlink_to(outside)
        qemu=self.root/"qemu-system-x86_64";qemu.write_bytes(b"synthetic emulator marker")
        code=self.root/"code.fd";code.write_bytes(b"fixture-code")
        variables=self.root/"vars.fd";variables.write_bytes(b"fixture-vars-template")
        with patch.object(r,"source_hash",return_value="a"*64),patch.object(r,"environment",return_value={"PATH":""}),                 patch.object(r.shutil,"which",return_value=str(qemu)),patch.object(r,"firmware",return_value=(code,variables)),                 patch.object(r,"run_system",side_effect=RuntimeError("delegate reached too early"))as guest:
            with self.assertRaises(ValueError):r.run_images()
            guest.assert_not_called()
        self.assertEqual(outside.read_bytes(),b"private synthetic sentinel")


if __name__ == "__main__":unittest.main()
