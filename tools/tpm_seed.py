"""Pre-seeded disposable swtpm states for the negative M1-HAUTH cases of run_tpm.py (TEST only).

Raw TPM 2.0 commands over a short-lived swtpm `--server` socket on a state
directory under the build tree, while PCR 13 is at its reset value (the
boot-path position of threat T5, out of scope until B04). No command or
response bytes are printed or stored: the foreign ownerAuth, the scratch
authValue and the HAUTH secret of the lockout cut are fresh TEST values that
are discarded. `observe` reads back only public facts.
"""
from __future__ import annotations

from contextlib import contextmanager
import hashlib
import os
from pathlib import Path
import socket
import struct
import subprocess
import time

HAUTH, LREC = 0x0100B304, 0x0100B305
# LoaderRecord BOUND (crates/update-core/src/trial.rs): magic, 40 zero bytes, SHA-256(HAUTH)[:16].
LREC_MAGIC, LREC_BINDING_LEN = b"NOSLRC01", 16
# TEST scratch index for the DA seed; defined and undefined within that seed.
SCRATCH = 0x0100B3DA
RH_OWNER, RH_NULL, RS_PW, RH_PLATFORM = 0x40000001, 0x40000007, 0x40000009, 0x4000000C
CC = {"NV_UndefineSpace": 0x122, "HierarchyChangeAuth": 0x129, "NV_DefineSpace": 0x12A,
      "NV_Write": 0x137, "NV_Read": 0x14E, "NV_ReadPublic": 0x169, "PolicyCommandCode": 0x16C,
      "StartAuthSession": 0x176, "GetCapability": 0x17A, "PolicyPCR": 0x17F}
# TPM_PT_PERMANENT and TPM_PT_LOCKOUT_COUNTER, as the loader reads them.
PT_PERMANENT, PT_LOCKOUT_COUNTER = 0x200, 0x20E
# TPM_RC_AUTH_FAIL on session 1: a wrong authValue on a DA-protected entity.
RC_AUTH_FAIL = 0x98E
# TPM_RC_RETRY: the command did not run; resend it. libtpms 0.9 (swtpm 0.7.3 in CI,
# USE_DA_USED) answers the first DA-checked authorization after Startup this way once.
RC_RETRY, RETRIES = 0x922, 3
# TPMA_NV (Part 2, 13.4) and the frozen templates of crates/update-core/src/trial_tpm.rs.
OWNERWRITE, AUTHWRITE, POLICYWRITE, WRITEALL = 1 << 1, 1 << 2, 1 << 3, 1 << 12
AUTHREAD, POLICYREAD, NO_DA, WRITTEN, PLATFORMCREATE = 1 << 18, 1 << 19, 1 << 25, 1 << 29, 1 << 30
ATTR_HAUTH = OWNERWRITE | POLICYREAD | WRITEALL | NO_DA
ATTR_LREC = POLICYWRITE | WRITEALL | AUTHREAD | NO_DA
POLICY_READ = bytes.fromhex("339de5436134b5c65b80a51f1efae32ed9d89f664bf5e132551e35c7ad82ae82")
POLICY_WRITE = bytes.fromhex("088d4acf1b7ad7d6a53a44d97668736524219be4e7148b45be4dcc3d00462816")
PCR13 = struct.pack(">IHB3s", 1, 0x000B, 3, b"\x00\x20\x00")


def b2(data):
    return struct.pack(">H", len(data)) + data


def password(secret=b""):
    area = struct.pack(">I", RS_PW) + b2(b"") + b"\x00" + b2(secret)
    return struct.pack(">I", len(area)) + area


class Tpm:
    def __init__(self, path):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.settimeout(10)
        self.sock.connect(str(path))

    def read(self, size):
        data = b""
        while len(data) < size:
            chunk = self.sock.recv(size - len(data))
            if not chunk:
                raise RuntimeError("swtpm closed the command socket")
            data += chunk
        return data

    def cmd(self, cc, handles=b"", auth=None, params=b""):
        """Response code and response bytes of one command."""
        body = handles + (auth or b"") + params
        for _ in range(RETRIES):
            self.sock.sendall(struct.pack(">HII", 0x8002 if auth else 0x8001, 10 + len(body), CC[cc]) + body)
            head = self.read(10)
            size, rc = struct.unpack(">I", head[2:6])[0], struct.unpack(">I", head[6:10])[0]
            if not 10 <= size <= 4096:
                raise RuntimeError("Invalid swtpm response size")
            response = head + self.read(size - 10)
            if rc != RC_RETRY:
                break
        return rc, response

    def ok(self, cc, *args, **kwargs):
        rc, response = self.cmd(cc, *args, **kwargs)
        if rc:
            raise RuntimeError(f"Seed command {cc} failed with TPM_RC 0x{rc:03X}")
        return response

    def define(self, hierarchy, index, attributes, policy=b"", size=64, auth=b""):
        public = struct.pack(">IHI", index, 0x000B, attributes) + b2(policy) + struct.pack(">H", size)
        self.ok("NV_DefineSpace", struct.pack(">I", hierarchy), password(), b2(auth) + b2(public))

    def public(self, index):
        """(attributes, Name) of a defined index, or the TPM_RC of NV_ReadPublic."""
        rc, r = self.cmd("NV_ReadPublic", struct.pack(">I", index))
        if rc:
            return rc
        size = struct.unpack(">H", r[10:12])[0]
        attributes = struct.unpack(">I", r[18:22])[0]  # size, nvIndex, nameAlg, attributes
        name = r[12 + size:]
        return attributes, name[2:2 + struct.unpack(">H", name[:2])[0]]

    def property(self, pt):
        """One TPM_CAP_TPM_PROPERTIES value."""
        r = self.ok("GetCapability", params=struct.pack(">III", 6, pt, 1))
        if struct.unpack(">I", r[-8:-4])[0] != pt:
            raise RuntimeError(f"GetCapability did not return property 0x{pt:X}")
        return struct.unpack(">I", r[-4:])[0]

    def lrec(self):
        r = self.ok("NV_Read", struct.pack(">II", LREC, LREC), password(), struct.pack(">HH", 64, 0))
        return r[16:80]

    def write_lrec(self, data):
        """Loader-only policy write of LREC: PolicyPCR(13 = 0) + PolicyCommandCode(NV_Write)."""
        r = self.ok("StartAuthSession", struct.pack(">II", RH_NULL, RH_NULL), None,
                    b2(os.urandom(16)) + b2(b"") + b"\x01" + struct.pack(">HH", 0x0010, 0x000B))
        session = struct.pack(">I", struct.unpack(">I", r[10:14])[0])
        self.ok("PolicyPCR", session, None, b2(b"") + PCR13)
        self.ok("PolicyCommandCode", session, None, struct.pack(">I", CC["NV_Write"]))
        auth = session + b2(b"") + b"\x00" + b2(b"")
        self.ok("NV_Write", struct.pack(">II", LREC, LREC), struct.pack(">I", len(auth)) + auth,
                b2(data) + struct.pack(">H", 0))


def squat(tpm):
    """C21: a PLATFORMCREATE HAUTH (empty platformAuth before firmware) that owner undefine cannot remove."""
    tpm.define(RH_PLATFORM, HAUTH, PLATFORMCREATE | AUTHWRITE | AUTHREAD | NO_DA)


def foreign_owner(tpm):
    """ownerAuth set by someone else before the first provisioning; no HAUTH or LREC."""
    tpm.ok("HierarchyChangeAuth", struct.pack(">I", RH_OWNER), password(), b2(os.urandom(32)))


def owner_squat_empty(tpm):
    """Probe S1 squat: an owner AUTHREAD | AUTHWRITE HAUTH under the empty ownerAuth."""
    tpm.define(RH_OWNER, HAUTH, AUTHWRITE | AUTHREAD | NO_DA)


def owner_squat(tpm):
    """The probe S1 squat, then a foreign ownerAuth that keeps it."""
    owner_squat_empty(tpm)
    foreign_owner(tpm)


def da_counted(tpm):
    """One DA-counted failure: a wrong (empty) authValue on a DA-protected scratch index, then
    the index is removed again, so only TPM_PT_LOCKOUT_COUNTER differs from a fresh state."""
    before = tpm.property(PT_LOCKOUT_COUNTER)
    tpm.define(RH_OWNER, SCRATCH, AUTHWRITE | AUTHREAD, size=8, auth=os.urandom(16))
    rc, _ = tpm.cmd("NV_Read", struct.pack(">II", SCRATCH, SCRATCH), password(), struct.pack(">HH", 8, 0))
    if rc != RC_AUTH_FAIL or tpm.property(PT_LOCKOUT_COUNTER) != before + 1:
        raise RuntimeError(f"The scratch failure was not DA-counted (TPM_RC 0x{rc:03X})")
    tpm.ok("NV_UndefineSpace", struct.pack(">II", RH_OWNER, SCRATCH), password())


def unbound(tpm):
    """After a real provisioning boot: one flipped binding byte of a well-formed LREC."""
    record = bytearray(tpm.lrec())
    record[48] ^= 0x01
    tpm.write_lrec(bytes(record))


# Power cuts after each step of the loader's provisioning sequence, in order.
def hauth_defined(tpm):
    """Power cut after defining HAUTH: an unwritten HAUTH, no LREC, no hierarchy auth."""
    tpm.define(RH_OWNER, HAUTH, ATTR_HAUTH, POLICY_READ)


def hauth_written(tpm, secret=None):
    """Power cut after writing HAUTH: a written HAUTH, no LREC, no hierarchy auth."""
    hauth_defined(tpm)
    tpm.ok("NV_Write", struct.pack(">II", RH_OWNER, HAUTH), password(),
           b2(secret or os.urandom(64)) + struct.pack(">H", 0))


def cut(tpm, secret=None):
    """Power cut after defining LREC: written HAUTH leftover, unwritten LREC, no hierarchy auth."""
    hauth_written(tpm, secret)
    tpm.define(RH_OWNER, LREC, ATTR_LREC, POLICY_WRITE)


def lrec_written(tpm, secret=None):
    """Power cut after writing LREC: HAUTH and a BOUND LREC, ownerAuth still empty."""
    secret = secret or os.urandom(64)
    cut(tpm, secret)
    tpm.write_lrec(LREC_MAGIC + bytes(40) + hashlib.sha256(secret).digest()[:LREC_BINDING_LEN])


def lockout_cut(tpm):
    """Power cut between the ownerAuth and lockoutAuth changes of a provisioning boot: HAUTH and
    a BOUND LREC written, ownerAuth = HAUTH[:32], lockoutAuth still empty."""
    secret = os.urandom(64)
    lrec_written(tpm, secret)
    tpm.ok("HierarchyChangeAuth", struct.pack(">I", RH_OWNER), password(), b2(secret[:32]))


SEEDS = {"squat": squat, "foreign-owner": foreign_owner, "owner-squat": owner_squat,
         "owner-squat-empty": owner_squat_empty, "da-counted": da_counted,
         "unbound": unbound, "hauth-defined": hauth_defined, "hauth-written": hauth_written,
         "cut": cut, "lrec-written": lrec_written, "lockout-cut": lockout_cut}


def observe(tpm):
    """Public facts only: PERMANENT, the DA lockout counter, both public areas, and a digest
    of the non-secret LREC."""
    facts = {"permanent": tpm.property(PT_PERMANENT),
             "lockout_counter": tpm.property(PT_LOCKOUT_COUNTER)}
    for label, index in (("hauth", HAUTH), ("lrec", LREC)):
        public = tpm.public(index)
        facts[label] = ({"rc": public} if isinstance(public, int)
                        else {"attributes": public[0], "name": public[1].hex()})
    if facts["lrec"].get("attributes", 0) & WRITTEN:
        facts["lrec"]["sha256"] = hashlib.sha256(tpm.lrec()).hexdigest()
    return facts


@contextmanager
def server(binary, env, directory: Path):
    """A swtpm with a command socket on `directory/state`, fully stopped on exit."""
    command, control = directory / "seed.sock", directory / "seed-ctrl.sock"
    # Fail before starting swtpm: a connect that fails later would leave it running.
    if len(os.fsencode(control)) > 107:
        raise ValueError("Use a shorter state directory for the bounded Unix socket name")
    for path in (command, control):
        path.unlink(missing_ok=True)
    process = subprocess.Popen(
        [str(binary), "socket", "--tpm2", "--tpmstate", "dir=state",
         "--server", "type=unixio,path=seed.sock", "--ctrl", "type=unixio,path=seed-ctrl.sock,mode=0600",
         "--flags", "not-need-init,startup-clear"],
        cwd=directory, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT)
    try:
        deadline = time.monotonic() + 5
        while not command.is_socket():
            if process.poll() is not None or time.monotonic() >= deadline:
                raise RuntimeError("Seed swtpm did not start")
            time.sleep(0.01)
        tpm = Tpm(command)
        try:
            yield tpm
        finally:
            tpm.sock.close()
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as stop:
                stop.settimeout(5)
                stop.connect(str(control))
                stop.sendall(struct.pack(">I", 3))  # CMD_SHUTDOWN stores the NV state
                stop.recv(4)
    finally:
        try:
            status = process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
            raise RuntimeError("Seed swtpm did not stop")
        if status != 0:
            raise RuntimeError(f"Seed swtpm exited with {status}")


def seed(binary, env, directory, name):
    """Apply one seed and return the state it produced, read back on the same server."""
    with server(binary, env, directory) as tpm:
        SEEDS[name](tpm)
        return observe(tpm)


def snapshot(binary, env, directory):
    with server(binary, env, directory) as tpm:
        return observe(tpm)
