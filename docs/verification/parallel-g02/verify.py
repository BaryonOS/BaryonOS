#!/usr/bin/env python3
"""Capture the opt-in native SDK bridge build; never claim VM execution."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
BASE = "1acf6e8cde2a63849dc49c948ad97ba7f2c39cad"

def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main() -> int:
    target = Path(os.environ.get("G02_BUILD_DIR", "/tmp/nieuw-os-par-g02-proof"))
    target.mkdir(parents=True, exist_ok=True)
    rlib, elf = target / "libnieuw_os_sdk_runtime.rlib", target / "m6c_sdk_probe.elf"
    shared = ["rustc", "+1.90.0", "--edition=2021", "--target=x86_64-unknown-none",
              "-C", "panic=abort", "-C", "opt-level=2", "-C", "code-model=small",
              "-C", "relocation-model=static", "-C", "no-redzone=yes"]
    commands = [
        ("runtime-build", [*shared, "--crate-name=nieuw_os_sdk_runtime", "--crate-type=rlib", "sdk/runtime/src/lib.rs", "-o", str(rlib)]),
        ("probe-build", [*shared, "--crate-name=os_user_m6c_sdk_probe", "--crate-type=bin",
                         "-C", "linker=rust-lld", "-C", "link-arg=-Tuserspace/m6c_linker.ld",
                         "-C", "link-arg=--no-pie", "-C", "link-arg=--build-id=none",
                         "-C", "link-arg=-zmax-page-size=0x1000", "--emit=link", "--cfg", "nos_sdk_runtime",
                         "--extern", f"nieuw_os_sdk_runtime={rlib}", "-o", str(elf), "userspace/m6c_probe.rs"]),
        ("rustfmt", ["rustfmt", "--edition", "2021", "--config", "skip_children=true", "--check", "userspace/sdk_probe_runtime.rs"]),
        ("diff-check", ["git", "diff", "--check"]),
    ]
    sources = ["userspace/m6c_probe.rs", "userspace/m6_runtime.rs", "userspace/sdk_probe_runtime.rs",
               "userspace/m6c_linker.ld", "services/catalog/src/user_service.rs", "services/catalog/src/journal.rs",
               "services/catalog/src/wire.rs", "sdk/runtime/src/lib.rs", "sdk/runtime/src/message.rs", "sdk/runtime/src/native.rs"]
    report = {"schema_version": 1, "task": "G02", "cwd": str(ROOT), "base_commit": BASE,
              "source_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
              "scope": "native crossbuild and source preservation; no VM execution", "checks": [],
              "source_sha256": {path: digest(ROOT / path) for path in sources},
              "rustc": subprocess.check_output(["rustc", "-vV"], cwd=ROOT, text=True).strip()}
    for label, command in commands:
        result = subprocess.run(command, cwd=ROOT, capture_output=True, check=False, timeout=120)
        stdout, stderr = HERE / f"{label}.stdout.txt", HERE / f"{label}.stderr.txt"
        stdout.write_bytes(result.stdout)
        stderr.write_bytes(result.stderr)
        report["checks"].append({"name": label, "command": command, "exit_code": result.returncode,
                                 "stdout": stdout.name, "stdout_sha256": digest(stdout),
                                 "stderr": stderr.name, "stderr_sha256": digest(stderr)})
        print(f"{label}: exit {result.returncode}", flush=True)
    preserved = ["userspace/m6_runtime.rs", "userspace/m6c_linker.ld", "services/catalog/src/user_service.rs",
                 "services/catalog/src/journal.rs", "services/catalog/src/wire.rs"]
    report["preserved_from_base"] = {
        path: subprocess.check_output(["git", "show", f"{BASE}:{path}"], cwd=ROOT) == (ROOT / path).read_bytes()
        for path in preserved
    }
    probe = (ROOT / "userspace/m6c_probe.rs").read_text()
    probe = probe.replace('#[cfg(not(nos_sdk_runtime))]\n', '').replace(
        '#[cfg(nos_sdk_runtime)]\n#[path = "sdk_probe_runtime.rs"]\nmod rt;\n', '')
    report["probe_roles_unchanged"] = probe.encode() == subprocess.check_output(
        ["git", "show", f"{BASE}:userspace/m6c_probe.rs"], cwd=ROOT)
    report["sources_unchanged"] = report["source_sha256"] == {path: digest(ROOT / path) for path in sources}
    if elf.is_file() and rlib.is_file():
        sys.path.insert(0, str(ROOT / "tools"))
        from build_users import inspect_user
        report["artifacts"] = {
            "runtime": {"path": str(rlib), "sha256": digest(rlib)},
            "probe": {"path": str(elf), "sha256": digest(elf), "bytes": elf.stat().st_size,
                      "elf_validation": inspect_user(elf)},
        }
    report["passed"] = (all(c["exit_code"] == 0 for c in report["checks"])
                        and all(report["preserved_from_base"].values()) and report["probe_roles_unchanged"]
                        and report["sources_unchanged"] and "artifacts" in report)
    (HERE / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1

if __name__ == "__main__":
    sys.exit(main())
