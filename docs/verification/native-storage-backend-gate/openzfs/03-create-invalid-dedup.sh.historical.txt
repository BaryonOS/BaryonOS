set -u
out=/tmp/nieuw-os-storage-zfs-20261102
export LD_LIBRARY_PATH="$out/root/usr/lib/x86_64-linux-gnu"
export PATH="$out/root/usr/sbin:$PATH"
export ZDB="$out/root/usr/sbin/zdb"
mkdir -p "$out/images/mirror"
cd "$out/images/mirror"
ulimit -c 0
ulimit -v 4194304
ulimit -f 4194304
/usr/bin/time -v -o "$out/logs/create-retry-time.txt" timeout --kill-after=5s 90s "$out/root/usr/sbin/ztest" -f "$out/images/mirror" -p nieuwos_n3 -v 1 -s 256M -m 2 -r 1 -R 0 -K raidz -C special=off -C dedup=off -d 2 -t 2 -T 15 -P 5 -k 0 -o zfs_arc_max=268435456 -o zfs_arc_min=67108864 -VVV > "$out/logs/create-retry.txt" 2>&1
rc=$?
printf '%s\n' "$rc" > "$out/logs/create-retry-exit.txt"
tail -70 "$out/logs/create-retry.txt"
printf 'EXIT=%s\n' "$rc"
exit "$rc"