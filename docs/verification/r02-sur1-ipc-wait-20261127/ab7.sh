#!/bin/bash
# R02 (1) A/B 5+5, KVM, CI flags --clipboard --workspace, scale 1. A = candidate (ipc_wait, 150 ticks), B = main post-#586.
set -uo pipefail
O=$HOME/projects/BaryonOS-scratch/r/ab7
A=$HOME/projects/BaryonOS-worktrees/r-turn-a; B=$HOME/projects/BaryonOS-worktrees/r-base
export NOS_QEMU_ACCEL=kvm CARGO_NET_OFFLINE=true OS_OVMF_CODE=/usr/share/OVMF/OVMF_CODE_4M.fd OS_OVMF_VARS=/usr/share/OVMF/OVMF_VARS_4M.fd
VGA=/usr/share/seabios/vgabios-stdvga.bin BIOS=/usr/share/seabios/bios-256k.bin
log(){ echo "$(date -u +%H:%M:%SZ) $*" >> $O/meta.txt; }
env_for(){ export NOS_BUILD_DIR=$1/build/ab NOS_TARGET_DIR=$1/build/ab-target; }
: > $O/meta.txt
log "A $(git -C $A rev-parse HEAD) dirty=[$(git -C $A status --short | tr '\n' ' ')] B $(git -C $B rev-parse HEAD) dirty=[$(git -C $B status --short | tr '\n' ' ')] qemu=[$(ps -eo args | grep -c '[q]emu-system')]"
for side in A B; do d=${!side}; cd $d; env_for $d
  python3 -B -u tools/build.py --kernel-feature native-window-session-probe > $O/build-$side.log 2>&1; rc=$?; log "build-$side exit $rc"; [ $rc = 0 ] || exit $rc
done
for run in A1 B1 B2 A2 A3 B3 B4 A4 A5 B5; do side=${run:0:1}; d=${!side}; cd $d; env_for $d
  rm -f $NOS_BUILD_DIR/window-session-report.json
  log "$run start load $(cut -d' ' -f1-3 /proc/loadavg)"
  python3 -B -u tools/run_window_session.py --clipboard --workspace --timeout 1800 --bios-rom $BIOS --vga-rom $VGA > $O/run-$run.log 2>&1
  log "$run exit $? load $(cut -d' ' -f1-3 /proc/loadavg)"
  cp $NOS_BUILD_DIR/window-session-report.json $O/report-$run.json 2>/dev/null
done
log done
