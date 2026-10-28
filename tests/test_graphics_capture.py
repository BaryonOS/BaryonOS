"""Synthetic host capture fixtures only; no boot, framebuffer or VM evidence."""
from copy import deepcopy
import io
import json
import os
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch
import zlib

sys.path.insert(0, str(Path(__file__).resolve().parents[1]/"tools"))
import capture_graphics as c


class GraphicsCapture(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="nieuw-os-b07-fixture-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)/"checkout"
        (self.root/"docs/verification").mkdir(parents=True)
        self.run = Path(self.temp.name)/"private-run"
        self.run.mkdir()
        self.qemu = Path(self.temp.name)/"qemu-system-x86_64"
        self.qemu.write_bytes(b"Synthetic executable identity; never executed.\n")
        self.frozen = "a"*64
        for name,value in (("ROOT", self.root), ("source_hash", lambda: self.frozen)):
            mocked = patch.object(c, name, value)
            mocked.start()
            self.addCleanup(mocked.stop)
        self.target = self.root/"docs/verification/fixture-capture"
        self.make_fixture()

    def write(self, name, raw):
        path = self.run/name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(raw)

    def save_report(self):
        self.write("result.json", json.dumps(self.report, indent=2).encode()+b"\n")

    def reseal(self, name):
        self.report["artifacts"][name] = c.digest((self.run/name).read_bytes())
        self.save_report()

    def make_fixture(self):
        width,height = 96,224
        data = bytearray(width*height*3)
        data[:16] = bytes(range(16))
        def frame(raw):
            return f"P6\n{width} {height}\n255\n".encode()+raw
        def change(raw,x,color):
            result = bytearray(raw)
            at = (194*width+x)*3
            result[at:at+3] = color
            return bytes(result)
        a = change(data,48,b"abc")
        ab = change(a,64,b"def")
        ac = change(a,64,b"ghi")
        self.frames = [frame(raw) for raw in (data,a,ab,a,ac)]
        for name,raw in zip(c.FRAMES,self.frames):
            self.write(name+".ppm",raw)
        for name in ("OVMF_CODE.fd", "OVMF_VARS.fd", "OVMF_VARS.template.fd", "VGA.rom", "esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI"):
            self.write(name,("Synthetic host-only bytes: "+name).encode())
        manifest = dict(source_fingerprint_version=3, source_sha256=self.frozen,
            kernel_feature="graphics-g0", loader_feature="graphics-g0",
            kernel={"sha256":c.digest((self.run/"esp/kernel.elf").read_bytes())},
            loader={"sha256":c.digest((self.run/"esp/EFI/BOOT/BOOTX64.EFI").read_bytes())})
        self.write("build-manifest.json",json.dumps(manifest).encode())
        lines = ["BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS",
            f"G0:FRAMEBUFFER:{width}:{height}:{width}:1", "G0:MAPPINGS:PASS", "G0:READY",
            "G0:INPUT:1", "G0:INPUT:2", "G0:INPUT:1", "G0:INPUT:2", "G0:PASS", "M1:PASS"]
        serial = "Synthetic host-validator fixture, never guest execution.\n"+"\n".join(lines)+"\n"
        self.write("serial.txt",serial.encode())
        self.write("stderr.txt",b"")
        requests = [{"execute":"qmp_capabilities","id":1}]
        for index,name in enumerate(c.FRAMES):
            if index:
                requests.append({"execute":"send-key","id":len(requests)+1,"arguments":{
                    "keys":[{"type":"qcode","data":("a","b","backspace","c")[index-1]}],"hold-time":30}})
            requests.append({"execute":"screendump","id":len(requests)+1,"arguments":{"filename":str(self.run/(name+".ppm"))}})
        requests.append({"execute":"send-key","id":11,"arguments":{"keys":[{"type":"qcode","data":"esc"}],"hold-time":30}})
        self.write("qmp-trace.json",json.dumps([{"request":r,"response":{"return":{},"id":r["id"]}} for r in requests]).encode())
        _,overall,steps = c.frame_checks(*self.frames)
        rom_hash = c.digest((self.run/"VGA.rom").read_bytes())
        self.report = dict(scope="Synthetic host fixture; not native evidence", passed=True,checks=dict.fromkeys(c.CHECKS,True),
            error=None,cwd=str(self.run),machine="pc-q35-7.2",qmp_transport="pipe",build_directory=str(self.root/"build"),
            raw_qemu_exit=33,duration_s=1.0,source_fingerprint_version=3,source_sha256=self.frozen,base_commit="b"*40,
            qemu_version="QEMU emulator version synthetic-host-fixture",qemu_entry_path=str(self.qemu),qemu_entry_sha256=c.digest(self.qemu.read_bytes()),
            run_directory=str(self.run),stdout=serial,artifacts={},changed_bounds=list(overall),step_changed_bounds=[list(s) for s in steps],
            vga_rom=dict(source_path=str(self.run/"VGA.rom"),copied_path="VGA.rom",sha256=rom_hash,expected_sha256=rom_hash,expected_hash_checked=True))
        self.report["command"] = c.expected_command(self.report,True)
        self.report["artifacts"] = {name:c.digest((self.run/name).read_bytes()) for name in c.BASE_ARTIFACTS|{"VGA.rom"}}
        self.save_report()

    def capture(self):
        with patch("sys.stdout",io.StringIO()):
            return c.capture(self.run,self.target)

    def rejected(self):
        with self.assertRaises((ValueError,OSError,UnicodeError)):
            self.capture()
        self.assertFalse(self.target.exists())

    def test_new_archive_has_only_exact_text_and_lossless_pngs(self):
        before = {p.relative_to(self.run):p.read_bytes() for p in self.run.rglob("*") if p.is_file()}
        self.assertEqual(self.capture(),self.target)
        expected = set(c.TEXT_FILES)|{"capture.json","image-roundtrip.json"}|{name+".png" for name in c.FRAMES}
        self.assertEqual({p.name for p in self.target.iterdir()},expected)
        capture = json.loads((self.target/"capture.json").read_text())
        for name,digest in capture["files"].items():
            self.assertEqual(c.sha256(self.target/name),digest)
        images = json.loads((self.target/"image-roundtrip.json").read_text())["images"]
        for image,original in zip(images,self.frames):
            width,height,data = c.decode_png((self.target/(image["name"]+".png")).read_bytes())
            self.assertEqual((width,height),(96,224))
            self.assertEqual(bytes.fromhex(image["original_header_hex"])+data,original)
        self.assertEqual(before,{p.relative_to(self.run):p.read_bytes() for p in self.run.rglob("*") if p.is_file()})

    def test_standard_vga_and_unix_private_profile_can_be_captured(self):
        self.report["vga_rom"] = None
        del self.report["artifacts"]["VGA.rom"]
        (self.run/"VGA.rom").unlink()
        self.report["qmp_transport"] = "unix"
        self.report["command"] = c.expected_command(self.report,False)
        self.save_report()
        self.capture()
        self.assertTrue((self.target/"capture.json").is_file())

    def test_later_serial_truncation_cannot_be_repaired_from_report(self):
        path = self.run/"serial.txt"
        raw = path.read_bytes()[:-len(b"G0:PASS\nM1:PASS\n")]
        path.write_bytes(raw)
        self.rejected()
        self.assertEqual(path.read_bytes(),raw)

    def test_resealed_missing_terminal_marker_still_fails(self):
        path = self.run/"serial.txt"
        raw = path.read_bytes().replace(b"G0:PASS\n",b"")
        path.write_bytes(raw)
        self.report["stdout"] = raw.decode()
        self.reseal("serial.txt")
        self.rejected()

    def test_changed_artifact_bytes_are_rejected(self):
        for name in ("VGA.rom","esp/kernel.elf","esp/EFI/BOOT/BOOTX64.EFI","OVMF_CODE.fd","typed.ppm"):
            path = self.run/name
            original = path.read_bytes()
            with self.subTest(name=name):
                path.write_bytes(original+b"changed")
                self.rejected()
            path.write_bytes(original)

    def test_manifest_hash_and_feature_cannot_be_resealed_away(self):
        path = self.run/"build-manifest.json"
        original = path.read_bytes()
        for change in ("kernel", "loader", "feature", "source"):
            with self.subTest(change=change):
                manifest = json.loads(original)
                if change in ("kernel","loader"):
                    manifest[change]["sha256"] = "0"*64
                elif change == "feature":
                    manifest["loader_feature"] = None
                else:
                    manifest["source_sha256"] = "0"*64
                path.write_text(json.dumps(manifest))
                self.reseal("build-manifest.json")
                self.rejected()
        path.write_bytes(original)

    def test_each_declared_check_and_exit_contract_is_required(self):
        original = deepcopy(self.report)
        for key in c.CHECKS:
            for value in (False,1,None):
                with self.subTest(check=key,value=value):
                    self.report = deepcopy(original)
                    self.report["checks"][key] = value
                    self.save_report()
                    self.rejected()
        for field,value in (("passed",1),("error","late failure"),("raw_qemu_exit",33.0),("raw_qemu_exit",-9),
                            ("duration_s",61),("duration_s",True),("source_sha256","0"*64),
                            ("source_fingerprint_version",3.0),("base_commit","missing")):
            with self.subTest(field=field,value=value):
                self.report = deepcopy(original)
                self.report[field] = value
                self.save_report()
                self.rejected()
        self.report = deepcopy(original)
        del self.report["checks"][next(iter(c.CHECKS))]
        self.save_report()
        self.rejected()

    def test_rom_pin_and_qemu_identity_are_not_report_only(self):
        self.report["vga_rom"]["expected_sha256"] = "0"*64
        self.save_report()
        self.rejected()
        self.report["vga_rom"]["expected_sha256"] = self.report["vga_rom"]["sha256"]
        self.save_report()
        self.qemu.write_bytes(b"different tool")
        self.rejected()

    def test_resealed_pixel_substitution_or_wrong_backspace_is_rejected(self):
        original = (self.run/"after-backspace.ppm").read_bytes()
        self.write("after-backspace.ppm",self.frames[2])
        self.reseal("after-backspace.ppm")
        self.rejected()
        self.write("after-backspace.ppm",original)
        self.reseal("after-backspace.ppm")
        changed = bytearray(self.frames[-1])
        header = len(changed)-96*224*3
        changed[header] ^= 1
        self.write("typed.ppm",changed)
        self.reseal("typed.ppm")
        self.rejected()

    def test_network_extra_device_foreign_qmp_path_and_cwd_are_rejected(self):
        original = deepcopy(self.report)
        commands = [original["command"]+["-drive","file=/dev/sda"],
                    ["user" if item == "none" and i == original["command"].index("-nic")+1 else item
                     for i,item in enumerate(original["command"])] ,
                    ["tcp:0.0.0.0:1234,server=on" if item == "stdio" else item for item in original["command"]]]
        for command in commands:
            self.report = deepcopy(original)
            self.report["command"] = command
            self.save_report()
            self.rejected()
        self.report = deepcopy(original)
        self.report["cwd"] = str(self.root)
        self.save_report()
        self.rejected()

    def test_qmp_response_identity_path_and_key_sequence_are_rechecked(self):
        path = self.run/"qmp-trace.json"
        original = json.loads(path.read_text())
        for mutation in ("id", "bool-id", "error", "frame", "key"):
            trace = deepcopy(original)
            if mutation == "id": trace[1]["response"]["id"] = 99
            if mutation == "bool-id": trace[0]["response"]["id"] = True
            if mutation == "error": trace[1]["response"]["error"] = {"class":"failure"}
            if mutation == "frame": trace[1]["request"]["arguments"]["filename"] = "/tmp/foreign.ppm"
            if mutation == "key": trace[6]["request"]["arguments"]["keys"][0]["data"] = "a"
            with self.subTest(mutation=mutation):
                path.write_text(json.dumps(trace))
                self.reseal("qmp-trace.json")
                self.rejected()

    def test_duplicate_json_fields_and_nonfinite_values_are_rejected(self):
        raw = (self.run/"result.json").read_bytes()
        for bad in (raw.replace(b'"passed": true',b'"passed": false, "passed": true'),
                    raw.replace(b'"duration_s": 1.0',b'"duration_s": NaN')):
            self.write("result.json",bad)
            self.rejected()
        self.write("result.json",raw)
        trace = (self.run/"qmp-trace.json").read_bytes().replace(b'"id": 1}',b'"id": 2, "id": 1}',1)
        self.write("qmp-trace.json",trace)
        self.reseal("qmp-trace.json")
        self.rejected()

    def test_missing_extra_and_path_escape_artifacts_are_rejected(self):
        original = deepcopy(self.report)
        for name in ("../../outside", "/etc/passwd", "extra.bin"):
            self.report = deepcopy(original)
            self.report["artifacts"][name] = "0"*64
            self.save_report()
            self.rejected()
        self.report = deepcopy(original)
        del self.report["artifacts"]["initial.ppm"]
        self.save_report()
        self.rejected()
        self.report = original
        self.save_report()
        self.write("unlisted.bin",b"Must not be copied or silently ignored")
        self.rejected()

    def test_symlinks_hardlinks_and_linked_parent_are_rejected(self):
        artifact = self.run/"VGA.rom"
        original = artifact.read_bytes()
        outside = Path(self.temp.name)/"outside"
        outside.write_bytes(original)
        artifact.unlink()
        artifact.symlink_to(outside)
        self.rejected()
        artifact.unlink()
        os.link(outside,artifact)
        self.rejected()
        artifact.unlink()
        artifact.write_bytes(original)
        linked = Path(self.temp.name)/"linked-run"
        linked.symlink_to(self.run,target_is_directory=True)
        with self.assertRaises(ValueError): c.capture(linked,self.target)
        esp = self.run/"esp"
        esp.rename(self.run/"real-esp")
        esp.symlink_to(self.run/"real-esp",target_is_directory=True)
        self.rejected()

    def test_bounded_regular_files_reject_oversize_and_fifo(self):
        path = self.run/"serial.txt"
        with path.open("wb") as stream:
            stream.truncate(c.LIMITS["serial.txt"]+1)
        self.rejected()
        path.unlink()
        os.mkfifo(path)
        self.rejected()

    def test_existing_parent_escape_and_symlink_destinations_are_preserved(self):
        for target in (self.root/"docs/verification",self.root/"docs/verification/../outside",self.root/"outside"):
            with self.assertRaises(ValueError): c.capture(self.run,target)
        self.target.mkdir()
        keep = self.target/"keep.txt"
        keep.write_bytes(b"Existing user evidence")
        with self.assertRaises(ValueError): self.capture()
        self.assertEqual(keep.read_bytes(),b"Existing user evidence")
        keep.unlink(); self.target.rmdir()
        self.target.symlink_to(self.root/"missing",target_is_directory=True)
        with self.assertRaises(ValueError): self.capture()
        self.assertTrue(self.target.is_symlink())

    def test_input_change_during_png_conversion_aborts_without_archive(self):
        original = c.encode_png
        def changing(frame):
            (self.run/"serial.txt").write_bytes(b"Late truncated serial")
            return original(frame)
        with patch.object(c,"encode_png",side_effect=changing): self.rejected()
        self.assertEqual((self.run/"serial.txt").read_bytes(),b"Late truncated serial")
        self.assertEqual(list((self.root/"docs/verification").iterdir()),[])

    def test_source_change_during_capture_aborts(self):
        with patch.object(c,"source_hash",side_effect=[self.frozen,"0"*64]): self.rejected()

    def test_staged_symlink_is_rejected_even_with_identical_bytes(self):
        original = c.snapshot
        calls = 0
        def tampered(*args):
            nonlocal calls
            result = original(*args)
            calls += 1
            if calls == 2:
                staged = next((self.root/"docs/verification").glob(".g0-capture-*"))
                path = staged/"serial.txt"
                path.unlink()
                path.symlink_to(self.run/"serial.txt")
            return result
        with patch.object(c,"snapshot",side_effect=tampered): self.rejected()

    def test_destination_created_at_final_rename_is_never_replaced(self):
        original = c.publish_new
        def racing(staged,destination):
            destination.mkdir()
            return original(staged,destination)
        with patch.object(c,"publish_new",side_effect=racing):
            with self.assertRaises(ValueError): self.capture()
        self.assertTrue(self.target.is_dir())
        self.assertEqual(list(self.target.iterdir()),[])

    def test_missing_atomic_no_replace_support_cannot_publish(self):
        with patch.object(c.ctypes,"CDLL",return_value=object()):
            with self.assertRaises(RuntimeError): self.capture()
        self.assertFalse(self.target.exists())
        self.assertEqual(list((self.root/"docs/verification").iterdir()),[])

    def test_corrupt_png_or_extra_decompressed_bytes_cannot_be_archived(self):
        valid = c.encode_png(self.frames[0])
        for altered in (valid[:-1],valid+b"extra",valid[:29]+bytes([valid[29]^1])+valid[30:]):
            with self.assertRaises(ValueError): c.decode_png(altered)
        with patch.object(c,"encode_png",return_value=valid[:-1]): self.rejected()
        width,height,_ = c.ppm(self.frames[0])
        header = c.png_chunk(b"IHDR",struct.pack(">IIBBBBB",width,height,8,2,0,0,0))
        bomb = c.PNG_SIGNATURE+header+c.png_chunk(b"IDAT",zlib.compress(bytes((width*3+1)*height+1)))+c.png_chunk(b"IEND",b"")
        with self.assertRaises(ValueError): c.decode_png(bomb)


if __name__ == "__main__":
    unittest.main()
