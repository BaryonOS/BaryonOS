#!/usr/bin/env python3
"""B04-T3: disposable OVMF Secure Boot TEST chain (firmware -> loader -> kernel).

Boots the B03 signed-boot-test loader under the distro OVMF secboot firmware
with the distro snakeoil variable store (PK/KEK/db = one public TEST cert).
Cases differ only in the Authenticode state of the same loader binary. TEST
anchors only: no production key, no physical firmware, not a release chain.
"""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import time
import uuid

from common import BUILD, ROOT, environment, qemu_machine, save_json, sha256, source_hash
import run_update

# Capture tools write only to a NEW direct child of this directory.
EVIDENCE_ROOT = ROOT / "docs" / "verification"

SECBOOT_CODE = Path("/usr/share/OVMF/OVMF_CODE_4M.secboot.fd")
SNAKEOIL_VARS = Path("/usr/share/OVMF/OVMF_VARS_4M.snakeoil.fd")
SNAKEOIL_CERT = Path("/usr/share/ovmf/PkKek-1-snakeoil.pem")
SNAKEOIL_KEY = Path("/usr/share/ovmf/PkKek-1-snakeoil.key")
# Public passphrase of the distro TEST key. Not documented on the host; source:
# Debian edk2 2025.02-8+deb13u1 debian/tests/shell.py passes "snakeoil" as the
# `password` of debian/python/UEFI/SignedBinary.py (openssl -passin). Assumed to
# hold for the Ubuntu build too. It protects no secret; never reuse for a real key.
SNAKEOIL_PASSPHRASE = "snakeoil"
# Pinned firmware (ovmf 2025.11-3ubuntu7.3); any other build fails closed.
EXPECTED_CODE_SHA256 = "33c413a04e9edf512c504e9b0accbcec4d009d406bca761cc0b8eacc93da60b6"
EXPECTED_VARS_SHA256 = "a6b8fface5ce0b5e945717036b0044a5bf4cd89b5ce7468fbd7c971dd0bd20e9"
READ_ONLY_PREFIXES = (Path("/usr"), Path("/etc"), Path("/boot"))
# Observed BdsDxe line (ovmf 2025.11-3ubuntu7.3) when image verification refuses the
# loader on the ESP (virtio FAT drive, index 0 at PCI 00:01.0 = boot option Boot0002).
DENIAL_MARKERS = ('Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x1,0x0)',
                  "Access Denied", "rejected probably by Secure Boot")
# Prefixes that may only appear once the loader (or anything after it) runs.
GUEST_PREFIXES = ("BOOT:", "KERNEL:", "BOOTINFO:", "UPDATE:", "M1:")
REFUSAL_TIMEOUT_S = 120
BOOT_TIMEOUT_S = run_update.MAX_TIMEOUT
CASES = {
    # name: (signer, tamper, store stage, expected outcome)
    "unsigned": (None, False, "install", "refused"),
    "signed": ("snakeoil", False, "install", "boot"),
    "wrong-key": ("ephemeral", False, "install", "refused"),
    "tampered": ("snakeoil", True, "install", "refused"),
    # Second link under SB: corrupted slot-B kernel image must still be rejected.
    "kernel-digest": ("snakeoil", False, "bad-digest", "boot"),
}
ORDER = ("unsigned", "signed", "wrong-key", "tampered", "kernel-digest")


def writable_destination(path: Path) -> Path:
    """Refuse to write firmware state outside the disposable build tree."""
    resolved = Path(path).resolve()
    if any(resolved.is_relative_to(prefix) for prefix in READ_ONLY_PREFIXES):
        raise ValueError(f"refusing to write firmware state to system path {resolved}")
    if not resolved.is_relative_to((BUILD / "run").resolve()):
        raise ValueError("firmware state must live under $NOS_BUILD_DIR/run")
    return resolved


def secure_boot_args(base: list[str], machine: str) -> list[str]:
    """Add SMM and secure pflash (required by the secboot CODE) to the B03 machine."""
    args = list(base)
    at = args.index("-machine") + 1
    if args[at] != machine:
        raise ValueError("unexpected machine argument in the shared B03 command")
    args[at] = f"{machine},smm=on"
    return args + ["-global", "driver=cfi.pflash01,property=secure,value=on"]


def timestamp_offset(pe: bytes) -> int:
    """Offset of the COFF TimeDateStamp: inside the Authenticode digest, never executed."""
    if pe[:2] != b"MZ":
        raise ValueError("not a PE image")
    header = int.from_bytes(pe[0x3C:0x40], "little")
    if header + 12 > len(pe) or pe[header:header + 4] != b"PE\0\0":
        raise ValueError("missing PE signature")
    return header + 8


def tamper(pe: bytes) -> bytes:
    data = bytearray(pe)
    data[timestamp_offset(pe)] ^= 0x01
    return bytes(data)


def classify_refusal(stdout: str, rc, timed_out: bool) -> list[str]:
    """Problems with a claimed firmware refusal; empty means refused as expected."""
    problems = []
    for line in stdout.splitlines():
        if any(prefix in line for prefix in GUEST_PREFIXES):
            problems.append(f"guest marker after expected refusal: {line.strip()[:80]}")
            break
    if not any(all(marker in line for marker in DENIAL_MARKERS) for line in stdout.splitlines()):
        problems.append("no firmware denial line observed")
    if rc == 33:
        problems.append("guest reached the smoke exit code")
    if not timed_out:
        # An early exit, even after the denial line, is not proof that nothing else ran.
        problems.append(f"refusal did not reach the host timeout; raw exit {rc}")
    return problems


def firmware_pin_problems(code_sha256: str, vars_sha256: str) -> list[str]:
    problems = []
    if code_sha256 != EXPECTED_CODE_SHA256:
        problems.append(f"secboot CODE sha256 {code_sha256} is not the pinned build")
    if vars_sha256 != EXPECTED_VARS_SHA256:
        problems.append(f"snakeoil VARS template sha256 {vars_sha256} is not the pinned build")
    return problems


def firmware_identity(report: dict) -> tuple:
    firmware = report["firmware"]
    return (firmware["code_sha256"], firmware["vars_template_sha256"], firmware["vars_run_before_sha256"])


def require_refusal_proof(reports: list[dict]) -> None:
    """A boot verdict counts only if this invocation also saw the same firmware refuse."""
    proofs = {firmware_identity(report) for report in reports
              if report["expected"] == "refused" and report["passed"]}
    for report in reports:
        if report["expected"] == "boot" and firmware_identity(report) not in proofs:
            report["problems"].append("unproven: no passing refusal case with identical firmware "
                                      "in this invocation; Secure Boot enforcement not shown")
            report["passed"] = False
            report["secure_boot_proof"] = None
        elif report["expected"] == "boot":
            report["secure_boot_proof"] = sorted(r["case"] for r in reports if r["expected"] == "refused"
                                                 and r["passed"] and firmware_identity(r) == firmware_identity(report))


def sign(env, source: Path, out: Path, key: Path, cert: Path) -> None:
    subprocess.run(["sbsign", "--key", str(key), "--cert", str(cert), "--output", str(out), str(source)],
                   env=env, stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=60, check=True)


def sbverify(env, image: Path, cert: Path) -> bool:
    return subprocess.run(["sbverify", "--cert", str(cert), str(image)], env=env,
                          stdin=subprocess.DEVNULL, capture_output=True, timeout=60).returncode == 0


def make_loader(case: str, loader: Path, work: Path, env) -> dict:
    """Produce the case's BOOTX64.EFI; key material lives only in `work` and is deleted."""
    signer, tampered, _, _ = CASES[case]
    out = work / "BOOTX64.EFI"
    key = work / "signing-key.pem"
    cert = SNAKEOIL_CERT
    try:
        if signer is None:
            shutil.copy2(loader, out)
        else:
            old = os.umask(0o077)
            try:
                if signer == "snakeoil":
                    subprocess.run(["openssl", "pkey", "-in", str(SNAKEOIL_KEY), "-passin", "env:NOS_SB_PASS",
                                    "-out", str(key)], env=dict(env, NOS_SB_PASS=SNAKEOIL_PASSPHRASE),
                                   stdin=subprocess.DEVNULL, capture_output=True, timeout=30, check=True)
                else:
                    cert = work / "ephemeral-cert.pem"
                    subprocess.run(["openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
                                    "-subj", "/CN=BaryonOS ephemeral non-enrolled TEST key/",
                                    "-keyout", str(key), "-out", str(cert)],
                                   env=env, stdin=subprocess.DEVNULL, capture_output=True, timeout=60, check=True)
            finally:
                os.umask(old)
            sign(env, loader, out, key, cert)
        if tampered:
            out.write_bytes(tamper(out.read_bytes()))
    finally:
        key.unlink(missing_ok=True)
    info = {"signer": signer, "tampered": tampered, "sha256": sha256(out),
            "sbverify_snakeoil": sbverify(env, out, SNAKEOIL_CERT)}
    if signer == "ephemeral":
        info["sbverify_own_cert"] = sbverify(env, out, cert)
        info["ephemeral_cert_sha256"] = sha256(cert)
        cert.unlink()
        if not info["sbverify_own_cert"]:
            raise ValueError(f"wrong-key loader does not verify against its own ephemeral cert: {info}")
    if tampered:
        info["tamper_offset"] = timestamp_offset(out.read_bytes())
    expected_ok = signer == "snakeoil" and not tampered
    if info["sbverify_snakeoil"] != expected_ok:
        raise ValueError(f"host sbverify disagrees with the {case} construction: {info}")
    return info


def prepare() -> tuple[dict, Path, str]:
    """Build/sign B03 TEST images for the current source if needed; return images and loader."""
    frozen = source_hash()
    images_path = run_update.IMAGES
    stale = (not images_path.exists()
             or json.loads(images_path.read_text()).get("source_sha256") != frozen)
    if stale:
        import test_update
        test_update.prepare_images(frozen)
    images = run_update.load_images()
    manifest = json.loads((BUILD / "manifest.json").read_text())
    loader = BUILD / "esp/EFI/BOOT/BOOTX64.EFI"
    if (manifest.get("loader_feature") != "signed-boot-test" or manifest.get("source_sha256") != frozen
            or sha256(loader) != images["loader_sha256"]):
        raise ValueError("ESP loader is not the signed-boot-test loader recorded with the images")
    return images, loader, frozen


def run_case(case: str, images: dict, loader: Path, frozen: str) -> dict:
    signer, tampered, stage, expected = CASES[case]
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("qemu-system-x86_64 not found")
    run_dir = writable_destination(BUILD / "run" / f"secure-boot-{case}-{uuid.uuid4().hex[:10]}")
    run_dir.mkdir(parents=True)
    shutil.copytree(BUILD / "esp", run_dir / "esp")
    loader_info = make_loader(case, loader, run_dir, env)
    shutil.move(run_dir / "BOOTX64.EFI", run_dir / "esp/EFI/BOOT/BOOTX64.EFI")
    shutil.copy2(SECBOOT_CODE, run_dir / "OVMF_CODE.fd")
    variables = writable_destination(run_dir / "OVMF_VARS.fd")
    shutil.copyfile(SNAKEOIL_VARS, variables)
    variables.chmod(stat.S_IRUSR | stat.S_IWUSR)
    vars_before = sha256(variables)
    store = run_dir / "store.img"
    run_update.pack_store(stage, images, env, store)
    if run_update.STAGES[stage]["mutate"]:
        data = bytearray(store.read_bytes())
        run_update.mutate(data, run_update.STAGES[stage]["mutate"])
        store.write_bytes(data)
    before = store.read_bytes()
    machine = qemu_machine()
    args = secure_boot_args(run_update.qemu_command(qemu, run_dir, run_dir), machine)
    timeout = BOOT_TIMEOUT_S if expected == "boot" else REFUSAL_TIMEOUT_S
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    rc, out, err, timed_out, _, _ = run_update.execute(args, env, timeout=timeout)
    duration = time.monotonic() - start
    (run_dir / "serial.txt").write_text(out, encoding="utf-8")
    (run_dir / "stderr.txt").write_text(err, encoding="utf-8")
    after = store.read_bytes()
    pin_problems = firmware_pin_problems(sha256(run_dir / "OVMF_CODE.fd"), vars_before)
    report = dict(schema_version=1, case=case, expected=expected, store_stage=stage,
                  started_utc=started, duration_s=round(duration, 3), timeout_s=timeout,
                  timed_out=timed_out, raw_qemu_exit=rc, command=args,
                  qemu_version=subprocess.check_output([qemu, "--version"], text=True).splitlines()[0],
                  machine=f"{machine},smm=on", source_fingerprint_version=3, source_sha256=frozen,
                  firmware={"code": str(SECBOOT_CODE), "code_sha256": sha256(run_dir / "OVMF_CODE.fd"),
                            "vars_template": str(SNAKEOIL_VARS), "vars_template_sha256": sha256(SNAKEOIL_VARS),
                            "db_cert": str(SNAKEOIL_CERT), "db_cert_sha256": sha256(SNAKEOIL_CERT),
                            "vars_run_before_sha256": vars_before, "vars_run_after_sha256": sha256(variables)},
                  unsigned_loader_sha256=sha256(loader), loader=loader_info,
                  kernels=images["kernels"], manifests=images["manifests"],
                  store_before_sha256=run_update.disk.sha256(before),
                  store_after_sha256=run_update.disk.sha256(after),
                  run_directory=str(run_dir), passed=False, problems=list(pin_problems))
    judge(report, out, rc, timed_out, before, after, images, frozen)
    save_json(run_dir / "result.json", report)
    return report


def judge(report: dict, out: str, rc, timed_out: bool, before: bytes, after: bytes,
          images: dict, frozen: str) -> None:
    """Add case verdict problems to those already recorded (e.g. firmware pins); never replace them."""
    stage = report["store_stage"]
    try:
        if report["expected"] == "boot":
            report["evidence"] = run_update.evaluate(stage, out, rc, timed_out, False, False)
            run_update.check_store(stage, before, after, images)
        else:
            report["problems"].extend(classify_refusal(out, rc, timed_out))
            if after != before:
                report["problems"].append("store changed although the loader never ran")
        report["passed"] = not report["problems"] and source_hash() == frozen
    except Exception as error:
        report["problems"].append(f"{type(error).__name__}: {error}")
    report["passed"] = report["passed"] and not report["problems"]


def reserve_evidence(path: Path) -> Path:
    """Create a NEW direct child of EVIDENCE_ROOT; refuse anything else without touching it."""
    root = EVIDENCE_ROOT.resolve()
    target = Path(path).resolve()
    if target.parent != root:
        raise ValueError(f"--evidence must be a new direct child of {root}, not {target}")
    target.mkdir()  # FileExistsError for an earlier capture; it is left unchanged
    return target


def publish(report: dict, evidence: Path | None) -> None:
    run_dir = Path(report["run_directory"])
    save_json(run_dir / "result.json", report)
    if evidence is not None:
        target = evidence / report["case"]
        target.mkdir()
        save_json(target / "result.json", report)
        shutil.copy2(run_dir / "serial.txt", target / "serial.txt")
    print(f"B04 secure-boot {report['case']}: expected={report['expected']} passed={report['passed']} "
          f"raw={report['raw_qemu_exit']} timeout={report['timed_out']} duration={report['duration_s']:.1f}s {run_dir}")
    for problem in report["problems"]:
        print(f"  {problem}")


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=ORDER + ("all",), required=True)
    parser.add_argument("--evidence", type=Path,
                        help="NEW direct child of docs/verification/ for per-case result.json and serial.txt")
    args = parser.parse_args(argv)
    pins = firmware_pin_problems(sha256(SECBOOT_CODE), sha256(SNAKEOIL_VARS))
    if pins:
        print("\n".join(pins))
        return 1
    evidence = None
    if args.evidence is not None:
        try:
            evidence = reserve_evidence(args.evidence)
        except (ValueError, FileExistsError, FileNotFoundError) as error:
            print(f"refusing --evidence: {error}")
            return 1
    try:
        images, loader, frozen = prepare()
        cases = ORDER if args.case == "all" else (args.case,)
        reports, crashed = [], []
        for case in cases:
            try:
                reports.append(run_case(case, images, loader, frozen))
            except Exception as error:  # one broken case must not discard the other cases' reports
                crashed.append(dict(schema_version=1, case=case, expected=CASES[case][3], passed=False,
                                    problems=[f"case did not complete: {type(error).__name__}: {error}"]))
        require_refusal_proof(reports)
        for report in reports:
            publish(report, evidence)
        for report in crashed:
            if evidence is not None:
                (evidence / report["case"]).mkdir()
                save_json(evidence / report["case"] / "result.json", report)
            print(f"B04 secure-boot {report['case']}: expected={report['expected']} passed=False")
            print(f"  {report['problems'][0]}")
    finally:
        if evidence is not None:
            try:
                evidence.rmdir()  # only succeeds while still empty: never removes a recorded report
            except OSError:
                pass
    passed = not crashed and all(report["passed"] for report in reports)
    print(f"B04 secure-boot TEST chain: {sum(r['passed'] for r in reports)}/{len(cases)} as expected")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
