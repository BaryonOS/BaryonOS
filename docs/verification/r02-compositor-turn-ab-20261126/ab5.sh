#!/bin/bash
# R02 (2) A/B: A = main ffea3883 (r-turn-a), B = once-per-turn clock (r-turn).
# Five interleaved window sessions per side plus Materiaal normal/recover/keyboard on B.
set -uo pipefail
O=$HOME/projects/BaryonOS-scratch/r/ab5; mkdir -p $O; : > $O/meta.txt
A=$HOME/projects/BaryonOS-worktrees/r-turn-a; B=$HOME/projects/BaryonOS-worktrees/r-turn
export NOS_QEMU_ACCEL=kvm CARGO_NET_OFFLINE=true OS_OVMF_CODE=/usr/share/OVMF/OVMF_CODE_4M.fd OS_OVMF_VARS=/usr/share/OVMF/OVMF_VARS_4M.fd
VGA=/usr/share/seabios/vgabios-stdvga.bin BIOS=/usr/share/seabios/bios-256k.bin
log(){ echo "$(date -u +%H:%M:%SZ) $*" >> $O/meta.txt; }
env_for(){ export NOS_BUILD_DIR=$1/build/$2 NOS_TARGET_DIR=$1/build/$2-target CARGO_TARGET_DIR=$1/build/$2-target; }
log "A $(git -C $A rev-parse HEAD) B $(git -C $B rev-parse HEAD) B_dirty $(git -C $B status --short | tr '\n' ' ')"
for side in A B; do d=${!side}; cd $d; env_for $d r-win
  python3 -B -u tools/build.py --kernel-feature native-window-session-probe > $O/build-$side.log 2>&1; rc=$?; log "build-$side exit $rc"; [ $rc = 0 ] || exit 1
done
for i in 1 2 3 4 5; do for side in A B; do
  [ $((i % 2)) = 0 ] && side=$([ $side = A ] && echo B || echo A)
  d=${!side}; cd $d; env_for $d r-win
  rm -f $NOS_BUILD_DIR/window-session-report.json
  log "$side$i load_before $(cat /proc/loadavg)"
  python3 -B -u tools/run_window_session.py --clipboard --timeout 1800 --bios-rom $BIOS --vga-rom $VGA > $O/run-$side$i.log 2>&1
  log "$side$i exit $? load_after $(cat /proc/loadavg)"
  cp $NOS_BUILD_DIR/window-session-report.json $O/report-$side$i.json 2>/dev/null
done; done
cd $B; env_for $B m
python3 -B -u tools/build.py --kernel-feature native-system-materiaal-probe > $O/probe-build-normal.log 2>&1; log "probe build normal exit $?"
for c in normal keyboard; do
  python3 -B -u tools/run_materiaal_probe.py --case $c --build-dir $NOS_BUILD_DIR --vga-rom $VGA --qmp-transport pipe --timeout 240 > $O/probe-$c.log 2>&1; log "probe-$c exit $?"
  [ $c = normal ] && parent=$(ls -dt $NOS_BUILD_DIR/run/materiaal-normal-*/ | head -1)
done
python3 -B -u tools/build.py --kernel-feature test-system-materiaal-recover > $O/probe-build-recover.log 2>&1; log "probe build recover exit $?"
python3 -B -u tools/run_materiaal_probe.py --case recover --build-dir $NOS_BUILD_DIR --vga-rom $VGA --parent "${parent%/}" > $O/probe-recover.log 2>&1; log "probe-recover exit $?"
log done
