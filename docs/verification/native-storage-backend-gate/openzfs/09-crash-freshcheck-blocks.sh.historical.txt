set -u
out=/tmp/nieuw-os-storage-zfs-20261102
export LD_LIBRARY_PATH="$out/root/usr/lib/x86_64-linux-gnu"
export MALLOC_ARENA_MAX=2 ZFS_STACK_SIZE=131072
ulimit -c 0
ulimit -v 8388608
/usr/bin/time -v -o "$out/logs/crash-fresh-zdb-time.txt" timeout --kill-after=5s 60s "$out/root/usr/sbin/zdb" -bccs -G -d -e -y -p "$out/images/crash" nieuwos_crash > "$out/logs/crash-fresh-zdb.txt" 2>&1
printf '%s\n' "$?" > "$out/logs/crash-fresh-zdb-exit.txt"
tail -32 "$out/logs/crash-fresh-zdb.txt"
timeout --kill-after=5s 60s "$out/root/usr/sbin/zdb" -bbbbccs -G -e -p "$out/images/mirror" nieuwos_n3 > "$out/logs/baseline-allblocks.txt" 2>&1
printf '%s\n' "$?" > "$out/logs/baseline-allblocks-exit.txt"
head -16 "$out/logs/baseline-allblocks.txt"