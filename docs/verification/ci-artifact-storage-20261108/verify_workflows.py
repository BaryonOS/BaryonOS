#!/usr/bin/env python3
"""Read-only workflow review and local shell simulations; no GitHub uploads.

The original validation used a partial snapshot with a local baseline.
Use --baseline for that fixture; the default is the actual upstream source SHA. GitHub expression evaluation is inspected,
not emulated as an actual runner. Test files are created only in TemporaryDirectory.
"""

from __future__ import annotations

import argparse
import copy
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

import yaml


class UniqueLoader(yaml.BaseLoader):
    """Keep Actions' `on` as text and reject duplicate YAML mapping keys."""


def unique_mapping(loader, node):
    result = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node)
        if key in result:
            raise ValueError(f"duplicate key {key!r}, line {key_node.start_mark.line + 1}")
        result[key] = loader.construct_object(value_node)
    return result


UniqueLoader.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, unique_mapping)


def digest(value):
    return hashlib.sha256(value).hexdigest()


def shell(script, env, cwd):
    return subprocess.run(
        ["bash", "--noprofile", "--norc", "-eo", "pipefail", "-c", script],
        env=os.environ | env,
        cwd=cwd,
        text=True,
        capture_output=True,
        timeout=10,
        check=False,
    )


def run(repo, baseline):
    checks = []
    hashes = {}

    def record(name, passed, **detail):
        checks.append({"name": name, "passed": bool(passed), **detail})

    def read_yaml(path):
        data = path.read_bytes()
        hashes[str(path.relative_to(repo))] = digest(data)
        return yaml.load(data.decode(), Loader=UniqueLoader)

    workflows = {}
    for path in sorted((repo / ".github/workflows").glob("*.yml")):
        try:
            workflows[str(path.relative_to(repo))] = read_yaml(path)
        except Exception as exc:
            record("yaml:" + path.name, False, error=str(exc))
        else:
            record("yaml:" + path.name, True)

    action_path = repo / ".github/actions/preserve-evidence/action.yml"
    action = read_yaml(action_path)
    record("yaml:preserve-evidence-action", True)
    steps = action["runs"]["steps"]
    by_id = {step["id"]: step for step in steps if "id" in step}
    upload_steps = [step for step in steps if step.get("uses", "").startswith("actions/upload-artifact@")]
    record("only-uploads-continue-on-error", len(upload_steps) == 3 and all(
        (step.get("continue-on-error") == "true") == (step in upload_steps) for step in steps
    ))
    record("immutable-action-version", all(
        step["uses"] == "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02"
        for step in upload_steps
    ))
    record("retention", {name: by_id[name]["with"]["retention-days"] for name in ("reports", "full", "package")}
           == {"reports": "3", "full": "1", "package": "1"})
    record("full-archive-only-explicit-dispatch", all(fragment in by_id["full"]["if"] for fragment in (
        "github.event_name == 'workflow_dispatch'", "inputs.full-evidence == 'true'"
    )))
    record("package-check-gates-delivery", "steps.package-check.outputs.present == 'true'" in by_id["package"]["if"])
    record("input-env-no-shell-interpolation", all(
        "${{" not in step.get("run", "") for step in steps
    ))

    expected_extra_test = "python3 -B -m unittest discover -s tests -p 'test_ci_evidence*.py'\n"
    archive_workflows = []
    for path, after_original in workflows.items():
        old_text = subprocess.check_output(["git", "show", baseline + ":" + path], cwd=repo, text=True)
        before_original = yaml.load(old_text, Loader=UniqueLoader)
        before, after = copy.deepcopy(before_original), copy.deepcopy(after_original)
        before_uploads = [step for job in before["jobs"].values() for step in job["steps"]
                          if step.get("uses", "").startswith("actions/upload-artifact@")]
        after_preserve = [step for job in after["jobs"].values() for step in job["steps"]
                          if step.get("uses") == "./.github/actions/preserve-evidence"]
        if before_uploads:
            archive_workflows.append(path)
            record("replacement-count:" + path, len(before_uploads) == len(after_preserve))
            for old, new in zip(before_uploads, after_preserve):
                record("evidence-path-and-name:" + path,
                       old["with"]["path"] == new["with"]["path"]
                       and old["with"]["name"] == new["with"]["name"])
                policy = new["with"].get("if-no-files-found", action["inputs"]["if-no-files-found"]["default"])
                record("no-files-policy:" + path, old["with"].get("if-no-files-found", "warn") == policy)
            dispatch = after["on"].get("workflow_dispatch", {})
            record("manual-default-compact:" + path,
                   dispatch.get("inputs", {}).get("full_evidence", {}).get("default") == "false")
            before["on"].pop("workflow_dispatch", None)
            after["on"].pop("workflow_dispatch", None)
        if path in (".github/workflows/storage-resilience.yml", ".github/workflows/volume-acceptance.yml"):
            pattern = ".github/actions/preserve-evidence/**"
            current_paths = after["on"]["pull_request"]["paths"]
            record("shared-action-trigger:" + path, pattern in current_paths)
            current_paths.remove(pattern)
        for doc in (before, after):
            for job in doc["jobs"].values():
                job["steps"] = [step for step in job["steps"]
                                if not step.get("uses", "").startswith("actions/upload-artifact@")
                                and step.get("uses") != "./.github/actions/preserve-evidence"]
        if path == ".github/workflows/ci.yml":
            occurrences = 0
            for job in after["jobs"].values():
                for step in job["steps"]:
                    if "run" in step:
                        occurrences += step["run"].count(expected_extra_test)
                        step["run"] = step["run"].replace(expected_extra_test, "")
            record("collector-test-added-once", occurrences == 1)
        record("original-tests-runners-gates-preserved:" + path, before == after)
    record("fifteen-archive-workflows", len(archive_workflows) == 15)

    report_step = steps[-1]
    report_script = report_step["run"]
    cases = [
        ("success-ordinary", {}),
        ("quota-reports", {"REPORTS_OUTCOME": "failure"}),
        ("reports-skipped", {"REPORTS_OUTCOME": "skipped"}),
        ("reports-cancelled", {"REPORTS_OUTCOME": "cancelled"}),
        ("collector-failed", {"COLLECT_OUTCOME": "failure", "REPORTS_OUTCOME": "skipped"}),
        ("tests-failed", {"CI_TEST_STATUS": "failure"}),
        ("tests-cancelled", {"CI_TEST_STATUS": "cancelled"}),
        ("tests-skipped", {"CI_TEST_STATUS": "skipped"}),
        ("missing-status", {"CI_TEST_STATUS": ""}),
        ("invalid-status", {"CI_TEST_STATUS": "success\n::error::injected"}),
        ("full-success", {"FULL_REQUESTED": "true", "FULL_OUTCOME": "success"}),
        ("full-quota", {"FULL_REQUESTED": "true", "FULL_OUTCOME": "failure"}),
        ("full-skipped", {"FULL_REQUESTED": "true", "FULL_OUTCOME": "skipped"}),
        ("package-success", {"HAS_PACKAGE": "true"}),
        ("package-quota", {"HAS_PACKAGE": "true", "PACKAGE_OUTCOME": "failure"}),
        ("package-skipped", {"HAS_PACKAGE": "true", "PACKAGE_OUTCOME": "skipped"}),
        ("package-missing", {"HAS_PACKAGE": "true", "PACKAGE_PRESENT": ""}),
        ("package-unverified-tests", {"HAS_PACKAGE": "true", "CI_TEST_STATUS": "failure"}),
        ("package-unverified-cancelled", {"HAS_PACKAGE": "true", "CI_TEST_STATUS": "cancelled"}),
        ("package-missing-url", {"HAS_PACKAGE": "true", "PACKAGE_URL": ""}),
        ("package-invalid-url", {"HAS_PACKAGE": "true", "PACKAGE_URL": "file:///tmp/package.zip"}),
        ("reports-invalid-url", {"REPORTS_URL": "file:///tmp/report"}),
        ("reports-missing-url", {"REPORTS_URL": ""}),
        ("full-package-success", {"HAS_PACKAGE": "true", "FULL_REQUESTED": "true", "FULL_OUTCOME": "success"}),
        ("full-package-missing", {"HAS_PACKAGE": "true", "FULL_REQUESTED": "true", "FULL_OUTCOME": "success", "PACKAGE_PRESENT": ""}),
        ("full-package-quota", {"HAS_PACKAGE": "true", "FULL_REQUESTED": "true", "FULL_OUTCOME": "failure"}),
        ("full-package-tests-failed", {"HAS_PACKAGE": "true", "FULL_REQUESTED": "true", "FULL_OUTCOME": "success", "CI_TEST_STATUS": "failure"}),
        ("all-uploads-quota", {"HAS_PACKAGE": "true", "FULL_REQUESTED": "true", "REPORTS_OUTCOME": "failure", "FULL_OUTCOME": "failure", "PACKAGE_OUTCOME": "failure"}),
    ]
    base_env = {
        "CI_TEST_STATUS": "success", "COLLECT_OUTCOME": "success",
        "REPORTS_OUTCOME": "success", "FULL_OUTCOME": "skipped",
        "PACKAGE_OUTCOME": "success", "PACKAGE_PRESENT": "true",
        "HAS_PACKAGE": "false", "FULL_REQUESTED": "false",
        "REPORTS_URL": "https://github.com/example/repo/actions/runs/1/artifacts/10",
        "FULL_URL": "https://github.com/example/repo/actions/runs/1/artifacts/11",
        "PACKAGE_URL": "https://github.com/example/repo/actions/runs/1/artifacts/12",
    }
    with tempfile.TemporaryDirectory(prefix="ci-artifact-review-") as temp:
        temp_path = Path(temp)
        summary = temp_path / "summary.md"
        for name, overrides in cases:
            summary.write_text("")
            env = base_env | overrides | {"GITHUB_STEP_SUMMARY": str(summary)}
            result = shell(report_script, env, temp_path)
            text = summary.read_text()
            test_status = env["CI_TEST_STATUS"]
            if test_status not in ("success", "failure", "cancelled", "skipped"):
                test_status = "unavailable"
            package_key = "FULL" if env["FULL_REQUESTED"] == "true" else "PACKAGE"
            expected_package = (env["HAS_PACKAGE"] == "true" and test_status == "success"
                                and env["PACKAGE_PRESENT"] == "true"
                                and env[package_key + "_OUTCOME"] == "success"
                                and env[package_key + "_URL"].startswith("https://github.com/"))
            errors = []
            if result.returncode != 0:
                errors.append("report step failed")
            if f"Job status before archiving: `{test_status}`." not in text:
                errors.append("original test status lost")
            if ("Downloadable test package: available in " in text) != expected_package:
                errors.append("incorrect package availability")
            for key, label in (("REPORTS", "Compact reports"), ("FULL", "Full raw evidence")):
                expected_link = env[key + "_OUTCOME"] == "success" and env[key + "_URL"].startswith("https://github.com/")
                if (label + ": https://" in text) != expected_link:
                    errors.append("incorrect " + key + " link")
            if len(text.encode()) >= 1024 * 1024:
                errors.append("summary exceeds GitHub limit")
            record("report-shell:" + name, not errors, exit_code=result.returncode,
                   errors=errors, stdout=result.stdout, stderr=result.stderr)

        missing_step = next(step for step in steps if step["name"].startswith("Require evidence"))
        result = shell(missing_step["run"], {}, temp_path)
        record("required-evidence-missing-exit", result.returncode == 1,
               exit_code=result.returncode, stdout=result.stdout, stderr=result.stderr)
        record("required-evidence-policy-condition", all(fragment in missing_step["if"] for fragment in (
            "steps.collect.outcome == 'success'", "steps.collect.outputs.has-files != 'true'",
            "inputs.if-no-files-found == 'error'"
        )))

        fixture_root = temp_path / "workspace"
        fixture_root.mkdir()
        (fixture_root / "bundle.zip").write_bytes(b"test fixture; packaging contents are outside this check")
        (fixture_root / "directory.zip").mkdir()
        (temp_path / "outside.zip").write_bytes(b"outside")
        (fixture_root / "symlink.zip").symlink_to(temp_path / "outside.zip")
        (fixture_root / "inside-symlink.zip").symlink_to(fixture_root / "bundle.zip")
        output = temp_path / "github-output"
        for package_path, expected_ok in [
            ("bundle.zip", True), ("missing.zip", False), ("directory.zip", False),
            ("symlink.zip", False), ("inside-symlink.zip", False), ("../outside.zip", False),
        ]:
            output.write_text("")
            result = shell(by_id["package-check"]["run"], {
                "GITHUB_WORKSPACE": str(fixture_root), "EVIDENCE_PACKAGE": package_path,
                "GITHUB_OUTPUT": str(output),
            }, temp_path)
            actual_ok = result.returncode == 0 and output.read_text() == "present=true\n"
            record("package-check:" + package_path, actual_ok == expected_ok
                   and (expected_ok or result.returncode != 0), exit_code=result.returncode,
                   output=output.read_text(), stderr=result.stderr)

    baseline = subprocess.check_output(["git", "rev-parse", baseline], cwd=repo, text=True).strip()
    return {
        "schema_version": 1,
        "checked_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "scope": "Local YAML structure, preservation of baseline tests/runners/gates and executed report/package-check shell simulations; no GitHub uploads, runner-expression engine, Rust or QEMU execution.",
        "repository_path": str(repo),
        "local_snapshot_baseline": baseline,
        "upstream_snapshot_source": "363b486b06023d3d578f8eca2b23ff49045a1757",
        "file_sha256": hashes,
        "report_shell_sha256": digest(report_script.encode()),
        "checks": checks,
        "check_count": len(checks),
        "passed": all(check["passed"] for check in checks),
        "failed_checks": [check["name"] for check in checks if not check["passed"]],
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--baseline", default="363b486b06023d3d578f8eca2b23ff49045a1757")
    args = parser.parse_args()
    report = run(args.repo.resolve(), args.baseline)
    report["verifier_sha256"] = digest(Path(__file__).read_bytes())
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({key: report[key] for key in ("check_count", "passed", "failed_checks", "report_shell_sha256")}))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
