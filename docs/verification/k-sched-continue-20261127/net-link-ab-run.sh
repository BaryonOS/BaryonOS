#!/usr/bin/env bash
# Local TCG A/B/C of the H04-T3 net-link gate (normal, nostatus, storm): A = main af7415df,
# B = C3 6a74ad67, C = C3 + "again does not continue" (k-sched worktree). Not hosted evidence.
set -u
D=~/projects/BaryonOS-scratch/k/net-ab
declare -A TREE=([a]=$D/a [b]=$D/b [c]=$HOME/projects/BaryonOS-worktrees/k-sched)
export NOS_QEMU_MACHINE=pc-q35-8.2
run() { local s=$1 n=$2 tag=$1$2; local out=$D/link-logs/$tag; mkdir -p $out; local T=${TREE[$s]}
  ( cd $T && export NOS_TARGET_DIR=$T/target/net-ab CARGO_TARGET_DIR=$T/target/net-ab
    rc=0
    export NOS_BUILD_DIR=$T/build/link-ab-$tag
    timeout 900 python3 -B tools/build.py --kernel-feature test-system-net-link >> $out/stdout.txt 2>&1 || rc=$?
    for c in normal nostatus; do timeout 600 python3 -B -u tools/run_net_link.py --case $c >> $out/stdout.txt 2>&1 || rc=$?; done
    export NOS_BUILD_DIR=$T/build/link-storm-ab-$tag
    timeout 900 python3 -B tools/build.py --kernel-feature test-system-net-link-storm >> $out/stdout.txt 2>&1 || rc=$?
    timeout 600 python3 -B -u tools/run_net_link.py --case storm >> $out/stdout.txt 2>&1 || rc=$?
    echo "exit=$rc" > $out/exit.txt )
  echo "$tag $(cat $out/exit.txt) $(grep -oE 'SYSTEM [a-z-]+: passed=\w+ raw=[0-9]+ duration=[0-9.]+s|passed=\w+|"passed": \w+' $out/stdout.txt | tr '\n' ' ')" >> $D/link-summary.txt; }
mkdir -p $D/link-logs; : > $D/link-summary.txt; date -u +"start %H:%M:%SZ" >> $D/link-summary.txt
for s in a b c; do run $s 0; done
i=0; for o in "a b c" "c b a"; do i=$((i+1)); for s in $o; do run $s $i; done; done
date -u +"end %H:%M:%SZ" >> $D/link-summary.txt
