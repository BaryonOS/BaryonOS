"""B06: independently re-read existing qsim/G0 artifacts; never run a VM."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import re
import struct
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
HEAD = "3e41bbf93822f7044c6020259f4a752925e1966d"
SOURCE = "307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60"
sys.path.insert(0, str(ROOT / "tools"))
from common import source_hash

report = {"scope": "Independent existing-artifact inspection; no new VM, build or suite",
          "source_commit": HEAD, "source_sha256": SOURCE,
          "started_utc": datetime.now(timezone.utc).isoformat(), "checks": {}, "files": {}}


def check(name, condition):
    report["checks"][name] = bool(condition)


def label(path):
    return str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path)


def file_hash(path, expected=None):
    path = Path(path)
    raw = path.read_bytes()
    actual = hashlib.sha256(raw).hexdigest()
    entry = {"bytes": len(raw), "sha256": actual}
    if expected is not None:
        entry.update(expected_sha256=expected, matched=actual == expected)
        check("hash:" + label(path), actual == expected)
    report["files"][label(path)] = entry
    return actual


def load(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError("Duplicate JSON field " + key)
            result[key] = value
        return result
    file_hash(path)
    return json.loads(path.read_bytes(), object_pairs_hook=unique)


def check_artifacts(directory, expected):
    for name, digest in expected.items():
        path = directory / name
        check("regular-bound-file:" + label(path), path.is_file() and not path.is_symlink()
              and path.resolve().is_relative_to(directory.resolve()))
        file_hash(path, digest)


def numbers(lines, prefix):
    found = []
    for line in lines:
        if line.startswith(prefix):
            match = re.fullmatch(re.escape(prefix) + r"([0-9]{20})", line)
            if not match or int(match[1]) >= 1 << 64:
                raise ValueError("Invalid numeric serial field " + prefix)
            found.append(int(match[1]))
    return found


def parse_ppm(raw):
    magic, dimensions, scale, pixels = raw.split(b"\n", 3)
    width, height = map(int, dimensions.split())
    if magic != b"P6" or scale != b"255" or (width, height) != (800, 600) or len(pixels) != width * height * 3:
        raise ValueError("Not a complete actual 800x600 P6 frame")
    return pixels


def pixel_difference(left, right):
    xs, ys = [], []
    for index in range(800 * 600):
        at = index * 3
        if left[at:at+3] != right[at:at+3]:
            xs.append(index % 800)
            ys.append(index // 800)
    return None if not xs else [min(xs), min(ys), max(xs)+1, max(ys)+1]


def verify():
    actual_head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    check("current-source-and-head", actual_head == HEAD and source_hash() == SOURCE)
    for name in ("tools/run_qsim.py", "tools/run_graphics.py", "kernel/src/m2/m3.rs",
                 "kernel/src/m2/m3/qsim.rs", "kernel/src/graphics.rs",
                 "experiments/quantum/native/src/probe.rs", "experiments/quantum/native/src/lib.rs"):
        file_hash(ROOT / name)

    qdir = ROOT / "docs/verification/parallel-final-qsim"
    accept = load(qdir / "acceptance.json")
    guest = load(qdir / "result.json")
    manifest = load(qdir / "build-manifest.json")
    artifact = load(qdir / "quantum-build-report.json")
    index = load(qdir / "artifact-index.json")
    check_artifacts(qdir, index["files"])
    qrun, inputs = Path(guest["run_directory"]), Path(accept["inputs_directory"])
    check("qsim-exact-archive-and-run-report", guest == accept["guest"] == load(qrun / "result.json"))
    check("qsim-source-binding", all(value.get("source_sha256") == SOURCE for value in (accept, guest, manifest, index))
          and guest["base_commit"] == index["source_commit"] == HEAD)
    check("qsim-exclusive-profile", manifest["kernel_feature"] == "native-qsim"
          and manifest["loader_feature"] is None and manifest["sdk_runtime"] is False)
    check("qsim-exact-artifact-set", set(guest["artifact_sha256"]) ==
          {"esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI", "OVMF_CODE.fd", "OVMF_VARS.template.fd"})
    check_artifacts(qrun, guest["artifact_sha256"])
    check("qsim-exact-input-set", set(accept["input_artifact_sha256"]) ==
          {"build-report.json", "disassembly.stdout.txt", "build-manifest.json", "qsim-native-probe.elf", "symbols.stdout.txt"})
    check_artifacts(inputs, accept["input_artifact_sha256"])
    check("qsim-build-snapshot", load(inputs / "build-manifest.json") == manifest
          and load(inputs / "build-report.json") == artifact == manifest["qsim_user"])
    for path, digest in artifact["source_sha256"].items():
        file_hash(ROOT / path, digest)
    user = (inputs / "qsim-native-probe.elf").read_bytes()
    kernel = (qrun / "esp/kernel.elf").read_bytes()
    file_hash(inputs / "qsim-native-probe.elf", artifact["binary_sha256"])
    check("qsim-exact-user-embedded-once", kernel.count(user) == 1)
    check("qsim-kernel-loader-manifest", guest["artifact_sha256"]["esp/kernel.elf"] == manifest["kernel"]["sha256"]
          and guest["artifact_sha256"]["esp/EFI/BOOT/BOOTX64.EFI"] == manifest["loader"]["sha256"])
    check("qsim-ELF-header", user[:7] == b"\x7fELF\x02\x01\x01")
    kind, machine, version, entry, phoff, _, _, ehsize, phsize, phnum, _, _, _ = struct.unpack_from("<HHIQQQIHHHHHH", user, 16)
    check("qsim-ELF-executable", (kind, machine, version, entry, ehsize, phsize) == (2, 62, 1, 0x40000000, 64, 56))
    segments = []
    for i in range(phnum):
        typ, flags, offset, address, _, size, memory, _ = struct.unpack_from("<IIQQQQQQ", user, phoff + i*phsize)
        if typ == 1 and memory:
            segments.append(dict(flags=flags, address=address, offset=offset, filesz=size, memsz=memory,
                                 page_start=address//4096*4096, page_end=(address+memory+4095)//4096*4096))
    check("qsim-real-ELF-WX-and-report", [s["flags"] for s in segments] == [5, 6, 4]
          and segments == artifact["elf"]["segments"] and all(s["offset"]+s["filesz"] <= len(user) for s in segments)
          and all(a["page_end"] <= b["page_start"] for a,b in zip(segments, segments[1:])))
    serial = (qrun / "serial.txt").read_bytes()
    stderr = (qrun / "stderr.txt").read_bytes()
    check("qsim-raw-streams-match", serial == (qdir / "serial.txt").read_bytes()
          and serial.decode("utf-8", "replace") == guest["stdout"]
          and stderr == (qdir / "stderr.txt").read_bytes() == b"" and guest["stderr"] == "")
    check("qsim-real-completion", type(guest["raw_qemu_exit"]) is int and guest["raw_qemu_exit"] == 33
          and guest["timed_out"] is False and guest["passed"] is True and accept["passed"] is True
          and 0 < guest["duration_s"] < guest["timeout_s"] <= 60)
    lines = serial.decode("utf-8", "replace").splitlines()
    start = lines.index("BOOT:EBS_OK")
    native = lines[start:]
    boot = ["BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS", "M2:PASS", "QSIM:MANIFEST:PASS"]
    cases = ["QSIM:BELL4096:PASS", "QSIM:SIX_QUBIT:PASS", "QSIM:CANCEL:PASS", "QSIM:DEADLINE:PASS", "QSIM:RESOURCE_LIMITS:PASS", "QSIM:PROCESS_ISOLATION:PASS"]
    end = ["QSIM:CLEANUP:PASS", "QSIM:PASS", "M1:PASS"]
    check("qsim-serial-grammar", len(native) == 28 and native[:8] == boot and native[16:22] == cases
          and native[25:] == end and all(lines.count(line) == 1 for line in boot + cases + end)
          and not re.search(r"(?:BOOT:|BOOTINFO:|KERNEL:|KS-|M[0-9]+:|G[0-9]+:|QSIM:)", "\n".join(lines[:start]))
          and not any(word in guest["stdout"] for word in ("FAIL", "PANIC")))
    values = {key: numbers(native, prefix) for key, prefix in (
        ("pids", "QSIM:PROCESS_PID:"), ("checkpoints", "QSIM:PROCESS_CHECKPOINTS:"),
        ("useful_arena", "QSIM:PROCESS_USEFUL_ARENA:"), ("work_gap", "QSIM:PROCESS_MAX_WORK_GAP:"),
        ("timer_irqs", "QSIM:TIMER_IRQS:"), ("syscalls", "QSIM:SYSCALL_CALLS:"), ("cleanup_frames", "QSIM:CLEANUP_FRAMES:"))}
    check("qsim-actual-process-progress-heap-cleanup", values == dict(pids=[1,2], checkpoints=[253,253], useful_arena=[7216,7216], work_gap=[448,448], timer_irqs=[4], syscalls=[505], cleanup_frames=[96]))
    report["qsim_observed"] = values
    report["qsim_user_kernel_offset"] = kernel.find(user)
    file_hash(Path(accept["qemu_entry_path"]), accept["qemu_entry_sha256"])
    for source, name in zip(guest["firmware_source"], ("OVMF_CODE.fd", "OVMF_VARS.template.fd")):
        file_hash(Path(source), guest["artifact_sha256"][name])
    check("qsim-networkless-single-CPU-profile", guest["command"][guest["command"].index("-machine")+1] == "pc-q35-7.2"
          and guest["command"][guest["command"].index("-smp")+1] == "1"
          and guest["command"][guest["command"].index("-nic")+1] == "none")

    gdir = ROOT / "docs/verification/parallel-final-g0"
    ga = load(gdir / "acceptance.json")
    g = load(gdir / "result.json")
    gmanifest = load(gdir / "build-manifest.json")
    grun = ROOT / "build/parallel-final-g0/run/graphics-g0-5ad6321b1d"
    check("g0-exact-run-and-source", Path(g["run_directory"]) == grun and g == load(grun / "result.json")
          and g["base_commit"] == ga["source_commit"] == HEAD
          and all(value["source_sha256"] == SOURCE for value in (ga, g, gmanifest)))
    check("g0-paired-build", gmanifest["kernel_feature"] == gmanifest["loader_feature"] == "graphics-g0"
          and gmanifest == load(grun / "build-manifest.json"))
    frames = ["initial.ppm", "a.ppm", "ab.ppm", "after-backspace.ppm", "typed.ppm"]
    required = set(frames) | {"qmp-trace.json", "build-manifest.json", "OVMF_CODE.fd", "OVMF_VARS.template.fd", "OVMF_VARS.fd", "serial.txt", "stderr.txt", "VGA.rom", "esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI"}
    check("g0-complete-artifact-set", set(g["artifacts"]) == required and set(ga["frame_sha256"]) == set(frames))
    check_artifacts(grun, g["artifacts"])
    check("g0-kernel-loader-manifest", g["artifacts"]["esp/kernel.elf"] == gmanifest["kernel"]["sha256"]
          and g["artifacts"]["esp/EFI/BOOT/BOOTX64.EFI"] == gmanifest["loader"]["sha256"])
    raw = [(grun/name).read_bytes() for name in frames]
    pixels = [parse_ppm(frame) for frame in raw]
    steps = [pixel_difference(a,b) for a,b in zip(pixels,pixels[1:])]
    overall = pixel_difference(pixels[0], pixels[-1])
    bounded = lambda rect: rect is not None and 48 <= rect[0] < rect[2] <= 80 and 192 <= rect[1] < rect[3] <= 212
    check("g0-real-transitions", all(step is not None for step in steps) and overall is not None)
    check("g0-input-row-only", all(bounded(rect) for rect in [*steps, overall]))
    check("g0-exact-backspace", raw[1] == raw[3] and raw[2] != raw[3])
    check("g0-nonempty-scene", len(set(pixels[0])) > 8)
    check("g0-pixel-bounds-match-report", steps == g["step_changed_bounds"] and overall == g["changed_bounds"])
    for name in frames:
        file_hash(grun/name, ga["frame_sha256"][name])
    report["g0_frames"] = dict(width=800, height=600, steps=steps, overall=overall,
                                backspace_byte_equal=True, initial_distinct_channel_values=len(set(pixels[0])))
    text = (grun/"serial.txt").read_bytes().decode("utf-8", "replace")
    check("g0-raw-serial-stderr", text == g["stdout"] and (grun/"serial.txt").read_bytes() == (gdir/"serial.txt").read_bytes()
          and (grun/"stderr.txt").read_bytes() == (gdir/"stderr.txt").read_bytes() == b"")
    expected = boot[:6] + ["G0:FRAMEBUFFER:800:600:800:1", "G0:MAPPINGS:PASS", "G0:READY", "G0:INPUT:1", "G0:INPUT:2", "G0:INPUT:1", "G0:INPUT:2", "G0:PASS", "M1:PASS"]
    glines = text.splitlines()
    check("g0-exact-native-serial", glines[glines.index("BOOT:EBS_OK"):] == expected
          and glines.count("BOOT:EBS_OK") == 1 and not any(word in text for word in ("FAIL", "PANIC")))
    check("g0-real-completion", type(g["raw_qemu_exit"]) is int and g["raw_qemu_exit"] == 33
          and g["passed"] is ga["passed"] is True and g["error"] is None and 0 < g["duration_s"] < 60)
    trace = load(grun/"qmp-trace.json")
    operations = ["qmp_capabilities", "screendump", "send-key", "screendump", "send-key", "screendump", "send-key", "screendump", "send-key", "screendump", "send-key"]
    check("g0-correlated-QMP-order", len(trace) == 11 and [item["request"]["execute"] for item in trace] == operations
          and all(type(item["request"]["id"]) is int and type(item["response"]["id"]) is int
                  and item["request"]["id"] == item["response"]["id"] == i+1
                  and item["response"] == {"return": {}, "id": i+1} for i,item in enumerate(trace)))
    keys = [item["request"]["arguments"]["keys"][0]["data"] for item in trace if item["request"]["execute"] == "send-key"]
    screenshots = [item["request"]["arguments"]["filename"] for item in trace if item["request"]["execute"] == "screendump"]
    check("g0-QMP-keys-and-actual-frame-paths", keys == ["a", "b", "backspace", "c", "esc"] and screenshots == [str(grun/name) for name in frames])
    pin = g["vga_rom"]
    check("g0-ROM-explicit-pin", pin["expected_hash_checked"] is True and pin["copied_path"] == "VGA.rom"
          and pin["sha256"] == pin["expected_sha256"] == ga["rom_sha256"] == g["artifacts"]["VGA.rom"]
          and "VGA,vgamem_mb=16,romfile=VGA.rom" in g["command"])
    file_hash(Path(pin["source_path"]), pin["expected_sha256"])
    wrong = ga["wrong_rom_pin"]
    check("g0-stored-negative-ROM-check", wrong["rejected"] is True and wrong["vm_started"] is False
          and wrong["expected_sha256"] == "0"*64 and wrong["actual_sha256"] == pin["sha256"]
          and wrong["error"] == "VGA ROM does not match the explicitly pinned SHA256")
    file_hash(Path(g["qemu_entry_path"]), g["qemu_entry_sha256"])
    check("shared-firmware-bytes", all(g["artifacts"][name] == guest["artifact_sha256"][name]
                                      for name in ("OVMF_CODE.fd", "OVMF_VARS.template.fd")))
    check("source-still-unchanged", source_hash() == SOURCE)


if __name__ == "__main__":
    try:
        verify()
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    report["passed"] = "error" not in report and all(report["checks"].values())
    report["finished_utc"] = datetime.now(timezone.utc).isoformat()
    report["verification_script_sha256"] = file_hash(Path(__file__))
    (OUT/"result.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"passed": report["passed"], "checks": len(report["checks"]), "files": len(report["files"]),
                      "failed": [name for name,value in report["checks"].items() if not value], "error": report.get("error")}, indent=2))
    raise SystemExit(0 if report["passed"] else 1)
