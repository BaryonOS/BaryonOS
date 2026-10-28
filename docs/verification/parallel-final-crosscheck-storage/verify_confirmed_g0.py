"""Read the replacement G0 artifacts and lossless PNGs, without running a VM."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import subprocess
import sys

import PIL
from PIL import Image

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
RUN = Path("/tmp/nieuw-os-g0-confirmed-pipe/graphics-g0-6c34d3a840")
ARCHIVE = ROOT / "docs/verification/parallel-final-g0-confirmed/pipe"
HEAD = "3e41bbf93822f7044c6020259f4a752925e1966d"
SOURCE = "307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60"
sys.path.insert(0, str(ROOT / "tools"))
from common import source_hash

report = {"scope": "Second independent read-only artifact check; no new VM, build, suite or image write",
          "source_commit": HEAD, "source_sha256": SOURCE, "run_directory": str(RUN),
          "started_utc": datetime.now(timezone.utc).isoformat(), "checks": {}, "files": {},
          "image_decoder": "Pillow " + PIL.__version__}


def check(name, value):
    report["checks"][name] = bool(value)


def digest(path, expected=None):
    raw = path.read_bytes()
    value = hashlib.sha256(raw).hexdigest()
    key = str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path)
    report["files"][key] = {"bytes": len(raw), "sha256": value}
    if expected is not None:
        report["files"][key]["expected_sha256"] = expected
        check("hash:" + key, value == expected)
    return value


def load(path):
    digest(path)
    return json.loads(path.read_bytes())


def pixels(raw):
    magic, dimensions, maximum, data = raw.split(b"\n", 3)
    if (magic, dimensions, maximum) != (b"P6", b"800 600", b"255") or len(data) != 800*600*3:
        raise ValueError("Invalid actual PPM bytes")
    return data


def bounds(first, second):
    points = [index for index in range(800*600) if first[index*3:index*3+3] != second[index*3:index*3+3]]
    return None if not points else [min(i % 800 for i in points), min(i // 800 for i in points),
                                   max(i % 800 for i in points)+1, max(i // 800 for i in points)+1]


def verify():
    check("current-source-and-head", source_hash() == SOURCE and subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip() == HEAD)
    current = load(RUN/"result.json")
    archived = load(ARCHIVE/"result.json")
    manifest = load(RUN/"build-manifest.json")
    check("same-raw-run-report", current == archived and current["run_directory"] == str(RUN))
    check("source-and-paired-build", current["base_commit"] == HEAD
          and current["source_sha256"] == manifest["source_sha256"] == SOURCE
          and current["source_fingerprint_version"] == manifest["source_fingerprint_version"] == 3
          and manifest["kernel_feature"] == manifest["loader_feature"] == "graphics-g0")
    names = ["initial", "a", "ab", "after-backspace", "typed"]
    mandatory = {name+".ppm" for name in names} | {"qmp-trace.json", "build-manifest.json", "OVMF_CODE.fd",
        "OVMF_VARS.template.fd", "OVMF_VARS.fd", "serial.txt", "stderr.txt", "VGA.rom", "esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI"}
    check("exact-fifteen-artifacts", set(current["artifacts"]) == mandatory)
    for name, expected in current["artifacts"].items():
        path = RUN/name
        check("regular-run-artifact:"+name, path.is_file() and not path.is_symlink() and path.resolve().is_relative_to(RUN))
        digest(path, expected)
    for name in ("serial.txt", "stderr.txt", "build-manifest.json", "qmp-trace.json"):
        digest(ARCHIVE/name, current["artifacts"][name])
    check("bound-kernel-and-loader", current["artifacts"]["esp/kernel.elf"] == manifest["kernel"]["sha256"]
          and current["artifacts"]["esp/EFI/BOOT/BOOTX64.EFI"] == manifest["loader"]["sha256"])
    text = (RUN/"serial.txt").read_bytes().decode("utf-8", "strict")
    serial = text.splitlines()
    expected = ["BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS",
        "G0:FRAMEBUFFER:800:600:800:1", "G0:MAPPINGS:PASS", "G0:READY", "G0:INPUT:1", "G0:INPUT:2",
        "G0:INPUT:1", "G0:INPUT:2", "G0:PASS", "M1:PASS"]
    check("complete-actual-serial", text == current["stdout"] and len((RUN/"serial.txt").read_bytes()) == 425
          and serial.count("BOOT:EBS_OK") == 1 and serial[serial.index("BOOT:EBS_OK"):] == expected
          and not any(word in text for word in ("FAIL", "PANIC")) and (RUN/"stderr.txt").read_bytes() == b"")
    check("real-exit-and-selected-profile", type(current["raw_qemu_exit"]) is int and current["raw_qemu_exit"] == 33
          and current["passed"] is True and current["error"] is None and all(current["checks"].values())
          and 0 < current["duration_s"] < 60 and current["qmp_transport"] == "pipe"
          and current["machine"] == "pc-q35-7.2" and current["command"][current["command"].index("-nic")+1] == "none")
    frames = [(RUN/(name+".ppm")).read_bytes() for name in names]
    rgb = [pixels(frame) for frame in frames]
    steps = [bounds(a,b) for a,b in zip(rgb,rgb[1:])]
    overall = bounds(rgb[0],rgb[-1])
    check("each-key-changes-pixels", overall is not None and all(step is not None for step in steps))
    check("only-input-row", all(rect is not None and 48 <= rect[0] < rect[2] <= 80 and 192 <= rect[1] < rect[3] <= 212
                                for rect in [*steps, overall]))
    check("exact-backspace-restoration", frames[1] == frames[3] and frames[2] != frames[3])
    check("nonempty-native-scene", len(set(rgb[0])) > 8)
    check("computed-bounds-match", steps == current["step_changed_bounds"] and overall == current["changed_bounds"])
    report["pixel_bounds"] = {"steps": steps, "overall": overall}
    images = load(ARCHIVE/"image-roundtrip.json")["images"]
    check("exact-five-lossless-images", [item["name"] for item in images] == names)
    report["roundtrips"] = []
    for item, original in zip(images,frames):
        name = item["name"]
        path = ARCHIVE/(name+".png")
        digest(path,item["png_sha256"])
        with Image.open(path) as image:
            check("PNG-mode-geometry:"+name, image.format == "PNG" and image.mode == "RGB"
                  and image.size == (item["width"],item["height"]) == (800,600))
            restored = bytes.fromhex(item["original_header_hex"]) + image.tobytes()
        restored_hash = hashlib.sha256(restored).hexdigest()
        exact = restored == original
        check("PNG-restores-original-PPM:"+name, exact and restored_hash == item["ppm_sha256"]
              == current["artifacts"][name+".ppm"])
        report["roundtrips"].append({"name": name, "exact_bytes": exact, "reconstructed_ppm_sha256": restored_hash})
    trace = load(RUN/"qmp-trace.json")
    operations = ["qmp_capabilities", "screendump", "send-key", "screendump", "send-key", "screendump", "send-key", "screendump", "send-key", "screendump", "send-key"]
    check("QMP-id-and-response-correlation", len(trace) == 11 and [item["request"]["execute"] for item in trace] == operations
          and all(type(item["request"]["id"]) is int and type(item["response"]["id"]) is int
                  and item["request"]["id"] == item["response"]["id"] == i+1
                  and item["response"] == {"return": {}, "id": i+1} for i,item in enumerate(trace)))
    check("QMP-keys-and-artifact-paths", [item["request"]["arguments"]["keys"][0]["data"] for item in trace if item["request"]["execute"] == "send-key"]
          == ["a", "b", "backspace", "c", "esc"] and [item["request"]["arguments"]["filename"] for item in trace if item["request"]["execute"] == "screendump"]
          == [str(RUN/(name+".ppm")) for name in names])
    pin = current["vga_rom"]
    check("explicit-ROM-pin", pin["expected_hash_checked"] is True and pin["sha256"] == pin["expected_sha256"] == current["artifacts"]["VGA.rom"]
          and "VGA,vgamem_mb=16,romfile=VGA.rom" in current["command"])
    digest(Path(pin["source_path"]),pin["expected_sha256"])
    digest(Path(current["qemu_entry_path"]),current["qemu_entry_sha256"])
    for source,name in (("OVMF_CODE_4M.fd","OVMF_CODE.fd"),("OVMF_VARS_4M.fd","OVMF_VARS.template.fd")):
        digest(Path("/tmp/nieuw-os-par-tools/qemu/usr/share/OVMF")/source,current["artifacts"][name])
    invalidated = load(ROOT/"docs/verification/parallel-final-g0/invalidated.json")
    check("original-failed-run-still-excluded", invalidated["accepted_as_final_evidence"] is False
          and hashlib.sha256((ROOT/"build/parallel-final-g0/run/graphics-g0-5ad6321b1d/serial.txt").read_bytes()).hexdigest()
          == invalidated["observed_later_serial_sha256"])
    unix = load(Path("/tmp/nieuw-os-g0-confirmed-unix/graphics-g0-a472268dd9/result.json"))
    check("separate-UNIX-failure-not-accepted", unix["passed"] is False and unix["qmp_transport"] == "unix"
          and "Operation not permitted" in unix["error"])
    report["excluded_unix_probe"] = {"run_directory": unix["run_directory"], "passed": unix["passed"], "error": unix["error"], "raw_qemu_exit": unix["raw_qemu_exit"]}
    for name,expected_hash in current["artifacts"].items():
        check("stable-after-crosscheck:"+name, hashlib.sha256((RUN/name).read_bytes()).hexdigest() == expected_hash)
    check("source-still-unchanged", source_hash() == SOURCE)


if __name__ == "__main__":
    try:
        verify()
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    report["passed"] = "error" not in report and all(report["checks"].values())
    report["finished_utc"] = datetime.now(timezone.utc).isoformat()
    report["verification_script_sha256"] = digest(Path(__file__))
    (OUT/"second-check.json").write_text(json.dumps(report,indent=2)+"\n")
    print(json.dumps({"passed": report["passed"], "checks": len(report["checks"]), "files": len(report["files"]),
        "failed": [key for key,value in report["checks"].items() if not value], "error": report.get("error")},indent=2))
    raise SystemExit(0 if report["passed"] else 1)
