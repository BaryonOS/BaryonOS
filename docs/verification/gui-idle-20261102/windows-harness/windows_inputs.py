"""Windows input checks for the unchanged native window acceptance runner."""
from __future__ import annotations

import ctypes
from ctypes import wintypes
import os
from pathlib import Path, PurePosixPath, PureWindowsPath
import re
import stat


FIXTURE_KIND = "host-rendered real Controller/Frame oracle; not native execution"
METADATA_FIELDS = {"kind", "source_sha256", "source_fingerprint_version",
                   "generator_command", "generator_exit", "files_sha256"}
HEX = re.compile(r"[0-9a-f]{64}\Z")
REPARSE_POINT = 0x400
MAX_PPM_BYTES = 3 * 1120 * 660 + 128


def _lexical_path(value) -> Path:
    path = Path(value)
    raw = os.fspath(value)
    if not isinstance(raw, str) or "\0" in raw or ".." in path.parts:
        raise ValueError("Invalid or parent-relative artifact path")
    if raw.startswith(("\\\\", "//")) or path.drive.startswith("\\\\"):
        raise ValueError("UNC and extended namespace paths are not accepted")
    if path.drive and not path.is_absolute():
        raise ValueError("Drive-relative paths are not accepted")
    return path.absolute()


def _check_components(path: Path) -> None:
    for current in (*reversed(path.parents), path):
        try:
            info = current.lstat()
        except FileNotFoundError:
            continue
        if (stat.S_ISLNK(info.st_mode)
                or getattr(info, "st_file_attributes", 0) & REPARSE_POINT
                or (hasattr(current, "is_junction") and current.is_junction())):
            raise ValueError("Linked or reparse artifact path: " + str(current))
        if not (stat.S_ISREG(info.st_mode) or stat.S_ISDIR(info.st_mode)):
            raise ValueError("Special artifact path: " + str(current))
        if current != path and not stat.S_ISDIR(info.st_mode):
            raise ValueError("Artifact ancestor is not a directory")


def assert_plain_path(path, *, root=None, regular=False) -> Path:
    """Reject links/reparse points before resolving; retain any caller root bound."""
    path = _lexical_path(path)
    _check_components(path)
    if root is not None:
        boundary = _lexical_path(root)
        _check_components(boundary)
        if boundary.exists() and not boundary.is_dir():
            raise ValueError("Artifact root is not a directory")
        if not path.is_relative_to(boundary):
            raise ValueError("Artifact escaped its root")
    if regular:
        info = path.lstat()
        if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
            raise ValueError("Artifact must be a regular single-link file")
    return path


def _generator_command(command) -> bool:
    if (not isinstance(command, list) or len(command) != 11
            or any(not isinstance(item, str) or "\0" in item for item in command)
            or command[:6] != ["cargo", "+1.90.0", "test", "--offline", "--locked", "--manifest-path"]
            or command[7:] != ["--test", "window_native_fixture", "--", "--nocapture"]):
        return False
    manifest = PureWindowsPath(command[6]) if PureWindowsPath(command[6]).drive else PurePosixPath(command[6])
    return (manifest.is_absolute() and ".." not in manifest.parts
            and not manifest.drive.startswith("\\\\")
            and manifest.parts[-3:] == ("ui", "native", "Cargo.toml"))


def load_fixture(runner, directory, frozen) -> dict:
    """Load byte-bound host fixtures without replacing the runner's path guard."""
    if not isinstance(frozen, str) or HEX.fullmatch(frozen) is None:
        raise ValueError("Invalid frozen source fingerprint")
    directory = assert_plain_path(directory, root=runner.ROOT)
    directory = runner.private_path(directory)
    if not directory.is_dir():
        raise ValueError("Missing host fixture directory")

    def checked(path):
        return runner.private_path(assert_plain_path(path, root=runner.ROOT, regular=True), regular=True)

    metadata_path = checked(directory / "fixture.json")
    metadata_digest = runner.sha256(metadata_path)
    metadata = runner.load_json(metadata_path)
    if (not isinstance(metadata, dict) or set(metadata) != METADATA_FIELDS
            or metadata["kind"] != FIXTURE_KIND
            or metadata["source_sha256"] != frozen
            or type(metadata["source_fingerprint_version"]) is not int
            or metadata["source_fingerprint_version"] != 3
            or type(metadata["generator_exit"]) is not int or metadata["generator_exit"] != 0
            or not _generator_command(metadata["generator_command"])):
        raise ValueError("Invalid or unbound host fixture metadata")
    phases = tuple(runner.FIXTURE_PHASES)
    if len(phases) != 9 or len(set(phases)) != 9:
        raise ValueError("Unexpected runner fixture contract")
    expected = {f"{width}x{height}-{phase}.ppm": (width, height, phase)
                for width, height in ((880, 510), (1120, 660)) for phase in phases}
    hashes = metadata["files_sha256"]
    if (not isinstance(hashes, dict) or set(hashes) != set(expected)
            or any(not isinstance(value, str) or HEX.fullmatch(value) is None for value in hashes.values())):
        raise ValueError("Host fixture hash inventory differs from the exact 18 images")
    # The generator emits only these two evidence files and the exact PPM set.
    allowed = set(expected) | {"fixture.json", "generator.log"}
    children = list(directory.iterdir())
    if {p.name for p in children} - allowed or {p.name for p in children if p.suffix.lower() == ".ppm"} != set(expected):
        raise ValueError("Missing, extra or nested fixture files")
    for child in children:
        checked(child)
    frames = {}
    for name, (width, height, phase) in expected.items():
        path = checked(directory / name)
        if not 0 < path.stat().st_size <= MAX_PPM_BYTES:
            raise ValueError("Oversized or empty host oracle")
        raw = path.read_bytes()
        if len(raw) > MAX_PPM_BYTES or runner.ppm(raw)[:2] != (width, height):
            raise ValueError("Wrong host oracle dimensions")
        if runner.sha256(path) != hashes[name]:
            raise ValueError("Host oracle hash mismatch: " + name)
        # Bind the bytes handed to the native comparer, not just a second read.
        import hashlib
        if hashlib.sha256(raw).hexdigest() != hashes[name]:
            raise ValueError("Host oracle changed while loading: " + name)
        if width == 880:
            frames[phase] = raw
    if runner.sha256(checked(metadata_path)) != metadata_digest:
        raise ValueError("Host oracle metadata changed while loading")
    return {"directory": directory, "metadata": metadata, "frames": frames}


def process_image(process, expected) -> Path:
    """Query the actual child handle; never infer image identity from argv."""
    if os.name != "nt":
        raise OSError("Windows process image queries require Windows")
    expected = assert_plain_path(expected, regular=True)
    handle = getattr(process, "_handle", None)
    if handle is None or getattr(handle, "closed", False):
        raise ValueError("Missing or closed Popen process handle")
    try:
        raw_handle = int(handle)
    except (TypeError, ValueError) as error:
        raise ValueError("Invalid Popen process handle") from error
    if raw_handle <= 0:
        raise ValueError("Invalid Popen process handle")
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    query = kernel.QueryFullProcessImageNameW
    query.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.LPWSTR, ctypes.POINTER(wintypes.DWORD)]
    query.restype = wintypes.BOOL
    capacity = 32768
    buffer = ctypes.create_unicode_buffer(capacity)
    length = wintypes.DWORD(capacity)
    if not query(wintypes.HANDLE(raw_handle), 0, buffer, ctypes.byref(length)):
        raise ctypes.WinError(ctypes.get_last_error())
    if not 0 < length.value < capacity:
        raise ValueError("Invalid process image path length")
    actual = assert_plain_path(buffer.value, regular=True)
    if not os.path.samefile(actual, expected):
        raise ValueError("Actual process image differs from the selected executable")
    return actual
