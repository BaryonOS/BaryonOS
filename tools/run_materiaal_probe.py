#!/usr/bin/env python3
"""Real QMP keys/mouse and pixels for the bounded native Materiaal chain.

Requires the separately integrated eight-process diagnostic profile. No guest
memory writes, host document implementation, user disk, network or fake pass.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import time
import uuid

from common import ROOT, BUILD, environment, firmware, qemu_machine, sha256, source_hash
from build_users import inspect_user
from run_fpu import private_path, publish, publish_json, load_json
from run_graphics import Qmp, QmpPipe, ppm, changed_bounds
from run_display_probe import BOOT, external_file, require, SERIAL_LIMIT
from run_system import check_image
import volume_disk as volume

FEATURES = {"normal": "native-system-materiaal-probe", "recover": "test-system-materiaal-recover"}
NAMES = ("desktop-init", "keyring-service", "volume-service", "materiaal-app", "native-compositor",
         "input-source", "input-policy", "desktop-controller")
GATES = ("MATERIAAL:SOURCE:READY", "MATERIAAL:POLICY:READY", "MATERIAAL:COMPOSITOR:READY", "MATERIAAL:FOCUS:READY")
FIELDS = ("READY", "PRESENTED_GEN", "FRAMES", "TILES", "KEYS", "MOUSE", "INPUT_ACKS", "SAVE_ACKS",
          "SAVED_REVISION", "SAVED_BYTES", "SAVED_SUM", "SCREEN", "DRAFT_DIRTY", "DRAFT_BYTES", "DRAFT_SUM",
          "OBJECT", "SCALE", "ERRORS")
FINAL_FIELDS = ("DISK_READS", "DISK_WRITES", "DISK_FLUSHES", "LIVE_PROCESSES", "LIVE_HANDLES", "FRAMES_RECOVERED")
END = ("MATERIAAL:CLEANUP:PASS", "MATERIAAL:CASE:PASS", "M1:PASS")
PHASES = {"normal": ("ready", "editor", "dirty", "saved", "list", "reopened"),
          "recover": ("ready", "reopened", "readonly")}
TEXT = b"nieuw os werkt"
TEXT_SUM = volume.fnv(TEXT)
U64_MAX = (1 << 64) - 1
HEX = re.compile(r"[0-9a-f]{64}")
WIDTH, HEIGHT = 800, 600
ASSIGNED = (16, 48, 768, 536)
TILES = ((ASSIGNED[2] + 31) // 32) * ((ASSIGNED[3] + 7) // 8)


def numeric(line: str, name: str) -> int:
    match = re.fullmatch(re.escape("MATERIAAL:" + name + ":") + r"([0-9]{20})", line)
    require(match is not None and int(match[1]) <= U64_MAX, "Malformed numeric field " + name)
    return int(match[1])


def snapshot_checks(current: dict, previous: dict | None, case: str) -> None:
    require(current["READY"] == 1 and current["PRESENTED_GEN"] > 0 and current["FRAMES"] > 0
            and current["TILES"] >= current["FRAMES"] * TILES and current["SCALE"] == 1
            and current["SCREEN"] in (1, 2, 3) and current["DRAFT_DIRTY"] in (0, 1)
            and current["INPUT_ACKS"] >= current["KEYS"] + current["MOUSE"]
            and current["SAVE_ACKS"] in (0, 1), "Invalid actual frame/counter snapshot")
    require(current["ERRORS"] == 0 if case == "normal" else current["ERRORS"] <= 1,
            "Unexpected app error")
    if current["SAVE_ACKS"]:
        require((current["SAVED_REVISION"], current["SAVED_BYTES"], current["SAVED_SUM"])
                == (1, len(TEXT), TEXT_SUM), "Wrong actual save response")
    else:
        require((current["SAVED_REVISION"], current["SAVED_BYTES"], current["SAVED_SUM"]) == (0, 0, 0),
                "Saved metadata without an actual COMMIT ACK")
    if case == "recover":
        require(current["SAVE_ACKS"] == 0 and current["DRAFT_DIRTY"] == 0, "Readonly app changed draft or committed")
    if previous is not None:
        require(current["PRESENTED_GEN"] > previous["PRESENTED_GEN"] and current["FRAMES"] > previous["FRAMES"]
                and current["TILES"] >= previous["TILES"] + (current["FRAMES"]-previous["FRAMES"])*TILES, "Repeated/stale/incomplete presented frame")
        require(all(current[key] >= previous[key] for key in ("KEYS", "MOUSE", "INPUT_ACKS", "SAVE_ACKS", "ERRORS")),
                "Regressing actual counters")


def transcript(raw: bytes, case="normal", *, final=False) -> dict:
    """Strict incremental grammar. Only complete newline-terminated records count."""
    require(case in FEATURES and isinstance(raw, bytes) and len(raw) <= SERIAL_LIMIT
            and not any(word in raw for word in (b"FAIL", b"PANIC", b"\0")), "Failed/oversized native transcript")
    text = raw.decode("utf-8", "strict")
    lines = text.splitlines()
    if text and not text.endswith("\n"):
        require(not final, "Truncated final native record")
        lines = lines[:-1]
    result = {"gates": [], "snapshots": [], "exits": [], "barrier": None, "controller_released": False, "final": {}, "finished": False}
    if BOOT[0] not in lines:
        require(not final and not any(re.search(r"(?:BOOT:|BOOTINFO:|KERNEL:|KS-|M[0-9]+:|MATERIAAL:)", s) for s in lines),
                "Native marker before boot handoff or missing boot")
        return result
    start = lines.index(BOOT[0])
    require(not any(re.search(r"(?:BOOT:|BOOTINFO:|KERNEL:|KS-|M[0-9]+:|MATERIAAL:)", s) for s in lines[:start]),
            "Native marker before handoff")
    lines = lines[start:]
    opening = BOOT + ("MATERIAAL:MANIFEST:PASS",)
    require(tuple(lines[:len(opening)]) == opening[:len(lines)], "Wrong boot/manifest sequence")
    if len(lines) < len(opening):
        require(not final, "Truncated boot")
        return result
    at = len(opening)
    while at < len(lines):
        line = lines[at]
        if line in GATES:
            require(not result["snapshots"] and line not in result["gates"], "Late or duplicate readiness gate")
            if line == GATES[3]:
                require(set(result["gates"]) == set(GATES[:3]), "Focus precedes ready services")
            result["gates"].append(line)
            at += 1
        elif line == "MATERIAAL:SNAPSHOT:BEGIN":
            require(set(result["gates"]) == set(GATES) and not result["exits"] and result["barrier"] is None,
                    "Snapshot outside the live ready-service interval")
            available = min(len(FIELDS), len(lines) - at - 1)
            values = {name: numeric(lines[at + 1 + i], name) for i, name in enumerate(FIELDS[:available])}
            if len(lines) < at + len(FIELDS) + 2:
                require(not final, "Truncated snapshot")
                break
            require(lines[at + len(FIELDS) + 1] == "MATERIAAL:SNAPSHOT:END", "Missing or altered snapshot end")
            snapshot_checks(values, result["snapshots"][-1] if result["snapshots"] else None, case)
            result["snapshots"].append(values)
            at += len(FIELDS) + 2
        elif line.startswith("MATERIAAL:F12_BARRIER:"):
            require(result["barrier"] is None and not result["exits"] and phases(result, case, complete=True),
                    "Premature/repeated F12 closure barrier")
            result["barrier"] = numeric(line, "F12_BARRIER")
            require(result["barrier"] == 1, "F12 must be one real new Press")
            at += 1
        elif line.startswith("MATERIAAL:EXIT_SLOT:"):
            require(result["barrier"] == 1 and result["controller_released"], "Process exited before observed F12/controller release")
            slot = numeric(line, "EXIT_SLOT")
            require(slot in (0, 3, 7) and slot not in result["exits"], "Wrong or repeated process exit")
            if at + 3 > len(lines):
                require(not final, "Truncated exit")
                break
            require(numeric(lines[at + 1], "EXIT_CODE") == 0 and numeric(lines[at + 2], "EXIT_STEP") == 0,
                    "Nonclean native process exit")
            result["exits"].append(slot)
            at += 3
        elif line == "MATERIAAL:CONTROLLER:RELEASED":
            require(result["barrier"] == 1 and not result["controller_released"] and not result["exits"],
                    "Premature/duplicate test controller release")
            result["controller_released"] = True
            at += 1
        elif line.startswith("MATERIAAL:" + FINAL_FIELDS[0] + ":"):
            require(set(result["exits"]) == {0, 3, 7} and not result["final"], "Final counters before real cleanup")
            available = min(len(FINAL_FIELDS), len(lines) - at)
            values = {name: numeric(lines[at + i], name) for i, name in enumerate(FINAL_FIELDS[:available])}
            if len(lines) < at + len(FINAL_FIELDS) + len(END):
                require(not final, "Truncated final counters/end")
                break
            require(tuple(lines[at + len(FINAL_FIELDS):]) == END, "Wrong or extra final native markers")
            require(values["DISK_READS"] > 0 and values["LIVE_PROCESSES"] == 0 and values["LIVE_HANDLES"] == 0
                    and values["FRAMES_RECOVERED"] == 1, "Incomplete actual native cleanup")
            require((values["DISK_WRITES"] > 0 and values["DISK_FLUSHES"] >= 2) if case == "normal"
                    else values["DISK_WRITES"] == values["DISK_FLUSHES"] == 0, "Wrong writable/readonly disk activity")
            result["final"] = values
            result["finished"] = True
            at = len(lines)
        else:
            raise ValueError("Unknown native record: " + line[:120])
    require(not final or result["finished"], "Incomplete actual native run")
    return result


def phase_matches(name: str, value: dict, case: str) -> bool:
    same = (value["DRAFT_BYTES"], value["DRAFT_SUM"], value["OBJECT"]) == (len(TEXT), TEXT_SUM, 0)
    screen, dirty, saves = value["SCREEN"], value["DRAFT_DIRTY"], value["SAVE_ACKS"]
    if name == "ready":
        return screen == 1 and dirty == saves == 0 and (value["DRAFT_BYTES"], value["DRAFT_SUM"], value["OBJECT"]) == (0, 0, U64_MAX)
    if name == "editor":
        return screen == 2 and dirty == 1 and saves == 0 and (value["DRAFT_BYTES"], value["DRAFT_SUM"], value["OBJECT"]) == (0, volume.FNV_OFFSET, 0)
    if name == "dirty":
        return screen == 2 and dirty == 1 and saves == 0 and same
    if name in ("saved", "reopened"):
        return screen == 2 and dirty == 0 and saves == (1 if case == "normal" else 0) and same and value["ERRORS"] == 0
    if name == "list":
        return screen == 1 and dirty == 0 and saves == 1 and same
    if name == "readonly":
        return screen == 2 and dirty == saves == 0 and same and value["ERRORS"] == 1
    return False


def phases(parsed: dict, case: str, *, complete=False) -> dict:
    selected = {}
    wanted = iter(PHASES[case])
    name = next(wanted, None)
    for snapshot in parsed["snapshots"]:
        if name and phase_matches(name, snapshot, case):
            selected[name] = snapshot
            name = next(wanted, None)
    require(not complete or name is None, "Missing actual ordered UI phase")
    return selected


def qmp_groups(directory: Path, case: str) -> list[tuple[str, list[dict]]]:
    require(case in FEATURES, "Unknown GUI case")
    def key(*codes):
        return {"execute": "send-key", "arguments": {"keys": [{"type": "qcode", "data": c} for c in codes], "hold-time": 30}}
    def shot(name):
        return {"execute": "screendump", "arguments": {"filename": str(directory / f"{name}.ppm")}}
    groups = [("ready", [shot("ready")])]
    if case == "normal":
        # The source-checked ui-core Cursor::new starts at global (0,0). Move
        # without buttons, in bounded relative steps, before a fresh click.
        x, y = 0, 0
        target_x, target_y = ASSIGNED[0] + 40, ASSIGNED[1] + 102
        moves = []
        while (x, y) != (target_x, target_y):
            dx, dy = max(-96, min(96, target_x-x)), max(-96, min(96, target_y-y))
            events = [{"type": "rel", "data": {"axis": axis, "value": value}} for axis, value in (("x", dx), ("y", dy)) if value]
            moves.append({"execute": "input-send-event", "arguments": {"events": events}})
            x += dx; y += dy
        for down in (True, False):
            moves.append({"execute": "input-send-event", "arguments": {"events": [{"type": "btn", "data": {"down": down, "button": "left"}}]}})
        groups += [("editor", moves + [shot("editor")]),
                   ("dirty", [key("spc" if c == " " else c) for c in TEXT.decode()] + [shot("dirty")]),
                   ("saved", [key("ctrl", "s"), shot("saved")]),
                   ("list", [key("esc"), shot("list")]),
                   ("reopened", [key("tab"), key("tab"), key("ret"), shot("reopened")])]
    else:
        groups += [("reopened", [key("tab"), key("ret"), shot("reopened")]),
                   ("readonly", [key("ctrl", "s"), shot("readonly")])]
    groups.append(("close", [key("f12")]))
    return groups


def qmp_requests(directory: Path, case: str) -> list[dict]:
    requests = [{"execute": "qmp_capabilities"}] + [r for _, group in qmp_groups(directory, case) for r in group]
    return [{**request, "id": i} for i, request in enumerate(requests, 1)]


def validate_qmp(trace: object, directory: Path, case: str) -> None:
    expected = qmp_requests(directory, case)
    require(isinstance(trace, list) and len(trace) == len(expected), "Missing/extra QMP hardware operation")
    for actual, request in zip(trace, expected):
        require(isinstance(actual, dict) and set(actual) == {"request", "response"}
                and json.dumps(actual["request"], sort_keys=True, allow_nan=False) == json.dumps(request, sort_keys=True, allow_nan=False)
                and json.dumps(actual["response"], sort_keys=True, allow_nan=False)
                == json.dumps({"return": {}, "id": request["id"]}, sort_keys=True), "Altered, uncorrelated or failed QMP command")


def disk_oracle(raw: bytes) -> dict:
    """Reuse the independent volume decoder, but check the actual typed bytes.
    volume.analyse intentionally accepts only its old generated binary fixture;
    do not relabel typed text as that fixture or skip checksums/canaries here.
    """
    require(len(raw) == volume.IMAGE_SECTORS * volume.SECTOR, "Wrong generated disk geometry")
    require(all(raw[i*volume.SECTOR:(i+1)*volume.SECTOR] == volume.canary(i)
                for i in range(volume.IMAGE_SECTORS) if not volume.FIRST <= i < volume.FIRST+volume.VOLUME_SECTORS),
            "Write outside granted test volume")
    data = raw[volume.FIRST*volume.SECTOR:(volume.FIRST+volume.VOLUME_SECTORS)*volume.SECTOR]
    table = volume.choose(data)
    item = table["entries"][0]
    require(table["generation"] == 1 and table["clocks"] == [1, 0] and item["present"]
            and item["revision"] == 1 and not item["history"] and item["collection"] == item["kind"] == item["tags"] == 0,
            "Wrong committed object metadata/history")
    version = item["current"]
    require(version["revision"] == 1 and version["length"] == len(TEXT) and version["name"] == b"Nieuw document"
            and volume.content(data, version) == TEXT, "Stored bytes/name/revision differ from real typed fixture")
    require(all(e == volume.empty_entry() for e in table["entries"][1:]), "Another object/domain changed")
    require(all(not c["present"] and c["revision"] == c["request_id"] == 0 and c["name"] == b"" for c in table["collections"]),
            "Unrelated collection changed")
    return {"object": 0, "revision": 1, "name": "Nieuw document", "length": len(TEXT), "sha256": hashlib.sha256(TEXT).hexdigest(),
            "fnv": TEXT_SUM, "generation": 1, "clocks": [1, 0], "canaries_intact": True}


def validate_frames(frames: dict[str, bytes], case: str) -> dict:
    require(set(frames) == set(PHASES[case]), "Missing or extra captured UI stage")
    decoded = {name: ppm(raw) for name, raw in frames.items()}
    require(all((w, h) == (WIDTH, HEIGHT) for w, h, _ in decoded.values()), "Wrong real GOP development geometry")
    names = PHASES[case]
    base = decoded[names[0]][2]
    require(len({base[i:i+3] for i in range(0,len(base),3)}) > 8, "Empty/unrendered native scene")
    x, y, width, height = ASSIGNED
    for name in names[1:]:
        pixels = decoded[name][2]
        for row in range(HEIGHT):
            start, end = row*WIDTH*3, (row+1)*WIDTH*3
            left, right = start+x*3, start+(x+width)*3
            require((base[start:end] == pixels[start:end]) if not y <= row < y+height else
                    base[start:left] == pixels[start:left] and base[right:end] == pixels[right:end],
                    "App paint escaped its assigned surface/system chrome changed")
    transitions = {f"{a}->{b}": changed_bounds(frames[a], frames[b]) for a,b in zip(names,names[1:])}
    require(all(bounds is not None for bounds in transitions.values()), "A real UI action changed no actual pixels")
    if case == "normal":
        # Saving must preserve the rendered text/caret area, while the genuine
        # status/control pixels change. Coordinates are the frozen frontend's
        # explicit 100% layout, not a host-rendered replacement UI.
        old, new = decoded["dirty"][2], decoded["saved"][2]
        for row in range(y+144, y+144+ASSIGNED[3]-208):
            start=(row*WIDTH+x+24)*3; end=start+(ASSIGNED[2]-48)*3
            require(old[start:end] == new[start:end], "Save changed rendered document bytes/caret")
    return {"width":WIDTH,"height":HEIGHT,"assigned":list(ASSIGNED),"changed_bounds":transitions,
            "system_chrome_preserved":True,"manual_visual_quality_review_performed":False}


def build_path(path: Path) -> Path:
    path = private_path(Path(path)).resolve()
    require(path.is_relative_to((ROOT / "build").resolve()), "Build outputs must stay under generated ROOT/build")
    return path


def inspected_bound(manifest: dict, artifact: dict, directory: Path, frozen: str) -> dict:
    require(isinstance(manifest, dict) and manifest.get("source_sha256") == frozen
            and type(manifest.get("source_fingerprint_version")) is int and manifest["source_fingerprint_version"] == 3
            and manifest.get("kernel_feature") in FEATURES.values() and manifest.get("loader_feature") == "framebuffer"
            and manifest.get("sdk_runtime") is False and manifest.get("document_session") is False,
            "Wrong/stale paired native Materiaal/framebuffer build")
    require(isinstance(artifact, dict) and set(artifact) == {"passed", "native_runtime_performed", "probe_controller", "apps"}
            and artifact["passed"] is True and artifact["native_runtime_performed"] is False and artifact["probe_controller"] is True
            and isinstance(artifact["apps"], list) and len(artifact["apps"]) == len(NAMES)
            and manifest.get("materiaal") == artifact, "Missing/unbound eight-process build inspection")
    kernel = private_path(directory / "esp/kernel.elf", regular=True).read_bytes()
    expected = {}
    for name, item in zip(NAMES, artifact["apps"]):
        require(isinstance(item, dict) and set(item) == {"name", "sha256", "elf"} and item["name"] == name
                and isinstance(item["sha256"], str) and HEX.fullmatch(item["sha256"]), "Wrong/reordered userspace image")
        image = private_path(directory / f"inputs/{name}.elf", regular=True)
        require(sha256(image) == item["sha256"] and inspect_user(image) == item["elf"] and image.read_bytes() in kernel,
                "Changed/nonembedded/uninspected userspace image")
    for key, name in (("kernel", "esp/kernel.elf"), ("loader", "esp/EFI/BOOT/BOOTX64.EFI")):
        details = manifest.get(key)
        require(isinstance(details, dict) and isinstance(details.get("sha256"), str) and HEX.fullmatch(details["sha256"])
                and sha256(private_path(directory / name, regular=True)) == details["sha256"], "Changed kernel/loader bytes")
        expected[name] = details["sha256"]
    files = set()
    for path in (directory / "esp").rglob("*"):
        private_path(path)
        if path.is_file(): files.add(path.relative_to(directory).as_posix())
    require(files == set(expected), "Unexpected ESP input")
    return {"manifest": manifest, "artifact": artifact, "expected": expected}


def validate_build(build: Path, frozen: str, case: str) -> dict:
    build = build_path(build)
    require(case in FEATURES and isinstance(frozen, str) and HEX.fullmatch(frozen), "Wrong case/source fingerprint")
    manifest = load_json(private_path(build / "manifest.json", regular=True))
    artifact = load_json(private_path(build / "users/materiaal/build-report.json", regular=True))
    require(manifest.get("kernel_feature") == FEATURES[case], "Build the exact requested diagnostic case first")
    # Inspect in-place through a read-only view of the selected build layout.
    # The run layout names the eight immutable copies under inputs/ instead.
    require(isinstance(artifact, dict) and isinstance(artifact.get("apps"), list) and len(artifact["apps"]) == len(NAMES),
            "Missing eight real user executables")
    require(manifest.get("source_sha256") == frozen and manifest.get("loader_feature") == "framebuffer"
            and manifest.get("source_fingerprint_version") == 3 and type(manifest.get("source_fingerprint_version")) is int
            and manifest.get("materiaal") == artifact and artifact.get("passed") is True
            and artifact.get("native_runtime_performed") is False and artifact.get("probe_controller") is True
            and set(artifact) == {"passed", "native_runtime_performed", "probe_controller", "apps"}
            and manifest.get("sdk_runtime") is False and manifest.get("document_session") is False,
            "Wrong or stale build/artifact manifest")
    kernel = private_path(build / "esp/kernel.elf", regular=True).read_bytes()
    for name, item in zip(NAMES, artifact["apps"]):
        path = private_path(build / f"users/materiaal/{name}.elf", regular=True)
        require(isinstance(item, dict) and set(item) == {"name", "sha256", "elf"} and item["name"] == name
                and isinstance(item["sha256"], str) and HEX.fullmatch(item["sha256"])
                and sha256(path) == item["sha256"] and inspect_user(path) == item["elf"] and path.read_bytes() in kernel,
                "Unbound actual user ELF")
    expected = {}
    for key, name in (("kernel", "esp/kernel.elf"), ("loader", "esp/EFI/BOOT/BOOTX64.EFI")):
        details=manifest.get(key)
        require(isinstance(details, dict) and isinstance(details.get("sha256"), str) and HEX.fullmatch(details["sha256"])
                and sha256(private_path(build / name, regular=True)) == details["sha256"], "Changed ESP executable")
        expected[name] = details["sha256"]
    files=set()
    for path in (build / "esp").rglob("*"):
        private_path(path)
        if path.is_file(): files.add(path.relative_to(build).as_posix())
    require(files == set(expected), "Unexpected ESP file")
    return {"manifest":manifest,"artifact":artifact,"expected":expected}


def qemu_arguments(qemu: str, machine: str, transport: str, case: str) -> list[str]:
    require(machine == qemu_machine(machine) and transport in ("unix", "pipe") and case in FEATURES, "Wrong VM profile")
    return [qemu,"-machine",machine,"-accel","tcg,thread=single","-cpu","qemu64","-smp","1","-m","256M",
            "-nodefaults","-display","none","-monitor","none","-serial","file:serial.txt" if transport == "pipe" else "stdio",
            "-nic","none","-no-reboot","-device","VGA,vgamem_mb=16,romfile=VGA.rom",
            "-qmp","stdio" if transport == "pipe" else "unix:qmp.sock,server=on,wait=off",
            "-drive","if=pflash,unit=0,format=raw,readonly=on,file=OVMF_CODE.fd",
            "-drive","if=pflash,unit=1,format=raw,file=OVMF_VARS.fd",
            "-drive","if=virtio,index=0,format=raw,readonly=on,file=fat:esp",
            "-drive",f"if=none,id=volume,format=raw,cache=writeback,readonly={'on' if case == 'recover' else 'off'},file=volume.img",
            "-device","virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-SYSTEM-TEST",
            "-object","rng-random,id=rng0,filename=/dev/urandom",
            "-device","virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000",
            "-device","isa-debug-exit,iobase=0xf4,iosize=0x04"]


def serial_bytes(directory: Path) -> bytes:
    with private_path(directory / "serial.txt",regular=True).open("rb") as stream: raw=stream.read(SERIAL_LIMIT+1)
    require(len(raw) <= SERIAL_LIMIT,"Oversized native transcript")
    return raw


def frame_bytes(directory: Path, name: str) -> bytes:
    path=private_path(directory / f"{name}.ppm",regular=True)
    require(path.stat().st_size <= WIDTH*HEIGHT*3+128,"Oversized actual screenshot")
    raw=path.read_bytes();require(ppm(raw)[:2] == (WIDTH,HEIGHT),"Wrong actual GOP dimensions")
    return raw


def replay(report: dict, directory: Path, frozen: str) -> dict:
    """Recompute the proof from frozen bytes, never trust a stored passed flag."""
    case=report.get("case");require(case in FEATURES,"Unknown recorded case")
    actual_hashes=artifact_hashes(directory)
    require(report.get("artifact_sha256") == actual_hashes,"Captured artifact hash set changed")
    require(report.get("source_sha256") == frozen and report.get("source_fingerprint_version") == 3
            and type(report.get("source_fingerprint_version")) is int and report.get("kernel_feature") == FEATURES[case]
            and report.get("profile") == f'{report.get("machine")}/qemu64/TCG/1CPU/256MiB/VGA/PS2'
            and type(report.get("raw_qemu_exit")) is int and report["raw_qemu_exit"] == 33
            and report.get("timed_out") is False and report.get("cwd") == str(directory)
            and report.get("command") == qemu_arguments(report.get("qemu_entry_path"),report.get("machine"),report.get("qmp_transport"),case),
            "Recorded VM run/profile/exit differs")
    frozen_files=report.get("frozen_artifacts")
    names={"esp/kernel.elf","esp/EFI/BOOT/BOOTX64.EFI","OVMF_CODE.fd","OVMF_VARS.template.fd","VGA.rom","inputs/manifest.json","inputs/build-report.json"}
    names |= {f"inputs/{name}.elf" for name in NAMES}
    require(isinstance(frozen_files,dict) and set(frozen_files) == names,"Wrong frozen input set")
    for name,value in frozen_files.items():
        require(isinstance(value,str) and HEX.fullmatch(value) and sha256(private_path(directory/name,regular=True)) == value,
                "Frozen run input changed")
    manifest=load_json(private_path(directory/"inputs/manifest.json",regular=True))
    artifact=load_json(private_path(directory/"inputs/build-report.json",regular=True))
    require(manifest.get("kernel_feature") == FEATURES[case],"Wrong case kernel")
    inspected_bound(manifest,artifact,directory,frozen)
    raw=serial_bytes(directory)
    require(report.get("stdout") == raw.decode("utf-8","strict"),"Reported serial differs from actual bytes")
    parsed=transcript(raw,case,final=True);selected=phases(parsed,case,complete=True)
    validate_qmp(load_json(private_path(directory/"qmp-trace.json",regular=True)),directory,case)
    stages=report.get("stages")
    require(isinstance(stages,list) and [s.get("phase") for s in stages] == list(PHASES[case]),"Missing/extra captured stages")
    requests=qmp_requests(directory,case);previous=0
    for stage,name in zip(stages,PHASES[case]):
        require(set(stage) == {"phase","snapshot","serial_size","serial_sha256","qmp_index"}
                and type(stage["serial_size"]) is int and previous <= stage["serial_size"] <= len(raw)
                and hashlib.sha256(raw[:stage["serial_size"]]).hexdigest() == stage["serial_sha256"],"Changed stage serial prefix")
        prefix=transcript(raw[:stage["serial_size"]],case)
        observed=phases(prefix,case).get(name)
        require(json.dumps(observed,sort_keys=True) == json.dumps(stage["snapshot"],sort_keys=True)
                and observed == selected[name],"Stage is not bound to a real complete serial snapshot")
        require(type(stage["qmp_index"]) is int and 1 <= stage["qmp_index"] <= len(requests)
                and requests[stage["qmp_index"]-1] == {"execute":"screendump","arguments":{"filename":str(directory/f"{name}.ppm")},"id":stage["qmp_index"]},
                "Captured stage is not bound to its actual screendump")
        previous=stage["serial_size"]
    pixels=validate_frames({name:frame_bytes(directory,name) for name in PHASES[case]},case)
    before=private_path(directory/"volume.before.img",regular=True);after=private_path(directory/"volume.after.img",regular=True)
    check_image(before);check_image(after)
    require(sha256(before) == report.get("before_sha256") and sha256(after) == report.get("after_sha256"),"Disk snapshot hash mismatch")
    oracle=disk_oracle(after.read_bytes())
    if case == "recover":
        require(before.read_bytes() == after.read_bytes(),"Cold readonly run changed disk bytes")
    else:
        data=before.read_bytes()
        require(all(data[i*volume.SECTOR:(i+1)*volume.SECTOR] == (bytes(volume.SECTOR) if volume.FIRST <= i < volume.FIRST+volume.VOLUME_SECTORS else volume.canary(i)) for i in range(volume.IMAGE_SECTORS)),"Normal run was not the canonical empty generated image")
    return {"native":parsed,"phases":selected,"frames":pixels,"disk":oracle}


def artifact_hashes(directory: Path) -> dict:
    return {p.relative_to(directory).as_posix():sha256(private_path(p,regular=True))
            for p in directory.rglob("*") if p.is_file() and p.name != "result.json"}


def parent_run(path: Path, build: Path, frozen: str) -> tuple[Path, dict]:
    directory=private_path(Path(path)).resolve(strict=True)
    require(directory.parent == (build/"run").resolve() and directory.name.startswith("materiaal-"),"Cold parent must be an owned generated run")
    report=load_json(private_path(directory/"result.json",regular=True))
    require(isinstance(report,dict) and report.get("passed") is True and report.get("case") == "normal","Cold parent must be a complete passing normal case")
    replay(report,directory,frozen)
    return private_path(directory/"volume.after.img",regular=True),report


def run_materiaal_probe(*,case="normal",build_directory=BUILD,run_root=None,parent=None,machine=None,
                       qmp_transport="pipe",vga_rom=None,vga_rom_sha256=None,timeout=240) -> dict:
    require(sys.platform == "linux" and case in FEATURES,"Linux/private-QMP diagnostic case required")
    require(type(timeout) in (int,float) and 1 <= timeout <= 240,"Unbounded VM timeout")
    require(qmp_transport in ("unix","pipe"),"Unknown QMP transport")
    machine=qemu_machine(machine);build=build_path(Path(build_directory));build.mkdir(parents=True,exist_ok=True)
    run_root=private_path(build/"run" if run_root is None else Path(run_root)).resolve()
    require(run_root == build/"run","Runs must use the dedicated BUILD/run directory")
    run_root.mkdir(parents=True,exist_ok=True)
    report_path=private_path(build/"materiaal-probe-report.json")
    publish_json(report_path,{"passed":False,"stage":"preflight","case":case})
    frozen=source_hash();bound=validate_build(build,frozen,case)
    require((parent is None) == (case == "normal"),"Recover requires one validated normal parent; normal accepts no parent/disk")
    parent_image,parent_report=parent_run(parent,build,frozen) if case == "recover" else (None,None)
    env=environment();qemu=shutil.which("qemu-system-x86_64",path=env["PATH"]);require(qemu is not None,"Missing existing QEMU")
    entry=external_file(Path(qemu),256*1024*1024);entry_hash=sha256(entry)
    require(vga_rom is not None and (vga_rom_sha256 is None or isinstance(vga_rom_sha256,str) and HEX.fullmatch(vga_rom_sha256)),"Explicit existing VGA ROM required")
    code,variables=firmware()
    directory=private_path(run_root/("materiaal-"+case+"-"+uuid.uuid4().hex[:10]));directory.mkdir()
    require(qmp_transport != "unix" or len(os.fsencode(directory/"qmp.sock")) < 104,"Long QMP path; use pipe transport")
    (directory/"inputs").mkdir();inputs={}
    for source,name in ((code,"OVMF_CODE.fd"),(variables,"OVMF_VARS.template.fd"),(vga_rom,"VGA.rom")):
        source=external_file(Path(source),1024*1024 if name == "VGA.rom" else 16*1024*1024)
        publish(directory/name,source.read_bytes());inputs[name]=sha256(directory/name)
    require(vga_rom_sha256 is None or inputs["VGA.rom"] == vga_rom_sha256,"Wrong explicitly pinned VGA ROM")
    publish(directory/"OVMF_VARS.fd",(directory/"OVMF_VARS.template.fd").read_bytes())
    for name,expected in bound["expected"].items():
        path=directory/name;path.parent.mkdir(parents=True,exist_ok=True);publish(path,private_path(build/name,regular=True).read_bytes())
        require(sha256(path) == expected,"ESP changed during snapshot");inputs[name]=expected;path.chmod(0o444)
    for name,item in zip(NAMES,bound["artifact"]["apps"]):
        publish(directory/f"inputs/{name}.elf",private_path(build/f"users/materiaal/{name}.elf",regular=True).read_bytes())
        require(sha256(directory/f"inputs/{name}.elf") == item["sha256"],"ELF changed during copy");inputs[f"inputs/{name}.elf"]=item["sha256"]
    for name,value in (("manifest.json",bound["manifest"]),("build-report.json",bound["artifact"])):
        publish_json(directory/"inputs"/name,value);inputs["inputs/"+name]=sha256(directory/"inputs"/name)
    if case == "normal": volume.create_image(directory/"volume.img","empty")
    else: publish(directory/"volume.img",parent_image.read_bytes())
    check_image(directory/"volume.img");publish(directory/"volume.before.img",(directory/"volume.img").read_bytes())
    before=sha256(directory/"volume.before.img")
    require(case == "normal" or before == parent_report["after_sha256"],"Broken cold disk chain")
    command=qemu_arguments(qemu,machine,qmp_transport,case)
    report={"schema_version":1,"passed":False,"scope":"partial native GUI/input/document chain on generated plaintext fixture; not full Materiaal, encrypted durable storage, release or physical hardware acceptance",
            "case":case,"source_sha256":frozen,"source_fingerprint_version":3,"kernel_feature":FEATURES[case],
            "source_commit":subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip(),
            "run_directory":str(directory),"build_directory":str(build),"cwd":str(directory),"command":command,
            "qemu_entry_path":qemu,"qemu_entry_sha256":entry_hash,"machine":machine,"qmp_transport":qmp_transport,
            "profile":f"{machine}/qemu64/TCG/1CPU/256MiB/VGA/PS2",
            "qemu_version":subprocess.check_output([qemu,"--version"],env=env,text=True,timeout=10).strip(),
            "frozen_artifacts":inputs,"before_sha256":before,"raw_qemu_exit":None,"timed_out":False,"timeout_s":timeout,
            "parent_run":str(parent) if parent else None,"stages":[],"checks":{},"visual_quality_acceptance":False}
    publish_json(report_path,report);publish_json(directory/"result.json",report)
    process=control=None;trace=[];start=time.monotonic();deadline=start+timeout
    try:
        require(source_hash() == frozen,"Source changed while preparing VM")
        with (directory/"serial.txt").open("xb") as stdout,(directory/"stderr.txt").open("xb") as stderr:
            process=subprocess.Popen(command,cwd=directory,env=env,stdout=subprocess.PIPE if qmp_transport == "pipe" else stdout,
                                     stdin=subprocess.PIPE if qmp_transport == "pipe" else subprocess.DEVNULL,stderr=stderr,bufsize=0)
            try:
                control=QmpPipe(process,deadline) if qmp_transport == "pipe" else Qmp(directory/"qmp.sock",deadline,process)
                implementation=external_file(Path(os.readlink(f"/proc/{process.pid}/exe")),256*1024*1024)
                report["qemu_implementation_path"]=str(implementation);report["qemu_implementation_sha256"]=sha256(implementation)
                for name,group in qmp_groups(directory,case):
                    for request in group:
                        if request["execute"] == "screendump":
                            while True:
                                raw=serial_bytes(directory);parsed=transcript(raw,case)
                                selected=phases(parsed,case)
                                if name in selected: break
                                require(time.monotonic() < deadline and process.poll() is None,"Guest did not complete actual UI phase "+name)
                                time.sleep(0.01)
                            control.command(request["execute"],request["arguments"])
                            frame_bytes(directory,name)
                            report["stages"].append({"phase":name,"snapshot":selected[name],"serial_size":len(raw),
                                                     "serial_sha256":hashlib.sha256(raw).hexdigest(),"qmp_index":len(control.trace)})
                        else:
                            # The first input follows an observed ready frame;
                            # F12 is last and follows the real reopened capture.
                            require(report["stages"],"No native ready/focus/frame proof before QMP input")
                            control.command(request["execute"],request["arguments"])
                            remaining=deadline-time.monotonic();require(remaining>0,"Shared VM deadline exceeded")
                            time.sleep(min(0.08,remaining))
                report["raw_qemu_exit"]=process.wait(timeout=max(0.01,deadline-time.monotonic()))
                require(time.monotonic()<deadline,"Native run exceeded its deadline")
            finally:
                if control is not None:trace=control.trace;control.close()
                if process.poll() is None:process.kill();process.wait(timeout=10)
                if report["raw_qemu_exit"] is None:report["raw_qemu_exit"]=process.returncode
        publish_json(directory/"qmp-trace.json",trace)
        publish(directory/"volume.after.img",private_path(directory/"volume.img",regular=True).read_bytes())
        report["after_sha256"]=sha256(directory/"volume.after.img");report["stdout"]=serial_bytes(directory).decode("utf-8","strict")
        report["artifact_sha256"]=artifact_hashes(directory)
        report["observed"]=replay(report,directory,frozen)
        report["checks"]={"native_transcript_qmp_pixels_disk_recomputed":True,"source_unchanged":source_hash() == frozen,
                          "qemu_entry_unchanged":sha256(entry) == entry_hash,"qemu_implementation_unchanged":sha256(implementation) == report["qemu_implementation_sha256"]}
        validate_build(build,frozen,case);report["checks"]["current_build_unchanged"]=True
        report["passed"]=all(report["checks"].values())
    except Exception as error:
        report["error"]=f"{type(error).__name__}: {error}";report["timed_out"]=isinstance(error,(TimeoutError,subprocess.TimeoutExpired)) or time.monotonic()>=deadline
    finally:
        if process is not None and process.poll() is None:process.kill();process.wait(timeout=10)
        publish_json(directory/"qmp-trace.json",trace)
        if (directory/"serial.txt").is_file():
            try:report["stdout"]=serial_bytes(directory).decode("utf-8","replace")
            except (OSError,ValueError) as error:
                report["passed"]=False
                report["serial_error"]=str(error)
        image=directory/"volume.img"
        if image.is_file():check_image(image);publish(directory/"volume.after.img",image.read_bytes());report["after_sha256"]=sha256(directory/"volume.after.img")
        socket=directory/"qmp.sock"
        if socket.exists() and stat.S_ISSOCK(socket.lstat().st_mode):socket.unlink()
        report["duration_s"]=time.monotonic()-start
        report["artifact_sha256"]=artifact_hashes(directory)
        publish_json(directory/"result.json",report);publish_json(report_path,report)
    return report


if __name__ == "__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case",choices=tuple(FEATURES),default="normal")
    parser.add_argument("--build-dir",type=Path,default=BUILD)
    parser.add_argument("--run-root",type=Path)
    parser.add_argument("--parent",type=Path)
    parser.add_argument("--machine",choices=("pc-q35-10.0","pc-q35-8.2","pc-q35-7.2"))
    parser.add_argument("--qmp-transport",choices=("unix","pipe"),default="pipe")
    parser.add_argument("--vga-rom",type=Path,required=True)
    parser.add_argument("--vga-rom-sha256")
    parser.add_argument("--timeout",type=int,default=240)
    args=parser.parse_args()
    result=run_materiaal_probe(case=args.case,build_directory=args.build_dir,run_root=args.run_root,parent=args.parent,
                              machine=args.machine,qmp_transport=args.qmp_transport,vga_rom=args.vga_rom,
                              vga_rom_sha256=args.vga_rom_sha256,timeout=args.timeout)
    print(f'Materiaal {args.case}: passed={result["passed"]}; raw={result["raw_qemu_exit"]}; {result["run_directory"]}')
    if "error" in result:print(result["error"])
    raise SystemExit(0 if result["passed"] else 1)
