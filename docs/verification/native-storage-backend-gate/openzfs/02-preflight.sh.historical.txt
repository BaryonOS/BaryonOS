set -u
out=/tmp/nieuw-os-storage-zfs-20261102
export LD_LIBRARY_PATH="$out/root/usr/lib/x86_64-linux-gnu"
ldd "$out/root/usr/sbin/ztest" | tee "$out/logs/ztest-ldd.txt"
ldd "$out/root/usr/sbin/zdb" | tee "$out/logs/zdb-ldd.txt"
"$out/root/usr/sbin/ztest" -h > "$out/logs/ztest-help.txt" 2>&1
cat "$out/logs/ztest-help.txt"
"$out/root/usr/sbin/zdb" -h > "$out/logs/zdb-help.txt" 2>&1
head -70 "$out/logs/zdb-help.txt"