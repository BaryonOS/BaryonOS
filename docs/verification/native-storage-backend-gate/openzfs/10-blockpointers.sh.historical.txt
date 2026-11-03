set -u
out=/tmp/nieuw-os-storage-zfs-20261102
export LD_LIBRARY_PATH="$out/root/usr/lib/x86_64-linux-gnu"
export MALLOC_ARENA_MAX=2 ZFS_STACK_SIZE=131072
ulimit -c 0
ulimit -v 8388608
timeout --kill-after=5s 40s "$out/root/usr/sbin/zdb" -bbbbb -e -p "$out/images/mirror" nieuwos_n3 > "$out/logs/baseline-blockpointers.txt" 2>&1
printf '%s\n' "$?" > "$out/logs/baseline-blockpointers-exit.txt"
head -28 "$out/logs/baseline-blockpointers.txt"