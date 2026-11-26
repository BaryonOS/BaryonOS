#!/bin/bash
# Local KVM A/B of the native window session: A = base worktree, B = fix
# worktree. Builds both images, runs the Materiaal normal probe on B, then ten
# interleaved window sessions per side. Exits non-zero if any step failed.
# The recorded run built A in the earlier batch (same 3b9a94c5 image, see README).
set -uo pipefail
O=$HOME/projects/BaryonOS-scratch/r/ab4
A=$HOME/projects/BaryonOS-worktrees/r-base; B=$HOME/projects/BaryonOS-worktrees/r-dirty
mkdir -p "$O" || exit 1
export NOS_QEMU_ACCEL=kvm CARGO_NET_OFFLINE=true OS_OVMF_CODE=/usr/share/OVMF/OVMF_CODE_4M.fd OS_OVMF_VARS=/usr/share/OVMF/OVMF_VARS_4M.fd
VGA=/usr/share/seabios/vgabios-stdvga.bin BIOS=/usr/share/seabios/bios-256k.bin
failed=0
log(){ echo "$*" >> "$O/meta.txt"; }
env_for(){ export NOS_BUILD_DIR=$1/build/$2 NOS_TARGET_DIR=$1/build/$2-target CARGO_TARGET_DIR=$1/build/$2-target; }
log "A $(git -C $A rev-parse HEAD) B $(git -C $B rev-parse HEAD) B_dirty $(git -C $B status --short | tr '\n' ' ')"
cd "$B"; env_for "$B" probe-normal
python3 -B -u tools/build.py --kernel-feature native-system-materiaal-probe > "$O/probe-normal-build.log" 2>&1; rc=$?; log "probe-normal build_exit $rc"; [ $rc = 0 ] || exit 1
python3 -B -u tools/run_materiaal_probe.py --case normal --build-dir "$NOS_BUILD_DIR" --vga-rom $VGA > "$O/probe-normal.log" 2>&1; rc=$?; log "probe-normal exit $rc"; [ $rc = 0 ] || failed=1
for side in A B; do d=${!side}; cd "$d"; env_for "$d" r-win
  python3 -B -u tools/build.py --kernel-feature native-window-session-probe > "$O/build-$side.log" 2>&1; rc=$?; log "build-$side exit $rc"; [ $rc = 0 ] || exit 1
done
for i in 1 2 3 4 5 6 7 8 9 10; do for side in A B; do
  [ $((i % 2)) = 0 ] && side=$([ $side = A ] && echo B || echo A)
  d=${!side}; cd "$d"; env_for "$d" r-win
  log "$side$i load_before $(cat /proc/loadavg)"
  rm -f "$NOS_BUILD_DIR/window-session-report.json"
  python3 -B -u tools/run_window_session.py --clipboard --timeout 240 --bios-rom $BIOS --vga-rom $VGA > "$O/run-$side$i.log" 2>&1
  rc=$?; log "$side$i exit $rc load_after $(cat /proc/loadavg)"; [ $rc = 0 ] || failed=1
  cp "$NOS_BUILD_DIR/window-session-report.json" "$O/report-$side$i.json" || failed=1
done; done
exit $failed
