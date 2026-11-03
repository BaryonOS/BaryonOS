set -eu
out=/tmp/nieuw-os-storage-zfs-20261102
win=/mnt/c/Users/simon/Documents/Codex/2026-11-02/native-storage-gate/openzfs
mkdir -p "$win/images" "$win/logs"
sha256sum "$out/images/mirror/nieuwos_n3.0a" "$out/images/mirror/nieuwos_n3.1a" > "$out/logs/baseline-sha256.txt"
cp --sparse=always "$out/images/mirror/nieuwos_n3.0a" "$win/images/nieuwos_n3.0a"
cp --sparse=always "$out/images/mirror/nieuwos_n3.1a" "$win/images/nieuwos_n3.1a"
sha256sum "$win/images/nieuwos_n3.0a" "$win/images/nieuwos_n3.1a" > "$out/logs/windows-copy-sha256.txt"
cp "$out/logs/create-bounded"* "$out/logs/baseline-sha256.txt" "$out/logs/windows-copy-sha256.txt" "$win/logs/"
cat "$out/logs/windows-copy-sha256.txt"
export LD_LIBRARY_PATH="$out/root/usr/lib/x86_64-linux-gnu"
export PATH="$out/root/usr/sbin:$PATH"
export MALLOC_ARENA_MAX=2
export ZFS_STACK_SIZE=131072
ulimit -c 0
ulimit -v 8388608
set +e
/usr/bin/time -v -o "$out/logs/fresh-zdb-time.txt" timeout --kill-after=5s 60s "$out/root/usr/sbin/zdb" -bcc -G -d -Y -e -y -p "$out/images/mirror" 9169139500157372876 > "$out/logs/fresh-zdb.txt" 2>&1
rc=$?
printf '%s\n' "$rc" > "$out/logs/fresh-zdb-exit.txt"
tail -30 "$out/logs/fresh-zdb.txt"
exit "$rc"