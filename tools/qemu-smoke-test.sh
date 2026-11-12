#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# L0 K06 section 9 regressions. These gates run sequentially after the matrix.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
regression_root="${NOS_BUILD_DIR:-$repo_root/build}/regressions"
regression_target="${NOS_TARGET_DIR:-$repo_root/target}/regressions"
if [ -e "$regression_root" ]; then
    echo "Regression evidence directory already exists: $regression_root" >&2
    exit 2
fi
mkdir -p "$regression_root/logs"

fingerprint() {
    PYTHONPATH="$repo_root/tools" python3 -B -c 'from common import source_hash; print(source_hash())'
}
frozen="$(fingerprint)"
printf 'Regression source_sha256=%s\n' "$frozen" | tee "$regression_root/source.txt"

run_step() {
    local stage="$1" started="$SECONDS" status
    shift
    if [ "$(fingerprint)" != "$frozen" ]; then
        echo "Source changed before regression stage $stage" >&2
        return 2
    fi
    printf 'Regression stage=%s command=' "$stage"
    printf '%q ' "$@"
    printf '\n'
    if "$@" 2>&1 | tee "$regression_root/logs/$stage.log"; then
        status=0
    else
        status=$?
    fi
    printf 'Regression stage=%s exit=%s elapsed_s=%s\n' "$stage" "$status" "$((SECONDS - started))"
    if [ "$status" -ne 0 ]; then
        return "$status"
    fi
    if [ "$(fingerprint)" != "$frozen" ]; then
        echo "Source changed during regression stage $stage" >&2
        return 2
    fi
}

# The complete Python oracles run in the aggregate job with pinned PyYAML.
# Keep Rust host checks here before the native regression sequence.
run_step host-tests cargo +1.90.0 test --locked -p boot-protocol -p smp-lock
run_step clippy cargo +1.90.0 clippy --locked -p boot-protocol -p smp-lock --all-targets -- -D warnings
run_step fmt cargo +1.90.0 fmt --all -- --check

# Existing runners retain their own guest watchdogs, quotas and fail-closed
# verdicts. Separate build and target directories prevent profile/image mixing.
run_step m2 env NOS_BUILD_DIR="$regression_root/m2" NOS_TARGET_DIR="$regression_target/m2" CARGO_TARGET_DIR="$regression_target/m2" \
    python3 -B tools/test_milestones.py --through m2
run_step k05 env NOS_BUILD_DIR="$regression_root/k05" NOS_TARGET_DIR="$regression_target/k05" CARGO_TARGET_DIR="$regression_target/k05" \
    python3 -B -u tools/test_supervisor.py
run_step k05-smp env NOS_BUILD_DIR="$regression_root/k05-smp" NOS_TARGET_DIR="$regression_target/k05-smp" CARGO_TARGET_DIR="$regression_target/k05-smp" \
    python3 -B -u tools/test_supervisor.py --smp
run_step k07 env NOS_BUILD_DIR="$regression_root/k07" NOS_TARGET_DIR="$regression_target/k07" CARGO_TARGET_DIR="$regression_target/k07" \
    python3 -B tools/test_resource_pressure.py
run_step rtc-build env NOS_BUILD_DIR="$regression_root/rtc" NOS_TARGET_DIR="$regression_target/rtc" CARGO_TARGET_DIR="$regression_target/rtc" \
    python3 -B tools/build.py --kernel-feature native-rtc
run_step rtc env NOS_BUILD_DIR="$regression_root/rtc" NOS_TARGET_DIR="$regression_target/rtc" CARGO_TARGET_DIR="$regression_target/rtc" \
    python3 -B tools/run_rtc.py --case all
printf 'Regression gates passed on source_sha256=%s\n' "$frozen"
