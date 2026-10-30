#!/usr/bin/env python3
"""B03 QEMU runner: one signed-update stage per invocation on a slot store and
an OVMF variable store that persist across the stages of a sequence (install,
reboot into the new slot, downgrade attempt, interrupted install, ...). The
loader prints which slot it verified or rejected, the kernel prints its own
image version and the update service's evidence words; this host side checks
the marker sequence, the raw exit, the numbers and the store bytes with the
independent oracle in update_disk.py."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import queue
import re
import shutil
import subprocess
import threading
import time
import uuid
from common import ROOT, BUILD, environment, firmware, qemu_machine, save_json, sha256, source_hash
import update_disk as disk

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")
FAILURES = ("PANIC", "FAIL", "M1:FAIL")
MAX_TIMEOUT = 900
IMAGES = BUILD / "update" / "images.json"
CUT_MARKER = "UPDATE:CUT:IMAGE"

# Each stage: which store/vars sequence it belongs to, how a fresh store is
# packed (None = continue the sequence), host tampering before boot, the
# expected loader/kernel/service lines and the numbers that must match.
#   pack: (active, {"slot-a": (manifest, kernel), "slot-b": ..., "inbox": ...})
#   mutate: list of (kind, args) applied by update_disk after packing/loading
# An empty (all-zero) slot is reported by the loader as REJECTED:FORMAT before
# the chosen slot's VERSION line; a valid but not chosen slot prints nothing.
STAGES: dict[str, dict] = {
    "install": dict(sequence="main", fresh=True, pack=("A", {"slot-a": ("m1", "k1"), "inbox": ("m2", "k2")}),
                    mutate=[], loader=["BOOT:FLOOR:0", "BOOT:SLOT:B:REJECTED:FORMAT", "BOOT:SLOT:A:VERSION:1"], outcome="INSTALLED",
                    expect={"IMAGE_VERSION": 1, "RESULT": 1, "INSTALLED": 2, "INSTALLED_VERSION": 2,
                            "TARGET_SLOT": 1, "RUNNING_SLOT": 0, "HEADER_ACTIVE": 0, "HEADER_GENERATION": 1,
                            "NEW_GENERATION": 2, "ACTIVE_VERSION": 1, "INBOX_VERSION": 2, "RESUMED": 0,
                            "BROKER:HEADER_WRITES": 1},
                    at_least={"BROKER:FLUSHES": 3, "COPIED_SECTORS": 64}),
    "boot-v2": dict(sequence="main", fresh=False, parent="install", mutate=[],
                    loader=["BOOT:FLOOR:1", "BOOT:SLOT:B:VERSION:2"], outcome="UP_TO_DATE",
                    expect={"IMAGE_VERSION": 2, "RESULT": 2, "HEADER_ACTIVE": 1, "HEADER_GENERATION": 2,
                            "ACTIVE_VERSION": 2, "OTHER_VERSION": 1, "RUNNING_SLOT": 1, "BROKER:WRITES": 0,
                            "BROKER:FLUSHES": 0},
                    at_least={}),
    "downgrade": dict(sequence="main", fresh=False, parent="boot-v2", mutate=[("set_active", 0)],
                      loader=["BOOT:FLOOR:2", "BOOT:SLOT:A:REJECTED:DOWNGRADE", "BOOT:SLOT:B:VERSION:2"],
                      outcome="UP_TO_DATE",
                      expect={"IMAGE_VERSION": 2, "RESULT": 2, "HEADER_ACTIVE": 0, "ACTIVE_VERSION": 1,
                              "OTHER_VERSION": 2, "RUNNING_SLOT": 1, "BROKER:WRITES": 0},
                      at_least={}),
    "bad-signature": dict(sequence="signature", fresh=True,
                          pack=("B", {"slot-a": ("m1", "k1"), "slot-b": ("m2", "k2"), "inbox": ("m2", "k2")}),
                          mutate=[("flip_manifest", "B", disk.MLDSA_SIGNATURE_OFFSET + 100)],
                          loader=["BOOT:FLOOR:0", "BOOT:SLOT:B:REJECTED:SIGNATURE", "BOOT:SLOT:A:VERSION:1"],
                          outcome="INSTALLED",
                          expect={"IMAGE_VERSION": 1, "RESULT": 1, "INSTALLED": 2, "HEADER_ACTIVE": 1,
                                  "ACTIVE_REJECT": 3, "OTHER_VERSION": 1, "RUNNING_SLOT": 0, "TARGET_SLOT": 1,
                                  "RESUMED": 0, "BROKER:HEADER_WRITES": 1},
                          at_least={"BROKER:FLUSHES": 3}),
    "bad-digest": dict(sequence="digest", fresh=True,
                       pack=("B", {"slot-a": ("m1", "k1"), "slot-b": ("m2", "k2"), "inbox": ("m2", "k2")}),
                       mutate=[("flip_image", "B", 1000)],
                       loader=["BOOT:FLOOR:0", "BOOT:SLOT:B:REJECTED:DIGEST", "BOOT:SLOT:A:VERSION:1"],
                       outcome="INSTALLED",
                       expect={"IMAGE_VERSION": 1, "RESULT": 1, "INSTALLED": 2, "HEADER_ACTIVE": 1,
                               "ACTIVE_VERSION": 2, "OTHER_VERSION": 1, "RUNNING_SLOT": 0, "TARGET_SLOT": 1,
                               "BROKER:HEADER_WRITES": 1},
                       at_least={"BROKER:FLUSHES": 3}),
    "cut": dict(sequence="cut", fresh=True, pack=("A", {"slot-a": ("mc", "kc"), "inbox": ("m2", "k2")}),
                mutate=[], loader=["BOOT:FLOOR:0", "BOOT:SLOT:B:REJECTED:FORMAT", "BOOT:SLOT:A:VERSION:1"], outcome="CUT",
                expect={"IMAGE_VERSION": 1}, at_least={}),
    "cut-resume": dict(sequence="cut", fresh=False, parent="cut", mutate=[],
                       loader=["BOOT:FLOOR:1", "BOOT:SLOT:B:REJECTED:FORMAT", "BOOT:SLOT:A:VERSION:1"], outcome="INSTALLED",
                       expect={"IMAGE_VERSION": 1, "RESULT": 1, "INSTALLED": 2, "RESUMED": 1,
                               "HEADER_ACTIVE": 0, "HEADER_GENERATION": 1, "NEW_GENERATION": 2,
                               "TARGET_SLOT": 1, "RUNNING_SLOT": 0, "BROKER:HEADER_WRITES": 1},
                       at_least={"BROKER:FLUSHES": 3}),
    "cut-final": dict(sequence="cut", fresh=False, parent="cut-resume", mutate=[],
                      loader=["BOOT:FLOOR:1", "BOOT:SLOT:B:VERSION:2"], outcome="UP_TO_DATE",
                      expect={"IMAGE_VERSION": 2, "RESULT": 2, "HEADER_ACTIVE": 1, "HEADER_GENERATION": 2,
                              "RUNNING_SLOT": 1, "BROKER:WRITES": 0},
                      at_least={}),
    "no-valid-slot": dict(sequence="none", fresh=True,
                          pack=("A", {"slot-a": ("m1", "k1"), "slot-b": ("m2", "k2")}),
                          mutate=[("flip_manifest", "A", disk.ED25519_SIGNATURE_OFFSET + 5),
                                  ("flip_manifest", "B", disk.MLDSA_SIGNATURE_OFFSET + 7)],
                          loader=["BOOT:FLOOR:0", "BOOT:SLOT:A:REJECTED:SIGNATURE",
                                  "BOOT:SLOT:B:REJECTED:SIGNATURE"],
                          outcome="NO_VALID_SLOT", expect={}, at_least={}),
    "inbox-signature": dict(sequence="inbox-signature", fresh=True,
                            pack=("A", {"slot-a": ("m1", "k1"), "inbox": ("m2", "k2")}),
                            mutate=[("flip_manifest", "inbox", disk.MLDSA_SIGNATURE_OFFSET + 100)],
                            loader=["BOOT:FLOOR:0", "BOOT:SLOT:B:REJECTED:FORMAT", "BOOT:SLOT:A:VERSION:1"], outcome="REJECTED:SIGNATURE",
                            expect={"IMAGE_VERSION": 1, "RESULT": 3, "INBOX_REJECT": 3, "INBOX_VERSION": 0,
                                    "BROKER:WRITES": 0, "BROKER:FLUSHES": 0},
                            at_least={}),
    "inbox-downgrade": dict(sequence="inbox-downgrade", fresh=True,
                            pack=("A", {"slot-a": ("m2", "k2"), "inbox": ("m1", "k1")}),
                            mutate=[], loader=["BOOT:FLOOR:0", "BOOT:SLOT:B:REJECTED:FORMAT", "BOOT:SLOT:A:VERSION:2"],
                            outcome="REJECTED:DOWNGRADE",
                            expect={"IMAGE_VERSION": 2, "RESULT": 3, "INBOX_REJECT": 5, "INBOX_VERSION": 1,
                                    "BROKER:WRITES": 0, "BROKER:FLUSHES": 0},
                            at_least={}),
    "inbox-key": dict(sequence="inbox-key", fresh=True,
                      pack=("A", {"slot-a": ("m1", "k1"), "inbox": ("mf", "k2")}),
                      mutate=[], loader=["BOOT:FLOOR:0", "BOOT:SLOT:B:REJECTED:FORMAT", "BOOT:SLOT:A:VERSION:1"],
                      outcome="REJECTED:UNKNOWN_KEY",
                      expect={"IMAGE_VERSION": 1, "RESULT": 3, "INBOX_REJECT": 2, "INBOX_VERSION": 0,
                              "BROKER:WRITES": 0, "BROKER:FLUSHES": 0},
                      at_least={}),
}
ORDER = ("install", "boot-v2", "downgrade", "bad-signature", "bad-digest", "cut", "cut-resume",
         "cut-final", "no-valid-slot", "inbox-signature", "inbox-downgrade", "inbox-key")
CONTROLLED_PREFIXES = ("BOOT:", "UPDATE:MANIFEST:PASS", "UPDATE:READY", "UPDATE:UP_TO_DATE",
                       "UPDATE:REJECTED:", "UPDATE:NO_BUNDLE", "UPDATE:CUT:", "UPDATE:RESOURCES:PASS",
                       "UPDATE:PASS", "M1:PASS", "KERNEL:ENTRY", "BOOTINFO:OK")
NUMBER = re.compile(r"(UPDATE:[A-Z_:]+|KERNEL:IMAGE_VERSION):([0-9]{20})$")


def numbers(stdout: str) -> dict[str, int]:
    found = {}
    for line in stdout.splitlines():
        match = NUMBER.fullmatch(line)
        if match:
            key = match.group(1)
            key = "IMAGE_VERSION" if key == "KERNEL:IMAGE_VERSION" else key[len("UPDATE:"):]
            if key in found:
                raise ValueError(f"duplicate evidence {key}")
            found[key] = int(match.group(2))
    return found


def controlled(stdout: str) -> list[str]:
    kept = []
    for line in stdout.splitlines():
        if NUMBER.fullmatch(line):
            continue
        if line.startswith(CONTROLLED_PREFIXES):
            kept.append(line)
    return kept


def expected_markers(stage: str) -> tuple[str, ...]:
    plan = STAGES[stage]
    lines = list(plan["loader"])
    if plan["outcome"] == "NO_VALID_SLOT":
        return tuple(lines + ["BOOT:FAIL:NO_VALID_SLOT"])
    lines += ["BOOT:SIGNED:OK", *BOOT, "UPDATE:MANIFEST:PASS"]
    if plan["outcome"] == "CUT":
        return tuple(lines + [CUT_MARKER])
    lines.append("UPDATE:READY")
    if plan["outcome"] != "INSTALLED":
        lines.append(f"UPDATE:{plan['outcome']}")
    return tuple(lines + ["UPDATE:RESOURCES:PASS", "UPDATE:PASS", "M1:PASS"])


def check_numbers(stage: str, found: dict[str, int]) -> None:
    plan = STAGES[stage]
    for key, value in plan["expect"].items():
        if found.get(key) != value:
            raise ValueError(f"{key}: expected {value}, found {found.get(key)}")
    for key, minimum in plan["at_least"].items():
        if found.get(key, -1) < minimum:
            raise ValueError(f"{key}: expected at least {minimum}, found {found.get(key)}")
    if plan["outcome"] not in ("CUT", "NO_VALID_SLOT"):
        if "TICKS" not in found or found["TICKS"] >= 60_000:
            raise ValueError("missing or exhausted tick budget")
        if plan["outcome"] == "INSTALLED" and found.get("INSTALLED") != plan["expect"]["INSTALLED"]:
            raise ValueError("INSTALLED: installed version differs")


def evaluate(stage: str, stdout: str, rc, timed_out: bool, killed: bool, running: bool) -> dict[str, int]:
    """Transcript verdict; the store oracle is checked separately."""
    plan = STAGES[stage]
    if controlled(stdout) != list(expected_markers(stage)):
        raise ValueError(f"marker sequence mismatch: {controlled(stdout)}")
    allowed_failure = "BOOT:FAIL:NO_VALID_SLOT" if plan["outcome"] == "NO_VALID_SLOT" else None
    for line in stdout.splitlines():
        if line == allowed_failure:
            continue
        if any(marker in line for marker in FAILURES):
            raise ValueError("failure marker present")
    if plan["outcome"] == "CUT":
        if not (killed and running and not timed_out):
            raise ValueError("cut was not followed by a real host kill of a live guest")
    elif plan["outcome"] == "NO_VALID_SLOT":
        if rc != 35 or timed_out:
            raise ValueError(f"raw exit {rc}, timeout {timed_out} (expected 35)")
        if "KERNEL:ENTRY" in stdout:
            raise ValueError("kernel entered after a failed selection")
    elif rc != 33 or timed_out:
        raise ValueError(f"raw exit {rc}, timeout {timed_out}")
    found = numbers(stdout)
    check_numbers(stage, found)
    return found


def check_store(stage: str, before: bytes, after: bytes, images: dict) -> dict:
    """Independent byte-level verdict on the slot store after the stage."""
    plan = STAGES[stage]
    oracle = disk.analyse(after)
    header = oracle["header"]
    prior = disk.analyse(before)["header"]
    k2 = (BUILD / "update" / "kernel-v2.elf").read_bytes()
    m2 = (BUILD / "update" / "manifest-v2.bin").read_bytes()
    if disk.sha256(k2) != images["kernels"]["k2"] or disk.sha256(m2) != images["manifests"]["m2"]:
        raise ValueError("reference images changed")
    if plan["outcome"] in ("UP_TO_DATE", "NO_VALID_SLOT") or plan["outcome"].startswith("REJECTED"):
        if after != before:
            raise ValueError("store changed without an install")
    elif plan["outcome"] == "INSTALLED":
        if header != {"active": 1, "generation": prior["generation"] + 1}:
            raise ValueError(f"header after install: {header}")
        if not disk.slot_identical(before, after, "A") or not disk.slot_identical(before, after, "inbox"):
            raise ValueError("install touched slot A or the inbox")
        if not disk.manifest_region_is(after, "B", m2) or not disk.image_region_is(after, "B", k2):
            raise ValueError("slot B does not hold exactly the v2 bundle")
        if not oracle["regions"]["B"]["image_matches_manifest"]:
            raise ValueError("oracle digest of slot B differs from its manifest")
    elif plan["outcome"] == "CUT":
        if header != prior or not disk.slot_identical(before, after, "A"):
            raise ValueError("cut changed the header or the active slot")
        if not disk.slot_identical(before, after, "inbox"):
            raise ValueError("cut changed the inbox")
        b = oracle["regions"]["B"]
        if b["manifest"] is not None or b["manifest_error"] is not None:
            raise ValueError("cut wrote a slot B manifest")
        written = b["image_nonzero_sectors"]
        total = disk.nonzero_sectors(k2)
        if not 64 <= written < total:
            raise ValueError(f"cut left {written} of {total} image sectors")
        prefix = disk.region(after, disk.SLOT_B_IMAGE, 64 * disk.SECTOR)
        if prefix != k2[:64 * disk.SECTOR]:
            raise ValueError("partial slot B image is not a prefix of the v2 image")
    return oracle


def execute(args, env, *, cut_marker=None, timeout=MAX_TIMEOUT):
    process = subprocess.Popen(args, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    out, err, lines = [], [], queue.Queue()

    def reader(pipe, sink, notify):
        for raw in iter(pipe.readline, b""):
            sink.append(raw)
            if notify:
                lines.put(raw.decode("utf-8", "replace").rstrip("\r\n"))
        pipe.close()

    threads = [threading.Thread(target=reader, args=(process.stdout, out, True), daemon=True),
               threading.Thread(target=reader, args=(process.stderr, err, False), daemon=True)]
    for thread in threads:
        thread.start()
    deadline = time.monotonic() + timeout
    timed_out = killed = running_before_kill = False
    while True:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            timed_out = True
            process.kill()
            break
        try:
            line = lines.get(timeout=min(remaining, 0.5))
        except queue.Empty:
            if process.poll() is not None and lines.empty():
                break
            continue
        if cut_marker is not None and line == cut_marker:
            running_before_kill = process.poll() is None
            if running_before_kill:
                process.kill()
                killed = True
            break
    process.wait()
    for thread in threads:
        thread.join(timeout=10)
    return (process.returncode, b"".join(out).decode("utf-8", "replace"),
            b"".join(err).decode("utf-8", "replace"), timed_out, killed, running_before_kill)


def update_tool(env, *args) -> None:
    subprocess.run(["cargo", "+1.90.0", "run", "--locked", "--release", "-q", "-p", "update-tool", "--", *args],
                   cwd=ROOT, env=env, check=True, capture_output=True, text=True)


def load_images() -> dict:
    images = json.loads(IMAGES.read_text())
    if images.get("source_sha256") != source_hash():
        raise ValueError("images.json was produced from different sources; run tools/test_update.py")
    for group in ("kernels", "manifests"):
        for name, digest in images[group].items():
            path = BUILD / "update" / images["files"][name]
            if sha256(path) != digest:
                raise ValueError(f"{name} changed since images.json was written")
    return images


def pack_store(stage: str, images: dict, env, path: Path) -> None:
    active, regions = STAGES[stage]["pack"]
    args = ["pack", "--out", str(path), "--active", active, "--generation", "1"]
    for region, (manifest, kernel) in regions.items():
        args += [f"--{region}-manifest", str(BUILD / "update" / images["files"][manifest]),
                 f"--{region}-image", str(BUILD / "update" / images["files"][kernel])]
    update_tool(env, *args)


def mutate(store: bytearray, steps) -> None:
    for step in steps:
        kind, *args = step
        if kind == "set_active":
            disk.set_active(store, args[0])
        elif kind == "flip_manifest":
            disk.flip(store, disk.manifest_offset(args[0], args[1]))
        elif kind == "flip_image":
            disk.flip(store, disk.image_offset(args[0], args[1]))
        else:
            raise ValueError(f"unknown mutation {kind}")


def run_update(stage: str, state_dir: Path | None = None, *, timeout=MAX_TIMEOUT):
    if stage not in STAGES or not 0 < timeout <= MAX_TIMEOUT:
        raise ValueError("unknown B03 stage or invalid timeout")
    plan = STAGES[stage]
    frozen = source_hash()
    images = load_images()
    manifest = json.loads((BUILD / "manifest.json").read_text())
    if manifest.get("loader_feature") != "signed-boot" or manifest.get("source_sha256") != frozen:
        raise ValueError("build an update kernel with the signed-boot loader from the current source first")
    if manifest["loader"]["sha256"] != images["loader_sha256"]:
        raise ValueError("the ESP loader differs from the one recorded with the images")
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("qemu-system-x86_64 not found")
    code, variables = firmware()
    run_dir = BUILD / "run" / f"update-{stage}-{uuid.uuid4().hex[:10]}"
    run_dir.mkdir(parents=True)
    shutil.copytree(BUILD / "esp", run_dir / "esp")
    shutil.copy2(code, run_dir / "OVMF_CODE.fd")
    parent = None
    if plan["fresh"]:
        if state_dir is not None:
            raise ValueError("a fresh stage always starts its own store and variable store")
        state_dir = BUILD / "run" / f"update-state-{plan['sequence']}-{uuid.uuid4().hex[:10]}"
        state_dir.mkdir(parents=True)
        pack_store(stage, images, env, state_dir / "store.img")
        shutil.copy2(variables, state_dir / "OVMF_VARS.fd")
    else:
        if state_dir is None:
            raise ValueError(f"{stage} continues a sequence; pass --state")
        state_dir = state_dir.resolve(strict=True)
        state_dir.relative_to((BUILD / "run").resolve())
        last = json.loads((state_dir / "last-stage.json").read_text())
        parent = last.get("stage")
        if (last.get("passed") is not True or parent != plan["parent"]
                or last.get("store_sha256") != sha256(state_dir / "store.img")
                or last.get("source_sha256") != frozen):
            raise ValueError(f"{stage} requires the exact state left by a passing {plan['parent']} stage")
    store_path = state_dir / "store.img"
    if store_path.stat().st_nlink != 1:
        raise ValueError("store image must have exactly one link")
    if plan["mutate"]:
        store = bytearray(store_path.read_bytes())
        mutate(store, plan["mutate"])
        store_path.write_bytes(store)
    before = store_path.read_bytes()
    rel = run_dir.relative_to(ROOT).as_posix()
    state_rel = state_dir.relative_to(ROOT).as_posix()
    args = [qemu, "-machine", qemu_machine(), "-accel", "tcg,thread=single", "-cpu", "qemu64",
            "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
            "-serial", "stdio", "-nic", "none", "-no-reboot",
            "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
            "-drive", f"if=pflash,unit=1,format=raw,file={state_rel}/OVMF_VARS.fd",
            "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
            "-drive", f"if=none,id=store,format=raw,cache=writeback,file={state_rel}/store.img",
            "-device", "virtio-blk-pci,drive=store,id=storeblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-B03-STORE"]
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    rc, out, err, timed_out, killed, running = execute(
        args, env, cut_marker=CUT_MARKER if plan["outcome"] == "CUT" else None, timeout=timeout)
    duration = time.monotonic() - start
    (run_dir / "serial.txt").write_text(out, encoding="utf-8")
    (run_dir / "stderr.txt").write_text(err, encoding="utf-8")
    after = store_path.read_bytes()
    report = dict(schema_version=1, stage=stage, sequence=plan["sequence"], parent_stage=parent,
                  started_utc=started, duration_s=duration, timeout_s=timeout, timed_out=timed_out,
                  raw_qemu_exit=rc, host_kill=killed, running_before_kill=running, command=args,
                  qemu_version=subprocess.check_output([qemu, "--version"], text=True).strip(),
                  source_fingerprint_version=3, source_sha256=frozen, state_directory=str(state_dir),
                  run_directory=str(run_dir), loader_sha256=manifest["loader"]["sha256"],
                  images=images, store_before_sha256=disk.sha256(before), store_after_sha256=disk.sha256(after),
                  vars_sha256=sha256(state_dir / "OVMF_VARS.fd"), stdout=out, stderr=err, passed=False)
    try:
        report["evidence"] = evaluate(stage, out, rc, timed_out, killed, running)
        report["oracle"] = check_store(stage, before, after, images)
        report["passed"] = source_hash() == frozen
        if not report["passed"]:
            raise ValueError("source changed during run")
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    save_json(run_dir / "result.json", report)
    save_json(state_dir / "last-stage.json", dict(stage=stage, passed=report["passed"],
              run_directory=str(run_dir), store_sha256=disk.sha256(after), source_sha256=frozen))
    print(f"B03 {stage}: passed={report['passed']} raw={rc} duration={duration:.1f}s {run_dir}")
    if "error" in report:
        print(report["error"])
    return report, state_dir


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stage", choices=ORDER, required=True)
    parser.add_argument("--state", type=Path, help="state directory of the previous stage of the sequence")
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    args = parser.parse_args()
    raise SystemExit(0 if run_update(args.stage, args.state, timeout=args.timeout)[0]["passed"] else 1)
