#!/usr/bin/env python3
"""B03-P2: independent loader/ring-3 authentication of immutable TEST vectors.

This is a read-only cryptographic probe, not a policy transition or native bundle
ingress. The regular signed image boots with an empty inbox. The slot image must
remain byte-identical and the broker must observe no write/flush. The ordinary
signed loader may advance its existing image floor in disposable OVMF variables.
The unchanged eighteen-stage tools/test_update.py gate remains required separately.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import shutil
import subprocess
import time
import uuid

from build import build
from common import BUILD, ROOT, TARGET, environment, firmware, save_json, sha256, source_hash
import run_update
import update_disk as disk

CASE_NAMES = (
    "policy-authentic", "binding-image-authentic", "successor-authentic",
    "policy-ed-tampered", "policy-pq-tampered", "policy-target", "policy-truncated",
    "policy-length", "policy-root", "root-ed-role", "root-pq-role", "successor-lineage",
    "binding-ed-tampered", "binding-pq-tampered", "binding-policy-digest",
    "binding-manifest-digest", "binding-target", "binding-truncated", "image-digest",
    "image-floor", "signer-revoked", "signer-premature", "signer-expired",
    "policy-trailing", "policy-domain-confusion", "binding-domain-confusion",
)
CASE_COUNT = 26
EXPECTED_MASK = 67_108_863
FIXTURES = ROOT / "crates/update-core/src/policy_envelope/probe-fixtures"
LOADER_MARKERS = (
    f"BOOT:POLICY_PROBE:COUNT:{CASE_COUNT}",
    f"BOOT:POLICY_PROBE:MASK:{EXPECTED_MASK}",
    "BOOT:POLICY_PROBE:PASS",
)
SERVICE_MARKERS = (
    f"UPDATE:POLICY_PROBE:COUNT:{CASE_COUNT:020d}",
    f"UPDATE:POLICY_PROBE:MASK:{EXPECTED_MASK:020d}",
    "UPDATE:POLICY_PROBE:PASS",
)
EXPECTED_MARKERS = (
    *LOADER_MARKERS, "BOOT:FLOOR:0", "BOOT:SLOT:B:REJECTED:FORMAT",
    "BOOT:SLOT:A:VERSION:1", "BOOT:SIGNED:OK", *run_update.BOOT,
    "UPDATE:MANIFEST:PASS", *SERVICE_MARKERS, "UPDATE:READY",
    "UPDATE:NO_BUNDLE", "UPDATE:RESOURCES:PASS", "UPDATE:PASS", "M1:PASS",
)
EXPECTED_NUMBERS = {
    "POLICY_PROBE:COUNT": CASE_COUNT, "POLICY_PROBE:MASK": EXPECTED_MASK,
    "IMAGE_VERSION": 1, "CURRENT_VERSION": 1, "RESULT": 4,
    "HEADER_ACTIVE": 0, "HEADER_GENERATION": 1, "ACTIVE_VERSION": 1,
    "RUNNING_SLOT": 0, "INBOX_VERSION": 0, "INBOX_REJECT": 0,
    "INSTALLED_VERSION": 0, "COPIED_SECTORS": 0,
    "SERVICE_WRITES": 0, "SERVICE_FLUSHES": 0, "BROKER:WRITES": 0,
    "BROKER:FLUSHES": 0, "BROKER:HEADER_WRITES": 0, "BROKER:OTHER_WRITES": 0,
}


def evaluate(stdout: str, rc, timed_out: bool, killed: bool, running: bool) -> dict[str, int]:
    """Require both complete phase-ordered reports and the actual native handover."""
    if not isinstance(stdout, str) or rc != 33 or timed_out or killed or running:
        raise ValueError("probe did not exit normally after the native kernel completed")
    lines = stdout.splitlines()
    if [line for line in lines if "POLICY_PROBE" in line] != [*LOADER_MARKERS, *SERVICE_MARKERS]:
        raise ValueError("missing, partial, duplicate or forged probe report")
    controlled = [line for line in lines if (not run_update.NUMBER.fullmatch(line)
                                             or line.startswith("UPDATE:POLICY_PROBE:"))
                  and (line.startswith(run_update.CONTROLLED_PREFIXES)
                       or line.startswith("UPDATE:POLICY_PROBE:"))]
    if controlled != list(EXPECTED_MARKERS):
        raise ValueError("probe/boot/service marker order mismatch")
    if any(marker in line for line in lines for marker in run_update.FAILURES):
        raise ValueError("native failure marker present")
    numbers = run_update.numbers(stdout)
    for key, expected in EXPECTED_NUMBERS.items():
        if numbers.get(key) != expected:
            raise ValueError(f"{key}: expected {expected}, found {numbers.get(key)}")
    if not 0 <= numbers.get("TICKS", -1) < 60_000:
        raise ValueError("missing or exhausted tick budget")
    return numbers


def check_store(before: bytes, after: bytes, manifest: bytes, image: bytes) -> dict:
    if before != after:
        raise ValueError("read-only probe changed the slot image")
    oracle = disk.analyse(after)
    if oracle["header"] != {"active": 0, "generation": 1}:
        raise ValueError("probe slot header differs from the explicit fixture")
    if not disk.manifest_region_is(after, "A", manifest) or not disk.image_region_is(after, "A", image):
        raise ValueError("slot A differs from the source-bound probe kernel")
    if any(disk.region(after, disk.SLOT_B_MANIFEST, disk.STORE_SECTORS * disk.SECTOR - disk.SLOT_B_MANIFEST * disk.SECTOR)):
        raise ValueError("probe requires an empty slot B and inbox")
    return oracle


def validate_build(manifest: dict, frozen: str) -> None:
    if (manifest.get("loader_feature") != "signed-policy-probe-test"
            or manifest.get("kernel_feature") != "native-update-policy-probe"
            or manifest.get("source_sha256") != frozen
            or manifest.get("update", {}).get("image_version") != 1):
        raise ValueError("probe requires its exact loader/service/kernel profile and source")


def fixture_hashes() -> dict[str, str]:
    return {path.name: sha256(path) for path in sorted(FIXTURES.iterdir()) if path.is_file()}


def run(*, timeout: int = run_update.MAX_TIMEOUT) -> dict:
    if not 0 < timeout <= run_update.MAX_TIMEOUT:
        raise ValueError("invalid bounded native-probe timeout")
    frozen = source_hash()
    env = environment()
    subprocess.run(["cargo", "+1.90.0", "run", "--offline", "--locked", "-q", "-p", "update-core",
                    "--example", "policy_probe_fixtures", "--features",
                    "experimental-policy-test,test-signing-anchors", "--", "--check"],
                   cwd=ROOT, env=env, check=True)
    manifest = build("signed-policy-probe-test", "native-update-policy-probe",
                     "policy-envelope-probe", image_version=1)
    validate_build(manifest, frozen)
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("qemu-system-x86_64 not found")
    run_dir = BUILD / "run" / f"policy-envelopes-{uuid.uuid4().hex[:10]}"
    run_dir.mkdir(parents=True)
    code, variables = firmware()
    shutil.copytree(BUILD / "esp", run_dir / "esp")
    if (run_dir / "esp/kernel.elf").exists():
        raise ValueError("signed probe ESP unexpectedly contains an unsigned fallback")
    shutil.copy2(code, run_dir / "OVMF_CODE.fd")
    shutil.copy2(variables, run_dir / "OVMF_VARS.fd")
    kernel = TARGET / "x86_64-unknown-none/release/os-kernel"
    if sha256(kernel) != manifest["kernel"]["sha256"]:
        raise ValueError("probe kernel changed after build")
    if sha256(run_dir / "esp/EFI/BOOT/BOOTX64.EFI") != manifest["loader"]["sha256"]:
        raise ValueError("probe loader changed after build")
    shutil.copy2(kernel, run_dir / "kernel.elf")
    signed = run_dir / "manifest.bin"
    run_update.update_tool(env, "sign", "--keys", str(ROOT / "tools/update-keys"), "--label", "TEST",
                           "--image", str(run_dir / "kernel.elf"), "--version", "1", "--out", str(signed))
    store = run_dir / "store.img"
    run_update.update_tool(env, "pack", "--out", str(store), "--active", "A", "--generation", "1",
                           "--slot-a-manifest", str(signed), "--slot-a-image", str(run_dir / "kernel.elf"))
    before = store.read_bytes()
    check_store(before, before, signed.read_bytes(), kernel.read_bytes())
    command = run_update.qemu_command(qemu, run_dir, run_dir)
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    rc, stdout, stderr, timed_out, killed, running = run_update.execute(command, env, timeout=timeout)
    duration = time.monotonic() - start
    (run_dir / "serial.txt").write_text(stdout, encoding="utf-8")
    (run_dir / "stderr.txt").write_text(stderr, encoding="utf-8")
    after = store.read_bytes()
    report = {
        "schema_version": 1, "profile": "native-update-policy-probe", "source_fingerprint_version": 3,
        "source_sha256": frozen, "started_utc": started, "duration_s": duration,
        "timeout_s": timeout, "raw_qemu_exit": rc, "timed_out": timed_out,
        "host_kill": killed, "running_before_kill": running, "command": command,
        "qemu_version": subprocess.check_output([qemu, "--version"], env=env, text=True).strip(),
        "firmware_code_sha256": sha256(code), "firmware_template_sha256": sha256(variables),
        "vars_after_sha256": sha256(run_dir / "OVMF_VARS.fd"), "build": manifest,
        "fixture_hashes": fixture_hashes(), "cases": list(CASE_NAMES), "expected_mask": EXPECTED_MASK,
        "store_before_sha256": disk.sha256(before), "store_after_sha256": disk.sha256(after),
        "test_inputs": {"tests/test_policy_envelopes.py": sha256(ROOT / "tests/test_policy_envelopes.py")},
        "run_directory": str(run_dir), "stdout": stdout, "stderr": stderr, "passed": False,
        "evidence_boundary": "Immutable TEST-vector native authentication only; no policy ingress, commit or rollback-resistance evidence. Existing firmware image floor may advance.",
    }
    try:
        report["evidence"] = evaluate(stdout, rc, timed_out, killed, running)
        report["oracle"] = check_store(before, after, signed.read_bytes(), kernel.read_bytes())
        if source_hash() != frozen:
            raise ValueError("source changed during the probe")
        report["passed"] = True
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    save_json(run_dir / "result.json", report)
    save_json(BUILD / "policy-envelopes-summary.json", report)
    print(f"B03 policy envelopes: passed={report['passed']} raw={rc} duration={duration:.1f}s {run_dir}")
    if "error" in report:
        print(report["error"])
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout", type=int, default=run_update.MAX_TIMEOUT)
    args = parser.parse_args()
    return 0 if run(timeout=args.timeout)["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
