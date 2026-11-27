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
                 "power-flush-timeout": "test-system-power-flush-timeout",
                 # H05-T4 (ADR-H05-S3-SYSTEM-20261123): one granted S3 cycle, then S5.
                 "power-s3": "test-system-s3",
                 # H05-T5b: the same cycle, then virtio-rng re-initialised after the wake.
                 "power-s3-reinit": "test-system-s3-reinit",
                 # Review R1: SUSPEND refuses a held completion (-8); one labelled
                 # gate-defect injection then sleeps, and only the post-wake latch
                 # keeps the held request's teardown reset off the device.
                 "power-s3-held-completion": "test-system-s3-held-completion",
                 # H05-T5c: then virtio-blk 00:05.0 re-initialised and self-tested
                 # (no capability issued), and four labelled injections that must
                 # quarantine lane 0 while the system continues to S5.
                 "power-s3-blk-reinit": "test-system-s3-blk-reinit",
                 "power-s3-blk-reinit-busy": "test-system-s3-blk-reinit-busy",
                 "power-s3-blk-reinit-identity": "test-system-s3-blk-reinit-identity",
                 "power-s3-blk-reinit-capacity": "test-system-s3-blk-reinit-capacity",
                 "power-s3-blk-reinit-no-ack": "test-system-s3-blk-reinit-no-reset-ack",
                 # H05-T5d A7 (labelled): a kernel entropy request keeps LOCK at -8,
                 # so SUSPEND refuses (-25) and init falls back to S5; no sleep.
                 "power-s3-lock-busy": "test-system-s3-lock-busy",
                 # B01-LOCK (ADR-KEYRING-LOCK-20261126): the labelled LOCK fixture
                 # (A2-A4, A6, A9, A10) while init's S5 grant waits, then S5.
                 "power-lock": "test-system-lock"}
POWER_CASES = tuple(POWER_FEATURE)
# B01-LOCK red controls: (kernel feature, the one expected kernel refusal, LOCKED
# markers). Without the stack scrub the canary survives LOCK (A10) and LOCKED
# never latches (F1, owner decision 26 November 2026); an own exit
# of the instance is never LOCKED (A5). H05-T5d: the S3 init's LOCK without the
# scrub request halts at the latch (KL4), before any SUSPEND.
LOCK_RED = {"power-lock-noscrub": ("test-system-lock-noscrub", "LOCK_CANARY_RESIDUE", 0),
            "power-lock-exit": ("test-system-lock-exit", "UNEXPECTED_EXIT", 0),
            "power-s3-noscrub": ("test-system-s3-lock-noscrub", "LOCK_STACK_RESIDUE", 0)}
S3_NOSCRUB = "power-s3-noscrub"
# Exact LOCK evidence: one LOCK, the fixture's six refusals (-1, -6, -4, -8, -8,
# -14), one verified whole-stack scrub, no canary after it; the minimums prove
# that the canary and the instance's frames were there to be erased.
LOCK_EXACT = {"LOCK_COUNT": 1, "LOCK_REFUSALS": 6, "LOCK_STACK_SCRUBS": 1, "LOCK_STACK_RESIDUE": 0,
              "LOCK_CANARY_AFTER": 0, "LOCK_APP_SEND": 10, "LOCK_APP_RECEIVE": 8}
LOCK_MINIMUM = {"LOCK_CANARY_BEFORE": 1, "LOCK_FRAMES_ZEROED": 1, "LOCK_GRANT_HOLDS": 1}
# B02-T4 (ADR-B02-KEY1-RELAY-20261123): the Key Service commits key state through
# init's commit ingress. Kept out of FEATURE (capture/gate iterate it as FR-I01).
RELAY_FEATURE = {"relay": "test-system-keystate-relay",
                 "relay-not-committed": "test-system-keystate-relay-not-committed",
                 "relay-timeout": "test-system-keystate-relay-timeout"}
# Red manifest case: the app also holds a commit SEND; the boot must stop.
RELAY_RED = {"relay-leak": "test-system-keystate-relay-leak"}
RELAY_CASES = (*RELAY_FEATURE, *RELAY_RED)
# H05-T5c: case -> (injection label, quarantine reason); None = validated re-init.
BLK_CASES = {"power-s3-blk-reinit": None, "power-s3-blk-reinit-busy": ("BUSY", "BROKER_BUSY"),
             "power-s3-blk-reinit-identity": ("IDENTITY", "REDISCOVER"),
             "power-s3-blk-reinit-capacity": ("CAPACITY", "CAPACITY"),
             "power-s3-blk-reinit-no-ack": ("NO_RESET_ACK", "RESET_ACK")}
S3_CASES = ("power-s3", "power-s3-reinit", "power-s3-held-completion", *BLK_CASES)
# H05-T5d: every S3 case LOCKs first (KL2); the A7 case never sleeps.
LOCK_BUSY = "power-s3-lock-busy"
S3_LOCK_CASES = (*S3_CASES, LOCK_BUSY)
LOCK_BUSY_ATTEMPTS = 8  # init's LOCK attempts, all refused (-8)
# B01-LOCK counters printed by every native-system-lock kernel. S3: init's one
# granted LOCK, one verified scrub, no fixture canary. A7: nothing ended.
S3_LOCK_EXACT = {"LOCK_COUNT": 1, "LOCK_REFUSALS": 0, "LOCK_STACK_SCRUBS": 1, "LOCK_STACK_RESIDUE": 0,
                 "LOCK_CANARY_BEFORE": 0, "LOCK_CANARY_AFTER": 0}
LOCK_BUSY_EXACT = {"LOCK_COUNT": 0, "LOCK_REFUSALS": LOCK_BUSY_ATTEMPTS, "LOCK_STACK_SCRUBS": 0,
                   "LOCK_STACK_RESIDUE": 0, "LOCK_CANARY_BEFORE": 0, "LOCK_CANARY_AFTER": 0,
                   "LOCK_ENTROPY_OBJECTS_BEFORE": 0, "LOCK_ENTROPY_OBJECTS_AFTER": 0}
LOCK_COUNTERS = ("LOCK_COUNT", "LOCK_REFUSALS", "LOCK_STACK_SCRUBS", "LOCK_STACK_RESIDUE", "LOCK_CANARY_BEFORE",
                 "LOCK_CANARY_AFTER", "LOCK_ENTROPY_OBJECTS_BEFORE", "LOCK_ENTROPY_OBJECTS_AFTER")
S3_LABEL_ERASED = "SYSTEM:S3:LABEL:instance erased; unwrap secret is a public TEST constant"
S3_LABEL_O5 = "SYSTEM:S3:LABEL:no key-state durability claimed (O5 open)"
S3_LOCK_BUSY_MARKER = "SYSTEM:S3:LOCK_BUSY:LOCK=-8:SUSPEND=-25:S5"
# Cases with the virtio-rng re-init (the blk re-init builds on it).
RNG_REINIT_CASES = ("power-s3-reinit", *BLK_CASES)
# The single re-init marker; CI's QEMU 8.2.2 may keep the BARs that 10.2.1 clears.
REINIT = re.compile(r"KEYRING:S3:REINIT:BARS=(KEPT|RESTORED):STATUS=15:SELFTEST=32")
BLK_REINIT = re.compile(r"VOLUME:S3:REINIT:BARS=(KEPT|RESTORED):STATUS=15:CAPACITY=[0-9]{20}:SELFTEST=READ")
QUARANTINED = 24  # volume::QUARANTINED, the post-wake refusal of a quarantined lane
S5_TAIL = ("POWER:S5:PM1A=0x0604:SLP_TYP=0:SCI_EN=1", "POWER:S5:WRITE")
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
    # Relay: init supplies the secret once (second supply and admin GENERATE
    # refused), never exports or restores; the Key Service's Ok now waits for
    # init's commit, so there is no wrong-secret restore (one tamper fewer).
    "relay": {"CRYPTO_SEAL": 3, "CRYPTO_UNSEAL": 2, "CRYPTO_TAMPER_DENIED": 9, "CRYPTO_RESTORES": 0,
              "KEYSTATE_EXPORTS": 0, "KEYSTATE_RESTORES": 0, "KEYSTATE_WRONG_SECRET_DENIED": 0,
              "KEYSTATE_FLOOR_DENIED": 0, "KEYSTATE_ADMIN_OP_DENIED": 1, "KEYSTATE_BYTES": 120,
              "KEYSTATE_GENERATION": 3, "ADMIN_REQUESTS": 3, "ADMIN_DENIED": 2,
              "RECORD_PERSISTED": 1, "RECORD_RESTORED_UNSEAL": 0, "RECORD_RESTORED_REVOKED_DENIED": 0,
              "RECORD_RESTORED_TAMPER_DENIED": 0, "RECORD_ADMIN_DENIED": 2, "RECORD_OBJECT_REVISION": 1},
}
RELAY = "KEYSTATE_RELAY_"
# Exact per-case relay counters (Key Service, init, app); all three cases end
# with the same three acknowledged commits (generation 3, revision 3).
_RELAY_COMMON = {"OK": 3, "LAST_GENERATION": 3, "SECRET_SUPPLIED": 1, "SECRET_REFUSED": 1,
                 "INIT_COMMITTED": 3, "INIT_BEGUN": 3, "INIT_REVISION": 3, "INIT_GENERATION": 3}
RELAY_EXACT = {case: {RELAY + key: value for key, value in {**_RELAY_COMMON, **values}.items()} for case, values in {
    "relay": {"CASE": 1, "REQUESTS": 3, "NOT_COMMITTED": 0, "UNKNOWN": 0, "INIT_RECEIVED": 3,
              "INIT_REFUSED": 0, "INIT_WITHHELD": 0, "APP_FIRST": 0, "APP_SECOND": 0},
    "relay-not-committed": {"CASE": 2, "REQUESTS": 5, "NOT_COMMITTED": 2, "UNKNOWN": 0, "INIT_RECEIVED": 5,
                            "INIT_REFUSED": 2, "INIT_WITHHELD": 0, "APP_FIRST": 1, "APP_SECOND": 1},
    "relay-timeout": {"CASE": 3, "REQUESTS": 4, "NOT_COMMITTED": 0, "UNKNOWN": 1, "INIT_RECEIVED": 4,
                      "INIT_REFUSED": 0, "INIT_WITHHELD": 1, "APP_FIRST": 7, "APP_SECOND": 7},
}.items()}
# wire::COMMIT_TICKS: an answered commit stays below it, a withheld one reaches it.
RELAY_COMMIT_TICKS = 1500
# 0 is legal for answered commits: on a fast (KVM) host each one can finish within one tick.
RELAY_TICKS = {"relay": range(0, RELAY_COMMIT_TICKS), "relay-not-committed": range(0, RELAY_COMMIT_TICKS),
               "relay-timeout": range(RELAY_COMMIT_TICKS, 2 * RELAY_COMMIT_TICKS)}


def record_plaintext() -> bytes:
    """Independent copy of the app's deterministic record plaintext."""
    return bytes((11 + i * 17 + i // 97) & 0xFF for i in range(128))
RANDOM_FEATURE = {"raw":"native-system-random", "sdk":"test-system-random-sdk", "unknown":"test-system-random-unknown", "wrong-caller":"test-system-random-wrong-caller"}
TCP_FEATURE = {"normal": "native-system-tcp", "recover": "test-system-tcp-recover"}
# H04-T3 link loss/reconnect profiles; driven by tools/run_net_link.py only.
LINK_FEATURES = ("test-system-net-link", "test-system-net-link-storm")
BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK")
COLD_CASES = ("keystate", "record-tag", "swapped-blocks", "withheld-secret")
# Relay disk: generation, chunks used, (domain 0, domain 1) clocks.
RELAY_VOLUME_STATE = (5, 5, [2, 3])
# A refusing cold boot stops before any broker run, case verdict or persistence;
# any of these in a cold log means the guest went further than a refusal.
COLD_FORBIDDEN = ("SYSTEM:CASE:", "M1:PASS", "SYSTEM:BROKERS:", "SYSTEM:CLEANUP:", "SYSTEM:RECORD_",
                  "SYSTEM:POWER:", "SYSTEM:KEYSTATE_")
# Old-generation replay is a documented open boundary, never a COLD_CASES pass.
COLD_ANTIROLLBACK = {"status": "UNPROTECTED", "replay": (
    "NOT_DETECTABLE: cold restore uses minimum_generation 0; older A/B slot (generation 2) has no "
    "key-state object; equal-generation stale image with same VOLUME_ID is accepted")}
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
        parent_feature = POWER_FEATURE[parent_case]
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
    if case not in FEATURE and case not in POWER_FEATURE and case not in RELAY_FEATURE:
        raise ValueError("unknown system case")
    return BOOT + ("SYSTEM:MANIFEST:PASS", "SYSTEM:BROKERS:PASS",
                   "SYSTEM:CLEANUP:PASS", f"SYSTEM:CASE:{case.upper().replace('-', '_')}:PASS", "M1:PASS")


def s3_machine() -> str:
    """power-s3's evidence boundary: the measured QEMU 8.2 q35/OVMF profile
    under single-thread TCG. An unset machine selects it; any other explicit
    machine or KVM is refused before anything is built or launched."""
    from run_sleep import MACHINE  # lazy, see run_system
    if os.environ.get("NOS_QEMU_MACHINE") not in (None, MACHINE) or qemu_accel() != "tcg,thread=single":
        raise ValueError(f"power-s3 runs only on {MACHINE} under single-thread TCG")
    return MACHINE


def check_suspend(lines: list[str], reinit: bool = False, blk: str | None = None) -> None:
    """One resolved plan at bootstrap, then exactly one ENTER/RESUMED pair
    inside init's run: after the manifest, before the broker evidence. With
    re-init, exactly one re-init marker between RESUMED and the brokers; with
    the blk re-init (H05-T5c) then its one success marker, or its one injection
    label followed by its one quarantine marker."""
    from run_sleep import ENTER, PLAN, RESUMED  # lazy, see run_system
    s3 = [line for line in lines if line.startswith("POWER:S3:")]
    if (len(s3) != 3 or not PLAN.fullmatch(s3[0]) or s3[1] != ENTER.format(1)
            or not RESUMED.fullmatch(s3[2]) or RESUMED.fullmatch(s3[2]).group(1) != "1"):
        raise ValueError(f"power-s3 lacks exactly one planned, entered and resumed cycle: {s3}")
    order = ["SYSTEM:MANIFEST:PASS", s3[1], s3[2], "SYSTEM:BROKERS:PASS"]
    if reinit:
        marker = [line for line in lines if line.startswith("KEYRING:S3:")]
        if len(marker) != 1 or not REINIT.fullmatch(marker[0]):
            raise ValueError(f"power-s3-reinit lacks exactly one re-init marker: {marker}")
        order.insert(3, marker[0])
    if blk is not None:
        volume = [line for line in lines if line.startswith("VOLUME:S3:")]
        injected = [line for line in lines if line.startswith("SYSTEM:FAULT_INJECTED:S3_BLK_")]
        if BLK_CASES[blk] is None:
            if len(volume) != 1 or not BLK_REINIT.fullmatch(volume[0]) or injected:
                raise ValueError(f"{blk} lacks exactly one blk re-init marker: {volume} {injected}")
        else:
            label, reason = BLK_CASES[blk]
            if (volume != [f"VOLUME:S3:REINIT:QUARANTINE:{reason}"]
                    or injected != [f"SYSTEM:FAULT_INJECTED:S3_BLK_{label}"]):
                raise ValueError(f"{blk} lacks exactly one injection and quarantine marker: {volume} {injected}")
            order.insert(4, injected[0])
        order.insert(len(order) - 1, volume[0])
    positions = [lines.index(marker) for marker in order]
    if positions != sorted(positions) or lines.index(s3[0]) > positions[0]:
        raise ValueError("power-s3 S3 markers are out of order")


def refuse_stray_suspend(case: str, stdout: str) -> None:
    """Only the S3 cases carry SYSTEM:SUSPEND* evidence, only re-init KEYRING:S3:."""
    lines = stdout.splitlines()
    if case not in S3_LOCK_CASES and any(line.startswith(("SYSTEM:SUSPEND", "SYSTEM:S3:")) for line in lines):
        raise ValueError("suspend evidence outside the power-s3 profile")
    if case not in RNG_REINIT_CASES and any(line.startswith("KEYRING:S3:") for line in lines):
        raise ValueError("re-init evidence outside the power-s3-reinit profile")
    if case not in BLK_CASES and any(line.startswith(("VOLUME:S3:", "SYSTEM:FAULT_INJECTED:S3_BLK_",
                                                      "SYSTEM:SUSPEND_BLK_")) for line in lines):
        raise ValueError("blk re-init evidence outside the power-s3-blk-reinit profiles")


# Review R3: after CLEANUP, each broker's device accesses at the wake (live
# counters, > 0), the post-wake accesses outside the accounted re-init path
# and the teardown resets the latch suppressed.
LIVE_AT_WAKE = ("DEVICE_IO_AT_WAKE_KEYRING", "DEVICE_IO_AT_WAKE_VOLUME")
DEVICE_ORDER = (*LIVE_AT_WAKE, "DEVICE_IO_AFTER_WAKE", "DEVICE_SUPPRESSED")
SUSPEND_ORDER = ("REFUSALS", "GRANTED", "TICK_AFTER_WAKE", "HELD_REFUSALS", "LOST_ENTROPY",
                 "LOST_BLOCK", *DEVICE_ORDER)
# Re-init: one 32-byte post-wake delivery, the pre-sleep ticket refused (-1),
# init's POWER handle refused by authority (-6), block still lost (-14).
REINIT_ORDER = ("REFUSALS", "GRANTED", "TICK_AFTER_WAKE", "HELD_REFUSALS", "REINIT_ENTROPY",
                "REINIT_STALE_TICKET", "REINIT_ENTROPY_AUTH", "LOST_BLOCK", *DEVICE_ORDER)
# H05-T5c: the blk outcome, the pre-sleep ticket refused (-1), the 8259 masks and
# INTx line seen after the wake, then block at authority (-6) or quarantined (-24);
# last the volume re-init's accounted device accesses.
# H05-T5d A7: SUSPEND refusals, no grant, the LOCK refusals, kernel requests.
LOCK_BUSY_ORDER = ("REFUSALS", "GRANTED", "LOCK_BUSY_REFUSALS", "LOCK_BUSY_REQUESTS", "LOCK_BUSY_DELIVERED")
BLK_ORDER = (*REINIT_ORDER[:7], "BLK_QUARANTINED", "BLK_STALE_TICKET", "BLK_PIC_MASTER", "BLK_PIC_SLAVE",
             "BLK_INTX_LINE", "BLK_LATCH_CARRIED", "LOST_BLOCK", *DEVICE_ORDER, "DEVICE_IO_ACCOUNTED_VOLUME")


def check_s3_lock(case: str, lines: list[str], found: dict) -> None:
    """H05-T5d: S3 cases LOCK once before the sleep (LOCKED between MANIFEST and
    POWER:S3:ENTER) and print both labels before the gate; A7 never LOCKs and
    prints only the O5 label and its fallback marker."""
    if lines.count("SYSTEM:LOCK:PASS") != 0:
        raise ValueError(f"{case} carries the power-lock fixture marker")
    missing = [key for key in LOCK_COUNTERS if key not in found]
    if missing:
        raise ValueError(f"{case} lacks LOCK counters: {missing}")
    gate = lines.index("SYSTEM:POWER:GATE:PASS") if "SYSTEM:POWER:GATE:PASS" in lines else -1
    cleanup = lines.index("SYSTEM:CLEANUP:PASS") if "SYSTEM:CLEANUP:PASS" in lines else len(lines)
    if case == LOCK_BUSY:
        exact, labels = LOCK_BUSY_EXACT, (S3_LOCK_BUSY_MARKER, S3_LABEL_O5)
        if "SYSTEM:LOCK:LOCKED" in lines or S3_LABEL_ERASED in lines:
            raise ValueError("power-s3-lock-busy was LOCKED or claims an erased instance")
    else:
        exact, labels = S3_LOCK_EXACT, (S3_LABEL_ERASED, S3_LABEL_O5)
        if lines.count("SYSTEM:LOCK:LOCKED") != 1 or "POWER:S3:ENTER:CYCLE=1:SCI_EN=1" not in lines:
            raise ValueError(f"{case} needs exactly one LOCKED marker before its one sleep")
        if not (lines.index("SYSTEM:MANIFEST:PASS") < lines.index("SYSTEM:LOCK:LOCKED")
                < lines.index("POWER:S3:ENTER:CYCLE=1:SCI_EN=1")):
            raise ValueError(f"{case} LOCKED marker is not between MANIFEST and the sleep")
        if not found["LOCK_ENTROPY_OBJECTS_AFTER"] < found["LOCK_ENTROPY_OBJECTS_BEFORE"]:
            raise ValueError("LOCK did not reclaim the instance's entropy object")
    for key, value in exact.items():
        if found.get(key) != value:
            raise ValueError(f"{key}: expected {value}, found {found.get(key)}")
    for label in labels:
        if lines.count(label) != 1 or not cleanup < lines.index(label) < gate:
            raise ValueError(f"{case} lacks the label between CLEANUP and the gate: {label}")
    if [line for line in lines if line.startswith("SYSTEM:S3:")] != list(labels):
        raise ValueError(f"{case} S3 label set differs")


def check_lock(case: str, stdout: str, found: dict) -> None:
    """B01-LOCK: one LOCKED marker before finish, exact refusals, canary scrubbed."""
    lines = stdout.splitlines()
    if case in S3_LOCK_CASES:
        check_s3_lock(case, lines, found)
        return
    if case != "power-lock":
        if any(line.startswith("SYSTEM:LOCK") for line in lines):
            raise ValueError("LOCK evidence outside the power-lock profile")
        return
    if lines.count("SYSTEM:LOCK:LOCKED") != 1 or lines.count("SYSTEM:LOCK:PASS") != 1:
        raise ValueError("power-lock needs exactly one LOCKED and one LOCK:PASS marker")
    if not (lines.index("SYSTEM:MANIFEST:PASS") < lines.index("SYSTEM:LOCK:LOCKED")
            < lines.index("SYSTEM:BROKERS:PASS") < lines.index("SYSTEM:LOCK:PASS")
            < lines.index("SYSTEM:CLEANUP:PASS")):
        raise ValueError("power-lock markers out of order")
    for key, value in LOCK_EXACT.items():
        if found.get(key) != value:
            raise ValueError(f"{key}: expected {value}, found {found.get(key)}")
    for key, value in LOCK_MINIMUM.items():
        if found.get(key, -1) < value:
            raise ValueError(f"{key}: expected at least {value}, found {found.get(key)}")
    if not found.get("LOCK_ENTROPY_OBJECTS_AFTER", 1 << 64) < found.get("LOCK_ENTROPY_OBJECTS_BEFORE", 0):
        raise ValueError("LOCK did not reclaim the instance's entropy object")


def check_power(case: str, stdout: str, found: dict) -> None:
    """Exact S5 tail after M1:PASS, or a refusal without any PM1 write."""
    lines = stdout.splitlines()
    if case in ("power", "power-lock", *S3_LOCK_CASES):
        if tuple(lines[-3:]) != ("M1:PASS", *S5_TAIL) or "SYSTEM:POWER:GATE:PASS" not in lines:
            raise ValueError("power case did not end with the gated S5 write")
        expected = {"POWER_INIT_REFUSALS": 3, "POWER_APP_REFUSALS": 2, "POWER_NOT_DURABLE": 0}
        if case == LOCK_BUSY:
            # A7: the plan only, never a sleep; SUSPEND -25, -6, -4, -1, -25 (five
            # refusals), every LOCK attempt refused, every kernel request delivered.
            from run_sleep import PLAN  # lazy, see run_system
            s3 = [line for line in lines if line.startswith(("POWER:S3:", "KEYRING:S3:", "VOLUME:S3:"))]
            if len(s3) != 1 or not PLAN.fullmatch(s3[0]):
                raise ValueError(f"power-s3-lock-busy must plan S3 and never enter it: {s3}")
            expected.update(SUSPEND_REFUSALS=5, SUSPEND_GRANTED=0,
                            SUSPEND_LOCK_BUSY_REFUSALS=LOCK_BUSY_ATTEMPTS)
            suspend = [line.split(":")[1] for line in lines if line.startswith("SYSTEM:SUSPEND")]
            if suspend != [f"SUSPEND_{key}" for key in LOCK_BUSY_ORDER]:
                raise ValueError(f"{case} suspend evidence out of order: {suspend}")
            requests = found.get("SUSPEND_LOCK_BUSY_REQUESTS", 0)
            if not 1 <= requests <= LOCK_BUSY_ATTEMPTS or found.get("SUSPEND_LOCK_BUSY_DELIVERED") != requests:
                raise ValueError("power-s3-lock-busy kernel requests were not all delivered")
            at = {line.split(":")[1]: index for index, line in enumerate(lines)
                  if line.startswith("SYSTEM:SUSPEND_")}
            if not at[f"SUSPEND_{LOCK_BUSY_ORDER[-1]}"] < lines.index("SYSTEM:CLEANUP:PASS"):
                raise ValueError(f"{case} suspend evidence out of order around CLEANUP")
        elif case in S3_CASES:
            reinit = case in RNG_REINIT_CASES
            blk = case if case in BLK_CASES else None
            held = int(case == "power-s3-held-completion")
            check_suspend(lines, reinit, blk)
            # -8 before SHUTDOWN, -6, -4, -1, [the held-completion -8,] then the
            # grant and the one-shot -14; after the wake init's entropy and block
            # requests each return -14. No broker device access after the wake
            # outside the accounted re-init; the latch suppressed exactly the held
            # request's teardown reset.
            expected.update(SUSPEND_REFUSALS=5 + held, SUSPEND_GRANTED=1, SUSPEND_TICK_AFTER_WAKE=1,
                            SUSPEND_HELD_REFUSALS=held, SUSPEND_LOST_BLOCK=14,
                            SUSPEND_DEVICE_IO_AFTER_WAKE=0, SUSPEND_DEVICE_SUPPRESSED=held)
            if reinit:
                expected.update(SUSPEND_REINIT_ENTROPY=32, SUSPEND_REINIT_STALE_TICKET=1,
                                SUSPEND_REINIT_ENTROPY_AUTH=6)
            else:
                expected.update(SUSPEND_LOST_ENTROPY=14)
            if blk is not None:
                quarantined = BLK_CASES[blk] is not None
                # The PIC masks and the line are recorded only on the validated path.
                expected.update(SUSPEND_BLK_QUARANTINED=int(quarantined),
                                SUSPEND_BLK_STALE_TICKET=0 if quarantined else 1,
                                SUSPEND_BLK_PIC_MASTER=0 if quarantined else 0xfe,
                                SUSPEND_BLK_PIC_SLAVE=0 if quarantined else 0xff,
                                SUSPEND_BLK_LATCH_CARRIED=1,
                                SUSPEND_LOST_BLOCK=QUARANTINED if quarantined else 6)
                if quarantined:
                    expected.update(SUSPEND_BLK_INTX_LINE=0)
                busy = blk == "power-s3-blk-reinit-busy"
                if (found.get("SUSPEND_DEVICE_IO_ACCOUNTED_VOLUME", -1) == 0) != busy:
                    raise ValueError("SUSPEND_DEVICE_IO_ACCOUNTED_VOLUME: the volume re-init must be "
                                     "accounted, and the busy refusal must touch no device")
            order = BLK_ORDER if blk else REINIT_ORDER if reinit else SUSPEND_ORDER
            suspend = [line.split(":")[1] for line in lines if line.startswith("SYSTEM:SUSPEND")]
            if suspend != [f"SUSPEND_{key}" for key in order]:
                raise ValueError(f"{case} suspend evidence out of order: {suspend}")
            # The kernel's real order: the wake evidence (ending in LOST_BLOCK)
            # before CLEANUP, the device evidence after every owner's teardown
            # and before the case marker.
            at = {line.split(":")[1]: index for index, line in enumerate(lines)
                  if line.startswith("SYSTEM:SUSPEND_")}
            if not (at["SUSPEND_LOST_BLOCK"] < lines.index("SYSTEM:CLEANUP:PASS")
                    < at[f"SUSPEND_{DEVICE_ORDER[0]}"]
                    and at[f"SUSPEND_{DEVICE_ORDER[-1]}"] < lines.index(expected_markers(case)[-2])):
                raise ValueError(f"{case} suspend evidence out of order around CLEANUP")
            for key in LIVE_AT_WAKE:
                if found.get(f"SUSPEND_{key}", 0) < 1:
                    raise ValueError(f"SUSPEND_{key}: expected a live counter (> 0), found {found.get(f'SUSPEND_{key}')}")
    else:
        fault = "FLUSH_ERROR" if case == "power-flush-error" else "FLUSH_TIMEOUT"
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
    # The kernel fixture's 32 post-wake bytes cannot pad the pre-sleep bound.
    if case in RNG_REINIT_CASES and found["ENTROPY_BYTES"] - found["SUSPEND_REINIT_ENTROPY"] < 256:
        raise ValueError("ENTROPY_BYTES: fewer than 256 bytes delivered before the sleep")


def evaluate(case: str, stdout: str, raw_exit: int | None, timed_out: bool) -> dict[str, int]:
    expected = expected_markers(case)
    controlled = [line for line in stdout.splitlines()
                  if line in BOOT or line == "M1:PASS" or
                  line.startswith(("SYSTEM:MANIFEST:", "SYSTEM:BROKERS:",
                                   "SYSTEM:CLEANUP:", "SYSTEM:CASE:"))]
    if controlled != list(expected):
        raise ValueError(f"system marker sequence mismatch: {controlled}")
    # Only a guest poweroff exits 0: the power runs override -no-reboot.
    if timed_out or raw_exit != (0 if case in ("power", "power-lock", *S3_LOCK_CASES) else 33):
        raise ValueError(f"raw exit {raw_exit}, timed_out {timed_out}")
    if any("PANIC" in line or "FAIL" in line for line in stdout.splitlines()):
        raise ValueError("guest failure marker")
    refuse_stray_suspend(case, stdout)
    relay = case in RELAY_FEATURE
    if not relay and any(line.startswith(f"SYSTEM:{RELAY}") for line in stdout.splitlines()):
        raise ValueError("key-state relay evidence outside the relay profile")
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
             **PERSIST_EXACT["recover" if case == "recover" else "relay" if relay else "normal"],
             **RELAY_EXACT.get(case, {}),
             # B01-LOCK: the canary request is one more refused admin request.
             **({"ADMIN_REQUESTS": 6, "ADMIN_DENIED": 4} if case == "power-lock" else {})}
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
    check_lock(case, stdout, found)
    if relay:
        relay_keys = {key for key in found if key.startswith(RELAY)}
        if relay_keys != {*RELAY_EXACT[case], RELAY + "MAX_COMMIT_TICKS", RELAY + "SUM"}:
            raise ValueError(f"relay evidence set differs: {sorted(relay_keys)}")
        if found[RELAY + "MAX_COMMIT_TICKS"] not in RELAY_TICKS[case]:
            raise ValueError(f"relay commit latency {found[RELAY + 'MAX_COMMIT_TICKS']} outside {RELAY_TICKS[case]}")
        if found[RELAY + "SUM"] == 0:
            raise ValueError("relay committed-blob checksum missing")
    expected_sum = volume_disk.fnv(volume_disk.fixture(0, 1, 4096))
    if found.get("APP_CONTENT_SUM") != expected_sum:
        raise ValueError("guest read content differs from the independent fixture")
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


def check_disk_relay(oracle: dict, checksum: int) -> None:
    """Disk oracle shared by all three relay cases: exactly the three
    acknowledged key-state generations (revisions 3, 2, 1 retained with
    HISTORY = 2), the current one matching init's reported FNV sum, and
    nothing written for a NOT_COMMITTED or withheld commit."""
    objects = oracle["objects"]
    _single_version(objects[0], "system document",
                    {"revision": 1, "length": 4096, "name": "systeemproef", "content_revision": 1})
    sealed = _opaque_bytes(_single_version(objects[SEALED_OBJECT], "sealed record",
                                           {"revision": 1, "length": SEALED_LENGTH, "name": SEALED_NAME}),
                           SEALED_LENGTH, "sealed record")
    plaintext = record_plaintext()
    if plaintext in sealed or sealed[24:24 + len(plaintext)] == plaintext or not any(sealed[:24]):
        raise ValueError("sealed record stores recognizable plaintext or an empty nonce")
    item = objects[KEYSTATE_OBJECT]
    if not item["present"] or item["revision"] != 3 or (item["collection"], item["tags"], item["kind"]) != (0, 0, 0):
        raise ValueError("relay key-state object is not at revision 3")
    versions = item["versions"]
    if [version.get("revision") for version in versions] != [3, 2, 1]:
        raise ValueError(f"relay key-state versions differ: {[v.get('revision') for v in versions]}")
    blobs = []
    for version in versions:
        if version.get("length") != KEYSTATE_LENGTH or version.get("name") != KEYSTATE_NAME:
            raise ValueError("relay key-state version length/name differ")
        blob = _opaque_bytes(version, KEYSTATE_LENGTH, "key-state snapshot")
        if blob[:len(KEYSTATE_HEADER)] != KEYSTATE_HEADER:
            raise ValueError("key-state snapshot header/volume binding is wrong")
        blobs.append(blob)
    nonces = [blob[32:56] for blob in blobs]
    if not all(any(nonce) for nonce in nonces) or len(set(nonces)) != len(nonces):
        raise ValueError("key-state snapshot nonces are empty or repeated")
    if volume_disk.fnv(blobs[0]) != checksum:
        raise ValueError("current key-state snapshot differs from init's acknowledged commit")
    untouched = [obj for index, obj in enumerate(objects) if index not in (0, SEALED_OBJECT, KEYSTATE_OBJECT)]
    if any(obj["present"] or obj["revision"] for obj in untouched):
        raise ValueError("guest touched another object or domain")
    if any(collection["present"] or collection["revision"] for collection in oracle["collections"]):
        raise ValueError("guest changed an unrelated collection")
    # Five commits: three key-state generations (domain 1), document and record (domain 0).
    if (oracle["generation"], oracle["chunks_used"], oracle["clocks"]) != RELAY_VOLUME_STATE:
        raise ValueError("unexpected relay volume generation, allocation or domain clock")


def evaluate_red(case: str, stdout: str, raw_exit: int | None, timed_out: bool,
                 before: str, after: str) -> dict:
    """relay-leak: the manifest check must stop the boot before anything runs."""
    if case not in RELAY_RED:
        raise ValueError("unknown red relay case")
    lines = stdout.splitlines()
    fails = [line for line in lines if line.startswith("SYSTEM:FAIL:")]
    if fails != ["SYSTEM:FAIL:APP_COMMIT_HANDLE"]:
        raise ValueError(f"red relay case stopped for another reason: {fails}")
    if any(line.startswith(f"SYSTEM:{RELAY}") or (line.startswith("SYSTEM:") and line.endswith(":PASS"))
           or line == "M1:PASS" for line in lines):
        raise ValueError("red relay case passed a system stage or produced relay evidence")
    if [line for line in lines if line in BOOT] != list(BOOT):
        raise ValueError("red relay case lacks the boot markers")
    if timed_out or raw_exit != 35:
        raise ValueError(f"red relay case raw exit {raw_exit}, timed_out {timed_out}")
    if before != after:
        raise ValueError("red relay case changed the disk")
    return {"fail": fails[0], "raw_exit": raw_exit}


def evaluate_lock_red(case: str, stdout: str, raw_exit: int | None, timed_out: bool) -> dict:
    """B01-LOCK red controls: exactly the expected kernel refusal, never LOCKED, no S5."""
    lines = stdout.splitlines()
    fails = [line for line in lines if line.startswith("SYSTEM:FAIL:")]
    if fails != [f"SYSTEM:FAIL:{LOCK_RED[case][1]}"]:
        raise ValueError(f"red lock case stopped for another reason: {fails}")
    if lines.count("SYSTEM:LOCK:LOCKED") != LOCK_RED[case][2]:
        raise ValueError("red lock case has the wrong number of LOCKED markers")
    if case == S3_NOSCRUB:
        # The S3 kernel resolves its plan once at bootstrap; it is the only S3
        # line allowed, and the case stops before any SUSPEND (no sleep, no wake).
        from run_sleep import PLAN  # lazy, see run_system
        s3 = [line for line in lines if line.startswith(("POWER:S3:", "KEYRING:S3:", "VOLUME:S3:"))]
        if len(s3) != 1 or not PLAN.fullmatch(s3[0]):
            raise ValueError(f"power-s3-noscrub must plan S3 exactly once: {s3}")
        lines = [line for line in lines if line != s3[0]]
    if any(line.startswith(("SYSTEM:LOCK_", "SYSTEM:LOCK:PASS", "POWER:", "SYSTEM:SUSPEND", "SYSTEM:S3:"))
           or line == "M1:PASS" for line in lines):
        raise ValueError("red lock case produced LOCK evidence, a PM1 write or M1:PASS")
    if [line for line in lines if line in BOOT] != list(BOOT):
        raise ValueError("red lock case lacks the boot markers")
    if timed_out or raw_exit != 35:
        raise ValueError(f"red lock case raw exit {raw_exit}, timed_out {timed_out}")
    return {"fail": fails[0], "raw_exit": raw_exit}


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
    refuse_stray_suspend(case, stdout)
    lines = stdout.splitlines()
    marker = f"SYSTEM:COLD:REFUSED:{case}"
    controlled = [line for line in lines if line.startswith("SYSTEM:COLD:")]
    if controlled != [marker] or raw_exit != 33 or timed_out:
        raise ValueError("cold refusal case/exit mismatch")
    if any(token in line for line in lines for token in COLD_FORBIDDEN):
        raise ValueError("cold log carries a success, case or persistence marker")
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


def check_cold_binding(cold_case: str, env_value: str | None, parent_before: str | None,
                       parent_after: str | None) -> None:
    """Bind the runner's fixture environment and the unchanged parent normal image to this run.

    env_value is the runner's NOS_SYSTEM_COLD_CASE, not proof of what the kernel was compiled
    with (option_env! is build-time); that binding is the exact SYSTEM:COLD:REFUSED:<case> marker.
    """
    if cold_case not in COLD_CASES or env_value != cold_case:
        raise ValueError("NOS_SYSTEM_COLD_CASE does not match --cold-case")
    if not parent_before or parent_before != parent_after:
        raise ValueError("cold run changed or lost the parent normal image")


def run_system(case: str, volume_dir: Path | None = None, *, timeout: int = MAX_TIMEOUT,
               kernel_feature: str | None = None, network_port: int | None = None,
               rtc_base: str | None = None, cold_case: str | None = None,
               nic_status: bool = True, qmp_hook=None):
    cases = {**FEATURE, **POWER_FEATURE, **RELAY_FEATURE, **RELAY_RED,
             **{name: feature for name, (feature, _, _) in LOCK_RED.items()}}
    if case not in cases or not 0 < timeout <= MAX_TIMEOUT:
        raise ValueError("unknown case or unbounded timeout")
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
    wake, machine = None, None
    if case in (*S3_LOCK_CASES, S3_NOSCRUB):
        # The S3 evidence boundary: the measured QEMU 8.2 q35/OVMF profile under TCG only.
        machine = s3_machine()
    if case in S3_CASES:
        from run_sleep import SystemWake  # lazy: run_sleep's wake hook reuses run_net_link, which imports this module
        wake = SystemWake(time.monotonic() + timeout)
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
        parent_before = sha256(parent_dir / "volume.img")
        volume_dir = run_dir / "cold-image"
        volume_dir.mkdir()
        (volume_dir / "volume.img").write_bytes(cold_image((parent_dir / "volume.img").read_bytes(), cold_case))
    image = volume_dir / "volume.img"
    check_image(image)
    before = sha256(image)
    rel = run_dir.relative_to(ROOT).as_posix()
    disk_rel = image.relative_to(ROOT).as_posix()
    readonly = "on" if case == "recover" else "off"
    command = [qemu, "-machine", machine or qemu_machine(), "-accel", qemu_accel(), "-cpu", "qemu64",
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
    if wake is not None:
        qmp_hook = wake
    if qmp_hook is not None:
        # Private QMP socket inside this run directory (relative to QEMU's
        # cwd). The link harness is its only client and permits nothing but
        # qmp_capabilities and set_link; no HMP, TAP or other listener.
        # AF_UNIX paths stop at 107 bytes; case names stay short enough for
        # build/power-system-gate (capture_system pins the 32-hex suffix).
        if len(f"{rel}/qmp.sock".encode()) >= 108:
            raise ValueError("Private QMP socket path too long; choose a shorter NOS_BUILD_DIR")
        command += ["-qmp", f"unix:{rel}/qmp.sock,server=on,wait=off"]
    started = datetime.now(timezone.utc).isoformat()
    begin = time.monotonic()
    on_line = None if qmp_hook is None else (lambda line, process: qmp_hook(line, process, run_dir))
    try:
        rc, stdout, stderr, timed_out, _, _ = execute(command, env, timeout=timeout, on_line=on_line)
    except Exception as error:
        # A failed link-control hook killed QEMU: keep the partial serial log.
        (run_dir / "serial.txt").write_text(getattr(error, "partial_stdout", ""))
        if wake is not None:
            (run_dir / "stderr.txt").write_text(getattr(error, "partial_stderr", ""))
            save_json(run_dir / "result.json", dict(case=case, passed=False, s3_control=wake.evidence(),
                                                    run_directory=str(run_dir),
                                                    error=f"{type(error).__name__}: {error}"))
            raise RuntimeError(f"{case} control failed ({run_dir}): {error}") from error
        raise
    finally:
        if wake is not None:
            wake.close()
    duration = time.monotonic() - begin
    (run_dir / "serial.txt").write_text(stdout)
    (run_dir / "stderr.txt").write_text(stderr)
    after = sha256(image)
    parent_after = sha256(parent_dir / "volume.img") if cold_case is not None else None
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
    if wake is not None:
        # Recorded before any verdict so a failed run keeps its QMP evidence.
        report["s3_control"] = wake.evidence()
    if use_network:
        report["network_gate"] = False
    if use_tcp:
        report["tcp_gate"] = False
    if link:
        report["nic_status"] = nic_status
        report["qmp_socket"] = "qmp.sock" if qmp_hook is not None else None
    try:
        report["cold_case"] = cold_case
        report["fixture_environment"] = {"NOS_SYSTEM_COLD_CASE": os.environ.get("NOS_SYSTEM_COLD_CASE")}
        report["cold_antirollback"] = dict(COLD_ANTIROLLBACK)
        if cold_case is not None:
            report["parent_before_sha256"], report["parent_after_sha256"] = parent_before, parent_after
            check_cold_binding(cold_case, os.environ.get("NOS_SYSTEM_COLD_CASE"), parent_before, parent_after)
        if case in RELAY_RED:
            report["evidence"] = evaluate_red(case, stdout, rc, timed_out, before, after)
            report["expected_fail"] = True
        elif case in LOCK_RED:
            report["evidence"] = evaluate_lock_red(case, stdout, rc, timed_out)
            report["expected_fail"] = True
        else:
            report["evidence"] = (evaluate_cold(cold_case, stdout, rc, timed_out) if cold_case is not None
                                  else evaluate(case, stdout, rc, timed_out))
        if wake is not None:
            wake.check()
        if case == "recover" and before != after:
            raise ValueError("read-only cold recovery changed the disk")
        oracle = volume_disk.analyse(image.read_bytes(), OPAQUE_OBJECTS)
        if case in RELAY_FEATURE:
            check_disk_relay(oracle, report["evidence"][RELAY + "SUM"])
        elif case in RELAY_RED:
            pass  # unchanged empty image, compared byte-for-byte above
        elif case in LOCK_RED:
            pass  # stopped mid-flow by design; its disk is not acceptance evidence
        elif cold_case is None:
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
    parser.add_argument("--case", choices=(*FEATURE, *POWER_FEATURE, *RELAY_CASES, *LOCK_RED), required=True)
    parser.add_argument("--volume", type=Path)
    parser.add_argument("--cold-case", choices=COLD_CASES)
    parser.add_argument("--timeout", type=int, default=MAX_TIMEOUT)
    options = parser.parse_args()
    raise SystemExit(0 if run_system(options.case, options.volume, timeout=options.timeout, cold_case=options.cold_case)[0]["passed"] else 1)
