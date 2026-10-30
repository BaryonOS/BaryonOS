#!/usr/bin/env python3
"""Contract tests and actual scalar ELF checks; no VM/native product claim."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

EXAMPLE = Path(__file__).resolve().parent
ROOT = EXAMPLE.parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from common import BUILD, run_logged

def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

EXPECTED_HOST_SCENARIOS = 20

def host_scenario_count(path: Path) -> int:
    output = path.read_text(encoding="utf-8")
    runs = re.findall(r"^running (\d+) tests?$", output, re.MULTILINE)
    lines = [line for line in output.splitlines() if line.startswith("test result:")]
    results = [re.fullmatch(
        r"test result: ok\. (\d+) passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in \d+(?:\.\d+)?s",
        line,
    ) for line in lines]
    if not runs or len(runs) != len(results) or any(result is None for result in results):
        raise RuntimeError("invalid or incomplete Materiaal host test output")
    counts = [result[1] for result in results if result is not None]
    if runs != counts:
        raise RuntimeError("Materiaal host test counts do not match the executed suites")
    count = sum(map(int, counts))
    if count != EXPECTED_HOST_SCENARIOS:
        raise RuntimeError(f"expected {EXPECTED_HOST_SCENARIOS} Materiaal host scenarios, got {count}")
    return count

def main() -> int:
    # A successful old ABI build is insufficient: check_abi would reject the
    # actual desktop kernel. Never silently patch shared SDK sources here.
    runtime = (ROOT / "sdk/runtime/src/lib.rs").read_text()
    match = re.search(r"pub const ABI_MAX_MINOR: u64 = (\d+);", runtime)
    if not match or int(match[1]) < 8:
        raise RuntimeError("requires the centrally integrated SDK ABI1.8; do not claim a compatible old-ABI executable")
    manifest = str(EXAMPLE / "Cargo.toml")
    target = ROOT / "build/materiaal-app-host"
    extra = {"CARGO_TARGET_DIR": str(target)}
    logs = []
    for name, args in [
        ("fmt", ["cargo", "fmt", "--manifest-path", manifest, "--", "--check"]),
        ("host", ["cargo", "test", "--offline", "--locked", "--manifest-path", manifest]),
        ("clippy", ["cargo", "clippy", "--offline", "--locked", "--manifest-path", manifest, "--all-targets", "--", "-D", "warnings"]),
        ("native-clippy", ["cargo", "clippy", "--offline", "--locked", "--manifest-path", manifest, "--features", "native", "--bin", "materiaal-app", "--target", "x86_64-unknown-none", "--", "-D", "warnings"]),
        ("native-elf", [sys.executable, "-B", str(EXAMPLE / "build.py")]),
    ]:
        flags = "\x1f".join(["-C", "no-redzone=yes", "-C", "target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2"])
        selected = {**extra, "CARGO_ENCODED_RUSTFLAGS": flags} if name == "native-clippy" else extra
        run_logged(args, f"materiaal-app/{name}", extra_env=selected if name != "native-elf" else None)
        logs.append(name)
    scenario_count = host_scenario_count(BUILD / "logs/materiaal-app/host.stdout.txt")
    native_target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "build/sdk-materiaal"))
    binary = native_target / "x86_64-unknown-none/release/materiaal-app"
    run_logged(["cargo", "test", "--offline", "--locked", "--manifest-path", manifest,
                "--features", "verify-native-elf", "--test", "elf", "--", "--nocapture"],
               "materiaal-app/elf-policy", extra_env={**extra, "NOS_MATERIAAL_ELF": str(binary)})
    logs.append("elf-policy")
    assembly = subprocess.check_output(["objdump", "-d", "--no-show-raw-insn", str(binary)], text=True)
    instructions = 0
    forbidden = []
    for line in assembly.splitlines():
        instruction = re.match(r"^\s*[0-9a-f]+:\s+([a-z][a-z0-9.]*)\b", line)
        if not instruction:
            continue
        instructions += 1
        mnemonic = instruction[1]
        if (re.search(r"%(?:xmm|ymm|zmm|mm[0-7]|st\b)", line)
                or mnemonic.startswith(("f", "v"))
                or mnemonic in ("emms", "wait", "ldmxcsr", "stmxcsr")):
            forbidden.append(line)
    symbols = subprocess.check_output(["nm", str(binary)], text=True)
    host = [line for line in symbols.splitlines()
            if re.search(r"(?i)\b(malloc|calloc|realloc|free|pthread_|mmap|munmap|__libc_)", line)]
    if instructions == 0 or forbidden or host:
        raise RuntimeError(f"unexpected native code/backend: {forbidden!r} {host!r}")
    segments = subprocess.check_output(["readelf", "-lW", str(binary)], text=True)
    roots = ["sdk/examples/materiaal", "sdk/input-client", "sdk/runtime", "sdk/catalog-client",
             "sdk/document-session", "ui/native", "ui/surface-protocol", "crates/ui-core",
             "crates/capability-core", "crates/kernel-core", "services/volume", "services/catalog", "services/input-core"]
    paths = []
    for directory in roots:
        for path in (ROOT / directory).rglob("*"):
            relative = path.relative_to(ROOT / directory)
            if not path.is_file() or any(p in ("target", "verification", "__pycache__") for p in relative.parts):
                continue
            if path.suffix in (".rs", ".toml", ".ld", ".py", ".lock"):
                paths.append(path)
    paths += [ROOT / "ui/prototype/tokens.css", ROOT / "sdk/examples/system/src/support.rs", ROOT / "tools/common.py", ROOT / "rust-toolchain.toml"]
    inputs = {str(p.relative_to(ROOT)): digest(p) for p in sorted(set(paths))}
    combined = hashlib.sha256(json.dumps(inputs, sort_keys=True).encode()).hexdigest()
    output = EXAMPLE / "verification"
    output.mkdir(exist_ok=True)
    for name in logs:
        for suffix in ("stdout.txt", "stderr.txt", "json"):
            source = BUILD / f"logs/materiaal-app/{name}.{suffix}"
            (output / f"{name}.{suffix}").write_bytes(source.read_bytes())
    (output / "readelf.txt").write_text(segments)
    patch = Path("/tmp/nieuw-os-desktop-sdk-wrappers.patch")
    sdk_diff = subprocess.check_output(["git", "diff", "--", "sdk/Cargo.toml", "sdk/runtime"], cwd=ROOT)
    if sdk_diff:
        (output / "sdk-build-input.json").write_text(json.dumps({"patch_utf8": sdk_diff.decode("utf-8"), "sha256": hashlib.sha256(sdk_diff).hexdigest()}, indent=2) + "\n")
    report = {
        "kind": "host-contracts-and-built-ELF-only", "native_execution": False,
        "source_version": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "source_inputs_sha256": combined, "inputs": inputs,
        "runtime_abi_minor": int(match[1]),
        "known_temporary_sdk_patch_sha256": digest(patch) if patch.exists() else None,
        "actual_shared_sdk_diff_sha256": hashlib.sha256(sdk_diff).hexdigest() if sdk_diff else None,
        "artifact": {"path": str(binary), "bytes": binary.stat().st_size, "sha256": digest(binary),
                     "scalar_instructions": instructions, "simd_fpu": forbidden, "host_backends": host},
        "host_scenarios": scenario_count, "actual_ELF_loader_policy_tests": 1,
        "build_claim": "actual x86_64-unknown-none executable; not execution, pixels, hardware input or durable encrypted storage",
    }
    (output / "pre-native.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"source_inputs_sha256": combined, "artifact": report["artifact"], "native_execution": False}))
    return 0

if __name__ == "__main__":
    sys.exit(main())
