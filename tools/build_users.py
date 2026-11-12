#!/usr/bin/env python3
"""Build the project's static user probe without Cargo's kernel linker script."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import shutil
import struct
from common import ROOT, BUILD, run_logged, save_json, sha256


SDK_DEPENDENCY_ID = "registry+https://github.com/rust-lang/crates.io-index#zeroize@1.9.1"


def sdk_dependency_command(root, build):
    """Select the runtime's locked graph without features from other SDK apps."""
    return ["cargo", "+1.90.0", "rustc", "--locked", "--release", "--no-default-features",
            "--manifest-path", str(root / "sdk/Cargo.toml"), "-p", "nieuw-os-sdk-runtime", "--lib",
            "--target", "x86_64-unknown-none", "--target-dir", str(build / "sdk-dependency-target"),
            "--message-format=json", "--", "-C", "panic=abort", "-C", "opt-level=2",
            "-C", "code-model=small", "-C", "relocation-model=static", "-C", "no-redzone=yes"]


def sdk_dependency_artifact(stdout, build):
    """Select only this Cargo invocation's exact scalar/no-feature rlib."""
    messages = [json.loads(line) for line in stdout.splitlines() if line.strip()]
    artifacts = [item for item in messages if item.get("reason") == "compiler-artifact"
                 and item.get("package_id") == SDK_DEPENDENCY_ID]
    if len(artifacts) != 1 or not any(item.get("reason") == "build-finished"
                                    and item.get("success") is True for item in messages):
        raise ValueError("Missing or ambiguous locked zeroize build artifact")
    artifact = artifacts[0]
    if artifact.get("features") != [] or artifact.get("target", {}).get("name") != "zeroize":
        raise ValueError("Unexpected zeroize target or features")
    paths = [Path(name) for name in artifact.get("filenames", []) if name.endswith(".rlib")]
    if len(paths) != 1:
        raise ValueError("Missing or ambiguous zeroize rlib")
    library = paths[0]
    release = build / "sdk-dependency-target/x86_64-unknown-none/release"
    if library.parent not in (release, release / "deps") or ".." in library.parts:
        raise ValueError("Zeroize artifact escaped the isolated target")
    if library.name != "libzeroize.rlib" and not (library.name.startswith("libzeroize-")
                                                 and library.name.endswith(".rlib")):
        raise ValueError("Unexpected zeroize artifact name")
    return library


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


def build_users(kind="probe", *, short_poll=False, sdk_runtime=False, document_session=False):
    if short_poll and kind != "m6b":
        raise ValueError("Short polling budget is an M6b test-only fixture")
    if sdk_runtime and kind != "m6c":
        raise ValueError("SDK transport bridge is an opt-in M6c fixture")
    if document_session and (kind != "m6c" or not sdk_runtime):
        raise ValueError("Document-session probe requires M6c and its SDK runtime bridge")
    fixtures = {"probe": ("probe", "linker.ld", "probe", "manifest.json"),
                "smp-unmap": ("smp_unmap", "linker.ld", "smp_unmap", "manifest-smp-unmap.json"),
                "ipc": ("ipc_probe", "ipc_linker.ld", "ipc_probe", "manifest-ipc.json"),
                "large": ("probe", "large_linker.ld", "probe_large", "manifest-large.json"),
                "m5": ("m5_probe", "m5_linker.ld", "m5_probe", "manifest-m5.json"),
                "m6": ("m6_probe", "m6_linker.ld", "m6_probe", "manifest-m6.json"),
                "m6b": ("m6b_probe", "m6b_linker.ld", "m6b_probe", "manifest-m6b.json"),
                "m6c": ("m6c_probe", "m6c_linker.ld", "m6c_probe", "manifest-m6c.json")}
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
    if short_poll:
        args.append("--cfg=nos_m6b_poll_limit")
    if sdk_runtime:
        result = run_logged(sdk_dependency_command(ROOT, BUILD), "build-sdk-zeroize")
        selected = sdk_dependency_artifact(result.stdout, BUILD)
        if selected.is_symlink() or not selected.is_file():
            raise ValueError("Missing or nonregular zeroize artifact")
        zeroize_library = BUILD / "users/libzeroize.rlib"
        shutil.copyfile(selected, zeroize_library)
        if sha256(selected) != sha256(zeroize_library):
            raise ValueError("Zeroize artifact changed while copying")
        runtime = BUILD / "users/libnieuw_os_sdk_runtime.rlib"
        run_logged(["rustc", "+1.90.0", "--edition=2021", "--crate-name=nieuw_os_sdk_runtime",
                    "--crate-type=rlib", "--target=x86_64-unknown-none", "-C", "panic=abort",
                    "-C", "opt-level=2", "-C", "code-model=small", "-C", "relocation-model=static",
                    "-C", "no-redzone=yes", "--extern", f"zeroize={zeroize_library}",
                    str(ROOT / "sdk/runtime/src/lib.rs"),
                    "-o", str(runtime)], "build-sdk-runtime")
        args += ["--cfg=nos_sdk_runtime", "--extern", f"nieuw_os_sdk_runtime={runtime}",
                 "-L", f"dependency={BUILD / 'users'}"]
    libraries = {}
    if document_session:
        libraries["nieuw_os_sdk_runtime"] = runtime
        libraries["zeroize"] = zeroize_library
        for crate, library_source in (
            ("catalog_core", "services/catalog/src/lib.rs"),
            ("volume_core", "services/volume/src/lib.rs"),
            ("nieuw_os_catalog_client", "sdk/catalog-client/src/lib.rs"),
            ("nieuw_os_document_session", "sdk/document-session/src/lib.rs"),
        ):
            library = BUILD / f"users/lib{crate}.rlib"
            command = ["rustc", "+1.90.0", "--edition=2021", f"--crate-name={crate}",
                       "--crate-type=rlib", "--target=x86_64-unknown-none", "-C", "panic=abort",
                       "-C", "opt-level=2", "-C", "code-model=small", "-C", "relocation-model=static",
                       "-C", "no-redzone=yes", "-L", f"dependency={BUILD / 'users'}"]
            dependencies = {
                "catalog_core": (),
                "volume_core": ("catalog_core",),
                "nieuw_os_catalog_client": ("nieuw_os_sdk_runtime", "catalog_core"),
                "nieuw_os_document_session": ("nieuw_os_sdk_runtime", "catalog_core", "nieuw_os_catalog_client", "volume_core"),
            }
            for dependency in dependencies[crate]:
                command += ["--extern", f"{dependency}={libraries[dependency]}"]
            command += [str(ROOT / library_source), "-o", str(library)]
            run_logged(command, f"build-document-library-{crate}")
            libraries[crate] = library
        args += ["--cfg=nos_document_session", "-L", f"dependency={BUILD / 'users'}"]
        for crate, library in libraries.items():
            if crate != "nieuw_os_sdk_runtime":
                args += ["--extern", f"{crate}={library}"]
    run_logged(args, f"build-users-{stem}")
    manifest = {"schema_version": 1, "purpose": "synthetic ring-3 acceptance probe",
                "binary_sha256": sha256(output), "source_sha256": sha256(source),
                "linker_sha256": sha256(linker), **inspect_user(output)}
    if kind == "m6b":
        manifest["short_poll_test_fixture"] = short_poll
    if kind == "m5":
        modules = ("userspace/m5_probe.rs", "userspace/m5_runtime.rs",
                   "services/block/src/wire.rs", "services/block/src/user_service.rs")
        manifest["module_sha256"] = {path: sha256(ROOT / path) for path in modules}
    if kind == "m6":
        modules = ("userspace/m6_probe.rs", "userspace/m6_runtime.rs",
                   "services/block/src/m6_wire.rs", "services/block/src/journal.rs",
                   "services/block/src/m6_user_service.rs", "services/block/src/wire.rs")
        manifest["module_sha256"] = {path: sha256(ROOT / path) for path in modules}
    if kind == "m6b":
        modules = ("userspace/m6b_probe.rs", "userspace/m6_runtime.rs",
                   "services/objects/src/wire.rs", "services/objects/src/journal.rs",
                   "services/objects/src/user_service.rs")
        manifest["module_sha256"] = {path: sha256(ROOT / path) for path in modules}
    if kind == "m6c":
        modules = ("userspace/m6c_probe.rs", "userspace/m6_runtime.rs",
                   "services/catalog/src/wire.rs", "services/catalog/src/journal.rs",
                   "services/catalog/src/user_service.rs")
        manifest["module_sha256"] = {path: sha256(ROOT / path) for path in modules}
        manifest["sdk_runtime"] = sdk_runtime
        manifest["document_session"] = document_session
        if sdk_runtime:
            manifest["sdk_dependency"] = {"package_id": SDK_DEPENDENCY_ID,
                                          "source_artifact": str(selected),
                                          "artifact_sha256": sha256(zeroize_library)}
            manifest["module_sha256"]["userspace/sdk_probe_runtime.rs"] = sha256(ROOT / "userspace/sdk_probe_runtime.rs")
            for module in sorted((ROOT / "sdk/runtime/src").glob("*.rs")):
                manifest["module_sha256"][module.relative_to(ROOT).as_posix()] = sha256(module)
        if document_session:
            inputs = [ROOT / "userspace/document_session_probe.rs", ROOT / "sdk/Cargo.toml", ROOT / "sdk/Cargo.lock"]
            for package in ("sdk/runtime", "sdk/catalog-client", "sdk/document-session", "services/catalog", "services/volume"):
                inputs.append(ROOT / package / "Cargo.toml")
                inputs.extend(sorted((ROOT / package / "src").rglob("*.rs")))
            manifest["module_sha256"].update({path.relative_to(ROOT).as_posix(): sha256(path) for path in inputs})
            manifest["library_sha256"] = {path.relative_to(BUILD).as_posix(): sha256(path)
                                          for path in libraries.values()}
    save_json(BUILD / "users" / manifest_name, manifest)
    print(f"Built own {stem} ELF: entry={manifest['entry']:#x}, {len(manifest['segments'])} segments")
    return output


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kind", choices=["probe", "smp-unmap", "ipc", "large", "m5", "m6", "m6b", "m6c"], default="probe")
    build_users(parser.parse_args().kind)
