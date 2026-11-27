#!/bin/bash
# R02 tick-split diagnostic: D = main bbdbea88 + local diag patch (never committed), M = clean main control. KVM, scale 1, CI flags.
set -uo pipefail
O=${R02_OUTPUT_DIR:?Set R02_OUTPUT_DIR to a new output directory}
if [ -e "$O" ]; then printf "Output path already exists: %s\n" "$O" >&2; exit 1; fi
mkdir -p "$O" || exit 1
O=$(cd "$O" && pwd -P) || exit 1
D=$HOME/projects/BaryonOS-scratch/r/diag-tick/src; M=$HOME/projects/BaryonOS-worktrees/r-tick
export NOS_QEMU_ACCEL=kvm CARGO_NET_OFFLINE=true OS_OVMF_CODE=/usr/share/OVMF/OVMF_CODE_4M.fd OS_OVMF_VARS=/usr/share/OVMF/OVMF_VARS_4M.fd
VGA=/usr/share/seabios/vgabios-stdvga.bin BIOS=/usr/share/seabios/bios-256k.bin
log(){ echo "$(date -u +%H:%M:%SZ) $*" >> "$O/meta.txt"; }
env_for(){ export NOS_BUILD_DIR="$1/build/tick" NOS_TARGET_DIR="$1/build/tick-target"; }
: > "$O/meta.txt"
log "D $(git -C "$D" rev-parse HEAD) dirty=[$(git -C "$D" status --short | tr '\n' ' ')] M $(git -C "$M" rev-parse HEAD) dirty=[$(git -C "$M" status --short | tr '\n' ' ')]"
for side in D M; do d=${!side}; cd "$d" || exit 1; env_for "$d"
  python3 -B -u tools/build.py --kernel-feature native-window-session-probe > "$O/build-$side.log" 2>&1; rc=$?; log "build-$side exit $rc"; [ $rc = 0 ] || exit $rc
done
[ "${BUILD_ONLY:-0}" = 1 ] && { log build-only; exit 0; }
failed=0
for run in D1 M1 M2 D2 D3 M3; do side=${run:0:1}; d=${!side}; cd "$d" || exit 1; env_for "$d"
  rm -f "$NOS_BUILD_DIR/window-session-report.json" "$O/report-$run.json"
  qemu_count=$(ps -eo args | grep -c '[q]emu-system')
  log "$run start load $(cut -d' ' -f1-3 /proc/loadavg) qemu=$qemu_count"
  if [ "$qemu_count" -ne 0 ]; then log "$run blocked: QEMU already active"; exit 1; fi
  python3 -B -u tools/run_window_session.py --clipboard --workspace --timeout 1800 --bios-rom "$BIOS" --vga-rom "$VGA" > "$O/run-$run.log" 2>&1
  rc=$?
  log "$run exit $rc load $(cut -d' ' -f1-3 /proc/loadavg)"
  if [ -f "$NOS_BUILD_DIR/window-session-report.json" ]; then
    cp "$NOS_BUILD_DIR/window-session-report.json" "$O/report-$run.json" || exit 1
  else
    log "$run MISSING report"
    failed=1
  fi
  [ "$rc" -eq 0 ] || failed=1
  ls -t "$NOS_BUILD_DIR/run" 2>/dev/null | head -1 | sed "s|^|$run rundir $NOS_BUILD_DIR/run/|" >> "$O/meta.txt"
done
log done
exit "$failed"
