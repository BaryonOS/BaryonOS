"""Shared host tooling. No global environment changes or guest network access."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import time
from collections.abc import Callable, Mapping

ROOT = Path(__file__).resolve().parents[1]
CONTROLLER_GIT = "/usr/bin/git"
CONTROLLER_GH = "/usr/bin/gh"
BUILD = Path(os.environ.get("NOS_BUILD_DIR", str(ROOT / "build"))).resolve()
TARGET = Path(os.environ.get("NOS_TARGET_DIR", str(ROOT / "target"))).resolve()
LOCAL = Path(os.environ.get("LOCALAPPDATA", str(ROOT / ".tools"))) / "nieuw-os"
QEMU_MACHINES = ("pc-q35-10.0", "pc-q35-8.2", "pc-q35-7.2")
# Pinned q35/OVMF profile without an IOMMU: MCFG present, DMAR absent. Any
# other ACPI status (rejected handoff, invalid tables, ...) is a gate failure.
EXPECTED_ACPI_MARKER = "ACPI:UNAVAILABLE:NO_DMAR"
DEVICE_EXECUTION_MARKER = "DEVICE-EXECUTION:BLOCKED:ADAPTER_ABSENT"
SOURCE_RECEIPT_NAME = ".baryonos-source-receipt.json"


def without_acpi_diagnostics(stdout: str) -> str:
    """Remove only the bounded ACPI status pair from the KS-02/KS-03 gap."""
    if not isinstance(stdout, str):
        raise ValueError("Malformed native serial output")
    lines = stdout.splitlines(keepends=True)
    values = [line.rstrip("\r\n") for line in lines]
    acpi = [i for i, line in enumerate(values) if line.startswith("ACPI:")]
    execution = [i for i, line in enumerate(values) if line.startswith("DEVICE-EXECUTION:")]
    if not acpi and not execution:
        return stdout
    if (len(acpi) != 1 or len(execution) != 1
            or values[acpi[0]] != EXPECTED_ACPI_MARKER
            or values[execution[0]] != DEVICE_EXECUTION_MARKER
            or execution[0] != acpi[0] + 1
            or acpi[0] == 0 or acpi[0] + 2 >= len(values)
            or values[acpi[0] - 1] != "KS-02:PASS"
            or values[acpi[0] + 2] != "KS-03:PASS"):
        raise ValueError("Malformed or misplaced ACPI boot diagnostics")
    del lines[acpi[0]:acpi[0] + 2]
    return "".join(lines)


def with_acpi_markers(required: tuple[str, ...]) -> tuple[str, ...]:
    """Require the native ACPI status pair directly after KS-02 in a live run."""
    at = required.index("KS-02:PASS") + 1
    return required[:at] + (EXPECTED_ACPI_MARKER, DEVICE_EXECUTION_MARKER) + required[at:]


def qemu_machine(value: str | None = None) -> str:
    """An explicit development profile, never an automatic machine fallback."""
    selected = os.environ.get("NOS_QEMU_MACHINE", "pc-q35-10.0") if value is None else value
    if selected not in QEMU_MACHINES:
        raise ValueError("Choose an explicitly supported and recorded QEMU machine")
    return selected


def qemu_accel(value: str | None = None, *, device: Path = Path("/dev/kvm")) -> str:
    """Keep TCG as the default and require an explicit, usable KVM device."""
    selected = os.environ.get("NOS_QEMU_ACCEL") if value is None else value
    if selected is None:
        return "tcg,thread=single"
    if selected != "kvm":
        raise ValueError("NOS_QEMU_ACCEL may select only the explicit KVM pilot")
    if not device.is_char_device() or not os.access(device, os.R_OK | os.W_OK):
        raise RuntimeError("KVM was requested but /dev/kvm is not a usable character device")
    return "kvm"


def environment() -> dict[str, str]:
    env = dict(os.environ, RUSTUP_AUTO_INSTALL="0", PYTHONIOENCODING="utf-8")
    paths = []
    if (LOCAL / "cargo/bin").is_dir() and (LOCAL / "rustup").is_dir():
        env["CARGO_HOME"] = str(LOCAL / "cargo")
        env["RUSTUP_HOME"] = str(LOCAL / "rustup")
        paths.append(str(LOCAL / "cargo/bin"))
    if (LOCAL / "qemu").is_dir():
        paths.append(str(LOCAL / "qemu"))
    # Rust 1.90 predates VS 2026. Discover the existing compiler without installing
    # or changing a developer shell; this environment belongs only to our children.
    if os.name == "nt":
        program_files = Path(os.environ.get("ProgramFiles", "C:/Program Files"))
        compiler_roots = sorted(program_files.glob("Microsoft Visual Studio/*/*/VC/Tools/MSVC/*"))
        kits = Path(os.environ.get("ProgramFiles(x86)", "C:/Program Files (x86)")) / "Windows Kits/10/Lib"
        sdk_roots = sorted(kits.glob("*")) if kits.exists() else []
        if compiler_roots and sdk_roots:
            compiler, sdk = compiler_roots[-1], sdk_roots[-1]
            paths.append(str(compiler / "bin/Hostx64/x64"))
            libs = [compiler / "lib/x64", sdk / "ucrt/x64", sdk / "um/x64"]
            env["LIB"] = os.pathsep.join(map(str, libs))
    env["PATH"] = os.pathsep.join(paths + [env.get("PATH", "")])
    rustc = shutil.which("rustc", path=env["PATH"])
    if rustc:
        sysroot = subprocess.run([rustc, "+1.100.0", "--print", "sysroot"], env=env, cwd=ROOT, text=True, capture_output=True, timeout=15)
        version = subprocess.run([rustc, "+1.100.0", "-vV"], env=env, cwd=ROOT, text=True, capture_output=True, timeout=15)
        if sysroot.returncode == version.returncode == 0:
            host = next(line.split(": ", 1)[1] for line in version.stdout.splitlines() if line.startswith("host: "))
            env["PATH"] = str(Path(sysroot.stdout.strip()) / "lib/rustlib" / host / "bin") + os.pathsep + env["PATH"]
    return env


def isolated_environment(extra: dict[str, str] | None = None) -> dict[str, str]:
    """Offline worker defaults; never inspect or inherit the controller environment."""
    env = {
        "PATH": "/usr/bin:/bin:/opt/rust/bin", "HOME": "/home/agent", "TMPDIR": "/tmp",
        "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "GITHUB_WORKSPACE": "/workspace",
        "CARGO_HOME": "/home/agent/.cargo", "CARGO_NET_OFFLINE": "true",
        "RUSTUP_AUTO_INSTALL": "0", "PYTHONDONTWRITEBYTECODE": "1",
        "PYTHONPYCACHEPREFIX": "/usr/.baryonos-pycache",
        "PYTHONIOENCODING": "utf-8", "GIT_CONFIG_NOSYSTEM": "1",
        "GIT_CONFIG_GLOBAL": "/dev/null", "GIT_TERMINAL_PROMPT": "0",
    }
    allowed = {
        "NOS_QEMU_MACHINE", "OS_OVMF_CODE", "OS_OVMF_VARS", "VGA_ROM",
        "NOS_BUILD_DIR", "NOS_TARGET_DIR", "CARGO_TARGET_DIR", "NODE_V8_COVERAGE",
        "RUSTFLAGS", "RUST_TOOLCHAIN", "COMPONENT_MANIFEST", "COMPONENT_FEATURES",
        "ACTIONLINT_VERSION", "ACTIONLINT_SHA256", "MACHINE", "OUT",
        "NOS_SOURCE_RECEIPT",
    }
    for key, value in (extra or {}).items():
        if not isinstance(value, str) or "\0" in value:
            raise ValueError("invalid isolated environment value")
        if key in env:
            if value != env[key]:
                raise ValueError(f"isolated environment fixes {key}")
        elif key not in allowed:
            raise ValueError(f"environment key is unavailable in isolation: {key}")
        else:
            env[key] = value
    return env


def controller_environment(*, authenticated: bool = False) -> dict[str, str]:
    """Fixed tool lookup, preserving configuration checks without ambient exec overrides."""
    if any(key.startswith("GIT_") and key not in {"GIT_TERMINAL_PROMPT", "GIT_PAGER"} for key in os.environ):
        raise ValueError("Git environment overrides require a verified controller adapter")
    env = {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8", "GIT_TERMINAL_PROMPT": "0",
           "GIT_PAGER": "/usr/bin/cat", "GIT_NO_LAZY_FETCH": "1"}
    for key in ("HOME", "XDG_CONFIG_HOME"):
        if key in os.environ:
            if not Path(os.environ[key]).is_absolute():
                raise ValueError("controller configuration directories must be absolute")
            env[key] = os.environ[key]
    if authenticated:
        for key in ("GH_TOKEN", "GITHUB_TOKEN", "GH_CONFIG_DIR"):
            if key in os.environ:
                if key == "GH_CONFIG_DIR" and not Path(os.environ[key]).is_absolute():
                    raise ValueError("GitHub configuration directory must be absolute")
                env[key] = os.environ[key]
    return env


def agent_relative(value: str) -> Path:
    """Only literal repository-relative file names cross the worker boundary."""
    if (not isinstance(value, str) or not value or value.startswith("/")
            or any(c in value for c in ("\\", ":", "\0", "\n", "\r", "\t"))
            or any(p in {"", ".", "..", ".git"} for p in value.split("/"))):
        raise ValueError("invalid worker source path")
    return Path(value)


def agent_file(root: Path, relative: str) -> Path:
    path = root / agent_relative(relative)
    for part in (path, *path.parents):
        if part == root:
            break
        if part.is_symlink():
            raise ValueError("worker source contains a symbolic link")
    if path.exists():
        mode = path.stat()
        if not stat.S_ISREG(mode.st_mode) or mode.st_nlink != 1:
            raise ValueError("worker source must be a private regular file")
    return path


def agent_source_paths(extra: tuple[str, ...] | list[str] = ()) -> list[str]:
    """The controller index supplies source names; no worker Git metadata is read."""
    result = subprocess.run([CONTROLLER_GIT, "--no-lazy-fetch", "ls-files", "-z"], cwd=ROOT, check=True,
                            env=controller_environment(), capture_output=True, text=True)
    paths = sorted(set(result.stdout.rstrip("\0").split("\0")) | set(extra))
    for path in paths:
        agent_relative(path)
    return paths


def stage_agent_source(source: Path, stage: Path, paths: list[str], *,
                       max_bytes: int, max_files: int) -> None:
    """Copy reviewed source only, excluding ignored credentials, caches and Git state."""
    selected = []
    total = 0
    for relative in paths:
        original = agent_file(source, relative)
        if not original.exists():
            continue
        selected.append((relative, original))
        total += original.stat().st_size
    if len(selected) > max_files or total > max_bytes:
        raise ValueError("worker source exceeds its owner-authorized staging budget")
    for relative, original in selected:
        target = stage / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(original, target, follow_symlinks=False)
        target.chmod(0o755 if original.stat().st_mode & 0o111 else 0o644)


def agent_source_selection_state(root: Path, paths: list[str]) -> dict[str, tuple[str, int] | None]:
    """Hash exactly the regular source selection, normalising its staged mode."""
    result = {}
    for relative in paths:
        path = agent_file(root, relative)
        result[relative] = ((sha256(path), 0o755 if path.stat().st_mode & 0o111 else 0o644)
                            if path.exists() else None)
    return result


def agent_source_state(root: Path) -> dict[str, tuple[str, int]]:
    result = {}
    for parent, directories, files in os.walk(root, followlinks=False):
        for name in directories:
            if (Path(parent) / name).is_symlink():
                raise ValueError("worker output contains a symbolic link")
        for name in files:
            relative = (Path(parent) / name).relative_to(root).as_posix()
            path = agent_file(root, relative)
            result[relative] = (sha256(path), stat.S_IMODE(path.stat().st_mode))
    return result


def import_agent_changes(stage: Path, destination: Path, before: dict,
                         allowed: tuple[str, ...] | list[str], *,
                         max_bytes: int, max_files: int) -> list[str]:
    """Validate the entire delta before importing any task-owned regular file."""
    after = agent_source_state(stage)
    changed = sorted(p for p in before.keys() | after.keys() if before.get(p) != after.get(p))
    if set(changed) - set(allowed):
        raise ValueError("worker changed files outside the owner-authorized write set")
    exported = sum((stage / relative).stat().st_size for relative in changed if relative in after)
    if len(changed) > max_files or exported > max_bytes:
        raise ValueError("worker output exceeds its owner-authorized export budget")
    for relative in changed:
        agent_file(destination, relative)
        if relative in after and after[relative][1] not in {0o644, 0o755}:
            raise ValueError("worker output has unsupported permissions")
    for relative in changed:
        target = destination / relative
        if relative not in after:
            target.unlink()
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        # Both trees are controller-private and no worker process survives Sandbox.run.
        shutil.copyfile(stage / relative, target, follow_symlinks=False)
        target.chmod(after[relative][1])
    return changed


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def require_sha256(value: object) -> str:
    if not isinstance(value, str) or re.fullmatch(r"[0-9a-f]{64}", value) is None:
        raise ValueError("missing or malformed SHA256")
    return value


def regular_files_below(root: Path, *, validate_path: Callable[[Path], Path]) -> set[Path]:
    """List regular descendants, validating each path before checking its type."""
    files: set[Path] = set()
    for path in root.rglob("*"):
        validate_path(path)
        if path.is_file():
            files.add(path)
    return files


def save_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def source_hash(root: Path | None = None) -> str:
    """Fingerprint source inputs without traversing generated artifacts or caches."""
    root = (ROOT if root is None else root).resolve()
    digest = hashlib.sha256()
    digest.update(b"nieuw-os source inputs v3\n")
    for folder in (root / "boot", root / "kernel", root / "crates", root / "drivers", root / "services", root / "userspace", root / "sdk", root / "ui/native", root / "ui/graphics", root / "ui/surface-protocol", root / "experiments/quantum/qsim-core", root / "experiments/quantum/native", root / "experiments/quantum/job-protocol", root / "tools", root / ".cargo"):
        if folder.exists():
            # Path ordering is case-insensitive on Windows. Explicit string
            # components preserve the v3 POSIX order on every build host.
            for path in sorted(folder.rglob("*"), key=lambda item: item.relative_to(folder).parts):
                if path.is_file() and not {"__pycache__", "verification", "target", "build", ".git"}.intersection(path.relative_to(folder).parts):
                    digest.update(path.relative_to(root).as_posix().encode())
                    digest.update(path.read_bytes())
    for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "experiments/quantum/Cargo.toml", "ui/prototype/tokens.css"):
        if (root / name).exists():
            digest.update(name.encode())
            digest.update((root / name).read_bytes())
    return digest.hexdigest()


def controller_source_commit(root: Path) -> str:
    """Read a commit identity without content filters, credentials or lazy fetching."""
    try:
        head = subprocess.check_output(
            [CONTROLLER_GIT, "--no-lazy-fetch", "rev-parse", "HEAD"], cwd=root.resolve(),
            env=controller_environment(), text=True, timeout=10).strip()
    except (OSError, subprocess.SubprocessError) as error:
        raise ValueError("Controller source commit is unavailable") from error
    if not re.fullmatch(r"[0-9a-f]{40}", head):
        raise ValueError("Git returned an invalid source commit")
    return head


def source_identity(root: Path | None = None,
                    environ: Mapping[str, str] | None = None) -> tuple[str, str]:
    """Bind the current source hash to Git or a controller-created read-only receipt."""
    root = (ROOT if root is None else root).resolve()
    frozen = source_hash(root)
    values = os.environ if environ is None else environ
    receipt_value = values.get("NOS_SOURCE_RECEIPT")
    if receipt_value is None:
        head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root,
                                       text=True, timeout=10).strip()
        if not re.fullmatch(r"[0-9a-f]{40}", head):
            raise ValueError("Git returned an invalid source commit")
        return head, frozen

    receipt = Path(receipt_value)
    expected = root / SOURCE_RECEIPT_NAME
    try:
        metadata = receipt.lstat()
    except OSError as error:
        raise ValueError("Source receipt is unavailable") from error
    if (receipt != expected or receipt.is_symlink() or not stat.S_ISREG(metadata.st_mode)
            or metadata.st_nlink != 1 or metadata.st_size > 4096
            or stat.S_IMODE(metadata.st_mode) & 0o222):
        raise ValueError("Source receipt is not the fixed private regular file")

    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError("Source receipt contains a duplicate key")
            result[key] = value
        return result

    try:
        data = json.loads(receipt.read_text(encoding="utf-8"), object_pairs_hook=unique)
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ValueError("Source receipt is not valid JSON") from error
    if (not isinstance(data, dict) or set(data) != {"schema", "source_commit", "source_sha256"}
            or type(data["schema"]) is not int or data["schema"] != 1
            or not isinstance(data["source_commit"], str)
            or not re.fullmatch(r"[0-9a-f]{40}", data["source_commit"])
            or data["source_sha256"] != frozen):
        raise ValueError("Source receipt does not bind the current source")
    return data["source_commit"], frozen


def run_logged(args: list[str], name: str, *, cwd: Path = ROOT, timeout: int = 180,
               extra_env: dict[str, str] | None = None) -> subprocess.CompletedProcess:
    started = time.time()
    env = environment()
    if extra_env:
        env.update(extra_env)
    args = [shutil.which(args[0], path=env["PATH"]) or args[0], *args[1:]]
    log = BUILD / "logs" / name
    log.parent.mkdir(parents=True, exist_ok=True)
    try:
        result = subprocess.run(args, cwd=cwd, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=timeout)
    except subprocess.TimeoutExpired as error:
        def partial_text(output):
            if isinstance(output, bytes):
                return output.decode("utf-8", "replace")
            return output or ""

        log.with_suffix(".stdout.txt").write_text(partial_text(error.stdout), encoding="utf-8")
        log.with_suffix(".stderr.txt").write_text(partial_text(error.stderr), encoding="utf-8")
        save_json(log.with_suffix(".json"), {
            "command": args, "cwd": str(cwd), "started_unix": started,
            "duration_s": time.time() - started, "returncode": None,
            "timed_out": True, "timeout_seconds": timeout,
        })
        raise
    log.with_suffix(".stdout.txt").write_text(result.stdout, encoding="utf-8")
    log.with_suffix(".stderr.txt").write_text(result.stderr, encoding="utf-8")
    save_json(log.with_suffix(".json"), {"command": args, "cwd": str(cwd), "started_unix": started, "duration_s": time.time()-started, "returncode": result.returncode})
    print(result.stdout, end="")
    print(result.stderr, end="")
    if result.returncode:
        raise subprocess.CalledProcessError(result.returncode, args)
    return result


def firmware() -> tuple[Path, Path]:
    code = os.environ.get("OS_OVMF_CODE")
    variables = os.environ.get("OS_OVMF_VARS")
    if bool(code) != bool(variables):
        raise RuntimeError("Set both OS_OVMF_CODE and OS_OVMF_VARS from the same build")
    if code:
        return Path(code).resolve(strict=True), Path(variables).resolve(strict=True)
    share = LOCAL / "qemu/share"
    descriptor = share / "firmware/60-edk2-x86_64.json"
    if descriptor.exists():
        mapping = json.loads(descriptor.read_text())["mapping"]
        # The upstream descriptor can contain its Linux packaging prefix.
        code = share / Path(mapping["executable"]["filename"]).name
        variables = share / Path(mapping["nvram-template"]["filename"]).name
        if code.is_file() and variables.is_file():
            return code, variables
    for folder in (BUILD / "firmware", Path("/usr/share/OVMF"), Path("/usr/share/edk2/ovmf")):
        for suffix in ("_4M", ""):
            code, variables = folder / f"OVMF_CODE{suffix}.fd", folder / f"OVMF_VARS{suffix}.fd"
            if code.is_file() and variables.is_file():
                return code, variables
    raise RuntimeError("No matching OVMF CODE/VARS pair found; set OS_OVMF_CODE and OS_OVMF_VARS")
