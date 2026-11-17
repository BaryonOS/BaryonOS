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
import signal
import stat
import subprocess
import sys

from common import isolated_environment


class SandboxError(RuntimeError):
    pass


_LAUNCH_ENV = {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8"}
_MOUNTPOINTS = ("dev", "proc", "sys", "tmp", "run", "home", "root", "workspace")
_MAX_TIMEOUT = 7200
_EXEC_BACKEND = (
    "import os,sys; fd=int(sys.argv[1]); os.set_inheritable(fd,False); "
    "os.execve(fd,sys.argv[2:],dict(os.environ))"
)


def _backend_command(descriptor: int, arguments: list[str]) -> list[str]:
    # Close the checked host descriptor on exec, before any sandbox payload exists.
    return ["/usr/bin/python3", "-I", "-S", "-B", "-c", _EXEC_BACKEND,
            str(descriptor), "bwrap", *arguments]


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
        fields = {"backend", "backend_sha256", "rootfs", "rootfs_sha256", "timeout_seconds"}
        if not isinstance(config, dict) or set(config) != fields:
            raise SandboxError("explicit complete sandbox configuration is required")
        if sys.platform != "linux":
            raise SandboxError("the sandbox requires Linux namespaces")
        for field in ("backend_sha256", "rootfs_sha256"):
            if not isinstance(config[field], str) or not re.fullmatch(r"[0-9a-f]{64}", config[field]):
                raise SandboxError(f"invalid {field}")
        limit = config["timeout_seconds"]
        if type(limit) is not int or not 1 <= limit <= _MAX_TIMEOUT:
            raise SandboxError("sandbox timeout must be between 1 and 7200 seconds")
        self._config = dict(config)
        self.backend = _absolute_path(config["backend"], "backend")
        self.rootfs = _absolute_path(config["rootfs"], "rootfs")
        self.timeout = limit

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
        descriptor = None
        try:
            descriptor = os.open(self.backend, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK)
            metadata = os.fstat(descriptor)
            if (not stat.S_ISREG(metadata.st_mode) or not metadata.st_mode & 0o111
                    or metadata.st_mode & 0o022 or metadata.st_nlink != 1):
                raise SandboxError("backend must be a regular, unshared executable")
            _unprivileged(descriptor, metadata)
            with os.fdopen(os.dup(descriptor), "rb") as source:
                actual = hashlib.file_digest(source, "sha256").hexdigest()
            if actual != self._config["backend_sha256"]:
                raise SandboxError("backend SHA256 does not match the configured pin")
            # Execute the checked inode, rather than resolving the pathname again.
            try:
                version = subprocess.run(
                    _backend_command(descriptor, ["--version"]), cwd="/", env=dict(_LAUNCH_ENV),
                    stdin=subprocess.DEVNULL, capture_output=True, text=True,
                    timeout=min(10, self.timeout), check=False, pass_fds=(descriptor,),
                )
            except subprocess.TimeoutExpired as exc:
                raise SandboxError("sandbox backend version check timed out") from exc
            match = re.fullmatch(r"bubblewrap (\d+)\.(\d+)\.(\d+)\n?", version.stdout)
            if version.returncode or not match or tuple(map(int, match.groups())) < (0, 13, 1):
                raise SandboxError("a pinned stable bubblewrap >= 0.13.1 is required")
            yield descriptor
        except OSError as exc:
            raise SandboxError("sandbox backend verification or startup failed") from exc
        finally:
            if descriptor is not None:
                os.close(descriptor)

    def verify(self) -> None:
        """Verify the pinned files; namespace availability is enforced at launch."""
        with self._verified_backend():
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
                or self.backend.is_relative_to(workspace)):
            raise SandboxError("workspace must be disposable and separate from the runtime")
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
        with self._verified_backend() as descriptor:
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
            with subprocess.Popen(
                _backend_command(descriptor, command), cwd="/", env=dict(_LAUNCH_ENV), stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, close_fds=True,
                pass_fds=(descriptor,), start_new_session=True, umask=0o022,
            ) as process:
                try:
                    stdout, stderr = process.communicate(timeout=deadline)
                except BaseException as exc:
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    stdout, stderr = process.communicate(timeout=5)
                    if isinstance(exc, subprocess.TimeoutExpired):
                        exc.cmd = argv
                        exc.stdout_bytes, exc.stderr_bytes = stdout, stderr
                        exc.output = stdout.decode("utf-8", "replace")
                        exc.stderr = stderr.decode("utf-8", "replace")
                    raise
                result = subprocess.CompletedProcess(argv, process.returncode,
                    stdout.decode("utf-8", "replace"), stderr.decode("utf-8", "replace"))
                result.stdout_bytes, result.stderr_bytes = stdout, stderr
                return result
