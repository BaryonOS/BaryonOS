"""Recheck actual M6a raw evidence, image bytes and recovery links before copying."""
from __future__ import annotations
import json
from pathlib import Path
from common import ROOT, BUILD, save_json, sha256, source_hash
from m6_disk import Volume, synthetic_initial, verify_volume
from run_m6 import (BOOT, FAILURES, UNKNOWN, WINDOWS_CRASH_EXIT, normal_success,
                    oracle_policy, unique_markers)
from run import is_success
from build import M6A_CASES


def required_groups():
    """The frozen gate shape, including every real crash-to-recovery link."""
    result = [(name, None, None) for name in ("strict-development-inventory", "core-protocol-journal-host-tests",
              "external-volume-and-crash-harness-tests", "m6-user-artifact-tests")]
    for suffix in ("a", "b"):
        parent = f"m6a-normal-{suffix}"
        result += [(parent, "normal", None), (parent+"-cold-recovery", "recover", parent)]
    result += [("same-normal-binary-two-real-images", None, None)]
    for case in M6A_CASES[:17]:
        result += [(case, case, None)]
        if case in UNKNOWN or case == "full":
            result += [(case+"-cold-recovery", "recover", case)]
    for point in range(1, 7): result += [(f"service-cut-{point}", f"service-cut-{point}", None)]
    for point in range(1, 7):
        parent = f"vm-cut-{point}"
        result += [(parent, parent, None), (parent+"-cold-recovery", "recover", parent)]
    result += [("m1-regression-after-m6a", "m1", None)]
    return result


def validate_coverage(groups, runs):
    expected = required_groups()
    if [g["name"] for g in groups] != [name for name, _, _ in expected]:
        raise ValueError("Incomplete/reordered/duplicate M6a gate coverage")
    by_name = {group["name"]: group for group in groups}
    by_dir = {run["run_directory"]: run for run in runs}
    if len(runs) != 49 or len(by_dir) != 49:
        raise ValueError("M6a requires 49 distinct actual VM invocations")
    for name, case, parent in expected:
        if case is None: continue
        directory = str(inside_run(by_name[name]["evidence"]))
        report = by_dir[directory]
        if (report.get("case") if case != "m1" else "m1") != case:
            raise ValueError("M6a group did not run its exact required case")
        if parent is not None and report["previous_stage"] != str(inside_run(by_name[parent]["evidence"])):
            raise ValueError("Recovery belongs to a different mutator/crash stage")
    normal = [run for run in runs if run.get("case") == "normal"]
    if len(normal) != 2:
        raise ValueError("M6a requires two normal images")
    a, b = normal
    checks = {"same_kernel": a["artifact_sha256"]["esp/kernel.elf"] == b["artifact_sha256"]["esp/kernel.elf"],
              "same_loader": a["artifact_sha256"]["esp/EFI/BOOT/BOOTX64.EFI"] == b["artifact_sha256"]["esp/EFI/BOOT/BOOTX64.EFI"],
              "different_actual_initial_images": a["disk_oracle"]["sha256_initial"] != b["disk_oracle"]["sha256_initial"],
              "different_received_values": a["disk_oracle"]["observed"]["value_digest"] != b["disk_oracle"]["observed"]["value_digest"]}
    saved = read_json(ROOT/by_name["same-normal-binary-two-real-images"]["evidence"])
    if not all(checks.values()) or saved.get("checks") != checks or saved.get("passed") is not True:
        raise ValueError("Actual two-image comparison no longer matches its evidence")


def inside_run(path):
    path = Path(path).resolve(strict=True)
    path.relative_to((BUILD / "run").resolve())
    return path


def read_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def raw_checks(directory, report, frozen):
    if (inside_run(report["run_directory"]) != directory or report.get("passed") is not True
            or report["source_fingerprint_version"] != 3 or report["source_sha256"] != frozen):
        raise ValueError(f"M6a run/source binding failed: {directory}")
    for name, key in (("serial.txt", "stdout"), ("stderr.txt", "stderr")):
        if (directory/name).read_bytes().decode("utf-8", "replace") != report[key]:
            raise ValueError(f"Changed raw output: {directory/name}")
        if report.get("case") and sha256(directory/name) != report[name.split('.')[0] + "_sha256"]:
            raise ValueError(f"Changed raw byte hash: {directory/name}")
    for name, expected in report["artifact_sha256"].items():
        artifact = (directory/name).resolve(strict=True)
        artifact.relative_to(directory)
        if sha256(artifact) != expected:
            raise ValueError(f"Changed artifact: {artifact}")


def check_m6(directory, report, frozen):
    raw_checks(directory, report, frozen)
    case = report["case"]
    image = inside_run(report["volume_path"])
    metadata = read_json(image.parent / "volume.json")
    volume = Volume(image.parent, metadata["seed"], synthetic_initial(metadata["seed"], metadata["variant"]),
                    metadata["variant"], metadata["scratch"], metadata["identity"])
    expected_variant = case if case in ("torn-tail", "corrupt-all") else "normal"
    if (volume.guard() != image or case != "recover" and (
            volume.scratch != (case == "device-rights") or volume.variant != expected_variant)):
        raise ValueError("M6a image identity/variant/scratch scope mismatch")
    before = (directory/"test-disk.before.img").read_bytes()
    after = (directory/"test-disk.after.img").read_bytes()
    if image.read_bytes() != after:
        raise ValueError("Actual image changed after this stage")
    oracle = verify_volume(volume, report["stdout"], **oracle_policy(case, before))
    if not oracle["passed"] or oracle != report["disk_oracle"]:
        raise ValueError(f"External M6a image oracle failed: {directory}")
    manifest = read_json(directory/"build-manifest.json")
    feature = "milestone-m6a" if case == "normal" else f"test-m6a-{case}"
    if (manifest.get("kernel_feature") != feature or manifest.get("loader_feature") is not None
            or manifest["source_sha256"] != frozen
            or manifest["kernel"]["sha256"] != report["artifact_sha256"]["esp/kernel.elf"]
            or manifest["loader"]["sha256"] != report["artifact_sha256"]["esp/EFI/BOOT/BOOTX64.EFI"]):
        raise ValueError("M6a exact feature/source/artifact manifest mismatch")
    unknown = [line for line in report["stdout"].splitlines() if line.startswith("M6A:OUTCOME")]
    if unknown != (["M6A:OUTCOME:UNKNOWN"] if case in UNKNOWN else []):
        raise ValueError("M6a Unknown evidence mismatch")
    if case.startswith("vm-cut-"):
        point = int(case[-1]); crash = report["host_crash"]; raw = report["raw_qemu_exit"]
        if (report["timed_out"] or raw not in (WINDOWS_CRASH_EXIT, -9)
                or not crash["kill_requested"] or not crash["was_running_before_kill"]
                or crash["real_process_returncode"] != raw or crash["termination_code_expected"] != raw
                or crash["requested_cut"] != point or crash["cut_lines"] != [f"M6A:CUT:{point}"]
                or not unique_markers(report["stdout"], BOOT + (f"M6A:CUT:{point}",))
                or not is_success(raw, report["stdout"], expected_exit=raw,
                                  required_markers=BOOT + (f"M6A:CUT:{point}",),
                                  forbidden_markers=FAILURES + ("M6A:PASS", "M1:PASS"))):
            raise ValueError("Not a real bounded externally terminated crash cut")
    elif not normal_success(report["raw_qemu_exit"], report["stdout"], report["timed_out"], case):
        raise ValueError(f"M6a raw guest markers/exit failed: {directory}")
    if case == "recover":
        previous = inside_run(report["previous_stage"])
        original = read_json(previous/"result.json")
        if (original.get("passed") is not True or original.get("case") == "recover"
                or original["volume_path"] != report["volume_path"] or previous == directory
                or original["disk_oracle"]["identity"] != volume.identity
                or (previous/"test-disk.after.img").read_bytes() != before or before != after):
            raise ValueError("M6a recovery was not a new invocation on the exact previous image")
    elif before != volume.initial:
        raise ValueError("Initial mutator stage did not start from its fresh synthetic image")
    return oracle


def archive_destination(destination=None):
    root = (ROOT/"docs/verification").resolve()
    target = (ROOT/(destination or "docs/verification/m6a")).resolve()
    if target.parent != root or target == root:
        raise ValueError("Destination must be one direct child of docs/verification")
    if destination is not None and target.exists():
        raise FileExistsError("Use a new archive destination; existing evidence is preserved")
    return target


def capture(destination=None):
    dest = archive_destination(destination)
    suite = read_json(BUILD/"m6-test-results.json")
    frozen = source_hash()
    if suite.get("status") != "completed" or suite.get("passed") is not True or suite["source_sha256"] != frozen:
        raise ValueError("Only a complete passed M6a suite on unchanged current source can be captured")
    dest.mkdir(parents=True, exist_ok=True)
    runs = []
    for group in suite["results"]:
        if group.get("passed") is not True:
            raise ValueError("Failed M6a group cannot be relabeled")
        evidence = Path(group["evidence"])
        if not evidence.is_absolute(): evidence = ROOT/evidence
        if evidence.is_dir():
            evidence = inside_run(evidence)
            report = read_json(evidence/"result.json")
            if group["name"] == "m1-regression-after-m6a":
                raw_checks(evidence, report, frozen)
                if (not is_success(report["raw_qemu_exit"], report["stdout"], report["timed_out"])
                        or "M6A:" in report["stdout"] or "volume_path" in report):
                    raise ValueError("M1 baseline is not an ordinary M1 run")
            else:
                check_m6(evidence, report, frozen)
            runs.append(report)
        elif evidence.is_file():
            evidence.resolve().relative_to(BUILD.resolve())
            paths = [evidence]
            if evidence.name.endswith((".stdout.txt", ".stderr.txt")):
                stem = evidence.name.removesuffix(".stdout.txt").removesuffix(".stderr.txt")
                meta = evidence.with_name(stem+".json")
                if read_json(meta)["returncode"] != 0: raise ValueError("Failed host command")
                paths = [evidence.with_name(stem+suffix) for suffix in (".stdout.txt", ".json", ".stderr.txt")]
            for path in paths:
                (dest/path.name).write_bytes(path.read_bytes())
        else:
            raise ValueError(f"Missing evidence: {evidence}")
    validate_coverage(suite["results"], runs)
    save_json(dest/"suite.json", suite)
    save_json(dest/"guest-runs.json", runs)
    print(f"Captured M6a: {len(suite['results'])} groups, {len(runs)} actual guest runs; crash exits preserved")


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination")
    capture(parser.parse_args().destination)
