#!/usr/bin/env python3
"""Offline, fail-closed storage replay preparation; no acquisition or installation."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import signal
import stat
import subprocess
import tarfile
import time


ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/verification/native-storage-backend-gate"
BTRFS = {
    "btrfs-progs_6.17.1-1build1_amd64.deb":
        "89fb4fe8fce0d2ca42d2f26d626c7f46181ef6c7c8e2656ea1462579ec9b67e8",
}
ZFS = {
    "libnvpair3linux_2.4.1-1ubuntu5.1_amd64.deb":
        "6fd0fc246b0822e56deeabbb27c0cc2256c89ae5ba476b4462e77c97290af125",
    "libuutil3linux_2.4.1-1ubuntu5.1_amd64.deb":
        "cfe5b4903ac4651c419a06a099f50a28b5011e65a752b2aaa50b93ce8bf438ad",
    "libzfs7linux_2.4.1-1ubuntu5.1_amd64.deb":
        "6af5d55513678cd18d4b082638a72eee407dfa9a661cb492979c26d3a9e5a82c",
    "libzpool7linux_2.4.1-1ubuntu5.1_amd64.deb":
        "eb548340dd223f7430838108aec905278cbcaf5a744cbc890c5d6b513411a50f",
    "zfs-test_2.4.1-1ubuntu5.1_amd64.deb":
        "0e6f7b8c20b219d2b306f3e1b88495061e5866cade9cab30d57438086e508aeb",
    "zfsutils-linux_2.4.1-1ubuntu5.1_amd64.deb":
        "a3c347cf864319693dbab830a7759de03ddf394c84935c2c9496105ca2ac030f",
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(path):
    h = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def checked_path(path, *, private=False):
    """Reject links, foreign owners and writable ancestors before touching data."""
    path = Path(os.path.abspath(path))
    for part in reversed((path, *path.parents)):
        info = part.lstat()
        require(not stat.S_ISLNK(info.st_mode), f"symlink: {part}")
        require(info.st_uid == os.getuid(), f"foreign owner: {part}") if private else None
        if part != path or stat.S_ISDIR(info.st_mode):
            require(stat.S_ISDIR(info.st_mode), f"not directory: {part}")
            require(not info.st_mode & 0o022, f"writable ancestor: {part}")
        else:
            require(stat.S_ISREG(info.st_mode), f"not regular file: {part}")
            require(info.st_nlink == 1, f"hardlink: {part}")
        if private and part == path:
            require(not info.st_mode & 0o077, f"not private: {part}")
    return path


def fresh_workspace(path, project=ROOT):
    project = checked_path(project)
    path = Path(os.path.abspath(path))
    require(path.is_relative_to(project), "workspace must be inside this project")
    require(path != project, "workspace cannot be project root")
    checked_path(path.parent)
    require(path.parent.stat().st_uid == os.getuid(), "workspace parent owner")
    # mkdir without exist_ok refuses reuse, including dangling links.
    path.mkdir(mode=0o700)
    checked_path(path, private=False)
    require(stat.S_IMODE(path.stat().st_mode) == 0o700, "workspace must be private")
    for name in ("packages", "root", "logs", "images", "exports"):
        (path / name).mkdir(mode=0o700)
    return path


def private_file(path):
    path = checked_path(path)
    info = path.stat()
    require(info.st_uid == os.getuid(), f"foreign owner: {path}")
    require(not info.st_mode & 0o022, f"writable input: {path}")
    return path


def copy_checked(source, destination, expected):
    source = private_file(source)
    # O_NOFOLLOW protects the final open as well as the preceding lstat.
    fd = os.open(source, os.O_RDONLY | os.O_NOFOLLOW)
    with os.fdopen(fd, "rb") as src, Path(destination).open("xb") as dst:
        info = os.fstat(src.fileno())
        require(info.st_uid == os.getuid() and info.st_nlink == 1, "input changed")
        require(stat.S_ISREG(info.st_mode), "input is not regular")
        for block in iter(lambda: src.read(1024 * 1024), b""):
            dst.write(block)
    require(sha256(destination) == expected, f"SHA256 mismatch: {source.name}")


def run_logged(workspace, name, argv, *, timeout=60, expected=0, marker=None, env=None):
    """Own session/process group, exclusive logs and cleanup even on interruption."""
    require(name.replace("-", "").isalnum(), "invalid log name")
    stdout = workspace / "logs" / f"{name}.stdout.txt"
    stderr = workspace / "logs" / f"{name}.stderr.txt"
    record = {"argv": [str(x) for x in argv], "expected_exit": expected}
    start = time.monotonic()
    process = None
    try:
        with stdout.open("xb") as out, stderr.open("xb") as err:
            process = subprocess.Popen(
                record["argv"], cwd=workspace, env=env, stdout=out, stderr=err,
                stdin=subprocess.DEVNULL, start_new_session=True,
            )
            record["pid"] = process.pid
            record["process_group"] = process.pid
            try:
                record["exit_code"] = process.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                record["timed_out"] = True
                raise
    finally:
        if process is not None:
            # The group was created by this invocation, not discovered by name.
            # Kill descendants also when the leader exited first.
            try:
                os.killpg(process.pid, signal.SIGKILL)
                record["cleanup_signal"] = "SIGKILL-owned-process-group"
            except ProcessLookupError:
                record["cleanup_signal"] = "owned-process-group-already-exited"
            process.wait()
            record["leader_reaped"] = True
        record["elapsed_seconds"] = round(time.monotonic() - start, 6)
        with (workspace / "logs" / f"{name}.json").open("x") as stream:
            json.dump(record, stream, indent=2)
    require(record["exit_code"] == expected, f"{name}: unexpected exit {record['exit_code']}")
    if marker is not None:
        require(marker in stdout.read_bytes() + stderr.read_bytes(), f"{name}: missing output")
    return stdout.read_bytes(), stderr.read_bytes()


def member_path(name):
    path = PurePosixPath(name)
    require(not path.is_absolute() and ".." not in path.parts, f"tar path escape: {name}")
    return Path(*path.parts)


def unpack_tar(data, root):
    """Materialize internal package aliases as copies, never filesystem links."""
    files = {}
    links = {}
    modes = {}
    total = 0
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:") as archive:
        for member in archive:
            name = member_path(member.name)
            if name == Path("."):
                require(member.isdir(), "non-directory archive root")
                continue
            require(name not in files and name not in links, "duplicate tar member")
            if member.isdir():
                (root / name).mkdir(mode=0o700, parents=True, exist_ok=True)
            elif member.isfile():
                total += member.size
                require(total <= 256 * 1024 * 1024, "package expansion limit")
                files[name] = archive.extractfile(member).read()
                modes[name] = 0o700 if member.mode & 0o111 else 0o600
            elif member.issym() or member.islnk():
                target = member_path(member.linkname)
                links[name] = member_path(str(name.parent / target)) if member.issym() else target
            else:
                raise ValueError(f"unsupported tar member: {member.name}")
    while links:
        pending = dict(links)
        for name, target in pending.items():
            if target in files:
                files[name] = files[target]
                modes[name] = modes[target]
                del links[name]
        require(len(links) < len(pending), "unresolved or cyclic package link")
    for name, contents in files.items():
        target = root / name
        target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        if target.exists():
            require(target.is_file() and target.read_bytes() == contents, "package collision")
        else:
            with target.open("xb") as stream:
                stream.write(contents)
            target.chmod(modes[name])


def prepare(workspace, package_dir, backend):
    package_dir = checked_path(package_dir)
    require(package_dir.stat().st_uid == os.getuid(), "package directory owner")
    pins = BTRFS if backend == "btrfs" else ZFS
    require({p.name for p in package_dir.iterdir()} == set(pins), "package allowlist mismatch")
    # Check and copy the entire allowlist before parsing or extracting any package.
    for filename, expected in pins.items():
        copy_checked(package_dir / filename, workspace / "packages" / filename, expected)
    for index, filename in enumerate(pins):
        package = workspace / "packages" / filename
        fields, _ = run_logged(
            workspace, f"package-fields-{index}",
            ["/usr/bin/dpkg-deb", "--field", package, "Package", "Version", "Architecture"],
        )
        name, version, arch = filename.removesuffix(".deb").split("_")
        expected_fields = f"Package: {name}\nVersion: {version}\nArchitecture: {arch}\n".encode()
        require(fields == expected_fields, "package identity mismatch")
        data, _ = run_logged(
            workspace, f"package-tar-{index}", ["/usr/bin/dpkg-deb", "--fsys-tarfile", package],
        )
        unpack_tar(data, workspace / "root")
    with (workspace / "package-receipt.json").open("x") as stream:
        json.dump({"pins": pins, "backend": backend, "acquisition": False,
                   "installation": False, "foreign_execution": False}, stream, indent=2)


def preflight(workspace, binaries, env=None):
    for name, binary, help_args, status, marker in binaries:
        binary = private_file(binary)
        require(os.access(binary, os.X_OK), f"missing executable: {name}")
        output, error = run_logged(
            workspace, f"{name}-ldd", ["/usr/bin/ldd", binary], env=env,
        )
        require(b"not found" not in output + error, f"{name}: unresolved library")
        run_logged(workspace, f"{name}-help", [binary, *help_args],
                   expected=status, marker=marker, env=env)


def checked_image_copy(source, destination, expected):
    require(sha256(private_file(source)) == expected, "baseline image hash mismatch")
    copy_checked(source, destination, expected)
    require(sha256(source) == expected, "source changed during copy")
    return expected


def zdb_check_argv(workspace, image_dir):
    require(image_dir.is_relative_to(workspace / "images"), "not a project image directory")
    return [workspace / "root/usr/sbin/zdb", "-bcc", "-G", "-d", "-Y", "-e", "-y",
            "-p", image_dir, "nieuwos_n3"]


def package_evidence(workspace):
    destination = workspace / "evidence"
    destination.mkdir(mode=0o700)
    (destination / "images").mkdir(mode=0o700)
    crash = workspace / "images/crash"
    require(crash.is_dir(), "missing crash images")
    for path in crash.rglob("*"):
        private_file(path) if path.is_file() else checked_path(path)
        require(not path.is_symlink(), "linked crash artifact")
    with tarfile.open(destination / "images/crash-pool.tar.gz", "x:gz") as archive:
        archive.add(crash, arcname="crash")
    return destination


def execute_btrfs(workspace):
    btrfs = workspace / "root/usr/bin/btrfs"
    mkfs = workspace / "root/usr/sbin/mkfs.btrfs"
    preflight(workspace, [
        ("btrfs", btrfs, ["--help"], 0, b"usage:"),
        ("mkfs", mkfs, ["--help"], 0, b"usage:"),
    ])
    seed = workspace / "seed"
    seed.mkdir(mode=0o700)
    payload = hashlib.shake_256(b"nieuw-os-replay-v1").digest(1024 * 1024)
    (seed / "synthetic.bin").write_bytes(payload)
    image = workspace / "images/baseline.img"
    with image.open("xb") as stream:
        stream.truncate(256 * 1024 * 1024)
    run_logged(workspace, "mkfs", [mkfs, "--rootdir", seed, "-d", "single", "-m", "dup", image])
    baseline = sha256(image)
    run_logged(workspace, "check", [btrfs, "check", "--readonly", "--check-data-csum", image])
    restore = workspace / "restore"
    restore.mkdir(mode=0o700)
    run_logged(workspace, "restore", [btrfs, "restore", "-S", "-x", "-m", image, restore])
    require((restore / "synthetic.bin").read_bytes() == payload, "restored payload mismatch")
    require(sha256(image) == baseline, "read-only image changed")
    with (workspace / "result.json").open("x") as stream:
        json.dump({"scope": "host-baseline-only", "native_acceptance": False,
                   "baseline_sha256": baseline, "payload_matches": True}, stream, indent=2)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, required=True)
    parser.add_argument("--packages", type=Path, required=True)
    parser.add_argument("--backend", choices=("btrfs", "openzfs"), required=True)
    parser.add_argument("--execute", action="store_true", help="Btrfs host baseline only")
    args = parser.parse_args(argv)
    require(not (args.execute and args.backend == "openzfs"),
            "OpenZFS execution blocked: upstream ztest uses /tmp; isolated replay not implemented")
    old_umask = os.umask(0o077)
    try:
        workspace = fresh_workspace(args.workspace)
        prepare(workspace, args.packages, args.backend)
        if args.execute:
            execute_btrfs(workspace)
    finally:
        os.umask(old_umask)


if __name__ == "__main__":
    main()
