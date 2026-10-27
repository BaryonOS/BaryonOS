#!/usr/bin/env python3
"""Build the project's static user probe without Cargo's kernel linker script."""
from __future__ import annotations
import struct
from common import ROOT, BUILD, run_logged, save_json, sha256


def inspect_user(path):
    data = path.read_bytes()
    if len(data) < 64 or data[:7] != b"\x7fELF\x02\x01\x01":
        raise ValueError("user fixture is not ELF64 little endian")
    kind, machine, version, entry, phoff, _, _, ehsize, phsize, phnum = struct.unpack_from("<HHIQQQIHHH", data, 16)
    if (kind, machine, version, ehsize, phsize) != (2, 62, 1, 64, 56) or not 1 <= phnum <= 64:
        raise ValueError("invalid user ELF header")
    if phoff + phnum * phsize > len(data):
        raise ValueError("truncated user program headers")
    segments = []
    for i in range(phnum):
        typ, flags, offset, va, pa, filesz, memsz, alignment = struct.unpack_from("<IIQQQQQQ", data, phoff + i * phsize)
        if typ not in (0, 1, 4, 6, 0x6474e551) or (typ == 0x6474e551 and flags & 1):
            raise ValueError("unsupported user program header")
        if typ != 1 or memsz == 0:
            continue
        if not (filesz <= memsz and offset + filesz <= len(data)
                and 0x40000000 <= va < va + memsz <= 0x60000000
                and flags in (4, 5, 6) and pa == va
                and alignment >= 4096 and alignment & (alignment - 1) == 0
                and va % alignment == offset % alignment):
            raise ValueError("unsafe user fixture segment")
        start, end = va // 4096 * 4096, (va + memsz + 4095) // 4096 * 4096
        if any(start < s["page_end"] and s["page_start"] < end for s in segments):
            raise ValueError("overlapping user fixture pages")
        segments.append(dict(flags=flags, address=va, offset=offset, filesz=filesz,
                             memsz=memsz, page_start=start, page_end=end))
    if sum(s["flags"] == 5 and s["address"] <= entry < s["address"] + s["filesz"] for s in segments) != 1:
        raise ValueError("user entry not in exactly one loaded RX segment")
    if not any(s["flags"] == 6 and s["address"] == 0x40010000 and s["memsz"] >= 32 for s in segments):
        raise ValueError("missing user probe data interface")
    return dict(entry=entry, segments=segments)


def build_users():
    output = BUILD / "users/probe.elf"
    output.parent.mkdir(parents=True, exist_ok=True)
    source, linker = ROOT / "userspace/probe.rs", ROOT / "userspace/linker.ld"
    args = ["rustc", "+1.90.0", "--edition=2021", "--crate-type=bin", "--crate-name=os_user_probe",
            "--target=x86_64-unknown-none", "-C", "panic=abort", "-C", "opt-level=2",
            "-C", "code-model=small", "-C", "relocation-model=static", "-C", "no-redzone=yes",
            "-C", "linker=rust-lld", "-C", f"link-arg=-T{linker}", "-C", "link-arg=--no-pie",
            "-C", "link-arg=--build-id=none", "-C", "link-arg=-zmax-page-size=0x1000",
            "--emit=link", "-o", str(output), str(source)]
    run_logged(args, "build-users-probe")
    manifest = {"schema_version": 1, "purpose": "synthetic ring-3 acceptance probe",
                "binary_sha256": sha256(output), "source_sha256": sha256(source),
                "linker_sha256": sha256(linker), **inspect_user(output)}
    save_json(BUILD / "users/manifest.json", manifest)
    print(f"Built own user ELF: entry={manifest['entry']:#x}, {len(manifest['segments'])} segments")
    return output


if __name__ == "__main__":
    build_users()
