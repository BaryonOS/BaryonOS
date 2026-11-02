"""Independent evidence/hash checks, not an additional filesystem acceptance test."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent

def read(relative):
    return json.loads((ROOT / relative).read_text(encoding="utf-8"))

def digest(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()

def main():
    checks = []
    for relative in ("btrfs/sha256-manifest.json", "btrfs/metadata/sha256-manifest.json"):
        manifest_path = ROOT / relative
        values = json.loads(manifest_path.read_text(encoding="utf-8"))
        count = 0
        for name, expected in values.items():
            target = (manifest_path.parent / name).resolve()
            assert target.is_relative_to((ROOT / "btrfs").resolve()), name
            assert target.is_file(), name
            assert digest(target) == expected, name
            count += 1
        checks.append({"check": relative, "matching_hashes": count})
    b = read("btrfs/evidence.json")
    assert b["baseline_restore"]["matches"] is True
    assert b["baseline_restore"]["item_count"] == 133
    assert b["primary-superblock"]["restore"]["matches"] is True
    assert b["payload-bitflip"]["check_exit_code"] == 1
    assert b["payload-bitflip"]["restore"]["exit_code"] == 0
    assert b["payload-bitflip"]["restore"]["matches"] is False
    assert b["payload-bitflip"]["restore"]["changed"] == ["documents/payload.bin"]
    assert b["independent_payload_difference_check"]["changed_offsets"] == [12345]
    assert b["truncated"]["check_exit_code"] == 1
    assert b["truncated"]["restore"]["exit_code"] == 1
    checks.append({"check": "Btrfs corruption outcomes", "passed": True})
    metadata = read("btrfs/metadata/metadata-evidence.json")
    assert metadata["image_unchanged"] is True
    assert metadata["metadata_roundtrip_complete"] is False
    checks.append({"check": "Metadata limitations preserved", "passed": True})
    results = {}
    for label in ("control-normal", "control-recover", "btrfs-readonly", "openzfs-leaf-readonly"):
        result = read(f"native/runs/{label}/result.json")
        results[label] = result
        assert result["source"]["head"] == "e08fb0e48c1bb0781e98357f588c2f73b6ae8500"
        assert result["source_unchanged"] is True
        assert result["timed_out"] is False
        assert result["command"][result["command"].index("-nic") + 1] == "none"
        if "readonly" in label:
            assert result["raw_qemu_exit"] == 35
            assert result["data_disk_read_only"] is True
            assert result["source_image_unchanged"] is True
            assert result["probe_image_unchanged"] is True
            assert digest(Path(result["source_image"])) == result["source_image_sha256_after"]
        else:
            assert result["raw_qemu_exit"] == 33
            assert result["control_passed"] is True
        assert digest(ROOT / f"native/runs/{label}/data.img") == result["probe_image_sha256_after"]
        checks.append({"check": label, "raw_qemu_exit": result["raw_qemu_exit"], "hash_and_scope_verified": True})
    assert results["control-normal"]["probe_image_sha256_after"] == results["control-recover"]["probe_image_sha256_before"]
    n1 = read("native/N1-result.json")
    assert n1["native_existing_volume_controls_passed"] is True
    assert n1["native_backend_adoption_gate_passed"] is False
    assert n1["foreign_format_safe_rejection_gate_passed"] is False
    z = read("openzfs/evidence.json")
    assert z["overall_decision"] == "NOT_ACCEPTED_FOR_NIEUW_OS_NATIVE_BACKEND_OR_FILE_RECOVERY"
    gates = {gate["id"]: gate for gate in z["gates"]}
    assert gates["fresh_import_check"]["block_count"] == 1141
    assert gates["fresh_import_check"]["no_leaks_message"] is True
    assert gates["fresh_check_without_Y"]["exit_code"] == 0
    assert gates["ztest_E_reimport"]["exit_code"] == 3
    assert gates["process_crash"]["actual_SIGKILL_count"] == 3
    assert gates["post_crash_fresh_check"]["reported_leaked_bytes"] == 9216
    assert gates["raw_DMU_backup_stream"]["exit_code"] == 134
    assert gates["raw_DMU_backup_stream"]["output_bytes"] == 0
    zmanifest = read("openzfs/artifact-sha256.json")
    zcount = 0
    for item in zmanifest["files"]:
        target = (ROOT / "openzfs" / item["path"]).resolve()
        assert target.is_relative_to((ROOT / "openzfs").resolve()), item["path"]
        assert target.is_file(), item["path"]
        assert digest(target) == item["sha256"], item["path"]
        zcount += 1
    checks.append({"check": "OpenZFS artifact hashes and negative oracle", "matching_hashes": zcount,
                   "native_and_file_recovery_accepted": False})
    report = {"kind": "evidence-integrity-check", "filesystem_acceptance": False,
              "checks": checks, "all_checks_passed": True}
    (ROOT / "verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))

if __name__ == "__main__":
    main()
