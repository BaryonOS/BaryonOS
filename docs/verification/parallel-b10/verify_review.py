"""B10 read-only source/evidence binding; no test suite, VM or remote mutation."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
SOURCE = "dbd35b04d742b18d66b40fdd65d8a9b900656cac"
MAIN = "391f02e9394cb96f37096c995dabcb7d624ba36b"
PR14 = "1acf6e8cde2a63849dc49c948ad97ba7f2c39cad"


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def read(name):
    return json.loads((ROOT / name).read_text())


def show(ref, name):
    return subprocess.check_output(["git", "show", ref + ":" + name], cwd=ROOT)


source_files = [
    "docs/verification/m6b-errors-review/README.md", "tools/run_graphics.py",
    "kernel/src/graphics.rs", "crates/ui-core/src/lib.rs", "tests/test_graphics_harness.py",
    "tools/m6b_disk.py", "tests/test_m6b_disk.py", "drivers/virtio-blk/src/lib.rs",
    "kernel/src/m2/m3/m6b.rs", "userspace/m6b_probe.rs", "services/objects/src/user_service.rs",
]
source_bindings = {}
for name in source_files:
    raw = (ROOT / name).read_bytes()
    assert raw == show(SOURCE, name), name
    source_bindings[name] = digest(raw)

snapshot = read("docs/verification/parallel-b10/github-snapshot.json")
threads = snapshot["review_threads"]
assert len(threads) == 7
assert snapshot["pr14"]["head_sha"] == PR14
assert snapshot["pr14"]["base_sha"] == MAIN
assert snapshot["pr15"]["head_sha"] == SOURCE
assert snapshot["pr15"]["base_sha"] == PR14

main = read("docs/verification/parallel-b10/main-tree.json")
assert main["truncated"] is False
trees = read("docs/verification/parallel-b10/local-trees.json")["trees"]
trees["main"] = {entry["path"]: entry["mode"] + " blob " + entry["sha"] for entry in main["entries"]}
preservation = []
for base, head in (("main", "pr14"), ("pr14", "current"), ("main", "current")):
    before, after = trees[base], trees[head]
    deleted = sorted(set(before) - set(after))
    assert not deleted
    preservation.append(dict(base=base, head=head, base_files=len(before), head_files=len(after),
        unchanged=sum(before[name] == after.get(name) for name in before),
        changed=[dict(path=name, base_object=before[name], head_object=after[name])
                 for name in sorted(before.keys() & after.keys()) if before[name] != after[name]],
        deleted=deleted, added_count=len(set(after) - set(before))))

preserved_contracts = ["services/catalog/src/wire.rs", "services/catalog/src/journal.rs",
    "services/catalog/src/user_service.rs", "kernel/src/m2/m3/m6c.rs",
    "tools/capture_m6c.py", "tests/test_m6c_capture.py", "drivers/virtio-blk/src/lib.rs"]
for name in preserved_contracts:
    assert trees["main"][name] == trees["pr14"][name] == trees["current"][name], name

b04 = read("docs/verification/parallel-b04/green.json")
b04_binding = {}
for name in ("tools/m6b_disk.py", "tests/test_m6b_disk.py", "drivers/virtio-blk/src/lib.rs",
             "kernel/src/m2/m3/m6b.rs", "userspace/m6b_probe.rs", "services/objects/src/user_service.rs"):
    actual = digest((ROOT / name).read_bytes())
    assert actual == b04["sha256"][name], name
    b04_binding[name] = actual

m6b = read("docs/verification/parallel-final-m6b/capture.json")
assert m6b["passed"] is True and m6b["groups"] == 32 and m6b["vm_invocations"] == 28
m6b_runs = read("docs/verification/parallel-final-m6b/guest-runs.json")
native_timeouts = []
for report in m6b_runs:
    if report.get("case") not in ("write-timeout", "reset-timeout", "write-poll-limit"):
        continue
    assert report["passed"] and report["raw_qemu_exit"] == 33
    assert report["base_commit"] == "3e41bbf93822f7044c6020259f4a752925e1966d"
    assert report["disk_oracle"]["state"]["sequence"] == 0
    assert report["disk_oracle"]["observed"] is None
    assert report["stdout"].splitlines().count("M6B:OUTCOME:UNKNOWN") == 1
    assert "M6B:NO_REPLAY:PASS" in report["stdout"].splitlines()
    native_timeouts.append(dict(case=report["case"], source_commit=report["base_commit"],
        journal_sequence=report["disk_oracle"]["state"]["sequence"], run_directory=report["run_directory"],
        raw_qemu_exit=report["raw_qemu_exit"], stdout_sha256=digest(report["stdout"].encode())))
assert len(native_timeouts) == 3

b08 = read("docs/verification/parallel-followup-g0/execution.json")
g0 = read("docs/verification/parallel-followup-g0/result.json")
last = read("docs/verification/parallel-followup-g0/final-observation.json")
assert b08["passed"] is True and last["passed"] is True
assert b08["source_commit"] == g0["base_commit"] == SOURCE
assert b08["source_sha256"] == g0["source_sha256"] == last["source_sha256"]
assert g0["raw_qemu_exit"] == 33 and all(value is True for value in g0["checks"].values())
assert g0["vga_rom"]["expected_hash_checked"] is True
assert g0["vga_rom"]["expected_sha256"] == g0["artifacts"]["VGA.rom"]
assert g0["artifacts"]["a.ppm"] == g0["artifacts"]["after-backspace.ppm"] != g0["artifacts"]["ab.ppm"]
assert len(g0["step_changed_bounds"]) == 4 and all(g0["step_changed_bounds"])

evidence_names = [
    "docs/verification/parallel-b04/green.json", "docs/verification/parallel-b04/green-oracle.txt",
    "docs/verification/parallel-final-m6b/capture.json", "docs/verification/parallel-final-m6b/suite.json",
    "docs/verification/parallel-final-m6b/guest-runs.json", "docs/verification/parallel-b09/after.json",
    "docs/verification/parallel-b09/after.stderr.txt", "docs/verification/parallel-followup-g0/execution.json",
    "docs/verification/parallel-followup-g0/result.json", "docs/verification/parallel-followup-g0/final-observation.json",
    "docs/verification/parallel-followup-g0/capture.json",
]
report = dict(scope=__doc__, completed_utc=datetime.now(timezone.utc).isoformat(), source_commit=SOURCE,
    source_files_sha256=source_bindings, passed=True, blocking_findings=[], source_changes_required=[],
    github_threads_still_open=sum(not thread["is_resolved"] for thread in threads),
    current_pr14_still_requires_pr15_integration=snapshot["pr14"]["head_sha"] != SOURCE,
    ancestry={name: {key: value for key, value in read("docs/verification/parallel-b10/" + name + ".json").items()
              if key in ("base", "head", "status", "ahead_by", "behind_by", "merge_base_commit")}
              for name in ("main_to_pr14", "pr14_to_current")},
    comparison_api_filelist_limit_note="PR15 compare returns 300 of 623 changed paths; preservation uses complete local trees and untruncated main Git-tree instead.",
    preservation=preservation, preserved_existing_contracts=preserved_contracts,
    b04_unchanged_tested_sources=b04_binding, m6b_historical_native_timeout_observations=native_timeouts,
    m6b_native_sequence_one_observed=False, g0_current_run_directory=g0["run_directory"],
    evidence_sha256={name: digest((ROOT / name).read_bytes()) for name in evidence_names},
    newly_executed_vm_runs=0, newly_executed_test_suites=0, remote_mutations=0)
(OUT / "assessment.json").write_text(json.dumps(report, indent=2) + "\n")
print("PASS: seven threads mapped to current source; no source fixes required; no deleted base paths")
print("PR14 itself remains on its old head until the stacked PR15 integration; evidence publication/review still applies")
