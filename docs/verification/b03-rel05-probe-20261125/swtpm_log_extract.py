#!/usr/bin/env python3
"""Extract TPM command/response pairs from a swtpm --log level=20 file.

Prints one line per TPM command: command code, and for PCR_Extend/PCR_Event
the PCR index and digest banks, plus the response code. Feasibility probe aid.
Usage: python3 -B swtpm_log_extract.py swtpm-u2.log
"""
import re
import struct
import sys

NAMES = {0x144: "Startup", 0x17A: "GetCapability", 0x182: "PCR_Extend", 0x13C: "PCR_Event",
         0x17E: "PCR_Read", 0x143: "SelfTest", 0x145: "Shutdown", 0x181: "HierarchyChangeAuth?",
         0x129: "HierarchyChangeAuth", 0x121: "HierarchyControl", 0x17B: "GetRandom",
         0x12B: "PCR_Allocate", 0x13D: "PCR_Reset", 0x14E: "NV_Read", 0x169: "NV_ReadPublic",
         0x173: "ReadPublic", 0x131: "CreatePrimary", 0x165: "FlushContext", 0x11F: "NV_UndefineSpace?",
         0x12A: "NV_DefineSpace", 0x137: "NV_Write", 0x186: "HashSequenceStart",
         0x15C: "SequenceUpdate", 0x13E: "SequenceComplete", 0x185: "EventSequenceComplete"}
ALGS = {0x0004: "sha1", 0x000B: "sha256", 0x000C: "sha384", 0x000D: "sha512", 0x0012: "sm3_256"}
DLEN = {0x0004: 20, 0x000B: 32, 0x000C: 48, 0x000D: 64, 0x0012: 32}


def frames(text):
    cur = None
    for line in text.splitlines():
        m = re.match(r"\s*SWTPM_IO_(Read|Write): length (\d+)", line)
        if m:
            if cur:
                yield cur
            cur = [m.group(1), b""]
            continue
        if cur is not None and re.fullmatch(r"\s*([0-9A-F]{2} )+\s*", line):
            cur[1] += bytes.fromhex(line)
        elif cur is not None:
            yield cur
            cur = None
    if cur:
        yield cur


def main(path):
    pending = None
    extended = {}
    for kind, data in frames(open(path, errors="replace").read()):
        if len(data) < 10:
            continue
        if kind == "Read":
            pending = data
            continue
        cmd, rc = pending, struct.unpack(">I", data[6:10])[0]
        cc = struct.unpack(">I", cmd[6:10])[0]
        extra = ""
        if cc in (0x182, 0x13C):
            pcr = struct.unpack(">I", cmd[10:14])[0]
            auth = struct.unpack(">I", cmd[14:18])[0]
            off = 18 + auth
            if cc == 0x182:
                count = struct.unpack(">I", cmd[off:off + 4])[0]
                off += 4
                banks = []
                for _ in range(count):
                    alg = struct.unpack(">H", cmd[off:off + 2])[0]
                    banks.append(ALGS.get(alg, hex(alg)))
                    off += 2 + DLEN.get(alg, 0)
                extra = f" pcr={pcr} banks={','.join(banks)}"
            else:
                extra = f" pcr={pcr} (event)"
            extended[pcr] = extended.get(pcr, 0) + 1
        elif cc == 0x17A:
            cap, prop = struct.unpack(">II", cmd[10:18])
            extra = f" cap=0x{cap:X} prop=0x{prop:X}"
            if cap == 5:  # TPM_CAP_PCRS: list allocated banks
                n = struct.unpack(">I", data[15:19])[0]
                off, banks = 19, []
                for _ in range(n):
                    alg, sz = struct.unpack(">HB", data[off:off + 3])
                    sel = data[off + 3:off + 3 + sz]
                    banks.append(f"{ALGS.get(alg, hex(alg))}:{'on' if any(sel) else 'off'}")
                    off += 3 + sz
                extra += f" -> banks {' '.join(banks)}"
        print(f"0x{cc:03X} {NAMES.get(cc, '?'):<22} rc=0x{rc:03X}{extra}")
    print("PCR extend count per index:", dict(sorted(extended.items())))
    print("PCR 13 extended by firmware:", 13 in extended)


if __name__ == "__main__":
    main(sys.argv[1])
