set -u
out=/tmp/nieuw-os-storage-zfs-20261102
export LD_LIBRARY_PATH="$out/root/usr/lib/x86_64-linux-gnu"
export PATH="$out/root/usr/sbin:$PATH"
export ZDB="$out/root/usr/sbin/zdb"
mkdir -p "$out/images/mirror"
cd "$out/images/mirror"
ulimit -c 0
export MALLOC_ARENA_MAX=2
export ZFS_STACK_SIZE=131072
ulimit -v 8388608
ulimit -f 4194304
/usr/bin/time -v -o "$out/logs/create-bounded-time.txt" timeout --kill-after=5s 90s "$out/root/usr/sbin/ztest" -f "$out/images/mirror" -p nieuwos_n3 -v 1 -s 256M -m 2 -r 1 -R 0 -K raidz -C special=off -d 2 -t 2 -T 15 -P 5 -k 0 -VVV > "$out/logs/create-bounded.txt" 2>&1
rc=$?
printf '%s\n' "$rc" > "$out/logs/create-bounded-exit.txt"
tail -70 "$out/logs/create-bounded.txt"
printf 'EXIT=%s\n' "$rc"
exit "$rc"