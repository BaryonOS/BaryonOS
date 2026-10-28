#!/usr/bin/env python3
"""Recheck a finished G0 run and atomically publish a new text/PNG archive.

No VM, repaired log, raw binary, firmware or disk image is produced. Linux
renameat2(RENAME_NOREPLACE) is required for publication without replacement.
PNG is the bounded RGB8/filter-0 subset of W3C PNG, 24 June 2025:
https://www.w3.org/TR/2025/REC-png-3-20250624/
"""
from __future__ import annotations

import argparse
import ctypes
from datetime import datetime, timezone
import errno
import hashlib
import json
import math
import os
from pathlib import Path
import re
import stat
import struct
import tempfile
import zlib

from common import ROOT, QEMU_MACHINES, sha256, source_hash
from run_graphics import frame_checks, ppm, valid_sequence

FRAMES = ("initial", "a", "ab", "after-backspace", "typed")
PIXEL_LIMIT = 1920 * 1080 * 3
LIMITS = {
    **{name + ".ppm": PIXEL_LIMIT + 128 for name in FRAMES},
    "result.json": 1024 * 1024,
    "build-manifest.json": 1024 * 1024,
    "qmp-trace.json": 128 * 1024,
    "serial.txt": 128 * 1024,
    "stderr.txt": 128 * 1024,
    "OVMF_CODE.fd": 8 * 1024 * 1024,
    "OVMF_VARS.template.fd": 8 * 1024 * 1024,
    "OVMF_VARS.fd": 8 * 1024 * 1024,
    "esp/kernel.elf": 32 * 1024 * 1024,
    "esp/EFI/BOOT/BOOTX64.EFI": 8 * 1024 * 1024,
    "VGA.rom": 1024 * 1024,
}
BASE_ARTIFACTS = frozenset(LIMITS) - {"result.json", "VGA.rom"}
CHECKS = frozenset(("real_key_changes_visible_pixels", "only_input_row_changes",
    "backspace_restores_previous_frame_exactly", "nonempty_native_scene",
    "real_exit33", "post_ebs_kernel_m2", "exact_g0_sequence", "no_guest_failure",
    "source_unchanged", "normal_end_marker"))
REPORT_FIELDS = frozenset(("scope", "passed", "checks", "error", "command", "cwd", "machine",
    "qmp_transport", "build_directory", "raw_qemu_exit", "duration_s", "source_fingerprint_version",
    "source_sha256", "base_commit", "qemu_version", "qemu_entry_path", "qemu_entry_sha256",
    "run_directory", "stdout", "artifacts", "changed_bounds", "step_changed_bounds", "vga_rom"))
TEXT_FILES = ("result.json", "build-manifest.json", "serial.txt", "stderr.txt", "qmp-trace.json")
ARCHIVE_FILES = frozenset(TEXT_FILES) | {name + ".png" for name in FRAMES} | {"image-roundtrip.json"}
PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def read_json(raw, *, object_required=True):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, "Duplicate JSON field")
            result[key] = value
        return result
    def invalid(_):
        raise ValueError("Non-finite JSON value")
    value = json.loads(raw, object_pairs_hook=unique, parse_constant=invalid)
    require(not object_required or isinstance(value, dict), "Expected a JSON object")
    return value


def directory(path):
    path = Path(path).absolute()
    require(".." not in path.parts and path.resolve(strict=True) == path and path.is_dir(),
            "Directory must be canonical and cannot traverse a symlink")
    return path


def read_file(root, name, limit):
    """Open every component without following links; reject devices/hardlinks."""
    path = Path(name)
    require(not path.is_absolute() and path.parts and all(p not in (".", "..") for p in path.parts),
            "Artifact path escapes its run")
    flags = os.O_RDONLY | os.O_NOFOLLOW
    fd = os.open(root, flags | os.O_DIRECTORY)
    try:
        for part in path.parts[:-1]:
            child = os.open(part, flags | os.O_DIRECTORY, dir_fd=fd)
            os.close(fd)
            fd = child
        child = os.open(path.name, flags | os.O_NONBLOCK, dir_fd=fd)
        with os.fdopen(child, "rb") as stream:
            before = os.fstat(stream.fileno())
            require(stat.S_ISREG(before.st_mode) and before.st_nlink == 1 and before.st_size <= limit,
                    "Artifact must be a bounded regular single-link file")
            raw = stream.read(limit + 1)
            after = os.fstat(stream.fileno())
            identity = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_ctime_ns)
            require(len(raw) == before.st_size and identity(before) == identity(after),
                    "Artifact changed while reading")
            return raw, identity(after)
    finally:
        os.close(fd)


def snapshot(root, names, transport):
    root = directory(root)
    allowed = set(names)
    directories = {"esp", "esp/EFI", "esp/EFI/BOOT"}
    seen = set()
    for path in root.rglob("*"):
        name = path.relative_to(root).as_posix()
        mode = path.lstat().st_mode
        require(not stat.S_ISLNK(mode), "Symlink in run artifacts")
        if stat.S_ISDIR(mode):
            require(name in directories, "Unexpected run directory")
        elif stat.S_ISSOCK(mode):
            require(transport == "unix" and name == "qmp.sock", "Unexpected run socket")
        else:
            require(name in allowed and stat.S_ISREG(mode), "Unexpected run artifact")
            seen.add(name)
    require(seen == allowed, "Missing run artifact")
    raw, identities = {}, {}
    for name in sorted(allowed):
        raw[name], identities[name] = read_file(root, name, LIMITS[name])
    return raw, identities


def expected_command(report, pinned_rom):
    machine, transport, executable = report.get("machine"), report.get("qmp_transport"), report.get("qemu_entry_path")
    require(machine in QEMU_MACHINES and transport in ("pipe", "unix"), "Unknown QEMU profile")
    require(isinstance(executable, str) and Path(executable).is_absolute()
            and Path(executable).name == "qemu-system-x86_64", "Invalid QEMU entry path")
    return [executable, "-machine", machine, "-accel", "tcg,thread=single", "-cpu", "qemu64",
        "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
        "-serial", "file:serial.txt" if transport == "pipe" else "stdio", "-nic", "none", "-no-reboot",
        "-device", "VGA,vgamem_mb=16" + (",romfile=VGA.rom" if pinned_rom else ""),
        "-qmp", "stdio" if transport == "pipe" else "unix:qmp.sock,server=on,wait=off",
        "-drive", "if=pflash,unit=0,format=raw,readonly=on,file=OVMF_CODE.fd",
        "-drive", "if=pflash,unit=1,format=raw,file=OVMF_VARS.fd",
        "-drive", "if=virtio,index=0,format=raw,readonly=on,file=fat:esp",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]


def validate_trace(raw, root):
    trace = read_json(raw, object_required=False)
    require(isinstance(trace, list) and len(trace) == 11, "Incomplete QMP transcript")
    requests = [("qmp_capabilities", None)]
    for index, name in enumerate(FRAMES):
        if index:
            key = ("a", "b", "backspace", "c")[index-1]
            requests.append(("send-key", {"keys": [{"type": "qcode", "data": key}], "hold-time": 30}))
        requests.append(("screendump", {"filename": str(root / (name + ".ppm"))}))
    requests.append(("send-key", {"keys": [{"type": "qcode", "data": "esc"}], "hold-time": 30}))
    for sequence, (item, (operation, arguments)) in enumerate(zip(trace, requests), 1):
        request = {"execute": operation, "id": sequence}
        if arguments is not None:
            request["arguments"] = arguments
        require(isinstance(item, dict) and set(item) == {"request", "response"}, "Malformed QMP transcript item")
        supplied, response = item["request"], item["response"]
        require(isinstance(supplied, dict) and isinstance(response, dict)
                and type(supplied.get("id")) is int and type(response.get("id")) is int
                and supplied == request and response == {"return": {}, "id": sequence},
                "Uncorrelated QMP command, foreign path or unexpected response")


def validate(root, frozen):
    root = directory(root)
    first, _ = read_file(root, "result.json", LIMITS["result.json"])
    report = read_json(first)
    require(set(report) == REPORT_FIELDS, "Unexpected or missing G0 result field")
    require(report.get("passed") is True and report.get("error") is None
            and type(report.get("raw_qemu_exit")) is int and report["raw_qemu_exit"] == 33,
            "G0 needs a real successful completion without an error")
    require(isinstance(report.get("checks"), dict) and set(report["checks"]) == CHECKS
            and all(value is True for value in report["checks"].values()), "Missing or failed G0 check")
    duration = report.get("duration_s")
    require(type(duration) in (int, float) and math.isfinite(duration) and 0 < duration <= 60, "Invalid G0 duration")
    require(type(report.get("source_fingerprint_version")) is int
            and report["source_fingerprint_version"] == 3 and report.get("source_sha256") == frozen,
            "Run does not bind the current source")
    require(isinstance(report.get("base_commit"), str) and re.fullmatch("[0-9a-f]{40}", report["base_commit"]),
            "Missing source commit")
    require(report.get("run_directory") == report.get("cwd") == str(root), "Run directory or cwd substitution")
    rom = report.get("vga_rom")
    expected = BASE_ARTIFACTS | ({"VGA.rom"} if rom is not None else set())
    require(isinstance(report.get("artifacts"), dict) and set(report["artifacts"]) == expected,
            "Missing or unexpected artifact declaration")
    require(all(isinstance(value, str) and re.fullmatch("[0-9a-f]{64}", value)
                for value in report["artifacts"].values()), "Malformed artifact hash")
    require(report.get("command") == expected_command(report, rom is not None), "Not the private networkless G0 command")
    raw, identities = snapshot(root, expected | {"result.json"}, report["qmp_transport"])
    require(raw["result.json"] == first, "Result changed during capture")
    for name, expected_hash in report["artifacts"].items():
        require(digest(raw[name]) == expected_hash, "Changed artifact: " + name)
    manifest = read_json(raw["build-manifest.json"])
    require(type(manifest.get("source_fingerprint_version")) is int
            and manifest["source_fingerprint_version"] == 3 and manifest.get("source_sha256") == frozen
            and manifest.get("kernel_feature") == manifest.get("loader_feature") == "graphics-g0",
            "Wrong graphics build manifest")
    require(isinstance(manifest.get("kernel"), dict) and isinstance(manifest.get("loader"), dict)
            and manifest["kernel"].get("sha256") == digest(raw["esp/kernel.elf"])
            and manifest.get("loader", {}).get("sha256") == digest(raw["esp/EFI/BOOT/BOOTX64.EFI"]),
            "Kernel or loader does not match the build manifest")
    if rom is not None:
        require(isinstance(rom, dict) and set(rom) == {"source_path", "copied_path", "sha256", "expected_sha256", "expected_hash_checked"},
                "Malformed ROM record")
        require(rom["copied_path"] == "VGA.rom" and rom["sha256"] == digest(raw["VGA.rom"]), "ROM substitution")
        require(0 < len(raw["VGA.rom"]) <= LIMITS["VGA.rom"], "Empty or oversized VGA ROM")
        checked = rom["expected_hash_checked"]
        require(type(checked) is bool and ((checked and rom["expected_sha256"] == rom["sha256"])
                or (not checked and rom["expected_sha256"] is None)), "Claimed ROM pin was not checked")
    executable = Path(report["qemu_entry_path"])
    tool_raw, tool_identity = read_file(directory(executable.parent), executable.name, 64*1024*1024)
    require(digest(tool_raw) == report.get("qemu_entry_sha256"), "QEMU entry changed")
    text = raw["serial.txt"].decode("utf-8", "strict")
    require(isinstance(report.get("stdout"), str) and report["stdout"].encode("utf-8") == raw["serial.txt"],
            "Actual serial bytes differ from reported stdout")
    raw["stderr.txt"].decode("utf-8", "strict")
    frames = [raw[name + ".ppm"] for name in FRAMES]
    width, height, _ = ppm(frames[0])
    checks, bounds, steps = frame_checks(*frames)
    require(all(checks.values()) and valid_sequence(text, width, height), "Actual G0 frame or serial check failed")
    lines = text.splitlines()
    handoff = lines.index("BOOT:EBS_OK")
    expected_lines = ["BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS", "KS-03:PASS",
        next(line for line in lines if line.startswith("G0:FRAMEBUFFER:")), "G0:MAPPINGS:PASS", "G0:READY",
        "G0:INPUT:1", "G0:INPUT:2", "G0:INPUT:1", "G0:INPUT:2", "G0:PASS", "M1:PASS"]
    require(lines[handoff:] == expected_lines and not any(word in text for word in ("FAIL", "PANIC"))
            and not re.search(r"(?:BOOT:|BOOTINFO:|KERNEL:|KS-|M[0-9]+:|G[0-9]+:|QSIM:)", "\n".join(lines[:handoff])),
            "Unexpected, missing or failed native serial line")
    require(report.get("changed_bounds") == list(bounds)
            and report.get("step_changed_bounds") == [list(step) for step in steps], "Changed frame-bound report")
    validate_trace(raw["qmp-trace.json"], root)
    return report, raw, identities, (executable, tool_identity, digest(tool_raw))


def png_chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))


def encode_png(frame):
    """Standard RGB8, no interlace, filter 0; no color/sample conversion."""
    width, height, pixels = ppm(frame)
    stride = width*3
    scanlines = b"".join(b"\0" + pixels[at:at+stride] for at in range(0, len(pixels), stride))
    return (PNG_SIGNATURE + png_chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
            + png_chunk(b"IDAT", zlib.compress(scanlines)) + png_chunk(b"IEND", b""))


def decode_png(raw):
    """Re-read our canonical PNG subset with bounded decompression and CRCs."""
    require(len(raw) <= PIXEL_LIMIT + 65536 and raw.startswith(PNG_SIGNATURE), "Invalid bounded PNG")
    offset, chunks = 8, []
    for expected in (b"IHDR", b"IDAT", b"IEND"):
        require(offset + 12 <= len(raw), "Truncated PNG chunk")
        length = struct.unpack_from(">I", raw, offset)[0]
        end = offset + 12 + length
        require(end <= len(raw), "PNG chunk overflow")
        kind, data = raw[offset+4:offset+8], raw[offset+8:end-4]
        require(kind == expected and struct.unpack_from(">I", raw, end-4)[0] == zlib.crc32(kind+data),
                "Wrong PNG chunk or checksum")
        chunks.append(data)
        offset = end
    require(offset == len(raw) and len(chunks[0]) == 13 and chunks[2] == b"", "Trailing PNG data or invalid header")
    width, height, depth, color, compression, filtering, interlace = struct.unpack(">IIBBBBB", chunks[0])
    require(1 <= width <= 1920 and 1 <= height <= 1080
            and (depth, color, compression, filtering, interlace) == (8, 2, 0, 0, 0), "Unsupported PNG profile")
    stride, amount = width*3+1, (width*3+1)*height
    decoder = zlib.decompressobj()
    pixels = decoder.decompress(chunks[1], amount+1)
    require(len(pixels) == amount and decoder.eof and not decoder.unused_data and not decoder.unconsumed_tail,
            "Wrong or excessive decompressed PNG data")
    require(all(pixels[at] == 0 for at in range(0, amount, stride)), "Unexpected PNG filter")
    return width, height, b"".join(pixels[at+1:at+stride] for at in range(0, amount, stride))


def archive_destination(destination):
    parent = directory(ROOT / "docs/verification")
    target = Path(destination)
    if not target.is_absolute():
        target = ROOT / target
    require(".." not in target.parts and target.parent == parent
            and re.fullmatch(r"[a-z0-9][a-z0-9-]{0,80}", target.name), "Destination must be one named direct child of docs/verification")
    require(not target.exists() and not target.is_symlink(), "Destination already exists")
    return target


def publish_new(staged, destination):
    """Atomic Linux no-replace rename; there is no overwriting fallback."""
    require(directory(staged) == staged, "Staged archive cannot be a symlink")
    require(directory(staged.parent) == directory(destination.parent), "Archive must stage in its destination parent")
    libc = ctypes.CDLL(None, use_errno=True)
    rename = getattr(libc, "renameat2", None)
    if rename is None:
        raise RuntimeError("Atomic new-only capture requires Linux renameat2")
    rename.argtypes = (ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint)
    rename.restype = ctypes.c_int
    parent_fd = os.open(destination.parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        if rename(parent_fd, os.fsencode(staged.name), parent_fd, os.fsencode(destination.name), 1) != 0:
            error = ctypes.get_errno()
            if error == errno.EEXIST:
                raise ValueError("Destination already exists")
            raise OSError(error, "Atomic new-only archive publication failed")
    finally:
        os.close(parent_fd)


def capture(run_directory, destination):
    target = archive_destination(destination)
    root = directory(run_directory)
    frozen = source_hash()
    report, raw, identities, tool = validate(root, frozen)
    # TemporaryDirectory uses the same parent/filesystem as the atomic rename.
    with tempfile.TemporaryDirectory(prefix=".g0-capture-", dir=target.parent) as temporary:
        staged = Path(temporary)
        for name in TEXT_FILES:
            (staged/name).write_bytes(raw[name])
        images = []
        for name in FRAMES:
            original = raw[name+".ppm"]
            width, height, pixels = ppm(original)
            header = original[:-len(pixels)]
            path = staged/(name+".png")
            path.write_bytes(encode_png(original))
            decoded_width, decoded_height, decoded = decode_png(path.read_bytes())
            require((width, height) == (decoded_width, decoded_height) and header+decoded == original,
                    "PNG does not reconstruct the exact original PPM")
            images.append(dict(name=name, width=width, height=height, original_header_hex=header.hex(),
                               ppm_sha256=digest(original), png_sha256=sha256(path)))
        (staged/"image-roundtrip.json").write_text(json.dumps({"scope": "Lossless actual QEMU pixels; exact PPM header plus RGB roundtrip", "images": images}, indent=2)+"\n")
        require({path.name for path in staged.iterdir()} == ARCHIVE_FILES,
                "Unexpected or missing staged archive file")
        captured = dict(schema_version=1, passed=True, source_fingerprint_version=3, source_sha256=frozen,
            source_commit=report["base_commit"], run_directory=str(root), captured_utc=datetime.now(timezone.utc).isoformat(),
            scope="Existing raw G0 artifacts rechecked; no VM, repaired result, binary or firmware copied",
            input_sha256={name:digest(value) for name,value in raw.items()},
            files={name:sha256(staged/name) for name in sorted(ARCHIVE_FILES)})
        (staged/"capture.json").write_text(json.dumps(captured, indent=2)+"\n")
        again, again_identities = snapshot(root, raw, report["qmp_transport"])
        executable, identity, tool_hash = tool
        tool_raw, final_identity = read_file(directory(executable.parent), executable.name, 64*1024*1024)
        require(again == raw and again_identities == identities and final_identity == identity
                and digest(tool_raw) == tool_hash and source_hash() == frozen, "Inputs or source changed before archive publication")
        require(all(path.is_file() and not path.is_symlink() and path.stat().st_nlink == 1 for path in staged.iterdir())
                and {path.name for path in staged.iterdir()} == ARCHIVE_FILES | {"capture.json"}
                and all(sha256(staged/name) == expected for name,expected in captured["files"].items())
                and read_json((staged/"capture.json").read_bytes()) == captured,
                "Staged archive changed before publication")
        archive_destination(target)
        publish_new(staged, target)
    print(f"G0 capture passed: five exact PPM/PNG roundtrips; {target}")
    return target


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-dir", required=True, type=Path, help="Existing completed G0 run directory")
    parser.add_argument("--destination", required=True, type=Path, help="New direct child of docs/verification; never replaced")
    arguments = parser.parse_args()
    capture(arguments.run_dir, arguments.destination)
