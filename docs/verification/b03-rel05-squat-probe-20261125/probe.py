#!/usr/bin/env python3
"""B03-REL05 M1-HAUTH squatting probe (not acceptance evidence).

Raw TPM 2.0 commands against a disposable swtpm 0.10.1 (TEST only). Checks the
security-review finding "HAUTH squatting" recorded in
docs/implementation/COLD-FRESHNESS-ANCHOR.md: with an empty ownerAuth (fresh
or cleared TPM), anyone who can submit commands can occupy the loader-owned
handles before the loader provisions them.
  N1  Name of NV_HAUTH before and after WRITTEN (frozen in trial_tpm_constants.rs);
  S1  owner squat with ATTR_HAUTH | AUTHREAD: loader DefineSpace -> NV_DEFINED,
      Name differs, an owner NV_Write of the secret is readable by password,
      an empty-ownerAuth owner undefine removes it;
  S2  owner squat with POLICY_DELETE: whether it can be defined and undefined;
  S3  platform-created squat (raw swtpm: platformAuth empty, U23 broken):
      owner undefine refused, survives TPM2_Clear;
  S4  record squat (U26 handle open; 0x0100B305 stands in): AUTHWRITE lets
      anyone write the record after the PCR 13 extend.
Secrets come from os.urandom and are redacted in the transcript.

Usage: python3 -B probe.py <workdir>   (workdir: disposable, holds swtpm state)
"""
import hashlib
import importlib.util
import os
import shutil
import struct
import subprocess
import sys
import time

_PH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "b03-rel05-hauth-probe-20261125", "probe.py")
_spec = importlib.util.spec_from_file_location("b03_rel05_hauth_probe", _PH)
ph = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(ph)
p0 = ph.p0
b2, decode_rc, log, check, pw = p0.b2, p0.decode_rc, p0.log, p0.check, ph.pw

p0.CC.update(NV_ReadPublic=0x169)
HAUTH, RECORD = ph.HAUTH, 0x0100B305
POLICY_DELETE, WRITTEN, PLATFORMCREATE = 1 << 10, 1 << 29, 1 << 30
RC_NV_DEFINED = 0x14C


def public(index, attrs, policy, size):
    return struct.pack(">IHI", index, p0.ALG_SHA256, attrs) + b2(policy) + struct.pack(">H", size)


def name(index, attrs, policy, size):
    return struct.pack(">H", p0.ALG_SHA256) + hashlib.sha256(public(index, attrs, policy, size)).digest()


def define_by(t, hierarchy, index, attrs, policy, size):
    return t.cmd(f"NV_DefineSpace 0x{index:08X} attrs 0x{attrs:08X} by 0x{hierarchy:08X}", "NV_DefineSpace",
                 struct.pack(">I", hierarchy), pw(), b2(b"") + b2(public(index, attrs, policy, size)))[0]


def read_public(t, index):
    """(rc, attributes, Name) from NV_ReadPublic (no session)."""
    rc, r = t.cmd(f"NV_ReadPublic 0x{index:08X}", "NV_ReadPublic", struct.pack(">I", index))
    if rc:
        return rc, 0, b""
    n = struct.unpack(">H", r[10:12])[0]
    attrs = struct.unpack(">I", r[18:22])[0]
    m = struct.unpack(">H", r[12 + n:14 + n])[0]
    return rc, attrs, r[14 + n:14 + n + m]


def write(t, auth_handle, index, data):
    return t.cmd(f"NV_Write 0x{index:08X} by 0x{auth_handle:08X} (empty auth)", "NV_Write",
                 struct.pack(">II", auth_handle, index), pw(), b2(data) + struct.pack(">H", 0))[0]


def clear_lockout(t):
    return t.cmd("Clear via lockout (empty)", "Clear", struct.pack(">I", ph.RH_LOCKOUT), pw())[0]


def main(work):
    work = os.path.abspath(work)
    state = os.path.join(work, "state-squat")
    shutil.rmtree(state, ignore_errors=True)
    os.makedirs(state)
    os.chdir(work)
    sw = subprocess.Popen(["swtpm", "socket", "--tpm2", "--tpmstate", "dir=state-squat",
                           "--server", "type=unixio,path=srv.sock", "--ctrl", "type=unixio,path=ctl.sock",
                           "--flags", "not-need-init,startup-clear", "--log", "file=swtpm-squat.log"])
    secret = os.urandom(64)
    try:
        for _ in range(100):
            if os.path.exists("srv.sock"):
                break
            time.sleep(0.05)
        t = p0.Tpm("srv.sock")
        p_read = p0.policy_digest(p0.Z32, p0.CC["NV_Read"])
        want = name(HAUTH, ph.ATTR_HAUTH, p_read, 64)
        want_w = name(HAUTH, ph.ATTR_HAUTH | WRITTEN, p_read, 64)
        log(f"# NV_HAUTH Name = {want.hex()}")
        log(f"# NV_HAUTH Name (WRITTEN) = {want_w.hex()}")

        log("\n### N1: Name of a correctly defined HAUTH, before and after the first write")
        check("N1 define HAUTH", define_by(t, p0.RH_OWNER, HAUTH, ph.ATTR_HAUTH, p_read, 64) == 0, "")
        rc, attrs, n = read_public(t, HAUTH)
        check("N1 Name before WRITTEN", n == want, n.hex() or decode_rc(rc))
        check("N1 write HAUTH", write(t, p0.RH_OWNER, HAUTH, os.urandom(64)) == 0, "")
        rc, attrs, n = read_public(t, HAUTH)
        check("N1 Name after WRITTEN", n == want_w and attrs & WRITTEN != 0, n.hex() or decode_rc(rc))
        check("N1 undefine", ph.undefine(t, HAUTH, p0.RH_OWNER) == 0, "")

        log("\n### S1: owner squat ATTR_HAUTH | AUTHREAD (empty ownerAuth)")
        squat = ph.ATTR_HAUTH | p0.AUTHREAD
        check("S1 squat defined", define_by(t, p0.RH_OWNER, HAUTH, squat, p_read, 64) == 0, "")
        rc, attrs, n = read_public(t, HAUTH)
        check("S1 Name differs from NV_HAUTH", rc == 0 and n not in (want, want_w), f"attrs 0x{attrs:08X}")
        rc = define_by(t, p0.RH_OWNER, HAUTH, ph.ATTR_HAUTH, p_read, 64)
        check("S1 loader DefineSpace refused", rc == RC_NV_DEFINED, decode_rc(rc))
        check("S1 owner NV_Write of the secret accepted", write(t, p0.RH_OWNER, HAUTH, secret) == 0, "")
        ph.extend_pcr13(t, b"BARYON-TRIAL-CLOSED-V1")
        rc, v = ph.read_pw(t, HAUTH, 64)
        check("S1 secret readable by password after PCR13 extend (leak)", v == secret, decode_rc(rc))
        check("S1 empty-ownerAuth owner undefine removes the squat", ph.undefine(t, HAUTH, p0.RH_OWNER) == 0, "")

        log("\n### S2: owner squat with POLICY_DELETE")
        rc = define_by(t, p0.RH_OWNER, HAUTH, squat | POLICY_DELETE, p_read, 64)
        log(f"   INFO owner define with POLICY_DELETE -> {decode_rc(rc)}")
        if rc == 0:
            rc = ph.undefine(t, HAUTH, p0.RH_OWNER)
            log(f"   INFO owner NV_UndefineSpace of a POLICY_DELETE index -> {decode_rc(rc)}")
            check("S2 TPM2_Clear(lockout, empty) on a fresh TPM", clear_lockout(t) == 0, "")
            rc, _, _ = read_public(t, HAUTH)
            log(f"   INFO POLICY_DELETE index after TPM2_Clear -> {decode_rc(rc)}")
        else:
            check("S2 owner define with POLICY_DELETE refused", True, decode_rc(rc))

        log("\n### S3: platform-created squat (platformAuth empty: U23 broken)")
        rc = define_by(t, ph.RH_PLATFORM, HAUTH, squat | PLATFORMCREATE, p_read, 64)
        check("S3 platform squat defined", rc == 0, decode_rc(rc))
        rc, attrs, n = read_public(t, HAUTH)
        check("S3 Name differs from NV_HAUTH", rc == 0 and n not in (want, want_w), f"attrs 0x{attrs:08X}")
        rc = ph.undefine(t, HAUTH, p0.RH_OWNER)
        check("S3 owner undefine refused", rc != 0, decode_rc(rc))
        check("S3 TPM2_Clear(lockout, empty)", clear_lockout(t) == 0, "")
        rc, attrs, n = read_public(t, HAUTH)
        check("S3 platform squat survives TPM2_Clear", rc == 0, decode_rc(rc))
        check("S3 platform undefine removes it", ph.undefine(t, HAUTH, ph.RH_PLATFORM) == 0, "")

        log("\n### S4: record squat (U26 handle open; 0x0100B305 stands in)")
        check("S4 owner squat AUTHWRITE | AUTHREAD",
              define_by(t, p0.RH_OWNER, RECORD, ph.AUTHWRITE | p0.AUTHREAD | p0.NO_DA, b"", 64) == 0, "")
        forged = b"\x77" * 64
        check("S4 empty-auth NV_Write of a forged record after PCR13 extend", write(t, RECORD, RECORD, forged) == 0, "")
        rc, v = ph.read_pw(t, RECORD, 64)
        check("S4 forged record reads back", v == forged, decode_rc(rc))
    finally:
        p0.ctrl_shutdown("ctl.sock")
        sw.wait(timeout=10)

    log("\n# SUMMARY")
    for label, ok, detail in p0.RESULTS:
        log(f"{'PASS' if ok else 'FAIL'}  {label}  {detail}")
    text = "\n".join(p0.OUT) + "\n"
    text = text.replace(secret.hex(), "<SECRET>")
    assert secret.hex() not in text
    with open("transcript-squat.txt", "w") as f:
        f.write(text)
    return 0 if all(ok for _, ok, _ in p0.RESULTS) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
