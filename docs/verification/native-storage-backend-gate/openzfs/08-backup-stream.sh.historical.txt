set -u
out=/tmp/nieuw-os-storage-zfs-20261102
export LD_LIBRARY_PATH="$out/root/usr/lib/x86_64-linux-gnu"
export MALLOC_ARENA_MAX=2 ZFS_STACK_SIZE=131072
ulimit -c 0
ulimit -v 8388608
ulimit -f 4194304
mkdir -p "$out/exports"
timeout --kill-after=5s 30s "$out/root/usr/sbin/zdb" -B -e -p "$out/images/mirror" nieuwos_n3/5 w > "$out/exports/ds0-raw.zfs" 2> "$out/logs/backup-stream.txt"
printf '%s\n' "$?" > "$out/logs/backup-stream-exit.txt"
wc -c "$out/exports/ds0-raw.zfs"
cat "$out/logs/backup-stream.txt"
timeout --kill-after=5s 30s "$out/root/usr/sbin/zdb" -dddddd -e -p "$out/images/mirror" nieuwos_n3/ds_0 > "$out/logs/ds0-objects.txt" 2>&1
printf '%s\n' "$?" > "$out/logs/ds0-objects-exit.txt"
head -90 "$out/logs/ds0-objects.txt"