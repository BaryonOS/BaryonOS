#!/usr/bin/env python3
"""I06: read retained bytes/logs independently; never start a VM or write a disk."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import random
import re
import struct
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from common import source_hash

HEAD = "3e41bbf93822f7044c6020259f4a752925e1966d"
SOURCE = "307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60"
ARCHIVE = ROOT / "docs/verification/parallel-final-m6b"
RUNS = ROOT / "build/parallel-final-m6b/run"
FAULTS = ("write-error", "flush-error", "write-timeout", "flush-timeout",
          "reset-timeout", "write-poll-limit", "flush-poll-limit")
UNCERTAIN_WRITE = ("write-timeout", "reset-timeout", "write-poll-limit")
CASES = ("errors", "full", "corrupt", "vm-cut-before-ack", "vm-cut-after-flush") + FAULTS
LOW, HIGH = 4 * 512, 36 * 512


def need(ok, message):
    if not ok:
        raise ValueError(message)


def read_json(path):
    return json.loads(path.read_text())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def checksum(data):
    """Existing storage FNV corruption checksum; no authenticity claim."""
    value = 0xCBF29CE484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001B3) & ((1 << 64) - 1)
    return value


def fixture(object_id, revision):
    return bytes((0x31 + object_id * 29 + revision * 7 + i * 3) & 255 for i in range(128))


def initial_bytes(metadata):
    seed = metadata["seed"]
    need(type(seed) is int and 0 <= seed < 1 << 64, "seed range")
    data = bytearray(random.Random(seed).randbytes(65536))
    data[:1024] = bytes(1024)
    label = b"NOS-M6B-TEST-VOLUME\0"
    data[:len(label)] = label
    data[LOW:HIGH] = bytes(HIGH - LOW)
    need(metadata["variant"] in ("normal", "corrupt"), "fixture variant")
    if metadata["variant"] == "corrupt":
        data[LOW] = 0x80
    return bytes(data)


def journal(data):
    need(len(data) == 65536, "volume length")
    state = {"sequence": 0, "objects": [dict(revision=0, present=False, length=0,
              digest=f"{checksum(b''):016x}") for _ in range(4)]}
    records, ended = [], False
    for slot in range(32):
        block = data[LOW + slot * 512:LOW + (slot + 1) * 512]
        if block == bytes(512):
            ended = True
            continue
        need(not ended, "noncontiguous records")
        version, header, flags, seq, request, obj, revision, operation, length = struct.unpack_from("<HHIQQQQHH", block, 8)
        need(block[:8] == b"NOSM6B01" and (version, header, flags) == (1, 64, 0), "record header")
        need(seq == slot + 1 and request > 0 and obj < 4 and 1 <= revision <= seq,
             "record sequence or identity")
        need(operation in (2, 3) and length <= 128 and (operation != 3 or length == 0), "record operation")
        need(not any(block[52:64]) and not any(block[64 + length:504]), "record padding")
        need(struct.unpack_from("<Q", block, 504)[0] == checksum(block[:504] + bytes(8)), "record checksum")
        old = state["objects"][obj]
        need(revision == old["revision"] + 1 and (operation != 3 or old["present"]), "revision transition")
        payload = block[64:64 + length]
        need(payload == (fixture(obj, revision) if operation == 2 else b""), "native fixture payload")
        record = dict(sequence=seq, request_id=request, object=obj, revision=revision,
                      operation=operation, payload_hex=payload.hex())
        records.append(record)
        state["objects"][obj] = dict(revision=revision, present=operation == 2,
                                     length=length, digest=f"{checksum(payload):016x}")
        state["sequence"] = seq
    return state, records


def fault_lines(case):
    name = case.upper().replace("-", "_")
    if case.endswith("error"):
        result = ["M6B:INJECT:" + name.removesuffix("_ERROR") + "_IOERR_SNAPSHOT"]
    else:
        operation = "FLUSH" if case.startswith("flush") else "WRITE"
        result = [f"M6B:INJECT:{operation}_NOTIFY_SUPPRESSED"]
        result += (["M6B:CANCEL:BLOCK_CAP_CLOSED", "M6B:POLL_LIMIT:CANCELLED"]
                   if case.endswith("poll-limit") else [f"M6B:DEADLINE:{operation}_TIMEOUT"])
        result += ["M6B:RESET:REQUESTED"]
        result += (["M6B:INJECT:RESET_ACK_MASKED", "M6B:RESET:QUARANTINED", "M6B:POOL_PINNED:PASS"]
                   if case == "reset-timeout" else ["M6B:RESET:ACKNOWLEDGED", "M6B:POOL_REUSE_AFTER_ACK:PASS"])
    return result + ["M6B:OUTCOME:UNKNOWN", "M6B:NO_REPLAY:PASS", "M6B:NO_STALE_READ:PASS",
                     f"M6B:CASE:{name}:PASS", "M6B:SURVIVOR_PROGRESS:PASS", "M6B:RESOURCES:PASS", "M6B:PASS"]


def main():
    before_head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    need(before_head == HEAD and source_hash() == SOURCE, "frozen source/HEAD")
    reports = read_json(ARCHIVE / "guest-runs.json")
    expected = []
    for name in ("normal-a", "normal-b") + CASES:
        expected.append(name)
        if name != "corrupt":
            expected.append(name + "-cold-recovery")
    expected.append("m1-after-m6b")
    need([r["label"] for r in reports] == expected and len(reports) == 28, "exact native coverage")
    directories = [Path(r["run_directory"]).resolve(strict=True) for r in reports]
    need(len(set(directories)) == 28 and all(p.parent == RUNS for p in directories), "private unique VM directories")
    need({p.parent for p in RUNS.glob("*/result.json")} == set(directories), "unexpected or missing saved VM")
    need(len({((p / "serial.txt").stat().st_dev, (p / "serial.txt").stat().st_ino) for p in directories}) == 28,
         "physical serial-file reuse")
    by_directory, outcomes, recoveries, volumes = {}, [], [], {}
    for report, directory in zip(reports, directories):
        label, case = report["label"], report.get("case")
        need(read_json(directory / "result.json") == report, label + ": captured/raw report")
        need(report["base_commit"] == HEAD and report["source_sha256"] == SOURCE
             and report["source_fingerprint_version"] == 3 and report["timed_out"] is False, label + ": source or timeout")
        stdout = (directory / "serial.txt").read_bytes()
        stderr = (directory / "stderr.txt").read_bytes()
        need(stdout.decode("utf-8", "replace") == report["stdout"] and stderr.decode("utf-8", "replace") == report["stderr"],
             label + ": raw output")
        required_artifacts = {"esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI", "OVMF_CODE.fd", "OVMF_VARS.template.fd"}
        if case is not None:
            required_artifacts |= {"OVMF_VARS.fd", "test-disk.before.img", "test-disk.after.img", "build-manifest.json", "volume-manifest.json"}
        need(set(report["artifact_sha256"]) == required_artifacts, label + ": exact required artifact set")
        for name, actual in report["artifact_sha256"].items():
            need(digest((directory / name).read_bytes()) == actual, label + ": artifact " + name)
        lines = report["stdout"].splitlines()
        need(all(lines.count(m) == 1 for m in ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")), label + ": native boot")
        need(not any(":FAIL" in line or "PANIC" in line for line in lines), label + ": failure marker")
        rc = report["raw_qemu_exit"]
        need(type(rc) is int and rc == (-9 if case in ("vm-cut-before-ack", "vm-cut-after-flush") else 33), label + ": raw exit")
        row = dict(label=label, case=case, run_directory=str(directory), raw_exit=rc,
                   serial_sha256=digest(stdout), artifact_count=len(report["artifact_sha256"]), artifacts_rehashed=True)
        if case is None:
            need(lines.count("M1:PASS") == 1 and not any(line.startswith("M6B:") for line in lines), "M1 baseline")
            outcomes.append(row)
            continue
        need(report["serial_sha256"] == digest(stdout) and report["stderr_sha256"] == digest(stderr), label + ": output hashes")
        manifest = read_json(directory / "build-manifest.json")
        need(manifest["kernel_feature"] == ("milestone-m6b" if case == "normal" else "test-m6b-" + case)
             and manifest["source_sha256"] == SOURCE, label + ": feature/source manifest")
        need(manifest["kernel"]["sha256"] == report["artifact_sha256"]["esp/kernel.elf"]
             and manifest["loader"]["sha256"] == report["artifact_sha256"]["esp/EFI/BOOT/BOOTX64.EFI"], label + ": executed build")
        metadata = read_json(directory / "volume-manifest.json")
        initial = initial_bytes(metadata)
        disk = Path(report["volume_path"]).resolve(strict=True)
        before = (directory / "test-disk.before.img").read_bytes()
        after = (directory / "test-disk.after.img").read_bytes()
        need(disk.parent.parent == RUNS and read_json(disk.parent / "volume.json") == metadata, label + ": volume identity")
        need(len(before) == len(after) == 65536 and disk.read_bytes() == after, label + ": actual volume bytes")
        need(digest(initial) == metadata["initial_sha256"] == report["disk_oracle"]["sha256_initial"], label + ": initial hash")
        need(digest(after) == report["disk_oracle"]["sha256_after"], label + ": oracle after hash")
        need(before[:LOW] == after[:LOW] == initial[:LOW] and before[HIGH:] == after[HIGH:] == initial[HIGH:], label + ": head/tail canaries")
        volumes[str(disk)] = metadata["identity"]
        if case == "corrupt":
            need(before == after == initial and metadata["variant"] == "corrupt", label + ": explicit corrupt fixture")
            try:
                journal(after)
            except ValueError:
                pass
            else:
                raise ValueError("corrupt fixture parsed as valid")
            state, records = None, []
        else:
            state, records = journal(after)
            need(state == report["disk_oracle"]["state"], label + ": independent journal state")
        if case == "recover":
            previous = Path(report["previous_stage"]).resolve(strict=True)
            need(previous in by_directory and previous != directory, label + ": prior distinct invocation")
            old = by_directory[previous]
            need(old["volume_path"] == str(disk) and old["disk_oracle"]["identity"] == metadata["identity"], label + ": same volume")
            need((previous / "test-disk.after.img").read_bytes() == before == after, label + ": exact cold link/no writes")
            need(lines.count("M6B:ACKS:0") == 1, label + ": no mutation acknowledgement")
            recoveries.append(dict(label=label, previous=str(previous), before_after_sha256=digest(after), exact_bytes=True, writes=False, mutation_acks=0))
        else:
            need(before == initial and report["previous_stage"] is None, label + ": fresh initial image")
        if case in UNCERTAIN_WRITE:
            canonical = dict(sequence=1, request_id=10, object=0, revision=1, operation=2, payload_hex=fixture(0, 1).hex())
            need((state["sequence"] == 0 and after == initial and records == [])
                 or (state["sequence"] == 1 and records == [canonical]), label + ": published WRITE outcome bound")
            row["published_write_outcome"] = "unchanged_sequence_0" if not records else "canonical_first_sequence_1"
        if case in FAULTS:
            controlled = [line for line in lines if line.startswith("M6B:")]
            need(controlled == fault_lines(case), label + ": exact unknown/no-ACK/no-replay/reset markers")
            row["unknown_no_ack_no_replay"] = True
            row["reset_policy"] = "quarantined_pool_pinned" if case == "reset-timeout" else "ack_before_pool_reuse" if not case.endswith("error") else "completion_snapshot_error"
        if case in ("normal", "recover", "errors", "full"):
            need(lines.count("M6B:SEQUENCE:" + str(state["sequence"])) == 1, label + ": native sequence")
            for i, obj in enumerate(state["objects"]):
                for field, value in (("REVISION", obj["revision"]), ("PRESENT", int(obj["present"])), ("LENGTH", obj["length"]), ("DIGEST", obj["digest"])):
                    need(lines.count(f"M6B:OBJECT{i}:{field}:{value}") == 1, label + ": native object state")
        if case in ("vm-cut-before-ack", "vm-cut-after-flush"):
            cut = 1 if case == "vm-cut-before-ack" else 2
            crash = report["host_crash"]
            need(crash["kill_requested"] is True and crash["was_running_before_kill"] is True
                 and crash["real_process_returncode"] == -9 and crash["cut_lines"] == [f"M6B:CUT:{cut}"], label + ": live-child crash")
            need("M1:PASS" not in lines and "M6B:PASS" not in lines, label + ": no success after cut")
        row.update(initial_sha256=digest(initial), before_sha256=digest(before), after_sha256=digest(after), canaries_unchanged=True,
                   sequence=state["sequence"] if state else None, records=len(records), actual_volume_matches_snapshot=True)
        by_directory[directory] = report
        outcomes.append(row)
    need(len(recoveries) == 13 and len(volumes) == 14, "cold/volume coverage")
    a, b = reports[0], reports[2]
    need(a["artifact_sha256"]["esp/kernel.elf"] == b["artifact_sha256"]["esp/kernel.elf"]
         and a["disk_oracle"]["sha256_initial"] != b["disk_oracle"]["sha256_initial"]
         and a["disk_oracle"]["state"] == b["disk_oracle"]["state"], "two distinct canary images, same native binary/state")
    need(source_hash() == SOURCE and subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip() == HEAD,
         "source changed during crosscheck")
    result = dict(task="I06 independent retained M6b byte/log crosscheck", passed=True, new_vm_runs=0,
                  source_commit=HEAD, source_sha256=SOURCE, native_runs_rechecked=28, raw33_count=26,
                  host_killed_raw_minus9_count=2, cold_recoveries=13, distinct_volumes=14,
                  crosscheck_script_sha256=digest(Path(__file__).read_bytes()),
                  archive_inputs={name:digest((ARCHIVE / name).read_bytes()) for name in ("guest-runs.json", "suite.json", "capture.json", "execution.json")},
                  source_and_head_unchanged=True, runs=outcomes, recovery_links=recoveries,
                  limits=["No additional VM run; this rechecks retained native artifacts.",
                          "The three published-WRITE native observations are sequence zero; canonical sequence one remains the allowed alternate boundary, not an observed outcome here.",
                          "FNV checks diagnose corruption and do not prove authenticity."])
    (HERE / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print("I06 accepted: 28 unique native runs; 13 byte-exact no-write cold links; all canaries and timeout/reset bounds match.")


if __name__ == "__main__":
    main()
