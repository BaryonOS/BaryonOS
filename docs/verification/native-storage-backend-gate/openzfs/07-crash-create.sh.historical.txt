set -u
out=/tmp/nieuw-os-storage-zfs-20261102
export LD_LIBRARY_PATH="$out/root/usr/lib/x86_64-linux-gnu"
export PATH="$out/root/usr/sbin:$PATH"
export ZDB="$out/root/usr/sbin/zdb"
export MALLOC_ARENA_MAX=2
export ZFS_STACK_SIZE=131072
mkdir -p "$out/images/crash"
cd "$out/images/crash"
ulimit -c 0
ulimit -v 8388608
ulimit -f 4194304
/usr/bin/time -v -o "$out/logs/crash-create-time.txt" timeout --kill-after=5s 120s "$out/root/usr/sbin/ztest" -f "$out/images/crash" -p nieuwos_crash -v 1 -s 256M -m 2 -r 1 -R 0 -K raidz -C special=off -d 2 -t 2 -T 15 -P 5 -k 100 -VVV > "$out/logs/crash-create.txt" 2>&1
rc=$?
printf '%s\n' "$rc" > "$out/logs/crash-create-exit.txt"
tail -40 "$out/logs/crash-create.txt"
exit "$rc"