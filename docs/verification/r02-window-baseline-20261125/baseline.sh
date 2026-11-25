#!/bin/bash
set -uo pipefail
W=$HOME/projects/BaryonOS-worktrees/r-track
S=$HOME/projects/BaryonOS-scratch/r
export NOS_BUILD_DIR=$W/build/r NOS_TARGET_DIR=$W/build/r-target CARGO_TARGET_DIR=$W/build/r-target NOS_QEMU_ACCEL=kvm CARGO_NET_OFFLINE=true
cd "$W"
echo "head $(git rev-parse HEAD)" > $S/baseline2/meta.txt
python3 -B -u tools/build.py --kernel-feature native-window-session-probe > $S/baseline2/build.log 2>&1; rc=$?; echo "build_exit $rc" >> $S/baseline2/meta.txt; [ $rc = 0 ] || exit 1
for i in 1 2 3 4 5; do
  echo "run$i load_before $(cat /proc/loadavg)" >> $S/baseline2/meta.txt
  python3 -B -u tools/run_window_session.py --clipboard --timeout 1800 --bios-rom /usr/share/seabios/bios-256k.bin --vga-rom /usr/share/seabios/vgabios-stdvga.bin > $S/baseline2/run$i.log 2>&1
  echo "run$i exit $? load_after $(cat /proc/loadavg)" >> $S/baseline2/meta.txt
  cp $NOS_BUILD_DIR/window-session-report.json $S/baseline2/report$i.json 2>/dev/null
done
