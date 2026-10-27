"""Reproduce only C06's pure-core checks, without changing the workspace lock."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[2]
OUTPUT = Path(__file__).resolve().parent / "verification"


def main() -> int:
    OUTPUT.mkdir(exist_ok=True)
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = str(ROOT / "build" / "m6-core-tests")
    cargo = [sys.executable, "tools/dev.py", "cargo", "+1.90.0"]
    commands = [
        ("fmt", cargo + ["fmt", "-p", "capability-core", "-p", "virtio-core", "--", "--check"]),
        ("host", cargo + ["test", "-p", "capability-core", "-p", "virtio-core", "--locked", "--offline", "--", "--test-threads=3"]),
        ("clippy", cargo + ["clippy", "-p", "capability-core", "-p", "virtio-core", "--all-targets", "--locked", "--offline", "--", "-D", "warnings"]),
        ("no_std", cargo + ["check", "-p", "capability-core", "-p", "virtio-core", "--target", "x86_64-unknown-none", "--locked", "--offline"]),
        ("diff", ["git", "diff", "--check", "--", "crates/capability-core", "crates/virtio-core"]),
    ]
    results = []
    for name, argv in commands:
        result = subprocess.run(argv, cwd=ROOT, env=env, stdout=subprocess.PIPE,
                                stderr=subprocess.STDOUT, encoding="utf-8", errors="replace")
        log = OUTPUT / f"m6-{name}.log"
        log.write_text(result.stdout, encoding="utf-8")
        results.append({"name": name, "argv": argv, "exit": result.returncode,
                        "log": str(log.relative_to(ROOT)),
                        "log_sha256": hashlib.sha256(log.read_bytes()).hexdigest()})
        print(f"{name}: exit {result.returncode}", flush=True)
        if result.returncode:
            print(result.stdout, flush=True)
    files = ["Cargo.toml", "README.md", "src/lib.rs", "src/tests.rs"]
    sources = [ROOT / "crates" / crate / name
               for crate in ("capability-core", "virtio-core") for name in files]
    sources += [Path(__file__).resolve(), Path(__file__).parent / "M6-API.md"]
    report = {
        "basis": "448974b1352493fba41158da08daf1ad1937d973",
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "utc": datetime.now(timezone.utc).isoformat(),
        "target_dir": env["CARGO_TARGET_DIR"], "commands": results,
        "sources": {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
                    for path in sources},
        "limits": "Pure host/no_std verification; no guest, DMA, FLUSH, crash/recovery or physical-powerloss proof.",
        "m5_test_adjustment": "WRITE32 is now known but absent from legacyroot29: Access; unknown128 remains Invalid. Both full-state atomicity assertions retained; root approved.",
        "earlier_diagnostics": [
            "First compile rejected a newly added Stats.objects test typo; corrected to existing Stats.endpoints.",
            "First host run: 29/30 capability tests passed; original bit32 unknown-mask assertion required the documented, approved contract update."
        ],
    }
    (OUTPUT / "m6-report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    return int(any(item["exit"] for item in results))


if __name__ == "__main__":
    raise SystemExit(main())
