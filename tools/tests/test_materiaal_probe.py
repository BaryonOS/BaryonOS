"""Host counterexamples for the evidence runner, never guest execution proof."""
from contextlib import ExitStack
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import run_materiaal_probe as d
import run_fpu as f
from test_display_probe import user_elf


def snapshots(case="normal"):
    values=[]
    for i,name in enumerate(d.PHASES[case],1):
        ready=name == "ready";editor=name == "editor";saved=case in d.WRITABLE_CASES and name in ("saved","list","reopened")
        length=0 if ready or editor else len(d.TEXT)
        keycount=i*8;mouse=6 if case == "normal" and i>1 else 0
        v=dict(READY=1,PRESENTED_GEN=i,FRAMES=i,TILES=i*d.TILES,KEYS=keycount,MOUSE=mouse,
               INPUT_ACKS=keycount+mouse,SAVE_ACKS=int(saved),SAVED_REVISION=int(saved),
               SAVED_BYTES=len(d.TEXT) if saved else 0,SAVED_SUM=d.TEXT_SUM if saved else 0,
               SCREEN=1 if name in ("ready","list") else 2,DRAFT_DIRTY=int(name in ("editor","dirty")),
               DRAFT_BYTES=length,DRAFT_SUM=0 if ready else d.volume.FNV_OFFSET if editor else d.TEXT_SUM,
               OBJECT=d.U64_MAX if ready else 0,SCALE=1,ERRORS=int(name == "readonly"))
        values.append(v)
    return values


def serial(case="normal"):
    text="\n".join((*d.BOOT,"MATERIAAL:MANIFEST:PASS",*d.GATES))+"\n";prefixes=[]
    for value in snapshots(case):
        text+="MATERIAAL:SNAPSHOT:BEGIN\n"+"".join(f"MATERIAAL:{name}:{value[name]:020d}\n" for name in d.FIELDS)+"MATERIAAL:SNAPSHOT:END\n"
        prefixes.append(text.encode())
    text+="MATERIAAL:F12_BARRIER:00000000000000000001\nMATERIAAL:CONTROLLER:RELEASED\n"
    for slot in (3,0,7):text+=f"MATERIAAL:EXIT_SLOT:{slot:020d}\nMATERIAAL:EXIT_CODE:{0:020d}\nMATERIAAL:EXIT_STEP:{0:020d}\n"
    values=dict(DISK_READS=30,DISK_WRITES=12 if case in d.WRITABLE_CASES else 0,DISK_FLUSHES=2 if case in d.WRITABLE_CASES else 0,LIVE_PROCESSES=0,LIVE_HANDLES=0,FRAMES_RECOVERED=1)
    text+="".join(f"MATERIAAL:{name}:{values[name]:020d}\n" for name in d.FINAL_FIELDS)+"\n".join(d.END)+"\n"
    return text.encode(),prefixes


def frames(case="normal"):
    pixels=bytearray(d.WIDTH*d.HEIGHT*3);result={};x,y,_,_=d.ASSIGNED
    for i in range(12):
        at=((y+5)*d.WIDTH+x+5+i)*3;pixels[at:at+3]=bytes((10+i,20+i,30+i))
    for i,name in enumerate(d.PHASES[case],1):
        at=((y+10)*d.WIDTH+x+10)*3;pixels[at:at+3]=bytes((i*20,30,40))
        if name in ("dirty","saved"):
            at=((y+150)*d.WIDTH+x+30)*3;pixels[at:at+3]=b"abc"
        result[name]=b"P6\n800 600\n255\n"+pixels
    return result


def text_disk(path):
    d.volume.create_image(path,"empty")
    raw=bytearray(path.read_bytes());v=d.volume
    version=dict(v.empty_version(),revision=1,length=len(d.TEXT),name=b"Nieuw document",content_sum=d.TEXT_SUM,chunks=[0],sums=[d.TEXT_SUM])
    entries=[v.empty_entry() for _ in range(v.OBJECTS)]
    entries[0]=dict(v.empty_entry(),revision=1,request_id=9,created=1,modified=1,present=True,current=version)
    for i,sector in enumerate(v.encode_table(1,9,entries)):
        at=(v.FIRST+v.TABLE_SECTORS+i)*v.SECTOR;raw[at:at+v.SECTOR]=sector
    at=(v.FIRST+v.DATA_START)*v.SECTOR;raw[at:at+len(d.TEXT)]=d.TEXT
    path.write_bytes(raw);return bytes(raw)


class GrammarTests(unittest.TestCase):
    def test_complete_normal_and_readonly_transcripts(self):
        for case in d.FEATURES:
            parsed=d.transcript(serial(case)[0],case,final=True)
            self.assertTrue(parsed["finished"])
            self.assertEqual(list(d.phases(parsed,case,complete=True)),list(d.PHASES[case]))

    def test_partial_snapshot_is_never_ready(self):
        raw,prefixes=serial();at=prefixes[0].index(b"MATERIAAL:SNAPSHOT:END")
        self.assertEqual(d.transcript(raw[:at])["snapshots"],[])
        with self.assertRaises(ValueError):d.transcript(raw[:at],final=True)

    def test_missing_duplicate_reordered_unknown_and_oversized_fail(self):
        raw=serial()[0]
        alternatives=[raw.replace((d.GATES[0]+"\n").encode(),b"",1),raw.replace(b"MATERIAAL:READY:",b"MATERIAAL:BOGUS:",1),
                      raw.replace(b"MATERIAAL:SNAPSHOT:END\n",b"MATERIAAL:SNAPSHOT:END\nMATERIAAL:SNAPSHOT:END\n",1),
                      raw.replace(b"MATERIAAL:READY:00000000000000000001",b"MATERIAAL:READY:00000000000000000000",1),
                      raw.replace(b"MATERIAAL:FRAMES:00000000000000000002",b"MATERIAAL:FRAMES:00000000000000000001",1),
                      raw.replace(b"MATERIAAL:OBJECT:18446744073709551615",b"MATERIAAL:OBJECT:18446744073709551616",1),
                      raw+b"MATERIAAL:EXTRA\n",raw+b"\0",b"x"*(d.SERIAL_LIMIT+1)]
        for altered in alternatives:
            with self.subTest(altered=altered[:50]),self.assertRaises(ValueError):d.transcript(altered,final=True)

    def test_source_and_focus_gates_precede_actual_snapshots(self):
        raw=serial()[0]
        for gate in d.GATES:
            with self.subTest(gate=gate),self.assertRaises(ValueError):d.transcript(raw.replace((gate+"\n").encode(),b"",1),final=True)
        with self.assertRaises(ValueError):d.transcript(raw.replace((d.GATES[0]+"\n").encode(),(d.GATES[3]+"\n").encode(),1),final=True)

    def test_barrier_cannot_replace_list_then_real_reopen(self):
        raw,prefixes=serial();early=prefixes[3]+raw[len(prefixes[-1]):]
        with self.assertRaises(ValueError):d.transcript(early,final=True)
        with self.assertRaises(ValueError):d.transcript(raw.replace(b"MATERIAAL:F12_BARRIER:00000000000000000001",b"MATERIAAL:F12_BARRIER:00000000000000000002"),final=True)

    def test_readonly_rejects_any_write_or_commit(self):
        raw=serial("recover")[0]
        for old,new in [(b"MATERIAAL:DISK_WRITES:00000000000000000000",b"MATERIAAL:DISK_WRITES:00000000000000000001"),
                        (b"MATERIAAL:SAVE_ACKS:00000000000000000000",b"MATERIAAL:SAVE_ACKS:00000000000000000001")]:
            with self.assertRaises(ValueError):d.transcript(raw.replace(old,new,1),"recover",final=True)

    def test_qmp_script_is_only_correlated_hardware_actions_and_frames(self):
        directory=Path("/owned/generated/run")
        for case in d.FEATURES:
            trace=[{"request":r,"response":{"return":{},"id":r["id"]}} for r in d.qmp_requests(directory,case)]
            d.validate_qmp(trace,directory,case)
            bad=copy.deepcopy(trace);bad[-1]["request"]={"execute":"human-monitor-command","arguments":{"command-line":"write_memory"},"id":len(trace)}
            with self.assertRaises(ValueError):d.validate_qmp(bad,directory,case)
            bad=copy.deepcopy(trace);bad[0]["response"]["id"]=True
            with self.assertRaises(ValueError):d.validate_qmp(bad,directory,case)
            bad=copy.deepcopy(trace);bad[-1]["response"]={"error":{"class":"Failure"},"id":len(trace)}
            with self.assertRaises(ValueError):d.validate_qmp(bad,directory,case)

    def test_keyboard_case_reaches_the_same_document_stages_without_pointer_events(self):
        directory=Path("/owned/generated/run")
        groups=d.qmp_groups(directory,"keyboard")
        self.assertEqual([name for name,_ in groups],["ready","editor","dirty","saved","list","reopened","close"])
        requests=[request for _,group in groups for request in group]
        self.assertFalse(any(request["execute"] == "input-send-event" for request in requests))
        self.assertEqual(groups[1][1][0]["arguments"]["keys"],[{"type":"qcode","data":"ret"}])
        self.assertEqual(d.FEATURES["keyboard"],d.FEATURES["normal"])
        self.assertEqual(d.PHASES["keyboard"],d.PHASES["normal"])

    def test_actual_frame_checks_reject_identical_wrong_geometry_and_chrome(self):
        image=frames();d.validate_frames(image,"normal")
        bad=dict(image);bad["editor"]=bad["ready"]
        with self.assertRaises(ValueError):d.validate_frames(bad,"normal")
        bad=dict(image);bad["saved"]=bad["saved"].replace(b"800 600",b"400 600",1)
        with self.assertRaises(ValueError):d.validate_frames(bad,"normal")
        bad=dict(image);data=bytearray(bad["saved"]);data[-1]^=1;bad["saved"]=bytes(data)
        with self.assertRaises(ValueError):d.validate_frames(bad,"normal")
        bad=dict(image);data=bytearray(bad["saved"]);header=len(b"P6\n800 600\n255\n");data[header+((d.ASSIGNED[1]+150)*800+d.ASSIGNED[0]+30)*3]^=1;bad["saved"]=bytes(data)
        with self.assertRaises(ValueError):d.validate_frames(bad,"normal")

    def test_independent_disk_checks_actual_text_and_scope(self):
        with tempfile.TemporaryDirectory() as name:
            raw=text_disk(Path(name)/"volume.img");self.assertEqual(d.disk_oracle(raw)["length"],14)
            bad=bytearray(raw);bad[0]^=1
            with self.assertRaises(ValueError):d.disk_oracle(bytes(bad))
            bad=bytearray(raw);bad[(d.volume.FIRST+d.volume.DATA_START)*d.volume.SECTOR]^=1
            with self.assertRaises(ValueError):d.disk_oracle(bytes(bad))


class ArtifactTests(unittest.TestCase):
    def setUp(self):
        self.temporary=tempfile.TemporaryDirectory();self.addCleanup(self.temporary.cleanup);self.root=Path(self.temporary.name)
        stack=ExitStack();self.addCleanup(stack.close)
        stack.enter_context(patch.object(d,"ROOT",self.root));stack.enter_context(patch.object(f,"ROOT",self.root))
        self.build=self.root/"build/fixture";(self.build/"users/materiaal").mkdir(parents=True);(self.build/"esp/EFI/BOOT").mkdir(parents=True)
        images=[];apps=[]
        for i,name in enumerate(d.NAMES):
            path=self.build/f"users/materiaal/{name}.elf";path.write_bytes(user_elf(i));images.append(path.read_bytes())
            apps.append({"name":name,"sha256":d.sha256(path),"elf":d.inspect_user(path)})
        (self.build/"esp/kernel.elf").write_bytes(b"HOST_ONLY_KERNEL_FIXTURE"+b"".join(images));(self.build/"esp/EFI/BOOT/BOOTX64.EFI").write_bytes(b"HOST_ONLY_LOADER")
        self.artifact={"passed":True,"native_runtime_performed":False,"probe_controller":True,"apps":apps}
        self.manifest={"kernel_feature":d.FEATURES["normal"],"loader_feature":"framebuffer","source_fingerprint_version":3,"source_sha256":"a"*64,
                       "sdk_runtime":False,"document_session":False,"materiaal":self.artifact,
                       "kernel":{"sha256":d.sha256(self.build/"esp/kernel.elf")},"loader":{"sha256":d.sha256(self.build/"esp/EFI/BOOT/BOOTX64.EFI")}}
        d.publish_json(self.build/"manifest.json",self.manifest);d.publish_json(self.build/"users/materiaal/build-report.json",self.artifact)

    def run_fixture(self,case="normal"):
        directory=self.build/f"run/materiaal-{case}-fixture";(directory/"inputs").mkdir(parents=True)
        shutil.copytree(self.build/"esp",directory/"esp")
        for name in d.NAMES:shutil.copyfile(self.build/f"users/materiaal/{name}.elf",directory/f"inputs/{name}.elf")
        manifest=copy.deepcopy(self.manifest);manifest["kernel_feature"]=d.FEATURES[case]
        d.publish_json(directory/"inputs/manifest.json",manifest);d.publish_json(directory/"inputs/build-report.json",self.artifact)
        for name in ("OVMF_CODE.fd","OVMF_VARS.template.fd","OVMF_VARS.fd","VGA.rom"):(directory/name).write_bytes(b"HOST_ONLY_FIRMWARE")
        raw,prefixes=serial(case);(directory/"serial.txt").write_bytes(raw);(directory/"stderr.txt").write_bytes(b"")
        for name,data in frames(case).items():(directory/f"{name}.ppm").write_bytes(data)
        trace=[{"request":r,"response":{"return":{},"id":r["id"]}} for r in d.qmp_requests(directory,case)];d.publish_json(directory/"qmp-trace.json",trace)
        if case in d.WRITABLE_CASES:d.volume.create_image(directory/"volume.before.img","empty")
        else:text_disk(directory/"volume.before.img")
        text_disk(directory/"volume.after.img");shutil.copyfile(directory/"volume.after.img",directory/"volume.img")
        names={"esp/kernel.elf","esp/EFI/BOOT/BOOTX64.EFI","OVMF_CODE.fd","OVMF_VARS.template.fd","VGA.rom","inputs/manifest.json","inputs/build-report.json"}|{f"inputs/{name}.elf" for name in d.NAMES}
        report={"scope":"HOST_ONLY_FIXTURE","passed":True,"case":case,"source_sha256":"a"*64,"source_fingerprint_version":3,"raw_qemu_exit":33,"timed_out":False,
                "kernel_feature":d.FEATURES[case],"profile":"pc-q35-8.2/qemu64/TCG/1CPU/256MiB/VGA/PS2",
                "qemu_entry_sha256":"b"*64,"qemu_implementation_sha256":"c"*64,"parent_run":None,
                "cwd":str(directory),"qemu_entry_path":"qemu","machine":"pc-q35-8.2","qmp_transport":"pipe","frozen_artifacts":{name:d.sha256(directory/name) for name in names},
                "stdout":raw.decode(),"before_sha256":d.sha256(directory/"volume.before.img"),"after_sha256":d.sha256(directory/"volume.after.img"),"stages":[]}
        report["command"]=d.qemu_arguments("qemu","pc-q35-8.2","pipe",case)
        for name,prefix,snapshot in zip(d.PHASES[case],prefixes,snapshots(case)):
            qmp_index=next(r["id"] for r in d.qmp_requests(directory,case) if r["execute"] == "screendump" and r["arguments"]["filename"] == str(directory/f"{name}.ppm"))
            report["stages"].append({"phase":name,"snapshot":snapshot,"serial_size":len(prefix),"serial_sha256":hashlib.sha256(prefix).hexdigest(),"qmp_index":qmp_index})
        report["artifact_sha256"]=d.artifact_hashes(directory);d.publish_json(directory/"result.json",report)
        return directory,report

    def test_exact_build_and_all_frozen_run_proof_replays(self):
        d.validate_build(self.build,"a"*64,"normal")
        for case in ("normal","keyboard"):
            directory,report=self.run_fixture(case)
            self.assertEqual(d.replay(report,directory,"a"*64)["disk"]["length"],14)

    def test_stale_source_changed_elf_and_nonembedded_binary_fail(self):
        with self.assertRaises(ValueError):d.validate_build(self.build,"b"*64,"normal")
        path=self.build/f"users/materiaal/{d.NAMES[0]}.elf";path.write_bytes(path.read_bytes()+b"changed")
        with self.assertRaises(ValueError):d.validate_build(self.build,"a"*64,"normal")

    def test_report_cannot_replace_serial_exit_qmp_prefix_or_capture_hash(self):
        directory,report=self.run_fixture()
        for key,value in [("stdout",report["stdout"].replace("MATERIAAL:SAVED_BYTES:","MATERIAAL:FORGED:")),("raw_qemu_exit",0),("timed_out",True)]:
            bad=copy.deepcopy(report);bad[key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):d.replay(bad,directory,"a"*64)
        bad=copy.deepcopy(report);bad["stages"][-1]["serial_sha256"]="0"*64
        with self.assertRaises(ValueError):d.replay(bad,directory,"a"*64)
        (directory/"reopened.ppm").write_bytes((directory/"ready.ppm").read_bytes())
        with self.assertRaises(ValueError):d.replay(report,directory,"a"*64)

    def test_cold_parent_is_recomputed_and_cannot_be_user_image(self):
        directory,report=self.run_fixture();image,_=d.parent_run(directory,self.build,"a"*64);self.assertEqual(image,directory/"volume.after.img")
        (directory/"serial.txt").write_bytes(b"MATERIAAL:CASE:PASS\n")
        with self.assertRaises(ValueError):d.parent_run(directory,self.build,"a"*64)
        with self.assertRaises(ValueError):d.parent_run(self.root,self.build,"a"*64)

    def test_active_qemu_disk_cannot_differ_from_checked_after_image(self):
        directory,report=self.run_fixture()
        image=directory/"volume.img";image.write_bytes(b"X"+image.read_bytes()[1:])
        # Even a refreshed generic file-hash sidecar cannot substitute a good
        # afterimage for the file named in QEMU's actual disk argument.
        report["artifact_sha256"]=d.artifact_hashes(directory)
        with self.assertRaises(ValueError):d.replay(report,directory,"a"*64)

    def test_capture_cannot_use_validly_hashed_final_serial_as_old_phase(self):
        directory,report=self.run_fixture();raw,prefixes=serial()
        for late in (raw,prefixes[-1],prefixes[1]):
            bad=copy.deepcopy(report)
            for stage in bad["stages"]:
                stage["serial_size"]=len(late);stage["serial_sha256"]=hashlib.sha256(late).hexdigest()
            with self.subTest(length=len(late)),self.assertRaises(ValueError):d.replay(bad,directory,"a"*64)

    def test_cold_offline_replay_requires_recomputed_normal_byte_chain(self):
        parent,_=self.run_fixture();directory,report=self.run_fixture("recover")
        with self.assertRaises(ValueError):d.replay(report,directory,"a"*64)
        report["parent_run"]=str(parent);d.replay(report,directory,"a"*64)
        bad=copy.deepcopy(report);bad["parent_run"]=str(directory)
        with self.assertRaises(ValueError):d.replay(bad,directory,"a"*64)
        bad=copy.deepcopy(report);bad["qemu_entry_sha256"]="d"*64
        with self.assertRaises(ValueError):d.replay(bad,directory,"a"*64)
        (parent/"volume.after.img").write_bytes(b"X"+(parent/"volume.after.img").read_bytes()[1:])
        with self.assertRaises(ValueError):d.replay(report,directory,"a"*64)

    def test_fatal_diagnostics_get_only_one_second_grace_and_no_success(self):
        directory=self.build/"run/diagnostic";directory.mkdir(parents=True)
        p=Mock();p.poll.return_value=None
        (directory/"serial.txt").write_bytes(b"MATERIAAL:TIMEOUT_SLOT:00000000000000000000\n")
        d.diagnostic_exit_grace(p,directory);p.wait.assert_called_once_with(timeout=1.0)
        p=Mock();p.poll.return_value=None;p.wait.side_effect=d.subprocess.TimeoutExpired("qemu",1)
        d.diagnostic_exit_grace(p,directory);p.wait.assert_called_once_with(timeout=1.0)
        (directory/"serial.txt").write_bytes(b"BOOT:EBS_OK\n")
        p=Mock();p.poll.return_value=None;d.diagnostic_exit_grace(p,directory);p.wait.assert_not_called()
        with self.assertRaises(ValueError):d.transcript(b"MATERIAAL:FAIL:TIMEOUT\n",final=True)

    def test_guest_timeout_diagnostics_are_structured_but_never_success(self):
        raw = (
            b"MATERIAAL:TIMEOUT_SLOT:00000000000000000000\n"
            b"MATERIAAL:AT:00000000000000000408\n"
            b"MATERIAAL:VALUE:00000000000000000001\n"
            b"MATERIAAL:AT:00000000000000000496\n"
            b"MATERIAAL:VALUE:00000000000000001608\n"
            b"MATERIAAL:FAIL:TIMEOUT\n"
        )
        diagnostics = d.guest_diagnostics(raw)
        self.assertEqual(diagnostics, {
            "guest_timeout": True,
            "guest_failure": True,
            "slots": [{"slot": 0, "words": [
                {"offset": 408, "value": 1}, {"offset": 496, "value": 1608},
            ]}],
        })
        with self.assertRaises(ValueError):
            d.transcript(raw, final=True)
        with self.assertRaises(ValueError):
            d.guest_diagnostics(b"MATERIAAL:TIMEOUT_SLOT:00000000000000000008\n")
        with self.assertRaises(ValueError):
            d.guest_diagnostics(
                b"MATERIAAL:TIMEOUT_SLOT:00000000000000000000\n"
                b"MATERIAAL:AT:00000000000000000041\n"
            )

    def test_symlink_hardlink_and_outside_build_paths_fail(self):
        image=self.build/"users/materiaal/desktop-init.elf";other=self.root/"outside";other.mkdir();os.link(image,other/"hard")
        with self.assertRaises(ValueError):d.validate_build(self.build,"a"*64,"normal")
        with self.assertRaises(ValueError):d.build_path(self.root/"personal")
        link=self.build/"linked";link.symlink_to(other,target_is_directory=True)
        with self.assertRaises(ValueError):d.build_path(link)

    def test_final_image_errors_publish_failed_reports(self):
        firmware=self.root/"firmware";firmware.write_bytes(b"HOST_ONLY_FIRMWARE")
        qemu=self.root/"qemu";qemu.write_bytes(b"HOST_ONLY_QEMU")
        real_check=d.check_image
        for failure in ("geometry", "hardlink", "oserror"):
            with self.subTest(failure=failure), ExitStack() as stack:
                stack.enter_context(patch.object(d,"source_hash",return_value="a"*64))
                stack.enter_context(patch.object(d,"qemu_machine",return_value="pc-q35-8.2"))
                stack.enter_context(patch.object(d,"environment",return_value={"PATH":""}))
                stack.enter_context(patch.object(d.shutil,"which",return_value=str(qemu)))
                stack.enter_context(patch.object(d,"firmware",return_value=(firmware,firmware)))
                stack.enter_context(patch.object(d,"external_file",side_effect=lambda path,limit:path))
                stack.enter_context(patch.object(d.subprocess,"check_output",return_value="HOST_ONLY"))
                checks=[];reports=[]
                real_publish_json=d.publish_json
                def capture_report(path,value):
                    if path.name == "result.json":reports.append(value)
                    real_publish_json(path,value)
                stack.enter_context(patch.object(d,"publish_json",side_effect=capture_report))
                def check(image):
                    checks.append(image)
                    if failure == "oserror" and len(checks) == 2:
                        raise OSError("image unavailable")
                    real_check(image)
                stack.enter_context(patch.object(d,"check_image",side_effect=check))
                def launch(*args,**kwargs):
                    # Model a late error after an earlier passing result.
                    reports[-1]["passed"]=True
                    image=Path(kwargs["cwd"])/"volume.img"
                    if failure == "geometry":image.write_bytes(b"invalid")
                    if failure == "hardlink":
                        outside=self.root/"linked-image";os.link(image,outside)
                    raise OSError("launch failed")
                stack.enter_context(patch.object(d.subprocess,"Popen",side_effect=launch))
                report=d.run_materiaal_probe(build_directory=self.build,vga_rom=firmware)
                self.assertFalse(report["passed"])
                self.assertIn("disk_error",report)
                self.assertNotIn("volume.img",report["artifact_sha256"])
                self.assertEqual(d.load_json(Path(report["run_directory"])/"result.json"),report)
                self.assertEqual(d.load_json(self.build/"materiaal-probe-report.json"),report)
                if failure == "oserror":self.assertEqual(report["disk_error"],"image unavailable")
                if failure == "hardlink":(self.root/"linked-image").unlink()

    def test_failed_preflight_retires_old_pass_without_launch(self):
        d.publish_json(self.build/"materiaal-probe-report.json",{"passed":True})
        with patch.object(d,"source_hash",return_value="b"*64),patch.object(d.subprocess,"Popen") as launch:
            with self.assertRaises(ValueError):d.run_materiaal_probe(build_directory=self.build,vga_rom=self.root/"not-used")
            launch.assert_not_called()
        self.assertIs(d.load_json(self.build/"materiaal-probe-report.json")["passed"],False)



class SparseFrameCounters(unittest.TestCase):
    """SUR1 sparse frames: full first frame, then at least the final tile."""

    def test_sparse_counts_accepted_and_incomplete_first_frame_rejected(self):
        values=snapshots()
        for i,value in enumerate(values):
            value["TILES"]=d.TILES+i
        previous=None
        for value in values:
            d.snapshot_checks(value,previous,"normal")
            previous=value
        first=dict(values[0],TILES=d.TILES-1)
        with self.assertRaisesRegex(ValueError,"Invalid actual frame/counter snapshot"):
            d.snapshot_checks(first,None,"normal")
        # Passes the cumulative bound on its own, so only the inter-snapshot
        # rule (at least one tile per newly presented frame) can reject it.
        before=dict(values[0],TILES=d.TILES+100)
        stale=dict(values[1],TILES=d.TILES+100)
        d.snapshot_checks(stale,None,"normal")
        with self.assertRaisesRegex(ValueError,"Repeated/stale/incomplete presented frame"):
            d.snapshot_checks(stale,before,"normal")


if __name__ == "__main__":unittest.main()
