#!/usr/bin/env bash
# R02 native: Materiaal recover in the SAME build dir as its validated normal parent (contract NATIVE-MATERIAAL-PROBE.md, ab5.sh).
set -uo pipefail
A=$HOME/projects/BaryonOS-worktrees/r-turn-a; O=$HOME/projects/BaryonOS-scratch/r/native-20261127
export CARGO_NET_OFFLINE=true OS_OVMF_CODE=/usr/share/OVMF/OVMF_CODE_4M.fd OS_OVMF_VARS=/usr/share/OVMF/OVMF_VARS_4M.fd
unset NOS_QEMU_ACCEL; VGA=/usr/share/seabios/vgabios-stdvga.bin
log(){ echo "$(date -u +%H:%M:%SZ) $*" >> $O/meta.txt; }
cd $A || exit 2
export NOS_BUILD_DIR=$A/build/r-mat NOS_TARGET_DIR=$A/build/r-target
parent=$A/build/r-mat/run/materiaal-normal-89c422c1f4
log "recover-rerun HEAD $(git rev-parse HEAD) dirty=[$(git status --short | tr '\n' ' ')] qemu_before=[$(ps -eo args | grep -c '[q]emu-system')] parent=$(basename $parent)"
python3 -B -u tools/build.py --kernel-feature test-system-materiaal-recover > $O/build-r-mat-recover.log 2>&1; rc=$?; log "build r-mat (test-system-materiaal-recover) exit $rc"; [ $rc = 0 ] || exit $rc
log "mat-recover start load $(cut -d' ' -f1-3 /proc/loadavg)"
python3 -B -u tools/run_materiaal_probe.py --case recover --build-dir $NOS_BUILD_DIR --parent $parent --qmp-transport pipe --vga-rom $VGA --timeout 240 > $O/mat-recover.log 2>&1
log "mat-recover exit $? load $(cut -d' ' -f1-3 /proc/loadavg)"; log done-recover
