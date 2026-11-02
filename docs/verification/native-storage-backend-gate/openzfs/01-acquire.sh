set -eu
out=/tmp/nieuw-os-storage-zfs-20261102
cd "$out/packages"
apt-cache show zfs-test=2.4.1-1ubuntu5.1 zfsutils-linux=2.4.1-1ubuntu5.1 libzpool7linux=2.4.1-1ubuntu5.1 libzfs7linux=2.4.1-1ubuntu5.1 libnvpair3linux=2.4.1-1ubuntu5.1 libuutil3linux=2.4.1-1ubuntu5.1 > "$out/logs/package-metadata.txt"
timeout 120 apt download zfs-test=2.4.1-1ubuntu5.1 zfsutils-linux=2.4.1-1ubuntu5.1 libzpool7linux=2.4.1-1ubuntu5.1 libzfs7linux=2.4.1-1ubuntu5.1 libnvpair3linux=2.4.1-1ubuntu5.1 libuutil3linux=2.4.1-1ubuntu5.1 > "$out/logs/download.txt" 2>&1
for pkg in ./*.deb; do dpkg-deb -x "$pkg" "$out/root"; done
sha256sum ./*.deb > "$out/logs/package-sha256.txt"
find "$out/root" -type f \( -name ztest -o -name zdb -o -name zpool -o -name 'libzpool.so*' \)
cat "$out/logs/download.txt"