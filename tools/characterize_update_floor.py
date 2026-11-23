#!/usr/bin/env python3
"""B03-F1 CHARACTERIZATION of audit finding REL-05 (not a fix; earns no points).

The signed loader raises the NV anti-rollback floor to a slot's version as soon
as that slot's signatures and digest verify, before the kernel has run, let
alone proven healthy (boot/loader/src/signed.rs select()). This runner boots
one disposable QEMU machine with ONE persistent OVMF variable store and ONE slot
store and exits 0 only when the hazard is observed:

  control  fresh variables, header A: the signed v1 M1 kernel boots healthy.
  boot 1   fresh variables, header B: the correctly signed v2 (a test-panic
           kernel standing in for a signed but broken release) is selected,
           the floor goes 0 -> 2 and the kernel panics.
  boot 2   same variables: the floor is still 2, v1 is rejected as DOWNGRADE
           and the broken v2 is selected and panics again.
  boot 3   the host switches the header back to A (rollback attempt) with the
           store bytes of the healthy control: v1 is still refused.

Exit 0 means the counterexample was reproduced. A future fix (for example a
floor raised only after a health confirmation) must make this runner fail by
design; that change needs an owner decision under ADR-049. Public TEST keys
only; QEMU/TCG only; a guest panic is not physical power loss."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import shutil
import subprocess
import time
import uuid
from common import BUILD, ROOT, TARGET, environment, firmware, save_json, sha256, source_hash
import run_update as ru
import update_disk as disk

OUT = BUILD / "update-floor"
TIMEOUT = 300
LABEL = ("CHARACTERIZATION ONLY (REL-05): exit 0 means the floor-before-health hazard was "
         "observed. This is not a fix and earns no progress points.")
FLOOR_VARIABLE = "NieuwOsMinVersion".encode("utf-16-le")
PREFIXES = ("BOOT:", "KERNEL:", "BOOTINFO:", "M1:", "UPDATE:")
PANIC = "M1:FAIL:KERNEL_PANIC"
REFUSED = ("BOOT:FLOOR:2", "BOOT:SLOT:A:REJECTED:DOWNGRADE", "BOOT:SLOT:B:VERSION:2")
# Each boot: state sequence, header slot set by the host before the boot,
# exact loader lines, and whether the selected kernel is the healthy v1.
BOOTS = (
    dict(name="control-v1", state="control", fresh=True, active=0,
         loader=("BOOT:FLOOR:0", "BOOT:SLOT:A:VERSION:1"), healthy=True),
    dict(name="boot-1-v2-panics", state="hazard", fresh=True, active=1,
         loader=("BOOT:FLOOR:0", "BOOT:SLOT:B:VERSION:2"), healthy=False),
    dict(name="boot-2-floor-kept", state="hazard", fresh=False, active=1, loader=REFUSED, healthy=False),
    dict(name="boot-3-rollback-refused", state="hazard", fresh=False, active=0, loader=REFUSED, healthy=False),
)
PLAN = {boot["name"]: boot for boot in BOOTS}


def expected_lines(name: str) -> list[str]:
    boot = PLAN[name]
    return [*boot["loader"], "BOOT:SIGNED:OK", *ru.BOOT, "M1:PASS" if boot["healthy"] else PANIC]


def evaluate(name: str, stdout: str, rc, timed_out: bool) -> list[str]:
    """Transcript verdict. Raw exit 35 alone is ambiguous (loader failure and
    kernel panic share it), so the exact ordered marker list is required."""
    kept = [line for line in stdout.splitlines() if line.startswith(PREFIXES)]
    if kept != expected_lines(name):
        raise ValueError(f"marker sequence mismatch: {kept}")
    if timed_out or rc != (33 if PLAN[name]["healthy"] else 35):
        raise ValueError(f"raw exit {rc}, timeout {timed_out}")
    return kept


def check_store(name: str, before: bytes, after: bytes, files: dict[str, bytes]) -> dict:
    """Independent byte-level verdict: no boot writes the store, both slots
    hold exactly their signed bundles and both verify against their manifests."""
    if after != before:
        raise ValueError("store changed during the boot")
    oracle = disk.analyse(after)
    if oracle["header"]["active"] != PLAN[name]["active"]:
        raise ValueError(f"header: {oracle['header']}")
    for slot, version in (("A", 1), ("B", 2)):
        facts = oracle["regions"][slot]
        if not (disk.manifest_region_is(after, slot, files[f"m{version}"])
                and disk.image_region_is(after, slot, files[f"k{version}"])):
            raise ValueError(f"slot {slot} does not hold exactly the v{version} bundle")
        if facts["manifest"] is None or facts["manifest"]["version"] != version \
                or facts["image_matches_manifest"] is not True:
            raise ValueError(f"oracle: slot {slot} is not an intact v{version} bundle")
    return oracle


def floor_variable_present(variables: bytes) -> bool:
    return FLOOR_VARIABLE in variables


def prepare(env) -> dict[str, bytes]:
    """Signed-boot-test loader with a healthy M1 kernel (v1) and a test-panic
    kernel (v2), both signed with the public TEST keys."""
    from build import build
    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir(parents=True)
    loader = None
    for name, feature in (("k1", None), ("k2", "test-panic")):
        manifest = build("signed-boot-test", feature, f"update-floor-{name}")
        if manifest["loader_feature"] != "signed-boot-test" or manifest["kernel_feature"] != feature:
            raise ValueError("unexpected build profile")
        shutil.copy2(TARGET / "x86_64-unknown-none/release/os-kernel", OUT / f"{name}.elf")
        if sha256(OUT / f"{name}.elf") != manifest["kernel"]["sha256"]:
            raise ValueError(f"{name} differs from its build manifest")
        if loader not in (None, manifest["loader"]["sha256"]):
            raise ValueError("the signed-boot loader differed between builds")
        loader = manifest["loader"]["sha256"]
    shutil.copytree(BUILD / "esp", OUT / "esp")
    if (OUT / "esp/kernel.elf").exists() or sha256(OUT / "esp/EFI/BOOT/BOOTX64.EFI") != loader:
        raise ValueError("ESP must hold exactly the signed loader and no unsigned kernel")
    for version in (1, 2):
        ru.update_tool(env, "sign", "--keys", str(ROOT / "tools/update-keys"), "--label", "TEST",
                       "--image", str(OUT / f"k{version}.elf"), "--version", str(version),
                       "--out", str(OUT / f"m{version}.bin"))
    return {f"{kind}{version}": (OUT / f"{kind}{version}.{'elf' if kind == 'k' else 'bin'}").read_bytes()
            for kind in "km" for version in (1, 2)}


def boot_once(name: str, state_dir, qemu: str, env, files: dict[str, bytes], frozen: str) -> dict:
    plan = PLAN[name]
    run_dir = OUT / "run" / f"{name}-{uuid.uuid4().hex[:8]}"
    run_dir.mkdir(parents=True)
    shutil.copytree(OUT / "esp", run_dir / "esp")
    shutil.copy2(firmware()[0], run_dir / "OVMF_CODE.fd")
    store_path = state_dir / "store.img"
    store = bytearray(store_path.read_bytes())
    disk.set_active(store, plan["active"])  # host-side header hint only
    store_path.write_bytes(store)
    before, vars_before = bytes(store), (state_dir / "OVMF_VARS.fd").read_bytes()
    args = ru.qemu_command(qemu, run_dir, state_dir)
    started, start = datetime.now(timezone.utc).isoformat(), time.monotonic()
    rc, out, err, timed_out, _, _ = ru.execute(args, env, timeout=TIMEOUT)
    duration = time.monotonic() - start
    (run_dir / "serial.txt").write_text(out, encoding="utf-8")
    after, vars_after = store_path.read_bytes(), (state_dir / "OVMF_VARS.fd").read_bytes()
    report = dict(schema_version=1, characterization=LABEL, boot=name, state=plan["state"],
                  header_active=plan["active"], started_utc=started, duration_s=duration, timeout_s=TIMEOUT,
                  timed_out=timed_out, raw_qemu_exit=rc, command=args,
                  qemu_version=subprocess.check_output([qemu, "--version"], text=True).strip(),
                  source_fingerprint_version=3, source_sha256=frozen,
                  run_directory=run_dir.relative_to(ROOT).as_posix(),
                  state_directory=state_dir.relative_to(ROOT).as_posix(),
                  store_before_sha256=disk.sha256(before), store_after_sha256=disk.sha256(after),
                  vars_before_sha256=disk.sha256(vars_before), vars_after_sha256=disk.sha256(vars_after),
                  floor_variable_before=floor_variable_present(vars_before),
                  floor_variable_after=floor_variable_present(vars_after),
                  expected=expected_lines(name), stdout=out, stderr=err, passed=False)
    try:
        report["markers"] = evaluate(name, out, rc, timed_out)
        report["oracle"] = check_store(name, before, after, files)
        if not report["floor_variable_after"]:
            raise ValueError("no floor variable in the persistent OVMF variable store")
        report["passed"] = True
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    save_json(run_dir / "result.json", report)
    print(f"REL-05 {name}: passed={report['passed']} raw={rc} duration={duration:.1f}s {run_dir}")
    if "error" in report:
        print(report["error"])
    return report


def main() -> int:
    argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter).parse_args()
    print(LABEL)
    frozen = source_hash()
    env = environment()
    qemu = shutil.which("qemu-system-x86_64", path=env["PATH"])
    if not qemu:
        raise RuntimeError("qemu-system-x86_64 not found")
    template = firmware()[1]
    summary = dict(schema_version=1, characterization=LABEL, finding="REL-05", criterion="B03 / FR-B03",
                   source_fingerprint_version=3, source_sha256=frozen, observed=False, boots=[])
    try:
        files = prepare(env)
        summary["artifacts"] = {name: disk.sha256(data) for name, data in files.items()}
        summary["loader_sha256"] = sha256(OUT / "esp/EFI/BOOT/BOOTX64.EFI")
        summary["template_has_floor_variable"] = floor_variable_present(template.read_bytes())
        if summary["template_has_floor_variable"]:
            raise ValueError("the OVMF variable template already holds a floor")
        states = {}
        for boot in BOOTS:
            if boot["fresh"]:
                state_dir = OUT / "state" / boot["state"]
                state_dir.mkdir(parents=True)
                ru.update_tool(env, "pack", "--out", str(state_dir / "store.img"), "--active", "A",
                               "--generation", "1",
                               "--slot-a-manifest", str(OUT / "m1.bin"), "--slot-a-image", str(OUT / "k1.elf"),
                               "--slot-b-manifest", str(OUT / "m2.bin"), "--slot-b-image", str(OUT / "k2.elf"))
                shutil.copy2(template, state_dir / "OVMF_VARS.fd")
                states[boot["state"]] = state_dir
            report = boot_once(boot["name"], states[boot["state"]], qemu, env, files, frozen)
            summary["boots"].append(report)
            save_json(OUT / "summary.json", summary)
            if not report["passed"]:
                break
        else:
            # The rollback boot saw byte-for-byte the store the control booted
            # healthy; only the persistent floor differs.
            same = summary["boots"][3]["store_before_sha256"] == summary["boots"][0]["store_before_sha256"]
            summary["rollback_store_equals_control"] = same
            summary["observed"] = same and source_hash() == frozen
    finally:
        from build import build
        build()  # restore the normal artifacts, as tools/test.py does
        save_json(OUT / "summary.json", summary)
    print(f"REL-05 hazard observed={summary['observed']} boots={len(summary['boots'])} {OUT / 'summary.json'}")
    return 0 if summary["observed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
