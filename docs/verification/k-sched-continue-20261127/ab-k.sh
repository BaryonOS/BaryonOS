#!/bin/bash
# R02-C3 (K) A/B: A = main at the frozen A SHA (worktree k-ab-a), B = #602 (main + C3, worktree k-sched).
# Five interleaved window sessions per side (CI flags --clipboard), then the high-contrast
# A1 B1 B2 A2 loop with --clipboard --workspace. Local KVM, not hosted. Derived from
# docs/verification/r02-compositor-turn-ab-20261126/ab5.sh and ab6.sh (Materiaal and nit parts dropped).
# Summary: python3 docs/verification/r02-compositor-turn-ab-20261126/analyse.py "$O"
set -uo pipefail
O=${O:-$HOME/projects/BaryonOS-scratch/k/ab}; mkdir -p "$O"; : > "$O/meta.txt"; : > "$O/meta-ws.txt"
A=$HOME/projects/BaryonOS-worktrees/k-ab-a; B=$HOME/projects/BaryonOS-worktrees/k-sched
export NOS_QEMU_ACCEL=kvm CARGO_NET_OFFLINE=true OS_OVMF_CODE=/usr/share/OVMF/OVMF_CODE_4M.fd OS_OVMF_VARS=/usr/share/OVMF/OVMF_VARS_4M.fd
VGA=/usr/share/seabios/vgabios-stdvga.bin BIOS=/usr/share/seabios/bios-256k.bin
log(){ echo "$(date -u +%H:%M:%SZ) $2" >> "$O/$1"; }
env_for(){ export NOS_BUILD_DIR=$1/build/$2 NOS_TARGET_DIR=$1/build/$2-target CARGO_TARGET_DIR=$1/build/$2-target; }
for f in meta.txt meta-ws.txt; do
  log $f "A $(git -C $A rev-parse HEAD) A_dirty [$(git -C $A status --short | tr '\n' ' ')] B $(git -C $B rev-parse HEAD) B_dirty [$(git -C $B status --short | tr '\n' ' ')]"
done
for side in A B; do d=${!side}; cd "$d"; env_for "$d" k-win
  python3 -B -u tools/build.py --kernel-feature native-window-session-probe > "$O/build-$side.log" 2>&1; rc=$?
  log meta.txt "build-$side exit $rc"; [ $rc = 0 ] || exit 1
done
for i in 1 2 3 4 5; do for side in A B; do
  [ $((i % 2)) = 0 ] && side=$([ $side = A ] && echo B || echo A)
  d=${!side}; cd "$d"; env_for "$d" k-win
  rm -f "$NOS_BUILD_DIR/window-session-report.json"
  log meta.txt "$side$i load_before $(cat /proc/loadavg)"
  python3 -B -u tools/run_window_session.py --clipboard --timeout 1800 --bios-rom $BIOS --vga-rom $VGA > "$O/run-$side$i.log" 2>&1
  log meta.txt "$side$i exit $? load_after $(cat /proc/loadavg)"
  cp "$NOS_BUILD_DIR/window-session-report.json" "$O/report-$side$i.json" 2>/dev/null
done; done
for run in A1 B1 B2 A2; do side=${run:0:1}; d=${!side}; cd "$d"; env_for "$d" k-win
  rm -f "$NOS_BUILD_DIR/window-session-report.json"
  log meta-ws.txt "ws-$run load_before $(cat /proc/loadavg)"
  python3 -B -u tools/run_window_session.py --clipboard --workspace --timeout 1800 --bios-rom $BIOS --vga-rom $VGA > "$O/ws-run-$run.log" 2>&1
  log meta-ws.txt "ws-$run exit $? load_after $(cat /proc/loadavg)"
  cp "$NOS_BUILD_DIR/window-session-report.json" "$O/ws-report-$run.json" 2>/dev/null
done
log meta.txt done
