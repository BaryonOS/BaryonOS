#!/usr/bin/env bash
# Called by CI after the existing pinned setup and locked dependency fetch.
set -euo pipefail
if [[ $# != 5 ]]; then
  printf 'usage: %s LEG ORDER A_TREE B_TREE REPORT_DIR\n' "$0" >&2
  exit 2
fi
leg=$1
order=$2
case "$leg" in
  native-scale2) flags=(--clipboard --workspace --scale 2 --timeout 2700) ;;
  switcher-scale2) flags=(--switcher --scale 2 --timeout 1800) ;;
  *) exit 2 ;;
esac
case "$order" in AB|BA) ;; *) exit 2 ;; esac
a_tree=$(realpath "$3")
b_tree=$(realpath "$4")
[[ "$a_tree" != "$b_tree" && -z ${NOS_QEMU_ACCEL:-} ]]
declare -A trees=([A]="$a_tree" [B]="$b_tree")
declare -A sources=(
  [A]=c5867e4584b4012c5db0f704a6d4e6ae1d9e4257
  [B]=560758fb6e34b0c215587fd5503dea0a49ee710d
)
for arm in A B; do
  [[ $(git -C "${trees[$arm]}" rev-parse HEAD) == "${sources[$arm]}" ]]
  [[ -z $(git -C "${trees[$arm]}" status --porcelain) ]]
done
mkdir -p "$5"
artifacts=$(realpath "$5")
bios=/usr/share/seabios/bios-256k.bin
vga=/usr/share/seabios/vgabios-stdvga.bin
export NOS_QEMU_MACHINE=pc-q35-8.2 CARGO_NET_OFFLINE=true
qemu=$(command -v qemu-system-x86_64)
{
  date -u +%FT%TZ
  printf 'order=%s leg=%s run=%s attempt=%s job=%s image=%s\n' \
    "$order" "$leg" "${GITHUB_RUN_ID:-local}" "${GITHUB_RUN_ATTEMPT:-}" "${GITHUB_JOB:-}" "${ImageVersion:-}"
  lscpu
  rustc +1.100.0 -vV
  "$qemu" --version
  sha256sum "$qemu" "$OS_OVMF_CODE" "$OS_OVMF_VARS" "$bios" "$vga"
} > "$artifacts/runner.txt"
run_arm() (
  local arm=$1
  local output="$artifacts/$arm"
  mkdir "$output" || exit $?
  cd "${trees[$arm]}" || exit $?
  local name="k-h1-$leg-$order"
  export NOS_BUILD_DIR="$PWD/build/$name"
  export NOS_TARGET_DIR="$PWD/target/$name-native"
  export CARGO_TARGET_DIR="$PWD/target/$name-host"
  [[ ! -e "$NOS_BUILD_DIR" ]] || exit 2
  git rev-parse HEAD > "$output/source.txt" || exit $?
  local started=$SECONDS
  local build_rc=0 gate_rc=0 report_rc=0
  python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi-probe > "$output/build.log" 2>&1 || build_rc=$?
  printf 'build_exit=%d build_seconds=%d\n' "$build_rc" "$((SECONDS-started))" > "$output/status.txt"
  if ((build_rc != 0)); then exit "$build_rc"; fi
  sha256sum "$NOS_BUILD_DIR/esp/kernel.elf" "$NOS_BUILD_DIR/manifest.json" > "$output/build.sha256" || exit $?
  started=$SECONDS
  python3 -B -u tools/run_window_session.py "${flags[@]}" --bios-rom "$bios" --vga-rom "$vga" > "$output/gate.log" 2>&1 || gate_rc=$?
  printf 'gate_exit=%d gate_seconds=%d\n' "$gate_rc" "$((SECONDS-started))" >> "$output/status.txt"
  cp "$NOS_BUILD_DIR/window-session-report.json" "$output/report.json" || exit $?
  python3 -B - "$output/report.json" "${sources[$arm]}" "$leg" "$PWD" <<'PY' || report_rc=$?
import hashlib
import json
import shutil
import sys
from pathlib import Path

path, source, leg, workspace = sys.argv[1:]
report = json.loads(Path(path).read_text())
assert report["source_commit"] == source
run = Path(report["run_directory"]).resolve(strict=True)
run.relative_to(Path(workspace).resolve())
serial = run / "serial.txt"
with serial.open("rb") as stream:
    assert hashlib.file_digest(stream, "sha256").hexdigest() == report["serial_sha256"]
shutil.copyfile(serial, Path(path).with_name("serial.txt"))
assert report["passed"] is True and report["raw_qemu_exit"] == 33
assert report["scale"] == 2 and report["kernel_feature"] == "native-window-session-hidpi-probe"
command = report["command"]
assert command[command.index("-accel") + 1] == "tcg,thread=single"
if leg == "switcher-scale2":
    assert report["switcher_gate"] is True and report["native_gate"] is False
    assert len(report["phases"]) == 18
else:
    required = ("native_gate", "native_scale200_gate", "clipboard_requested", "clipboard_gate",
                "workspace_requested", "workspace_gate", "document_tools_gate", "document_menu_gate",
                "pointer_leave_gate", "app_high_contrast_gate", "window_states_gate",
                "keyboard_window_move_gate", "notification_panel_gate", "trusted_lock_gate", "locked_f7_gate")
    assert all(report[key] is True for key in required)
    assert report["smoke_only"] is False
    guest_disk = run / "volume.img"
    disk_relative = guest_disk.resolve(strict=True).relative_to(Path(workspace).resolve())
    with guest_disk.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    Path(path).with_name("volume.sha256").write_text(f"{digest}  {disk_relative}\n")
PY
  printf 'report_exit=%d\n' "$report_rc" >> "$output/status.txt"
  [[ -z $(git status --porcelain) ]] || exit 2
  ((gate_rc == 0 && report_rc == 0))
)
pair_rc=0
for ((n=0; n<2; n++)); do
  run_arm "${order:n:1}" || pair_rc=1
done
exit "$pair_rc"
