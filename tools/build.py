#!/usr/bin/env python3
"""Build the actual UEFI loader and ELF kernel; inspect artifacts and prepare ESP."""
from __future__ import annotations
import argparse
import shutil
import struct
from common import ROOT, BUILD, run_logged, save_json, sha256, source_hash
from gate_contract import M3_CASES, M4_CASES
M5_CASES = ("service-errors", "device-rights", "async", "bad-descriptor", "used-id",
            "used-index", "used-short", "used-long", "used-status", "io-error", "timeout",
            "reset-timeout", "restart", "cpu-mmio", "cpu-dma", "cpu-pci", "no-device")
M6A_CASES = ("service-errors", "device-rights", "full", "write-error", "flush-error",
             "write-timeout", "flush-timeout", "reset-timeout", "no-flush", "readonly",
             "bad-descriptor", "used-write", "used-flush", "torn-tail", "corrupt-all",
             "stale-generation", "no-device", *(f"service-cut-{i}" for i in range(1, 7)),
             *(f"vm-cut-{i}" for i in range(1, 7)), "recover")
M6B_TIMEOUT_CASES = ("write-timeout", "flush-timeout", "reset-timeout", "write-poll-limit", "flush-poll-limit")
M6B_FAULT_CASES = ("write-error", "flush-error") + M6B_TIMEOUT_CASES
M6B_CASES = ("recover", "errors", "full", "corrupt", "vm-cut-before-ack", "vm-cut-after-flush") + M6B_FAULT_CASES
KERNEL_FEATURES = ["test-panic", "test-hang", "milestone-m2", "test-m2-ro", "test-m2-nx",
                   "test-m2-absent", "test-m2-kernel-fault", "test-m2-double-fault",
                   "milestone-m3", "test-m3-idle", "test-m3-large-image", *(f"test-m3-{case}" for case in M3_CASES),
                   "milestone-m4", *(f"test-m4-{case}" for case, _ in M4_CASES),
                   "milestone-m5", *(f"test-m5-{case}" for case in M5_CASES),
                   "milestone-m6a", *(f"test-m6a-{case}" for case in M6A_CASES),
                   "milestone-m6b", *(f"test-m6b-{case}" for case in M6B_CASES)]


def inspect_elf(path):
    data = path.read_bytes()
    if len(data) < 64 or data[:7] != b"\x7fELF\x02\x01\x01":
        raise ValueError("not ELF64 little endian")
    kind, machine, version, entry, phoff, shoff, flags, ehsize, phsize, phnum, shsize, shnum, shstr = struct.unpack_from("<HHIQQQIHHHHHH", data, 16)
    if (kind, machine, version, ehsize, phsize) != (2, 62, 1, 64, 56) or not 1 <= phnum <= 64:
        raise ValueError("invalid ELF header")
    segments = []
    if phoff + phsize * phnum > len(data):
        raise ValueError("truncated program header table")
    for n in range(phnum):
        typ, fl, off, virt, phys, filesz, memsz, align = struct.unpack_from("<IIQQQQQQ", data, phoff + n * phsize)
        if typ not in (0, 1, 4, 6, 0x6474e551) or (typ == 0x6474e551 and fl & 1):
            raise ValueError("unsupported or executable metadata segment")
        if typ != 1 or memsz == 0:
            continue
        if not (filesz <= memsz and off + filesz <= len(data) and phys == virt and 0x2000000 <= virt < virt + memsz <= 0x3000000 and fl in (4, 5, 6) and align >= 4096 and align & (align-1) == 0 and off % align == virt % align):
            raise ValueError("invalid load segment")
        low, high = virt // 4096 * 4096, (virt + memsz + 4095) // 4096 * 4096
        if any(low < old["page_end"] and old["page_start"] < high for old in segments):
            raise ValueError("overlapping segment pages")
        segments.append(dict(flags=fl, offset=off, address=virt, filesz=filesz, memsz=memsz, page_start=low, page_end=high))
    if sum(s["flags"] == 5 and s["address"] <= entry < s["address"] + s["filesz"] for s in segments) != 1:
        raise ValueError("entry not in exactly one loaded RX segment")
    if shnum == 0 or shsize != 64 or shoff + shnum * shsize > len(data):
        raise ValueError("missing/invalid build-time section table")
    section_headers = [struct.unpack_from("<IIQQQQIIQQ", data, shoff + n * shsize) for n in range(shnum)]
    if shstr >= shnum:
        raise ValueError("invalid section name table")
    names_head = section_headers[shstr]
    names = data[names_head[4]:names_head[4]+names_head[5]]
    sections = []
    for section in section_headers:
        name = names[section[0]:].split(b"\0", 1)[0].decode("ascii", "strict")
        if section[2] & 2:
            if name not in (".text", ".rodata", ".data", ".bss"):
                raise ValueError(f"unexpected allocatable section {name}")
            if not any(s["address"] <= section[3] and section[3] + section[5] <= s["address"] + s["memsz"] for s in segments):
                raise ValueError(f"section outside load plan: {name}")
        if section[1] in (4, 6, 9, 11) and section[5]:
            raise ValueError(f"dynamic/relocation metadata: {name}")
        sections.append(dict(name=name, type=section[1], flags=section[2], size=section[5]))
    return dict(entry=entry, segments=segments, sections=sections)


def inspect_pe(path):
    data = path.read_bytes()
    if data[:2] != b"MZ" or len(data) < 64:
        raise ValueError("loader is not PE")
    pe = struct.unpack_from("<I", data, 60)[0]
    if data[pe:pe+4] != b"PE\0\0" or struct.unpack_from("<H", data, pe+4)[0] != 0x8664:
        raise ValueError("loader is not AMD64 PE")
    opt = pe+24
    if struct.unpack_from("<H", data, opt)[0] != 0x20b or struct.unpack_from("<H", data, opt+68)[0] != 10:
        raise ValueError("loader is not PE32+ EFI application")
    if struct.unpack_from("<II", data, opt+120) != (0, 0):
        raise ValueError("loader unexpectedly imports host DLLs")
    return {"format": "AMD64 PE32+", "subsystem": "EFI_APPLICATION", "imports": 0}


def build(loader_feature=None, kernel_feature=None, label="normal"):
    extra_env = None
    if kernel_feature and any(m in kernel_feature for m in ("m3", "m4", "m5", "m6a", "m6b")):
        from build_users import build_users
        extra_env = {"NOS_USER_ELF": str(build_users("large" if kernel_feature == "test-m3-large-image" else "probe").resolve())}
        if any(m in kernel_feature for m in ("m4", "m5", "m6a", "m6b")):
            extra_env["NOS_IPC_ELF"] = str(build_users("ipc").resolve())
        if any(m in kernel_feature for m in ("m5", "m6a", "m6b")):
            extra_env["NOS_M5_ELF"] = str(build_users("m5").resolve())
        if "m6a" in kernel_feature:
            extra_env["NOS_M6_ELF"] = str(build_users("m6").resolve())
        if "m6b" in kernel_feature:
            extra_env["NOS_M6B_ELF"] = str(build_users("m6b", short_poll=kernel_feature in
                ("test-m6b-write-poll-limit", "test-m6b-flush-poll-limit")).resolve())
    if not (ROOT / "Cargo.lock").exists():
        run_logged(["cargo", "+1.90.0", "generate-lockfile"], "lockfile")
    for package, target, feature in (("os-loader", "x86_64-unknown-uefi", loader_feature), ("os-kernel", "x86_64-unknown-none", kernel_feature)):
        args = ["cargo", "+1.90.0", "build", "--locked", "--release", "--target-dir", str(ROOT / "target"), "-p", package, "--target", target]
        if feature:
            args += ["--features", feature]
        run_logged(args, f"build-{label}-{package}", extra_env=extra_env if package == "os-kernel" else None)
    loader = ROOT / "target/x86_64-unknown-uefi/release/os-loader.efi"
    kernel = ROOT / "target/x86_64-unknown-none/release/os-kernel"
    manifest = {"schema_version": 1, "profile": label, "kernel_feature": kernel_feature,
                "loader_feature": loader_feature, "source_fingerprint_version": 3,
                "source_sha256": source_hash(), "loader": {"sha256": sha256(loader), **inspect_pe(loader)},
                "kernel": {"sha256": sha256(kernel), **inspect_elf(kernel)}}
    esp = BUILD / "esp"
    (esp / "EFI/BOOT").mkdir(parents=True, exist_ok=True)
    shutil.copy2(loader, esp / "EFI/BOOT/BOOTX64.EFI")
    shutil.copy2(kernel, esp / "kernel.elf")
    save_json(BUILD / "manifest.json", manifest)
    save_json(BUILD / f"logs/build-{label}-manifest.json", manifest)
    print(f"Built and inspected {label}: {len(manifest['kernel']['segments'])} ELF load segments")
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--loader-feature", choices=["test-ebs-retry"])
    parser.add_argument("--kernel-feature", choices=KERNEL_FEATURES)
    args = parser.parse_args()
    build(args.loader_feature, args.kernel_feature, args.loader_feature or args.kernel_feature or "normal")
