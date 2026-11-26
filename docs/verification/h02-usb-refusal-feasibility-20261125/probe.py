"""Feasibility probe only: swap the port-1 usb-kbd for a candidate device at launch time."""
import sys, re, pathlib
root = pathlib.Path(sys.argv[1]); label = sys.argv[2]
sys.path.insert(0, str(root / "tools"))
import run_usb_hid
build = pathlib.Path(run_usb_hid.BUILD)  # honours NOS_BUILD_DIR, defaults to <checkout>/build
disk = build / "run" / "probe-disk.img"
disk.parent.mkdir(parents=True, exist_ok=True); disk.write_bytes(b"\0" * (1 << 20))
REPL = {
    "hub": ["-device", "usb-hub,id=probe,bus=xhci.0,port=1"],
    "tablet": ["-device", "usb-tablet,id=probe,bus=xhci.0,port=1,usb_version=1"],
    "storage": ["-drive", f"if=none,id=probe-disk,format=raw,file={disk}",
                "-device", "usb-storage,id=probe,drive=probe-disk,bus=xhci.0,port=1"],
    "hs-kbd": ["-device", "usb-kbd,id=probe,bus=xhci.0,port=1,usb_version=2"],
}[label]
orig = run_usb_hid.controlled_guest
seen = {}
def patched(args, **kw):
    i = args.index("usb-kbd,id=hid-kbd,bus=xhci.0,port=1,usb_version=1")
    args[i - 1:i + 1] = REPL
    seen["dir"] = kw["run_dir"]
    return orig(args, **kw)
run_usb_hid.controlled_guest = patched
try:
    run_usb_hid.run_case("reports")
    print("UNEXPECTED: reports case passed")
except Exception as exc:
    print("runner:", type(exc).__name__, str(exc)[:160])
out = (pathlib.Path(seen["dir"]) / "usb-stdout.bin").read_bytes().decode("utf-8", "replace")
lines = [l for l in out.splitlines() if re.match(r"(USB|PANIC|M2:|.*FAIL)", l)]
print("\n".join(lines[-8:]))
