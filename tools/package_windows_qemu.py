#!/usr/bin/env python3
"""Package the real native Materiaal profile for QEMU on Windows."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import tempfile
import zipfile

import volume_disk
from build_users import inspect_user
from common import BUILD, ROOT, sha256, source_hash

PROFILE = "native-system-materiaal"
SESSION_PROFILE = "native-system-session-preview"
APP_NAMES = (
    "desktop-init",
    "keyring-service",
    "volume-service",
    "materiaal-app",
    "native-compositor",
    "input-source",
    "input-policy",
)
HEX = re.compile(r"[0-9a-f]{64}")
MAX_FIRMWARE_BYTES = 32 * 1024 * 1024
MAX_ROM_BYTES = 1024 * 1024
MAX_LICENSE_BYTES = 1024 * 1024
PACKAGE_FILES = (
    ("tools/windows-qemu/Start-NieuwOS.ps1", "Start-NieuwOS.ps1"),
    ("tools/windows-qemu/Acceleration.ps1", "Acceleration.ps1"),
    ("tools/windows-qemu/README.md", "README.md"),
)


def regular_file(path: Path, label: str, limit: int) -> Path:
    details = path.lstat()
    if not stat.S_ISREG(details.st_mode) or details.st_nlink != 1:
        raise ValueError(f"{label} must be a regular, single-link file")
    if not 0 < details.st_size <= limit:
        raise ValueError(f"{label} has an invalid size")
    return path


def load_json(path: Path, label: str, limit: int = 4 * 1024 * 1024) -> dict:
    def unique_object(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f"{label} contains a duplicate JSON key")
            result[key] = value
        return result

    regular_file(path, label, limit)
    value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_object)
    if not isinstance(value, dict):
        raise ValueError(f"{label} must contain a JSON object")
    return value


def check_esp(build: Path, manifest: dict) -> dict[str, str]:
    esp = build / "esp"
    if not stat.S_ISDIR(esp.lstat().st_mode):
        raise ValueError("ESP must be a real directory")
    expected = {
        "kernel.elf": manifest["kernel"]["sha256"],
        "EFI/BOOT/BOOTX64.EFI": manifest["loader"]["sha256"],
    }
    files = set()
    for path in esp.rglob("*"):
        details = path.lstat()
        relative = path.relative_to(esp).as_posix()
        if stat.S_ISDIR(details.st_mode):
            if relative not in {"EFI", "EFI/BOOT"}:
                raise ValueError(f"Unexpected ESP directory: {relative}")
        elif stat.S_ISREG(details.st_mode) and details.st_nlink == 1:
            files.add(relative)
        else:
            raise ValueError(f"Linked or special ESP entry: {relative}")
    if files != set(expected):
        raise ValueError("ESP must contain exactly the inspected loader and kernel")
    for relative, digest in expected.items():
        path = regular_file(esp / relative, f"ESP {relative}", MAX_FIRMWARE_BYTES)
        if not isinstance(digest, str) or not HEX.fullmatch(digest) or sha256(path) != digest:
            raise ValueError(f"ESP artifact differs from its manifest: {relative}")
    return expected


def validate_build(build_directory: Path, *, verify_source: bool = True) -> tuple[Path, dict]:
    build = build_directory.resolve(strict=True)
    if not stat.S_ISDIR(build.lstat().st_mode):
        raise ValueError("Build path must be a real directory")
    manifest = load_json(build / "manifest.json", "build manifest")
    if (manifest.get("kernel_feature") not in (PROFILE, SESSION_PROFILE)
            or manifest.get("loader_feature") != "framebuffer"
            or manifest.get("source_fingerprint_version") != 3
            or type(manifest.get("source_fingerprint_version")) is not int
            or not isinstance(manifest.get("source_sha256"), str)
            or not HEX.fullmatch(manifest["source_sha256"])):
        raise ValueError("Build manifest is not the expected native Materiaal profile")
    if verify_source and manifest["source_sha256"] != source_hash():
        raise ValueError("Build source fingerprint differs from the current checkout")

    materiaal = manifest.get("materiaal")
    if (not isinstance(materiaal, dict)
            or set(materiaal) != {"passed", "native_runtime_performed", "probe_controller", "apps"}
            or materiaal.get("passed") is not True
            or materiaal.get("native_runtime_performed") is not False
            or materiaal.get("probe_controller") is not False
            or not isinstance(materiaal.get("apps"), list)
            or len(materiaal["apps"]) != len(APP_NAMES)):
        raise ValueError("Build must contain the seven-app non-probe Materiaal profile")

    kernel_path = regular_file(build / "esp/kernel.elf", "kernel", MAX_FIRMWARE_BYTES)
    kernel = kernel_path.read_bytes()
    for expected_name, item in zip(APP_NAMES, materiaal["apps"], strict=True):
        if (not isinstance(item, dict) or set(item) != {"name", "sha256", "elf"}
                or item.get("name") != expected_name
                or not isinstance(item.get("sha256"), str)
                or not HEX.fullmatch(item["sha256"])):
            raise ValueError("Userspace manifest is missing, changed, or reordered")
        app = regular_file(build / "users/materiaal" / f"{expected_name}.elf",
                           f"userspace ELF {expected_name}", MAX_FIRMWARE_BYTES)
        if (sha256(app) != item["sha256"] or inspect_user(app) != item["elf"]
                or app.read_bytes() not in kernel):
            raise ValueError(f"Userspace ELF is not bound into the kernel: {expected_name}")
    check_esp(build, manifest)
    return build, manifest


def file_record(path: Path, label: str, limit: int) -> tuple[bytes, dict]:
    regular_file(path, label, limit)
    data = path.read_bytes()
    return data, {"sha256": hashlib.sha256(data).hexdigest(), "size_bytes": len(data)}


def git_commit() -> str:
    return subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True, timeout=10
    ).strip()


def add_entry(archive: zipfile.ZipFile, name: str, data: bytes) -> None:
    entry = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
    entry.compress_type = zipfile.ZIP_DEFLATED
    entry.external_attr = 0o100644 << 16
    archive.writestr(entry, data, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)


def package_bundle(
    *,
    build_directory: Path,
    output: Path,
    ovmf_code: Path,
    ovmf_vars: Path,
    seabios_bios: Path,
    vga_rom: Path,
    ovmf_license: Path,
    seabios_license: Path,
    ovmf_version: str,
    seabios_version: str,
    verify_source: bool = True,
) -> dict:
    build, manifest = validate_build(build_directory, verify_source=verify_source)
    if output.exists():
        raise FileExistsError(f"Refusing to overwrite existing package: {output}")
    parent = Path(os.path.abspath(output.parent))
    if parent.resolve() != parent:
        raise ValueError("Package output parent must not traverse symlinks")
    parent.mkdir(parents=True, exist_ok=True)
    if parent.resolve() != parent:
        raise ValueError("Package output parent must not traverse symlinks")
    output = parent / output.name

    payload: dict[str, bytes] = {}
    hashes: dict[str, dict] = {}
    for source, name, limit, label in (
        (ovmf_code, "OVMF_CODE.fd", MAX_FIRMWARE_BYTES, "OVMF code"),
        (ovmf_vars, "OVMF_VARS.template.fd", MAX_FIRMWARE_BYTES, "OVMF variables template"),
        (seabios_bios, "bios-256k.bin", MAX_ROM_BYTES, "SeaBIOS system ROM"),
        (vga_rom, "VGA.rom", MAX_ROM_BYTES, "SeaBIOS VGA ROM"),
        (ovmf_license, "licenses/OVMF-COPYRIGHT", MAX_LICENSE_BYTES, "OVMF license notice"),
        (seabios_license, "licenses/SeaBIOS-COPYRIGHT", MAX_LICENSE_BYTES, "SeaBIOS license notice"),
    ):
        data, record = file_record(source, label, limit)
        payload[name] = data
        hashes[name] = record

    for source_name, package_name in PACKAGE_FILES:
        path = ROOT / source_name
        data, record = file_record(path, source_name, MAX_LICENSE_BYTES)
        payload[package_name] = data
        hashes[package_name] = record

    with tempfile.TemporaryDirectory(prefix="nieuw-os-qemu-package-") as temporary:
        image = Path(temporary) / "volume.empty.img"
        volume_disk.create_image(image, "empty")
        data, record = file_record(image, "generated synthetic volume", volume_disk.IMAGE_SECTORS * volume_disk.SECTOR)
        payload["volume.empty.img"] = data
        hashes["volume.empty.img"] = record

    for relative in ("esp/kernel.elf", "esp/EFI/BOOT/BOOTX64.EFI"):
        path = build / relative
        data, record = file_record(path, relative, MAX_FIRMWARE_BYTES)
        expected = manifest["kernel" if relative == "esp/kernel.elf" else "loader"]["sha256"]
        if record["sha256"] != expected:
            raise ValueError(f"Build artifact changed while packaging: {relative}")
        payload[relative] = data
        hashes[relative] = record

    if verify_source and source_hash() != manifest["source_sha256"]:
        raise ValueError("Source changed while packaging")

    package_manifest = {
        "schema_version": 1,
        "kind": "Nieuw OS native Materiaal QEMU test version",
        "scope": (
            f"Own x86_64 UEFI kernel; {manifest['kernel_feature']}; one q35-8.2 CPU, "
            "automatic WHPX initialization probe with explicit TCG fallback, 256 MiB RAM; "
            "network disabled; read-only ESP directory exposed "
            "via fat:esp; no direct host disk or other host directories configured."
        ),
        "source_commit": git_commit(),
        "source_fingerprint_version": manifest["source_fingerprint_version"],
        "source_sha256": manifest["source_sha256"],
        "kernel_feature": manifest["kernel_feature"],
        "loader_feature": "framebuffer",
        "qemu": {
            "minimum_version": "10.2.1",
            "required_rng_backend": "rng-builtin",
            "machine": "pc-q35-8.2",
            "interactive_acceleration": "auto-whpx-then-tcg",
            "acceptance_acceleration": "tcg,thread=single",
        },
        "firmware": {
            "ovmf_package_version": ovmf_version,
            "seabios_package_version": seabios_version,
        },
        "files": hashes,
        "limitations": [
            "Not an installer or full release.",
            "Windows validation is specific to the tested source and bundle; packaging does not run a VM acceptance test.",
            "The 256 MiB VM profile is a development test profile, not a supported-hardware claim.",
        ],
    }
    payload["manifest.json"] = (json.dumps(package_manifest, indent=2, sort_keys=True) + "\n").encode()

    temporary_output = None
    try:
        with tempfile.NamedTemporaryFile(dir=output.parent, prefix=output.name + ".", delete=False) as temp:
            temporary_output = Path(temp.name)
        with zipfile.ZipFile(temporary_output, "w", allowZip64=False) as archive:
            for name in sorted(payload):
                add_entry(archive, name, payload[name])
        os.link(temporary_output, output)
    except FileExistsError:
        raise FileExistsError(f"Refusing to overwrite existing package: {output}") from None
    finally:
        if temporary_output is not None:
            temporary_output.unlink(missing_ok=True)

    return {
        "output": str(output),
        "sha256": sha256(output),
        "source_commit": package_manifest["source_commit"],
        "source_sha256": package_manifest["source_sha256"],
        "files": hashes,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build-dir", type=Path, default=BUILD)
    parser.add_argument("--output", type=Path, default=BUILD / "NieuwOS-Testversie-Windows-QEMU.zip")
    parser.add_argument("--ovmf-code", type=Path, required=True)
    parser.add_argument("--ovmf-vars", type=Path, required=True)
    parser.add_argument("--seabios-bios", type=Path, required=True)
    parser.add_argument("--vga-rom", type=Path, required=True)
    parser.add_argument("--ovmf-license", type=Path, required=True)
    parser.add_argument("--seabios-license", type=Path, required=True)
    parser.add_argument("--ovmf-version", required=True)
    parser.add_argument("--seabios-version", required=True)
    args = parser.parse_args()
    result = package_bundle(
        build_directory=args.build_dir,
        output=args.output,
        ovmf_code=args.ovmf_code,
        ovmf_vars=args.ovmf_vars,
        seabios_bios=args.seabios_bios,
        vga_rom=args.vga_rom,
        ovmf_license=args.ovmf_license,
        seabios_license=args.seabios_license,
        ovmf_version=args.ovmf_version,
        seabios_version=args.seabios_version,
    )
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
