"""Shared host tooling. No global environment changes or guest network access."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
BUILD = Path(os.environ.get("NOS_BUILD_DIR", str(ROOT / "build"))).resolve()
TARGET = Path(os.environ.get("NOS_TARGET_DIR", str(ROOT / "target"))).resolve()
LOCAL = Path(os.environ.get("LOCALAPPDATA", str(ROOT / ".tools"))) / "nieuw-os"
QEMU_MACHINES = ("pc-q35-10.0", "pc-q35-8.2", "pc-q35-7.2")


def qemu_machine(value: str | None = None) -> str:
    """An explicit development profile, never an automatic machine fallback."""
    selected = os.environ.get("NOS_QEMU_MACHINE", "pc-q35-10.0") if value is None else value
    if selected not in QEMU_MACHINES:
        raise ValueError("Choose an explicitly supported and recorded QEMU machine")
    return selected


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
        sysroot = subprocess.run([rustc, "+1.90.0", "--print", "sysroot"], env=env, cwd=ROOT, text=True, capture_output=True, timeout=15)
        version = subprocess.run([rustc, "+1.90.0", "-vV"], env=env, cwd=ROOT, text=True, capture_output=True, timeout=15)
        if sysroot.returncode == version.returncode == 0:
            host = next(line.split(": ", 1)[1] for line in version.stdout.splitlines() if line.startswith("host: "))
            env["PATH"] = str(Path(sysroot.stdout.strip()) / "lib/rustlib" / host / "bin") + os.pathsep + env["PATH"]
    return env


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def save_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def source_hash() -> str:
    """Fingerprint source inputs without traversing generated artifacts or caches."""
    digest = hashlib.sha256()
    digest.update(b"nieuw-os source inputs v3\n")
    for folder in (ROOT / "boot", ROOT / "kernel", ROOT / "crates", ROOT / "drivers", ROOT / "services", ROOT / "userspace", ROOT / "sdk", ROOT / "experiments/quantum/qsim-core", ROOT / "experiments/quantum/native", ROOT / "experiments/quantum/job-protocol", ROOT / "tools", ROOT / ".cargo"):
        if folder.exists():
            for path in sorted(folder.rglob("*")):
                if path.is_file() and not {"__pycache__", "verification", "target", "build", ".git"}.intersection(path.relative_to(folder).parts):
                    digest.update(path.relative_to(ROOT).as_posix().encode())
                    digest.update(path.read_bytes())
    for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "experiments/quantum/Cargo.toml"):
        if (ROOT / name).exists():
            digest.update(name.encode())
            digest.update((ROOT / name).read_bytes())
    return digest.hexdigest()


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
