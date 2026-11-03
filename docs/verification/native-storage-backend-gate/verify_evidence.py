"""Verify committed historical bindings, or explicitly supplied local payloads.

Neither mode executes archive code or provides new native filesystem evidence.
"""
import argparse
import hashlib
import json
from pathlib import Path
import stat


ROOT = Path(__file__).resolve().parent


def require(condition, message):
    if not condition:
        raise ValueError(message)


def target(root, relative):
    path = Path(relative)
    require(not path.is_absolute() and ".." not in path.parts, f"path escape: {relative}")
    result = root / path
    for part in (result, *result.parents):
        require(not part.is_symlink(), f"symlink: {part}")
        if part == root:
            break
    info = result.stat()
    require(stat.S_ISREG(info.st_mode) and info.st_nlink == 1, f"not independent file: {result}")
    return result


def digest(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def read(root, relative):
    return json.loads(target(root, relative).read_text(encoding="utf-8"))


def archive_checks(root):
    original = read(root, "archive-manifest.json.historical.txt")
    current = read(root, "archive-manifest.json")
    require(len(original) == len(current) == 207, "expected 207 original archive bindings")
    require(len({entry["archive"] for entry in current}) == 207, "duplicate archive binding")
    for before, after in zip(original, current):
        require({k: v for k, v in before.items() if k != "archive"}
                == {k: v for k, v in after.items() if k != "archive"},
                "original provenance changed")
        require(after["archive"] in (before["archive"], before["archive"] + ".historical.txt"),
                "unexpected snapshot relocation")
        require(digest(target(root, after["archive"])) == before["archive_sha256"],
                f"archive hash mismatch: {after['archive']}")
    return {"check": "207 original archive bindings", "matching_hashes": 207}


def json_checks(root):
    b = read(root, "btrfs/evidence.json")
    require(b["baseline_restore"]["matches"] is True
            and b["baseline_restore"]["item_count"] == 133, "Btrfs baseline")
    require(b["primary-superblock"]["restore"]["matches"] is True, "Btrfs superblock")
    require(b["payload-bitflip"]["check_exit_code"] == 1
            and b["payload-bitflip"]["restore"]["exit_code"] == 0
            and b["payload-bitflip"]["restore"]["matches"] is False
            and b["payload-bitflip"]["restore"]["changed"] == ["documents/payload.bin"]
            and b["independent_payload_difference_check"]["changed_offsets"] == [12345],
            "Btrfs negative corruption oracle")
    require(b["truncated"]["check_exit_code"] == 1
            and b["truncated"]["restore"]["exit_code"] == 1, "Btrfs truncation")
    metadata = read(root, "btrfs/metadata/metadata-evidence.json")
    require(metadata["image_unchanged"] is True
            and metadata["metadata_roundtrip_complete"] is False, "metadata limitations")
    z = read(root, "openzfs/evidence.json")
    require(z["overall_decision"] == "NOT_ACCEPTED_FOR_NIEUW_OS_NATIVE_BACKEND_OR_FILE_RECOVERY",
            "OpenZFS decision")
    gates = {gate["id"]: gate for gate in z["gates"]}
    require(gates["fresh_import_check"]["block_count"] == 1141
            and gates["fresh_import_check"]["no_leaks_message"] is True
            and gates["fresh_check_without_Y"]["exit_code"] == 0
            and gates["ztest_E_reimport"]["exit_code"] == 3
            and gates["process_crash"]["actual_SIGKILL_count"] == 3
            and gates["post_crash_fresh_check"]["reported_leaked_bytes"] == 9216
            and gates["raw_DMU_backup_stream"]["exit_code"] == 134
            and gates["raw_DMU_backup_stream"]["output_bytes"] == 0, "OpenZFS negative oracle")
    for prefix, head in (
        ("native", "e08fb0e48c1bb0781e98357f588c2f73b6ae8500"),
        ("native-current", "74a34d05ef5591b782a0da9da34e4669910d0722"),
    ):
        results = {}
        for label in ("control-normal", "control-recover", "btrfs-readonly", "openzfs-leaf-readonly"):
            result = read(root, f"{prefix}/runs/{label}/result.json")
            results[label] = result
            require(result["source"]["head"] == head and result["source_unchanged"] is True
                    and result["timed_out"] is False, f"{prefix}/{label}: source/scope")
            command = result["command"]
            require(command[command.index("-nic") + 1] == "none", "network must be disabled")
            if "readonly" in label:
                require(result["raw_qemu_exit"] == 35 and result["data_disk_read_only"] is True
                        and result["source_image_unchanged"] is True
                        and result["probe_image_unchanged"] is True
                        and result["source_image_sha256_before"] == result["source_image_sha256_after"]
                        and result["probe_image_sha256_before"] == result["probe_image_sha256_after"],
                        f"{prefix}/{label}: readonly")
            else:
                require(result["raw_qemu_exit"] == 33 and result["control_passed"] is True,
                        f"{prefix}/{label}: control")
            build = read(root, f"{prefix}/builds/{result['profile']}/manifest.json")
            require(result["kernel_sha256"] == build["kernel"]["sha256"]
                    and result["loader_sha256"] == build["loader"]["sha256"], "build binding")
        require(results["control-normal"]["probe_image_sha256_after"]
                == results["control-recover"]["probe_image_sha256_before"], "cold-image binding")
        summary = read(root, f"{prefix}/N1-result.json")
        require(summary["native_existing_volume_controls_passed"] is True
                and summary["native_backend_adoption_gate_passed"] is False, "native nonacceptance")
    require(read(root, "native/N1-result.json")["foreign_format_safe_rejection_gate_passed"] is False,
            "foreign rejection remains unaccepted")
    return {"check": "historical JSON invariants (not rerun)", "passed": True}


def local_checks(root, evidence_root):
    """Local root uses the original layout; source images have portable aliases."""
    require(evidence_root.is_dir() and not evidence_root.is_symlink(), "invalid evidence root")
    count = 0
    for relative in ("btrfs/sha256-manifest.json", "btrfs/metadata/sha256-manifest.json"):
        manifest = read(evidence_root, relative)
        require(bool(manifest), "empty local manifest")
        for name, expected in manifest.items():
            path = Path(relative).parent / name
            require(digest(target(evidence_root, path)) == expected, f"local hash: {path}")
            count += 1
    for item in read(root, "openzfs/artifact-sha256.json")["files"]:
        path = Path("openzfs") / item["path"]
        require(digest(target(evidence_root, path)) == item["sha256"], f"local hash: {path}")
        count += 1
    for prefix in ("native", "native-current"):
        for label in ("control-normal", "control-recover", "btrfs-readonly", "openzfs-leaf-readonly"):
            result = read(root, f"{prefix}/runs/{label}/result.json")
            paths = {
                f"{prefix}/runs/{label}/data.img": result["probe_image_sha256_after"],
                f"{prefix}/runs/{label}/esp/kernel.elf": result["kernel_sha256"],
                f"{prefix}/runs/{label}/esp/EFI/BOOT/BOOTX64.EFI": result["loader_sha256"],
            }
            if "readonly" in label:
                # Never follow the historical, absolute Windows source_image.
                paths[f"source-images/{prefix}/{label}.img"] = result["source_image_sha256_after"]
            for path, expected in paths.items():
                require(digest(target(evidence_root, path)) == expected, f"local hash: {path}")
                count += 1
    return {"check": "explicit local artifacts", "matching_hashes": count}


def verify(root=ROOT, evidence_root=None):
    checks = [archive_checks(root), json_checks(root)]
    if evidence_root is not None:
        checks.append(local_checks(root, evidence_root))
    return {"kind": "evidence-integrity-check",
            "mode": "local-payloads" if evidence_root is not None else "committed",
            "filesystem_acceptance": False, "new_native_execution": False,
            "local_payloads_checked": evidence_root is not None,
            "checks": checks, "all_checks_passed": True}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evidence-root", type=Path)
    parser.add_argument("--output", type=Path, help="new report only; existing files are refused")
    args = parser.parse_args(argv)
    report = verify(evidence_root=args.evidence_root)
    text = json.dumps(report, indent=2) + "\n"
    if args.output:
        with args.output.open("x", encoding="utf-8") as stream:
            stream.write(text)
    print(text, end="")


if __name__ == "__main__":
    main()
