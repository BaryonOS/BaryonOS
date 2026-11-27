#!/usr/bin/env bash
# Window-session A/B/C (local KVM, not hosted): A = main af7415df, B = C3 6a74ad67,
# C = C3 + "again does not continue" (k-sched worktree). Five --clipboard sessions per side,
# interleaved; derived from docs/verification/k-sched-continue-20261127/ab-k.sh.
set -uo pipefail
D=~/projects/BaryonOS-scratch/k/net-ab; O=$D/win; mkdir -p $O; : > $O/meta.txt
declare -A TREE=([A]=$D/a [B]=$D/b [C]=$HOME/projects/BaryonOS-worktrees/k-sched)
export NOS_QEMU_ACCEL=kvm CARGO_NET_OFFLINE=true OS_OVMF_CODE=/usr/share/OVMF/OVMF_CODE_4M.fd OS_OVMF_VARS=/usr/share/OVMF/OVMF_VARS_4M.fd
VGA=/usr/share/seabios/vgabios-stdvga.bin BIOS=/usr/share/seabios/bios-256k.bin
log(){ echo "$(date -u +%H:%M:%SZ) $1" >> $O/meta.txt; }
env_for(){ export NOS_BUILD_DIR=$1/build/k-win NOS_TARGET_DIR=$1/build/k-win-target CARGO_TARGET_DIR=$1/build/k-win-target; }
for s in A B C; do log "$s $(git -C ${TREE[$s]} rev-parse HEAD) dirty [$(git -C ${TREE[$s]} status --short -- kernel | tr '\n' ' ')]"; done
for s in A B C; do d=${TREE[$s]}; cd $d; env_for $d
  python3 -B -u tools/build.py --kernel-feature native-window-session-probe > $O/build-$s.log 2>&1; rc=$?; log "build-$s exit $rc"; [ $rc = 0 ] || exit 1; done
i=0; for order in "A B C" "C B A" "B C A" "A C B" "C A B"; do i=$((i+1)); for s in $order; do d=${TREE[$s]}; cd $d; env_for $d
  rm -f $NOS_BUILD_DIR/window-session-report.json; log "$s$i load_before $(cat /proc/loadavg)"
  python3 -B -u tools/run_window_session.py --clipboard --timeout 1800 --bios-rom $BIOS --vga-rom $VGA > $O/run-$s$i.log 2>&1
  log "$s$i exit $? load_after $(cat /proc/loadavg)"; cp $NOS_BUILD_DIR/window-session-report.json $O/report-$s$i.json 2>/dev/null; done; done
python3 -I - $O > $O/summary.txt <<'PY'
import json,statistics,sys,os
d=sys.argv[1]
for s in "ABC":
    rs=[json.load(open(os.path.join(d,f"report-{s}{i}.json"))) for i in range(1,6)]
    du=[round(r["duration_s"],2) for r in rs]; cpu=[round(r["qemu_host_cpu"]["user_seconds"]+r["qemu_host_cpu"]["system_seconds"],2) for r in rs]
    print(s, rs[0]["source_commit"][:8], "raw", [r["raw_qemu_exit"] for r in rs], "passed", [r["passed"] for r in rs])
    print(s, "duration_s", du, "median", statistics.median(du), "| qemu_cpu_s", cpu, "median", statistics.median(cpu))
    frames={p:{r["phases"][p].get("frame_sha256") for r in rs} for p in rs[0]["phases"]}
    print(s, "phases", len(frames), "phases with >1 frame hash", [p for p,v in frames.items() if len(v)>1] or "none")
PY
log done
