#!/usr/bin/env python3
"""Read-only workflow review and local shell simulations; no GitHub uploads.

The default baseline is the integrated hosted-runner/evidence configuration.
Use --baseline for a historical artifact-only comparison. A new or missing
workflow fails preservation; it is never silently accepted without a baseline.
GitHub expressions are inspected, not emulated as an actual runner. Test files
are created only in TemporaryDirectory.
"""

from __future__ import annotations

import argparse
import copy
import datetime as dt
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import tempfile

import yaml


DEFAULT_BASELINE = "338bec8ca34ba8eb2451eb660a0f1790c194efd2"


class UniqueLoader(yaml.BaseLoader):
    """Load text scalars, lists and mappings, rejecting duplicate mapping keys.

    BaseLoader preserves Actions' `on` and boolean spelling as text. It never
    constructs Python objects from YAML tags; this is not yaml.UnsafeLoader.
    """


def unique_mapping(loader, node):
    """Load a mapping without silently replacing an earlier duplicate key."""
    result = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node)
        if key in result:
            raise ValueError(f"duplicate key {key!r}, line {key_node.start_mark.line + 1}")
        result[key] = loader.construct_object(value_node)
    return result


UniqueLoader.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, unique_mapping)


def digest(value):
    """Bind recorded evidence to the exact supplied bytes."""
    return hashlib.sha256(value).hexdigest()


def shell(script, env, cwd):
    """Run a repository shell fragment with a ten-second fixture timeout."""
    return subprocess.run(
        ["bash", "--noprofile", "--norc", "-eo", "pipefail", "-c", script],
        env=os.environ | env,
        cwd=cwd,
        text=True,
        capture_output=True,
        timeout=10,
        check=False,
    )


def package_is_selected(package, selection):
    """Prove inclusion by an explicit file or directory, without glob guesses.

    The current package workflows use literal paths. Reject exclusions, hidden
    paths and traversal because a successful raw upload alone does not establish
    that the separately verified ZIP was included in that upload.
    """
    target = PurePosixPath(package)
    if (not package or target.is_absolute() or ".." in target.parts
            or any(part.startswith(".") for part in target.parts)
            or any(char in package for char in "*?[")):
        return False
    paths = [line.strip() for line in selection.splitlines() if line.strip()]
    if any(path.startswith("!") for path in paths):
        return False
    for path in paths:
        selected = PurePosixPath(path)
        if selected.is_absolute() or ".." in selected.parts or any(char in path for char in "*?["):
            continue
        if target == selected or (path.endswith("/") and target.is_relative_to(selected)):
            return True
    return False


def run(repo, baseline=DEFAULT_BASELINE):
    """Check workflow contracts and execute bounded local reporting fixtures."""
    checks = []
    hashes = {}
    comparisons = {}

    def record(name, passed, **detail):
        """Keep each assertion and its observed evidence in the JSON report."""
        checks.append({"name": name, "passed": bool(passed), **detail})

    def read_yaml(path):
        """Hash and parse a YAML file while preserving Actions scalar spelling."""
        data = path.read_bytes()
        hashes[str(path.relative_to(repo))] = digest(data)
        return yaml.load(data.decode(), Loader=UniqueLoader)

    resolved = subprocess.run(
        ["git", "rev-parse", "--verify", "--end-of-options", baseline + "^{commit}"],
        cwd=repo, text=True, capture_output=True, check=False,
    )
    baseline_sha = resolved.stdout.strip() if resolved.returncode == 0 else None
    record("baseline-resolves-to-commit", baseline_sha is not None,
           requested=baseline, error=resolved.stderr.strip())
    baseline_paths = set()
    if baseline_sha is not None:
        listing = subprocess.run(
            ["git", "ls-tree", "-r", "--name-only", baseline_sha, "--", ".github/workflows"],
            cwd=repo, text=True, capture_output=True, check=False,
        )
        record("baseline-workflow-inventory", listing.returncode == 0,
               error=listing.stderr.strip())
        if listing.returncode == 0:
            baseline_paths = {path for path in listing.stdout.splitlines()
                              if Path(path).suffix in (".yml", ".yaml")}

    workflows = {}
    for path in sorted((repo / ".github/workflows").glob("*.y*ml")):
        try:
            workflows[str(path.relative_to(repo))] = read_yaml(path)
        except Exception as exc:
            record("yaml:" + path.name, False, error=str(exc))
        else:
            record("yaml:" + path.name, True)
    record("workflow-inventory-preserved", baseline_paths == set(workflows),
           added=sorted(set(workflows) - baseline_paths),
           removed=sorted(baseline_paths - set(workflows)))

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
    record("default-required-evidence-policy", action["inputs"]["if-no-files-found"]["default"] == "error")
    record("input-env-no-shell-interpolation", all(
        "${{" not in step.get("run", "") for step in steps
    ))
    for name, package, selection, expected in [
        ("exact-file", "build/evidence/bundle.zip", "build/evidence/bundle.zip", True),
        ("parent-directory", "build/evidence/bundle.zip", "build/evidence/", True),
        ("absent-package", "build/evidence/bundle.zip", "build/result.json", False),
        ("similar-prefix", "build/evidence-other/bundle.zip", "build/evidence/", False),
        ("excluded-package", "build/evidence/bundle.zip", "build/evidence/\n!build/evidence/bundle.zip", False),
        ("hidden-package", "build/.private/bundle.zip", "build/", False),
        ("outside-workspace", "../bundle.zip", "../bundle.zip", False),
        ("unproven-glob", "build/evidence/bundle.zip", "build/**/*.zip", False),
    ]:
        record("package-selection-fixture:" + name,
               package_is_selected(package, selection) == expected)

    expected_extra_test = "python3 -B -m unittest discover -s tests -p 'test_ci_evidence*.py'\n"
    archive_workflows = []
    baseline_archive_workflows = []
    for path, after_original in workflows.items():
        after = copy.deepcopy(after_original)
        after_preserve = [step for job in after["jobs"].values() for step in job["steps"]
                          if step.get("uses") == "./.github/actions/preserve-evidence"]
        if after_preserve:
            archive_workflows.append(path)
            for new in after_preserve:
                record("preservation-invocation:" + path,
                       new.get("if") == "${{ always() }}"
                       and new.get("continue-on-error", "false") == "false"
                       and new["with"].get("job-status") == "${{ job.status }}")
                package = new["with"].get("package-path", "")
                if package:
                    selection = new["with"]["path"]
                    record("package-in-full-selection:" + path,
                           package_is_selected(package, selection), package_path=package)
                    without_package = "\n".join(
                        line for line in selection.splitlines()
                        if not package_is_selected(package, line)
                    )
                    record("removed-package-selection-rejected:" + path,
                           not package_is_selected(package, without_package))
            dispatch = after["on"].get("workflow_dispatch", {})
            record("manual-default-compact:" + path,
                   dispatch.get("inputs", {}).get("full_evidence", {}).get("default") == "false")

        if path not in baseline_paths:
            record("original-tests-runners-gates-preserved:" + path, False,
                   error="Workflow has no readable baseline; preservation is not proved.")
            comparisons[path] = {
                "status": "not-compared",
                "reason": "Workflow is absent from the selected baseline; preservation is not proved.",
            }
            continue
        old = subprocess.run(
            ["git", "show", baseline_sha + ":" + path],
            cwd=repo, text=True, capture_output=True, check=False,
        )
        try:
            if old.returncode != 0:
                raise ValueError(old.stderr.strip())
            before = yaml.load(old.stdout, Loader=UniqueLoader)
        except (ValueError, yaml.YAMLError) as exc:
            record("original-tests-runners-gates-preserved:" + path, False, error=str(exc))
            comparisons[path] = {"status": "not-compared", "reason": str(exc)}
            continue
        before_evidence = [step for job in before["jobs"].values() for step in job["steps"]
                           if step.get("uses", "").startswith("actions/upload-artifact@")
                           or step.get("uses") == "./.github/actions/preserve-evidence"]
        before_uploads = [step for step in before_evidence
                          if step.get("uses", "").startswith("actions/upload-artifact@")]
        before_preserve = [step for step in before_evidence
                           if step.get("uses") == "./.github/actions/preserve-evidence"]
        if before_evidence:
            baseline_archive_workflows.append(path)
            record("replacement-count:" + path, len(before_evidence) == len(after_preserve))
            for original, new in zip(before_evidence, after_preserve):
                record("evidence-path-and-name:" + path,
                       original["with"]["path"] == new["with"]["path"]
                       and original["with"]["name"] == new["with"]["name"])
                old_default = "warn" if original in before_uploads else "error"
                policy = new["with"].get("if-no-files-found", action["inputs"]["if-no-files-found"]["default"])
                record("no-files-policy:" + path, original["with"].get("if-no-files-found", old_default) == policy)
                if original not in before_uploads:
                    record("existing-preservation-step-unchanged:" + path, original == new)
        if before_uploads:
            # Only the historical upload-to-collector migration adds these inputs.
            before["on"].pop("workflow_dispatch", None)
            after["on"].pop("workflow_dispatch", None)
        else:
            record("preservation-calls-unchanged:" + path, before_preserve == after_preserve)
        if path in (".github/workflows/storage-resilience.yml", ".github/workflows/volume-acceptance.yml"):
            pattern = ".github/actions/preserve-evidence/**"
            current_paths = after["on"]["pull_request"]["paths"]
            record("shared-action-trigger:" + path, pattern in current_paths)
            if pattern not in before["on"]["pull_request"]["paths"] and pattern in current_paths:
                current_paths.remove(pattern)
        if before_uploads:
            for doc in (before, after):
                for job in doc["jobs"].values():
                    job["steps"] = [step for step in job["steps"]
                                    if not step.get("uses", "").startswith("actions/upload-artifact@")
                                    and step.get("uses") != "./.github/actions/preserve-evidence"]
        if path == ".github/workflows/ci.yml":
            baseline_occurrences = sum(step.get("run", "").count(expected_extra_test)
                                       for job in before["jobs"].values() for step in job["steps"])
            occurrences = 0
            for job in after["jobs"].values():
                for step in job["steps"]:
                    if "run" in step:
                        occurrences += step["run"].count(expected_extra_test)
                        if baseline_occurrences == 0:
                            step["run"] = step["run"].replace(expected_extra_test, "")
            record("collector-test-present-once", occurrences == 1)
        record("original-tests-runners-gates-preserved:" + path, before == after)
        comparisons[path] = {"status": "compared", "remaining_configuration_preserved": before == after}
    record("archive-workflow-set-preserved", bool(baseline_archive_workflows)
           and set(baseline_archive_workflows) == set(archive_workflows),
           baseline=sorted(baseline_archive_workflows), current=sorted(archive_workflows))
    for path in sorted(baseline_paths - workflows.keys()):
        record("baseline-workflow-still-present:" + path, False)
        comparisons[path] = {"status": "missing-from-checkout"}

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

    return {
        "schema_version": 2,
        "checked_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "scope": "Local YAML and evidence contracts, preservation of baseline tests/runners/gates and executed report/package-check shell simulations; no GitHub uploads, runner-expression engine, Rust or QEMU execution. A missing baseline comparison fails preservation.",
        "repository_path": ".",
        "local_snapshot_baseline": baseline_sha,
        "requested_baseline": baseline,
        "baseline_comparisons": comparisons,
        "archive_workflows": archive_workflows,
        "archive_workflow_count": len(archive_workflows),
        "file_sha256": hashes,
        "report_shell_sha256": digest(report_script.encode()),
        "checks": checks,
        "check_count": len(checks),
        "passed": all(check["passed"] for check in checks),
        "failed_checks": [check["name"] for check in checks if not check["passed"]],
    }


def main():
    """Write the verification report and fail when any recorded check fails."""
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--baseline", default=DEFAULT_BASELINE,
                        help="Preservation baseline; defaults to the integrated hosted/evidence source.")
    args = parser.parse_args()
    report = run(args.repo.resolve(), args.baseline)
    report["verifier_sha256"] = digest(Path(__file__).read_bytes())
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({key: report[key] for key in ("check_count", "passed", "failed_checks", "report_shell_sha256")}))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
