#!/usr/bin/env python3
"""Recompute the claims that are derivable from this committed evidence bundle."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import statistics
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--evidence-dir",
        type=Path,
        default=Path(__file__).resolve().parent,
        help="directory containing the four committed JSON evidence files",
    )
    return parser.parse_args()


def load(root: Path, name: str) -> dict:
    return json.loads((root / name).read_text(encoding="utf-8"))


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def timestamp(value: str) -> dt.datetime:
    return dt.datetime.fromisoformat(value.replace("Z", "+00:00"))


def percentile(values: list[float], quantile: float) -> float:
    ordered = sorted(values)
    position = (len(ordered) - 1) * quantile
    lower = int(position)
    upper = min(lower + 1, len(ordered) - 1)
    return ordered[lower] + (ordered[upper] - ordered[lower]) * (position - lower)


def peak(intervals: list[tuple[dt.datetime, dt.datetime]]) -> int:
    events = sorted(
        [(start, 1) for start, _ in intervals]
        + [(end, -1) for _, end in intervals]
    )
    current = maximum = 0
    for _, delta in events:
        current += delta
        maximum = max(maximum, current)
    return maximum


def running_per_minute(
    intervals: list[tuple[dt.datetime, dt.datetime]], start: str, end: str
) -> list[int]:
    point = timestamp(start)
    stop = timestamp(end)
    values: list[int] = []
    while point <= stop:
        values.append(sum(1 for left, right in intervals if left <= point < right))
        point += dt.timedelta(minutes=1)
    return values


def verify_outputs(root: Path, sources: dict) -> None:
    expected_outputs = {
        "capacity_burst": "queue-snapshot.json",
        "job_medians": "job-timing-snapshot.json",
        "verifier": "verify.py",
    }
    for key, name in expected_outputs.items():
        expected = sources["outputs"][key]
        path = root / name
        assert expected["file"] == name
        assert expected["bytes"] == path.stat().st_size
        assert expected["sha256"] == sha256(path)


def verify_queue(queue: dict, metrics: dict) -> dict:
    records = queue["job_records"]
    assert len(queue["run_attempts"]) == 99
    assert len(records) == 127
    assert queue["source_binding"]["response_count"] == 99
    assert queue["source_binding"]["retained_response_count"] == 96
    assert len(queue["source_binding"]["backfilled_responses"]) == 3

    non_matrix = [row["wait_seconds"] for row in records if not row["matrix_throttled"]]
    matrix = [row["wait_seconds"] for row in records if row["matrix_throttled"]]
    cutoff = timestamp(queue["snapshot_cutoff"])
    intervals = [
        (
            timestamp(row["started_at"]),
            min(
                timestamp(row["completed_at"]) if row["completed_at"] else cutoff,
                cutoff,
            ),
        )
        for row in records
    ]
    minute_counts = running_per_minute(
        intervals,
        queue["run_created_window"]["start"],
        queue["run_created_window"]["end"],
    )
    measured = {
        "runs": len(queue["run_attempts"]),
        "runner_jobs": len(records),
        "non_matrix_count": len(non_matrix),
        "non_matrix_median": percentile(non_matrix, 0.5),
        "non_matrix_p90": percentile(non_matrix, 0.9),
        "non_matrix_max": max(non_matrix),
        "matrix_count": len(matrix),
        "matrix_median": percentile(matrix, 0.5),
        "matrix_p90": percentile(matrix, 0.9),
        "matrix_max": max(matrix),
        "peak": peak(intervals),
        "per_minute_median": percentile(minute_counts, 0.5),
        "per_minute_p90": percentile(minute_counts, 0.9),
    }
    target = metrics["capacity_burst"]
    assert measured["runs"] == target["runs"]
    assert measured["runner_jobs"] == target["runner_jobs"]
    assert measured["non_matrix_count"] == target["non_matrix_wait_seconds"]["count"]
    assert measured["non_matrix_median"] == target["non_matrix_wait_seconds"]["median"]
    assert int(measured["non_matrix_p90"] + 0.5) == target["non_matrix_wait_seconds"]["p90"]
    assert measured["non_matrix_max"] == target["non_matrix_wait_seconds"]["max"]
    assert measured["matrix_count"] == target["matrix_limited_wait_seconds"]["count"]
    assert measured["matrix_median"] == target["matrix_limited_wait_seconds"]["median"]
    assert int(measured["matrix_p90"] + 0.5) == target["matrix_limited_wait_seconds"]["p90"]
    assert measured["matrix_max"] == target["matrix_limited_wait_seconds"]["max"]
    assert measured["peak"] == target["observed_peak_concurrent_jobs"] == 20
    assert measured["per_minute_median"] == target["running_jobs_per_minute"]["median"] == 18
    assert measured["per_minute_p90"] == target["running_jobs_per_minute"]["p90"] == 19
    return measured


def verify_job_medians(jobs: dict, metrics: dict) -> dict:
    records = jobs["job_records"]
    assert jobs["selection"]["record_count"] == len(records) == 35
    run_count = len({row["run_id"] for row in records})
    assert jobs["selection"]["run_count"] == run_count == 23
    mapping = {
        "CI / host-quality": "host_quality",
        "CI / qemu-smoke": "qemu_smoke",
        "CodeQL Advanced / Analyze (rust)": "codeql_rust",
        "Native SMP bring-up (K06 step 1) / native-smp": "native_smp",
        "FR-I01 joint native system acceptance / native-system": "native_system",
        "Native window session and Windows QEMU bundle / host-checks": "native_window_host_checks",
        "Native window session and Windows QEMU bundle / native-window-session (native-scale2)": "native_window_scale2",
        "Native window session and Windows QEMU bundle / native-window-session (switcher-scale2)": "native_window_switcher_scale2",
        "Merge hold / merge-hold": "merge_hold",
        "Merge hold re-check / recheck": "merge_hold_recheck",
    }
    for group, metric_key in mapping.items():
        workflow, job_name = group.rsplit(" / ", 1)
        selected = [
            row
            for row in records
            if row["workflow_name"] == workflow and row["job_name"] == job_name
        ]
        durations = [
            (timestamp(row["completed_at"]) - timestamp(row["started_at"])).total_seconds()
            for row in selected
        ]
        assert statistics.median(durations) == metrics["median_job_execution_seconds"][metric_key]
        assert jobs["recomputed_groups"][group]["job_ids"] == [row["job_id"] for row in selected]
    return {"records": len(records), "runs": run_count}


def verify_baseline(sources: dict, metrics: dict) -> dict:
    records = sources["historical_baseline_snapshot"]["records"]
    assert len(records) == 60
    measured = {
        "median_minutes_to_required_green": round(
            statistics.median(
                row["push_to_required_green_min"]
                for row in records
                if row["push_to_required_green_min"] is not None
            ),
            1,
        ),
        "mean_ci_rounds_per_pr": round(statistics.mean(row["rounds"] for row in records), 2),
        "mean_runner_minutes_per_pr": round(
            statistics.mean(row["runner_minutes"] for row in records), 1
        ),
        "mean_workflow_runs_per_pr": round(
            statistics.mean(row["workflow_runs"] for row in records), 1
        ),
        "mean_executed_jobs_per_pr": round(
            statistics.mean(row["jobs_executed"] for row in records), 1
        ),
        "cancelled_runner_minutes_percent": round(
            100
            * sum(row["cancelled_job_minutes"] for row in records)
            / sum(row["runner_minutes"] for row in records),
            1,
        ),
    }
    target = {
        key: value
        for key, value in metrics["historical_60_pr_reference"].items()
        if key not in {"period_start", "period_end"}
    }
    assert measured == target
    return measured


def verify_stale_green(sources: dict, metrics: dict) -> dict:
    stale = sources["stale_green_snapshot"]
    runs = stale["head_filtered_runs"]
    target = metrics["stale_green_observation"]
    assert runs
    assert all(
        row["head_sha"] == stale["head_sha"] and row["event"] == "pull_request"
        for row in runs
    )
    holds = [row for row in runs if row["name"] == "Merge hold"]
    passing_holds = [row for row in holds if row["conclusion"] == "success"]
    ci_attempts = [
        row
        for row in runs
        if row["name"] == "CI" and row["run_attempt"] == 2
    ]
    assert len(ci_attempts) == 1
    assert passing_holds
    ci_attempt = ci_attempts[0]
    latest_passing_hold = max(passing_holds, key=lambda row: timestamp(row["updated_at"]))
    measured = {
        "ci_attempt_two_completed_at": ci_attempt["updated_at"],
        "ci_attempt_two_started_at": ci_attempt["run_started_at"],
        "ci_run_attempt": ci_attempt["run_attempt"],
        "head_filtered_merge_hold_count": len(holds),
        "hold_success_predates_ci_attempt_two": timestamp(latest_passing_hold["updated_at"])
        < timestamp(ci_attempt["run_started_at"]),
        "merge_hold_latest_success_completed_at": latest_passing_hold["updated_at"],
        "merge_hold_success": latest_passing_hold["conclusion"] == "success",
        "newer_merge_hold_present": any(
            timestamp(row["run_started_at"]) > timestamp(ci_attempt["run_started_at"])
            for row in holds
        ),
    }
    assert measured == stale["recomputed_assertions"]
    selected = stale["selected_runs"]
    assert set(selected) == {str(ci_attempt["id"]), str(latest_passing_hold["id"])}
    assert selected[str(ci_attempt["id"])] == ci_attempt
    assert selected[str(latest_passing_hold["id"])] == latest_passing_hold
    assert stale["pull_request"] == target["pull_request"]
    assert stale["head_sha"] == target["head_sha"]
    assert {
        "run_id": latest_passing_hold["id"],
        "attempt": latest_passing_hold["run_attempt"],
        "conclusion": latest_passing_hold["conclusion"],
        "started_at": latest_passing_hold["run_started_at"],
        "completed_at": latest_passing_hold["updated_at"],
    } == target["merge_hold"]
    assert {
        "run_id": ci_attempt["id"],
        "attempt": ci_attempt["run_attempt"],
        "conclusion": ci_attempt["conclusion"],
        "started_at": ci_attempt["run_started_at"],
        "completed_at": ci_attempt["updated_at"],
    } == target["later_ci"]
    assert measured["head_filtered_merge_hold_count"] == 1
    assert measured["merge_hold_success"] is True
    assert measured["merge_hold_latest_success_completed_at"] == target["merge_hold"]["completed_at"]
    assert measured["ci_run_attempt"] == target["later_ci"]["attempt"]
    assert measured["ci_attempt_two_started_at"] == target["later_ci"]["started_at"]
    assert measured["ci_attempt_two_completed_at"] == target["later_ci"]["completed_at"]
    assert measured["hold_success_predates_ci_attempt_two"] is True
    assert measured["newer_merge_hold_present"] is False
    assert target["newer_merge_hold_count"] == 0
    return measured


def verify_capture_provenance(sources: dict, queue: dict) -> dict:
    capacity = sources["capacity_burst"]
    counterfactual = sources["main_branch_filter_counterfactual"]
    raw_capture = capacity["raw_runs_file"]
    shared_sources = [
        entry
        for entry in counterfactual["sources"]
        if entry["path"] == raw_capture["path"]
    ]
    assert shared_sources == [raw_capture]
    assert capacity["retained_capture_query"] == counterfactual["runs_query"]
    assert capacity["retained_capture_records"] == 100
    analysis_filter = capacity["analysis_filter"]
    assert analysis_filter["selected_records"] == len(queue["run_attempts"]) == 99
    assert analysis_filter["created_at_start"] == queue["run_created_window"]["start"]
    assert analysis_filter["created_at_end"] == queue["run_created_window"]["end"]
    window = counterfactual["window"]
    assert analysis_filter["created_at_start"] == window["start"]
    assert analysis_filter["created_at_end"] < window["end"]
    assert window["start"] in counterfactual["runs_query"]
    assert window["end"] in counterfactual["runs_query"]
    excluded = capacity["excluded_capture_record"]
    assert excluded["id"] == 45778270642
    assert analysis_filter["created_at_end"] < excluded["created_at"] <= window["end"]
    late_match = counterfactual["late_match"]
    assert late_match["recheck_run_id"] == excluded["id"]
    assert late_match["recheck_created_at"] == excluded["created_at"]
    assert counterfactual["late_match"]["source_run_id"] == 45775440667
    replay = counterfactual["replay_validation"]
    assert replay["id_set_matches_capture"] is True
    assert replay["records"] == capacity["retained_capture_records"]
    assert replay["maximum_created_at"] == excluded["created_at"]
    return {
        "capture_records": capacity["retained_capture_records"],
        "queue_records": analysis_filter["selected_records"],
        "late_recheck": late_match["recheck_run_id"],
        "shared_capture_sha256": raw_capture["sha256"],
    }


def main() -> None:
    if sys.flags.optimize != 0:
        raise RuntimeError("verification requires Python assertions; unset PYTHONOPTIMIZE")
    root = parse_args().evidence_dir.resolve()
    queue = load(root, "queue-snapshot.json")
    jobs = load(root, "job-timing-snapshot.json")
    sources = load(root, "sources.json")
    metrics = load(root, "metrics.json")

    assert sources["organization"] == "BaryonOS"
    assert sources["repository"] == "BaryonOS/BaryonOS"
    assert "/home/" not in json.dumps(sources)
    verify_outputs(root, sources)
    result = {
        "status": "clean",
        "queue": verify_queue(queue, metrics),
        "job_medians": verify_job_medians(jobs, metrics),
        "historical_baseline": verify_baseline(sources, metrics),
        "stale_green": verify_stale_green(sources, metrics),
        "capture_provenance": verify_capture_provenance(sources, queue),
        "scope_limit": (
            "PR 550, same-tree duplication and branch-filter counts remain bound "
            "to external raw-source hashes; this clean-checkout verifier does not "
            "recompute those three claims."
        ),
    }
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
