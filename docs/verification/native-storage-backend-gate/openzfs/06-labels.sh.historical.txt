set -eu
out=/tmp/nieuw-os-storage-zfs-20261102
export LD_LIBRARY_PATH="$out/root/usr/lib/x86_64-linux-gnu"
export MALLOC_ARENA_MAX=2 ZFS_STACK_SIZE=131072
ulimit -c 0
ulimit -v 8388608
"$out/root/usr/sbin/zdb" -l "$out/images/mirror/nieuwos_n3.0a" > "$out/logs/baseline-labels.txt"
head -110 "$out/logs/baseline-labels.txt"
grep -n 'spa_destroy\|spa_export\|ztest_destroy\|ztest_export' "$out/ztest-2.4.1.c" | tail -28
grep -n 'dump_copy\|copy_object\|copy_file\|case .r.\|dump_opt.*r.*=' "$out/zdb-2.4.1.c" | tail -18