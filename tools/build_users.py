#!/usr/bin/env python3
"""Build the project's static user probe without Cargo's kernel linker script."""
from __future__ import annotations
import argparse
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


def build_users(kind="probe"):
    fixtures = {"probe": ("probe", "linker.ld", "probe", "manifest.json"),
                "ipc": ("ipc_probe", "ipc_linker.ld", "ipc_probe", "manifest-ipc.json"),
                "large": ("probe", "large_linker.ld", "probe_large", "manifest-large.json"),
                "m5": ("m5_probe", "m5_linker.ld", "m5_probe", "manifest-m5.json")}
    if kind not in fixtures:
        raise ValueError("unknown synthetic user fixture")
    source_stem, linker_name, stem, manifest_name = fixtures[kind]
    output = BUILD / f"users/{stem}.elf"
    output.parent.mkdir(parents=True, exist_ok=True)
    source = ROOT / f"userspace/{source_stem}.rs"
    linker = ROOT / f"userspace/{linker_name}"
    args = ["rustc", "+1.90.0", "--edition=2021", "--crate-type=bin", f"--crate-name=os_user_{stem}",
            "--target=x86_64-unknown-none", "-C", "panic=abort", "-C", "opt-level=2",
            "-C", "code-model=small", "-C", "relocation-model=static", "-C", "no-redzone=yes",
            "-C", "linker=rust-lld", "-C", f"link-arg=-T{linker}", "-C", "link-arg=--no-pie",
            "-C", "link-arg=--build-id=none", "-C", "link-arg=-zmax-page-size=0x1000",
            "--emit=link", "-o", str(output), str(source)]
    run_logged(args, f"build-users-{stem}")
    manifest = {"schema_version": 1, "purpose": "synthetic ring-3 acceptance probe",
                "binary_sha256": sha256(output), "source_sha256": sha256(source),
                "linker_sha256": sha256(linker), **inspect_user(output)}
    if kind == "m5":
        modules = ("userspace/m5_probe.rs", "userspace/m5_runtime.rs",
                   "services/block/src/wire.rs", "services/block/src/user_service.rs")
        manifest["module_sha256"] = {path: sha256(ROOT / path) for path in modules}
    save_json(BUILD / "users" / manifest_name, manifest)
    print(f"Built own {stem} ELF: entry={manifest['entry']:#x}, {len(manifest['segments'])} segments")
    return output


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kind", choices=["probe", "ipc", "large", "m5"], default="probe")
    build_users(parser.parse_args().kind)
