"""B08 driver: frozen actual G0 build/run and unchanged capture CLI."""
import argparse
from datetime import datetime, timezone
import hashlib
from io import BytesIO
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--commit", required=True)
parser.add_argument("--source", required=True)
args = parser.parse_args()
ROOT = Path(__file__).resolve().parents[2]
BUILD = Path(__file__).resolve().parent
DESTINATION = ROOT / "docs/verification/parallel-followup-g0"
sys.path.insert(0, str(ROOT / "tools"))
from common import source_hash

ROM = Path("/tmp/nieuw-os-par-tools/qemu/usr/share/seabios/vgabios-stdvga.bin")
PIN = "cc2f735f19b6318922ac3de9506dee498f149a6b75534f7e5c176d4441a7fa4a"
record = dict(schema_version=1, source_commit=args.commit,
    source_fingerprint_version=3, source_sha256=args.source, passed=False,
    scope="Actual isolated G0 build and pipe VM; unchanged reusable capture; independent archived PNG byte check",
    started_utc=datetime.now(timezone.utc).isoformat(), commands=[])


def save():
    (BUILD / "execution.json").write_text(json.dumps(record, indent=2) + "\n")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def frozen():
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip() == args.commit
    assert source_hash() == args.source


def execute(name, command, timeout):
    frozen()
    started = time.monotonic()
    item = dict(name=name, command=command, cwd=str(ROOT), started_utc=datetime.now(timezone.utc).isoformat())
    with (BUILD / (name + ".stdout.txt")).open("wb") as stdout, (BUILD / (name + ".stderr.txt")).open("wb") as stderr:
        result = subprocess.run(command, cwd=ROOT, stdout=stdout, stderr=stderr, timeout=timeout)
    item.update(exit_status=result.returncode, duration_s=round(time.monotonic() - started, 6))
    record["commands"].append(item)
    save()
    print(f"{name}: exit={result.returncode}, seconds={item['duration_s']}", flush=True)
    if result.returncode:
        raise RuntimeError(f"{name} failed; preserve original outputs under {BUILD}")
    frozen()


try:
    frozen()
    assert os.environ["NOS_BUILD_DIR"] == str(BUILD)
    assert os.environ["NOS_TARGET_DIR"] == os.environ["CARGO_TARGET_DIR"] == "/tmp/nieuw-os-par-followup-g0-target"
    assert os.environ["NOS_SDK_RUNTIME"] == "0" and os.environ["CARGO_BUILD_JOBS"] == "2"
    assert os.environ["NOS_QEMU_MACHINE"] == "pc-q35-7.2"
    assert not DESTINATION.exists()
    assert sha(ROM) == PIN
    record["environment"] = {key: os.environ[key] for key in ("NOS_BUILD_DIR", "NOS_TARGET_DIR", "CARGO_TARGET_DIR", "CARGO_HOME", "CARGO_BUILD_JOBS", "NOS_QEMU_MACHINE", "NOS_SDK_RUNTIME", "OS_OVMF_CODE", "OS_OVMF_VARS")}
    save()
    execute("build", [sys.executable, "-B", "tools/build.py", "--kernel-feature", "graphics-g0"], 180)
    execute("run", [sys.executable, "-B", "tools/run_graphics.py", "--qmp-transport", "pipe",
        "--machine", "pc-q35-7.2", "--run-root", "/tmp/nieuw-os-followup-g0",
        "--vga-rom", str(ROM), "--vga-rom-sha256", PIN], 90)
    report = json.loads((BUILD / "graphics-g0-result.json").read_text())
    assert report["passed"] is True and report["raw_qemu_exit"] == 33 and report["error"] is None
    assert all(value is True for value in report["checks"].values())
    run = Path(report["run_directory"])
    record.update(run_directory=str(run), raw_qemu_exit=report["raw_qemu_exit"], native_checks=report["checks"])
    save()
    execute("capture", [sys.executable, "-B", "tools/capture_graphics.py", "--run-dir", str(run),
        "--destination", "docs/verification/parallel-followup-g0"], 60)

    # Independent standard PNG decoder, never the capture's own decoder.
    from PIL import Image, __version__ as pillow_version
    captured = json.loads((DESTINATION / "capture.json").read_text())
    first_input = {name: sha(run / name) for name in captured["input_sha256"]}
    first_archive = {name: sha(DESTINATION / name) for name in captured["files"]}
    assert first_input == captured["input_sha256"]
    assert first_archive == captured["files"]
    assert (DESTINATION / "serial.txt").read_bytes() == (run / "serial.txt").read_bytes() == report["stdout"].encode()
    index = json.loads((DESTINATION / "image-roundtrip.json").read_text())
    assert [entry["name"] for entry in index["images"]] == ["initial", "a", "ab", "after-backspace", "typed"]
    images = []
    for entry in index["images"]:
        original = (run / (entry["name"] + ".ppm")).read_bytes()
        encoded = (DESTINATION / (entry["name"] + ".png")).read_bytes()
        with Image.open(BytesIO(encoded)) as image:
            image.load()
            assert image.mode == "RGB" and image.size == (entry["width"], entry["height"])
            rebuilt = bytes.fromhex(entry["original_header_hex"]) + image.tobytes()
        assert rebuilt == original
        assert hashlib.sha256(original).hexdigest() == entry["ppm_sha256"] == report["artifacts"][entry["name"] + ".ppm"]
        assert hashlib.sha256(encoded).hexdigest() == entry["png_sha256"]
        images.append(dict(name=entry["name"], ppm_sha256=entry["ppm_sha256"], png_sha256=entry["png_sha256"], byte_exact=True))
    assert first_input == {name: sha(run / name) for name in captured["input_sha256"]}
    assert first_archive == {name: sha(DESTINATION / name) for name in captured["files"]}
    frozen()
    record.update(passed=True, completed_utc=datetime.now(timezone.utc).isoformat(),
        archive_crosscheck=dict(input_files=len(first_input), archived_files=len(first_archive),
            input_sha256=first_input, archive_sha256=first_archive, independent_decoder="Pillow " + pillow_version,
            images=images, actual_serial_bytes=len((run / "serial.txt").read_bytes()), actual_serial_sha256=sha(run / "serial.txt")))
    save()
    for name in ("execute_gate.py", "execution.json", "build.stdout.txt", "build.stderr.txt", "run.stdout.txt", "run.stderr.txt", "capture.stdout.txt", "capture.stderr.txt"):
        shutil.copy2(BUILD / name, DESTINATION / name)
    shutil.copytree(BUILD / "logs", DESTINATION / "build-logs")
    print("B08 accepted: actual G0 exit33, reusable capture, five independent exact PPM reconstructions", flush=True)
except Exception as error:
    record.update(error=f"{type(error).__name__}: {error}", completed_utc=datetime.now(timezone.utc).isoformat())
    save()
    raise
