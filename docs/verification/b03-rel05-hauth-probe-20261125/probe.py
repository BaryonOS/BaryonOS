#!/usr/bin/env python3
"""B03-REL05 M1 hierarchy-auth feasibility probe (not acceptance evidence).

Raw TPM 2.0 commands against a disposable swtpm (TEST only). Checks the
owner-approved mitigation of the anchor undefine/redefine/replay revival
(ADR-B03-REL05-TPM-20261125, amendment M1-HAUTH):
  U21  a fresh TPM_NT_EXTEND index starts at 0^32 (first extend = H(0^32 || d));
  H1   with ownerAuth set, empty-auth NV_UndefineSpace / NV_DefineSpace refused;
  H2   the HAUTH index is readable only under PolicyPCR(13 = 0^32) + CC NV_Read;
  H3   with lockoutAuth set, empty-auth TPM2_Clear(lockout) refused;
  H4   authorised recovery: undefine + redefine + genesis with the secret;
  U24  platform-hierarchy reachability (swtpm: platformAuth empty, no firmware);
  PERM TPM_PT_PERMANENT ownerAuthSet/lockoutAuthSet track provisioning cuts.
The secret is generated per run from os.urandom and redacted in the transcript.

Usage: python3 -B probe.py <workdir>   (workdir: disposable, holds swtpm state)
"""
import importlib.util
import os
import shutil
import struct
import subprocess
import sys
import time

# Shared raw-command helpers of the S0 probe, loaded under a distinct name.
_S0 = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "b03-rel05-probe-20261125", "probe.py")
_spec = importlib.util.spec_from_file_location("b03_rel05_s0_probe", _S0)
p0 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(p0)
H, Z32, b2, decode_rc, log, check = p0.H, p0.Z32, p0.b2, p0.decode_rc, p0.log, p0.check

p0.CC.update(NV_UndefineSpace=0x122, Clear=0x126, HierarchyChangeAuth=0x129, NV_Extend=0x136,
             DictionaryAttackLockReset=0x139)
RH_LOCKOUT, RH_ENDORSEMENT, RH_PLATFORM = 0x4000000A, 0x4000000B, 0x4000000C
AUTHWRITE, OWNERWRITE, POLICYREAD = 1 << 2, 1 << 1, 1 << 19
NT_EXTEND = 4 << 4
ANCHOR0, HAUTH = 0x0100B310, 0x0100B304
ATTR_ANCHOR = NT_EXTEND | AUTHWRITE | p0.AUTHREAD | p0.NO_DA
ATTR_HAUTH = OWNERWRITE | POLICYREAD | p0.WRITEALL | p0.NO_DA


def pw(secret=b""):
    # attributes 0, as drivers/tpm-crb/src/codec.rs sends TPM_RS_PW
    a = struct.pack(">I", p0.RS_PW) + b2(b"") + b"\x00" + b2(secret)
    return struct.pack(">I", len(a)) + a


def define(t, index, attrs, policy, size, owner=b""):
    pub = struct.pack(">IHI", index, p0.ALG_SHA256, attrs) + b2(policy) + struct.pack(">H", size)
    return t.cmd(f"NV_DefineSpace 0x{index:08X}", "NV_DefineSpace", struct.pack(">I", p0.RH_OWNER),
                 pw(owner), b2(b"") + b2(pub))[0]


def undefine(t, index, hierarchy, auth=b""):
    return t.cmd(f"NV_UndefineSpace 0x{index:08X} by 0x{hierarchy:08X}", "NV_UndefineSpace",
                 struct.pack(">II", hierarchy, index), pw(auth))[0]


def extend(t, index, data):
    return t.cmd(f"NV_Extend 0x{index:08X}", "NV_Extend", struct.pack(">II", index, index), pw(),
                 b2(data))[0]


def read_pw(t, index, size):
    rc, r = t.cmd(f"NV_Read 0x{index:08X} (password)", "NV_Read", struct.pack(">II", index, index),
                  pw(), struct.pack(">HH", size, 0))
    return rc, (r[16:16 + size] if rc == 0 else None)


def read_policy(t, index, size):
    rc, s = p0.start_session(t)
    p0.satisfy(t, s, "NV_Read")
    rc, r = t.cmd(f"NV_Read 0x{index:08X} via policy session", "NV_Read",
                  struct.pack(">II", index, index), p0.policy_auth(s), struct.pack(">HH", size, 0))
    if rc:
        p0.flush(t, s)
    return rc, (r[16:16 + size] if rc == 0 else None)


def change_auth(t, hierarchy, old, new):
    return t.cmd(f"HierarchyChangeAuth 0x{hierarchy:08X}", "HierarchyChangeAuth",
                 struct.pack(">I", hierarchy), pw(old), b2(new))[0]


def permanent(t):
    rc, r = t.cmd("GetCapability TPM_PT_PERMANENT", "GetCapability", params=struct.pack(">III", 6, 0x200, 1))
    return struct.unpack(">I", r[-4:])[0]


def da_counter(t):
    rc, r = t.cmd("GetCapability TPM_PT_LOCKOUT_COUNTER", "GetCapability",
                  params=struct.pack(">III", 6, 0x20E, 1))
    return struct.unpack(">I", r[-4:])[0]


def extend_pcr13(t, event):
    t.cmd("PCR_Extend 13", "PCR_Extend", struct.pack(">I", 13), pw(), struct.pack(">IH", 1, p0.ALG_SHA256) + H(event))



def main(work):
    work = os.path.abspath(work)
    state = os.path.join(work, "state-hauth")
    shutil.rmtree(state, ignore_errors=True)
    os.makedirs(state)
    os.chdir(work)
    sw = subprocess.Popen(["swtpm", "socket", "--tpm2", "--tpmstate", "dir=state-hauth",
                           "--server", "type=unixio,path=srv.sock", "--ctrl", "type=unixio,path=ctl.sock",
                           "--flags", "not-need-init,startup-clear", "--log", "file=swtpm-hauth.log"])
    owner_s, lockout_s = os.urandom(32), os.urandom(32)
    try:
        for _ in range(100):
            if os.path.exists("srv.sock"):
                break
            time.sleep(0.05)
        t = p0.Tpm("srv.sock")
        p_read = p0.policy_digest(Z32, p0.CC["NV_Read"])
        log(f"# host authPolicy HAUTH (PolicyPCR(13=0) + CC NV_Read) = {p_read.hex()}")
        log(f"# ATTR_ANCHOR = 0x{ATTR_ANCHOR:08X}  ATTR_HAUTH = 0x{ATTR_HAUTH:08X}")
        perm0 = permanent(t)
        check("PERM fresh TPM: ownerAuthSet/lockoutAuthSet clear", perm0 & 0b101 == 0, f"0x{perm0:08X}")

        log("\n### U21: fresh extend index")
        check("define anchor 0", define(t, ANCHOR0, ATTR_ANCHOR, b"", 32) == 0, "")
        rc, _ = read_pw(t, ANCHOR0, 32)
        check("U21 read before first extend refused", rc != 0, decode_rc(rc))
        d = H(b"genesis-event")
        check("U21 NV_Extend", extend(t, ANCHOR0, d) == 0, "")
        rc, v = read_pw(t, ANCHOR0, 32)
        check("U21 value == H(0^32 || d)", v == H(Z32 + d), v.hex() if v else "")
        check("U21 2nd extend", extend(t, ANCHOR0, d) == 0, "")
        rc, v2 = read_pw(t, ANCHOR0, 32)
        check("U22-host value == H(prev || d)", v2 == H(v + d), "")

        log("\n### Residual before mitigation: empty ownerAuth undefine succeeds")
        check("M1 residual: undefine anchor with empty ownerAuth", undefine(t, ANCHOR0, p0.RH_OWNER) == 0, "")
        check("redefine anchor", define(t, ANCHOR0, ATTR_ANCHOR, b"", 32) == 0, "")
        extend(t, ANCHOR0, d)

        log("\n### Provisioning: HAUTH index, then HierarchyChangeAuth")
        check("define HAUTH", define(t, HAUTH, ATTR_HAUTH, p_read, 64) == 0, "")
        rc = t.cmd("NV_Write HAUTH (owner auth, empty)", "NV_Write", struct.pack(">II", p0.RH_OWNER, HAUTH), pw(),
                   b2(owner_s + lockout_s) + struct.pack(">H", 0))[0]
        check("write HAUTH with owner (still empty)", rc == 0, decode_rc(rc))
        perm = permanent(t)
        check("PERM cut after HAUTH write: ownerAuthSet clear", perm & 1 == 0, f"0x{perm:08X}")
        check("HierarchyChangeAuth owner", change_auth(t, p0.RH_OWNER, b"", owner_s) == 0, "")
        perm = permanent(t)
        check("PERM cut after owner change: ownerAuthSet set, lockoutAuthSet clear", perm & 0b101 == 0b001, f"0x{perm:08X}")
        check("HierarchyChangeAuth lockout", change_auth(t, RH_LOCKOUT, b"", lockout_s) == 0, "")
        perm = permanent(t)
        check("PERM provisioned: ownerAuthSet + lockoutAuthSet", perm & 0b101 == 0b101, f"0x{perm:08X}")

        log("\n### H2: HAUTH read only at PCR13 = 0")
        rc, _ = read_pw(t, HAUTH, 64)
        check("H2 password read of HAUTH refused", rc != 0, decode_rc(rc))
        rc, v = read_policy(t, HAUTH, 64)
        check("H2 policy read at PCR13=0 returns the secret", v == owner_s + lockout_s, decode_rc(rc))

        log("\n### H1: empty-auth undefine/define refused (kernel phase simulated after PCR13 extend)")
        extend_pcr13(t, b"BARYON-TRIAL-CLOSED-V1")
        da0 = da_counter(t)
        rc = undefine(t, ANCHOR0, p0.RH_OWNER)
        check("H1 undefine anchor with empty ownerAuth refused", rc != 0, decode_rc(rc))
        log(f"   INFO DA lockout counter {da0} -> {da_counter(t)} after one wrong ownerAuth")
        rc = define(t, ANCHOR0 + 1, ATTR_ANCHOR, b"", 32)
        check("H1 define with empty ownerAuth refused", rc != 0, decode_rc(rc))
        rc = undefine(t, HAUTH, p0.RH_OWNER)
        check("H1 undefine HAUTH with empty ownerAuth refused", rc != 0, decode_rc(rc))
        rc, _ = read_policy(t, HAUTH, 64)
        check("H2 policy read after PCR13 extend refused", rc != 0, decode_rc(rc))
        rc = t.cmd("Clear via lockout (empty)", "Clear", struct.pack(">I", RH_LOCKOUT), pw())[0]
        check("H3 Clear(lockout) with empty auth refused", rc != 0, decode_rc(rc))
        rc = change_auth(t, RH_ENDORSEMENT, b"", b"x" * 8)
        log(f"   INFO HierarchyChangeAuth(endorsement) with empty auth -> {decode_rc(rc)} (EK-check denial path)")

        log("\n### H4: authorised recovery with the secret")
        log(f"   INFO DA lockout counter before recovery = {da_counter(t)}")
        check("H4 undefine anchor with ownerAuth", undefine(t, ANCHOR0, p0.RH_OWNER, owner_s) == 0, "")
        check("H4 redefine anchor with ownerAuth", define(t, ANCHOR0, ATTR_ANCHOR, b"", 32, owner_s) == 0, "")
        check("H4 genesis extend", extend(t, ANCHOR0, H(b"fresh-instance")) == 0, "")

        log("\n### U24: platform hierarchy (swtpm without firmware: platformAuth empty)")
        rc = undefine(t, ANCHOR0, RH_PLATFORM)
        log(f"   INFO platform NV_UndefineSpace of an owner index -> {decode_rc(rc)}")
        rc = t.cmd("Clear via platform (empty)", "Clear", struct.pack(">I", RH_PLATFORM), pw())[0]
        log(f"   INFO platform TPM2_Clear -> {decode_rc(rc)}")
        perm = permanent(t)
        log(f"   INFO PERM after platform clear attempt = 0x{perm:08X}")
        rc, _ = read_pw(t, HAUTH, 64)
        log(f"   INFO HAUTH after platform clear attempt -> {decode_rc(rc)}")
    finally:
        p0.ctrl_shutdown("ctl.sock")
        sw.wait(timeout=10)

    log("\n# SUMMARY")
    for name, ok, detail in p0.RESULTS:
        log(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")
    redact = {owner_s.hex(): "<OWNER-SECRET>", lockout_s.hex(): "<LOCKOUT-SECRET>"}
    text = "\n".join(p0.OUT) + "\n"
    for k, v in redact.items():
        text = text.replace(k, v)
    assert owner_s.hex() not in text and lockout_s.hex() not in text
    with open("transcript-hauth.txt", "w") as f:
        f.write(text)
    return 0 if all(ok for _, ok, _ in p0.RESULTS) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
