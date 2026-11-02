#!/usr/bin/env python3
"""Host/scalar checks only. Never starts an OS, accesses a disk or claims GUI acceptance."""
from __future__ import annotations
import contextlib
import hashlib
import io
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = ROOT / "ui/native"
sys.path.insert(0, str(ROOT / "tools"))
from common import BUILD, run_logged, source_hash  # noqa: E402


def fingerprint():
    files = sorted(
        p for p in PACKAGE.rglob("*")
        if p.is_file() and not {"verification", "target", "build", "__pycache__"}.intersection(p.relative_to(PACKAGE).parts)
    ) + [ROOT / "ui/prototype/tokens.css"]
    hashes = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
    # Bind the frontend files and CSS token input explicitly alongside common
    # source v3, which also covers the native UI and surface transport.
    inputs = {"common_source_v3": source_hash(), "frontend_files": hashes}
    digest = hashlib.sha256(json.dumps(inputs, sort_keys=True).encode()).hexdigest()
    return digest, inputs


def main():
    before, inputs = fingerprint()
    host_env = {"CARGO_TARGET_DIR": str(BUILD / "native-materiaal-target")}
    scalar_env = {"CARGO_TARGET_DIR": str(BUILD / "native-materiaal-none-target"), "RUSTFLAGS": "-C no-redzone=yes -C target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2"}
    commands = [
        ("host", ["cargo", "+1.90.0", "test", "--offline", "--locked", "--manifest-path", "ui/native/Cargo.toml"], host_env),
        ("clippy", ["cargo", "+1.90.0", "clippy", "--offline", "--locked", "--manifest-path", "ui/native/Cargo.toml", "--all-targets", "--", "-D", "warnings"], host_env),
        ("fmt-check", ["cargo", "+1.90.0", "fmt", "--manifest-path", "ui/native/Cargo.toml", "--", "--check"], host_env),
        ("features", ["cargo", "+1.90.0", "tree", "--offline", "--locked", "--manifest-path", "ui/native/Cargo.toml", "-e", "features"], host_env),
        ("compiler", ["rustc", "+1.90.0", "-vV"], host_env),
        ("scalar-build", ["cargo", "+1.90.0", "build", "--offline", "--locked", "--manifest-path", "ui/native/Cargo.toml", "--release", "--lib", "--target", "x86_64-unknown-none", "--message-format=json-render-diagnostics"], scalar_env),
    ]
    results = {}
    for name, command, env in commands:
        with contextlib.redirect_stdout(io.StringIO()):
            results[name] = run_logged(command, "native-materiaal/" + name, cwd=ROOT, extra_env=env, timeout=120)
        print(name + ": exit 0")
    artifacts = []
    for line in results["scalar-build"].stdout.splitlines():
        event = json.loads(line)
        if event.get("reason") != "compiler-artifact":
            continue
        paths = [Path(p) for p in event["filenames"] if p.endswith(".rlib")]
        if len(paths) != 1:
            raise RuntimeError("unexpected compiler artifact")
        path = paths[0]
        with contextlib.redirect_stdout(io.StringIO()):
            dump = run_logged(["objdump", "-d", "--no-show-raw-insn", str(path)], "native-materiaal/disassembly-" + event["target"]["name"], cwd=ROOT)
            symbols = run_logged(["nm", "-u", str(path)], "native-materiaal/undefined-" + event["target"]["name"], cwd=ROOT)
        count = 0
        for code in dump.stdout.splitlines():
            match = re.match(r"^\s*[0-9a-f]+:\s*([a-z][a-z0-9.]*)\s*(.*)$", code)
            if not match:
                continue
            count += 1
            mnemonic, operands = match.groups()
            if re.search(r"%(?:[xyz]mm\d+|mm[0-7]|st(?:\([0-7]\))?)(?!\w)", operands) or mnemonic.startswith(("f", "v")) or mnemonic in {"emms", "wait", "ldmxcsr", "stmxcsr"}:
                raise RuntimeError("non-scalar object: " + code)
        for symbol in symbols.stdout.splitlines():
            if symbol.split()[-1:] and symbol.split()[-1] in {"malloc", "calloc", "realloc", "free", "mmap", "sbrk", "VirtualAlloc"}:
                raise RuntimeError("unexpected host memory backend")
        artifacts.append({"name": event["target"]["name"], "path": str(path.relative_to(ROOT)), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "bytes": path.stat().st_size, "instructions_checked": count, "forbidden_scalar_matches": [], "features": event["features"]})
    expected = {"zeroize", "ui_core", "catalog_core", "nieuw_os_sdk_runtime",
                "nieuw_os_catalog_client", "nieuw_os_input_core",
                "nieuw_os_document_session", "volume_core", "noto_sans_mono_bitmap",
                "nieuw_os_materiaal"}
    # The SDK already depends on zeroize. Check the exact locked graph rather
    # than the obsolete pre-zeroize count; every archive above is still scanned.
    if len(artifacts) != len(expected) or {item["name"] for item in artifacts} != expected:
        raise RuntimeError("unexpected locked dependency graph: " + repr([item["name"] for item in artifacts]))
    if fingerprint()[0] != before:
        raise RuntimeError("source changed during checks")
    output = PACKAGE / "verification"
    output.mkdir(exist_ok=True)
    logs = BUILD / "logs/native-materiaal"
    for name in results:
        metadata = json.loads((logs / (name + ".json")).read_text())
        text = "command: " + json.dumps(metadata["command"]) + "\nexit: 0\n\nstdout:\n" + results[name].stdout + "\nstderr:\n" + results[name].stderr
        (output / (name + ".txt")).write_text(text.rstrip() + "\n")
    report = {"task": "TASK-NATIVE-MATERIAAL", "base_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), "source_sha256": before, "inputs": inputs, "artifacts": artifacts, "rustflags": scalar_env["RUSTFLAGS"], "native_execution": False, "generic_controller_monomorphized_in_native_app": False, "screenreader": False, "physical_hardware": False}
    (output / "checks.json").write_text(json.dumps(report, indent=2) + "\n")
    print("source_sha256=" + before)
    print("artifacts=" + str(len(artifacts)) + "; native_execution=false")


if __name__ == "__main__":
    main()
