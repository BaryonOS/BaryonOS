#!/usr/bin/env python3
"""H03-T3a QEMU runner: kernel-only virtio-snd PCM output fixture.

One case per disposable guest. The kernel (native-virtio-snd-probe or a test-snd-*
variant) plays a fixed stereo S16 48 kHz pattern through QEMU 8.2
`virtio-sound-pci` at 00:08.0 into a WAV audiodev. `-icount shift=0,sleep=off`
puts the guest TSC, the PIT and QEMU's audio timer on one virtual clock.

Oracles independent of the kernel's own report:
- the exact serial line sequence after the single EBS handoff and raw exit 33;
- the WAV container (RIFF/WAVE, exactly one PCM `fmt ` chunk with 2 channels,
  48000 Hz, 16 bit, then `data`) must be finalised: QEMU writes the sizes at
  exit, so a killed or timed-out guest leaves a header that fails here;
- the PCM payload is compared byte for byte with the frozen pattern
  (`TRANSPORT.md` §7): at most 4800 silent frames before and after, no other
  tolerance. Fence/re-init: an exact prefix of pattern A no longer than what
  the guest submitted, at most 4800 silent frames, all of pattern B, never A
  after B. Quarantine: only a prefix of A.

Scope: QEMU/TCG emulator output only. A TX completion means QEMU accepted the
samples, the WAV shows what its audio backend wrote; nothing here measures
audible or physical playback, latency or DMA isolation.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import re
import shutil
import struct
import subprocess
import time
import uuid

from common import BUILD, ROOT, environment, firmware, qemu_machine, save_json, sha256, source_hash

FEATURE = {
    "normal": "native-virtio-snd-probe",
    "no-device": "native-virtio-snd-probe",
    "access-platform": "native-virtio-snd-probe",
    "fence-reinit": "test-snd-fence-reinit",
    "quarantine": "test-snd-quarantine",
}
MAX_TIMEOUT = 240
MAX_OUTPUT = 4 * 1024 * 1024
MAX_WAV = 16 * 1024 * 1024
QEMU_SERIES = "QEMU emulator version 8.2."
FIXTURE_FRAMES = 48_000
PERIOD_FRAMES = 480
FENCE_AFTER_PERIODS = 10
SILENCE_WINDOW = 4_800
FRAME = 4
VERSION_1 = 1 << 32
ACCESS_PLATFORM = 1 << 33

BOOT = ("BOOT:EBS_OK", "KERNEL:ENTRY", "BOOTINFO:OK", "KS-01:PASS", "KS-02:PASS",
        "ACPI:UNAVAILABLE:NO_DMAR", "DEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT")
END = ("KS-03:PASS", "M2:PASS", "M1:PASS")
FAILURES = ("PANIC", "FAIL")


def audiodev(rel: str) -> list[str]:
    return ["-audiodev",
            f"wav,id=snd0,path={rel}/out.wav,out.frequency=48000,out.channels=2,out.format=s16"]


def device(case: str) -> list[str]:
    if case not in FEATURE:
        raise ValueError("unknown virtio-snd case")
    if case == "no-device":
        return []
    option = "virtio-sound-pci,addr=08.0,audiodev=snd0,streams=1"
    if case == "access-platform":
        option += ",iommu_platform=on"
    return ["-device", option]


def qemu_arguments(case: str, qemu: str, rel: str) -> list[str]:
    """Self-built command (as run_keyring.py); tools/run.py stays unchanged."""
    return [qemu, "-machine", qemu_machine(), "-accel", "tcg,thread=single",
            "-icount", "shift=0,sleep=off", "-cpu", "qemu64", "-smp", "1", "-m", "256M",
            "-nodefaults", "-display", "none", "-monitor", "none", "-serial", "stdio",
            "-nic", "none", "-no-reboot",
            "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={rel}/OVMF_CODE.fd",
            "-drive", f"if=pflash,unit=1,format=raw,file={rel}/OVMF_VARS.fd",
            "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{rel}/esp",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
            *audiodev(rel), *device(case)]


def frame_a(n: int) -> bytes:
    x = n & 0xFFFF
    return struct.pack("<HH", x, x ^ 0x8000)


def frame_b(n: int) -> bytes:
    x = ~n & 0xFFFF
    return struct.pack("<HH", x, x ^ 0x2000)


PATTERN_A = b"".join(frame_a(n) for n in range(FIXTURE_FRAMES))
PATTERN_B = b"".join(frame_b(n) for n in range(FIXTURE_FRAMES))
SILENT = b"\0" * FRAME


def parse_wav(data: bytes) -> bytes:
    """Strict RIFF/WAVE container; returns the PCM payload of `data`."""
    if len(data) < 12 or data[:4] != b"RIFF" or data[8:12] != b"WAVE":
        raise ValueError("not a RIFF/WAVE file")
    if struct.unpack_from("<I", data, 4)[0] != len(data) - 8:
        raise ValueError("RIFF size does not match the file: header not finalised or truncated")
    position, fmt, payload = 12, None, None
    while position < len(data):
        if position + 8 > len(data):
            raise ValueError("truncated chunk header")
        name = data[position:position + 4]
        size = struct.unpack_from("<I", data, position + 4)[0]
        body = data[position + 8:position + 8 + size]
        if len(body) != size:
            raise ValueError(f"chunk {name!r} exceeds the file")
        if name == b"fmt ":
            if fmt is not None or payload is not None:
                raise ValueError("unexpected or repeated fmt chunk")
            if size != 16 or struct.unpack("<HHIIHH", body) != (1, 2, 48_000, 192_000, 4, 16):
                raise ValueError("fmt chunk is not PCM, 2 channels, 48000 Hz, 16 bit")
            fmt = body
        elif name == b"data":
            if fmt is None or payload is not None:
                raise ValueError("data chunk without a preceding single fmt chunk")
            if size % FRAME:
                raise ValueError("data is not a whole number of stereo S16 frames")
            payload = body
        else:
            raise ValueError(f"unexpected chunk {name!r}")
        position += 8 + size + (size & 1)
    if payload is None:
        raise ValueError("missing data chunk")
    return payload


def _silence(payload: bytes, frame: int, limit: int, what: str) -> int:
    count = 0
    while (frame + count) * FRAME < len(payload) and \
            payload[(frame + count) * FRAME:(frame + count + 1) * FRAME] == SILENT:
        count += 1
    if count > limit:
        raise ValueError(f"{what}: {count} silent frames exceed the {limit}-frame window")
    return count


def _prefix(payload: bytes, frame: int, pattern: bytes) -> int:
    count = 0
    limit = min(len(payload) // FRAME - frame, len(pattern) // FRAME)
    while count < limit and payload[(frame + count) * FRAME:(frame + count + 1) * FRAME] == \
            pattern[count * FRAME:(count + 1) * FRAME]:
        count += 1
    return count


def check_audio(case: str, payload: bytes, submitted_a: int = 0) -> dict:
    """Frozen oracle: exact frames, only the bounded silence windows."""
    if case not in FEATURE or case in ("no-device", "access-platform"):
        raise ValueError("this case carries no expected audio")
    frames = len(payload) // FRAME
    at = lead = _silence(payload, 0, SILENCE_WINDOW, "leading silence")
    result = {"frames": frames, "leading_silent_frames": lead}
    if case == "normal":
        if payload[at * FRAME:(at + FIXTURE_FRAMES) * FRAME] != PATTERN_A:
            raise ValueError("pattern A is not present completely and byte-identically")
        at += FIXTURE_FRAMES
        result["pattern_a_frames"] = FIXTURE_FRAMES
    else:
        prefix = _prefix(payload, at, PATTERN_A)
        if prefix > submitted_a:
            raise ValueError(f"{prefix} frames of pattern A exceed the {submitted_a} submitted")
        at += prefix
        result["pattern_a_frames"] = prefix
        if case == "fence-reinit":
            gap = _silence(payload, at, SILENCE_WINDOW, "silence between A and B")
            at += gap
            if payload[at * FRAME:(at + FIXTURE_FRAMES) * FRAME] != PATTERN_B:
                raise ValueError("pattern B does not follow the A prefix completely and byte-identically")
            at += FIXTURE_FRAMES
            result.update(gap_silent_frames=gap, pattern_b_frames=FIXTURE_FRAMES)
    trail = _silence(payload, at, SILENCE_WINDOW, "trailing silence")
    at += trail
    if at != frames:
        raise ValueError(f"unexpected frame at {at} (gap, repeat, missing frame, altered byte or A after B)")
    result["trailing_silent_frames"] = trail
    return result


def check_no_audio(path: Path) -> dict:
    """No device or an offline device: no file, or a finalised empty one."""
    if not path.exists():
        return {"wav": "absent", "frames": 0}
    payload = parse_wav(path.read_bytes())
    if payload:
        raise ValueError(f"{len(payload) // FRAME} frames written although no stream may play")
    return {"wav": "empty", "frames": 0}


def expected_lines(case: str) -> list:
    """Exact SND line sequence; tuples are (regex, key) for recorded values."""
    if case not in FEATURE:
        raise ValueError("unknown virtio-snd case")
    lines: list = ["SND:BEGIN", (r"SND:TSC_PER_MS:([1-9][0-9]*)", "tsc_per_ms")]
    if case == "no-device":
        return lines + ["SND:ABSENT"]
    lines += ["SND:DEVICE:1af4:1059:00:08.0", (r"SND:BAR_BASE:([0-9a-f]{16})", "bar_base"),
              (r"SND:BAR_SIZE:([0-9a-f]{16})", "bar_size"),
              (r"SND:MMIO_PAGES:([1-9][0-9]*)", "mmio_pages"),
              (r"SND:FEATURES:([0-9a-f]{16})", "features")]
    if case == "access-platform":
        return lines + ["SND:OFFLINE:FEATURES", "SND:OFFLINE:BUS_MASTER=0:DRIVER_OK=0"]

    def bring_up(epoch: int) -> list:
        return ["SND:DRIVER_FEATURES:0000000100000000", "SND:CONFIG:JACKS=0:STREAMS=1:CHMAPS=0",
                "SND:RXQ:DISABLED", f"SND:DRIVER_OK:EPOCH={epoch}",
                (r"SND:PCM_INFO:USED_LEN=(4|36):RECORDS_LEN=36", f"pcm_info_used_len_{epoch}"),
                "SND:STREAM:0:OUTPUT",
                "SND:PARAMS:BUFFER=7680:PERIOD=1920:FEATURES=0:CHANNELS=2:FORMAT=S16:RATE=48000",
                "SND:PREPARE:OK", "SND:START:OK"]

    def played(submitted, frames: int, completions: int, key: str) -> list:
        submitted_line = (r"SND:SUBMITTED_FRAMES:([0-9]+)", key) if submitted is None \
            else f"SND:SUBMITTED_FRAMES:{submitted}"
        return [submitted_line, f"SND:FRAMES:{frames}", f"SND:TX_COMPLETIONS:{completions}"]

    finish = ["SND:DRAIN:200MS:RUNNING", "SND:STOP:OK", "SND:IDLE:200MS:STOPPED", "SND:RELEASE:OK",
              "SND:RELEASE_TWICE:REFUSED:NOT_SENT", "SND:DEVICE_STATUS:0x0f", "SND:EVENTS:INVALID=0",
              "SND:QUIESCE:ACK", "SND:USED:STABLE", "SND:PAGES:RETAINED", "SND:PASS"]
    full = FIXTURE_FRAMES
    fenced = FENCE_AFTER_PERIODS * PERIOD_FRAMES
    if case == "normal":
        return lines + bring_up(0) + played(full, full, 100, "") + finish
    if case == "fence-reinit":
        return (lines + bring_up(0) + played(fenced, fenced, FENCE_AFTER_PERIODS, "")
                + ["SND:FENCE:ACK", "SND:USED:STABLE", "SND:EPOCH:1",
                   (r"SND:FEATURES:([0-9a-f]{16})", "features_epoch_1")]
                + bring_up(1) + played(full, full, 100, "") + finish)
    return (lines + bring_up(0) + played(None, fenced, FENCE_AFTER_PERIODS, "submitted_frames")
            + ["SND:QUARANTINED", "SND:PUBLISH:REFUSED", "SND:PAGES:RETAINED", "SND:PASS"])


def evaluate_serial(case: str, stdout: str, raw_exit, timed_out: bool) -> dict:
    if timed_out or raw_exit != 33:
        raise ValueError(f"raw exit {raw_exit} (timed_out={timed_out}), expected 33")
    lines = stdout.splitlines()
    if any(marker in line for line in lines for marker in FAILURES):
        raise ValueError("failure marker present")
    if lines.count(BOOT[0]) != 1:
        raise ValueError("missing or repeated EBS handoff")
    start = lines.index(BOOT[0])
    if any(line.startswith("SND:") for line in lines[:start]):
        raise ValueError("SND marker before the kernel handoff")
    tail = lines[start:]
    if tuple(tail[:len(BOOT)]) != BOOT:
        raise ValueError(f"unexpected boot sequence: {tail[:len(BOOT)]}")
    body = tail[len(BOOT):]
    if tuple(body[-len(END):]) != END:
        raise ValueError(f"unexpected end sequence: {body[-len(END):]}")
    observed = body[:-len(END)]
    expected = expected_lines(case)
    if len(observed) != len(expected):
        raise ValueError(f"SND sequence has {len(observed)} lines, expected {len(expected)}: {observed}")
    values: dict = {}
    for line, want in zip(observed, expected):
        if isinstance(want, str):
            if line != want:
                raise ValueError(f"expected {want!r}, found {line!r}")
            continue
        match = re.fullmatch(want[0], line)
        if not match:
            raise ValueError(f"expected /{want[0]}/, found {line!r}")
        values[want[1]] = match.group(1)
    if "features" in values:
        for key in ("features", "features_epoch_1"):
            if key not in values:
                continue
            offered = int(values[key], 16)
            if not offered & VERSION_1:
                raise ValueError("device did not offer VERSION_1")
            if bool(offered & ACCESS_PLATFORM) != (case == "access-platform"):
                raise ValueError("ACCESS_PLATFORM offer does not match the case")
    if case == "quarantine":
        submitted = int(values["submitted_frames"])
        fenced = FENCE_AFTER_PERIODS * PERIOD_FRAMES
        if submitted % PERIOD_FRAMES or not fenced <= submitted <= fenced + 3 * PERIOD_FRAMES:
            raise ValueError(f"implausible submitted frame count {submitted}")
    return values


def submitted_a(case: str, values: dict) -> int:
    if case == "quarantine":
        return int(values["submitted_frames"])
    if case == "fence-reinit":
        return FENCE_AFTER_PERIODS * PERIOD_FRAMES
    return FIXTURE_FRAMES


def bounded(data: bytes) -> str:
    return data[:MAX_OUTPUT].decode("utf-8", "replace")


def run_case(case: str, *, timeout: int = 180) -> dict:
    if case not in FEATURE or not 0 < timeout <= MAX_TIMEOUT:
        raise ValueError("unknown virtio-snd case or invalid timeout")
    frozen = source_hash()
    manifest_path = BUILD / "manifest.json"
    manifest_hash = sha256(manifest_path)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if (manifest.get("kernel_feature") != FEATURE[case] or manifest.get("loader_feature") is not None
            or manifest.get("source_fingerprint_version") != 3
            or manifest.get("source_sha256") != frozen):
        raise ValueError(f"build {FEATURE[case]} from the current source first")
    if not BUILD.resolve().is_relative_to(ROOT.resolve()):
        raise ValueError("NOS_BUILD_DIR must be inside the checkout for QEMU runs")
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("qemu-system-x86_64 not found")
    version = subprocess.run([qemu, "--version"], env=env, capture_output=True, text=True,
                             timeout=30, check=True).stdout.strip()
    if not version.startswith(QEMU_SERIES):
        raise RuntimeError(f"fixture pinned to QEMU 8.2 virtio-sound-pci behaviour, found {version!r}")
    code, variables = firmware()
    run_dir = BUILD / "run" / f"snd-{case}-{uuid.uuid4().hex[:10]}"
    run_dir.mkdir(parents=True)
    shutil.copytree(BUILD / "esp", run_dir / "esp")
    for name, key in (("esp/kernel.elf", "kernel"), ("esp/EFI/BOOT/BOOTX64.EFI", "loader")):
        if sha256(run_dir / name) != manifest[key]["sha256"]:
            raise ValueError(f"snapshotted {name} does not match the build manifest")
    shutil.copy2(variables, run_dir / "OVMF_VARS.fd")
    shutil.copy2(code, run_dir / "OVMF_CODE.fd")
    rel = run_dir.relative_to(ROOT).as_posix()
    args = qemu_arguments(case, qemu, rel)
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    timed_out = False
    try:
        result = subprocess.run(args, cwd=ROOT, env=env, capture_output=True, timeout=timeout)
        rc, out, err = result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired as exc:
        timed_out, rc, out, err = True, None, exc.stdout or b"", exc.stderr or b""
    duration = time.monotonic() - start
    stdout, stderr = bounded(out), bounded(err)
    (run_dir / "serial.txt").write_text(stdout, encoding="utf-8")
    (run_dir / "stderr.txt").write_text(stderr, encoding="utf-8")
    wav = run_dir / "out.wav"
    report = dict(schema_version=1, task="H03-T3a", case=case, kernel_feature=FEATURE[case],
                  started_utc=started, duration_s=round(duration, 3), timeout_s=timeout,
                  timed_out=timed_out, raw_qemu_exit=rc, command=args, qemu_version=version,
                  qemu_machine=qemu_machine(), source_fingerprint_version=3, source_sha256=frozen,
                  manifest_sha256=manifest_hash, kernel_sha256=manifest["kernel"]["sha256"],
                  loader_sha256=manifest["loader"]["sha256"], run_directory=str(run_dir),
                  ovmf_code_sha256=sha256(run_dir / "OVMF_CODE.fd"),
                  ovmf_vars_sha256=sha256(run_dir / "OVMF_VARS.fd"),
                  scope="QEMU 8.2 TCG -icount emulator WAV output of a kernel-only fixture; "
                        "no syscall, capability, ring-3 broker, audible/physical playback, "
                        "latency or DMA-isolation claim",
                  stdout=stdout, stderr=stderr, passed=False)
    try:
        if wav.exists():
            size = wav.stat().st_size
            if size > MAX_WAV:
                raise ValueError("WAV output exceeds its bound")
            report["wav"] = {"bytes": size, "sha256": sha256(wav)}
        values = evaluate_serial(case, stdout, rc, timed_out)
        report["serial"] = values
        if case in ("no-device", "access-platform"):
            report["audio"] = check_no_audio(wav)
        else:
            if not wav.exists():
                raise ValueError("no WAV output")
            report["audio"] = check_audio(case, parse_wav(wav.read_bytes()), submitted_a(case, values))
        if source_hash() != frozen or sha256(manifest_path) != manifest_hash:
            raise ValueError("source or manifest changed during the run")
        report["passed"] = True
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    save_json(run_dir / "result.json", report)
    save_json(BUILD / f"virtio-snd-{case}-report.json", report)
    print(f"H03-T3a {case}: passed={report['passed']} raw={rc} duration={duration:.1f}s {run_dir}")
    if "error" in report:
        print(report["error"])
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=sorted(FEATURE), required=True)
    parser.add_argument("--timeout", type=int, default=180)
    arguments = parser.parse_args()
    raise SystemExit(0 if run_case(arguments.case, timeout=arguments.timeout)["passed"] else 1)
