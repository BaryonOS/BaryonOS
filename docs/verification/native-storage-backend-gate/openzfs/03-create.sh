set -eu
out=/tmp/nieuw-os-storage-zfs-20261102
export LD_LIBRARY_PATH="$out/root/usr/lib/x86_64-linux-gnu"
export PATH="$out/root/usr/sbin:$PATH"
export ZDB="$out/root/usr/sbin/zdb"
mkdir -p "$out/images/mirror"
rm -f "$out/logs/baseline-pool-guid.txt"
if find "$out/images/mirror" -mindepth 1 -print -quit | grep -q .; then
  printf 'Create requires an empty mirror directory\n' >&2
  exit 1
fi
cd "$out/images/mirror"
ulimit -c 0
export MALLOC_ARENA_MAX=2
export ZFS_STACK_SIZE=131072
ulimit -v 8388608
ulimit -f 4194304
set +e
/usr/bin/time -v -o "$out/logs/create-bounded-time.txt" timeout --kill-after=5s 90s "$out/root/usr/sbin/ztest" -f "$out/images/mirror" -p nieuwos_n3 -v 1 -s 256M -m 2 -r 1 -R 0 -K raidz -C special=off -d 2 -t 2 -T 15 -P 5 -k 0 -VVV > "$out/logs/create-bounded.txt" 2>&1
rc=$?
set -e
printf '%s\n' "$rc" > "$out/logs/create-bounded-exit.txt"
tail -70 "$out/logs/create-bounded.txt"
printf 'EXIT=%s\n' "$rc"
if [ "$rc" -ne 0 ]; then exit "$rc"; fi
# Derive the identity from both newly created members; reject ambiguous labels.
for member in 0 1; do
  timeout --kill-after=5s 60s "$ZDB" -l "$out/images/mirror/nieuwos_n3.${member}a" > "$out/logs/create-label-${member}.txt"
done
python3 - "$out/logs/create-label-0.txt" "$out/logs/create-label-1.txt" "$out/logs/baseline-pool-guid.txt" <<'PYGUID'
import pathlib, re, sys
identities = []
for name in sys.argv[1:3]:
    text = pathlib.Path(name).read_text()
    names = set(re.findall(r"^\s*name: '([^']+)'\s*$", text, re.M))
    guids = set(re.findall(r'^\s*pool_guid: ([0-9]+)\s*$', text, re.M))
    if names != {'nieuwos_n3'} or len(guids) != 1 or int(next(iter(guids))) == 0:
        raise SystemExit(f'Invalid or ambiguous pool label: {name}')
    identities.append(next(iter(guids)))
if identities[0] != identities[1]:
    raise SystemExit('Mirror members have different pool GUIDs')
pathlib.Path(sys.argv[3]).write_text(identities[0] + '\n')
PYGUID
