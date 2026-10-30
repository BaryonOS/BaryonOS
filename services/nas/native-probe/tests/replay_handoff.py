"""Private-run artifact checks for the exact-token cold replay probe.

This is test tooling only. It never interprets bytes as a token or grants
authority; only the NAS core does that inside the guest.
"""
import hashlib
import json
import os
import re
import stat
from pathlib import Path

MANIFEST_KEYS = {
    "schema_version",
    "source_commit",
    "origin_run_id",
    "origin_role",
    "origin_stage",
    "capture_stage",
    "completed",
    "payload_length",
    "payload_sha256",
}
TOKEN_PAIR_BYTES = 80
HEX40 = re.compile(r"[0-9a-f]{40}\Z")
HEX64 = re.compile(r"[0-9a-f]{64}\Z")
RUN_ID = re.compile(r"[A-Za-z0-9._-]{1,128}\Z")


class HandoffError(ValueError):
    pass


def _private_file(path: Path, repository_root: Path, maximum: int) -> bytes:
    try:
        info = path.lstat()
    except OSError as error:
        raise HandoffError("private handoff file unavailable") from error
    if (
        stat.S_ISLNK(info.st_mode)
        or not stat.S_ISREG(info.st_mode)
        or info.st_nlink != 1
        or info.st_mode & 0o077
        or info.st_size > maximum
    ):
        raise HandoffError("private handoff file mode/type/size rejected")
    resolved = path.resolve(strict=True)
    root = repository_root.resolve(strict=True)
    if resolved == root or root in resolved.parents:
        raise HandoffError("handoff material must remain outside the repository")
    if resolved.parent.stat().st_mode & 0o077:
        raise HandoffError("handoff directory must be private")
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
    try:
        descriptor = os.open(resolved, flags)
        with os.fdopen(descriptor, "rb") as stream:
            opened = os.fstat(stream.fileno())
            if (opened.st_dev, opened.st_ino) != (info.st_dev, info.st_ino):
                raise HandoffError("handoff file changed while opening")
            data = stream.read(maximum + 1)
    except OSError as error:
        raise HandoffError("private handoff file could not be read") from error
    if len(data) != info.st_size or len(data) > maximum:
        raise HandoffError("private handoff file changed while reading")
    return data


def load_handoff(
    manifest_path: Path,
    payload_path: Path,
    repository_root: Path,
    expected_source_commit: str,
    expected_origin_run_id: str,
) -> tuple[dict, bytes]:
    manifest_bytes = _private_file(manifest_path, repository_root, 4096)
    payload = _private_file(payload_path, repository_root, TOKEN_PAIR_BYTES)
    if manifest_path.resolve().parent != payload_path.resolve().parent:
        raise HandoffError("manifest and payload must share one private run directory")
    try:
        manifest = json.loads(manifest_bytes)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise HandoffError("handoff manifest is not valid JSON") from error
    if not isinstance(manifest, dict) or set(manifest) != MANIFEST_KEYS:
        raise HandoffError("handoff manifest fields do not match the fixed schema")
    if (
        not isinstance(expected_source_commit, str)
        or HEX40.fullmatch(expected_source_commit) is None
        or not isinstance(expected_origin_run_id, str)
        or RUN_ID.fullmatch(expected_origin_run_id) is None
        or type(manifest["schema_version"]) is not int
        or manifest["schema_version"] != 1
        or not isinstance(manifest["source_commit"], str)
        or HEX40.fullmatch(manifest["source_commit"]) is None
        or manifest["source_commit"] != expected_source_commit
        or not isinstance(manifest["origin_run_id"], str)
        or RUN_ID.fullmatch(manifest["origin_run_id"]) is None
        or manifest["origin_run_id"] != expected_origin_run_id
        or manifest["origin_role"] != "guest"
        or manifest["origin_stage"] != "normal"
        or manifest["capture_stage"] != "normal-after-stage"
        or manifest["completed"] is not True
        or type(manifest["payload_length"]) is not int
        or manifest["payload_length"] != TOKEN_PAIR_BYTES
        or len(payload) != TOKEN_PAIR_BYTES
        or not isinstance(manifest["payload_sha256"], str)
        or HEX64.fullmatch(manifest["payload_sha256"]) is None
        or hashlib.sha256(payload).hexdigest() != manifest["payload_sha256"]
    ):
        raise HandoffError("handoff provenance or payload binding rejected")
    return manifest, payload


def create_cold_input(payload: bytes, path: Path, repository_root: Path) -> str:
    if len(payload) != TOKEN_PAIR_BYTES:
        raise HandoffError("cold replay input must contain exactly 80 bytes")
    root = repository_root.resolve(strict=True)
    parent = path.parent.resolve(strict=True)
    if path.exists() or path.is_symlink() or parent.stat().st_mode & 0o077:
        raise HandoffError("cold replay destination is not a private new file")
    if parent == root or root in parent.parents:
        raise HandoffError("cold replay input must remain outside the repository")
    digest = hashlib.sha256(payload).hexdigest()
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(path, flags, 0o600)
    with os.fdopen(descriptor, "wb") as stream:
        stream.write(payload)
        stream.flush()
        os.fsync(stream.fileno())
    os.chmod(path, 0o400)
    verify_cold_input(path, digest, repository_root)
    return digest


def verify_cold_input(path: Path, expected_digest: str, repository_root: Path) -> bytes:
    if HEX64.fullmatch(expected_digest) is None:
        raise HandoffError("cold input digest is malformed")
    info = path.lstat()
    if info.st_mode & 0o222:
        raise HandoffError("cold replay input must remain read-only")
    payload = _private_file(path, repository_root, TOKEN_PAIR_BYTES)
    if len(payload) != TOKEN_PAIR_BYTES or hashlib.sha256(payload).hexdigest() != expected_digest:
        raise HandoffError("cold replay input changed after origin binding")
    return payload
