#!/usr/bin/env python3
"""Bounded, local-only audit of enumerable Git and GitHub publication bytes.

The audit is deliberately split into ``snapshot``, ``scan-batch`` and
``finalize``.  No intermediate phase can report a successful audit.  A final
success is limited to the immutable manifest and never represents publication,
licence, privacy or deletion clearance.
"""
from __future__ import annotations

import argparse
import codecs
from collections import Counter
from contextlib import AbstractContextManager, contextmanager
from datetime import datetime, timezone
from email.utils import parsedate_to_datetime
import fcntl
import hashlib
import json
import math
import os
from pathlib import Path
import re
import resource
import shutil
import stat
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
import zipfile


SCHEMA_VERSION = 2
GITLEAKS_VERSION = "8.30.1"
REPOSITORY = "BaryonOS/BaryonOS"
REPOSITORY_ID = 1435549142
MAX_JSON_BYTES = 32 * 1024 * 1024
MAX_REPORT_BYTES = 64 * 1024 * 1024
MAX_RESOURCE_DOWNLOAD = 1024 * 1024 * 1024
MAX_RESOURCE_EXPANDED = 2 * 1024 * 1024 * 1024
MAX_RESOURCE_ENTRIES = 50_000
CONFIG_BYTES = b"[extend]\nuseDefault = true\n"
IGNORE_BYTES = b""
RESOURCE_KEY = re.compile(r"(?:git-history|git-objects|git-metadata|run-\d+-attempt-\d+|artifact-\d+)")
ARCHIVE_NAME = re.compile(r"\.(?:zip|tar|gz|tgz|bz2|xz|7z|rar)$", re.IGNORECASE)
RETAINED_PATHS = {
    "git-history": (
        "empty.ignore",
        "findings.json",
        "gitleaks-defaults.toml",
        "scanner.log",
    ),
    "git-metadata": (
        "empty.ignore",
        "findings.json",
        "gitleaks-defaults.toml",
        "metadata/advertised-refs.json",
        "metadata/commit-metadata.bin",
        "metadata/historical-paths.bin",
        "metadata/objects.bin",
        "scanner.log",
    ),
}
ALLOWED_STORAGE_SUFFIXES = (
    ".blob.core.windows.net",
    ".amazonaws.com",
    ".githubusercontent.com",
)

LIMIT_CEILINGS = {
    "max_total_requests": 200_000,
    "max_total_download_bytes": 2 * 1024**4,
    "max_total_expanded_bytes": 4 * 1024**4,
    "max_total_entries": 20_000_000,
    "max_total_state_bytes": 4 * 1024**4,
    "max_total_seconds": 14 * 24 * 60 * 60,
    "max_command_seconds": 24 * 60 * 60,
}
DEFAULT_LIMITS = {
    "max_total_requests": 100_000,
    "max_total_download_bytes": 512 * 1024**3,
    "max_total_expanded_bytes": 1024**4,
    "max_total_entries": 10_000_000,
    "max_total_state_bytes": 1536 * 1024**3,
    "max_total_seconds": 7 * 24 * 60 * 60,
    "max_command_seconds": 60 * 60,
}


class AuditError(Exception):
    """A redaction-safe, stable failure code."""

    def __init__(self, code: str):
        """Normalize failures to stable redaction-safe identifiers."""
        if not re.fullmatch(r"[a-z0-9_]+", code):
            code = "invalid_internal_error_code"
        self.code = code
        super().__init__(code)


def canonical_bytes(value: object) -> bytes:
    """Return the only JSON representation used for content binding."""
    try:
        return json.dumps(
            value,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=True,
            allow_nan=False,
        ).encode()
    except (TypeError, ValueError) as error:
        raise AuditError("json_value_invalid") from error


def sha256_bytes(value: bytes) -> str:
    """Return the lowercase SHA-256 digest for exact bytes."""
    return hashlib.sha256(value).hexdigest()


def object_hash(value: object) -> str:
    """Hash a value through the canonical JSON representation."""
    return sha256_bytes(canonical_bytes(value))


def file_hash(path: Path, budget: Budget | None = None) -> str:
    """Hash one regular file while honoring an optional command deadline."""
    if path.is_symlink() or not path.is_file():
        raise AuditError("bound_file_not_regular")
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            if budget is not None:
                budget.remaining()
            digest.update(block)
    return digest.hexdigest()


class StateReservation:
    """Track a bounded transient write until its exact final size is known."""

    def __init__(self, accountant: StateAccountant, maximum: int, old_size: int):
        """Create a reservation owned by one state accountant."""
        self.accountant = accountant
        self.maximum = maximum
        self.old_size = old_size
        self.active = True

    def commit(self, actual_size: int) -> None:
        """Replace the reservation with the exact retained byte delta."""
        if not self.active:
            raise AuditError("state_reservation_inactive")
        self.accountant._commit(self.maximum, self.old_size, actual_size)
        self.active = False

    def cancel_clean(self) -> None:
        """Release an unused reservation only after complete output cleanup."""
        if self.active:
            self.accountant._cancel(self.maximum)
            self.active = False


class StateAccountant:
    """Measure state once, then account exact replacement and transient deltas."""

    def __init__(self, root: Path, limit: int):
        """Bind one accountant to an absolute private-state root and limit."""
        if not isinstance(limit, int) or isinstance(limit, bool) or limit < 0:
            raise AuditError("state_limit_invalid")
        self.root = root.absolute()
        self.limit = limit
        self.used = 0
        self.reserved = 0
        self.reconcile()

    def _inside(self, path: Path) -> None:
        """Reject writes outside the accountant's bound state root."""
        try:
            path.absolute().relative_to(self.root)
        except ValueError as error:
            raise AuditError("state_write_outside_root") from error

    def reconcile(self, *, reset_reservations: bool = False) -> None:
        """Replace cached usage with one exact fail-closed tree measurement."""
        if self.reserved and not reset_reservations:
            raise AuditError("state_reservation_active")
        self.reserved = 0
        self.used = directory_size(self.root)
        if self.used > self.limit:
            raise AuditError("max_total_state_bytes_exhausted")

    def remaining(self) -> int:
        """Return unreserved retained-state capacity without walking the tree."""
        value = self.limit - self.used - self.reserved
        if value <= 0:
            raise AuditError("max_total_state_bytes_exhausted")
        return value

    def reserve_file(self, path: Path, maximum: int) -> StateReservation:
        """Reserve a file replacement including simultaneous old and new bytes."""
        self._inside(path)
        if not isinstance(maximum, int) or isinstance(maximum, bool) or maximum < 0:
            raise AuditError("state_reservation_invalid")
        if path.is_symlink() or (path.exists() and not path.is_file()):
            raise AuditError("state_file_not_regular")
        old_size = path.stat().st_size if path.exists() else 0
        if self.used + self.reserved + maximum > self.limit:
            raise AuditError("max_total_state_bytes_exhausted")
        self.reserved += maximum
        return StateReservation(self, maximum, old_size)

    def reserve_bulk(self, maximum: int) -> StateReservation:
        """Reserve bounded bytes for a new multi-file or streamed output."""
        if not isinstance(maximum, int) or isinstance(maximum, bool) or maximum < 0:
            raise AuditError("state_reservation_invalid")
        if self.used + self.reserved + maximum > self.limit:
            raise AuditError("max_total_state_bytes_exhausted")
        self.reserved += maximum
        return StateReservation(self, maximum, 0)

    def _commit(self, maximum: int, old_size: int, actual_size: int) -> None:
        """Apply a completed reservation to cached retained-byte usage."""
        if not isinstance(actual_size, int) or not 0 <= actual_size <= maximum:
            raise AuditError("state_reservation_commit_invalid")
        self.reserved -= maximum
        self.used = self.used - old_size + actual_size
        if self.reserved < 0 or self.used < 0 or self.used + self.reserved > self.limit:
            raise AuditError("state_accounting_invalid")

    def _cancel(self, maximum: int) -> None:
        """Release a reservation whose output was completely removed."""
        self.reserved -= maximum
        if self.reserved < 0:
            raise AuditError("state_accounting_invalid")

    def remove(self, path: Path) -> None:
        """Remove one retained file/tree and subtract its exact local byte size."""
        self._inside(path)
        if not path.exists() and not path.is_symlink():
            return
        if path.is_symlink():
            raise AuditError("state_symlink_present")
        if path.is_dir():
            removed = directory_size(path)
            shutil.rmtree(path)
        elif path.is_file():
            removed = path.stat().st_size
            path.unlink()
        else:
            raise AuditError("state_file_not_regular")
        self.used -= removed
        if self.used < 0:
            raise AuditError("state_accounting_invalid")


def safe_write_bytes(
    path: Path,
    value: bytes,
    *,
    accountant: StateAccountant | None = None,
) -> None:
    """Atomically write a private regular file without following symlinks."""
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    if path.is_symlink() or (path.exists() and not path.is_file()):
        raise AuditError("state_file_not_regular")
    reservation = accountant.reserve_file(path, len(value)) if accountant is not None else None
    temporary = path.with_name(path.name + ".tmp")
    replaced = False
    try:
        descriptor = os.open(
            temporary,
            os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0),
            0o600,
        )
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(value)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
        replaced = True
        if reservation is not None:
            reservation.commit(len(value))
        os.chmod(path, 0o600, follow_symlinks=False)
    except FileExistsError as error:
        raise AuditError("state_temporary_exists") from error
    finally:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass
        if reservation is not None and reservation.active and not replaced:
            reservation.cancel_clean()


def safe_write_json(
    path: Path,
    value: object,
    *,
    accountant: StateAccountant | None = None,
) -> None:
    """Encode strict JSON and write it through the private atomic writer."""
    try:
        encoded = json.dumps(value, sort_keys=True, indent=2, allow_nan=False).encode() + b"\n"
    except (TypeError, ValueError) as error:
        raise AuditError("json_value_invalid") from error
    safe_write_bytes(path, encoded, accountant=accountant)


def strict_json(raw: bytes) -> object:
    """Decode JSON while rejecting duplicate keys and non-finite numbers."""
    def pairs(items: list[tuple[str, object]]) -> dict:
        """Build one object while rejecting ambiguous duplicate keys."""
        result: dict[str, object] = {}
        for key, value in items:
            if key in result:
                raise AuditError("json_duplicate_key")
            result[key] = value
        return result

    def constant(unused: str) -> object:
        """Reject non-finite JSON constants accepted by Python by default."""
        raise AuditError("json_nonfinite_number")

    try:
        return json.loads(raw, object_pairs_hook=pairs, parse_constant=constant)
    except AuditError:
        raise
    except (UnicodeError, ValueError) as error:
        raise AuditError("json_syntax_invalid") from error


def load_json(path: Path, *, maximum: int = MAX_JSON_BYTES) -> object:
    """Load bounded strict JSON from a private regular file."""
    if path.is_symlink() or not path.is_file():
        raise AuditError("state_file_not_regular")
    if path.stat().st_size > maximum:
        raise AuditError("state_json_too_large")
    try:
        return strict_json(path.read_bytes())
    except OSError as error:
        raise AuditError("state_json_invalid") from error


def directory_size(root: Path) -> int:
    """Measure regular-file bytes under a tree while rejecting symlinks."""
    total = 0
    for parent, directories, files in os.walk(root, followlinks=False):
        for name in directories:
            path = Path(parent, name)
            if path.is_symlink():
                raise AuditError("state_symlink_present")
        for name in files:
            path = Path(parent, name)
            if path.is_symlink() or not path.is_file():
                raise AuditError("state_file_not_regular")
            total += path.stat().st_size
    return total


class StateLock(AbstractContextManager):
    """Validate the private state directory and enforce one writer."""

    def __init__(self, root: Path):
        """Bind a lock instance to one absolute private-state directory."""
        self.root = root.absolute()
        self.descriptor: int | None = None

    def __enter__(self) -> "StateLock":
        """Validate directory/lock modes and acquire a nonblocking writer lock."""
        for parent in reversed(self.root.parents):
            if parent.exists() and parent.is_symlink():
                raise AuditError("state_parent_symlink")
        if self.root.exists() and self.root.is_symlink():
            raise AuditError("state_directory_symlink")
        self.root.mkdir(parents=True, exist_ok=True, mode=0o700)
        metadata = self.root.lstat()
        if not stat.S_ISDIR(metadata.st_mode):
            raise AuditError("state_not_directory")
        if stat.S_IMODE(metadata.st_mode) != 0o700:
            raise AuditError("state_directory_permissions")
        lock = self.root / ".publication-audit.lock"
        try:
            self.descriptor = os.open(
                lock,
                os.O_RDWR | os.O_CREAT | getattr(os, "O_NOFOLLOW", 0),
                0o600,
            )
        except OSError as error:
            raise AuditError("state_lock_invalid") from error
        if stat.S_IMODE(os.fstat(self.descriptor).st_mode) != 0o600:
            os.close(self.descriptor)
            self.descriptor = None
            raise AuditError("state_lock_permissions")
        try:
            fcntl.flock(self.descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            os.close(self.descriptor)
            self.descriptor = None
            raise AuditError("state_locked") from error
        return self

    def __exit__(self, *unused: object) -> None:
        """Release the writer lock and its descriptor."""
        if self.descriptor is not None:
            fcntl.flock(self.descriptor, fcntl.LOCK_UN)
            os.close(self.descriptor)
            self.descriptor = None


def validate_limits(limits: dict[str, int]) -> None:
    """Validate aggregate limits and their cross-field relationships."""
    if set(limits) != set(LIMIT_CEILINGS):
        raise AuditError("invalid_limit_keys")
    for name, ceiling in LIMIT_CEILINGS.items():
        value = limits.get(name)
        if not isinstance(value, int) or isinstance(value, bool) or not 1 <= value <= ceiling:
            raise AuditError("invalid_limit_value")
    if limits["max_command_seconds"] > limits["max_total_seconds"]:
        raise AuditError("command_limit_exceeds_total")


class UsageLedger:
    """Persist aggregate budgets before irreversible work is attempted."""

    FIELDS = {
        "requests": "max_total_requests",
        "download_bytes": "max_total_download_bytes",
        "expanded_bytes": "max_total_expanded_bytes",
        "entries": "max_total_entries",
    }
    KEYS = {
        "schema_version",
        "limits",
        "requests",
        "download_bytes",
        "expanded_bytes",
        "entries",
        "elapsed_seconds",
        "active_started_at",
        "exhausted",
    }
    EXHAUSTION_CODES = {
        "max_total_requests_exhausted",
        "max_total_download_bytes_exhausted",
        "max_total_expanded_bytes_exhausted",
        "max_total_entries_exhausted",
        "total_time_budget_exhausted",
        "usage_clock_invalid",
    }

    def __init__(self, state: Path, limits: dict[str, int], accountant: StateAccountant | None = None):
        """Load or initialize monotonic usage under one state accountant."""
        validate_limits(limits)
        self.path = state / "usage.json"
        self.limits = dict(limits)
        self.accountant = accountant or StateAccountant(state, limits["max_total_state_bytes"])
        if self.path.exists():
            value = load_json(self.path)
            self._validate(value)
            self.value = value
        else:
            self.value = {
                "schema_version": SCHEMA_VERSION,
                "limits": self.limits,
                "requests": 0,
                "download_bytes": 0,
                "expanded_bytes": 0,
                "entries": 0,
                "elapsed_seconds": 0.0,
                "active_started_at": None,
                "exhausted": None,
            }
            self._save()

    def _validate(self, value: object) -> None:
        """Validate the exact persisted usage schema and counter ranges."""
        if not isinstance(value, dict) or set(value) != self.KEYS:
            raise AuditError("usage_schema_invalid")
        if value.get("schema_version") != SCHEMA_VERSION:
            raise AuditError("usage_schema_invalid")
        if value.get("limits") != self.limits:
            raise AuditError("usage_limit_binding_mismatch")
        for field, limit_name in self.FIELDS.items():
            counter = value.get(field)
            if (
                not isinstance(counter, int)
                or isinstance(counter, bool)
                or not 0 <= counter <= self.limits[limit_name]
            ):
                raise AuditError("usage_counter_invalid")
        elapsed = value.get("elapsed_seconds")
        if (
            not isinstance(elapsed, (int, float))
            or isinstance(elapsed, bool)
            or not math.isfinite(elapsed)
            or not 0 <= elapsed <= self.limits["max_total_seconds"]
        ):
            raise AuditError("usage_elapsed_invalid")
        active = value.get("active_started_at")
        if active is not None and (
            not isinstance(active, (int, float))
            or isinstance(active, bool)
            or not math.isfinite(active)
            or active < 0
        ):
            raise AuditError("usage_active_time_invalid")
        exhausted = value.get("exhausted")
        if exhausted is not None and exhausted not in self.EXHAUSTION_CODES:
            raise AuditError("usage_exhaustion_invalid")

    def _save(self) -> None:
        """Persist usage through O(1) replacement accounting."""
        safe_write_json(self.path, self.value, accountant=self.accountant)

    def start(self) -> None:
        """Persist a new active interval and charge a crashed predecessor."""
        now = time.time()
        previous = self.value.get("active_started_at")
        if previous is not None:
            if not isinstance(previous, (int, float)) or now < previous:
                raise AuditError("usage_clock_invalid")
            self.value["elapsed_seconds"] += now - previous
        if self.value.get("exhausted"):
            raise AuditError(str(self.value["exhausted"]))
        if self.value["elapsed_seconds"] >= self.limits["max_total_seconds"]:
            self.value["elapsed_seconds"] = self.limits["max_total_seconds"]
            self._exhaust("total_time_budget_exhausted")
        self.value["active_started_at"] = now
        self._save()

    def finish(self) -> None:
        """Close and persist the active interval, failing on total overrun."""
        started = self.value.get("active_started_at")
        now = time.time()
        if isinstance(started, (int, float)):
            if now < started:
                self._exhaust("usage_clock_invalid")
            self.value["elapsed_seconds"] += now - started
        self.value["active_started_at"] = None
        exhausted = self.value["elapsed_seconds"] > self.limits["max_total_seconds"]
        if exhausted:
            self.value["elapsed_seconds"] = self.limits["max_total_seconds"]
            self.value["exhausted"] = "total_time_budget_exhausted"
        self._save()
        if exhausted:
            raise AuditError("total_time_budget_exhausted")

    def _exhaust(self, code: str) -> None:
        """Persist a terminal budget-exhaustion code before raising it."""
        self.value["exhausted"] = code
        self._save()
        raise AuditError(code)

    def consume(self, field: str, amount: int) -> None:
        """Persistently charge an aggregate counter before bounded work."""
        if field not in self.FIELDS or not isinstance(amount, int) or amount < 0:
            raise AuditError("invalid_budget_consumption")
        limit = self.limits[self.FIELDS[field]]
        if self.value[field] + amount > limit:
            self._exhaust(self.FIELDS[field] + "_exhausted")
        self.value[field] += amount
        self._save()

    def remaining_seconds(self) -> float:
        """Return the uncharged aggregate time remaining right now."""
        elapsed = float(self.value["elapsed_seconds"])
        started = self.value.get("active_started_at")
        if isinstance(started, (int, float)):
            elapsed += max(0.0, time.time() - started)
        return max(0.0, self.limits["max_total_seconds"] - elapsed)


class Budget(AbstractContextManager):
    """One command deadline backed by the persistent aggregate ledger."""

    def __init__(self, ledger: UsageLedger):
        """Bind one command deadline to a persistent usage ledger."""
        self.ledger = ledger
        self.started = False
        self.deadline = 0.0

    def __enter__(self) -> "Budget":
        """Start the persisted command interval and derive its deadline."""
        self.ledger.start()
        self.started = True
        allowance = min(
            self.ledger.remaining_seconds(),
            float(self.ledger.limits["max_command_seconds"]),
        )
        if allowance <= 0:
            raise AuditError("total_time_budget_exhausted")
        self.deadline = time.monotonic() + allowance
        return self

    def __exit__(self, *unused: object) -> None:
        """Persist elapsed time and reject command-deadline overruns."""
        if self.started:
            late = time.monotonic() > self.deadline
            try:
                self.ledger.finish()
            finally:
                self.started = False
            if late:
                raise AuditError("command_time_budget_exhausted")

    def remaining(self) -> float:
        """Return positive command and aggregate time remaining."""
        value = min(self.deadline - time.monotonic(), self.ledger.remaining_seconds())
        if value <= 0:
            raise AuditError("command_time_budget_exhausted")
        return value

    def consume(self, field: str, amount: int) -> None:
        """Charge a persistent resource counter within the deadline."""
        self.remaining()
        self.ledger.consume(field, amount)

    def state_remaining(self) -> int:
        """Return cached unreserved state capacity without a directory walk."""
        return self.ledger.accountant.remaining()

    def reserve_file(self, path: Path, maximum: int) -> StateReservation:
        """Reserve bounded output bytes for one atomic file replacement."""
        return self.ledger.accountant.reserve_file(path, maximum)

    def reserve_bulk(self, maximum: int) -> StateReservation:
        """Reserve bounded output bytes for one multi-file operation."""
        return self.ledger.accountant.reserve_bulk(maximum)

    def remove(self, path: Path) -> None:
        """Remove retained state and update cached usage after success."""
        self.ledger.accountant.remove(path)

    def assert_state_budget(self) -> None:
        """Reject unsettled or over-limit cached state without walking it."""
        accountant = self.ledger.accountant
        if accountant.reserved or accountant.used > accountant.limit:
            raise AuditError("max_total_state_bytes_exhausted")

    def write_bytes(self, path: Path, value: bytes) -> None:
        """Write private bytes through cached transient accounting."""
        safe_write_bytes(path, value, accountant=self.ledger.accountant)

    def write_json(self, path: Path, value: object) -> None:
        """Write private JSON through cached transient accounting."""
        safe_write_json(path, value, accountant=self.ledger.accountant)


GIT_ENV_PASSTHROUGH = (
    "PATH",
    "LANG",
    "LC_ALL",
    "HTTPS_PROXY",
    "https_proxy",
    "NO_PROXY",
    "no_proxy",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
    "TMPDIR",
)
ASKPASS_TOKEN_ENV = "BARYON_AUDIT_GIT_TOKEN"
ASKPASS_BYTES = b"""#!/bin/sh
case "$1" in
  *sername*) printf '%s\\n' 'x-access-token' ;;
  *assword*) test -n "${BARYON_AUDIT_GIT_TOKEN:-}" || exit 1
              printf '%s\\n' "$BARYON_AUDIT_GIT_TOKEN" ;;
  *) exit 1 ;;
esac
"""


def isolated_git_environment(
    source: dict[str, str] | None = None,
    *,
    allow_scoped_credential: bool = False,
) -> dict[str, str]:
    """Build a minimal config-isolated Git environment with replacements off."""
    original = os.environ if source is None else source
    environment = {name: original[name] for name in GIT_ENV_PASSTHROUGH if original.get(name)}
    if allow_scoped_credential:
        for name in (ASKPASS_TOKEN_ENV, "GIT_ASKPASS", "GIT_ASKPASS_REQUIRE", "HOME", "XDG_CONFIG_HOME"):
            if original.get(name):
                environment[name] = original[name]
    environment.update(
        {
            "GIT_NO_REPLACE_OBJECTS": "1",
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_SYSTEM": os.devnull,
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_TERMINAL_PROMPT": "0",
            "GCM_INTERACTIVE": "Never",
            "GIT_OPTIONAL_LOCKS": "0",
        }
    )
    return environment


@contextmanager
def advertised_ref_environment(private: bool):
    """Yield isolated ls-remote auth; keep the private token only in process env."""
    with tempfile.TemporaryDirectory(prefix="baryon-audit-git-") as directory:
        home = Path(directory)
        os.chmod(home, 0o700)
        base = isolated_git_environment()
        base["HOME"] = str(home)
        base["XDG_CONFIG_HOME"] = str(home)
        if private:
            token = os.environ.get("GH_TOKEN", "")
            if not token:
                raise AuditError("missing_git_credential_environment")
            helper = home / "askpass"
            descriptor = os.open(
                helper,
                os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0),
                0o700,
            )
            with os.fdopen(descriptor, "wb") as stream:
                stream.write(ASKPASS_BYTES)
                stream.flush()
                os.fsync(stream.fileno())
            base[ASKPASS_TOKEN_ENV] = token
            base["GIT_ASKPASS"] = str(helper)
            base["GIT_ASKPASS_REQUIRE"] = "force"
        else:
            base["GIT_ASKPASS"] = shutil.which("false") or "/bin/false"
            base["GIT_ASKPASS_REQUIRE"] = "force"
        # ``base`` was constructed from the minimal environment above.  Keep
        # its throw-away HOME for both routes; the public route contains no
        # token and uses an askpass helper that always refuses credentials.
        yield base


def run_command(
    arguments: list[str],
    budget: Budget,
    *,
    cwd: Path | None = None,
    input_bytes: bytes | None = None,
    maximum: int = MAX_JSON_BYTES,
    environment: dict[str, str] | None = None,
    preserve_scoped_git_environment: bool = False,
) -> bytes:
    """Run a bounded command; every Git process uses the isolated environment."""
    if arguments and Path(arguments[0]).name == "git":
        environment = isolated_git_environment(
            environment,
            allow_scoped_credential=preserve_scoped_git_environment,
        )
    try:
        result = subprocess.run(
            arguments,
            cwd=cwd,
            input=input_bytes,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            env=environment,
            timeout=budget.remaining(),
            check=False,
        )
    except subprocess.TimeoutExpired as error:
        raise AuditError("local_command_timeout") from error
    if result.returncode:
        raise AuditError("local_command_failed")
    if len(result.stdout) > maximum:
        raise AuditError("local_command_output_too_large")
    return result.stdout


def run_command_to_file(
    arguments: list[str],
    budget: Budget,
    output: Path,
    *,
    cwd: Path,
    input_bytes: bytes,
    maximum: int,
) -> int:
    """Run Git into one reserved atomic output file under the shared deadline."""
    if output.is_symlink() or (output.exists() and not output.is_file()):
        raise AuditError("local_command_output_invalid")
    allowance = min(maximum, max(0, budget.state_remaining() - MAX_JSON_BYTES))
    if allowance <= 0:
        raise AuditError("max_total_state_bytes_exhausted")
    reservation = budget.reserve_file(output, allowance)
    part = output.with_name(output.name + ".part")
    if part.exists() or part.is_symlink():
        raise AuditError("local_command_output_invalid")

    def limit_output() -> None:
        """Apply the child process's hard output-file ceiling."""
        resource.setrlimit(resource.RLIMIT_FSIZE, (allowance, allowance))

    descriptor = os.open(
        part,
        os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0),
        0o600,
    )
    replaced = False
    try:
        with os.fdopen(descriptor, "wb") as stream:
            result = subprocess.run(
                arguments,
                cwd=cwd,
                input=input_bytes,
                stdout=stream,
                stderr=subprocess.DEVNULL,
                env=isolated_git_environment(),
                timeout=budget.remaining(),
                check=False,
                preexec_fn=limit_output,
            )
        if result.returncode or not part.is_file() or part.stat().st_size > maximum:
            raise AuditError("local_command_failed")
        size = part.stat().st_size
        os.replace(part, output)
        replaced = True
        reservation.commit(size)
        return size
    except subprocess.TimeoutExpired as error:
        raise AuditError("local_command_timeout") from error
    finally:
        try:
            part.unlink()
        except FileNotFoundError:
            pass
        if reservation.active and not replaced and not part.exists():
            reservation.cancel_clean()


def parse_time(value: object) -> datetime:
    """Parse one timezone-aware inventory timestamp as UTC."""
    if not isinstance(value, str):
        raise AuditError("inventory_timestamp_invalid")
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as error:
        raise AuditError("inventory_timestamp_invalid") from error
    if parsed.tzinfo is None:
        raise AuditError("inventory_timestamp_invalid")
    return parsed.astimezone(timezone.utc)


def cutoff_text(value: datetime) -> str:
    """Serialize a cutoff as canonical whole-second UTC text."""
    return value.astimezone(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


class NoRedirect(urllib.request.HTTPRedirectHandler):
    """Expose redirects to the caller so each destination can be validated."""

    def redirect_request(self, request, fp, code, message, headers, new_url):
        """Disable urllib's implicit redirect following."""
        return None


class GitHubApi:
    """Read-only GitHub client with explicit redirect and rate-limit handling."""

    def __init__(self, token: str, budget: Budget):
        """Create a read-only API client bound to one shared budget."""
        if not token:
            raise AuditError("missing_token_environment")
        self.token = token
        self.budget = budget
        self.opener = urllib.request.build_opener(NoRedirect())
        self.remaining: int | None = None

    def _api_get(self, suffix: str, repository_name: str = REPOSITORY):
        """Issue one bounded authenticated request without following redirects."""
        if (suffix and not suffix.startswith("/")) or ".." in suffix:
            raise AuditError("invalid_api_path")
        if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository_name):
            raise AuditError("invalid_repository_path")
        self.budget.consume("requests", 1)
        request = urllib.request.Request(
            "https://api.github.com/repos/" + repository_name + suffix,
            headers={
                "Authorization": "Bearer " + self.token,
                "Accept": "application/vnd.github+json",
                "X-GitHub-Api-Version": "2022-11-28",
                "User-Agent": "baryon-local-publication-audit-v2",
            },
        )
        try:
            response = self.opener.open(request, timeout=min(45, self.budget.remaining()))
        except urllib.error.HTTPError as error:
            response = error
        remaining = response.headers.get("X-RateLimit-Remaining")
        if remaining and remaining.isdecimal():
            self.remaining = int(remaining)
            if self.remaining < 10:
                response.close()
                raise AuditError("github_rate_limit_reserve_reached")
        return response

    def json(self, suffix: str, repository_name: str = REPOSITORY) -> tuple[object, object]:
        """Return bounded strict JSON and response headers from a 200 response."""
        with self._api_get(suffix, repository_name) as response:
            if response.status != 200:
                raise AuditError("api_inventory_http_error")
            raw = response.read(MAX_JSON_BYTES + 1)
            headers = response.headers
        if len(raw) > MAX_JSON_BYTES:
            raise AuditError("api_json_size_limit")
        try:
            return strict_json(raw), headers
        except AuditError as error:
            raise AuditError("api_json_invalid") from error

    def repository_alias(self, repository_name: str) -> object:
        """Resolve one GitHub rename redirect while binding the canonical ID."""
        response = self._api_get("", repository_name)
        if response.status in (301, 302, 307, 308):
            location = response.headers.get("Location", "")
            response.close()
            parsed = urllib.parse.urlsplit(location)
            allowed_paths = (
                f"/repositories/{REPOSITORY_ID}",
                "/repos/" + REPOSITORY,
            )
            if (
                parsed.scheme != "https"
                or parsed.hostname != "api.github.com"
                or parsed.username
                or parsed.password
                or parsed.query
                or parsed.fragment
                or parsed.path not in allowed_paths
            ):
                raise AuditError("repository_alias_redirect_invalid")
            self.budget.consume("requests", 1)
            request = urllib.request.Request(
                location,
                headers={
                    "Authorization": "Bearer " + self.token,
                    "Accept": "application/vnd.github+json",
                    "X-GitHub-Api-Version": "2022-11-28",
                    "User-Agent": "baryon-local-publication-audit-v2",
                },
            )
            try:
                response = self.opener.open(request, timeout=min(45, self.budget.remaining()))
            except urllib.error.HTTPError as error:
                response = error
        with response:
            if response.status != 200:
                raise AuditError("repository_alias_http_error")
            raw = response.read(MAX_JSON_BYTES + 1)
        if len(raw) > MAX_JSON_BYTES:
            raise AuditError("api_json_size_limit")
        try:
            return strict_json(raw)
        except AuditError as error:
            raise AuditError("api_json_invalid") from error

    def inventory(self, suffix: str, field: str) -> list[dict]:
        """Fetch a complete stable-count paginated inventory with unique IDs."""
        rows: list[dict] = []
        identifiers: set[int] = set()
        expected: int | None = None
        page = 1
        while True:
            data, unused = self.json(f"{suffix}?per_page=100&page={page}")
            if not isinstance(data, dict):
                raise AuditError("inventory_shape_invalid")
            count = data.get("total_count")
            batch = data.get(field)
            if not isinstance(count, int) or isinstance(count, bool) or count < 0:
                raise AuditError("inventory_total_invalid")
            if expected is None:
                expected = count
            elif count != expected:
                raise AuditError("inventory_changed_during_pagination")
            if not isinstance(batch, list) or any(not isinstance(row, dict) for row in batch):
                raise AuditError("inventory_rows_invalid")
            for row in batch:
                identifier = row.get("id")
                if (
                    not isinstance(identifier, int)
                    or isinstance(identifier, bool)
                    or identifier <= 0
                    or identifier in identifiers
                ):
                    raise AuditError("inventory_id_invalid")
                identifiers.add(identifier)
                rows.append(row)
            if len(batch) < 100 or len(rows) >= expected:
                break
            page += 1
        if len(rows) != expected:
            raise AuditError("inventory_count_mismatch")
        return rows

    def download(self, suffix: str, target: Path) -> tuple[int, int]:
        """Download one API resource through approved storage with upfront space."""
        with self._api_get(suffix) as response:
            if response.status in (404, 410):
                return response.status, 0
            if response.status != 302:
                raise AuditError("api_download_http_error")
            location = response.headers.get("Location", "")
        for unused in range(5):
            parsed = urllib.parse.urlsplit(location)
            host = (parsed.hostname or "").lower()
            if (
                parsed.scheme != "https"
                or parsed.username
                or parsed.password
                or not any(host.endswith(suffix) for suffix in ALLOWED_STORAGE_SUFFIXES)
            ):
                raise AuditError("download_origin_unapproved")
            self.budget.consume("requests", 1)
            request = urllib.request.Request(location, headers={"User-Agent": "baryon-local-publication-audit-v2"})
            try:
                response = self.opener.open(request, timeout=min(45, self.budget.remaining()))
            except urllib.error.HTTPError as error:
                response = error
            with response:
                if response.status in (301, 302, 303, 307, 308):
                    location = urllib.parse.urljoin(location, response.headers.get("Location", ""))
                    continue
                if response.status != 200:
                    raise AuditError("storage_download_http_error")
                length = response.headers.get("Content-Length")
                if length and (not length.isdecimal() or int(length) > MAX_RESOURCE_DOWNLOAD):
                    raise AuditError("resource_download_limit")
                part = target.with_name(target.name + ".part")
                if part.exists() or target.is_symlink():
                    raise AuditError("download_target_invalid")
                size = 0
                if length:
                    allowance = int(length)
                else:
                    allowance = min(
                        MAX_RESOURCE_DOWNLOAD,
                        max(0, self.budget.state_remaining() - MAX_JSON_BYTES),
                    )
                if allowance <= 0:
                    raise AuditError("max_total_state_bytes_exhausted")
                reservation = self.budget.reserve_file(target, allowance)
                descriptor = os.open(
                    part,
                    os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0),
                    0o600,
                )
                replaced = False
                try:
                    with os.fdopen(descriptor, "wb") as output:
                        while block := response.read(1024 * 1024):
                            size += len(block)
                            if size > MAX_RESOURCE_DOWNLOAD or size > allowance:
                                raise AuditError("resource_download_limit")
                            self.budget.consume("download_bytes", len(block))
                            output.write(block)
                        output.flush()
                        os.fsync(output.fileno())
                    os.replace(part, target)
                    replaced = True
                    reservation.commit(size)
                finally:
                    try:
                        part.unlink()
                    except FileNotFoundError:
                        pass
                    if reservation.active and not replaced and not part.exists():
                        reservation.cancel_clean()
                return 200, size
        raise AuditError("download_redirect_limit")


def server_cutoff(headers: object) -> datetime:
    """Derive the immutable UTC snapshot cutoff from GitHub's Date header."""
    try:
        value = headers.get("Date")
        parsed = parsedate_to_datetime(value)
    except (AttributeError, TypeError, ValueError) as error:
        raise AuditError("api_date_header_invalid") from error
    if parsed.tzinfo is None:
        raise AuditError("api_date_header_invalid")
    return parsed.astimezone(timezone.utc).replace(microsecond=0)


def verify_repository(metadata: object, *, allow_public_local: bool) -> dict:
    """Bind canonical repository identity and the authorized local route."""
    if not isinstance(metadata, dict):
        raise AuditError("repository_metadata_invalid")
    if metadata.get("id") != REPOSITORY_ID or str(metadata.get("full_name", "")).lower() != REPOSITORY.lower():
        raise AuditError("repository_identity_mismatch")
    clone_url = metadata.get("clone_url")
    expected_clone_url = "https://github.com/" + REPOSITORY + ".git"
    if clone_url != expected_clone_url:
        raise AuditError("repository_clone_url_mismatch")
    private = metadata.get("private")
    if private is True:
        visibility = "private"
    elif private is False and allow_public_local and not os.environ.get("GITHUB_ACTIONS"):
        visibility = "public-local-explicit"
    elif private is False and os.environ.get("GITHUB_ACTIONS"):
        raise AuditError("public_actions_execution_rejected")
    else:
        raise AuditError("repository_visibility_rejected")
    return {
        "id": REPOSITORY_ID,
        "full_name": REPOSITORY,
        "canonical_clone_url": expected_clone_url,
        "visibility": visibility,
    }


def at_or_before(row: dict, cutoff: datetime) -> bool:
    """Return whether an inventory row existed at the fixed cutoff."""
    return parse_time(row.get("created_at")) <= cutoff


def run_identity(row: dict) -> dict:
    """Normalize the immutable fields of one workflow run."""
    attempt = row.get("run_attempt", 1)
    if not isinstance(attempt, int) or isinstance(attempt, bool) or not 1 <= attempt <= 10_000:
        raise AuditError("run_attempt_invalid")
    if row.get("status") not in ("queued", "in_progress", "completed", "waiting", "requested", "pending"):
        raise AuditError("run_status_invalid")
    return {
        "id": row["id"],
        "run_attempt": attempt,
        "status": row.get("status"),
        "conclusion": row.get("conclusion"),
        "head_sha": row.get("head_sha"),
        "created_at": row.get("created_at"),
        "updated_at": row.get("updated_at"),
    }


def artifact_identity(row: dict) -> dict:
    """Normalize the immutable fields of one Actions artifact."""
    expired = row.get("expired")
    if not isinstance(expired, bool):
        raise AuditError("artifact_expiry_invalid")
    return {
        "id": row["id"],
        "digest": row.get("digest"),
        "expired": expired,
        "size_in_bytes": row.get("size_in_bytes"),
        "created_at": row.get("created_at"),
        "updated_at": row.get("updated_at"),
        "expires_at": row.get("expires_at"),
        "workflow_run_id": (row.get("workflow_run") or {}).get("id")
        if isinstance(row.get("workflow_run"), dict)
        else None,
    }


def normalized_inventory(runs: list[dict], artifacts: list[dict], cutoff: datetime) -> dict:
    """Filter, normalize and sort inventory rows at the fixed cutoff."""
    selected_runs = [run_identity(row) for row in runs if at_or_before(row, cutoff)]
    selected_artifacts = [artifact_identity(row) for row in artifacts if at_or_before(row, cutoff)]
    selected_runs.sort(key=lambda item: item["id"])
    selected_artifacts.sort(key=lambda item: item["id"])
    return {"runs": selected_runs, "artifacts": selected_artifacts}


def inventory_resources(inventory: dict) -> tuple[list[dict], list[str]]:
    """Expand inventory rows into deterministic scan resources and blockers."""
    resources: list[dict] = []
    blockers: list[str] = []
    for run in inventory["runs"]:
        attempts = run["run_attempt"]
        completed_attempts = attempts if run["status"] == "completed" else attempts - 1
        if run["status"] != "completed":
            blockers.append("active_run_at_cutoff")
        for attempt in range(1, completed_attempts + 1):
            identity = {**run, "kind": "run-attempt", "attempt": attempt}
            resources.append(
                {
                    "key": f"run-{run['id']}-attempt-{attempt}",
                    "endpoint": f"/actions/runs/{run['id']}/attempts/{attempt}/logs",
                    "identity": identity,
                }
            )
    for artifact in inventory["artifacts"]:
        identity = {**artifact, "kind": "artifact"}
        resources.append(
            {
                "key": f"artifact-{artifact['id']}",
                "endpoint": f"/actions/artifacts/{artifact['id']}/zip",
                "identity": identity,
            }
        )
        if artifact["expired"]:
            blockers.append("expired_artifact_at_cutoff")
    resources.sort(key=lambda item: item["key"])
    return resources, sorted(set(blockers))


def origin_repository_name(url: str) -> str:
    """Parse an exact GitHub SSH/HTTPS origin into owner/name form."""
    if re.fullmatch(r"git@github\.com:[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+(?:\.git)?", url):
        path = url.split(":", 1)[1]
    else:
        parsed = urllib.parse.urlsplit(url)
        if (
            parsed.scheme not in ("https", "ssh")
            or parsed.hostname != "github.com"
            or parsed.username not in (None, "git")
            or parsed.password
            or parsed.query
            or parsed.fragment
        ):
            raise AuditError("origin_url_invalid")
        path = parsed.path.lstrip("/")
    if path.endswith(".git"):
        path = path[:-4]
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", path):
        raise AuditError("origin_url_invalid")
    return path


def parse_advertised_refs(raw: bytes) -> list[dict]:
    """Parse and canonicalize a complete ``git ls-remote`` response."""
    refs: list[dict] = []
    for line in raw.splitlines():
        try:
            oid, name = line.split(b"\t", 1)
            oid_text = oid.decode("ascii")
            name_text = name.decode("utf-8")
        except (ValueError, UnicodeError) as error:
            raise AuditError("remote_ref_inventory_invalid") from error
        if (
            not re.fullmatch(r"[0-9a-f]{40}", oid_text)
            or not re.fullmatch(r"refs/(?:heads|tags)/.+|refs/pull/\d+/(?:head|merge)", name_text)
        ):
            raise AuditError("remote_ref_inventory_invalid")
        refs.append({"name": name_text, "oid": oid_text})
    refs.sort(key=lambda item: item["name"])
    if not refs or len({item["name"] for item in refs}) != len(refs):
        raise AuditError("remote_ref_inventory_invalid")
    return refs


def advertised_refs(canonical_url: str, budget: Budget, *, private: bool) -> list[dict]:
    """Read advertised refs with isolated public or scoped private auth."""
    budget.consume("requests", 1)
    with advertised_ref_environment(private) as environment:
        raw = run_command(
            [
                "git",
                "ls-remote",
                "--refs",
                canonical_url,
                "refs/heads/*",
                "refs/tags/*",
                "refs/pull/*/head",
                "refs/pull/*/merge",
            ],
            budget,
            cwd=Path(environment["HOME"]),
            maximum=MAX_JSON_BYTES,
            environment=environment,
            # The environment is already scrubbed by
            # ``advertised_ref_environment``.  This preserves only its
            # throw-away HOME and, for the private route, scoped askpass data.
            preserve_scoped_git_environment=True,
        )
    return parse_advertised_refs(raw)


def stable_advertised_refs(canonical_url: str, budget: Budget, *, private: bool) -> list[dict]:
    """Require two identical complete remote-ref advertisements."""
    first = advertised_refs(canonical_url, budget, private=private)
    second = advertised_refs(canonical_url, budget, private=private)
    if first != second:
        raise AuditError("remote_refs_changed_during_snapshot")
    return first


def git_snapshot(
    root: Path,
    budget: Budget,
    repository_binding: dict,
    api: GitHubApi,
    expected_head: str | None = None,
) -> dict:
    """Bind local HEAD, canonical origin and complete advertised ref tips."""
    shallow = run_command(["git", "rev-parse", "--is-shallow-repository"], budget, cwd=root).strip()
    if shallow != b"false":
        raise AuditError("shallow_git_history")
    head = run_command(["git", "rev-parse", "HEAD"], budget, cwd=root).decode().strip()
    if not re.fullmatch(r"[0-9a-f]{40}", head):
        raise AuditError("git_head_invalid")
    if expected_head and head != expected_head:
        raise AuditError("source_commit_mismatch")
    configured_url = run_command(
        ["git", "remote", "get-url", "origin"], budget, cwd=root, maximum=4096
    ).decode().strip()
    configured_name = origin_repository_name(configured_url)
    alias = api.repository_alias(configured_name)
    if (
        not isinstance(alias, dict)
        or alias.get("id") != REPOSITORY_ID
        or str(alias.get("full_name", "")).lower() != REPOSITORY.lower()
        or alias.get("clone_url") != repository_binding["canonical_clone_url"]
    ):
        raise AuditError("origin_repository_mismatch")
    refs = stable_advertised_refs(
        repository_binding["canonical_clone_url"],
        budget,
        private=repository_binding.get("visibility") == "private",
    )
    tips = sorted({item["oid"] for item in refs})
    raw_objects = run_command(
        ["git", "cat-file", "--batch-check=%(objectname) %(objecttype)"],
        budget,
        cwd=root,
        maximum=MAX_JSON_BYTES,
        input_bytes=("\n".join(tips) + "\n").encode(),
    )
    object_lines = raw_objects.splitlines()
    if len(object_lines) != len(tips):
        raise AuditError("remote_ref_tip_missing_locally")
    for expected, line in zip(tips, object_lines, strict=True):
        fields = line.decode("ascii", "strict").split()
        if len(fields) != 2 or fields[0] != expected or fields[1] == "missing":
            raise AuditError("remote_ref_tip_missing_locally")
    if head not in tips:
        raise AuditError("source_head_not_advertised")
    return {
        "head": head,
        "origin": {
            "configured_url": configured_url,
            "configured_repository": configured_name,
            "canonical_url": repository_binding["canonical_clone_url"],
            "repository_id": REPOSITORY_ID,
        },
        "refs": refs,
        "refs_sha256": object_hash(refs),
    }


def scanner_binding(binary: Path, budget: Budget) -> dict:
    """Bind the pinned scanner binary, configuration and auditor source."""
    resolved = binary.resolve(strict=True)
    version = run_command([str(resolved), "version"], budget, maximum=4096).decode().strip().lstrip("v")
    if version != GITLEAKS_VERSION:
        raise AuditError("scanner_version_mismatch")
    return {
        "name": "gitleaks",
        "version": GITLEAKS_VERSION,
        "binary_sha256": file_hash(resolved, budget),
        "config_sha256": sha256_bytes(CONFIG_BYTES),
        "ignore_sha256": sha256_bytes(IGNORE_BYTES),
        "auditor_sha256": file_hash(Path(__file__).resolve(), budget),
        "max_archive_depth": 0,
        "max_decode_depth": 5,
    }


def manifest_hash(manifest: dict) -> str:
    """Return the canonical content identifier for an immutable manifest."""
    return object_hash(manifest)


def validate_manifest_bindings(value: dict) -> None:
    """Validate repository, source and scanner trust bindings."""
    expected_repository_keys = {"id", "full_name", "canonical_clone_url", "visibility"}
    repository_value = value.get("repository")
    if (
        not isinstance(repository_value, dict)
        or set(repository_value) != expected_repository_keys
        or repository_value.get("id") != REPOSITORY_ID
        or repository_value.get("full_name") != REPOSITORY
        or repository_value.get("canonical_clone_url") != "https://github.com/" + REPOSITORY + ".git"
        or repository_value.get("visibility") not in ("private", "public-local-explicit")
    ):
        raise AuditError("manifest_repository_binding_invalid")
    source_value = value.get("source")
    if not isinstance(source_value, dict) or set(source_value) != {"head", "origin", "refs", "refs_sha256"}:
        raise AuditError("manifest_source_binding_invalid")
    refs = source_value.get("refs")
    if not isinstance(refs, list) or not refs or any(not isinstance(item, dict) for item in refs):
        raise AuditError("manifest_source_binding_invalid")
    if refs != sorted(refs, key=lambda item: item.get("name", "")):
        raise AuditError("manifest_source_binding_invalid")
    names: set[str] = set()
    tips: set[str] = set()
    for item in refs:
        if (
            not isinstance(item, dict)
            or set(item) != {"name", "oid"}
            or not isinstance(item["name"], str)
            or not re.fullmatch(r"refs/(?:heads|tags)/.+|refs/pull/\d+/(?:head|merge)", item["name"])
            or not re.fullmatch(r"[0-9a-f]{40}", str(item["oid"]))
            or item["name"] in names
        ):
            raise AuditError("manifest_source_binding_invalid")
        names.add(item["name"])
        tips.add(item["oid"])
    if source_value.get("refs_sha256") != object_hash(refs) or source_value.get("head") not in tips:
        raise AuditError("manifest_source_binding_invalid")
    origin = source_value.get("origin")
    if (
        not isinstance(origin, dict)
        or set(origin) != {"configured_url", "configured_repository", "canonical_url", "repository_id"}
        or origin.get("repository_id") != REPOSITORY_ID
        or origin.get("canonical_url") != repository_value["canonical_clone_url"]
        or origin_repository_name(str(origin.get("configured_url", ""))) != origin.get("configured_repository")
    ):
        raise AuditError("manifest_origin_binding_invalid")
    scanner = value.get("scanner")
    scanner_keys = {
        "name",
        "version",
        "binary_sha256",
        "config_sha256",
        "ignore_sha256",
        "auditor_sha256",
        "max_archive_depth",
        "max_decode_depth",
    }
    if (
        not isinstance(scanner, dict)
        or set(scanner) != scanner_keys
        or scanner.get("name") != "gitleaks"
        or scanner.get("version") != GITLEAKS_VERSION
        or any(not re.fullmatch(r"[0-9a-f]{64}", str(scanner.get(name, ""))) for name in (
            "binary_sha256", "config_sha256", "ignore_sha256", "auditor_sha256"
        ))
        or scanner.get("max_archive_depth") != 0
        or scanner.get("max_decode_depth") != 5
    ):
        raise AuditError("manifest_scanner_binding_invalid")


def validate_manifest(value: object) -> dict:
    """Validate shape, bindings and deterministic manifest reconstruction."""
    if not isinstance(value, dict) or value.get("schema_version") != SCHEMA_VERSION:
        raise AuditError("manifest_schema_invalid")
    required = {"repository", "source", "cutoff", "scanner", "limits", "inventory", "resources", "blockers"}
    if not required.issubset(value):
        raise AuditError("manifest_shape_invalid")
    validate_manifest_bindings(value)
    validate_limits(value["limits"])
    parse_time(value["cutoff"])
    resources = value["resources"]
    if not isinstance(resources, list):
        raise AuditError("manifest_resources_invalid")
    keys: set[str] = set()
    for item in resources:
        if not isinstance(item, dict) or set(item) != {"key", "endpoint", "identity"}:
            raise AuditError("manifest_resource_invalid")
        key = item["key"]
        if not isinstance(key, str) or not RESOURCE_KEY.fullmatch(key) or key in keys:
            raise AuditError("manifest_resource_invalid")
        if not isinstance(item["identity"], dict):
            raise AuditError("manifest_resource_invalid")
        keys.add(key)
    if [item["key"] for item in resources] != sorted(keys, key=resource_sort_key):
        raise AuditError("manifest_resource_order_invalid")
    try:
        rebuilt = build_manifest(
            repository=value["repository"],
            source=value["source"],
            cutoff=parse_time(value["cutoff"]),
            binding=value["scanner"],
            limits=value["limits"],
            inventory=value["inventory"],
        )
    except (KeyError, TypeError) as error:
        raise AuditError("manifest_semantics_invalid") from error
    if rebuilt != value:
        raise AuditError("manifest_semantics_invalid")
    return value


def resource_sort_key(key: str) -> tuple[int, str]:
    """Return the canonical scan ordering group for one resource key."""
    if key == "git-history":
        return (0, key)
    if key == "git-objects":
        return (1, key)
    if key == "git-metadata":
        return (2, key)
    if key.startswith("run-"):
        return (3, key)
    return (4, key)


def build_manifest(
    *,
    repository: dict,
    source: dict,
    cutoff: datetime,
    binding: dict,
    limits: dict[str, int],
    inventory: dict,
) -> dict:
    """Construct the canonical immutable snapshot manifest."""
    action_resources, blockers = inventory_resources(inventory)
    git_identity = {
        "kind": "git-history",
        "head": source["head"],
        "refs_sha256": source["refs_sha256"],
        "refs": source["refs"],
        "tips": sorted({item["oid"] for item in source["refs"]}),
    }
    resources = [
        {"key": "git-history", "endpoint": "local-git", "identity": git_identity},
        {
            "key": "git-objects",
            "endpoint": "local-git",
            "identity": {**git_identity, "kind": "git-objects"},
        },
        {
            "key": "git-metadata",
            "endpoint": "local-git",
            "identity": {**git_identity, "kind": "git-metadata"},
        },
        *action_resources,
    ]
    resources.sort(key=lambda item: resource_sort_key(item["key"]))
    return {
        "schema_version": SCHEMA_VERSION,
        "repository": repository,
        "source": source,
        "cutoff": cutoff_text(cutoff),
        "scanner": binding,
        "limits": dict(limits),
        "inventory": inventory,
        "resources": resources,
        "blockers": blockers,
        "scope": {
            "included": "Git refs and GitHub Actions runs/artifacts enumerable at the fixed cutoff",
            "excluded": [
                "resources deleted before the snapshot",
                "unreachable Git objects",
                "Git LFS payloads and submodule repositories",
                "discussions, attachments, account settings and third-party publication",
            ],
            "publication_clearance": False,
        },
    }


def private_directory(path: Path) -> None:
    """Create or validate one mode-0700 state subdirectory."""
    if path.exists():
        if path.is_symlink() or not path.is_dir():
            raise AuditError("state_directory_invalid")
    else:
        path.mkdir(parents=True, mode=0o700)
    if stat.S_IMODE(path.stat().st_mode) != 0o700:
        raise AuditError("state_subdirectory_permissions")


def archive_magic(value: bytes) -> bool:
    """Recognize supported archive signatures in a bounded prefix."""
    return value.startswith(
        (b"PK\x03\x04", b"PK\x05\x06", b"\x1f\x8b", b"BZh", b"\xfd7zXZ\x00", b"7z\xbc\xaf\x27\x1c", b"Rar!")
    ) or value[257:262] == b"ustar"


def utf8_without_nul(value: bytes) -> bool:
    """Return whether bytes are strict UTF-8 text without NUL."""
    if b"\0" in value:
        return False
    try:
        value.decode("utf-8", "strict")
        return True
    except UnicodeDecodeError:
        return False


def extract_zip(source: Path, target: Path, budget: Budget) -> dict:
    """Extract an Actions ZIP to generated names under aggregate hard limits."""
    if target.exists():
        if target.is_symlink() or not target.is_dir():
            raise AuditError("archive_target_invalid")
        budget.remove(target)
    target.mkdir(mode=0o700)
    names: list[dict] = []
    expanded = 0
    non_text_members = 0
    nested_members = 0
    encrypted_members = 0
    unsupported_members = 0
    try:
        archive = zipfile.ZipFile(source)
    except (OSError, zipfile.BadZipFile) as error:
        raise AuditError("archive_invalid") from error
    with archive:
        members = archive.infolist()
        if len(members) > MAX_RESOURCE_ENTRIES:
            raise AuditError("resource_entry_limit")
        budget.consume("entries", len(members))
        planned_expanded = 0
        for member in members:
            if member.is_dir() or member.flag_bits & 1:
                continue
            kind = stat.S_IFMT(member.external_attr >> 16)
            if kind not in (0, stat.S_IFREG, stat.S_IFLNK):
                continue
            if member.file_size < 0 or planned_expanded + member.file_size > MAX_RESOURCE_EXPANDED:
                raise AuditError("resource_expansion_limit")
            planned_expanded += member.file_size
        if planned_expanded > max(0, budget.state_remaining() - MAX_JSON_BYTES):
            raise AuditError("resource_expansion_limit")
        budget.consume("expanded_bytes", planned_expanded)
        reservation = budget.reserve_bulk(planned_expanded)
        for index, member in enumerate(members):
            names.append({"index": index, "name": member.filename})
            if member.is_dir():
                continue
            if member.flag_bits & 1:
                encrypted_members += 1
                continue
            kind = stat.S_IFMT(member.external_attr >> 16)
            if kind not in (0, stat.S_IFREG, stat.S_IFLNK):
                unsupported_members += 1
                continue
            if (
                member.file_size < 0
                or expanded + member.file_size > MAX_RESOURCE_EXPANDED
            ):
                raise AuditError("resource_expansion_limit")
            expanded += member.file_size
            destination = target / f"{index}.data"
            descriptor = os.open(
                destination,
                os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0),
                0o600,
            )
            size = 0
            has_nul = False
            decoder = codecs.getincrementaldecoder("utf-8")("strict")
            with archive.open(member) as input_stream, os.fdopen(descriptor, "wb") as output:
                while block := input_stream.read(1024 * 1024):
                    budget.remaining()
                    size += len(block)
                    if size > member.file_size:
                        raise AuditError("archive_member_size_mismatch")
                    has_nul = has_nul or b"\0" in block
                    if decoder is not None:
                        try:
                            decoder.decode(block)
                        except UnicodeDecodeError:
                            decoder = None
                    output.write(block)
            if size != member.file_size:
                raise AuditError("archive_member_size_mismatch")
            if decoder is not None:
                try:
                    decoder.decode(b"", final=True)
                except UnicodeDecodeError:
                    decoder = None
            if has_nul or decoder is None:
                non_text_members += 1
            with destination.open("rb") as child:
                prefix = child.read(512)
            if archive_magic(prefix) or ARCHIVE_NAME.search(member.filename):
                nested_members += 1
        reservation.commit(expanded)
    budget.write_json(target / "member-names.json", names)
    return {
        "entries": len(names),
        "expanded_bytes": expanded,
        "non_utf8_or_nul_members": non_text_members,
        "nested_archive_members": nested_members,
        "encrypted_members": encrypted_members,
        "unsupported_members": unsupported_members,
    }


def retained_files_binding(root: Path, expected_paths: tuple[str, ...], budget: Budget | None = None) -> dict:
    """Bind an exact expected regular-file set by relative path, size and hash."""
    if tuple(sorted(set(expected_paths))) != expected_paths or any(
        not path or Path(path).is_absolute() or ".." in Path(path).parts for path in expected_paths
    ):
        raise AuditError("retained_paths_invalid")
    actual: list[str] = []
    for parent, directories, files in os.walk(root, followlinks=False):
        for name in directories:
            path = Path(parent, name)
            if path.is_symlink():
                raise AuditError("retained_file_invalid")
        for name in files:
            path = Path(parent, name)
            if path.is_symlink() or not path.is_file():
                raise AuditError("retained_file_invalid")
            actual.append(path.relative_to(root).as_posix())
    actual.sort()
    if tuple(actual) != expected_paths:
        raise AuditError("retained_file_set_mismatch")
    files = []
    for relative in actual:
        path = root / relative
        size = path.stat().st_size
        files.append({"path": relative, "size": size, "sha256": file_hash(path, budget)})
    return {
        "files": files,
        "total_bytes": sum(item["size"] for item in files),
        "manifest_sha256": object_hash(files),
    }


def validate_retained_files_binding(value: object, expected_paths: tuple[str, ...]) -> None:
    """Validate the canonical retained-file manifest without touching state."""
    if not isinstance(value, dict) or set(value) != {"files", "total_bytes", "manifest_sha256"}:
        raise AuditError("receipt_retained_manifest_invalid")
    files = value.get("files")
    if not isinstance(files, list) or len(files) != len(expected_paths):
        raise AuditError("receipt_retained_manifest_invalid")
    paths: list[str] = []
    total = 0
    for item in files:
        if (
            not isinstance(item, dict)
            or set(item) != {"path", "size", "sha256"}
            or not isinstance(item.get("path"), str)
            or not isinstance(item.get("size"), int)
            or isinstance(item.get("size"), bool)
            or item["size"] < 0
            or not re.fullmatch(r"[0-9a-f]{64}", str(item.get("sha256", "")))
        ):
            raise AuditError("receipt_retained_manifest_invalid")
        paths.append(item["path"])
        total += item["size"]
    if (
        tuple(paths) != expected_paths
        or value.get("total_bytes") != total
        or value.get("manifest_sha256") != object_hash(files)
    ):
        raise AuditError("receipt_retained_manifest_invalid")


def compact_findings(findings: object) -> dict:
    """Return only redaction-safe counts; never return paths or matched values."""
    if not isinstance(findings, list) or any(not isinstance(item, dict) for item in findings):
        raise AuditError("scanner_report_invalid")
    rules = Counter(str(item.get("RuleID", "unknown")) for item in findings)
    return {"finding_count": len(findings), "rules": dict(sorted(rules.items()))}


def run_scanner(
    binary: Path,
    mode: str,
    source: Path,
    report: Path,
    budget: Budget,
    *,
    extra: list[str] | None = None,
) -> dict:
    """Run the pinned scanner under the shared deadline and report-file bound."""
    if mode not in ("git", "dir"):
        raise AuditError("scanner_mode_invalid")
    private_directory(report.parent)
    if report.is_symlink():
        raise AuditError("scanner_report_target_invalid")
    if report.exists():
        if not report.is_file():
            raise AuditError("scanner_report_target_invalid")
        budget.remove(report)
    config = report.parent / "gitleaks-defaults.toml"
    ignore = report.parent / "empty.ignore"
    budget.write_bytes(config, CONFIG_BYTES)
    budget.write_bytes(ignore, IGNORE_BYTES)
    log = report.parent / "scanner.log"
    if log.is_symlink():
        raise AuditError("scanner_log_target_invalid")
    if log.exists():
        if not log.is_file():
            raise AuditError("scanner_log_target_invalid")
        budget.remove(log)
    remaining = max(1, int(budget.remaining()))
    state_allowance = max(0, budget.state_remaining() - MAX_JSON_BYTES)
    per_file_limit = min(MAX_REPORT_BYTES, state_allowance // 2)
    if per_file_limit < 4096:
        raise AuditError("max_total_state_bytes_exhausted")
    report_reservation = budget.reserve_file(report, per_file_limit)
    log_reservation = budget.reserve_file(log, per_file_limit)
    arguments = [
        str(binary.resolve(strict=True)),
        mode,
        str(source),
        *(extra or []),
        "--config",
        str(config),
        "--gitleaks-ignore-path",
        str(ignore),
        "--ignore-gitleaks-allow",
        "--redact=100",
        "--no-banner",
        "--no-color",
        "--max-archive-depth=0",
        "--max-decode-depth=5",
        f"--timeout={remaining}",
        "--report-format=json",
        "--report-path",
        str(report),
    ]
    environment = isolated_git_environment()

    def limit_output() -> None:
        """Apply the scanner child's per-file output ceiling."""
        resource.setrlimit(resource.RLIMIT_FSIZE, (per_file_limit, per_file_limit))

    descriptor = os.open(
        log,
        os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0),
        0o600,
    )
    try:
        with os.fdopen(descriptor, "wb") as output:
            result = subprocess.run(
                arguments,
                env=environment,
                stdout=output,
                stderr=subprocess.STDOUT,
                timeout=budget.remaining(),
                check=False,
                preexec_fn=limit_output,
            )
    except subprocess.TimeoutExpired as error:
        raise AuditError("scanner_timeout") from error
    if not log.is_file() or log.stat().st_size > per_file_limit:
        raise AuditError("scanner_execution_failed")
    log_reservation.commit(log.stat().st_size)
    if not report.is_file() or report.stat().st_size > per_file_limit:
        raise AuditError("scanner_execution_failed")
    report_reservation.commit(report.stat().st_size)
    if result.returncode not in (0, 1):
        raise AuditError("scanner_execution_failed")
    findings = load_json(report, maximum=MAX_REPORT_BYTES)
    compact = compact_findings(findings)
    if (result.returncode == 0) != (compact["finding_count"] == 0):
        raise AuditError("scanner_report_exit_mismatch")
    return {
        "content_scanned": True,
        "scanner_exit_code": result.returncode,
        **compact,
        "report_sha256": file_hash(report, budget),
    }


def exact_tips(identity: dict) -> tuple[list[str], str]:
    """Validate, deduplicate and bound advertised Git tip arguments."""
    tips = identity.get("tips")
    if not isinstance(tips, list) or not tips or any(not re.fullmatch(r"[0-9a-f]{40}", str(item)) for item in tips):
        raise AuditError("git_tips_invalid")
    unique = sorted(set(tips))
    joined = " ".join(unique)
    if len(joined.encode()) > 1024 * 1024:
        raise AuditError("git_tip_argument_limit")
    return unique, joined


def scan_git_history(resource_item: dict, root: Path, work: Path, binary: Path, budget: Budget) -> dict:
    """Scan advertised Git history and bind every retained history work byte."""
    unused_tips, joined = exact_tips(resource_item["identity"])
    result = run_scanner(
        binary,
        "git",
        root,
        work / "findings.json",
        budget,
        extra=[f"--log-opts={joined} --full-history -m"],
    )
    return {
        **result,
        "coverage_gaps": [],
        "report_relpath": "findings.json",
        "retained_files": retained_files_binding(work, RETAINED_PATHS["git-history"], budget),
    }


def scan_git_objects(resource_item: dict, root: Path, work: Path, binary: Path, budget: Budget) -> dict:
    """Scan every reachable UTF-8 object byte and flag every other blob class."""
    tips, unused_joined = exact_tips(resource_item["identity"])
    listed = run_command(
        ["git", "rev-list", "--objects", *tips],
        budget,
        cwd=root,
        maximum=MAX_REPORT_BYTES,
    )
    object_ids = sorted({line.split(b" ", 1)[0] for line in listed.splitlines()} | {tip.encode() for tip in tips})
    info_raw = run_command(
        ["git", "cat-file", "--batch-check=%(objectname) %(objecttype) %(objectsize)"],
        budget,
        cwd=root,
        input_bytes=b"\n".join(object_ids) + b"\n",
        maximum=MAX_REPORT_BYTES,
    )
    info: list[tuple[bytes, bytes, int]] = []
    for expected, line in zip(object_ids, info_raw.splitlines(), strict=True):
        budget.remaining()
        try:
            oid, kind, raw_size = line.split()
            size = int(raw_size)
        except (ValueError, TypeError) as error:
            raise AuditError("git_object_inventory_invalid") from error
        if oid != expected or kind not in (b"blob", b"commit", b"tag", b"tree") or size < 0:
            raise AuditError("git_object_inventory_invalid")
        info.append((oid, kind, size))
    if len(info) != len(object_ids):
        raise AuditError("git_object_inventory_truncated")
    object_bytes = sum(item[2] for item in info)
    if object_bytes > MAX_RESOURCE_EXPANDED:
        raise AuditError("resource_expansion_limit")
    budget.consume("entries", len(info))
    budget.consume("expanded_bytes", object_bytes)
    maximum = object_bytes + len(info) * 100 + 1
    if maximum > MAX_RESOURCE_EXPANDED:
        raise AuditError("resource_expansion_limit")
    payload = work / "reachable-objects.batch"
    run_command_to_file(
        ["git", "cat-file", "--batch"],
        budget,
        payload,
        cwd=root,
        input_bytes=b"\n".join(object_ids) + b"\n",
        maximum=maximum,
    )
    text = work / "object-text"
    if text.exists():
        if text.is_symlink() or not text.is_dir():
            raise AuditError("git_object_text_directory_invalid")
        budget.remove(text)
    text.mkdir(mode=0o700)
    binary_blobs = 0
    binary_commits = 0
    binary_tags = 0
    with payload.open("rb") as stream:
        for index, (expected, expected_kind, expected_size) in enumerate(info):
            budget.remaining()
            header = stream.readline(256)
            try:
                oid, kind, raw_size = header.rstrip(b"\n").split()
                size = int(raw_size)
            except (ValueError, TypeError) as error:
                raise AuditError("git_object_payload_invalid") from error
            if (oid, kind, size) != (expected, expected_kind, expected_size):
                raise AuditError("git_object_payload_invalid")
            body = stream.read(size)
            if len(body) != size or stream.read(1) != b"\n":
                raise AuditError("git_object_payload_truncated")
            if kind in (b"blob", b"commit", b"tag"):
                textual = utf8_without_nul(body)
                if textual:
                    budget.write_bytes(text / f"{index}.txt", body)
                elif kind == b"blob":
                    binary_blobs += 1
                elif kind == b"tag":
                    binary_tags += 1
                else:
                    binary_commits += 1
        if stream.read(1):
            raise AuditError("git_object_payload_trailing_data")
    gaps: list[str] = []
    if binary_blobs:
        gaps.append("non_utf8_or_nul_git_blobs")
    if binary_commits:
        gaps.append("non_utf8_or_nul_commit_metadata")
    if binary_tags:
        gaps.append("non_utf8_or_nul_tag_metadata")
    result = run_scanner(binary, "dir", text, work / "findings.json", budget)
    return {
        **result,
        "coverage_gaps": gaps,
        "payload_relpath": "reachable-objects.batch",
        "payload_sha256": file_hash(payload, budget),
        "report_relpath": "findings.json",
        "object_counts": {
            "objects": len(info),
            "object_bytes": object_bytes,
            "non_utf8_or_nul_blobs": binary_blobs,
            "non_utf8_or_nul_commits": binary_commits,
            "non_utf8_or_nul_tags": binary_tags,
        },
    }


def parse_small_blobs(root: Path, object_ids: list[bytes], budget: Budget) -> int:
    """Count exact Git LFS pointer blobs through bounded batch framing."""
    lfs = 0
    for start in range(0, len(object_ids), 4000):
        batch_ids = object_ids[start : start + 4000]
        output = run_command(
            ["git", "cat-file", "--batch"],
            budget,
            cwd=root,
            input_bytes=b"\n".join(batch_ids) + b"\n",
            maximum=8 * 1024 * 1024,
        )
        offset = 0
        for expected in batch_ids:
            end = output.find(b"\n", offset)
            if end < 0:
                raise AuditError("git_blob_inventory_truncated")
            try:
                oid, kind, raw_length = output[offset:end].split()
                length = int(raw_length)
            except (ValueError, TypeError) as error:
                raise AuditError("git_blob_inventory_invalid") from error
            if oid != expected or kind != b"blob" or not 0 <= length <= 1024:
                raise AuditError("git_blob_inventory_invalid")
            offset = end + 1
            blob = output[offset : offset + length]
            if len(blob) != length or output[offset + length : offset + length + 1] != b"\n":
                raise AuditError("git_blob_inventory_truncated")
            offset += length + 1
            lfs += int(blob.startswith(b"version https://git-lfs.github.com/spec/v1\n"))
        if offset != len(output):
            raise AuditError("git_blob_inventory_trailing_data")
    return lfs


def scan_git_metadata(resource_item: dict, root: Path, work: Path, binary: Path, budget: Budget) -> dict:
    """Scan retained Git metadata, classify gaps and bind every retained byte."""
    tips, unused_joined = exact_tips(resource_item["identity"])
    metadata = work / "metadata"
    private_directory(metadata)
    refs = resource_item["identity"].get("refs")
    if not isinstance(refs, list) or object_hash(refs) != resource_item["identity"].get("refs_sha256"):
        raise AuditError("git_ref_inventory_invalid")
    budget.write_json(metadata / "advertised-refs.json", refs)
    commit_metadata = run_command(
        ["git", "log", "--format=%H%x00%an%x00%ae%x00%cn%x00%ce%x00%B%x00", *tips],
        budget,
        cwd=root,
        maximum=MAX_REPORT_BYTES,
    )
    budget.write_bytes(metadata / "commit-metadata.bin", commit_metadata)
    raw_paths = run_command(
        ["git", "log", "--format=", "--name-only", "-z", *tips],
        budget,
        cwd=root,
        maximum=MAX_REPORT_BYTES,
    )
    paths = sorted(set(path for path in raw_paths.split(b"\0") if path.strip()))
    budget.write_bytes(metadata / "historical-paths.bin", b"\n".join(paths))
    raw_modes = run_command(
        ["git", "log", "--format=", "--raw", "-z", "--no-abbrev", *tips],
        budget,
        cwd=root,
        maximum=MAX_REPORT_BYTES,
    )
    submodules = bool(re.search(rb":(?:160000 [0-7]{6}|[0-7]{6} 160000) ", raw_modes))
    objects = run_command(
        ["git", "rev-list", "--objects", *tips],
        budget,
        cwd=root,
        maximum=MAX_REPORT_BYTES,
    )
    budget.write_bytes(metadata / "objects.bin", objects)
    object_ids = [line.split(b" ", 1)[0] for line in objects.splitlines()]
    info = run_command(
        ["git", "cat-file", "--batch-check=%(objectname) %(objecttype) %(objectsize)"],
        budget,
        cwd=root,
        input_bytes=b"\n".join(object_ids) + b"\n",
        maximum=MAX_REPORT_BYTES,
    )
    small: list[bytes] = []
    for line in info.splitlines():
        try:
            oid, kind, raw_size = line.split()
            size = int(raw_size)
        except (ValueError, TypeError) as error:
            raise AuditError("git_object_inventory_invalid") from error
        if kind == b"blob" and size <= 1024:
            small.append(oid)
    lfs = parse_small_blobs(root, small, budget) if small else 0
    archives = sum(bool(ARCHIVE_NAME.search(path.decode("utf-8", "surrogateescape"))) for path in paths)
    non_utf8_paths = sum(not utf8_without_nul(path) for path in paths)
    gaps: list[str] = []
    if archives:
        gaps.append("historical_archive_payloads")
    if lfs:
        gaps.append("git_lfs_payloads")
    if submodules:
        gaps.append("submodule_repositories")
    if non_utf8_paths:
        gaps.append("non_utf8_historical_paths")
    result = run_scanner(binary, "dir", metadata, work / "findings.json", budget)
    return {
        **result,
        "coverage_gaps": gaps,
        "report_relpath": "findings.json",
        "retained_files": retained_files_binding(work, RETAINED_PATHS["git-metadata"], budget),
        "metadata_counts": {
            "historical_paths": len(paths),
            "historical_archive_paths": archives,
            "lfs_pointer_blobs": lfs,
            "submodules_seen": submodules,
            "non_utf8_paths": non_utf8_paths,
        },
    }


def receipt_base(manifest: dict, resource_item: dict) -> dict:
    """Return immutable manifest, resource and scanner receipt bindings."""
    return {
        "schema_version": SCHEMA_VERSION,
        "manifest_sha256": manifest_hash(manifest),
        "resource_key": resource_item["key"],
        "identity_sha256": object_hash(resource_item["identity"]),
        "scanner": manifest["scanner"],
    }


def scan_action_resource(
    resource_item: dict,
    work: Path,
    binary: Path,
    api: GitHubApi,
    budget: Budget,
) -> dict:
    """Download, classify and scan one Actions log or artifact resource."""
    identity = resource_item["identity"]
    if identity.get("kind") == "artifact" and identity.get("expired") is True:
        return {
            "content_scanned": False,
            "finding_count": 0,
            "rules": {},
            "coverage_gaps": ["expired_artifact_at_cutoff"],
            "unavailable_reason": "expired_at_snapshot",
        }
    payload = work / "payload.zip"
    if payload.is_symlink():
        raise AuditError("payload_target_invalid")
    if payload.exists():
        if not payload.is_file():
            raise AuditError("payload_target_invalid")
        budget.remove(payload)
    status, downloaded = api.download(resource_item["endpoint"], payload)
    if status in (404, 410):
        return {
            "content_scanned": False,
            "finding_count": 0,
            "rules": {},
            "coverage_gaps": ["resource_unavailable"],
            "unavailable_reason": f"http_{status}",
        }
    payload_digest = file_hash(payload, budget)
    expected = identity.get("digest")
    if expected and expected != "sha256:" + payload_digest:
        raise AuditError("artifact_digest_mismatch")
    unpacked = work / "unpacked"
    archive = extract_zip(payload, unpacked, budget)
    try:
        result = run_scanner(binary, "dir", unpacked, work / "findings.json", budget)
    finally:
        if unpacked.exists() and not unpacked.is_symlink():
            budget.remove(unpacked)
    gaps = []
    if archive["non_utf8_or_nul_members"]:
        gaps.append("non_utf8_or_nul_archive_members")
    if archive["nested_archive_members"]:
        gaps.append("nested_archive_members")
    if archive["encrypted_members"]:
        gaps.append("encrypted_archive_members")
    if archive["unsupported_members"]:
        gaps.append("unsupported_archive_members")
    return {
        **result,
        "coverage_gaps": gaps,
        "payload_sha256": payload_digest,
        "payload_relpath": "payload.zip",
        "report_relpath": "findings.json",
        "download_bytes": downloaded,
        "archive": archive,
    }


def receipt_path(state: Path, key: str) -> Path:
    """Return the private canonical receipt path for one resource key."""
    if not RESOURCE_KEY.fullmatch(key):
        raise AuditError("resource_key_invalid")
    return state / "receipts" / f"{key}.json"


def resource_work(state: Path, key: str) -> Path:
    """Return the private canonical work directory for one resource key."""
    if not RESOURCE_KEY.fullmatch(key):
        raise AuditError("resource_key_invalid")
    base = state / "work"
    private_directory(base)
    work = base / key
    private_directory(work)
    return work


def write_receipt(state: Path, key: str, value: dict, budget: Budget | None = None) -> None:
    """Persist one strict private receipt through bounded accounting."""
    directory = state / "receipts"
    private_directory(directory)
    if budget is None:
        safe_write_json(receipt_path(state, key), value)
    else:
        budget.write_json(receipt_path(state, key), value)


def _nonnegative_integer(value: object) -> bool:
    """Recognize non-Boolean nonnegative integer counters."""
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def expected_receipt_gaps(resource_item: dict, value: dict) -> tuple[list[str], set[str]]:
    """Derive exact coverage gaps and type-specific receipt keys."""
    key = resource_item["key"]
    if not value["content_scanned"]:
        if value.get("finding_count") != 0 or value.get("rules") != {}:
            raise AuditError("receipt_unavailable_findings_invalid")
        if key.startswith("git-"):
            raise AuditError("receipt_unavailable_reason_invalid")
        reason = value.get("unavailable_reason")
        if reason == "expired_at_snapshot":
            return ["expired_artifact_at_cutoff"], {"unavailable_reason"}
        if reason in ("http_404", "http_410"):
            return ["resource_unavailable"], {"unavailable_reason"}
        raise AuditError("receipt_unavailable_reason_invalid")
    if resource_item["identity"].get("kind") == "artifact" and resource_item["identity"].get("expired") is True:
        raise AuditError("receipt_expired_artifact_scanned")
    if key == "git-history":
        return [], {"retained_files"}
    if key == "git-objects":
        counts = value.get("object_counts")
        expected_keys = {
            "objects",
            "object_bytes",
            "non_utf8_or_nul_blobs",
            "non_utf8_or_nul_commits",
            "non_utf8_or_nul_tags",
        }
        if (
            not isinstance(counts, dict)
            or set(counts) != expected_keys
            or any(not _nonnegative_integer(counts[name]) for name in expected_keys)
            or sum(counts[name] for name in (
                "non_utf8_or_nul_blobs", "non_utf8_or_nul_commits", "non_utf8_or_nul_tags"
            )) > counts["objects"]
        ):
            raise AuditError("receipt_object_counts_invalid")
        gaps: list[str] = []
        if counts["non_utf8_or_nul_blobs"]:
            gaps.append("non_utf8_or_nul_git_blobs")
        if counts["non_utf8_or_nul_commits"]:
            gaps.append("non_utf8_or_nul_commit_metadata")
        if counts["non_utf8_or_nul_tags"]:
            gaps.append("non_utf8_or_nul_tag_metadata")
        return gaps, {"payload_relpath", "payload_sha256", "object_counts"}
    if key == "git-metadata":
        counts = value.get("metadata_counts")
        expected_keys = {
            "historical_paths",
            "historical_archive_paths",
            "lfs_pointer_blobs",
            "submodules_seen",
            "non_utf8_paths",
        }
        if not isinstance(counts, dict) or set(counts) != expected_keys:
            raise AuditError("receipt_metadata_counts_invalid")
        for name in expected_keys - {"submodules_seen"}:
            if not _nonnegative_integer(counts[name]):
                raise AuditError("receipt_metadata_counts_invalid")
        if not isinstance(counts["submodules_seen"], bool):
            raise AuditError("receipt_metadata_counts_invalid")
        if (
            counts["historical_archive_paths"] > counts["historical_paths"]
            or counts["non_utf8_paths"] > counts["historical_paths"]
        ):
            raise AuditError("receipt_metadata_counts_invalid")
        gaps = []
        if counts["historical_archive_paths"]:
            gaps.append("historical_archive_payloads")
        if counts["lfs_pointer_blobs"]:
            gaps.append("git_lfs_payloads")
        if counts["submodules_seen"]:
            gaps.append("submodule_repositories")
        if counts["non_utf8_paths"]:
            gaps.append("non_utf8_historical_paths")
        return gaps, {"metadata_counts", "retained_files"}
    archive = value.get("archive")
    archive_keys = {
        "entries",
        "expanded_bytes",
        "non_utf8_or_nul_members",
        "nested_archive_members",
        "encrypted_members",
        "unsupported_members",
    }
    if (
        not isinstance(archive, dict)
        or set(archive) != archive_keys
        or any(not _nonnegative_integer(archive[name]) for name in archive_keys)
        or not _nonnegative_integer(value.get("download_bytes"))
        or any(
            archive[name] > archive["entries"]
            for name in (
                "non_utf8_or_nul_members", "nested_archive_members", "encrypted_members", "unsupported_members"
            )
        )
    ):
        raise AuditError("receipt_archive_counts_invalid")
    gaps = []
    if archive["non_utf8_or_nul_members"]:
        gaps.append("non_utf8_or_nul_archive_members")
    if archive["nested_archive_members"]:
        gaps.append("nested_archive_members")
    if archive["encrypted_members"]:
        gaps.append("encrypted_archive_members")
    if archive["unsupported_members"]:
        gaps.append("unsupported_archive_members")
    return gaps, {"payload_relpath", "payload_sha256", "download_bytes", "archive"}


def validate_receipt(
    manifest: dict,
    resource_item: dict,
    state: Path,
    budget: Budget | None = None,
) -> dict:
    """Revalidate one receipt and every retained byte it claims."""
    path = receipt_path(state, resource_item["key"])
    if not path.exists():
        raise AuditError("receipt_missing")
    value = load_json(path)
    if not isinstance(value, dict):
        raise AuditError("receipt_invalid")
    base = receipt_base(manifest, resource_item)
    if any(value.get(name) != expected for name, expected in base.items()):
        raise AuditError("receipt_binding_mismatch")
    if not isinstance(value.get("content_scanned"), bool):
        raise AuditError("receipt_invalid")
    if not isinstance(value.get("finding_count"), int) or value["finding_count"] < 0:
        raise AuditError("receipt_invalid")
    if not isinstance(value.get("rules"), dict) or not isinstance(value.get("coverage_gaps"), list):
        raise AuditError("receipt_invalid")
    expected_gaps, extra_keys = expected_receipt_gaps(resource_item, value)
    if value["coverage_gaps"] != expected_gaps:
        raise AuditError("receipt_coverage_mismatch")
    base_keys = set(receipt_base(manifest, resource_item))
    common_keys = {"content_scanned", "finding_count", "rules", "coverage_gaps"}
    if value["content_scanned"]:
        common_keys |= {"scanner_exit_code", "report_relpath", "report_sha256"}
        expected_exit = 0 if value["finding_count"] == 0 else 1
        if value.get("scanner_exit_code") != expected_exit:
            raise AuditError("receipt_scanner_exit_invalid")
    if set(value) != base_keys | common_keys | extra_keys:
        raise AuditError("receipt_schema_invalid")
    if value["content_scanned"]:
        work = resource_work(state, resource_item["key"])
        report = work / "findings.json"
        if value.get("report_relpath") != "findings.json" or value.get("report_sha256") != file_hash(report, budget):
            raise AuditError("receipt_report_binding_mismatch")
        findings = load_json(report, maximum=MAX_REPORT_BYTES)
        compact = compact_findings(findings)
        if compact["finding_count"] != value["finding_count"] or compact["rules"] != value["rules"]:
            raise AuditError("receipt_report_content_mismatch")
        if resource_item["key"].startswith(("run-", "artifact-")):
            payload = work / "payload.zip"
            if (
                value.get("payload_relpath") != "payload.zip"
                or value.get("payload_sha256") != file_hash(payload, budget)
            ):
                raise AuditError("receipt_payload_binding_mismatch")
        elif resource_item["key"] == "git-objects":
            payload = work / "reachable-objects.batch"
            if (
                value.get("payload_relpath") != "reachable-objects.batch"
                or value.get("payload_sha256") != file_hash(payload, budget)
            ):
                raise AuditError("receipt_payload_binding_mismatch")
        retained_paths = RETAINED_PATHS.get(resource_item["key"])
        if retained_paths is not None:
            validate_retained_files_binding(value.get("retained_files"), retained_paths)
            try:
                actual_retained = retained_files_binding(work, retained_paths, budget)
            except AuditError as error:
                raise AuditError("receipt_retained_bytes_mismatch") from error
            if actual_retained != value["retained_files"]:
                raise AuditError("receipt_retained_bytes_mismatch")
    elif not isinstance(value.get("unavailable_reason"), str):
        raise AuditError("receipt_invalid")
    return value


def existing_receipts(
    manifest: dict,
    state: Path,
    budget: Budget | None = None,
) -> dict[str, dict]:
    """Load and fully validate every receipt currently present."""
    values: dict[str, dict] = {}
    for resource_item in manifest["resources"]:
        path = receipt_path(state, resource_item["key"])
        if path.exists():
            if budget is not None:
                budget.remaining()
            values[resource_item["key"]] = validate_receipt(manifest, resource_item, state, budget)
    return values


def coverage(manifest: dict, state: Path, budget: Budget | None = None) -> dict:
    """Summarize validated receipt coverage and finalizability."""
    receipts = existing_receipts(manifest, state, budget)
    missing = len(manifest["resources"]) - len(receipts)
    unavailable = sum(not receipt["content_scanned"] for receipt in receipts.values())
    findings = sum(receipt["finding_count"] for receipt in receipts.values())
    gaps = sum(len(receipt["coverage_gaps"]) for receipt in receipts.values())
    return {
        "resources": len(manifest["resources"]),
        "receipts": len(receipts),
        "missing_receipts": missing,
        "unavailable_resources": unavailable,
        "finding_count": findings,
        "coverage_gap_count": gaps,
        "manifest_blockers": len(manifest["blockers"]),
        "finalizable": not any((missing, unavailable, findings, gaps, len(manifest["blockers"]))),
    }


def assert_state_budget(budget: Budget) -> None:
    """Validate cached state usage without rescanning the complete tree."""
    budget.assert_state_budget()


def fetch_inventory(api: GitHubApi, cutoff: datetime) -> dict:
    """Fetch and normalize one complete Actions inventory pass."""
    runs = api.inventory("/actions/runs", "workflow_runs")
    artifacts = api.inventory("/actions/artifacts", "artifacts")
    return normalized_inventory(runs, artifacts, cutoff)


def stable_inventory(api: GitHubApi, cutoff: datetime) -> dict:
    """Require two identical complete Actions inventory passes."""
    first = fetch_inventory(api, cutoff)
    second = fetch_inventory(api, cutoff)
    if first != second:
        raise AuditError("actions_snapshot_changed")
    return first


def read_manifest(state: Path) -> dict:
    """Load and fully validate the immutable state manifest."""
    return validate_manifest(load_json(state / "manifest.json"))


def phase_snapshot(
    state: Path,
    root: Path,
    binary: Path,
    budget: Budget,
    limits: dict[str, int],
    *,
    allow_public_local: bool,
    expected_head: str | None,
) -> dict:
    """Create and persist one stable immutable publication snapshot."""
    if (state / "manifest.json").exists():
        raise AuditError("state_already_initialized")
    api = GitHubApi(os.environ.get("GH_TOKEN", ""), budget)
    metadata, headers = api.json("")
    repository = verify_repository(metadata, allow_public_local=allow_public_local)
    source = git_snapshot(root, budget, repository, api, expected_head)
    binding = scanner_binding(binary, budget)
    cutoff = server_cutoff(headers)
    inventory = stable_inventory(api, cutoff)
    manifest = build_manifest(
        repository=repository,
        source=source,
        cutoff=cutoff,
        binding=binding,
        limits=limits,
        inventory=inventory,
    )
    validate_manifest(manifest)
    budget.write_json(state / "manifest.json", manifest)
    assert_state_budget(budget)
    return {
        "phase": "snapshot",
        "phase_complete": True,
        "audit_complete": False,
        "snapshot_audit_passed": False,
        "publication_clearance": False,
        "manifest_sha256": manifest_hash(manifest),
        "resources": len(manifest["resources"]),
        "blockers": len(manifest["blockers"]),
        "cutoff": manifest["cutoff"],
    }


def phase_scan_batch(
    state: Path,
    root: Path,
    binary: Path,
    budget: Budget,
    *,
    batch_size: int,
) -> dict:
    """Scan and bind at most one deterministic bounded resource batch."""
    if not 1 <= batch_size <= 100:
        raise AuditError("batch_size_invalid")
    manifest = read_manifest(state)
    if file_hash(Path(__file__).resolve(), budget) != manifest["scanner"].get("auditor_sha256"):
        raise AuditError("auditor_binding_mismatch")
    if scanner_binding(binary, budget) != manifest["scanner"]:
        raise AuditError("scanner_binding_mismatch")
    api = GitHubApi(os.environ.get("GH_TOKEN", ""), budget)
    metadata, unused_headers = api.json("")
    allow_public = manifest["repository"].get("visibility") == "public-local-explicit"
    if verify_repository(metadata, allow_public_local=allow_public) != manifest["repository"]:
        raise AuditError("repository_binding_changed")
    if git_snapshot(root, budget, manifest["repository"], api, manifest["source"]["head"]) != manifest["source"]:
        raise AuditError("git_snapshot_changed")
    receipts = existing_receipts(manifest, state, budget)
    selected = [item for item in manifest["resources"] if item["key"] not in receipts][:batch_size]
    for item in selected:
        work = resource_work(state, item["key"])
        budget.remove(work)
        private_directory(work)
        if item["key"] == "git-history":
            result = scan_git_history(item, root, work, binary, budget)
        elif item["key"] == "git-objects":
            result = scan_git_objects(item, root, work, binary, budget)
        elif item["key"] == "git-metadata":
            result = scan_git_metadata(item, root, work, binary, budget)
        else:
            result = scan_action_resource(item, work, binary, api, budget)
        write_receipt(state, item["key"], {**receipt_base(manifest, item), **result}, budget)
        validate_receipt(manifest, item, state, budget)
        assert_state_budget(budget)
    current = coverage(manifest, state, budget)
    return {
        "phase": "scan-batch",
        "phase_complete": True,
        "audit_complete": False,
        "snapshot_audit_passed": False,
        "publication_clearance": False,
        "manifest_sha256": manifest_hash(manifest),
        "processed_this_batch": len(selected),
        **current,
    }


def phase_finalize(state: Path, root: Path, budget: Budget) -> tuple[dict, bool]:
    """Revalidate the complete snapshot and return the only success candidate."""
    manifest = read_manifest(state)
    if file_hash(Path(__file__).resolve(), budget) != manifest["scanner"].get("auditor_sha256"):
        raise AuditError("auditor_binding_mismatch")
    api = GitHubApi(os.environ.get("GH_TOKEN", ""), budget)
    metadata, unused_headers = api.json("")
    allow_public = manifest["repository"].get("visibility") == "public-local-explicit"
    if verify_repository(metadata, allow_public_local=allow_public) != manifest["repository"]:
        raise AuditError("repository_binding_changed")
    cutoff = parse_time(manifest["cutoff"])
    inventory = stable_inventory(api, cutoff)
    if inventory != manifest["inventory"]:
        raise AuditError("actions_inventory_changed_since_snapshot")
    if git_snapshot(root, budget, manifest["repository"], api, manifest["source"]["head"]) != manifest["source"]:
        raise AuditError("git_snapshot_changed")
    current = coverage(manifest, state, budget)
    passed = current["finalizable"]
    final = {
        "schema_version": SCHEMA_VERSION,
        "phase": "finalize",
        "phase_complete": True,
        "audit_complete": passed,
        "snapshot_audit_passed": passed,
        "publication_clearance": False,
        "manifest_sha256": manifest_hash(manifest),
        "inventory_sha256": object_hash(inventory),
        "source_commit": manifest["source"]["head"],
        "source_refs_sha256": manifest["source"]["refs_sha256"],
        "cutoff": manifest["cutoff"],
        "scope_limitations": manifest["scope"]["excluded"],
        **current,
    }
    assert_state_budget(budget)
    return final, passed


def invalidate_final_record(state: Path) -> None:
    """Remove any earlier final record before a new finalize attempt."""
    path = state / "final.json"
    if path.is_symlink():
        path.unlink()
        raise AuditError("state_symlink_present")
    if not path.exists():
        return
    if not path.is_file():
        raise AuditError("state_file_not_regular")
    path.unlink()


def add_limit_arguments(parser: argparse.ArgumentParser) -> None:
    """Add every bounded aggregate-limit option to the snapshot parser."""
    for name, default in DEFAULT_LIMITS.items():
        parser.add_argument("--" + name.replace("_", "-"), type=int, default=default)


def limits_from_args(args: argparse.Namespace) -> dict[str, int]:
    """Extract and validate aggregate limits from parsed snapshot arguments."""
    limits = {name: getattr(args, name) for name in DEFAULT_LIMITS}
    validate_limits(limits)
    return limits


def parser() -> argparse.ArgumentParser:
    """Build the three-phase command-line interface."""
    value = argparse.ArgumentParser(description=__doc__)
    value.add_argument("--state-dir", type=Path, required=True)
    value.add_argument("--repo", type=Path, default=Path("."))
    commands = value.add_subparsers(dest="phase", required=True)

    snapshot = commands.add_parser("snapshot", help="Create a stable, immutable publication manifest")
    snapshot.add_argument("--gitleaks", type=Path, required=True)
    snapshot.add_argument("--source-commit")
    snapshot.add_argument("--allow-public-local", action="store_true")
    add_limit_arguments(snapshot)

    batch = commands.add_parser("scan-batch", help="Scan a deterministic bounded manifest subset")
    batch.add_argument("--gitleaks", type=Path, required=True)
    batch.add_argument("--batch-size", type=int, default=20)

    commands.add_parser("finalize", help="Revalidate the snapshot and require every bound receipt")
    return value


def failure(phase: str, code: str) -> dict:
    """Return a redaction-safe terminal failure record with no success claim."""
    return {
        "phase": phase,
        "phase_complete": False,
        "audit_complete": False,
        "snapshot_audit_passed": False,
        "publication_clearance": False,
        "error": code,
    }


def reject_repository_state_overlap(root: Path, state: Path) -> None:
    """Reject state inside the repository and repositories inside state."""
    resolved_root = root.resolve(strict=True)
    if not (resolved_root / ".git").exists():
        raise AuditError("repository_root_required")
    resolved_state = state.resolve(strict=False)
    if (
        resolved_state == resolved_root
        or resolved_root in resolved_state.parents
        or resolved_state in resolved_root.parents
    ):
        raise AuditError("state_repository_overlap")


def main(arguments: list[str] | None = None) -> int:
    """Run one bounded audit phase and print only a redaction-safe result."""
    os.umask(0o077)
    args = parser().parse_args(arguments)
    root = args.repo.resolve()
    state = args.state_dir.absolute()
    try:
        reject_repository_state_overlap(root, state)
        with StateLock(state):
            if args.phase == "finalize":
                invalidate_final_record(state)
            if args.phase == "snapshot":
                limits = limits_from_args(args)
            else:
                manifest = read_manifest(state)
                limits = manifest["limits"]
            ledger = UsageLedger(state, limits)
            with Budget(ledger) as budget:
                if args.phase == "snapshot":
                    result = phase_snapshot(
                        state,
                        root,
                        args.gitleaks,
                        budget,
                        limits,
                        allow_public_local=args.allow_public_local,
                        expected_head=args.source_commit,
                    )
                    exit_code = 0
                elif args.phase == "scan-batch":
                    result = phase_scan_batch(
                        state,
                        root,
                        args.gitleaks,
                        budget,
                        batch_size=args.batch_size,
                    )
                    exit_code = 0
                else:
                    result, passed = phase_finalize(state, root, budget)
                    exit_code = 0 if passed else 1
            if args.phase == "finalize":
                safe_write_json(state / "final.json", result, accountant=ledger.accountant)
    except AuditError as error:
        result = failure(args.phase, error.code)
        exit_code = 1
    except (OSError, subprocess.SubprocessError, zipfile.BadZipFile):
        result = failure(args.phase, "bounded_operation_failed")
        exit_code = 1
    except Exception:
        # Never expose exception text: it may contain signed URLs or findings.
        result = failure(args.phase, "unexpected_exception")
        exit_code = 1
    print(canonical_bytes(result).decode(), flush=True)
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
