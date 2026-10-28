#!/usr/bin/env python3
"""Capture G01 host checks and bare-metal compilation, never native execution."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent

def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main() -> int:
    paths = sorted(p for p in (ROOT / "sdk").rglob("*") if p.is_file() and "__pycache__" not in p.parts)
    paths += [ROOT / p for p in ("crates/capability-core/src/lib.rs", "crates/capability-core/src/tests.rs",
                                "services/catalog/src/lib.rs", "services/catalog/src/wire.rs",
                                "services/catalog/src/journal.rs", "tools/build_users.py", "tools/common.py")]
    before = {str(p.relative_to(ROOT)): digest(p) for p in paths}
    env = os.environ.copy()
    env.setdefault("CARGO_TARGET_DIR", "/tmp/nieuw-os-par-g01-final-target")
    rust_files = sorted(str(p.relative_to(ROOT)) for p in (ROOT / "sdk").rglob("*.rs"))
    commands = [
        ("host-tests", ["cargo", "test", "--offline", "--locked", "--manifest-path", "sdk/Cargo.toml", "-p", "nieuw-os-sdk-runtime"]),
        ("host-clippy", ["cargo", "clippy", "--offline", "--locked", "--manifest-path", "sdk/Cargo.toml", "-p", "nieuw-os-sdk-runtime", "--all-targets", "--", "-D", "warnings"]),
        ("rustfmt", ["rustfmt", "--edition", "2021", "--check", *rust_files]),
        ("native-runtime-clippy", ["cargo", "clippy", "--offline", "--locked", "--manifest-path", "sdk/Cargo.toml", "-p", "nieuw-os-sdk-runtime", "--target", "x86_64-unknown-none", "--", "-D", "warnings"]),
        ("native-example-clippy", ["cargo", "clippy", "--offline", "--locked", "--manifest-path", "sdk/Cargo.toml", "-p", "nieuw-os-catalog-read-example", "--features", "native", "--target", "x86_64-unknown-none", "--", "-D", "warnings"]),
        ("native-example-build", [sys.executable, "-B", "sdk/examples/catalog-read/build.py"]),
        ("diff-check", ["git", "diff", "--check"]),
    ]
    report = {"schema_version": 1, "task": "G01", "started_utc": datetime.now(timezone.utc).isoformat(),
              "source_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
              "base_commit": "1acf6e8cde2a63849dc49c948ad97ba7f2c39cad", "cwd": str(ROOT),
              "target_dir": env["CARGO_TARGET_DIR"], "source_sha256": before,
              "scope": "host tests, static checks, x86_64-unknown-none compilation; no VM run", "checks": []}
    report["tool_versions"] = {
        tool: subprocess.check_output(command, cwd=ROOT, env=env, text=True).strip()
        for tool, command in (("rustc", ["rustc", "-vV"]), ("cargo", ["cargo", "--version"]),
                              ("python", [sys.executable, "--version"]))
    }
    for label, command in commands:
        outcome = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, check=False, timeout=120)
        stdout, stderr = HERE / f"{label}.stdout.txt", HERE / f"{label}.stderr.txt"
        stdout.write_bytes(outcome.stdout)
        stderr.write_bytes(outcome.stderr)
        report["checks"].append({"name": label, "command": command, "exit_code": outcome.returncode,
                                  "stdout": stdout.name, "stdout_sha256": digest(stdout),
                                  "stderr": stderr.name, "stderr_sha256": digest(stderr)})
        print(f"{label}: exit {outcome.returncode}", flush=True)
    after = {str(p.relative_to(ROOT)): digest(p) for p in paths}
    report["sources_unchanged"] = before == after
    artifact = Path(env["CARGO_TARGET_DIR"]) / "x86_64-unknown-none/release/catalog-read"
    if artifact.is_file():
        sys.path.insert(0, str(ROOT / "tools"))
        from build_users import inspect_user
        report["artifact"] = {"path": str(artifact), "sha256": digest(artifact), "bytes": artifact.stat().st_size,
                              "elf_validation": inspect_user(artifact)}
    report["passed"] = (report["sources_unchanged"] and "artifact" in report
                        and all(c["exit_code"] == 0 for c in report["checks"]))
    report["completed_utc"] = datetime.now(timezone.utc).isoformat()
    (HERE / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1

if __name__ == "__main__":
    sys.exit(main())
