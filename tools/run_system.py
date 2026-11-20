#!/usr/bin/env python3
"""FR-I01: real joint-service guest and cold recovery on a guarded test image.

This is an integration gate, not the full release runner. It never accepts an
arbitrary disk and independently checks the actual volume after the guest exits.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import time
import uuid

from common import ROOT, BUILD, environment, firmware, qemu_accel, qemu_machine, save_json, sha256, source_hash
from run_volume import execute
import volume_disk

FEATURE = {"normal": "native-system", "recover": "test-system-recover"}
# H05 (ADR-076): power cases run the normal flow, then init's POWER_OFF. They
# stay out of FEATURE, which capture/gate drivers iterate as the FR-I01 cases.
POWER_FEATURE = {"power": "native-system-power", "power-flush-error": "test-system-power-flush-error",
                 "power-flush-timeout": "test-system-power-flush-timeout"}
# H05-T4: the same flow, but init first requests one synchronous ACPI S3 cycle
# (syscall 0x91) behind the same storage gate. The harness alone wakes the
# guest over QMP; stale-token stops at the rejected wake, not-durable never
# sleeps. Separate from POWER_FEATURE so the H05 POWER cases stay unchanged.
S3_FEATURE = {"s3": "native-system-s3", "s3-stale-token": "test-system-s3-stale-token",
              "s3-not-durable": "test-system-s3-not-durable"}
S3_WAKEUPS = {"s3": 1, "s3-stale-token": 1, "s3-not-durable": 0}
POWER_CASES = (*POWER_FEATURE, *S3_FEATURE)
S5_TAIL = ("POWER:S5:PM1A=0x0604:SLP_TYP=0:SCI_EN=1", "POWER:S5:WRITE")
# Kernel-observed S3 evidence (system.rs SLEEP_*): exact values and minimums.
SLEEP_EXACT = {
    "s3": {"SLEEP_INIT_REFUSALS": 3, "SLEEP_APP_REFUSALS": 2, "SLEEP_GENERATION": 1,
           "SLEEP_DEVICE_IO_AFTER_WAKE": 0, "SLEEP_DEVICE_REFUSALS": 0},
    "s3-not-durable": {"SLEEP_APP_REFUSALS": 2, "SLEEP_GENERATION": 0, "SLEEP_TIMER_IRQS": 0,
                       "SLEEP_DEVICE_IO_AFTER_WAKE": 0, "SLEEP_DEVICE_REFUSALS": 0},
}
SLEEP_MINIMUM = {"s3": {"SLEEP_TIMER_IRQS": 2}, "s3-not-durable": {"SLEEP_INIT_REFUSALS": 4}}
SLEEP_TAIL = ("SYSTEM:SLEEP:TIMER:WAIT", "SYSTEM:SLEEP:RESUMED:PASS")
# B02 persistence objects: the app's sealed record (domain 0) and init's
# encrypted key-state snapshot (domain 1). Both are ciphertext, never fixtures.
SEALED_OBJECT, SEALED_LENGTH, SEALED_NAME = 1, 168, "verzegeld"
KEYSTATE_OBJECT, KEYSTATE_LENGTH, KEYSTATE_NAME = 8, 120, "sleutelstatus"
OPAQUE_OBJECTS = frozenset({SEALED_OBJECT, KEYSTATE_OBJECT})
VOLUME_ID = b"NOS-SYSTEM-VOL01"
KEYSTATE_HEADER = b"NOSKEY01" + (1).to_bytes(2, "little") * 2 + VOLUME_ID + bytes(4)
# Per-case B02 evidence: normal exports/commits before the app commits
# ciphertext; recover restores from the unchanged image before any key work.
PERSIST_EXACT = {
    "normal": {"CRYPTO_SEAL": 3, "CRYPTO_UNSEAL": 2, "CRYPTO_TAMPER_DENIED": 10, "CRYPTO_RESTORES": 1,
               "KEYSTATE_EXPORTS": 1, "KEYSTATE_RESTORES": 1, "KEYSTATE_WRONG_SECRET_DENIED": 1,
               "KEYSTATE_FLOOR_DENIED": 1, "KEYSTATE_ADMIN_OP_DENIED": 1, "KEYSTATE_BYTES": 120,
               "KEYSTATE_GENERATION": 3, "ADMIN_REQUESTS": 5, "ADMIN_DENIED": 3,
               "RECORD_PERSISTED": 1, "RECORD_RESTORED_UNSEAL": 0, "RECORD_RESTORED_REVOKED_DENIED": 0,
               "RECORD_RESTORED_TAMPER_DENIED": 0, "RECORD_ADMIN_DENIED": 2, "RECORD_OBJECT_REVISION": 1},
    "recover": {"CRYPTO_SEAL": 2, "CRYPTO_UNSEAL": 3, "CRYPTO_TAMPER_DENIED": 11, "CRYPTO_RESTORES": 1,
                "KEYSTATE_EXPORTS": 0, "KEYSTATE_RESTORES": 1, "KEYSTATE_WRONG_SECRET_DENIED": 1,
                "KEYSTATE_FLOOR_DENIED": 1, "KEYSTATE_ADMIN_OP_DENIED": 1, "KEYSTATE_BYTES": 120,
                "KEYSTATE_GENERATION": 3, "ADMIN_REQUESTS": 4, "ADMIN_DENIED": 3,
                "RECORD_PERSISTED": 0, "RECORD_RESTORED_UNSEAL": 1, "RECORD_RESTORED_REVOKED_DENIED": 1,
                "RECORD_RESTORED_TAMPER_DENIED": 1, "RECORD_ADMIN_DENIED": 2, "RECORD_OBJECT_REVISION": 1},
}


def record_plaintext() -> bytes:
    """Independent copy of the app's deterministic record plaintext."""
    return bytes((11 + i * 17 + i // 97) & 0xFF for i in range(128))
RANDOM_FEATURE = {"raw":"native-system-random", "sdk":"test-system-random-sdk", "unknown":"test-system-random-unknown", "wrong-caller":"test-system-random-wrong-caller"}
TCP_FEATURE = {"normal": "native-system-tcp", "recover": "test-system-tcp-recover"}
# H04-T3 link loss/reconnect profiles; driven by tools/run_net_link.py only.
LINK_FEATURES = ("test-system-net-link", "test-system-net-link-storm")
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")
COLD_CASES = ("keystate", "record-tag", "swapped-blocks", "withheld-secret")
MAX_TIMEOUT = 180


def check_esp(esp: Path, manifest: dict) -> None:
    """Check the entire source tree before copying any byte into guest media."""
    esp = esp.absolute()
    for parent in (esp, *esp.parents):
        details = parent.lstat()
        if not stat.S_ISDIR(details.st_mode):
            raise ValueError("ESP directory/ancestor must be a real directory")
    expected = {"kernel.elf", "EFI/BOOT/BOOTX64.EFI"}
    files = set()
    for entry in esp.rglob("*"):
        details = entry.lstat()
        relative = entry.relative_to(esp).as_posix()
        if stat.S_ISDIR(details.st_mode):
            if relative not in {"EFI", "EFI/BOOT"}:
                raise ValueError("Unexpected ESP directory")
        elif stat.S_ISREG(details.st_mode) and details.st_nlink == 1:
            files.add(relative)
        else:
            raise ValueError("Linked or special ESP entry")
    if files != expected:
        raise ValueError("ESP must contain exactly the inspected loader and kernel")
    for relative, key in (("kernel.elf", "kernel"), ("EFI/BOOT/BOOTX64.EFI", "loader")):
        path = esp / relative
        details = path.lstat()
        if not stat.S_ISREG(details.st_mode) or details.st_nlink != 1:
            raise ValueError(f"ESP {relative} must be a regular single-link copy")
        if sha256(path) != manifest[key]["sha256"]:
            raise ValueError(f"ESP {relative} differs from the inspected build manifest")


def check_image(image: Path) -> None:
    """Reject links/devices/directories before opening or hashing the disk."""
    details = image.lstat()
    if (not stat.S_ISREG(details.st_mode) or details.st_nlink != 1 or
            details.st_size != volume_disk.IMAGE_SECTORS * volume_disk.SECTOR):
        raise ValueError("test disk must be a regular single-link generated image of the exact geometry")


def network_arguments(port: int, *, status: bool = True) -> list[str]:
    if type(port) is not int or not 1024 <= port <= 65535:
        raise ValueError("network test peer requires an ephemeral local port")
    if type(status) is not bool:
        raise ValueError("NIC status feature selection must be a boolean")
    # OVMF can transmit IPv6 during boot, before our driver exists. Disable its
    # network drivers via the documented EDK2 runtime policy, plus the NIC ROM.
    # The PCI NIC remains present for the real Nieuw OS driver and strict peer.
    return ["-fw_cfg", "name=opt/org.tianocore/VirtioNetSupport,string=no",
            "-fw_cfg", "name=opt/org.tianocore/IPv4Support,string=no",
            "-fw_cfg", "name=opt/org.tianocore/IPv6Support,string=no",
            "-netdev", f"socket,id=net0,connect=127.0.0.1:{port}",
            "-device", "virtio-net-pci,netdev=net0,id=net,addr=0x7,disable-legacy=on,mac=02:00:00:00:00:07,romfile="
            + ("" if status else ",status=off")]


def check_parent(previous: dict, frozen: str, image_sha256: str, *, network: bool, tcp=False) -> None:
    if tcp and not network:
        raise ValueError("TCP recovery requires the native network profile")
    parent_feature = TCP_FEATURE["normal"] if tcp else ("native-system-net" if network else FEATURE["normal"])
    # A powered-off (or power-refused) image must recover exactly like a normal one.
    parent_case = previous.get("case")
    if not network and parent_case in POWER_CASES:
        parent_feature = {**POWER_FEATURE, **S3_FEATURE}[parent_case]
    else:
        parent_case = "normal" if parent_case == "normal" else None
    if not (parent_case is not None and previous.get("passed") is True
            and previous.get("kernel_feature") == parent_feature
            and previous.get("source_sha256") == frozen
            and previous.get("after_sha256") == image_sha256):
        raise ValueError("recovery needs the unchanged image from a passing normal run")
    # Generic system assertions precede the peer's final transcript/shutdown
    # verdict. A process crash in that interval cannot authorize net recovery.
    if network and previous.get("network_gate") is not True:
        raise ValueError("network recovery requires the completed network peer gate")
    if tcp and previous.get("tcp_gate") is not True:
        raise ValueError("TCP recovery requires the completed TCP peer gate")


def expected_markers(case: str) -> tuple[str, ...]:
    if case not in FEATURE and case not in POWER_CASES:
        raise ValueError("unknown system case")
    if case == "s3-stale-token":
        # The kernel stops at the rejected wake; nothing after the manifest.
        return BOOT + ("SYSTEM:MANIFEST:PASS",)
    return BOOT + ("SYSTEM:MANIFEST:PASS", "SYSTEM:BROKERS:PASS",
                   "SYSTEM:CLEANUP:PASS", f"SYSTEM:CASE:{case.upper().replace('-', '_')}:PASS", "M1:PASS")


def check_power(case: str, stdout: str, found: dict) -> None:
    """Exact S5 tail after M1:PASS, or a refusal without any PM1 write."""
    lines = stdout.splitlines()
    if case in ("power", "s3"):
        if tuple(lines[-3:]) != ("M1:PASS", *S5_TAIL) or "SYSTEM:POWER:GATE:PASS" not in lines:
            raise ValueError("power case did not end with the gated S5 write")
        expected = {"POWER_INIT_REFUSALS": 3, "POWER_APP_REFUSALS": 2, "POWER_NOT_DURABLE": 0}
    else:
        fault = "FLUSH_TIMEOUT" if case == "power-flush-timeout" else "FLUSH_ERROR"
        if (any(line.startswith("POWER:") for line in lines)
                or lines.count(f"SYSTEM:FAULT_INJECTED:{fault}") != 1
                or "SYSTEM:POWER:REFUSED:NOT_DURABLE" not in lines):
            raise ValueError("refused power case wrote PM1 or lacks its fault/refusal marker")
        expected = {"POWER_APP_REFUSALS": 2, "POWER_NOT_DURABLE": 1}
        if found.get("POWER_INIT_REFUSALS", 0) < 4:
            raise ValueError("refused power case lacks the final -16 refusal")
    for key, value in expected.items():
        if found.get(key) != value:
            raise ValueError(f"{key}: expected {value}, found {found.get(key)}")


def check_sleep(case: str, stdout: str, events: list[str] | None) -> dict:
    """Exact S3 sub-sequence of a ring-3 S3 case plus the independent QMP events.

    `s3`: after the manifest and before BROKERS exactly PLAN, ENTER (cycle 1),
    RESUMED (directly after ENTER), the timer wait and RESUMED:PASS.
    `s3-stale-token`: PLAN, ENTER, then the token rejection ends the guest.
    `s3-not-durable`: no S3 line and no QMP SUSPEND at all.
    """
    from run_sleep import ENTER, PLAN, RESUMED, STALE
    if case not in S3_FEATURE:
        raise ValueError("not an S3 system case")
    if events is None or any(type(event) is not str for event in events):
        raise ValueError("S3 system case needs its recorded QMP events")
    sleep_events = [event for event in events if event in ("SUSPEND", "WAKEUP")]
    if sleep_events != ["SUSPEND", "WAKEUP"] * S3_WAKEUPS[case]:
        raise ValueError(f"unexpected QMP sleep events: {sleep_events}")
    # The guest's S5 after the wake is a QMP SHUTDOWN (measured on QEMU 8.2):
    # exactly one, after WAKEUP, and only in the case that powers off.
    shutdown = [event for event in events if event == "SHUTDOWN"]
    if case == "s3":
        if shutdown != ["SHUTDOWN"] or events.index("SHUTDOWN") < events.index("WAKEUP"):
            raise ValueError(f"missing or misplaced QMP SHUTDOWN after the wake: {events}")
    elif shutdown:
        raise ValueError("QMP SHUTDOWN in a case that must not power off")
    lines = stdout.splitlines()
    manifest = lines.index("SYSTEM:MANIFEST:PASS")
    found = [(index, line) for index, line in enumerate(lines)
             if line.startswith(("POWER:S3:", "SYSTEM:SLEEP:"))]
    texts = [line for _, line in found]
    observed = {"qmp_sleep_events": sleep_events, "qmp_shutdown_events": len(shutdown)}
    if case == "s3-not-durable":
        if texts:
            raise ValueError("not-durable case reached the S3 path")
        return observed
    if any(index < manifest for index, _ in found):
        raise ValueError("S3 marker before the system manifest")
    if len(texts) < 3 or not (plan := PLAN.fullmatch(texts[0])) or texts[1] != ENTER.format(1):
        raise ValueError(f"missing S3 plan/entry: {texts[:2]}")
    real, pm32 = (int(value, 16) for value in plan.groups())
    if not (0x1000 <= real < 0xa0000 and real % 0x1000 == 0 and 0 < pm32 < 1 << 32 and pm32 % 16 == 0):
        raise ValueError("wake entries outside the real-mode page / 32-bit image bounds")
    if found[2][0] != found[1][0] + 1:
        raise ValueError("S3 entry not directly followed by the wake report")
    observed.update(real_mode_entry=hex(real), protected_entry=hex(pm32))
    if case == "s3-stale-token":
        if texts[2:] != [STALE] or [line for line in lines if line][-1] != STALE:
            raise ValueError(f"stale token did not end the guest at the rejected wake: {texts[2:]}")
        if any(re.fullmatch(r"SYSTEM:[A-Z_]+:[0-9]{20}", line) for line in lines):
            raise ValueError("stale-token guest reported system evidence after the wake")
        observed["rejection"] = STALE
        return observed
    resumed = RESUMED.fullmatch(texts[2])
    if not resumed or resumed.group(1) != "1" or not int(resumed.group(2), 16) & 0x8000:
        raise ValueError(f"unexpected S3 wake report: {texts[2]}")
    if tuple(texts[3:]) != SLEEP_TAIL:
        raise ValueError(f"unexpected post-wake sequence: {texts[3:]}")
    if found[-1][0] > lines.index("SYSTEM:BROKERS:PASS"):
        raise ValueError("S3 sequence after the broker check")
    observed.update(pm1_status=resumed.group(2), sci_en=resumed.group(3))
    return observed


def evaluate(case: str, stdout: str, raw_exit: int | None, timed_out: bool,
             events: list[str] | None = None) -> dict[str, int]:
    expected = expected_markers(case)
    controlled = [line for line in stdout.splitlines()
                  if line in BOOT or line == "M1:PASS" or
                  line.startswith(("SYSTEM:MANIFEST:", "SYSTEM:BROKERS:",
                                   "SYSTEM:CLEANUP:", "SYSTEM:CASE:"))]
    if controlled != list(expected):
        raise ValueError(f"system marker sequence mismatch: {controlled}")
    # Only a guest poweroff exits 0: the power runs override -no-reboot.
    if timed_out or raw_exit != (0 if case in ("power", "s3") else 33):
        raise ValueError(f"raw exit {raw_exit}, timed_out {timed_out}")
    if any("PANIC" in line or "FAIL" in line for line in stdout.splitlines()):
        raise ValueError("guest failure marker")
    if case in S3_FEATURE:
        sleep = check_sleep(case, stdout, events)
        if case == "s3-stale-token":
            # The guest stopped at the rejected wake: no kernel evidence follows.
            return {"sleep": sleep}
    elif events is not None:
        raise ValueError("QMP events belong to the S3 system cases only")
    found = {}
    for line in stdout.splitlines():
        match = re.fullmatch(r"SYSTEM:([A-Z_]+):([0-9]{20})", line)
        if match:
            key = match.group(1)
            if key in found:
                raise ValueError(f"duplicate system evidence {key}")
            found[key] = int(match.group(2))
    # Checked against real IPC/syscall/driver observations in the kernel, not
    # a host-side substitute for OS functionality. The disk is checked below.
    exact = {"APP_READY": 2, "APP_DOCS": 1, "APP_REVISION": 1, "APP_BYTES": 4096,
             "APP_DENIED": 3, "WORKER_SPAWNS": 2, "WORKER_FAULTS": 1,
             "WORKER_RESTARTS": 1, "WORKER_KILLS": 1,
             "LIVE_PROCESSES_AFTER": 0, "FRAMES_RECOVERED": 1,
             "RECORD_SEAL": 1, "RECORD_UNSEAL": 1,
             "RECORD_CONTEXT_DENIED": 10, "RECORD_REVOKED_DENIED": 2,
             **PERSIST_EXACT["recover" if case == "recover" else "normal"]}
    minimum = {"APP_DENIED": 3, "DISK_READS": 1, "ENTROPY_BYTES": 256,
               "CRYPTO_SEAL": 1, "CRYPTO_UNSEAL": 1, "CRYPTO_TAMPER_DENIED": 1}
    for key, value in exact.items():
        if found.get(key) != value:
            raise ValueError(f"{key}: expected {value}, found {found.get(key)}")
    for key, value in minimum.items():
        if found.get(key, -1) < value:
            raise ValueError(f"{key}: expected at least {value}, found {found.get(key)}")
    if case != "recover" and found.get("DISK_WRITES", 0) < 1:
        raise ValueError("non-recovery guest did not write")
    if case == "recover" and found.get("DISK_WRITES") != 0:
        raise ValueError("recovery guest attempted a write")
    # Power cases add init's SHUTDOWN flush (refused cases: published, never completed).
    if ((case != "recover" and found.get("DISK_FLUSHES", 0) < (4 if case in POWER_CASES else 2)) or
            (case == "recover" and found.get("DISK_FLUSHES") != 0)):
        raise ValueError("wrong durability/readonly flush evidence")
    if case in POWER_CASES:
        check_power(case, stdout, found)
    if case in S3_FEATURE:
        for key, value in SLEEP_EXACT[case].items():
            if found.get(key) != value:
                raise ValueError(f"{key}: expected {value}, found {found.get(key)}")
        for key, value in SLEEP_MINIMUM[case].items():
            if found.get(key, -1) < value:
                raise ValueError(f"{key}: expected at least {value}, found {found.get(key)}")
    elif any(key.startswith("SLEEP_") for key in found):
        raise ValueError("S3 evidence in a profile without S3")
    expected_sum = volume_disk.fnv(volume_disk.fixture(0, 1, 4096))
    if found.get("APP_CONTENT_SUM") != expected_sum:
        raise ValueError("guest read content differs from the independent fixture")
    if case in S3_FEATURE:
        return {**found, "sleep": sleep}
    return found


def _single_version(item: dict, what: str, expected: dict) -> dict:
    if not item["present"] or item["revision"] != 1 or len(item["versions"]) != 1:
        raise ValueError(f"missing or incorrect committed {what}")
    version = item["versions"][0]
    if any(version.get(key) != value for key, value in expected.items()):
        raise ValueError(f"{what} bytes/name/revision differ from oracle")
    if (item["collection"], item["tags"], item["kind"]) != (0, 0, 0):
        raise ValueError(f"unexpected {what} metadata")
    return version


def _opaque_bytes(version: dict, length: int, what: str) -> bytes:
    content = version.get("content_hex")
    if type(content) is not str:
        raise ValueError(f"{what} has no independently decoded content")
    data = bytes.fromhex(content)
    if len(data) != length or version.get("content_revision") is not None:
        raise ValueError(f"{what} is not opaque ciphertext of the expected length")
    return data


def check_disk(oracle: dict) -> None:
    objects = oracle["objects"]
    _single_version(objects[0], "system document",
                    {"revision": 1, "length": 4096, "name": "systeemproef", "content_revision": 1})
    # B02: the sealed record is committed ciphertext (nonce | ciphertext | tag),
    # never the known plaintext, and the key-state snapshot carries only the
    # public format/suite/volume header outside its AEAD body.
    sealed = _opaque_bytes(_single_version(objects[SEALED_OBJECT], "sealed record",
                                           {"revision": 1, "length": SEALED_LENGTH, "name": SEALED_NAME}),
                           SEALED_LENGTH, "sealed record")
    plaintext = record_plaintext()
    if plaintext in sealed or sealed[24:24 + len(plaintext)] == plaintext or not any(sealed[:24]):
        raise ValueError("sealed record stores recognizable plaintext or an empty nonce")
    keystate = _opaque_bytes(_single_version(objects[KEYSTATE_OBJECT], "key-state snapshot",
                                             {"revision": 1, "length": KEYSTATE_LENGTH, "name": KEYSTATE_NAME}),
                             KEYSTATE_LENGTH, "key-state snapshot")
    if keystate[:len(KEYSTATE_HEADER)] != KEYSTATE_HEADER or not any(keystate[32:56]):
        raise ValueError("key-state snapshot header/volume binding or nonce is wrong")
    untouched = [obj for index, obj in enumerate(objects) if index not in (0, SEALED_OBJECT, KEYSTATE_OBJECT)]
    if any(obj["present"] or obj["revision"] for obj in untouched):
        raise ValueError("guest touched another object or domain")
    if any(collection["present"] or collection["revision"] for collection in oracle["collections"]):
        raise ValueError("guest changed an unrelated collection")
    # Three commits: document and sealed record (domain 0), snapshot (domain 1).
    if (oracle["generation"], oracle["chunks_used"], oracle["clocks"]) != (3, 3, [2, 1]):
        raise ValueError("unexpected volume generation, allocation or domain clock")


def cold_image(raw: bytes, case: str) -> bytes:
    """Mutate AEAD bytes while repairing the noncryptographic volume checksums.

    The sole sealed record has one volume chunk. swapped-blocks exchanges its
    two 64-byte ciphertext blocks; this does not prove multi-record ordering.
    """
    if case not in COLD_CASES:
        raise ValueError("unknown cold case")
    check_disk(volume_disk.analyse(raw, OPAQUE_OBJECTS))
    if case == "withheld-secret":
        return raw
    disk = volume_disk
    image = bytearray(raw)
    volume = raw[disk.FIRST * disk.SECTOR:]
    table = disk.choose(volume)
    obj = KEYSTATE_OBJECT if case == "keystate" else SEALED_OBJECT
    version = table["entries"][obj]["current"]
    start = (disk.FIRST + disk.DATA_START + version["chunks"][0] * disk.CHUNK_SECTORS) * disk.SECTOR
    length = version["length"]
    if case == "swapped-blocks":
        image[start + 24:start + 152] = image[start + 88:start + 152] + image[start + 24:start + 88]
    else:
        image[start + (56 if case == "keystate" else length - 1)] ^= 1
    checksum = disk.fnv(image[start:start + length])
    for slot in range(2):
        sectors, _ = disk.slot_states(volume, slot)
        current = disk.decode_table(slot, sectors)
        entry = current["entries"][obj]
        if entry["present"]:
            entry["current"]["content_sum"] = checksum
            entry["current"]["sums"] = [checksum]
            for index, sector in enumerate(disk.encode_table(current["generation"], current["request_id"],
                    current["entries"], current["collections"], current["clocks"])):
                offset = (disk.FIRST + slot * disk.TABLE_SECTORS + index) * disk.SECTOR
                image[offset:offset + disk.SECTOR] = sector
    # Decode all content independently: a volume checksum error is not AEAD evidence.
    disk.analyse(bytes(image), OPAQUE_OBJECTS)
    return bytes(image)


def evaluate_cold(case: str, stdout: str, raw_exit: int | None, timed_out: bool) -> dict:
    if case not in COLD_CASES:
        raise ValueError("unknown cold case")
    lines = stdout.splitlines()
    marker = f"SYSTEM:COLD:REFUSED:{case}"
    controlled = [line for line in lines if line.startswith("SYSTEM:COLD:")]
    if controlled != [marker] or raw_exit != 33 or timed_out:
        raise ValueError("cold refusal case/exit mismatch")
    if any("FAIL" in line or "PANIC" in line or "PLAINTEXT" in line and
           line != "SYSTEM:COLD_PLAINTEXT_REPLIES:00000000000000000000" for line in lines):
        raise ValueError("cold refusal exposed plaintext or failed")
    found = {}
    for key in ("COLD_PLAINTEXT_REPLIES", "DISK_READS", "DISK_WRITES", "DISK_FLUSHES"):
        matches = [line for line in lines if line.startswith(f"SYSTEM:{key}:")]
        if len(matches) != 1 or not re.fullmatch(rf"SYSTEM:{key}:[0-9]{{20}}", matches[0]):
            raise ValueError(f"missing/duplicate cold evidence {key}")
        found[key] = int(matches[0].rsplit(":", 1)[1])
    if found["DISK_READS"] < 1 or any(found[k] for k in ("COLD_PLAINTEXT_REPLIES", "DISK_WRITES", "DISK_FLUSHES")):
        raise ValueError("cold plaintext or disk mutation")
    if [line for line in lines if line in BOOT] != list(BOOT) or lines.count("SYSTEM:MANIFEST:PASS") != 1:
        raise ValueError("cold boot/manifest evidence missing")
    return found


def run_system(case: str, volume_dir: Path | None = None, *, timeout: int = MAX_TIMEOUT,
               kernel_feature: str | None = None, network_port: int | None = None,
               rtc_base: str | None = None, cold_case: str | None = None,
               nic_status: bool = True, qmp_hook=None):
    cases = {**FEATURE, **POWER_FEATURE, **S3_FEATURE}
    if case not in cases or not 0 < timeout <= MAX_TIMEOUT:
        raise ValueError("unknown case or unbounded timeout")
    if case in S3_FEATURE:
        from run_sleep import MACHINE
        if qemu_machine() != MACHINE:
            raise ValueError(f"S3 system cases are pinned to {MACHINE}; set NOS_QEMU_MACHINE={MACHINE}")
    if cold_case is not None and (case != "recover" or cold_case not in COLD_CASES or kernel_feature is not None):
        raise ValueError("cold cases require the ordinary recover profile")
    selected = cases[case] if kernel_feature is None else kernel_feature
    network_feature = {"normal": "native-system-net", "recover": "test-system-net-recover"}.get(case)
    diagnostic = case == "normal" and selected in (
        *RANDOM_FEATURE.values(), "native-system-images", "native-system-clock")
    link = case == "normal" and selected in LINK_FEATURES
    if selected not in (cases[case], network_feature, TCP_FEATURE.get(case)) and not diagnostic and not link:
        raise ValueError("wrong native system feature for this case")
    if (qmp_hook is not None or nic_status is not True) and not link:
        raise ValueError("QMP link control and a STATUS-less NIC belong to the link profile only")
    use_tcp = selected == TCP_FEATURE.get(case)
    use_network = selected == network_feature or use_tcp or link
    if use_network != (type(network_port) is int and 1024 <= network_port <= 65535):
        raise ValueError("network profile requires a dedicated local test peer port")
    if not use_network and network_port is not None:
        raise ValueError("ordinary system profile has no networking")
    from run_fpu import private_path
    private_path(BUILD)
    frozen = source_hash()
    manifest = json.loads((BUILD / "manifest.json").read_text())
    if manifest.get("kernel_feature") != selected or manifest.get("source_sha256") != frozen:
        raise ValueError(f"build {selected} on the current source first")
    if not manifest.get("system"):
        raise ValueError("missing inspected system-program manifest")
    rtc_arguments = []
    if rtc_base is not None:
        parsed = datetime.strptime(rtc_base, "%Y-%m-%dT%H:%M:%S")
        if parsed.strftime("%Y-%m-%dT%H:%M:%S") != rtc_base:
            raise ValueError("RTC base must be a canonical UTC calendar")
        rtc_arguments = ["-rtc", f"base={rtc_base},clock=vm,driftfix=none"]
    if (selected == "native-system-clock") != (rtc_base is not None):
        raise ValueError("the wall-clock acceptance profile requires its explicit UTC RTC base")
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("qemu-system-x86_64 not found")
    code, variables = firmware()
    # Reject unknown files/links in the source before copytree can follow them.
    # Checking only the copied known files would expose extra host data to FAT.
    check_esp(BUILD / "esp", manifest)
    run_dir = private_path(BUILD / "run" / f"system-{case}-{uuid.uuid4().hex}")
    run_dir.mkdir(parents=True, exist_ok=False)
    # Runners only support generated paths inside this checkout.
    run_dir.resolve().relative_to(ROOT.resolve())
    shutil.copytree(BUILD / "esp", run_dir / "esp")
    check_esp(run_dir / "esp", manifest)
    shutil.copy2(code, run_dir / "OVMF_CODE.fd")
    shutil.copy2(variables, run_dir / "OVMF_VARS.fd")
    if case != "recover":
        if volume_dir is not None:
            raise ValueError("normal always creates a new disposable disk")
        volume_dir = BUILD / "run" / f"system-image-{uuid.uuid4().hex}"
        volume_dir.mkdir(parents=True, exist_ok=False)
        volume_disk.create_image(volume_dir / "volume.img", "empty")
    else:
        if volume_dir is None:
            raise ValueError("cold recovery needs the previous normal image")
        volume_dir = volume_dir.resolve(strict=True)
        volume_dir.relative_to((BUILD / "run").resolve())
        check_image(volume_dir / "volume.img")
        previous = json.loads((volume_dir / "last-system-stage.json").read_text())
        check_parent(previous, frozen, sha256(volume_dir / "volume.img"), network=use_network, tcp=use_tcp)
    if cold_case is not None:
        parent_dir = volume_dir
        volume_dir = run_dir / "cold-image"
        volume_dir.mkdir()
        (volume_dir / "volume.img").write_bytes(cold_image((parent_dir / "volume.img").read_bytes(), cold_case))
    image = volume_dir / "volume.img"
    check_image(image)
    before = sha256(image)
    rel = run_dir.relative_to(ROOT).as_posix()
    disk_rel = image.relative_to(ROOT).as_posix()
    readonly = "on" if case == "recover" else "off"
    command = [qemu, "-machine", qemu_machine(), "-accel", qemu_accel(), "-cpu", "qemu64",
               "-smp", "1", "-m", "256M", "-nodefaults", "-display", "none", "-monitor", "none",
               "-serial", "stdio", "-nic", "none", "-no-reboot",
               "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
               "-drive", f"if=pflash,unit=1,format=raw,file={rel}/OVMF_VARS.fd",
               "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
               "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
               "-drive", f"if=none,id=volume,format=raw,cache=writeback,readonly={readonly},file={disk_rel}",
               "-device", "virtio-blk-pci,drive=volume,id=volblk,addr=0x5,disable-legacy=on,num-queues=1,queue-size=8,serial=NOS-SYSTEM-TEST",
               "-object", "rng-random,id=rng0,filename=/dev/urandom",
               "-device", "virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000"]
    command += rtc_arguments
    if case in POWER_CASES:
        # A reset re-runs firmware (second EBS) instead of exiting 0.
        command += ["-action", "reboot=reset"]
    if use_network:
        # An ephemeral port is supplied by the owned synthetic protocol peer.
        # No TAP, bridge, default user network, host share or public listener.
        command += network_arguments(network_port, status=nic_status)
    if qmp_hook is not None:
        # Private QMP socket inside this run directory (relative to QEMU's
        # cwd). The link harness is its only client and permits nothing but
        # qmp_capabilities and set_link; no HMP, TAP or other listener.
        command += ["-qmp", f"unix:{rel}/qmp.sock,server=on,wait=off"]
    started = datetime.now(timezone.utc).isoformat()
    begin = time.monotonic()
    on_line = None if qmp_hook is None else (lambda line, process: qmp_hook(line, process, run_dir))
    s3_control = None
    if case in S3_FEATURE:
        # The S3 controller replaces `-serial stdio` with a file in run_dir and
        # owns a private `-qmp stdio`: query-status, system_wakeup, events.
        from run_sleep import controlled_guest
        s3_control = controlled_guest(command, cwd=ROOT, env=env, timeout=timeout, run_dir=run_dir,
                                      wakeups=S3_WAKEUPS[case])
        rc, timed_out = s3_control["returncode"], s3_control["timed_out"]
        stdout = s3_control["stdout"].decode("utf-8", errors="replace")
        stderr = s3_control["stderr"].decode("utf-8", errors="replace")
    else:
        try:
            rc, stdout, stderr, timed_out, _, _ = execute(command, env, timeout=timeout, on_line=on_line)
        except Exception as error:
            # A failed link-control hook killed QEMU: keep the partial serial log.
            (run_dir / "serial.txt").write_text(getattr(error, "partial_stdout", ""))
            raise
    duration = time.monotonic() - begin
    (run_dir / "serial.txt").write_text(stdout)
    (run_dir / "stderr.txt").write_text(stderr)
    after = sha256(image)
    report = dict(schema_version=1, case=case, source_commit=subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), source_fingerprint_version=3,
        source_sha256=frozen, kernel_feature=selected, started_utc=started,
        duration_s=duration, timeout_s=timeout, raw_qemu_exit=rc, timed_out=timed_out,
        command=command, qemu_version=subprocess.check_output([qemu, "--version"], env=env, text=True).strip(),
        manifest=manifest, run_directory=str(run_dir), volume_directory=str(volume_dir),
        before_sha256=before, after_sha256=after,
        firmware_sha256={"code": sha256(code), "vars_template": sha256(variables)},
        esp_sha256={p.relative_to(run_dir).as_posix(): sha256(p)
                    for p in (run_dir / "esp").rglob("*") if p.is_file()}, passed=False)
    if use_network:
        report["network_gate"] = False
    if use_tcp:
        report["tcp_gate"] = False
    if link:
        report["nic_status"] = nic_status
        report["qmp_socket"] = "qmp.sock" if qmp_hook is not None else None
    if s3_control is not None:
        report["s3_control"] = {key: s3_control[key] for key in ("trace", "events", "observations", "error")}
    try:
        if s3_control is not None and s3_control["error"] is not None:
            raise RuntimeError(f"S3 controller: {s3_control['error']}")
        report["cold_case"] = cold_case
        report["fixture_environment"] = {"NOS_SYSTEM_COLD_CASE": os.environ.get("NOS_SYSTEM_COLD_CASE")}
        report["cold_antirollback"] = "UNPROTECTED: no independently trusted cold generation floor"
        report["evidence"] = (evaluate_cold(cold_case, stdout, rc, timed_out) if cold_case is not None
                              else evaluate(case, stdout, rc, timed_out,
                                            None if s3_control is None else s3_control["events"]))
        if case == "recover" and before != after:
            raise ValueError("read-only cold recovery changed the disk")
        oracle = volume_disk.analyse(image.read_bytes(), OPAQUE_OBJECTS)
        if cold_case is None:
            check_disk(oracle)
        report["oracle"] = oracle
        if source_hash() != frozen:
            raise ValueError("source changed during guest run")
        report["passed"] = True
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    save_json(run_dir / "result.json", report)
    save_json(volume_dir / "last-system-stage.json", report)
    shutil.copy2(image, run_dir / "volume.after.img")
    print(f"SYSTEM {case}: passed={report['passed']} raw={rc} duration={duration:.1f}s {run_dir}")
    if "error" in report:
        print(report["error"])
    return report, volume_dir


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=(*FEATURE, *POWER_FEATURE, *S3_FEATURE), required=True)
    parser.add_argument("--volume", type=Path)
    parser.add_argument("--cold-case", choices=COLD_CASES)
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    options = parser.parse_args()
    raise SystemExit(0 if run_system(options.case, options.volume, timeout=options.timeout, cold_case=options.cold_case)[0]["passed"] else 1)
