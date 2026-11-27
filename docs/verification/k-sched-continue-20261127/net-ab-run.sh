#!/usr/bin/env bash
# Local TCG A/B/C of tools/test_net_system.py: A = main af7415df, B = C3 6a74ad67,
# C = C3 + "again does not continue" (k-sched worktree, uncommitted). Not hosted evidence.
set -u
D=~/projects/BaryonOS-scratch/k/net-ab
declare -A TREE=([a]=$D/a [b]=$D/b [c]=$HOME/projects/BaryonOS-worktrees/k-sched)
export NOS_QEMU_MACHINE=pc-q35-8.2
run() { local s=$1 n=$2; local tag=$s$n; local out=$D/logs/$tag; mkdir -p $out
  ( cd ${TREE[$s]} && NOS_BUILD_DIR=${TREE[$s]}/build/net-ab-$tag NOS_TARGET_DIR=${TREE[$s]}/target/net-ab CARGO_TARGET_DIR=${TREE[$s]}/target/net-ab \
    timeout 1800 python3 -B -u tools/test_net_system.py > $out/stdout.txt 2>&1; echo "exit=$?" > $out/exit.txt )
  echo "$tag $(cat $out/exit.txt) $(grep -oE 'SYSTEM [a-z-]+: passed=\w+ raw=[0-9]+ duration=[0-9.]+s' $out/stdout.txt | tr '\n' ' ')" >> $D/summary.txt; }
mkdir -p $D/logs; : > $D/summary.txt; date -u +"start %H:%M:%SZ" >> $D/summary.txt
for s in a b c; do run $s 0; done   # warm-up: cold builds, timings discarded
i=0; for o in "a b c" "c b a" "b c a"; do i=$((i+1)); for s in $o; do run $s $i; done; done
date -u +"end %H:%M:%SZ" >> $D/summary.txt
