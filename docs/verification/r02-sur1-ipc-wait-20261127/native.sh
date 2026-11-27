#!/usr/bin/env bash
# R02 (1) native acceptance on the final head (U: s1+s2 raw exit 33 + passed=true; Materiaal normal/keyboard/recover).
# TCG like hosted CI (NOS_QEMU_ACCEL unset). Sequential; one QEMU at a time.
set -uo pipefail
A=$HOME/projects/BaryonOS-worktrees/r-turn-a; O=$HOME/projects/BaryonOS-scratch/r/native-20261127
export CARGO_NET_OFFLINE=true OS_OVMF_CODE=/usr/share/OVMF/OVMF_CODE_4M.fd OS_OVMF_VARS=/usr/share/OVMF/OVMF_VARS_4M.fd
unset NOS_QEMU_ACCEL
VGA=/usr/share/seabios/vgabios-stdvga.bin BIOS=/usr/share/seabios/bios-256k.bin
log(){ echo "$(date -u +%H:%M:%SZ) $*" >> $O/meta.txt; }
cd $A || exit 2
: > $O/meta.txt
log "HEAD $(git rev-parse HEAD) dirty=[$(git status --short | tr '\n' ' ')] qemu_before=[$(ps -eo args | grep -c '[q]emu-system')]"
ws(){ local name=$1 dir=$2 scale=$3 tmo=$4; export NOS_BUILD_DIR=$A/build/$dir NOS_TARGET_DIR=$A/build/r-target
  rm -f $NOS_BUILD_DIR/window-session-report.json; log "$name start load $(cut -d' ' -f1-3 /proc/loadavg)"
  python3 -B -u tools/run_window_session.py --clipboard --workspace --scale $scale --timeout $tmo --bios-rom $BIOS --vga-rom $VGA > $O/$name.log 2>&1
  log "$name exit $? load $(cut -d' ' -f1-3 /proc/loadavg)"; cp $NOS_BUILD_DIR/window-session-report.json $O/$name-report.json 2>/dev/null; }
mat(){ local case=$1 dir=$2 feature=$3; export NOS_BUILD_DIR=$A/build/$dir NOS_TARGET_DIR=$A/build/r-target
  if [ ! -f $O/build-$dir.done ]; then python3 -B -u tools/build.py --kernel-feature $feature > $O/build-$dir.log 2>&1; rc=$?; log "build $dir ($feature) exit $rc"; [ $rc = 0 ] || return; touch $O/build-$dir.done; fi
  log "mat-$case start load $(cut -d' ' -f1-3 /proc/loadavg)"
  python3 -B -u tools/run_materiaal_probe.py --case $case --build-dir $NOS_BUILD_DIR --qmp-transport pipe --vga-rom $VGA --timeout 240 > $O/mat-$case.log 2>&1
  log "mat-$case exit $? load $(cut -d' ' -f1-3 /proc/loadavg)"; }
ws tcg-s1 r 1 1800
ws tcg-s2 r-hidpi 2 2700
mat normal r-mat native-system-materiaal-probe
mat keyboard r-mat native-system-materiaal-probe
mat recover r-matrec test-system-materiaal-recover
log done
