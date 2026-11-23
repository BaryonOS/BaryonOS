"""Summarise per-phase input-to-image intervals from passing window-session reports.

This is a descriptive baseline only. It reads report JSON files written by
tools/run_window_session.py, never runs a VM and never accepts a budget: the
output always carries ``accepted: null`` and ``budget: null``. The intervals are
host/QMP/observation/capture bounds, not guest or scanout latency, and TCG
figures are not the WHPX reference that U02 acceptance requires.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from window_latency import _quantile  # noqa: E402

FIELD = "last_input_to_verified_app_image_s"
# Every report must share these, so samples come from one source and profile.
IDENTITY = ("source_commit", "source_sha256", "source_fingerprint_version", "kernel_feature",
            "scale", "viewport", "qemu_sha256", "qemu_machine", "qemu_accel", "clipboard_requested", "workspace_requested",
            "switcher_requested", "library_requested", "session_route_requested")
SCOPE = ("TCG host-observed baseline, not U02 acceptance, no budget. Each interval "
         "runs from the start of the last injected QMP input to an exact full "
         "app-body oracle match and includes QMP, polling and capture overhead.")


def load(path: Path) -> tuple[dict, str]:
    raw = path.read_bytes()
    report = json.loads(raw)
    if type(report) is not dict:
        raise ValueError(f"{path}: report is not a JSON object")
    if report.get("passed") is not True or report.get("raw_qemu_exit") != 33 or "error" in report:
        raise ValueError(f"{path}: report did not pass (passed/raw exit 33/error)")
    if type(report.get("phases")) is not dict or not report["phases"]:
        raise ValueError(f"{path}: report has no verified phases")
    for name, phase in report["phases"].items():
        if type(phase) is not dict:
            raise ValueError(f"{path}: phase {name} is not a JSON object")
    duration = report.get("duration_s")
    if type(duration) not in (int, float) or not 0 <= duration < 86400:
        raise ValueError(f"{path}: invalid duration_s")
    command = report.get("command")
    if type(command) is not list or not report.get("source_fingerprint_version"):
        raise ValueError(f"{path}: report lacks its command or source fingerprint version")
    for option in ("-machine", "-accel"):
        if command.count(option) != 1 or command.index(option) + 1 >= len(command):
            raise ValueError(f"{path}: command must name exactly one {option}")
    # Derived from the recorded command so samples pool only within one QEMU profile.
    report["qemu_machine"] = command[command.index("-machine") + 1]
    report["qemu_accel"] = command[command.index("-accel") + 1]
    if type(report["qemu_machine"]) is not str or type(report["qemu_accel"]) is not str:
        raise ValueError(f"{path}: -machine and -accel values must be strings")
    if report["qemu_accel"].split(",")[0] != "tcg":
        raise ValueError(f"{path}: this baseline covers TCG only, not {report['qemu_accel']}")
    return report, hashlib.sha256(raw).hexdigest()


def summarise(paths: list[Path]) -> dict:
    if not paths:
        raise ValueError("At least one report is required")
    reports = [(path, *load(path)) for path in paths]
    first = reports[0][1]
    for key in ("source_commit", "source_sha256"):
        if not first.get(key):
            raise ValueError(f"{reports[0][0]}: {key} is missing; samples must bind to one source")
    for key in IDENTITY:
        if first.get(key) is None:
            raise ValueError(f"{reports[0][0]}: {key} is missing; samples must bind to one profile")
    for path, report, _ in reports[1:]:
        for key in IDENTITY:
            if report.get(key) != first.get(key):
                raise ValueError(f"{path}: {key} differs from {reports[0][0]}")
    for values in ([report.get("run_directory") for _, report, _ in reports],
                   [digest for _, _, digest in reports]):
        if len(set(values)) != len(values):
            raise ValueError("Duplicate report or run directory; every run must be a separate cold start")
    samples: dict[str, list[float]] = {}
    kinds: dict[str, set[str]] = {}
    for path, report, _ in reports:
        for name, phase in report["phases"].items():
            if FIELD not in phase:
                continue
            value = phase[FIELD]
            if type(value) not in (int, float) or not 0 <= value < 86400:
                raise ValueError(f"{path}: invalid {FIELD} for {name}")
            samples.setdefault(name, []).append(value)
            kinds.setdefault(name, set()).add(phase.get("last_input_kind", "unrecorded"))
    if not samples:
        raise ValueError("No phase carries " + FIELD)
    phases = {}
    for name, values in samples.items():
        if len(values) != len(reports):
            raise ValueError(f"{name}: present in {len(values)} of {len(reports)} reports")
        phases[name] = {"count": len(values), "min_s": min(values),
                        "p50_s": _quantile(values, 50), "p95_s": _quantile(values, 95),
                        "max_s": max(values), "last_input_kinds": sorted(kinds[name])}
    return {"kind": "baseline", "accepted": None, "budget": None, "scope": SCOPE,
            "identity": {key: first.get(key) for key in IDENTITY},
            "runs": [{"report": path.name, "sha256": digest,
                      # Base names only: no local home path enters committed evidence.
                      "run_directory": Path(str(report.get("run_directory"))).name,
                      "duration_s": report.get("duration_s")} for path, report, digest in reports],
            "phases": phases}


def text(summary: dict) -> str:
    lines = [summary["scope"], f"runs={len(summary['runs'])} accepted=null budget=null",
             "phase count min_ms p50_ms p95_ms max_ms last_input"]
    for name, row in summary["phases"].items():
        lines.append(" ".join([name, str(row["count"])] + [
            f"{row[key] * 1000:.0f}" for key in ("min_s", "p50_s", "p95_s", "max_s")]
            + [",".join(row["last_input_kinds"])]))
    return "\n".join(lines)


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reports", nargs="+", type=Path)
    parser.add_argument("--output", type=Path, help="Write the JSON summary here")
    args = parser.parse_args(argv)
    try:
        summary = summarise(args.reports)
    except (OSError, ValueError) as error:
        print("window_latency_baseline: " + str(error), file=sys.stderr)
        return 1
    if args.output:
        args.output.write_text(json.dumps(summary, indent=2, sort_keys=True, allow_nan=False) + "\n", encoding="utf-8")
    print(text(summary))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
