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
import secrets
import selectors
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
_PROFILE_FIELDS = {
    "memory_high_bytes", "memory_max_bytes", "tasks_max", "cpu_quota_percent",
    "output_max_bytes", "open_files_max",
}
_EXEC_DESCRIPTOR = (
    "import os,sys; fd=int(sys.argv[1]); os.set_inheritable(fd,False); "
    "os.execve(fd,sys.argv[2:],dict(os.environ))"
)
_RESOURCE_GUARD = r"""
import json, os, resource, sys
from pathlib import Path

def fail(message):
    print("resource guard: " + message, file=sys.stderr)
    raise SystemExit(125)

try:
    descriptor = int(sys.argv[1])
    receipt_descriptor = int(sys.argv[2])
    scope = sys.argv[3]
    high, maximum, tasks, cpu, open_files = map(int, sys.argv[4:9])
    memberships = [line.split("::", 1)[1] for line in Path("/proc/self/cgroup").read_text().splitlines()
                   if line.startswith("0::")]
    if len(memberships) != 1:
        fail("unified cgroup v2 membership is unavailable")
    cgroup = Path("/sys/fs/cgroup") / memberships[0].lstrip("/")
    if cgroup.name != scope:
        fail("process is outside its owner-selected resource scope")
    (cgroup / "memory.oom.group").write_text("1\n")
    expected = {
        "memory.high": str(high), "memory.max": str(maximum), "memory.swap.max": "0",
        "memory.oom.group": "1", "pids.max": str(tasks),
    }
    for name, value in expected.items():
        if (cgroup / name).read_text().strip() != value:
            fail(name + " does not match policy")
    quota, period = (cgroup / "cpu.max").read_text().split()
    if quota == "max" or int(quota) * 100 != cpu * int(period):
        fail("cpu.max does not match policy")
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    resource.setrlimit(resource.RLIMIT_NOFILE, (open_files, open_files))
    receipt = {
        "cgroup": str(cgroup), "memory_high": expected["memory.high"],
        "memory_max": expected["memory.max"], "memory_swap_max": expected["memory.swap.max"],
        "memory_oom_group": expected["memory.oom.group"], "tasks_max": expected["pids.max"],
        "cpu_quota": quota, "cpu_period": period, "open_files_max": str(open_files),
    }
    os.write(receipt_descriptor, (json.dumps(receipt, sort_keys=True) + "\n").encode())
    os.close(receipt_descriptor)
    os.set_inheritable(descriptor, False)
    os.execve(descriptor, ["bwrap", *sys.argv[9:]], {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8"})
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
        except OSError as exc:
            raise SandboxError(f"{label} verification or startup failed") from exc
        yield descriptor
    finally:
        if descriptor is not None:
            os.close(descriptor)


def _resource_environment() -> dict[str, str]:
    runtime = Path("/run/user") / str(os.getuid())
    bus = runtime / "bus"
    try:
        runtime_metadata = runtime.lstat()
        bus_metadata = bus.lstat()
    except OSError as exc:
        raise SandboxError("the private user resource manager is unavailable") from exc
    if (runtime.is_symlink() or bus.is_symlink() or not stat.S_ISDIR(runtime_metadata.st_mode)
            or runtime_metadata.st_uid != os.getuid() or stat.S_IMODE(runtime_metadata.st_mode) != 0o700
            or not stat.S_ISSOCK(bus_metadata.st_mode) or bus_metadata.st_uid != os.getuid()):
        raise SandboxError("the private user resource manager is unsafe")
    return {**_LAUNCH_ENV, "XDG_RUNTIME_DIR": str(runtime),
            "DBUS_SESSION_BUS_ADDRESS": f"unix:path={bus}"}


def _validated_profiles(value: object) -> dict[str, dict[str, int]]:
    if not isinstance(value, dict) or set(value) != {"agent", "gate"}:
        raise SandboxError("explicit agent and gate resource profiles are required")
    profiles = {}
    for name, profile in value.items():
        if not isinstance(profile, dict) or set(profile) != _PROFILE_FIELDS:
            raise SandboxError(f"resource profile {name} is incomplete")
        if any(type(profile[field]) is not int for field in _PROFILE_FIELDS):
            raise SandboxError(f"resource profile {name} requires integer limits")
        high = profile["memory_high_bytes"]
        maximum = profile["memory_max_bytes"]
        if not 16 * 1024 * 1024 <= high <= maximum <= 64 * 1024 * 1024 * 1024:
            raise SandboxError(f"resource profile {name} has invalid memory limits")
        if not 16 <= profile["tasks_max"] <= 4096:
            raise SandboxError(f"resource profile {name} has invalid task limit")
        if not 25 <= profile["cpu_quota_percent"] <= 800:
            raise SandboxError(f"resource profile {name} has invalid CPU quota")
        if not 1024 * 1024 <= profile["output_max_bytes"] <= 256 * 1024 * 1024:
            raise SandboxError(f"resource profile {name} has invalid output limit")
        if not 64 <= profile["open_files_max"] <= 4096:
            raise SandboxError(f"resource profile {name} has invalid file-descriptor limit")
        profiles[name] = dict(profile)
    return profiles


def _communicate_bounded(process: subprocess.Popen, limit: int, timeout: int,
                         terminate) -> tuple[bytes, bytes, bool, bool]:
    """Drain both pipes without unbounded memory or payload-wide file-size limits."""
    if process.stdout is None or process.stderr is None:
        raise SandboxError("isolated output pipes are unavailable")
    streams = {process.stdout: bytearray(), process.stderr: bytearray()}
    selector = selectors.DefaultSelector()
    for stream in streams:
        selector.register(stream, selectors.EVENT_READ)
    deadline = time.monotonic() + timeout
    stopped_at = None
    exceeded = False
    timed_out = False
    try:
        while selector.get_map() or process.poll() is None:
            now = time.monotonic()
            if stopped_at is None and now >= deadline:
                timed_out = True
                terminate()
                stopped_at = now
            if stopped_at is not None and now - stopped_at >= 5:
                break
            wait = 0.05 if stopped_at is not None else max(0.0, min(0.05, deadline - now))
            for key, _events in selector.select(wait):
                data = os.read(key.fd, 65536)
                if not data:
                    selector.unregister(key.fileobj)
                    key.fileobj.close()
                    continue
                output = streams[key.fileobj]
                remaining = max(0, limit - len(output))
                output.extend(data[:remaining])
                if len(data) > remaining and not exceeded:
                    exceeded = True
                    terminate()
                    stopped_at = time.monotonic()
        return bytes(streams[process.stdout]), bytes(streams[process.stderr]), exceeded, timed_out
    finally:
        selector.close()


def _guard_receipt(stream, scope: str, limits: dict[str, int]) -> dict[str, str]:
    stream.seek(0)
    raw = stream.read(4097)
    if len(raw) > 4096 or not raw.endswith(b"\n") or raw.count(b"\n") != 1:
        raise SandboxError("resource guard did not produce a bounded receipt")
    try:
        receipt = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise SandboxError("resource guard receipt is invalid") from exc
    expected = {
        "memory_high": str(limits["memory_high_bytes"]),
        "memory_max": str(limits["memory_max_bytes"]),
        "memory_swap_max": "0", "memory_oom_group": "1",
        "tasks_max": str(limits["tasks_max"]),
        "cpu_quota": str(limits["cpu_quota_percent"] * 1000),
        "cpu_period": "100000", "open_files_max": str(limits["open_files_max"]),
    }
    cgroup_value = receipt.pop("cgroup", None) if isinstance(receipt, dict) else None
    cgroup = Path(cgroup_value) if isinstance(cgroup_value, str) else Path()
    if (receipt != expected or not cgroup.is_absolute()
            or not cgroup.is_relative_to(Path("/sys/fs/cgroup")) or cgroup.name != scope):
        raise SandboxError("resource guard receipt does not match policy")
    return {"cgroup": str(cgroup), **receipt}


def resource_evidence(outcome) -> dict[str, object]:
    """Return only controller-measured resource fields suitable for bounded logs."""
    status = getattr(outcome, "resource_status", None)
    if not isinstance(status, dict):
        return {}
    fields = ("profile", "unit")
    evidence = {name: status[name] for name in fields
                if isinstance(status.get(name), str) and len(status[name]) <= 256}
    if status.get("LoadState") == "loaded":
        for name in ("Result", "MemoryPeak", "MemoryHigh", "MemoryMax",
                     "MemorySwapMax", "TasksMax", "CPUUsageNSec"):
            if isinstance(status.get(name), str) and len(status[name]) <= 256:
                evidence[name] = status[name]
    verified = status.get("verified")
    if isinstance(verified, dict):
        allowed = (
            "cgroup", "memory_high", "memory_max", "memory_swap_max",
            "memory_oom_group", "tasks_max", "cpu_quota", "cpu_period", "open_files_max",
        )
        evidence["verified"] = {name: verified[name] for name in allowed
                                if isinstance(verified.get(name), str)
                                and len(verified[name]) <= 512}
    return evidence


def _properties(output: str) -> dict[str, str]:
    return dict(line.split("=", 1) for line in output.splitlines() if "=" in line)


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
        fields = {
            "backend", "backend_sha256", "rootfs", "rootfs_sha256",
            "resource_manager", "resource_manager_sha256",
            "resource_control", "resource_control_sha256", "timeout_seconds", "profiles",
        }
        if not isinstance(config, dict) or set(config) != fields:
            raise SandboxError("explicit complete sandbox configuration is required")
        if sys.platform != "linux":
            raise SandboxError("the sandbox requires Linux namespaces")
        for field in ("backend_sha256", "rootfs_sha256", "resource_manager_sha256",
                      "resource_control_sha256"):
            if not isinstance(config[field], str) or not re.fullmatch(r"[0-9a-f]{64}", config[field]):
                raise SandboxError(f"invalid {field}")
        limit = config["timeout_seconds"]
        if type(limit) is not int or not 1 <= limit <= _MAX_TIMEOUT:
            raise SandboxError("sandbox timeout must be between 1 and 7200 seconds")
        self._config = dict(config)
        self.backend = _absolute_path(config["backend"], "backend")
        self.rootfs = _absolute_path(config["rootfs"], "rootfs")
        self.resource_manager = _absolute_path(config["resource_manager"], "resource manager")
        self.resource_control = _absolute_path(config["resource_control"], "resource control")
        self.profiles = _validated_profiles(config["profiles"])
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
        with _verified_executable(self.backend, self._config["backend_sha256"], "backend") as descriptor:
            # Execute the checked inode, rather than resolving the pathname again.
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
    def _verified_resource_tools(self):
        environment = _resource_environment()
        with _verified_executable(
                self.resource_manager, self._config["resource_manager_sha256"],
                "resource manager") as manager, _verified_executable(
                self.resource_control, self._config["resource_control_sha256"],
                "resource control") as control:
            try:
                version = subprocess.run(
                    _descriptor_command(manager, "systemd-run", ["--version"]), cwd="/",
                    env=dict(_LAUNCH_ENV), stdin=subprocess.DEVNULL, capture_output=True, text=True,
                    timeout=min(10, self.timeout), check=False, pass_fds=(manager,),
                )
            except subprocess.TimeoutExpired as exc:
                raise SandboxError("resource manager version check timed out") from exc
            first = version.stdout.splitlines()[0] if version.stdout else ""
            match = re.fullmatch(r"systemd (\d+) \(.+\)", first)
            if version.returncode or not match or int(match.group(1)) < 259:
                raise SandboxError("a pinned stable systemd-run >= 259 is required")
            yield manager, control, environment

    def _control(self, descriptor: int, environment: dict[str, str], arguments: list[str]):
        try:
            return subprocess.run(
                _descriptor_command(descriptor, "systemctl", ["--user", *arguments]), cwd="/",
                env=environment, stdin=subprocess.DEVNULL, capture_output=True, text=True,
                timeout=min(10, self.timeout), check=False, pass_fds=(descriptor,),
            )
        except subprocess.TimeoutExpired as exc:
            raise SandboxError("resource control timed out") from exc

    def _unit_properties(self, descriptor: int, environment: dict[str, str], unit: str) -> dict[str, str]:
        names = (
            "LoadState,ActiveState,SubState,Result,ExecMainCode,ExecMainStatus,ControlGroup,"
            "MemoryCurrent,MemoryPeak,MemoryHigh,MemoryMax,MemorySwapMax,TasksCurrent,TasksMax,"
            "CPUUsageNSec,CPUQuotaPerSecUSec,CPUQuotaPeriodUSec,KillMode,RuntimeMaxUSec"
        )
        result = self._control(descriptor, environment, ["show", unit, f"--property={names}"])
        if result.returncode:
            raise SandboxError("cannot inspect the isolated resource scope")
        return _properties(result.stdout)

    @staticmethod
    def _scope_populated(properties: dict[str, str]) -> bool:
        relative = properties.get("ControlGroup", "").lstrip("/")
        if not relative:
            return False
        cgroup = Path("/sys/fs/cgroup") / relative
        try:
            resolved = cgroup.resolve(strict=True)
            if not resolved.is_relative_to(Path("/sys/fs/cgroup")):
                raise SandboxError("resource scope escaped cgroup v2")
            events = dict(line.split(None, 1) for line in (resolved / "cgroup.events").read_text().splitlines())
            return events.get("populated") == "1"
        except FileNotFoundError:
            return False
        except OSError as exc:
            raise SandboxError("cannot verify resource-scope cleanup") from exc

    def _cleanup_scope(self, descriptor: int, environment: dict[str, str], unit: str,
                       initial: dict[str, str]) -> None:
        state = initial
        if self._scope_populated(state):
            killed = self._control(descriptor, environment, [
                "kill", "--kill-whom=all", "--signal=SIGKILL", "--wait", unit,
            ])
            if killed.returncode:
                raise SandboxError("cannot kill the complete isolated resource scope")
        reset = self._control(descriptor, environment, ["reset-failed", unit])
        expires = time.monotonic() + 5
        while True:
            state = self._unit_properties(descriptor, environment, unit)
            if (state.get("LoadState") == "not-found"
                    or (state.get("ActiveState") in {"inactive", "failed"}
                        and not self._scope_populated(state))):
                break
            if time.monotonic() >= expires:
                raise SandboxError("isolated resource scope did not become empty")
            time.sleep(0.02)
        if reset.returncode and state.get("LoadState") != "not-found":
            raise SandboxError("cannot release the isolated resource scope")

    def _terminate_scope(self, descriptor: int, environment: dict[str, str], unit: str) -> None:
        killed = self._control(descriptor, environment, [
            "kill", "--kill-whom=all", "--signal=SIGKILL", "--wait", unit,
        ])
        if killed.returncode:
            state = self._unit_properties(descriptor, environment, unit)
            if state.get("LoadState") != "not-found":
                raise SandboxError("cannot terminate the complete isolated resource scope")

    def verify(self) -> None:
        """Verify pinned files and the fail-closed host resource prerequisites."""
        with self._verified_backend(), self._verified_resource_tools():
            pass

    def run(self, argv: list[str], workspace: Path, *, profile: str,
            env: dict[str, str] | None = None, timeout: int | None = None,
            read_only: bool = False) -> subprocess.CompletedProcess:
        if (not isinstance(argv, list) or not argv
                or any(not isinstance(item, str) or "\0" in item for item in argv) or not argv[0]):
            raise SandboxError("sandbox command must be a nonempty argument list")
        if profile not in self.profiles:
            raise SandboxError("an explicit configured resource profile is required")
        limits = self.profiles[profile]
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
                or self.resource_manager.is_relative_to(workspace)
                or self.resource_control.is_relative_to(workspace)):
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
        with self._verified_backend() as descriptor, \
                self._verified_resource_tools() as (manager, control, resource_environment):
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
            unit = f"baryonos-agent-{secrets.token_hex(12)}.scope"
            runtime = Path(resource_environment["XDG_RUNTIME_DIR"])
            with tempfile.TemporaryFile(prefix="baryonos-agent-receipt-", dir=runtime) as receipt_file:
                guard = [
                    "/usr/bin/python3", "-I", "-S", "-B", "-c", _RESOURCE_GUARD,
                    str(descriptor), str(receipt_file.fileno()), unit,
                    str(limits["memory_high_bytes"]), str(limits["memory_max_bytes"]),
                    str(limits["tasks_max"]), str(limits["cpu_quota_percent"]),
                    str(limits["open_files_max"]), *command,
                ]
                managed = _descriptor_command(manager, "systemd-run", [
                    "--user", "--scope", "--quiet", "--no-ask-password", "--expand-environment=no",
                    f"--unit={unit}", f"--property=RuntimeMaxSec={deadline}s",
                    f"--property=MemoryHigh={limits['memory_high_bytes']}",
                    f"--property=MemoryMax={limits['memory_max_bytes']}",
                    "--property=MemorySwapMax=0", "--property=MemoryAccounting=yes",
                    f"--property=TasksMax={limits['tasks_max']}", "--property=TasksAccounting=yes",
                    f"--property=CPUQuota={limits['cpu_quota_percent']}%",
                    "--property=CPUQuotaPeriodSec=100ms",
                    "--property=KillMode=control-group", "--property=SendSIGKILL=yes",
                    "--property=TimeoutStopSec=1s", "--property=OOMPolicy=kill", "--", *guard,
                ])
                try:
                    process = subprocess.Popen(
                        managed, cwd="/", env=resource_environment, stdin=subprocess.DEVNULL,
                        stdout=subprocess.PIPE, stderr=subprocess.PIPE, close_fds=True,
                        pass_fds=(descriptor, manager, receipt_file.fileno()),
                        start_new_session=True, umask=0o022,
                    )
                except OSError as exc:
                    raise SandboxError("isolated resource scope failed to start") from exc
                stop_error = None

                def stop_scope():
                    nonlocal stop_error
                    if stop_error is None:
                        try:
                            self._terminate_scope(control, resource_environment, unit)
                        except BaseException as exc:
                            stop_error = exc

                stdout = stderr = b""
                output_exceeded = timed_out = False
                execution_error = launcher_error = None
                cleanup_error = None
                try:
                    stdout, stderr, output_exceeded, timed_out = _communicate_bounded(
                        process, limits["output_max_bytes"], deadline, stop_scope)
                except BaseException as exc:
                    execution_error = exc
                finally:
                    if process.poll() is None:
                        stop_scope()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        try:
                            os.killpg(process.pid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                        try:
                            process.wait(timeout=5)
                        except subprocess.TimeoutExpired as exc:
                            launcher_error = exc
                    try:
                        resource_status = self._unit_properties(control, resource_environment, unit)
                        self._cleanup_scope(control, resource_environment, unit, resource_status)
                    except BaseException as exc:
                        resource_status = {}
                        cleanup_error = exc
                if cleanup_error is not None:
                    raise SandboxError(f"isolated resource cleanup failed: {cleanup_error}") from cleanup_error
                if stop_error is not None:
                    raise SandboxError(f"isolated resource termination failed: {stop_error}") from stop_error
                if launcher_error is not None:
                    raise SandboxError("isolated scope launcher did not stop") from launcher_error
                if execution_error is not None:
                    raise execution_error
                verified = _guard_receipt(receipt_file, unit, limits)
                if output_exceeded:
                    error = SandboxResourceExceeded("captured output exceeded its resource profile")
                    error.stdout_bytes, error.stderr_bytes = stdout, stderr
                    error.resource_status = {"profile": profile, "unit": unit,
                                             "verified": verified, **resource_status}
                    raise error
                if timed_out or resource_status.get("Result") == "timeout":
                    error = subprocess.TimeoutExpired(argv, deadline, stdout, stderr)
                    error.stdout_bytes, error.stderr_bytes = stdout, stderr
                    error.output = stdout.decode("utf-8", "replace")
                    error.stderr = stderr.decode("utf-8", "replace")
                    error.resource_status = {"profile": profile, "unit": unit,
                                             "verified": verified, **resource_status}
                    raise error
                result = subprocess.CompletedProcess(argv, process.returncode,
                    stdout.decode("utf-8", "replace"), stderr.decode("utf-8", "replace"))
                result.stdout_bytes, result.stderr_bytes = stdout, stderr
                result.resource_status = {"profile": profile, "unit": unit,
                                          "verified": verified, **resource_status}
                return result
