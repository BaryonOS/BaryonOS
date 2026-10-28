"""Run real QEMU counterexamples in a new disposable Git worktree.

Two malformed clients and a retrying service are real guest mutations. Hiding
the receive observation and inserting a publication count are explicitly
observer instrumentation, not claims of unobserved client behavior/device I/O.
The main checkout, its source fingerprint, and its images remain untouched.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import uuid

MAIN = Path(__file__).resolve().parents[1]
KERNEL = "kernel/src/m2/m3/m6.rs"
PROBE = "userspace/m6_probe.rs"
SERVICE = "services/block/src/m6_user_service.rs"
MUTANTS = {
    "skip-full": (PROBE,
        "if case==3 {let m=request(&conn,5,4,wire::PUT,Some(&current));negative(&mut conn,1,m,wire::Status::Full);}",
        "if false {let m=request(&conn,5,4,wire::PUT,Some(&current));negative(&mut conn,1,m,wire::Status::Full);}",
        "full", "assert_eq!((r.bad_requests, r.bad_replies, r.bad_received), (1, 1, 1));", "guest-client"),
    "repeat-error-id": (PROBE, "for fault in 0..15 {", "for _ in 0..15 { let fault=0;",
        "service-errors", 'assert_eq!(self.bad_masks[phase] & bit, 0,', "guest-client"),
    "retry-after-unknown": (SERVICE,
        "uncertain=true;status=Status::Unknown;rt::put(232,(-16i64) as u64);",
        "uncertain=true;status=Status::Unknown;rt::put(232,(-16i64) as u64); let _ = flush(cap);",
        "write-error", 'assert!(\n            !matches!(n, 0x42 | 0x43)', "guest-service-extra-flush-syscall"),
    "hide-unknown-receive": (KERNEL, "r.unknown_received += 1;",
        "// Test instrumentation: omit only the receive observation.",
        "flush-error", "assert_eq!((r.unknown_sent, r.unknown_received), (1, 1));", "observer-instrumentation"),
    "prepared-write-count": (KERNEL,
        "let publications = self.mutating_publications();",
        "if case() == 11 { self.publications[1] += 1; } // Instrumentation, no actual device I/O.\n        let publications = self.mutating_publications();",
        "bad-descriptor", "assert_eq!(r.mutating_publications(), expected);", "observer-instrumentation"),
    "post-unknown-count": (KERNEL, "r.check_unknown_publications();\n        let expected = match case()",
        "r.publications[2] += 1; // Instrumentation, no actual device I/O.\n        r.check_unknown_publications();\n        let expected = match case()",
        "write-error", 'Some(self.mutating_publications()),', "observer-instrumentation"),
}

def worker(worktree: Path, name: str, old: bool):
    sys.path.insert(0, str(worktree / "tools"))
    from build import build
    from run_m6 import run_m6
    relative, before, after, case, assertion, kind = MUTANTS[name]
    source = worktree / relative
    original = source.read_text(encoding="utf-8")
    if original.count(before) != 1:
        raise ValueError(f"Mutation anchor is not unique: {name}")
    source.write_text(original.replace(before, after), encoding="utf-8", newline="\n")
    kernel = (worktree / KERNEL).read_text(encoding="utf-8")
    # PanicInfo prints the assertion macro's first line; multi-line checks use
    # the preceding assert_eq!/assert! line rather than an inner expression.
    expected_line = None
    if not old:
        position = kernel.index(assertion)
        if name == "post-unknown-count":
            position = kernel.rfind("assert_eq!(", 0, position)
        expected_line = kernel[:position].count("\n") + 1
    build(kernel_feature=f"test-m6a-{case}", label=f"observer-{name}")
    fingerprints = sorted((worktree / "target/x86_64-unknown-none/release/.fingerprint").glob(
        "os-kernel-*/bin-os-kernel.json"), key=lambda path: path.stat().st_mtime)
    rustflags = json.loads(fingerprints[-1].read_text(encoding="utf-8"))["rustflags"]
    if rustflags.count("link-arg=-Tkernel/linker.ld") != 1:
        raise RuntimeError("Cargo inherited duplicate linker scripts; invalid experiment environment")
    report, _ = run_m6(case, label=f"observer-{name}")
    actual_lines = [int(line.split(":")[-1]) for line in report["stdout"].splitlines()
                    if line.startswith("M2:PANIC_LINE:")]
    if old:
        rejected = report["passed"] is True
    else:
        rejected = (report["passed"] is False and not report["timed_out"]
                    and report["raw_qemu_exit"] == 35
                    and "M1:FAIL:KERNEL_PANIC" in report["stdout"]
                    and "M6A:PASS" not in report["stdout"]
                    and actual_lines == [expected_line])
    result = {"name":name, "kind":kind, "old_observer":old,
              "expected":"old false-green reproduced" if old else "rejected at exact observer assertion",
              "check_passed":rejected, "expected_panic_line":expected_line,
              "actual_panic_lines":actual_lines,
              "mutation_file":relative, "mutated_file_sha256":hashlib.sha256(source.read_bytes()).hexdigest(),
              "rustflags":rustflags,
              "run":report}
    (worktree / "build/observer-result.json").write_text(json.dumps(result,indent=2),encoding="utf-8")
    if not rejected:
        raise RuntimeError(f"Observer counterexample outcome unexpected: {name}")

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--worker", type=Path)
    parser.add_argument("--name", choices=MUTANTS)
    parser.add_argument("--old", action="store_true")
    args = parser.parse_args()
    if args.worker:
        return worker(args.worker.resolve(strict=True), args.name, args.old)
    head = subprocess.check_output(["git","rev-parse","HEAD"],cwd=MAIN,text=True).strip()
    identifier = uuid.uuid4().hex[:10]
    directory = MAIN / "build/observer-checks" / identifier
    directory.mkdir(parents=True)
    # Cargo concatenates ancestor config arrays. A worktree nested in MAIN
    # would inherit .cargo/config.toml twice, including two linker scripts.
    worktree = Path(os.environ["LOCALAPPDATA"]) / "nieuw-os/observer-checks" / identifier / "checkout"
    worktree.parent.mkdir(parents=True)
    subprocess.run(["git","worktree","add","--detach",str(worktree),head],cwd=MAIN,check=True)
    report = {"base_commit":head,"script_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "scope":"separate disposable worktree; only synthetic QEMU disks; expected rejected guest runs stay passed:false", "status":"running","passed":False,"results":[]}
    destination = MAIN / "build/m6-observer-checks.json"
    sources = {p: subprocess.check_output(["git","show",f"{head}:{p}"],cwd=MAIN) for p in (KERNEL,PROBE,SERVICE)}
    try:
        for old, names in [(True, ("skip-full","repeat-error-id")), (False,tuple(MUTANTS))]:
            for name in names:
                for relative in sources:
                    data = (subprocess.check_output(["git","show",f"803bf43:{relative}"],cwd=MAIN)
                            if old else sources[relative])
                    (worktree / relative).write_bytes(data)
                label = ("old-" if old else "new-") + name
                command = [sys.executable,"-B","-X","utf8",str(Path(__file__).resolve()),"--worker",str(worktree),"--name",name]
                if old:
                    command.append("--old")
                with (directory / f"{label}.txt").open("w",encoding="utf-8",newline="\n") as log:
                    completed = subprocess.run(command,cwd=worktree,stdout=log,stderr=subprocess.STDOUT,timeout=240)
                entry = json.loads((worktree / "build/observer-result.json").read_text(encoding="utf-8"))
                entry["command"] = command
                entry["command_exit"] = completed.returncode
                report["results"].append(entry)
                destination.write_text(json.dumps(report,indent=2),encoding="utf-8")
                print(label,entry["check_passed"],"child exit",completed.returncode,flush=True)
                if completed.returncode or not entry["check_passed"]:
                    raise RuntimeError(label)
        report.update(status="completed",passed=True)
    except Exception as error:
        report.update(status="failed",passed=False,error=str(error))
        raise
    finally:
        report["worktree"] = str(worktree)
        destination.write_text(json.dumps(report,indent=2),encoding="utf-8")
        # Keep the disposable worktree and raw logs/images for audit. Never reset
        # or delete anything in the main checkout during this verification.

if __name__ == "__main__":
    main()
