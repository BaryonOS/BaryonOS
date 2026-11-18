"""Fail-closed execution in an explicitly provisioned, offline Linux runtime."""
from __future__ import annotations

from contextlib import contextmanager
import errno
import hashlib
from itertools import chain
import json
import os
from pathlib import Path
import re
import resource
import signal
import stat
import subprocess
import sys
import tempfile
import time

from common import isolated_environment


class SandboxError(RuntimeError):
    pass


class SandboxResourceExceeded(SandboxError):
    pass


_LAUNCH_ENV = {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8"}
_MOUNTPOINTS = ("dev", "proc", "sys", "tmp", "run", "home", "root", "workspace")
_MAX_TIMEOUT = 7200
_EXEC_DESCRIPTOR = (
    "import os,sys; fd=int(sys.argv[1]); os.set_inheritable(fd,False); "
    "os.execve(fd,sys.argv[2:],{'PATH':'/usr/bin:/bin','LANG':'C.UTF-8'})"
)
_RESOURCE_GUARD = r"""
import os, resource, sys
from pathlib import Path

def fail(message):
    print("resource guard: " + message, file=sys.stderr)
    raise SystemExit(125)

try:
    descriptor = int(sys.argv[1])
    scope = sys.argv[2]
    memory = int(sys.argv[3])
    tasks = int(sys.argv[4])
    cpu_percent = int(sys.argv[5])
    file_limit = int(sys.argv[6])
    open_files = int(sys.argv[7])
    lines = [line.split("::", 1)[1] for line in Path("/proc/self/cgroup").read_text().splitlines()
             if line.startswith("0::")]
    if len(lines) != 1:
        fail("unified cgroup v2 membership is unavailable")
    cgroup = Path("/sys/fs/cgroup") / lines[0].lstrip("/")
    if cgroup.name != scope:
        fail("process is outside its owner-selected resource scope")
    if (cgroup / "memory.max").read_text().strip() != str(memory):
        fail("memory.max does not match policy")
    if (cgroup / "memory.swap.max").read_text().strip() != "0":
        fail("memory.swap.max does not match policy")
    if (cgroup / "pids.max").read_text().strip() != str(tasks):
        fail("pids.max does not match policy")
    quota, period = (cgroup / "cpu.max").read_text().split()
    if quota == "max" or int(quota) * 100 != cpu_percent * int(period):
        fail("cpu.max does not match policy")
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    resource.setrlimit(resource.RLIMIT_FSIZE, (file_limit, file_limit))
    resource.setrlimit(resource.RLIMIT_NOFILE, (open_files, open_files))
    os.set_inheritable(descriptor, False)
    os.execve(descriptor, ["bwrap", *sys.argv[8:]], {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8"})
except (OSError, ValueError) as error:
    fail(str(error))
"""


def _descriptor_command(descriptor: int, name: str, arguments: list[str]) -> list[str]:
    # Close the checked host descriptor on exec, before the selected program starts.
    return ["/usr/bin/python3", "-I", "-S", "-B", "-c", _EXEC_DESCRIPTOR,
            str(descriptor), name, *arguments]


def _absolute_path(value: object, label: str) -> Path:
    if not isinstance(value, (str, Path)) or not str(value) or "\0" in str(value):
        raise SandboxError(f"{label} requires an absolute existing path")
    path = Path(value)
    if not path.is_absolute() or ".." in path.parts:
        raise SandboxError(f"{label} requires an absolute canonical path")
    try:
        for component in (*reversed(path.parents), path):
            if component.is_symlink():
                raise SandboxError(f"{label} has a symlink component")
        path.stat()
    except OSError as exc:
        raise SandboxError(f"{label} is unavailable") from exc
    return path


def _unprivileged(path: Path | int, metadata: os.stat_result) -> None:
    if metadata.st_mode & (stat.S_ISUID | stat.S_ISGID):
        raise SandboxError("setuid/setgid runtime files are forbidden")
    try:
        capabilities = os.getxattr(path, "security.capability")
    except OSError as exc:
        if exc.errno not in (errno.ENODATA, errno.ENOTSUP):
            raise SandboxError("cannot verify runtime file capabilities") from exc
    else:
        if capabilities:
            raise SandboxError("runtime file capabilities are forbidden")


@contextmanager
def _verified_executable(path: Path, expected: str, label: str):
    descriptor = None
    try:
        descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK)
        metadata = os.fstat(descriptor)
        if (not stat.S_ISREG(metadata.st_mode) or not metadata.st_mode & 0o111
                or metadata.st_mode & 0o022 or metadata.st_nlink != 1):
            raise SandboxError(f"{label} must be a regular, unshared executable")
        _unprivileged(descriptor, metadata)
        with os.fdopen(os.dup(descriptor), "rb") as source:
            actual = hashlib.file_digest(source, "sha256").hexdigest()
        if actual != expected:
            raise SandboxError(f"{label} SHA256 does not match the configured pin")
        yield descriptor
    except OSError as exc:
        raise SandboxError(f"{label} verification or startup failed") from exc
    finally:
        if descriptor is not None:
            os.close(descriptor)


def _resource_environment() -> dict[str, str]:
    runtime = Path(f"/run/user/{os.getuid()}")
    try:
        metadata = runtime.lstat()
        bus = (runtime / "bus").lstat()
    except OSError as exc:
        raise SandboxError("the private user resource manager is unavailable") from exc
    if (runtime.is_symlink() or not stat.S_ISDIR(metadata.st_mode) or metadata.st_uid != os.getuid()
            or stat.S_IMODE(metadata.st_mode) != 0o700 or not stat.S_ISSOCK(bus.st_mode)
            or bus.st_uid != os.getuid()):
        raise SandboxError("the private user resource manager is unsafe")
    return {**_LAUNCH_ENV, "XDG_RUNTIME_DIR": str(runtime),
            "DBUS_SESSION_BUS_ADDRESS": f"unix:path={runtime / 'bus'}"}


def _mount_field(value: str) -> str:
    return re.sub(r"\\([0-7]{3})", lambda match: chr(int(match.group(1), 8)), value)


def _require_private_tmpfs(path: Path) -> None:
    metadata = path.lstat()
    if (path.is_symlink() or not stat.S_ISDIR(metadata.st_mode) or metadata.st_uid != os.getuid()
            or stat.S_IMODE(metadata.st_mode) != 0o700):
        raise SandboxError("scratch root must be a private controller directory")
    candidates = []
    try:
        for line in Path("/proc/self/mountinfo").read_text().splitlines():
            fields = line.split()
            separator = fields.index("-")
            mountpoint = Path(_mount_field(fields[4]))
            if path == mountpoint or path.is_relative_to(mountpoint):
                candidates.append((len(mountpoint.parts), fields[separator + 1]))
    except (OSError, ValueError) as exc:
        raise SandboxError("cannot verify the scratch filesystem") from exc
    if not candidates or max(candidates)[1] != "tmpfs":
        raise SandboxError("scratch root must reside on tmpfs for aggregate write bounds")


def _workspace_usage(root: Path, byte_limit: int, entry_limit: int) -> tuple[int, int]:
    total = 0
    entries = 0
    pending = [root]
    try:
        while pending:
            directory = pending.pop()
            with os.scandir(directory) as iterator:
                for item in iterator:
                    try:
                        metadata = item.stat(follow_symlinks=False)
                    except FileNotFoundError:
                        continue
                    entries += 1
                    if entries > entry_limit:
                        raise SandboxResourceExceeded("workspace entry limit exceeded")
                    if stat.S_ISDIR(metadata.st_mode):
                        pending.append(Path(item.path))
                    elif stat.S_ISREG(metadata.st_mode):
                        total += metadata.st_size
                        if total > byte_limit:
                            raise SandboxResourceExceeded("workspace byte limit exceeded")
                    else:
                        raise SandboxResourceExceeded("workspace gained a link or special file")
    except SandboxResourceExceeded:
        raise
    except OSError as exc:
        raise SandboxResourceExceeded("workspace changed during resource accounting") from exc
    return total, entries


def _stop_process(process: subprocess.Popen) -> None:
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired as exc:
        raise SandboxError("isolated process group did not stop") from exc


def _read_output(stream, limit: int) -> tuple[bytes, bool]:
    size = os.fstat(stream.fileno()).st_size
    stream.seek(0)
    data = stream.read(min(size, limit))
    return data, size >= limit


def _entries(root: Path, *, internal_hardlinks: bool = False):
    """Walk without following links, rejecting links outside the supplied tree."""
    pending = [root]
    links = {}
    linked = []
    while pending:
        directory = pending.pop()
        for path in sorted(directory.iterdir(), key=lambda entry: entry.name):
            metadata = path.lstat()
            if stat.S_ISLNK(metadata.st_mode):
                target = os.readlink(path)
                if Path(target).is_absolute():
                    raise SandboxError("absolute or escaping runtime links are forbidden")
                try:
                    resolved = path.resolve(strict=True)
                except (OSError, RuntimeError) as exc:
                    raise SandboxError("runtime links must resolve inside their tree") from exc
                if not resolved.is_relative_to(root):
                    raise SandboxError("absolute or escaping runtime links are forbidden")
            elif stat.S_ISDIR(metadata.st_mode):
                pending.append(path)
            elif stat.S_ISREG(metadata.st_mode):
                if metadata.st_nlink != 1:
                    if not internal_hardlinks:
                        raise SandboxError("runtime hardlinks are forbidden")
                    identity = (metadata.st_dev, metadata.st_ino)
                    links[identity] = links.get(identity, 0) + 1
                    linked.append((identity, metadata.st_nlink))
            else:
                raise SandboxError("special files are forbidden in runtime trees")
            yield path, metadata
    if any(links[identity] != count for identity, count in linked):
        raise SandboxError("workspace hardlinks must stay inside the disposable tree")


def tree_digest(root: Path) -> str:
    """Bind a runtime receipt to its regular contents, paths, links and modes."""
    root = _absolute_path(root, "rootfs")
    if root == Path("/") or not root.is_dir():
        raise SandboxError("rootfs must be a dedicated directory")
    digest = hashlib.sha256(b"baryonos isolated runtime v1\n")
    try:
        for path, metadata in chain(((root, root.stat()),), _entries(root)):
            if not stat.S_ISLNK(metadata.st_mode):
                if metadata.st_mode & 0o022:
                    raise SandboxError("rootfs must not be group/world writable")
                _unprivileged(path, metadata)
            record = [path.relative_to(root).as_posix(), metadata.st_mode]
            if stat.S_ISLNK(metadata.st_mode):
                record.append(os.readlink(path))
            elif stat.S_ISREG(metadata.st_mode):
                with path.open("rb") as source:
                    record.append(hashlib.file_digest(source, "sha256").hexdigest())
                    after = os.fstat(source.fileno())
                    if ((after.st_dev, after.st_ino, after.st_mode, after.st_nlink,
                         after.st_size, after.st_mtime_ns, after.st_ctime_ns)
                            != (metadata.st_dev, metadata.st_ino, metadata.st_mode,
                                metadata.st_nlink, metadata.st_size,
                                metadata.st_mtime_ns, metadata.st_ctime_ns)):
                        raise SandboxError("rootfs changed while being verified")
            digest.update(json.dumps(record, ensure_ascii=True).encode("ascii") + b"\n")
    except OSError as exc:
        raise SandboxError("cannot verify runtime contents") from exc
    return digest.hexdigest()


class Sandbox:
    def __init__(self, config: dict):
        fields = {"backend", "backend_sha256", "rootfs", "rootfs_sha256",
                  "resource_manager", "resource_manager_sha256", "scratch_root",
                  "timeout_seconds", "memory_max_bytes", "tasks_max", "cpu_quota_percent",
                  "file_max_bytes", "open_files_max", "workspace_max_bytes",
                  "workspace_max_entries"}
        if not isinstance(config, dict) or set(config) != fields:
            raise SandboxError("explicit complete sandbox configuration is required")
        if sys.platform != "linux":
            raise SandboxError("the sandbox requires Linux namespaces")
        for field in ("backend_sha256", "rootfs_sha256", "resource_manager_sha256"):
            if not isinstance(config[field], str) or not re.fullmatch(r"[0-9a-f]{64}", config[field]):
                raise SandboxError(f"invalid {field}")
        limit = config["timeout_seconds"]
        if type(limit) is not int or not 1 <= limit <= _MAX_TIMEOUT:
            raise SandboxError("sandbox timeout must be between 1 and 7200 seconds")
        bounds = {
            "memory_max_bytes": (16 * 1024 * 1024, 64 * 1024**3),
            "tasks_max": (2, 4096),
            "cpu_quota_percent": (1, 1000),
            "file_max_bytes": (4096, 8 * 1024**3),
            "open_files_max": (16, 65536),
            "workspace_max_bytes": (4096, 64 * 1024**3),
            "workspace_max_entries": (1, 1_000_000),
        }
        for field, (minimum, maximum) in bounds.items():
            value = config[field]
            if type(value) is not int or not minimum <= value <= maximum:
                raise SandboxError(f"invalid {field}")
        if config["file_max_bytes"] > config["workspace_max_bytes"]:
            raise SandboxError("single-file limit must not exceed the workspace limit")
        self._config = dict(config)
        self.backend = _absolute_path(config["backend"], "backend")
        self.rootfs = _absolute_path(config["rootfs"], "rootfs")
        self.resource_manager = _absolute_path(config["resource_manager"], "resource manager")
        self.scratch_root = _absolute_path(config["scratch_root"], "scratch root")
        self.timeout = limit
        self.memory_max = config["memory_max_bytes"]
        self.tasks_max = config["tasks_max"]
        self.cpu_quota = config["cpu_quota_percent"]
        self.file_max = config["file_max_bytes"]
        self.open_files_max = config["open_files_max"]
        self.workspace_max = config["workspace_max_bytes"]
        self.workspace_entries = config["workspace_max_entries"]

    @contextmanager
    def _verified_backend(self):
        _absolute_path(self.backend, "backend")
        _absolute_path(self.rootfs, "rootfs")
        if tree_digest(self.rootfs) != self._config["rootfs_sha256"]:
            raise SandboxError("rootfs SHA256 does not match its provisioned receipt")
        for name in _MOUNTPOINTS:
            path = self.rootfs / name
            if path.is_symlink() or not path.is_dir() or any(path.iterdir()):
                raise SandboxError(f"rootfs requires an empty directory at /{name}")
        with _verified_executable(self.backend, self._config["backend_sha256"], "backend") as descriptor:
            try:
                version = subprocess.run(
                    _descriptor_command(descriptor, "bwrap", ["--version"]), cwd="/", env=dict(_LAUNCH_ENV),
                    stdin=subprocess.DEVNULL, capture_output=True, text=True,
                    timeout=min(10, self.timeout), check=False, pass_fds=(descriptor,),
                )
            except subprocess.TimeoutExpired as exc:
                raise SandboxError("sandbox backend version check timed out") from exc
            match = re.fullmatch(r"bubblewrap (\d+)\.(\d+)\.(\d+)\n?", version.stdout)
            if version.returncode or not match or tuple(map(int, match.groups())) < (0, 13, 1):
                raise SandboxError("a pinned stable bubblewrap >= 0.13.1 is required")
            yield descriptor

    @contextmanager
    def _verified_resource_manager(self):
        _absolute_path(self.resource_manager, "resource manager")
        with _verified_executable(self.resource_manager, self._config["resource_manager_sha256"],
                                  "resource manager") as descriptor:
            try:
                version = subprocess.run(
                    _descriptor_command(descriptor, "systemd-run", ["--version"]), cwd="/",
                    env=dict(_LAUNCH_ENV), stdin=subprocess.DEVNULL, capture_output=True, text=True,
                    timeout=min(10, self.timeout), check=False, pass_fds=(descriptor,),
                )
            except subprocess.TimeoutExpired as exc:
                raise SandboxError("resource manager version check timed out") from exc
            first = version.stdout.splitlines()[0] if version.stdout else ""
            match = re.fullmatch(r"systemd (\d+) \(.+\)", first)
            if version.returncode or not match or int(match.group(1)) < 259:
                raise SandboxError("a pinned stable systemd-run >= 259 is required")
            yield descriptor

    def verify(self) -> None:
        """Verify pinned files and the fail-closed host resource prerequisites."""
        _require_private_tmpfs(self.scratch_root)
        _resource_environment()
        with self._verified_backend(), self._verified_resource_manager():
            pass

    def run(self, argv: list[str], workspace: Path, *, env: dict[str, str] | None = None,
            timeout: int | None = None, read_only: bool = False) -> subprocess.CompletedProcess:
        if (not isinstance(argv, list) or not argv
                or any(not isinstance(item, str) or "\0" in item for item in argv) or not argv[0]):
            raise SandboxError("sandbox command must be a nonempty argument list")
        deadline = self.timeout if timeout is None else timeout
        if type(deadline) is not int or not 1 <= deadline <= self.timeout:
            raise SandboxError("command timeout must not exceed the sandbox timeout")
        if env is not None and not isinstance(env, dict):
            raise SandboxError("sandbox environment must be a dictionary")
        if type(read_only) is not bool:
            raise SandboxError("source mount mode must be explicit")
        try:
            child_env = isolated_environment(env)
        except (TypeError, ValueError) as exc:
            raise SandboxError("sandbox environment is outside the allowed policy") from exc
        workspace = _absolute_path(workspace, "workspace")
        if (not workspace.is_dir() or workspace == Path("/") or workspace.name == ".git"
                or self.rootfs.is_relative_to(workspace) or workspace.is_relative_to(self.rootfs)
                or self.backend.is_relative_to(workspace)
                or not workspace.is_relative_to(self.scratch_root)):
            raise SandboxError("workspace must be disposable and separate from the runtime")
        _require_private_tmpfs(self.scratch_root)
        _workspace_usage(workspace, self.workspace_max, self.workspace_entries)
        try:
            for path, _metadata in _entries(workspace, internal_hardlinks=True):
                if path.name == ".git":
                    raise SandboxError("publication Git metadata is forbidden in the workspace")
        except OSError as exc:
            raise SandboxError("cannot verify the disposable workspace") from exc
        if read_only:
            for name in ("build", "target"):
                output = workspace / name
                if output.is_symlink() or (output.exists() and not output.is_dir()):
                    raise SandboxError("gate output directory is unsafe")
                output.mkdir(mode=0o700, exist_ok=True)
        with self._verified_backend() as descriptor, \
                self._verified_resource_manager() as manager_descriptor:
            command = [
                "--unshare-all", "--unshare-user", "--unshare-cgroup", "--die-with-parent", "--new-session",
                "--disable-userns", "--assert-userns-disabled", "--cap-drop", "ALL",
                "--ro-bind", str(self.rootfs), "/", "--proc", "/proc", "--dev", "/dev",
            ]
            for directory in ("/tmp", "/run", "/home", "/root"):
                command.extend(("--tmpfs", directory))
            command.extend(("--dir", "/home/agent", "--ro-bind" if read_only else "--bind",
                            str(workspace), "/workspace"))
            if read_only:
                for name in ("build", "target"):
                    command.extend(("--bind", str(workspace / name), f"/workspace/{name}"))
            command.extend(("--chdir", "/workspace", "--clearenv"))
            for key, value in sorted(child_env.items()):
                command.extend(("--setenv", key, value))
            command.extend(("--", *argv))
            unit = f"baryonos-agent-{os.getpid()}-{time.monotonic_ns()}"
            scope = unit + ".scope"
            guard = ["/usr/bin/python3", "-I", "-S", "-B", "-c", _RESOURCE_GUARD,
                     str(descriptor), scope, str(self.memory_max), str(self.tasks_max),
                     str(self.cpu_quota), str(self.file_max), str(self.open_files_max), *command]
            managed = _descriptor_command(manager_descriptor, "systemd-run", [
                "--user", "--scope", "--quiet", "--collect", "--no-ask-password",
                "--expand-environment=no", f"--unit={unit}",
                f"--property=MemoryMax={self.memory_max}", "--property=MemorySwapMax=0",
                f"--property=TasksMax={self.tasks_max}",
                f"--property=CPUQuota={self.cpu_quota}%", "--", *guard,
            ])
            with tempfile.TemporaryFile(dir=self.scratch_root) as stdout_file, \
                    tempfile.TemporaryFile(dir=self.scratch_root) as stderr_file, \
                    subprocess.Popen(
                        managed, cwd="/", env=_resource_environment(), stdin=subprocess.DEVNULL,
                        stdout=stdout_file, stderr=stderr_file, close_fds=True,
                        pass_fds=(descriptor, manager_descriptor), start_new_session=True, umask=0o022,
                    ) as process:
                failure = None
                try:
                    expires = time.monotonic() + deadline
                    while process.poll() is None:
                        _workspace_usage(workspace, self.workspace_max, self.workspace_entries)
                        remaining = expires - time.monotonic()
                        if remaining <= 0:
                            raise subprocess.TimeoutExpired(argv, deadline)
                        time.sleep(min(0.05, remaining))
                    _workspace_usage(workspace, self.workspace_max, self.workspace_entries)
                except BaseException as exc:
                    _stop_process(process)
                    failure = exc
                stdout, stdout_full = _read_output(stdout_file, self.file_max)
                stderr, stderr_full = _read_output(stderr_file, self.file_max)
                if stdout_full or stderr_full:
                    failure = SandboxResourceExceeded("raw output limit reached")
                if failure is not None:
                    if isinstance(failure, (subprocess.TimeoutExpired, SandboxResourceExceeded)):
                        failure.cmd = argv
                        failure.stdout_bytes, failure.stderr_bytes = stdout, stderr
                        failure.output = stdout.decode("utf-8", "replace")
                        failure.stderr = stderr.decode("utf-8", "replace")
                    raise failure
                result = subprocess.CompletedProcess(argv, process.returncode,
                    stdout.decode("utf-8", "replace"), stderr.decode("utf-8", "replace"))
                result.stdout_bytes, result.stderr_bytes = stdout, stderr
                return result
