#!/usr/bin/env python3
"""Read-only evidence replay; never builds, starts QEMU or alters producer files."""

import argparse
import copy
import hashlib
import importlib
import json
from pathlib import Path, PurePosixPath
import subprocess
import sys
import tarfile


COMMIT = "0818f5ec651b72c32b8795ff0d3d9a6ebdc9783d"
SOURCE = "26b41e5586434ce51202a4e234ccaa99bad25c5a76ec3efdb3af9d7020c14f5f"
ARCHIVE = "3c476387f7aa7657207e2a0ee96b7d1b159695de1716305cdbefa795b26447b2"
MEMBERS = "f3d16ff804dec187b01949c1755539ca047b4dcee98e409794a62b3a43185db1"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", required=True, type=Path)
    parser.add_argument("--handoff", required=True, type=Path)
    parser.add_argument("--source-root", required=True, type=Path)
    args = parser.parse_args()
    sys.dont_write_bytecode = True
    root = args.source_root.resolve(strict=True)
    archive = args.archive.read_bytes()
    require(len(archive) == 2682413 and digest(archive) == ARCHIVE, "Bundle identity mismatch")
    handoff_raw = args.handoff.read_bytes()
    handoff = json.loads(handoff_raw)
    require(handoff["source_commit"] == COMMIT and handoff["source_sha256"] == SOURCE,
            "Handoff source mismatch")
    directory = Path(handoff["run_directory"])
    require(directory.is_relative_to(root / "build"), "Run is outside supplied candidate build")
    member_hashes = {}
    with tarfile.open(args.archive, "r:gz") as bundle:
        members = bundle.getmembers()
        names = [member.name for member in members]
        require(len(names) == len(set(names)) == 41, "Duplicate/missing archive member")
        for member in members:
            path = PurePosixPath(member.name)
            require(member.isfile() and not path.is_absolute() and ".." not in path.parts
                    and 0 <= member.size <= 32 * 1024 * 1024, "Unsafe archive member")
        manifest_raw = bundle.extractfile("members.sha256.json").read()
        require(digest(manifest_raw) == MEMBERS, "Member manifest identity mismatch")
        manifest = json.loads(manifest_raw)
        require(len(manifest) == 40 and set(manifest) == set(names) - {"members.sha256.json"},
                "Member manifest coverage mismatch")
        for name, expected in manifest.items():
            raw = bundle.extractfile(name).read()
            require(digest(raw) == expected, "Bundle member hash mismatch: " + name)
            member_hashes[name] = expected
            if name.startswith("run/"):
                require((directory / name.removeprefix("run/")).read_bytes() == raw,
                        "Canonical run differs from bundle: " + name)
        report = json.loads(bundle.extractfile("run/result.json").read())
    require(report["source_commit"] == COMMIT and report["source_sha256"] == SOURCE,
            "Run source mismatch")
    for key in ("source_commit", "source_sha256", "kernel_feature", "command", "profile",
                "frozen_artifacts", "before_sha256", "after_sha256", "stages",
                "raw_qemu_exit", "timed_out", "qemu_entry_sha256", "qemu_implementation_sha256"):
        require(report[key] == handoff[key], "Handoff/run mismatch: " + key)

    tools = root / "tools"
    # Read candidate code from Git, compare before importing it, and reuse its
    # existing verifier without changing path guards, assertions or reports.
    repository = Path(__file__).resolve().parents[3]
    tool_hashes = {}
    for path in sorted(tools.glob("*.py")):
        relative = "tools/" + path.name
        expected = subprocess.check_output(
            ["git", "show", f"{COMMIT}:{relative}"], cwd=repository
        )
        require(path.read_bytes() == expected, "Candidate tool source changed: " + relative)
        tool_hashes[relative] = digest(expected)
    sys.path.insert(0, str(tools))
    runner = importlib.import_module("run_materiaal_probe")
    common = importlib.import_module("common")
    require(common.ROOT == root and common.source_hash() == SOURCE,
            "Candidate runtime source fingerprint changed")
    observed = runner.replay(report, directory, SOURCE)
    require(observed["disk"] == handoff["disk"], "Recomputed disk differs from handoff")
    require(json.dumps(observed["frames"], sort_keys=True)
            == json.dumps(handoff["frames"], sort_keys=True),
            "Recomputed frames differ from handoff")
    rejected = []
    for name in ("wrong-exit", "wrong-source", "wrong-member-hash", "wrong-capture-index"):
        changed = copy.deepcopy(report)
        if name == "wrong-exit":
            changed["raw_qemu_exit"] = 0
        elif name == "wrong-source":
            changed["source_sha256"] = "0" * 64
        elif name == "wrong-member-hash":
            changed["artifact_sha256"]["serial.txt"] = "0" * 64
        else:
            changed["stages"][0]["qmp_index"] = 1
        try:
            runner.replay(changed, directory, SOURCE)
        except ValueError:
            rejected.append(name)
        else:
            raise ValueError("Invalid evidence accepted: " + name)
    for key in ("qemu_entry_path", "qemu_implementation_path"):
        path = Path(report[key])
        require(digest(path.read_bytes()) == report[key.removesuffix("_path") + "_sha256"],
                "Existing QEMU binary changed")
    trace = json.loads((directory / "qmp-trace.json").read_bytes())
    for name in runner.PHASES["normal"]:
        require(member_hashes[f"run/{name}.ppm"] == handoff["screenshots"][name]["ppm_sha256"]
                and member_hashes[f"frames/{name}.png"] == handoff["screenshots"][name]["png_sha256"],
                "Screenshot handoff binding mismatch")
    # Recheck source and canonical bytes after replay: no producer output is written.
    require(common.source_hash() == SOURCE, "Source changed during verification")
    for name, expected in member_hashes.items():
        if name.startswith("run/"):
            require(digest((directory / name.removeprefix("run/")).read_bytes()) == expected,
                    "Canonical artifact changed during verification")
    print(json.dumps({
        "schema_version": 1,
        "scope": "Independent read-only replay of producer run; no new VM or human acceptance",
        "passed": True,
        "source_commit": COMMIT,
        "source_sha256": SOURCE,
        "archive_sha256": ARCHIVE,
        "member_manifest_sha256": MEMBERS,
        "handoff_sha256": digest(handoff_raw),
        "verified_members": member_hashes,
        "verifier_tool_hashes": tool_hashes,
        "run_directory": str(directory),
        "profile": report["profile"],
        "command": report["command"],
        "frozen_artifacts": report["frozen_artifacts"],
        "qemu_version": report["qemu_version"],
        "qemu_sha256": report["qemu_implementation_sha256"],
        "producer_duration_s": report["duration_s"],
        "producer_raw_exit": report["raw_qemu_exit"],
        "qmp_operations": len(trace),
        "negative_replay_checks_rejected": rejected,
        "stages": report["stages"],
        "native_final": observed["native"]["final"],
        "frames": observed["frames"],
        "disk": observed["disk"],
        "before_sha256": report["before_sha256"],
        "after_sha256": report["after_sha256"],
        "new_vm_executed": False,
        "full_fr_acceptance": False,
        "human_review": False,
    }, indent=2))


if __name__ == "__main__":
    main()
