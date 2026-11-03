set -eu
out=/tmp/nieuw-os-storage-zfs-20261102
win=/mnt/c/Users/simon/Documents/Codex/2026-11-02/native-storage-gate/openzfs
mkdir -p "$win/logs" "$win/packages" "$win/sources" "$win/licenses" "$win/exports"
cp "$out/logs/"* "$win/logs/"
cp "$out/packages/"*.deb "$win/packages/"
cp "$out/"*.c "$win/sources/"
cp "$out/root/usr/share/doc/zfs-test/copyright" "$win/licenses/ubuntu-zfs-copyright"
cp "$out/root/usr/share/doc/zfs-test/README.Debian" "$win/licenses/README.Debian"
cp "$out/exports/ds0-raw.zfs" "$win/exports/"
tar -czf "$win/images/crash-pool.tar.gz" -C "$out/images" crash
sha256sum "$out/root/usr/sbin/ztest" "$out/root/usr/sbin/zdb" "$out/root/usr/lib/x86_64-linux-gnu/libzpool.so.7" > "$win/logs/executed-binary-sha256.txt"
sha256sum "$out/images/mirror/nieuwos_n3.0a" "$out/images/mirror/nieuwos_n3.1a" "$out/images/crash/nieuwos_crash.0a" "$out/images/crash/nieuwos_crash.1a" > "$win/logs/final-images-sha256.txt"
ls -lh "$win/images/crash-pool.tar.gz"