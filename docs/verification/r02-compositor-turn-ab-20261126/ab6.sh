#!/bin/bash
# R02 (2) high-contrast A/B (CI flags --clipboard --workspace, scale 1), then the comment-nit rebuild hash on B.
set -uo pipefail
O=$HOME/projects/BaryonOS-scratch/r/ab5; S=$HOME/projects/BaryonOS-scratch/r
A=$HOME/projects/BaryonOS-worktrees/r-turn-a; B=$HOME/projects/BaryonOS-worktrees/r-turn
export NOS_QEMU_ACCEL=kvm CARGO_NET_OFFLINE=true OS_OVMF_CODE=/usr/share/OVMF/OVMF_CODE_4M.fd OS_OVMF_VARS=/usr/share/OVMF/OVMF_VARS_4M.fd
VGA=/usr/share/seabios/vgabios-stdvga.bin BIOS=/usr/share/seabios/bios-256k.bin
log(){ echo "$(date -u +%H:%M:%SZ) $*" >> $O/meta-ws.txt; }
env_for(){ export NOS_BUILD_DIR=$1/build/$2 NOS_TARGET_DIR=$1/build/$2-target CARGO_TARGET_DIR=$1/build/$2-target; }
: > $O/meta-ws.txt
log "A $(git -C $A rev-parse HEAD) B $(git -C $B rev-parse HEAD) B_dirty $(git -C $B status --short | tr '\n' ' ')"
for run in A1 B1 B2 A2; do side=${run:0:1}; d=${!side}; cd $d; env_for $d r-win
  rm -f $NOS_BUILD_DIR/window-session-report.json
  log "ws-$run load_before $(cat /proc/loadavg)"
  python3 -B -u tools/run_window_session.py --clipboard --workspace --timeout 1800 --bios-rom $BIOS --vga-rom $VGA > $O/ws-run-$run.log 2>&1
  log "ws-$run exit $? load_after $(cat /proc/loadavg)"
  cp $NOS_BUILD_DIR/window-session-report.json $O/ws-report-$run.json 2>/dev/null
done
# Comment-only nit: rebuild B and compare the kernel ELF (which embeds window-compositor).
cd $B; env_for $B r-win
before=$(sha256sum $NOS_BUILD_DIR/esp/kernel.elf 2>/dev/null || find $NOS_BUILD_DIR -name kernel.elf -path '*esp*' -exec sha256sum {} \; | head -1)
log "B kernel.elf before nit: $before"
sed -i 's|// monotonic_ticks call is a scheduler switch point on one CPU\.|// monotonic_ticks call is a scheduler switch point on the single-CPU path.|' sdk/examples/window-session/src/main.rs
log "nit diff: $(git diff --stat | tail -1)"
python3 -B -u tools/build.py --kernel-feature native-window-session-probe > $O/build-B-nit.log 2>&1; log "build-B-nit exit $?"
after=$(sha256sum $NOS_BUILD_DIR/esp/kernel.elf 2>/dev/null || find $NOS_BUILD_DIR -name kernel.elf -path '*esp*' -exec sha256sum {} \; | head -1)
log "B kernel.elf after nit: $after"
log done
