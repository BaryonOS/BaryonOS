#!/usr/bin/env python3
"""B03-REL05 U26 loader record (LREC) probe (not acceptance evidence).

Raw TPM 2.0 commands against a disposable swtpm 0.10.1 (TEST only). LREC
(0x0100B305) reuses the TRIAL/FLOOR template of REL-05 section 6.1
(ATTR_UPDATE_ORDINARY, POLICY_LOADER_WRITE, 64 bytes); U1/U7 proved that
template at 0x0100B300. This probe pins the LREC-specific facts:
  L1  define with the frozen public area; unwritten Name = trial_tpm_constants.rs;
  L2  owner NV_Write and index-password NV_Write refused (no OWNERWRITE/AUTHWRITE);
  L3  policy NV_Write at PCR 13 = 0^32 accepted; written Name frozen; AUTHREAD
      with empty authValue reads the record back;
  L4  after the PCR 13 extend the policy write is refused and the record is unchanged;
  L5  a run-time definer (empty ownerAuth, as after a clear) can recreate the
      public area but cannot write it, so the index keeps the unwritten Name.
The record bytes are a fixed test pattern; no secret is involved.

Usage: python3 -B probe.py <workdir>   (workdir: disposable, holds swtpm state)
"""
import importlib.util
import os
import shutil
import subprocess
import sys
import time


def _load(directory, module):
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", directory, "probe.py")
    spec = importlib.util.spec_from_file_location(module, path)
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


sq = _load("b03-rel05-squat-probe-20261125", "b03_rel05_squat_probe")
ph, p0 = sq.ph, sq.p0
log, check, decode_rc = p0.log, p0.check, p0.decode_rc

LREC = 0x0100B305
ATTR = 0x02041008  # ATTR_UPDATE_ORDINARY
NAME = "000bad14c18b2e6f42a9e51a1cdfb8d6ef84d0dba6fcfd0598852bde1504447c4782"
NAME_WRITTEN = "000b282efe987c36f93dc5464cfc70b668f196f74e7bcf62ace9c1a3757ef81f7a60"
RC_NV_AUTHORIZATION = 0x149
RC_AUTH_UNAVAILABLE = 0x12F


def policy_write(t, data):
    """One policy session per attempt; flush it if the command failed."""
    rc, s = p0.start_session(t)
    p0.satisfy(t, s, "NV_Write")
    rc = p0.nv_write(t, LREC, s, data)
    if rc:
        p0.flush(t, s)
    return rc


def main(work):
    work = os.path.abspath(work)
    state = os.path.join(work, "state-lrec")
    shutil.rmtree(state, ignore_errors=True)
    os.makedirs(state)
    os.chdir(work)
    sw = subprocess.Popen(["swtpm", "socket", "--tpm2", "--tpmstate", "dir=state-lrec",
                           "--server", "type=unixio,path=srv.sock", "--ctrl", "type=unixio,path=ctl.sock",
                           "--flags", "not-need-init,startup-clear", "--log", "file=swtpm-lrec.log"])
    record = bytes(range(64))
    try:
        for _ in range(100):
            if os.path.exists("srv.sock"):
                break
            time.sleep(0.05)
        t = p0.Tpm("srv.sock")
        policy = p0.policy_digest(p0.Z32, p0.CC["NV_Write"])
        check("L0 POLICY_LOADER_WRITE", policy.hex()
              == "088d4acf1b7ad7d6a53a44d97668736524219be4e7148b45be4dcc3d00462816", policy.hex())
        check("L0 frozen Names recomputed", sq.name(LREC, ATTR, policy, 64).hex() == NAME
              and sq.name(LREC, ATTR | sq.WRITTEN, policy, 64).hex() == NAME_WRITTEN, "")

        log("\n### L1: define LREC with the frozen public area")
        check("L1 define", sq.define_by(t, p0.RH_OWNER, LREC, ATTR, policy, 64) == 0, "")
        rc, attrs, n = sq.read_public(t, LREC)
        check("L1 unwritten Name", n.hex() == NAME and attrs & sq.WRITTEN == 0, n.hex() or decode_rc(rc))

        log("\n### L2: writes outside the policy")
        rc = sq.write(t, p0.RH_OWNER, LREC, record)
        check("L2 owner NV_Write refused (no OWNERWRITE)", rc == RC_NV_AUTHORIZATION, decode_rc(rc))
        rc = sq.write(t, LREC, LREC, record)
        check("L2 index-password NV_Write refused (no AUTHWRITE: TPM_RC_AUTH_UNAVAILABLE)",
              rc == RC_AUTH_UNAVAILABLE, decode_rc(rc))

        log("\n### L3: policy write at PCR 13 = 0^32")
        check("L3 PCR 13 is zero", p0.pcr13(t) == p0.Z32, "")
        check("L3 policy NV_Write of 64 bytes", policy_write(t, record) == 0, "")
        rc, attrs, n = sq.read_public(t, LREC)
        check("L3 written Name", n.hex() == NAME_WRITTEN and attrs & sq.WRITTEN != 0, n.hex() or decode_rc(rc))
        rc, v = p0.nv_read(t, LREC, 64)
        check("L3 AUTHREAD (empty authValue) reads the record", v == record, decode_rc(rc))

        log("\n### L4: after the PCR 13 extend")
        ph.extend_pcr13(t, b"BARYON-TRIAL-CLOSED-V1")
        rc = policy_write(t, bytes(64))
        check("L4 policy NV_Write refused", rc != 0, decode_rc(rc))
        rc, v = p0.nv_read(t, LREC, 64)
        check("L4 record unchanged", v == record, decode_rc(rc))

        log("\n### L5: run-time definer with empty ownerAuth")
        check("L5 owner undefine (empty ownerAuth)", ph.undefine(t, LREC, p0.RH_OWNER) == 0, "")
        check("L5 redefine with the frozen public area", sq.define_by(t, p0.RH_OWNER, LREC, ATTR, policy, 64) == 0, "")
        rc = policy_write(t, bytes(64))
        check("L5 policy NV_Write refused", rc != 0, decode_rc(rc))
        rc, attrs, n = sq.read_public(t, LREC)
        check("L5 Name stays unwritten", n.hex() == NAME and attrs & sq.WRITTEN == 0, n.hex() or decode_rc(rc))
    finally:
        p0.ctrl_shutdown("ctl.sock")
        sw.wait(timeout=10)

    log("\n# SUMMARY")
    for label, ok, detail in p0.RESULTS:
        log(f"{'PASS' if ok else 'FAIL'}  {label}  {detail}")
    with open("transcript-lrec.txt", "w") as f:
        f.write("\n".join(p0.OUT) + "\n")
    return 0 if all(ok for _, ok, _ in p0.RESULTS) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
