"""Independent evidence checks for the native Materiaal library walkthrough.

Only disposable, source-bound window-probe runs may supply a cold-reopen image.
This module decodes stopped disk bytes; app counters do not certify persistence.
"""
from __future__ import annotations

from pathlib import Path
import re

from common import BUILD, sha256
from run_fpu import load_json, private_path
import volume_disk

TITLE = "Reisnotities"
COLLECTION = "Reis"
ORIGINAL = b"een week aan de kust\n\nwe reizen met de trein en fietsen door de duinen\nhet huisje ligt vlak bij zee"
EDITED = b"een week aan de kust\n\nwe vertrekken op zaterdag en nemen de fiets mee\nhet huisje ligt vlak bij zee"
PHASES = ("library-empty", "library-named-draft", "library-saved",
          "library-organized", "library-edited", "library-revision-saved",
          "library-history", "library-restored", "library-search", "library-opened",
          "library-cold-list", "library-cold-search", "library-cold-opened", "library-cold-history")


def walkthrough(*, key, wait, cold: bool) -> None:
    """Real keys only. Every checkpoint requires its entire exact app image."""
    def checkpoint(name, *, contents=None, dirty=None, saves=None):
        def ready(raw, state):
            if not all(state.values()):
                return False
            current = state["MATERIAL"][-1]
            if current["ERRORS"] != 0 or current["PROBES"] != 3:
                raise ValueError("Native library error or missing isolation probes")
            if contents is not None and (current["DRAFT_BYTES"], current["DRAFT_SUM"]) != (len(contents), volume_disk.fnv(contents)):
                return False
            return ((dirty is None or current["DIRTY"] == dirty)
                    and (saves is None or current["SAVE_ACKS"] == saves))
        return wait(name, ready, name)

    def text(value):
        for character in value:
            if character == " ":
                key("spc")
            elif character == "\n":
                key("ret")
            elif "A" <= character <= "Z":
                key("shift", character.lower())
            elif "a" <= character <= "z":
                key(character)
            else:
                raise ValueError("Unsupported synthetic test character")

    if cold:
        checkpoint("library-cold-list", dirty=0, saves=0)
        key("ctrl", "f"); text("reis")
        checkpoint("library-cold-search", dirty=0, saves=0)
        key("ret")
        checkpoint("library-cold-opened", contents=ORIGINAL, dirty=0, saves=0)
        key("alt", "h")
        checkpoint("library-cold-history", contents=ORIGINAL, dirty=0, saves=0)
        key("esc")
        return
    checkpoint("library-empty", dirty=0, saves=0)
    key("ret"); key("f2"); text(TITLE); key("ret")
    text(ORIGINAL.decode("ascii"))
    checkpoint("library-named-draft", contents=ORIGINAL, dirty=1, saves=0)
    key("ctrl", "s")
    checkpoint("library-saved", contents=ORIGINAL, dirty=0, saves=1)
    key("ctrl", "g"); text(COLLECTION); key("ret")
    checkpoint("library-organized", contents=ORIGINAL, dirty=0, saves=1)
    key("ctrl", "a"); text(EDITED.decode("ascii"))
    checkpoint("library-edited", contents=EDITED, dirty=1, saves=1)
    key("ctrl", "s")
    checkpoint("library-revision-saved", contents=EDITED, dirty=0, saves=2)
    key("alt", "h")
    checkpoint("library-history", contents=EDITED, dirty=0, saves=2)
    key("tab"); key("ret")
    checkpoint("library-restored", contents=ORIGINAL, dirty=0, saves=2)
    key("esc"); key("ctrl", "f"); text("reis")
    checkpoint("library-search", contents=ORIGINAL, dirty=0, saves=2)
    key("ret")
    checkpoint("library-opened", contents=ORIGINAL, dirty=0, saves=2)


def disk_oracle(image: bytes) -> dict:
    # `opaque` requests exact content bytes from the existing strict decoder;
    # this does not exempt checksums, table structure, bounds or disk canaries.
    decoded = volume_disk.analyse(image, opaque=frozenset({0}))
    present = [item for item in decoded["objects"] if item["present"]]
    if len(present) != 1 or present[0]["object"] != 0:
        raise ValueError("Library must contain exactly its one actual document")
    document = present[0]
    versions = document["versions"]
    if (document["collection"] != 1 or document["tags"] != 0 or document["kind"] != 0
            or len(versions) != 2 or document["revision"] != 4
            or versions[0]["name"] != TITLE
            or bytes.fromhex(versions[0]["content_hex"]) != ORIGINAL
            or bytes.fromhex(versions[1]["content_hex"]) != EDITED
            or any(item["name"] != TITLE for item in versions)
            or (versions[0]["revision"], versions[1]["revision"]) != (4, 3)):
        raise ValueError("Wrong native rename, organization or restore-as-new-revision")
    collections = [item for item in decoded["collections"] if item["present"]]
    if (len(collections) != 1 or collections[0]["domain"] != 0
            or collections[0]["index"] != 1 or collections[0]["name"] != COLLECTION
            or collections[0]["revision"] != 1):
        raise ValueError("Wrong persisted collection")
    if (any(item["revision"] for item in decoded["objects"][1:])
            or any(item["revision"] for item in decoded["collections"][1:])
            or decoded["generation"] != 5 or decoded["clocks"] != [5, 0]):
        raise ValueError("Library touched another document or domain")
    return decoded


def cold_parent(directory: Path, *, source: str, manifest: dict) -> dict:
    """Revalidate a stopped successful parent; never use a user disk as input."""
    directory = private_path(directory)
    try:
        relative = directory.relative_to(private_path(BUILD / "run"))
    except ValueError as error:
        raise ValueError("Cold library input escaped its disposable run directory") from error
    if len(relative.parts) != 1 or re.fullmatch(r"window-[0-9a-f]{32}", relative.name) is None:
        raise ValueError("Cold library input is not a window runner directory")
    report_path = private_path(directory / "result.json", regular=True)
    report = load_json(report_path)
    required = {
        "passed": True, "library_gate": True, "library_requested": True,
        "library_cold": False, "native_gate": False, "raw_qemu_exit": 33,
        "smoke_only": False,
        "kernel_feature": "native-window-session-probe", "source_fingerprint_version": 3,
        "source_sha256": source, "manifest": manifest,
        "run_directory": str(directory),
    }
    if any(type(report.get(key)) is not type(value) or report.get(key) != value
           for key, value in required.items()):
        raise ValueError("Cold library parent is not a current accepted writable run")
    if report.get("harness_terminated_process") or report.get("error"):
        raise ValueError("Cold library parent did not finish normally")
    phases = report.get("phases", {})
    if list(phases) != list(PHASES[:10]):
        raise ValueError("Cold library parent lacks its complete ordered workload")
    for name in PHASES[:10]:
        frame = private_path(directory / (name + ".ppm"), regular=True)
        if (phases[name].get("oracle") != name or frame.stat().st_size > 3 * 1920 * 1080 + 128
                or phases[name].get("frame_sha256") != sha256(frame)):
            raise ValueError("Cold library parent screenshot changed")
    qemu = report.get("qemu_sha256")
    implementation = report.get("qemu_implementation", {}).get("sha256")
    command = report.get("command")
    if (not isinstance(qemu, str) or re.fullmatch(r"[0-9a-f]{64}", qemu) is None
            or not isinstance(implementation, str) or re.fullmatch(r"[0-9a-f]{64}", implementation) is None
            or not isinstance(command, list) or not command or not all(isinstance(item, str) for item in command)):
        raise ValueError("Cold library parent lacks a bound emulator profile")
    disk = private_path(directory / "volume.img", regular=True)
    transcript = private_path(directory / "serial.txt", regular=True)
    if (disk.stat().st_size != volume_disk.IMAGE_SECTORS * volume_disk.SECTOR
            or transcript.stat().st_size > 4 * 1024 * 1024
            or report.get("volume_sha256") != sha256(disk)
            or report.get("serial_sha256") != sha256(transcript)):
        raise ValueError("Cold library parent disk or transcript changed")
    raw = transcript.read_bytes()
    if (not raw.endswith(b"WINDOW:CLEANUP:PASS\nWINDOW:CASE:PASS\nM1:PASS\n")
            or b"WINDOW:FAIL:" in raw or b"PANIC" in raw or b"\0" in raw):
        raise ValueError("Cold library parent lacks real verified cleanup")
    image = disk.read_bytes()
    decoded = disk_oracle(image)
    if report.get("disk_oracle") != decoded:
        raise ValueError("Cold library parent oracle does not match actual bytes")
    return {"image": image, "volume_sha256": sha256(disk),
            "report_sha256": sha256(report_path), "directory": str(directory),
            "qemu_sha256": qemu, "qemu_implementation_sha256": implementation, "command": command}
