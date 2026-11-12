#!/usr/bin/env python3
"""Diagnostic only: boot an SMP image N times, timestamp every serial marker on
the host and sample the QEMU threads' scheduler statistics. Not acceptance."""
import json, os, re, shutil, subprocess, sys, tempfile, threading, time

esp, cpus, hostcpus, repeat, out = sys.argv[1], int(sys.argv[2]), sys.argv[3], int(sys.argv[4]), sys.argv[5]
accel = sys.argv[6] if len(sys.argv) > 6 else "multi"
CODE, VARS = "/usr/share/OVMF/OVMF_CODE_4M.fd", "/usr/share/OVMF/OVMF_VARS_4M.fd"


def steal():
    with open("/proc/stat") as f:
        v = f.readline().split()
    return int(v[8]), sum(int(x) for x in v[1:9])


def sample(pid, box, stop):
    while not stop.is_set():
        snap = {}
        try:
            for tid in os.listdir(f"/proc/{pid}/task"):
                with open(f"/proc/{pid}/task/{tid}/comm") as f:
                    comm = f.read().strip()
                with open(f"/proc/{pid}/task/{tid}/schedstat") as f:
                    run, wait, slices = map(int, f.read().split())
                invol = 0
                with open(f"/proc/{pid}/task/{tid}/status") as f:
                    for line in f:
                        if line.startswith("nonvoluntary_ctxt_switches"):
                            invol = int(line.split()[1])
                snap[tid] = (comm, run, wait, slices, invol)
        except OSError:
            pass
        if any(v[0].startswith("CPU ") for v in snap.values()):
            box["threads"] = snap
        stop.wait(0.05)


with open(out, "a") as log:
    for i in range(repeat):
        tmp = tempfile.mkdtemp(prefix="nosdiag-")
        shutil.copy(VARS, f"{tmp}/vars.fd")
        cmd = ["taskset", "-c", hostcpus, "qemu-system-x86_64", "-machine", "pc-q35-8.2",
               "-accel", f"tcg,thread={accel}", "-cpu", "qemu64", "-smp", str(cpus), "-m", "256M",
               "-name", "diag,debug-threads=on", "-nodefaults", "-display", "none", "-monitor", "none", "-serial", "stdio",
               "-nic", "none", "-no-reboot",
               "-drive", f"if=pflash,unit=0,format=raw,readonly=on,file={CODE}",
               "-drive", f"if=pflash,unit=1,format=raw,file={tmp}/vars.fd",
               "-drive", f"if=virtio,index=0,format=raw,readonly=on,file=fat:{esp}",
               "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
        s0, t0 = steal(), time.monotonic()
        p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        box, stop = {}, threading.Event()
        threading.Thread(target=sample, args=(p.pid, box, stop), daemon=True).start()
        timer = threading.Timer(120, p.kill)
        timer.start()
        marks, bsp_ser, prev = [], None, t0
        for raw in p.stdout:
            now = time.monotonic()
            line = raw.decode("latin1").strip()
            if line.startswith("SMP:SER:0:"):
                bsp_ser = now
            elif re.match(r"(SMP|M1|M2):", line) and not line.startswith("SMP:SER:"):
                marks.append((line, round(now - t0, 4), round(now - prev, 4)))
                prev = now
                if line.startswith("SMP:SERIAL:PASS") and bsp_ser:
                    marks.append(("DIAG:SYNC_WAIT_AFTER_BSP", round(now - t0, 4), round(now - bsp_ser, 4)))
        code = p.wait()
        timer.cancel()
        stop.set()
        s1 = steal()
        vcpu = [v for v in box.get("threads", {}).values() if v[0].startswith("CPU ")]
        log.write(json.dumps({
            "i": i, "cpus": cpus, "hostcpus": hostcpus, "accel": accel, "exit": code,
            "wall": round(time.monotonic() - t0, 3),
            "steal_ticks": s1[0] - s0[0], "total_ticks": s1[1] - s0[1],
            "marks": marks,
            "vcpu": sorted(vcpu),
            "other": sorted(v for v in box.get("threads", {}).values() if not v[0].startswith("CPU ")),
        }) + "\n")
        log.flush()
        shutil.rmtree(tmp, ignore_errors=True)
