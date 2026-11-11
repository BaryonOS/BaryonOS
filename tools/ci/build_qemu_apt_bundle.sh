#!/usr/bin/env bash
# Build a verified partial Ubuntu snapshot mirror for .github/actions/setup-qemu.
#
# Usage: tools/ci/build_qemu_apt_bundle.sh <24.04|26.04> <output.tar> <package=version>...
#
# The bundle holds Ubuntu's signed InRelease files, the original Packages.xz
# indexes and every .deb in the dependency closure of the given pins, computed
# against an empty dpkg status. Nothing in it is trusted on its own: the CI
# consumer points apt at it with Signed-By the Ubuntu archive keyring, so apt
# re-verifies the signature, every index hash and every package hash.
# SOURCES.txt lists the matching source packages (GPL corresponding source).
set -euo pipefail
export LC_ALL=C

# Any official Ubuntu mirror works: apt checks the Ubuntu signature, not the host.
snapshot="${UBUNTU_MIRROR:-https://snapshot.ubuntu.com/ubuntu/20261108T000000Z}"
keyring='/usr/share/keyrings/ubuntu-archive-keyring.gpg'
profile="${1:?profile}"; output="${2:?output tar}"; shift 2
test "$#" -gt 0
case "$profile" in
  24.04) codename=noble ;;
  26.04) codename=resolute ;;
  *) echo "unknown profile $profile" >&2; exit 2 ;;
esac
suites=("$codename" "$codename-updates" "$codename-security")
components=(main universe)

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
root="$work/mirror"
mkdir -p "$work/lists/partial" "$work/cache/archives/partial" "$root"
: > "$work/status"

fetch() {  # <url> <file>; mirrors may refuse connections intermittently
  for attempt in 1 2 3 4 5 6 7 8; do
    curl -fsSL --max-time 300 -o "$2" "$1" && return 0
    sleep "$attempt"
  done
  return 1
}

write_sources() {  # <uri> <types> <targets>
  cat > "$work/ubuntu.sources" <<SRC
Types: $2
Targets: $3
URIs: $1
Suites: ${suites[*]}
Components: ${components[*]}
Architectures: amd64
Signed-By: $keyring
SRC
}

apt_options=(
  -o "Dir::Etc::sourcelist=$work/ubuntu.sources" -o Dir::Etc::sourceparts=-
  -o Dir::Etc::preferences=/dev/null -o "Dir::Etc::preferencesparts=$work/lists/partial"
  -o "Dir::State::lists=$work/lists" -o "Dir::State::status=$work/status"
  -o "Dir::Cache=$work/cache" -o Debug::NoLocking=1
  -o APT::Architecture=amd64 -o APT::Architectures=amd64
  -o APT::Update::Error-Mode=any -o Acquire::Retries=5
  -o Acquire::ForceHash=SHA256
  -o APT::Get::Download-Only=true
)

write_sources "$snapshot" 'deb deb-src' 'Packages Sources'
apt-get "${apt_options[@]}" -qq update

# Closure from an empty status: every dependency the runner could need.
apt-get "${apt_options[@]}" -qq install --print-uris -y --no-install-recommends "$@" \
  > "$work/uris"
test -s "$work/uris"
while read -r uri _ size hash; do
  uri="${uri//\'/}"; path="${uri#"$snapshot"/}"
  test "$path" != "$uri"; case "$path" in pool/*) ;; *) exit 1 ;; esac
  mkdir -p "$root/$(dirname "$path")"
  fetch "$uri" "$root/$path"
  test "$(stat -c %s "$root/$path")" = "$size"
  echo "${hash#SHA256:}  $root/$path" | sha256sum --check --quiet
done < "$work/uris"

# Original signed indexes; apt verifies them again on every consumer run.
for suite in "${suites[@]}"; do
  mkdir -p "$root/dists/$suite"
  fetch "$snapshot/dists/$suite/InRelease" "$root/dists/$suite/InRelease"
  gpgv --keyring "$keyring" "$root/dists/$suite/InRelease" 2>/dev/null
  ! grep -q '^Valid-Until:' "$root/dists/$suite/InRelease"
  for component in "${components[@]}"; do
    index="$component/binary-amd64/Packages.xz"
    mkdir -p "$root/dists/$suite/$component/binary-amd64"
    fetch "$snapshot/dists/$suite/$index" "$root/dists/$suite/$index"
    expected="$(awk -v f="$index" '/^SHA256:/{s=1;next} /^[^ ]/{s=0} s&&$3==f{print $1}' \
      "$root/dists/$suite/InRelease")"
    echo "$expected  $root/dists/$suite/$index" | sha256sum --check --quiet
  done
done

# Corresponding source for every binary in the bundle.
{
  echo "# Source packages for the binaries in this bundle, from $snapshot"
  # shellcheck disable=SC2046
  apt-get "${apt_options[@]}" -qq source --print-uris $(
    for deb in $(find "$root/pool" -name '*.deb' | sort); do
      dpkg-deb -f "$deb" Package Version Source |
        awk -F': ' '/^Package/{p=$2} /^Version/{v=$2} /^Source/{s=$2}
          END{split(s,a," "); src=a[1]?a[1]:p; ver=a[2]?substr(a[2],2,length(a[2])-2):v; print src"="ver}'
    done | sort -u)
} > "$root/SOURCES.txt"

# Self-check: the consumer path must install the pins from the bundle alone.
rm -rf "$work/lists" "$work/cache"; mkdir -p "$work/lists/partial" "$work/cache/archives/partial"
write_sources "file:$root" deb Packages
apt-get "${apt_options[@]}" -o Acquire::By-Hash=false -qq update
apt-get "${apt_options[@]}" -qq install --print-uris -y --no-install-recommends "$@" \
  > "$work/self-check"
test "$(grep -vc "^'file:$root/pool/" "$work/self-check" || true)" = 0

(cd "$root" && find pool -type f -name '*.deb' | sort | xargs sha256sum) > "$root/SHA256SUMS"
tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner -C "$root" -cf "$output" .
sha256sum "$output"
