#!/usr/bin/env python3
"""B03-REL05-P0 feasibility probe (not acceptance evidence).

Sends raw TPM 2.0 command bytes to a disposable swtpm to test spec
assumptions U1 (unbound unsalted policy session with PolicyPCR +
PolicyCommandCode authorises NV_Write/NV_Increment with empty nonce/hmac)
and U7 (PCR 13 not resettable at locality 0). No tpm2-tools, no HMAC code.

Usage: python3 -B probe.py <workdir>   (workdir: disposable, holds swtpm state)
"""
import hashlib
import os
import shutil
import socket
import struct
import subprocess
import sys
import time

H = lambda b: hashlib.sha256(b).digest()
Z32 = bytes(32)

# Constants (TPM 2.0 Library Part 2)
ST_NO_SESSIONS, ST_SESSIONS = 0x8001, 0x8002
RH_OWNER, RH_NULL, RS_PW = 0x40000001, 0x40000007, 0x40000009
ALG_SHA256, ALG_NULL = 0x000B, 0x0010
SE_POLICY = 0x01
CC = dict(NV_DefineSpace=0x12A, PCR_Reset=0x13D, NV_Increment=0x134, NV_Write=0x137,
          NV_Read=0x14E, PolicyCommandCode=0x16C, NV_ReadPublic=0x169, FlushContext=0x165,
          StartAuthSession=0x176, PCR_Read=0x17E, PolicyPCR=0x17F, PCR_Extend=0x182,
          PolicyGetDigest=0x189, GetCapability=0x17A)
# TPMA_NV bits
POLICYWRITE, WRITEALL, AUTHREAD, NO_DA = 1 << 3, 1 << 12, 1 << 18, 1 << 25
NT_COUNTER = 1 << 4

# Spec section 6.1 layout
PHASE_PCR = 13
OPEN_EVENT = b"BARYON-TRIAL-OPEN-V1"
P_OPEN = H(Z32 + H(OPEN_EVENT))
TRIAL, CONFIRM, FLOOR, GEN = 0x0100B300, 0x0100B301, 0x0100B302, 0x0100B303

OUT = []


def log(line=""):
    print(line)
    OUT.append(line)


def b2(data):
    return struct.pack(">H", len(data)) + data


def pcr_select(pcr):
    sel = bytearray(3)
    sel[pcr // 8] |= 1 << (pcr % 8)
    return struct.pack(">IHB", 1, ALG_SHA256, 3) + bytes(sel)


def policy_digest(pcr_value, cc):
    """Host-side constant exactly as spec 6.1 (what S2 would freeze)."""
    d1 = H(Z32 + struct.pack(">I", CC["PolicyPCR"]) + pcr_select(PHASE_PCR) + H(pcr_value))
    return H(d1 + struct.pack(">I", CC["PolicyCommandCode"]) + struct.pack(">I", cc))


def pw_auth():
    # TPMS_AUTH_COMMAND: TPM_RS_PW, empty nonce, attrs continueSession, empty hmac
    a = struct.pack(">I", RS_PW) + b2(b"") + b"\x01" + b2(b"")
    return struct.pack(">I", len(a)) + a


def policy_auth(session, hmac=b"", attrs=0x00):
    # Policy session: empty nonceCaller, attrs (continueSession clear), hmac as given
    a = struct.pack(">I", session) + b2(b"") + bytes([attrs]) + b2(hmac)
    return struct.pack(">I", len(a)) + a


RC_NAMES = {0x000: "TPM_RC_SUCCESS", 0x09D: "TPM_RC_POLICY_FAIL", 0x124: "TPM_RC_POLICY_CC",
            0x128: "TPM_RC_PCR_CHANGED", 0x907: "TPM_RC_LOCALITY (warning)", 0x095: "TPM_RC_SIZE",
            0x14A: "TPM_RC_NV_RANGE", 0x14C: "TPM_RC_NV_DEFINED", 0x14D: "TPM_RC_NV_UNINITIALIZED",
            0x149: "TPM_RC_NV_AUTHORIZATION", 0x08E: "TPM_RC_AUTH_FAIL", 0x08F: "TPM_RC_NONCE",
            0x085: "TPM_RC_HANDLE", 0x0A2: "TPM_RC_BAD_AUTH", 0x084: "TPM_RC_VALUE"}


def decode_rc(rc):
    if rc == 0:
        return "TPM_RC_SUCCESS"
    if rc & 0x80:  # format 1
        base = (rc & 0x3F) | 0x80
        n = (rc >> 8) & 0xF
        where = "parameter" if rc & 0x40 else ("session" if n & 0x8 else "handle")
        idx = n if rc & 0x40 else (n & 0x7)
        return f"{RC_NAMES.get(base, hex(base))} ({where} {idx})"
    return RC_NAMES.get(rc & 0xFFF, hex(rc))


def ctrl_shutdown(path):
    """swtpm CMD_SHUTDOWN (0x03) over the control socket; the sandbox may forbid signals."""
    c = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    c.connect(path)
    c.sendall(struct.pack(">I", 3))
    c.recv(4)
    c.close()


class Tpm:
    def __init__(self, path):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(path)

    def raw(self, cmd):
        self.sock.sendall(cmd)
        hdr = b""
        while len(hdr) < 10:
            hdr += self.sock.recv(10 - len(hdr))
        size = struct.unpack(">I", hdr[2:6])[0]
        body = hdr
        while len(body) < size:
            body += self.sock.recv(size - len(body))
        return body

    def cmd(self, label, cc, handles=b"", auth=None, params=b""):
        tag = ST_SESSIONS if auth is not None else ST_NO_SESSIONS
        body = handles + (auth or b"") + params
        c = struct.pack(">HII", tag, 10 + len(body), CC[cc]) + body
        r = self.raw(c)
        rc = struct.unpack(">I", r[6:10])[0]
        log(f"== {label}")
        log(f"   cmd  {c.hex()}")
        log(f"   resp {r.hex()}")
        log(f"   rc   0x{rc:03X} {decode_rc(rc)}")
        return rc, r


def define(t, index, attrs, policy, size):
    pub = struct.pack(">IHI", index, ALG_SHA256, attrs) + b2(policy) + struct.pack(">H", size)
    return t.cmd(f"NV_DefineSpace 0x{index:08X}", "NV_DefineSpace", struct.pack(">I", RH_OWNER),
                 pw_auth(), b2(b"") + b2(pub))[0]


def read_public_policy(t, index):
    rc, r = t.cmd(f"NV_ReadPublic 0x{index:08X}", "NV_ReadPublic", struct.pack(">I", index))
    # resp: hdr(10) TPM2B_NV_PUBLIC{size u16, index u32, nameAlg u16, attrs u32, authPolicy TPM2B, dataSize}
    off = 12
    _, _, attrs = struct.unpack(">IHI", r[off:off + 10])
    plen = struct.unpack(">H", r[off + 10:off + 12])[0]
    return attrs, r[off + 12:off + 12 + plen]


def start_session(t, nonce=b"BaryonTrialNonce"):
    rc, r = t.cmd("StartAuthSession POLICY unbound unsalted", "StartAuthSession",
                  struct.pack(">II", RH_NULL, RH_NULL), None,
                  b2(nonce) + b2(b"") + bytes([SE_POLICY]) + struct.pack(">H", ALG_NULL)
                  + struct.pack(">H", ALG_SHA256))
    return rc, (struct.unpack(">I", r[10:14])[0] if rc == 0 else None)


def satisfy(t, s, cc, pcr_digest=b""):
    t.cmd("PolicyPCR(13), empty pcrDigest" if not pcr_digest else "PolicyPCR(13), explicit pcrDigest",
          "PolicyPCR", struct.pack(">I", s), None, b2(pcr_digest) + pcr_select(PHASE_PCR))
    t.cmd(f"PolicyCommandCode({cc})", "PolicyCommandCode", struct.pack(">I", s), None,
          struct.pack(">I", CC[cc]))
    rc, r = t.cmd("PolicyGetDigest", "PolicyGetDigest", struct.pack(">I", s))
    return r[12:44]


def flush(t, s):
    t.cmd(f"FlushContext 0x{s:08X}", "FlushContext", params=struct.pack(">I", s))


def nv_write(t, index, s, data, hmac=b""):
    return t.cmd(f"NV_Write 0x{index:08X} via policy session", "NV_Write",
                 struct.pack(">II", index, index), policy_auth(s, hmac), b2(data) + struct.pack(">H", 0))[0]


def nv_inc(t, index, s):
    return t.cmd(f"NV_Increment 0x{index:08X} via policy session", "NV_Increment",
                 struct.pack(">II", index, index), policy_auth(s))[0]


def nv_read(t, index, size):
    rc, r = t.cmd(f"NV_Read 0x{index:08X} (password, empty auth)", "NV_Read",
                  struct.pack(">II", index, index), pw_auth(), struct.pack(">HH", size, 0))
    return rc, (r[16:16 + size] if rc == 0 else None)


def pcr13(t):
    rc, r = t.cmd("PCR_Read sha256 PCR13", "PCR_Read", params=pcr_select(PHASE_PCR))
    return r[-32:]


def try_write(t, index, cc, expect_ok, data=None, hmac=b""):
    """One policy session per attempt; flush it if the TPM did not consume it."""
    rc, s = start_session(t)
    got = satisfy(t, s, cc)
    if cc == "NV_Write":
        rc = nv_write(t, index, s, data, hmac)
    else:
        rc = nv_inc(t, index, s)
    if rc != 0:
        flush(t, s)  # a failed authorisation leaves the session loaded (checked via FlushContext rc)
    return rc, got


RESULTS = []


def check(name, ok, detail):
    RESULTS.append((name, ok, detail))
    log(f"** CHECK {'PASS' if ok else 'FAIL'}: {name}: {detail}")


def main(work):
    work = os.path.abspath(work)
    state = os.path.join(work, "state-u1u7")
    shutil.rmtree(state, ignore_errors=True)
    os.makedirs(state)
    os.chdir(work)
    sw = subprocess.Popen(["swtpm", "socket", "--tpm2", "--tpmstate", "dir=state-u1u7",
                           "--server", "type=unixio,path=srv.sock", "--ctrl", "type=unixio,path=ctl.sock",
                           "--flags", "not-need-init,startup-clear", "--log", "file=swtpm-u1u7.log"])
    try:
        for _ in range(100):
            if os.path.exists("srv.sock"):
                break
            time.sleep(0.05)
        t = Tpm("srv.sock")
        log("# Locality: never changed via ctrl channel -> swtpm default locality 0")
        p_trial = policy_digest(Z32, CC["NV_Write"])
        p_gen = policy_digest(Z32, CC["NV_Increment"])
        p_conf = policy_digest(P_OPEN, CC["NV_Write"])
        log(f"# host authPolicy TRIAL/FLOOR = {p_trial.hex()}")
        log(f"# host authPolicy GEN         = {p_gen.hex()}")
        log(f"# host authPolicy CONFIRM     = {p_conf.hex()}")
        log(f"# P_OPEN                      = {P_OPEN.hex()}")

        base = POLICYWRITE | AUTHREAD | NO_DA
        for idx, attrs, pol, size in ((TRIAL, base | WRITEALL, p_trial, 64), (CONFIRM, base | WRITEALL, p_conf, 64),
                                      (FLOOR, base | WRITEALL, p_trial, 64), (GEN, base | NT_COUNTER, p_gen, 8)):
            check(f"define 0x{idx:08X}", define(t, idx, attrs, pol, size) == 0, "")
            a, p = read_public_policy(t, idx)
            check(f"NV_ReadPublic authPolicy 0x{idx:08X} == host constant", p == pol, f"attrs=0x{a:08X}")

        check("PCR13 initially zero", pcr13(t) == Z32, "")
        rec = b"NOSTRL01" + bytes(56)

        log("\n### Phase A: PCR13 == 0")
        rc, got = try_write(t, TRIAL, "NV_Write", True, rec)
        check("U1+ PolicyGetDigest == TRIAL authPolicy", got == p_trial, got.hex())
        check("U1+ NV_Write TRIAL (empty nonce, empty hmac)", rc == 0, decode_rc(rc))
        rc, data = nv_read(t, TRIAL, 64)
        check("NV_Read TRIAL == written", data == rec, "")
        rc, _ = try_write(t, FLOOR, "NV_Write", True, b"NOSTFL01" + bytes(56))
        check("U1+ NV_Write FLOOR", rc == 0, decode_rc(rc))
        rc, got = try_write(t, GEN, "NV_Increment", True)
        check("U1+ PolicyGetDigest == GEN authPolicy", got == p_gen, got.hex())
        check("U1+ NV_Increment GEN", rc == 0, decode_rc(rc))
        rc, data = nv_read(t, GEN, 8)
        log(f"   GEN value after first increment = {data.hex() if data else None}")
        rc, _ = try_write(t, CONFIRM, "NV_Write", False, rec)
        check("U1- NV_Write CONFIRM refused at PCR13=0", rc != 0, decode_rc(rc))
        # Wrong command code in policy: session bound to NV_Write used for NV_Increment
        rc, s = start_session(t)
        satisfy(t, s, "NV_Write")
        rc = nv_inc(t, GEN, s)
        check("U1- NV_Increment GEN with NV_Write-bound session refused", rc != 0, decode_rc(rc))
        if rc:
            flush(t, s)
        # Non-empty garbage hmac on an otherwise valid session (informational)
        rc, _ = try_write(t, TRIAL, "NV_Write", None, rec, hmac=bytes(32))
        log(f"   INFO garbage 32-byte hmac on valid policy session -> {decode_rc(rc)}")
        # Stale session: PolicyPCR at 0, then PCR13 extended, then write
        rc, stale = start_session(t)
        satisfy(t, stale, "NV_Write")

        log("\n### Extend PCR13 with H(OPEN) (SHA-256 bank only)")
        t.cmd("PCR_Extend 13 sha256 H(OPEN)", "PCR_Extend", struct.pack(">I", PHASE_PCR), pw_auth(),
              struct.pack(">IH", 1, ALG_SHA256) + H(OPEN_EVENT))
        check("PCR13 == P_OPEN", pcr13(t) == P_OPEN, "")

        log("\n### Phase B: PCR13 == P_OPEN")
        rc = nv_write(t, TRIAL, stale, rec)
        check("U1- stale session (PolicyPCR before extend) refused", rc != 0, decode_rc(rc))
        if rc:
            flush(t, stale)
        rc, got = try_write(t, CONFIRM, "NV_Write", True, rec)
        check("U1+ PolicyGetDigest == CONFIRM authPolicy", got == p_conf, got.hex())
        check("U1+ NV_Write CONFIRM at P_OPEN", rc == 0, decode_rc(rc))
        rc, data = nv_read(t, CONFIRM, 64)
        check("NV_Read CONFIRM == written", data == rec, "")
        for idx, cc in ((TRIAL, "NV_Write"), (FLOOR, "NV_Write"), (GEN, "NV_Increment")):
            rc, _ = try_write(t, idx, cc, False, rec)
            check(f"U1- {cc} 0x{idx:08X} refused at P_OPEN", rc != 0, decode_rc(rc))

        log("\n### U7: PCR_Reset at locality 0")
        rc, _ = t.cmd("PCR_Reset 13", "PCR_Reset", struct.pack(">I", 13), pw_auth())
        check("U7 PCR_Reset(13) refused at locality 0", rc != 0, decode_rc(rc))
        check("U7 PCR13 unchanged (still P_OPEN)", pcr13(t) == P_OPEN, "")
        for p in (16, 23):
            rc, _ = t.cmd(f"PCR_Reset {p} (positive control)", "PCR_Reset", struct.pack(">I", p), pw_auth())
            check(f"U7 control PCR_Reset({p}) succeeds at locality 0", rc == 0, decode_rc(rc))

        log("\n### Incidental U3: StartAuthSession with 0-byte nonceCaller")
        rc, s = start_session(t, b"")
        log(f"   INFO 0-byte nonceCaller -> {decode_rc(rc)}")
        if rc == 0:
            flush(t, s)
        rc, s = start_session(t, b"x" * 15)
        log(f"   INFO 15-byte nonceCaller -> {decode_rc(rc)}")
        if rc == 0:
            flush(t, s)
    finally:
        ctrl_shutdown("ctl.sock")
        sw.wait(timeout=10)

    log("\n# SUMMARY")
    for name, ok, detail in RESULTS:
        log(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")
    return 0 if all(ok for _, ok, _ in RESULTS) else 1


if __name__ == "__main__":
    rc = main(sys.argv[1])
    with open("transcript-u1u7.txt", "w") as f:
        f.write("\n".join(OUT) + "\n")
    sys.exit(rc)
