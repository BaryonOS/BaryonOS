#!/usr/bin/env python3
"""Read-only recheck of archived K06 CI evidence. Never starts a guest.

Exit 0 requires a complete 74 x 50 matrix and its aggregate. --partial is
strictly diagnostic, always exits 1 and can never produce accepted=true.
Raw binaries/firmware are not included in the compact CI artifacts; their
recorded hashes are cross-checked, but their bytes cannot be rehashed here.
"""
from __future__ import annotations
import argparse
from collections import Counter, defaultdict
from datetime import datetime, timezone
import hashlib
import json
import math
from pathlib import Path
import re
import subprocess
import sys

SOAK = "m3-soak-four"
CODE_SHA = "874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7"
VARS_SHA = "5d2ac383371b408398accee7ec27c8c09ea5b74a0de0ceea6513388b15be5d1e"


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def repository_path(explicit):
    if explicit:
        root = Path(explicit).resolve(strict=True)
        require((root / "tools/run_smp.py").is_file(), "repository lacks SMP oracle")
        return root
    for root in Path(__file__).resolve().parents:
        if (root / "tools/run_smp.py").is_file():
            return root
    raise ValueError("supply --repository with the tested oracle checkout")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input-dir", required=True, type=Path)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--source-sha256", required=True)
    parser.add_argument("--repository", type=Path)
    parser.add_argument("--aggregate", type=Path)
    parser.add_argument("--partial", action="store_true")
    args = parser.parse_args()
    require(re.fullmatch(r"[0-9a-f]{40}", args.commit), "invalid expected commit")
    require(re.fullmatch(r"[0-9a-f]{64}", args.source_sha256), "invalid expected source hash")
    repo = repository_path(args.repository)
    sys.path.insert(0, str(repo / "tools"))
    import run_smp
    IPC_TIMEOUT_S = getattr(run_smp, "IPC_TIMEOUT_S", run_smp.GUEST_TIMEOUT_S)  # absent before the R3-5 addendum
    LOAD_TIMEOUT_S = getattr(run_smp, "LOAD_TIMEOUT_S", run_smp.GUEST_TIMEOUT_S)  # absent before ADR-082 D5
    require(len(run_smp.CASES) == 74, "oracle does not define the required 74 cases")
    oracle_hashes = {}
    for name in ("tools/run_smp.py", "tools/run.py", "tools/common.py", "tools/gate_contract.py"):
        frozen = subprocess.run(["git", "show", f"{args.commit}:{name}"], cwd=repo,
                                check=True, capture_output=True).stdout
        current = (repo / name).read_bytes()
        require(current == frozen, f"oracle dependency differs from tested commit: {name}")
        oracle_hashes[name] = digest(current)
    root = args.input_dir.resolve(strict=True)
    require(root.is_dir(), "artifact root must be a directory")
    all_errors = []
    failures = []
    shards = []
    observed = defaultdict(list)
    all_runs = set()
    all_starts = set()
    qemu_versions = set()
    counts = Counter()
    probe_checked = False

    def read(path):
        require(path.is_file() and not path.is_symlink(), f"not a regular file: {path}")
        require(path.resolve().is_relative_to(root), f"file escapes artifact root: {path}")
        return path.read_bytes()

    def load(path):
        return json.loads(read(path).decode("utf-8"))

    def validate_soak(text):
        lines = text.splitlines()
        load = re.findall(r"^SMP:LOAD:WORKERS=(\d+):ELAPSED_MS=(\d+)$", text, re.M)
        require(len(load) == 1 and int(load[0][0]) == 16 and int(load[0][1]) >= 600000,
                "soak lacks 16 workers for >=600000 guest milliseconds")
        cpus = re.findall(r"^SMP:M3:CPU=(\d+):USER_TICKS=(\d+):SWITCHES=(\d+)$", text, re.M)
        require(len(cpus) == 4 and {int(x[0]) for x in cpus} == set(range(4))
                and all(int(x[1]) > 0 and int(x[2]) >= 1000 for x in cpus),
                "soak lacks real work and >=1000 switches on each of four CPUs")
        fair = re.findall(r"^SMP:FAIR:CPU=(\d+):WAIT_MS=(\d+):DEFERRED=(\d+):LIMIT_MS=(\d+):LIMIT_TICKS=(\d+)$", text, re.M)
        require(len(fair) == 4 and {int(x[0]) for x in fair} == set(range(4))
                and all(int(x[1]) <= 400 and int(x[2]) <= 40 and x[3:] == ('400', '40') for x in fair),
                "soak fairness exceeds unchanged 400ms/40tick bounds or is incomplete")
        for marker in ("M4:CASE:QUOTAS:PASS", "M4:SURVIVOR_PROGRESS:PASS", "M4:CLEANUP:PASS",
                       "SMP:SOAK:K07:M4:PASS:CPUS=4", "SMP:SOAK:K07:QSIM:PASS:CPUS=4",
                       "SMP:IPC:PASS", "SMP:M3:CLEANUP:PASS", "SMP:M3:PASS:CPUS=4", "M1:PASS"):
            require(lines.count(marker) == 1, f"missing or duplicate soak marker: {marker}")
        for marker in ("BELL4096", "SIX_QUBIT", "CANCEL", "DEADLINE", "RESOURCE_LIMITS", "PROCESS_ISOLATION"):
            require(lines.count(f"QSIM:{marker}:PASS") == 1, f"missing or duplicate qsim proof: {marker}")
        for field in ("REMOTE_WAKEUPS", "RESCHEDULE_IPIS"):
            values = re.findall(rf"^SMP:IPC:{field}=(\d+)$", text, re.M)
            require(len(values) == 1 and int(values[0]) > 0, f"missing cross-CPU IPC {field}")
        return int(load[0][1])

    for report_path in sorted(root.rglob("smp-report.json")):
        shard_errors = []
        relative = report_path.relative_to(root).as_posix()
        summary = {"report": relative, "passed": False}
        try:
            report = load(report_path)
            reps = report.get("repetitions")
            require(isinstance(reps, dict) and len(reps) == 1, "report must contain exactly one case")
            case = next(iter(reps))
            require(case in run_smp.CASES, "unknown case")
            profile, cpus, threads, outcome = run_smp.CASES[case]
            artifact = report_path.relative_to(root).parts[0]
            probe = artifact in ("runner-environment-probe", "runner-environment-probe-reports")
            require(not probe or (case == "m4-quotas-four" and not probe_checked), "invalid/duplicate environment probe")
            if probe:
                probe_checked = True
            repeat = 1 if probe else 5 if case == SOAK else 50
            if not args.partial:
                expected_name = (r"smp-soak-[0-9]" if case == SOAK else re.escape("smp-case-" + case)) + "(?:-reports)?"
                require(probe or re.fullmatch(expected_name, artifact), "artifact name does not identify the expected unique case/shard")
            summary.update(case=case, artifact=artifact, environment_probe=probe,
                           requested=repeat, reported_passed=report.get("passed"))
            require(report.get("schema_version") == 1 and report.get("source_fingerprint_version") == 3,
                    "unsupported report schema/fingerprint version")
            require(report.get("source_sha256") == args.source_sha256, "report source hash differs")
            require(report.get("kernel_feature") == profile and report.get("repeat") == repeat,
                    "report profile/repeat differs")
            manifest_path = report_path.parent / "manifest.json"
            manifest_bytes = read(manifest_path)
            manifest = json.loads(manifest_bytes.decode("utf-8"))
            require(digest(manifest_bytes) == report.get("manifest_sha256"), "manifest bytes do not match report hash")
            require(manifest.get("schema_version") == 1 and manifest.get("source_fingerprint_version") == 3
                    and manifest.get("source_sha256") == args.source_sha256
                    and manifest.get("kernel_feature") == profile and manifest.get("loader_feature") is None,
                    "manifest profile/source mismatch")
            expected_artifacts = {"esp/kernel.elf": manifest["kernel"]["sha256"],
                                  "esp/EFI/BOOT/BOOTX64.EFI": manifest["loader"]["sha256"]}
            require(all(re.fullmatch(r"[0-9a-f]{64}", x) for x in expected_artifacts.values()),
                    "invalid recorded binary hashes")
            runs = report.get("cases")
            require(isinstance(runs, list) and len(runs) == repeat, "missing or extra guest records")
            require(all(type(x.get("iteration")) is int for x in runs)
                    and [x.get("iteration") for x in runs] == list(range(1, repeat + 1)),
                    "iterations are missing, duplicated or reordered")
            raw_result_paths = sorted((report_path.parent / "run").rglob("result.json"))
            require(len(raw_result_paths) == repeat, "raw result count differs from report")
            raw_used = set()
            verified = 0
            reported_passed = 0
            serial_hashes = []
            soak_durations = []
            for item in runs:
                iteration = item.get("iteration")
                local = []
                guest = item.get("guest", {})
                try:
                    require(item.get("case") == case and type(item.get("cpus")) is int and item.get("cpus") == cpus
                            and item.get("tcg_threads") == threads, "guest case/cpu/TCG mismatch")
                    require(isinstance(guest, dict) and guest, "guest record missing")
                    identity = guest.get("run_directory")
                    require(isinstance(identity, str) and identity and identity not in all_runs,
                            "missing or duplicate guest run identity")
                    all_runs.add(identity)
                    start = guest.get("started_utc")
                    require(isinstance(start, str) and start not in all_starts, "missing or duplicate guest start identity")
                    all_starts.add(start)
                    datetime.fromisoformat(start)
                    label = "smp-" + case + (f"-r{iteration:03d}-of-{repeat:03d}" if repeat > 1 else "")
                    require(guest.get("label") == label, "guest iteration label differs")
                    leaf = Path(identity).name
                    require(re.fullmatch(re.escape(label) + r"-[0-9a-f]{10}", leaf), "unexpected guest directory name")
                    run_dir = report_path.parent / "run" / leaf
                    result_path = run_dir / "result.json"
                    full = load(result_path)
                    if "stdout_sha256" in guest:
                        # Compact shard report: result.json holds the transcript bound by hash.
                        require(isinstance(full.get("stdout"), str)
                                and digest(full["stdout"].encode("utf-8")) == guest["stdout_sha256"]
                                and {k: v for k, v in full.items() if k != "stdout"}
                                == {k: v for k, v in guest.items() if k != "stdout_sha256"},
                                "raw result.json differs from the hash-bound guest record")
                        guest = full
                    require(full == guest, "raw result.json differs from embedded guest")
                    raw_used.add(result_path)
                    raw_serial, raw_stderr = read(run_dir / "serial.txt"), read(run_dir / "stderr.txt")
                    serial = raw_serial.decode("utf-8")
                    stderr = raw_stderr.decode("utf-8")
                    require(serial == guest.get("stdout") and stderr == guest.get("stderr"),
                            "raw serial/stderr bytes differ from embedded text")
                    serial_hashes.append({"iteration": iteration, "result_sha256": digest(read(result_path)),
                                          "serial_sha256": digest(raw_serial), "stderr_sha256": digest(raw_stderr)})
                    require(guest.get("base_commit") == args.commit
                            and guest.get("source_fingerprint_version") == 3
                            and guest.get("source_sha256") == args.source_sha256, "guest commit/source mismatch")
                    require(guest.get("required_markers") == list(run_smp.BOOT + run_smp.required_end(outcome))
                            and guest.get("forbidden_markers") == list(run_smp.FORBIDDEN)
                            and guest.get("expected_exit") == 33,
                            "stored harness acceptance contract differs")
                    if case == "duplicate-madt":
                        require(item.get("acpi_table_sha256") == digest(run_smp.duplicate_madt()),
                                "duplicate MADT fixture hash differs")
                    require(guest.get("artifact_binding_passed") is True
                            and guest.get("expected_artifact_sha256") == expected_artifacts
                            and all(guest.get("artifact_sha256", {}).get(k) == v for k, v in expected_artifacts.items()),
                            "guest binary hashes are not bound to manifest")
                    require(guest.get("artifact_sha256", {}).get("OVMF_CODE.fd") == CODE_SHA
                            and guest.get("artifact_sha256", {}).get("OVMF_VARS.template.fd") == VARS_SHA,
                            "firmware hashes differ from pinned Ubuntu26.04 profile")
                    version = guest.get("qemu_version")
                    require(isinstance(version, str) and version.startswith("QEMU emulator version 10.2.1 "),
                            "QEMU version differs from pinned 10.2.1")
                    qemu_versions.add(version)
                    command = guest.get("command")
                    require(isinstance(command, list), "guest command missing")
                    def option(key):
                        require(command.count(key) == 1, f"missing or duplicate QEMU option {key}")
                        return command[command.index(key) + 1]
                    for key, value in (("-machine", "pc-q35-10.0"), ("-cpu", "qemu64"),
                                       ("-accel", f"tcg,thread={threads}"), ("-smp", str(cpus)),
                                       ("-m", "256M"), ("-nic", "none")):
                        require(option(key) == value, f"QEMU {key} differs")
                    deadline = run_smp.SOAK_TIMEOUT_S if case == SOAK else {
                        "m3-ipc": IPC_TIMEOUT_S, "m3-load": LOAD_TIMEOUT_S}.get(outcome, run_smp.GUEST_TIMEOUT_S)
                    duration = guest.get("duration_s")
                    require(guest.get("timed_out") is False and guest.get("timeout_s") == deadline
                            and type(duration) in (int, float) and math.isfinite(duration)
                            and 0 <= duration <= deadline, "guest timeout/watchdog failure")
                    watchdog = item.get("host_watchdog", {})
                    require(watchdog.get("deadline_s") == deadline and watchdog.get("measured_s") == duration
                            and watchdog.get("remaining_s") == deadline - duration
                            and watchdog.get("measured_over_deadline") == duration / deadline,
                            "host watchdog accounting differs")
                    # Re-run the exact strict serial oracle over every archived guest.
                    observed_now = run_smp.evaluate(case, serial, guest.get("raw_qemu_exit"), guest.get("timed_out"))
                    require(observed_now == item.get("observed"), "recomputed oracle evidence differs")
                    if case == SOAK:
                        soak_durations.append(validate_soak(serial))
                    require(item.get("passed") is True and guest.get("passed") is True,
                            "guest or case is marked failed")
                except (ValueError, KeyError, TypeError, IndexError, OSError, UnicodeError) as exc:
                    local.append(str(exc))
                reported_passed += item.get("passed") is True
                if local:
                    shard_errors.extend(f"iteration {iteration}: {message}" for message in local)
                    failure_markers = [x for x in guest.get("stdout", "").splitlines()
                                       if x.startswith(("SMP:CLOCK:DRIFT:", "SMP:CLOCK:SAMPLE:", "M1:FAIL:"))]
                    failures.append({"case": case, "iteration": iteration,
                                     "raw_exit": guest.get("raw_qemu_exit"), "errors": local,
                                     "failure_markers": failure_markers})
                else:
                    verified += 1
            require(set(raw_result_paths) == raw_used, "raw results contain unreferenced or duplicate guests")
            rep = reps[case]
            require(rep.get("requested") == repeat and rep.get("attempted") == repeat
                    and rep.get("passed") == reported_passed and rep.get("failed") == repeat - reported_passed
                    and rep.get("complete") is True
                    and rep.get("flaky") is (0 < reported_passed < repeat), "stored repeat summary is inconsistent")
            if report.get("passed") is not True:
                shard_errors.append("report is marked failed")
            require(report.get("passed") is (reported_passed == repeat), "report pass flag contradicts run records")
            summary.update(attempted=len(runs), reported_passed_runs=reported_passed,
                           independently_verified_passed_runs=verified,
                           manifest_sha256=digest(manifest_bytes), report_sha256=digest(read(report_path)),
                           raw_evidence_sha256=digest(json.dumps(serial_hashes, sort_keys=True).encode()),
                           soak_elapsed_ms=soak_durations)
            if not probe:
                observed[case].append(summary)
                counts[case] += verified
        except (ValueError, KeyError, TypeError, IndexError, OSError, UnicodeError) as exc:
            shard_errors.append(str(exc))
        summary["errors"] = shard_errors
        summary["passed"] = not shard_errors
        shards.append(summary)
        all_errors.extend(f"{relative}: {message}" for message in shard_errors)

    require(shards, "no SMP reports found")
    missing_cases = sorted(set(run_smp.CASES) - set(observed))
    for case in run_smp.CASES:
        found = observed[case]
        need = 10 if case == SOAK else 1
        if len(found) != need or counts[case] != 50:
            all_errors.append(f"{case}: expected {need} shards and 50 verified runs; found {len(found)} shards/{counts[case]} verified runs")
    if len({x.get("artifact") for x in observed[SOAK]}) != len(observed[SOAK]):
        all_errors.append("soak shard identities are duplicated")
    matrix_run_count = sum(x.get("attempted", 0) for x in shards if not x.get("environment_probe"))
    if matrix_run_count != 3700:
        all_errors.append(f"matrix guest run count {matrix_run_count}, expected 3700")
    aggregate_path = args.aggregate
    if aggregate_path is None:
        candidates = list(root.glob("smp-section9-matrix-report*/**/matrix-report.json"))
        aggregate_path = candidates[0] if len(candidates) == 1 else None
    aggregate_ok = False
    if aggregate_path is not None:
        try:
            aggregate = load(aggregate_path.resolve())
            require(aggregate.get("passed") is True and aggregate.get("case_count") == 74
                    and aggregate.get("guest_run_count") == 3700 and aggregate.get("repetitions_per_case") == 50
                    and aggregate.get("source_sha256") == args.source_sha256, "aggregate totals/source/pass mismatch")
            require(set(aggregate.get("cases", {})) == set(run_smp.CASES), "aggregate case set differs")
            require(all(x.get("attempted") == 50 and x.get("passed") == 50 for x in aggregate["cases"].values()),
                    "aggregate contains a case below 50/50")
            aggregate_ok = True
        except (ValueError, KeyError, TypeError, OSError, UnicodeError) as exc:
            all_errors.append(f"aggregate: {exc}")
    else:
        all_errors.append("aggregate artifact is missing")
    if args.partial:
        all_errors.append("partial diagnostic mode cannot accept the matrix")
    result = {
        "schema": 1, "generated_at": datetime.now(timezone.utc).isoformat(),
        "accepted": not all_errors and aggregate_ok,
        "mode": "partial-diagnostic" if args.partial else "complete-matrix",
        "expected_commit": args.commit, "expected_source_sha256": args.source_sha256,
        "oracle_dependency_sha256": oracle_hashes,
        "verifier_sha256": digest(Path(__file__).read_bytes()),
        "reports_checked": len(shards), "matrix_guest_runs_checked": matrix_run_count,
        "independently_verified_passed_runs": sum(counts.values()),
        "aggregate_verified": aggregate_ok, "environment_probe_checked": probe_checked,
        "case_verified_counts": dict(sorted(counts.items())), "missing_cases": missing_cases,
        "qemu_versions": sorted(qemu_versions), "shards": shards, "failed_runs": failures,
        "errors": all_errors,
        "limits": ["No guest is executed by this verifier.",
                   "The strict serial oracle is rerun from the selected checkout; this is an independent artifact readback, not an independent oracle implementation.",
                   "Compact artifacts omit kernel/loader ELF/PE and firmware bytes; recorded hashes are cross-bound to manifests, not independently recomputed from absent binaries.",
                   "GitHub artifact ZIP digests and run metadata must be checked separately against the retained API inventory.",
                   "Final tools/gate.sh execution and all required regression reports must be verified separately before accepting K06.",
                   "No physical hardware or human security audit evidence is provided."],
    }
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0 if result["accepted"] else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, OSError, KeyError, TypeError, subprocess.CalledProcessError) as exc:
        print(json.dumps({"accepted": False, "error": str(exc)}))
        raise SystemExit(1)
