"""Host unit controls; mocked launches do not establish namespace isolation."""
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import agent_sandbox as sandbox
from common import isolated_environment


class SandboxTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name)
        self.rootfs = self.directory / "runtime"
        self.rootfs.mkdir(mode=0o755)
        for name in ("dev", "proc", "sys", "tmp", "run", "home", "root", "workspace", "usr"):
            (self.rootfs / name).mkdir(mode=0o755)
        (self.rootfs / "usr/bin").mkdir(mode=0o755)
        self.tool = self.rootfs / "usr/bin/tool"
        self.tool.write_bytes(b"runtime fixture\n")
        self.tool.chmod(0o755)
        (self.rootfs / "bin").symlink_to("usr/bin")
        self.scratch_root = self.directory / "scratch"
        self.scratch_root.mkdir(mode=0o700)
        self.scratch_root.chmod(0o700)
        self.container = self.scratch_root / "disposable"
        self.container.mkdir(mode=0o700)
        for name in ("workspace", "tmp", "run", "home", "root"):
            (self.container / name).mkdir(mode=0o700)
        (self.container / "home" / "agent").mkdir(mode=0o700)
        self.workspace = self.container / "workspace"
        self.backend = self.directory / "backend"
        self.backend.write_bytes(b"pinned backend fixture\n")
        self.backend.chmod(0o755)
        self.manager = self.directory / "systemd-run"
        self.manager.write_bytes(b"pinned resource manager fixture\n")
        self.manager.chmod(0o755)
        self.control = self.directory / "systemctl"
        self.control.write_bytes(b"pinned resource control fixture\n")
        self.control.chmod(0o755)
        profile = {
            "memory_high_bytes": 32 * 1024 * 1024,
            "memory_max_bytes": 64 * 1024 * 1024,
            "tasks_max": 16,
            "cpu_quota_percent": 100,
            "output_max_bytes": 1024 * 1024,
            "open_files_max": 256,
        }
        self.config = {
            "backend": str(self.backend),
            "backend_sha256": hashlib.sha256(self.backend.read_bytes()).hexdigest(),
            "rootfs": str(self.rootfs),
            "rootfs_sha256": sandbox.tree_digest(self.rootfs),
            "resource_manager": str(self.manager),
            "resource_manager_sha256": hashlib.sha256(self.manager.read_bytes()).hexdigest(),
            "resource_control": str(self.control),
            "resource_control_sha256": hashlib.sha256(self.control.read_bytes()).hexdigest(),
            "timeout_seconds": 60,
            "profiles": {"agent": dict(profile), "gate": dict(profile)},
            "scratch": {
                "roots": [str(self.scratch_root)],
                "capacity_bytes": 128 * 1024 * 1024,
                "capacity_inodes": 8192,
                "source_max_bytes": 64 * 1024 * 1024,
                "source_max_files": 4096,
                "export_max_bytes": 16 * 1024 * 1024,
                "export_max_files": 128,
            },
        }
        self.backend_version = subprocess.CompletedProcess([], 0, "bubblewrap 0.13.1\n", "")
        self.manager_version = subprocess.CompletedProcess([], 0, "systemd 259 (259.9)\n", "")
        self.scope_states = []

        def command(argv, **_options):
            name = argv[7]
            if name == "bwrap":
                result = self.backend_version
            elif name == "systemd-run":
                result = self.manager_version
            elif name == "systemctl" and "show" in argv:
                state = self.scope_states.pop(0) if self.scope_states else {
                    "LoadState": "not-found", "ActiveState": "inactive", "ControlGroup": "",
                }
                result = subprocess.CompletedProcess(argv, 0, "".join(f"{key}={value}\n" for key, value in state.items()), "")
            elif name == "systemctl":
                result = subprocess.CompletedProcess(argv, 0, "", "")
            else:
                raise AssertionError(argv)
            if isinstance(result, BaseException):
                raise result
            return result

        self.version = self.enterContext(mock.patch.object(sandbox.subprocess, "run", side_effect=command))
        self.launch = self.enterContext(mock.patch.object(sandbox.subprocess, "Popen"))
        self.kill = self.enterContext(mock.patch.object(sandbox.os, "killpg"))
        self.resource_env = self.enterContext(mock.patch.object(
            sandbox, "_resource_environment", return_value={
                "PATH": "/usr/bin:/bin", "LANG": "C.UTF-8", "XDG_RUNTIME_DIR": str(self.directory),
                "DBUS_SESSION_BUS_ADDRESS": f"unix:path={self.directory / 'bus'}",
            }))
        self.communication = self.enterContext(mock.patch.object(sandbox, "_communicate_bounded"))
        self.receipt = self.enterContext(mock.patch.object(
            sandbox, "_guard_receipt", return_value={"memory_max": str(64 * 1024 * 1024)}))
        self.scratch_verification = self.enterContext(mock.patch.object(
            sandbox.Sandbox, "_verify_scratch_root", return_value={
                "mount_id": "fixture", "device": "0:1",
                "capacity_bytes": 128 * 1024 * 1024, "capacity_inodes": 8192,
            }))

    def lease_for(self, instance, path=None):
        path = self.workspace if path is None else path
        descriptor = os.open(self.scratch_root, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC)
        self.addCleanup(os.close, descriptor)
        token = f"fixture-{len(instance._leases)}"
        lease = sandbox.ScratchLease(
            instance, self.scratch_root, self.container, descriptor, token,
            {"mount_id": "fixture", "device": "0:1",
             "capacity_bytes": 128 * 1024 * 1024, "capacity_inodes": 8192})
        if path != self.workspace:
            lease.path = path
            lease._identity = (path.stat().st_dev, path.stat().st_ino)
        instance._leases[token] = lease
        return lease

    def remove_fixture_container(self):
        for path in sorted(self.container.rglob("*"), reverse=True):
            path.rmdir()
        self.container.rmdir()

    def assert_quarantined(self, lease):
        self.assertTrue(lease.receipt["quarantined"])
        self.assertFalse(lease.receipt["cleaned"])
        self.assertTrue(lease.quarantine_reasons)

    def run_sandbox(self, argv, *, instance=None, path=None, **options):
        instance = sandbox.Sandbox(self.config) if instance is None else instance
        return instance.run(argv, self.lease_for(instance, path), **options)

    def process(self, code=0, stdout="output\n", stderr=""):
        process = mock.MagicMock()
        process.pid = 123456789
        process.returncode = code
        process.wait.return_value = code
        process.poll.return_value = code
        self.launch.return_value = process
        self.communication.return_value = (stdout.encode(), stderr.encode(), False, False)
        self.scope_states = [
            {
                "LoadState": "loaded", "ActiveState": "inactive", "SubState": "dead",
                "Result": "success", "ExecMainCode": "1", "ExecMainStatus": str(code),
                "ControlGroup": "", "MemoryHigh": str(32 * 1024 * 1024),
                "MemoryMax": str(64 * 1024 * 1024), "MemorySwapMax": "0",
                "TasksMax": "16", "CPUQuotaPerSecUSec": "1s", "CPUQuotaPeriodUSec": "100ms",
            },
            {"LoadState": "not-found", "ActiveState": "inactive", "ControlGroup": ""},
        ]
        return process

    def test_explicit_complete_configuration_and_bounded_timeout_are_required(self):
        malformed = (None, {}, {**self.config, "extra": True},
                     {**self.config, "backend_sha256": "bad"},
                     {**self.config, "rootfs_sha256": 123},
                     {**self.config, "resource_manager_sha256": "bad"},
                     {**self.config, "resource_control_sha256": "bad"},
                     {**self.config, "profiles": {}})
        for config in malformed:
            with self.subTest(config=config), self.assertRaises(sandbox.SandboxError):
                sandbox.Sandbox(config)
        for timeout in (0, -1, 7201, True, 1.5, "60"):
            with self.subTest(timeout=timeout), self.assertRaises(sandbox.SandboxError):
                sandbox.Sandbox({**self.config, "timeout_seconds": timeout})
        for field, value in (
            ("memory_high_bytes", 1), ("memory_max_bytes", 1), ("tasks_max", True),
            ("cpu_quota_percent", 801), ("output_max_bytes", 1), ("open_files_max", 8),
        ):
            profiles = {name: {**profile, field: value}
                        for name, profile in self.config["profiles"].items()}
            with self.subTest(field=field), self.assertRaises(sandbox.SandboxError):
                sandbox.Sandbox({**self.config, "profiles": profiles})
        self.version.assert_not_called()
        self.launch.assert_not_called()

    def test_scratch_policy_requires_finite_distinct_caps_and_roots(self):
        policy = self.config["scratch"]
        second = self.directory / "scratch-two"
        second.mkdir(mode=0o700)
        nested = self.scratch_root / "nested"
        nested.mkdir(mode=0o700)
        malformed = (
            None,
            {},
            {**policy, "extra": 1},
            {**policy, "roots": []},
            {**policy, "roots": [str(self.scratch_root)] * 2},
            {**policy, "roots": [str(self.scratch_root), str(nested)]},
            {**policy, "roots": [str(self.scratch_root), str(second)] * 2},
            {**policy, "capacity_bytes": True},
            {**policy, "capacity_bytes": 1024},
            {**policy, "capacity_inodes": 1},
            {**policy, "source_max_bytes": policy["capacity_bytes"] + 1},
            {**policy, "source_max_files": policy["capacity_inodes"] + 1},
            {**policy, "export_max_bytes": policy["capacity_bytes"] + 1},
            {**policy, "export_max_files": policy["capacity_inodes"] + 1},
            {**policy, "roots": [str(self.rootfs)]},
        )
        for scratch in malformed:
            with self.subTest(scratch=scratch), self.assertRaises(sandbox.SandboxError):
                sandbox.Sandbox({**self.config, "scratch": scratch})
        self.version.assert_not_called()
        self.launch.assert_not_called()

    def test_mount_identity_type_and_exact_caps_are_required(self):
        descriptor = os.open(self.scratch_root, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC)
        self.addCleanup(os.close, descriptor)
        metadata = os.fstat(descriptor)
        device = f"{os.major(metadata.st_dev)}:{os.minor(metadata.st_dev)}"
        filesystem = mock.Mock(
            f_blocks=(128 * 1024 * 1024) // 4096,
            f_frsize=4096,
            f_files=8192,
            f_flag=0,
        )

        def metadata_text(path, *_args, **_kwargs):
            if str(path).startswith("/proc/self/fdinfo/"):
                return "mnt_id:\t77\n"
            if path == Path("/proc/self/mountinfo"):
                return (f"77 1 {device} / {self.scratch_root} rw,nosuid - "
                        "tmpfs tmpfs rw,size=131072k,nr_inodes=8192\n")
            raise AssertionError(path)

        with mock.patch.object(Path, "read_text", autospec=True, side_effect=metadata_text), \
                mock.patch.object(sandbox.os, "fstatvfs", return_value=filesystem):
            receipt = sandbox._scratch_mount(
                descriptor, self.scratch_root, 128 * 1024 * 1024, 8192)
        self.assertEqual((receipt["mount_id"], receipt["device"]), ("77", device))

        cases = (
            ("missing", "position:\t0\n", ""),
            ("non-tmpfs", "mnt_id:\t77\n",
             f"77 1 {device} / {self.scratch_root} rw - ext4 /dev/x rw\n"),
            ("wrong-root", "mnt_id:\t77\n",
             f"77 1 {device} /sub {self.scratch_root} rw - tmpfs tmpfs rw\n"),
            ("wrong-mountpoint", "mnt_id:\t77\n",
             f"77 1 {device} / {self.directory} rw - tmpfs tmpfs rw\n"),
        )
        for name, fdinfo, mountinfo in cases:
            def invalid_text(path, *_args, **_kwargs):
                return fdinfo if str(path).startswith("/proc/self/fdinfo/") else mountinfo
            with self.subTest(name=name), \
                    mock.patch.object(Path, "read_text", autospec=True, side_effect=invalid_text), \
                    mock.patch.object(sandbox.os, "fstatvfs", return_value=filesystem), \
                    self.assertRaises(sandbox.SandboxError):
                sandbox._scratch_mount(descriptor, self.scratch_root, 128 * 1024 * 1024, 8192)

        for field, value in (("f_blocks", filesystem.f_blocks - 1), ("f_files", 8191)):
            changed = mock.Mock(**{
                "f_blocks": filesystem.f_blocks,
                "f_frsize": 4096,
                "f_files": 8192,
                "f_flag": 0,
                field: value,
            })
            with self.subTest(field=field), \
                    mock.patch.object(Path, "read_text", autospec=True, side_effect=metadata_text), \
                    mock.patch.object(sandbox.os, "fstatvfs", return_value=changed), \
                    self.assertRaises(sandbox.SandboxError):
                sandbox._scratch_mount(descriptor, self.scratch_root, 128 * 1024 * 1024, 8192)
        self.launch.assert_not_called()

    def test_lease_uses_an_unlocked_pool_root_and_cleans_after_errors(self):
        self.remove_fixture_container()
        second = self.directory / "scratch-two"
        second.mkdir(mode=0o700)
        second.chmod(0o700)
        config = {**self.config, "scratch": {
            **self.config["scratch"],
            "roots": [str(self.scratch_root), str(second)],
        }}
        instance = sandbox.Sandbox(config)
        held = os.open(self.scratch_root, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC)
        self.addCleanup(os.close, held)
        sandbox.fcntl.flock(held, sandbox.fcntl.LOCK_EX | sandbox.fcntl.LOCK_NB)
        with instance.lease() as normal:
            self.assertEqual(normal.root, second)
            (normal.path / "output").write_text("bounded")
        self.assertFalse(normal.active)
        self.assertTrue(normal.receipt["cleaned"])
        self.assertEqual(list(second.iterdir()), [])
        with self.assertRaisesRegex(RuntimeError, "body failed"):
            with instance.lease() as lease:
                self.assertEqual(lease.root, second)
                (lease.path / "output").write_text("bounded")
                raise RuntimeError("body failed")
        self.assertFalse(lease.active)
        self.assertTrue(lease.receipt["cleaned"])
        self.assertEqual(list(second.iterdir()), [])

    def test_dirty_quarantined_root_is_skipped_but_invalid_mount_aborts(self):
        self.remove_fixture_container()
        second = self.directory / "scratch-two"
        second.mkdir(mode=0o700)
        second.chmod(0o700)
        config = {**self.config, "scratch": {
            **self.config["scratch"],
            "roots": [str(self.scratch_root), str(second)],
        }}
        instance = sandbox.Sandbox(config)
        with mock.patch.object(sandbox.shutil, "rmtree", side_effect=OSError("fixture cleanup")), \
                self.assertRaisesRegex(sandbox.SandboxError, "scratch cleanup failed"):
            with instance.lease() as lease:
                (lease.path / "output").write_text("bounded")
        self.assertFalse(lease.active)
        self.assert_quarantined(lease)
        self.assertTrue(lease.path.exists())

        with instance.lease() as later:
            self.assertEqual(later.root, second)
        self.assertTrue(later.receipt["cleaned"])

        with mock.patch.object(
                instance, "_verify_scratch_root",
                side_effect=sandbox.SandboxError("invalid mount identity")) as verification, \
                self.assertRaisesRegex(sandbox.SandboxError, "invalid mount identity"):
            with instance.lease():
                pass
        verification.assert_called_once()

    def test_body_exception_remains_primary_when_lease_cleanup_fails(self):
        self.remove_fixture_container()
        instance = sandbox.Sandbox(self.config)
        with mock.patch.object(sandbox.shutil, "rmtree", side_effect=OSError("fixture cleanup")), \
                self.assertRaisesRegex(RuntimeError, "primary body failure") as raised:
            with instance.lease() as lease:
                raise RuntimeError("primary body failure")
        self.assert_quarantined(lease)
        details = "\n".join(getattr(raised.exception, "__notes__", ()))
        details += f"\n{raised.exception.__cause__!r}\n{raised.exception.__context__!r}"
        self.assertIn("fixture cleanup", details)

    def test_run_rejects_arbitrary_foreign_and_inactive_leases(self):
        instance = sandbox.Sandbox(self.config)
        foreign = sandbox.Sandbox(self.config)
        lease = self.lease_for(instance)
        for value in (self.workspace, lease):
            if value is lease:
                lease.active = False
            with self.subTest(value=type(value).__name__), \
                    self.assertRaisesRegex(sandbox.SandboxError, "active owned scratch lease"):
                instance.run(["tool"], value, profile="agent")
        lease.active = True
        with self.assertRaisesRegex(sandbox.SandboxError, "active owned scratch lease"):
            foreign.run(["tool"], lease, profile="agent")
        self.version.assert_not_called()
        self.launch.assert_not_called()

    def test_non_linux_is_unavailable(self):
        with mock.patch.object(sandbox.sys, "platform", "win32"), self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox(self.config)
        self.version.assert_not_called()

    def test_absent_relative_or_symlink_backend_never_executes(self):
        link = self.directory / "alias"
        link.symlink_to(self.backend)
        for value in ("backend", str(link), str(self.directory / "missing")):
            with self.subTest(value=value), self.assertRaises(sandbox.SandboxError):
                sandbox.Sandbox({**self.config, "backend": value}).verify()
        self.version.assert_not_called()
        self.launch.assert_not_called()

    def test_symlink_parent_directory_is_forbidden(self):
        alias = self.directory / "runtime-alias"
        alias.symlink_to(self.rootfs, target_is_directory=True)
        with self.assertRaises(sandbox.SandboxError):
            sandbox.tree_digest(alias / "usr")

    def test_backend_hash_and_permissions_are_checked_before_execution(self):
        self.backend.write_bytes(b"different fixture\n")
        with self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox(self.config).verify()
        for mode in (0o644, 0o777, 0o4755, 0o2755):
            self.backend.chmod(mode)
            with self.subTest(mode=mode), self.assertRaises(sandbox.SandboxError):
                sandbox.Sandbox(self.config).verify()
        self.version.assert_not_called()
        self.launch.assert_not_called()

    def test_backend_fifo_and_hardlink_are_rejected(self):
        self.backend.unlink()
        os.mkfifo(self.backend)
        with self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox(self.config).verify()
        self.backend.unlink()
        self.backend.write_bytes(b"pinned backend fixture\n")
        self.backend.chmod(0o755)
        os.link(self.backend, self.directory / "second-name")
        with self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox(self.config).verify()
        self.version.assert_not_called()

    def test_backend_capabilities_are_rejected(self):
        original = os.getxattr

        def capabilities(path, name):
            return b"capability-canary" if isinstance(path, int) else original(path, name)

        with mock.patch.object(sandbox.os, "getxattr", side_effect=capabilities), \
                self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox(self.config).verify()
        self.version.assert_not_called()

    def test_unverifiable_capabilities_are_rejected(self):
        with mock.patch.object(sandbox.os, "getxattr", side_effect=PermissionError()), \
                self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox(self.config).verify()
        self.version.assert_not_called()

    def test_checked_executable_inodes_use_a_minimal_launcher_environment(self):
        descriptors = []

        def version(argv, **kwargs):
            descriptor = kwargs["pass_fds"][0]
            descriptors.append(descriptor)
            name = argv[7]
            expected = self.backend if name == "bwrap" else self.manager
            self.assertEqual(os.fstat(descriptor).st_ino, expected.stat().st_ino)
            self.assertEqual(argv[:5], ["/usr/bin/python3", "-I", "-S", "-B", "-c"])
            self.assertEqual(argv[6:], [str(descriptor), name, "--version"])
            self.assertIn("os.set_inheritable(fd,False)", argv[5])
            self.assertIn("os.execve(fd,", argv[5])
            self.assertEqual(kwargs["env"], {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8"})
            self.assertEqual(kwargs["stdin"], subprocess.DEVNULL)
            output = "bubblewrap 0.13.1\n" if name == "bwrap" else "systemd 259 (259.9)\n"
            return subprocess.CompletedProcess(argv, 0, output, "")

        self.version.side_effect = version
        sandbox.Sandbox(self.config).verify()
        self.assertEqual(len(descriptors), 2)
        for descriptor in descriptors:
            with self.assertRaises(OSError):
                os.fstat(descriptor)
        self.launch.assert_not_called()

    def test_legacy_prerelease_malformed_or_failed_backend_is_unavailable(self):
        for output, code in (("bubblewrap 0.13.0\n", 0), ("bubblewrap 0.13.1-dev\n", 0),
                             ("bubblewrap 0.13.1\nextra\n", 0), ("other 99.1.1\n", 0),
                             ("bubblewrap 0.13.1\n", 1)):
            self.backend_version = subprocess.CompletedProcess([], code, output, "diagnostic")
            with self.subTest(output=output), self.assertRaises(sandbox.SandboxError):
                sandbox.Sandbox(self.config).verify()
        self.launch.assert_not_called()

    def test_backend_version_timeout_is_a_closed_failure(self):
        self.backend_version = subprocess.TimeoutExpired("backend", 10)
        with self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox(self.config).verify()
        self.launch.assert_not_called()

    def test_runtime_digest_binds_contents_names_modes_and_relative_aliases(self):
        first = sandbox.tree_digest(self.rootfs)
        self.assertEqual(first, sandbox.tree_digest(self.rootfs))
        self.tool.write_bytes(b"changed fixture\n")
        second = sandbox.tree_digest(self.rootfs)
        self.assertNotEqual(first, second)
        self.tool.chmod(0o744)
        third = sandbox.tree_digest(self.rootfs)
        self.assertNotEqual(second, third)
        self.tool.rename(self.tool.with_name("other"))
        self.assertNotEqual(third, sandbox.tree_digest(self.rootfs))

    def test_rootfs_change_is_rejected_before_backend_execution(self):
        self.tool.write_bytes(b"changed fixture\n")
        with self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox(self.config).verify()
        self.version.assert_not_called()

    def test_rootfs_special_files_hardlinks_and_privileged_modes_are_forbidden(self):
        extra = self.rootfs / "extra"
        os.mkfifo(extra)
        with self.assertRaises(sandbox.SandboxError):
            sandbox.tree_digest(self.rootfs)
        extra.unlink()
        os.link(self.tool, extra)
        with self.assertRaises(sandbox.SandboxError):
            sandbox.tree_digest(self.rootfs)
        extra.unlink()
        for mode in (0o777, 0o4755):
            self.tool.chmod(mode)
            with self.subTest(mode=mode), self.assertRaises(sandbox.SandboxError):
                sandbox.tree_digest(self.rootfs)

    def test_rootfs_absolute_escaping_dangling_and_looping_links_are_forbidden(self):
        outside = self.directory / "outside-canary"
        outside.write_text("generic boundary canary\n")
        link = self.rootfs / "extra"
        for target in (str(self.tool), "../outside-canary", "missing", "extra"):
            link.symlink_to(target)
            with self.subTest(target=target), self.assertRaises(sandbox.SandboxError):
                sandbox.tree_digest(self.rootfs)
            link.unlink()

    def test_host_root_and_nonempty_or_aliased_mountpoints_are_forbidden(self):
        with self.assertRaises(sandbox.SandboxError):
            sandbox.tree_digest(Path("/"))
        (self.rootfs / "home/file").write_text("mountpoint canary")
        (self.rootfs / "home/file").chmod(0o644)
        config = {**self.config, "rootfs_sha256": sandbox.tree_digest(self.rootfs)}
        with self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox(config).verify()
        (self.rootfs / "home/file").unlink()
        (self.rootfs / "home").rmdir()
        (self.rootfs / "home").symlink_to("usr")
        config = {**self.config, "rootfs_sha256": sandbox.tree_digest(self.rootfs)}
        with self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox(config).verify()
        self.version.assert_not_called()

    def test_git_metadata_cannot_be_mounted(self):
        for nested in (False, True):
            parent = self.workspace / "nested" if nested else self.workspace
            parent.mkdir(exist_ok=True)
            metadata = parent / ".git"
            metadata.write_text("generic metadata canary")
            with self.subTest(nested=nested), self.assertRaises(sandbox.SandboxError):
                self.run_sandbox(["tool"], profile="agent")
            metadata.unlink()
        self.launch.assert_not_called()
        self.version.assert_not_called()

    def test_workspace_may_not_contain_external_links_or_special_files(self):
        link = self.workspace / "outside"
        link.symlink_to(self.tool)
        with self.assertRaises(sandbox.SandboxError):
            self.run_sandbox(["tool"], profile="agent")
        link.unlink()
        os.link(self.tool, link)
        with self.assertRaises(sandbox.SandboxError):
            self.run_sandbox(["tool"], profile="agent")
        link.unlink()
        os.mkfifo(link)
        with self.assertRaises(sandbox.SandboxError):
            self.run_sandbox(["tool"], profile="agent")
        self.launch.assert_not_called()

    def test_overlapping_workspace_is_rejected_before_execution(self):
        for path in (self.rootfs, self.rootfs / "usr", self.directory, Path("/")):
            with self.subTest(path=path), self.assertRaises(sandbox.SandboxError):
                self.run_sandbox(["tool"], path=path, profile="agent")
        self.version.assert_not_called()
        self.launch.assert_not_called()

    def test_command_environment_and_timeout_validation_precede_execution(self):
        instance = sandbox.Sandbox(self.config)
        lease = self.lease_for(instance)
        for command in ([], "tool", [""], ["tool", "\0"], ["tool", 1]):
            with self.subTest(command=command), self.assertRaises(sandbox.SandboxError):
                instance.run(command, lease, profile="agent")
        for timeout in (0, True, 61, "5"):
            with self.subTest(timeout=timeout), self.assertRaises(sandbox.SandboxError):
                instance.run(["tool"], lease, profile="agent", timeout=timeout)
        for env in ([], ["invalid"], {"SSH_AUTH_SOCK": "/run/canary"}, {"LD_PRELOAD": "canary"},
                    {"PYTHONPATH": "/workspace"}, {"HTTPS_PROXY": "canary"}, {"HOME": "/root"}):
            with self.subTest(env=env), self.assertRaises(sandbox.SandboxError):
                instance.run(["tool"], lease, profile="agent", env=env)
        self.version.assert_not_called()
        self.launch.assert_not_called()

    def test_transaction_output_limit_accepts_zero_to_profile_cap_only(self):
        instance = sandbox.Sandbox(self.config)
        lease = self.lease_for(instance)
        profile_cap = self.config["profiles"]["agent"]["output_max_bytes"]
        for value in (4096, profile_cap):
            self.process()
            result = instance.run(["tool"], lease, profile="agent", output_limit=value)
            self.assertEqual(result.returncode, 0)
            self.assertEqual(self.communication.call_args.args[1], value)

        self.process()
        self.communication.return_value = (b"", b"", True, False)
        with self.assertRaises(sandbox.SandboxResourceExceeded):
            instance.run(["tool"], lease, profile="agent", output_limit=0)
        self.assertEqual(self.communication.call_args.args[1], 0)

        self.version.reset_mock()
        self.launch.reset_mock()
        for value in (-1, True, 1.5, "4096", profile_cap + 1):
            with self.subTest(output_limit=value), self.assertRaises(sandbox.SandboxError):
                instance.run(["tool"], lease, profile="agent", output_limit=value)
        self.version.assert_not_called()
        self.launch.assert_not_called()

    def test_mounts_namespaces_environment_and_raw_output_are_preserved(self):
        process = self.process(code=33, stdout="raw output\n", stderr="raw diagnostic\n")
        extra = {"NOS_BUILD_DIR": "/workspace/build", "MACHINE": "q35"}
        with mock.patch.dict(os.environ, {"PRIVATE_CANARY": "absent", "SSH_AUTH_SOCK": "/host/socket"}):
            result = self.run_sandbox(["tool", "arg"], profile="agent", env=extra, timeout=7)
        self.assertEqual((result.args, result.returncode, result.stdout, result.stderr),
                         (["tool", "arg"], 33, "raw output\n", "raw diagnostic\n"))
        command = self.launch.call_args.args[0]
        options = self.launch.call_args.kwargs
        for flag in ("--unshare-all", "--unshare-user", "--unshare-cgroup", "--die-with-parent", "--new-session",
                     "--disable-userns", "--assert-userns-disabled", "--clearenv"):
            self.assertIn(flag, command)
        self.assertEqual(command[command.index("--cap-drop") + 1], "ALL")
        self.assertEqual(command[command.index("--ro-bind") + 1:command.index("--ro-bind") + 3],
                         [str(self.rootfs), "/"])
        binds = [command[index + 1:index + 3]
                 for index, item in enumerate(command) if item == "--bind"]
        self.assertEqual([pair[1] for pair in binds],
                         ["/dev/shm", "/tmp", "/run", "/home", "/root", "/workspace"])
        sessions = {Path(pair[0]).parent for pair in binds[:5]}
        self.assertEqual(len(sessions), 1)
        self.assertEqual(next(iter(sessions)).parent, self.container)
        self.assertTrue(next(iter(sessions)).name.startswith("session-"))
        self.assertEqual(binds[-1][0], str(self.workspace))
        readonly_mounts = [command[index + 1]
                           for index, item in enumerate(command) if item == "--remount-ro"]
        self.assertEqual(readonly_mounts, ["/dev", "/dev/pts"])
        shm = next(index for index, item in enumerate(command)
                   if item == "--bind" and command[index + 2] == "/dev/shm")
        self.assertLess(command.index("--dev"), command.index("--remount-ro"))
        self.assertLess(max(index for index, item in enumerate(command)
                            if item == "--remount-ro"), shm)
        self.assertNotIn("--tmpfs", command)
        environment = {command[index + 1]: command[index + 2]
                       for index, item in enumerate(command) if item == "--setenv"}
        self.assertEqual(environment, isolated_environment(extra))
        self.assertEqual(options["env"], self.resource_env.return_value)
        self.assertEqual(options["stdin"], subprocess.DEVNULL)
        self.assertTrue(options["close_fds"])
        self.assertTrue(options["start_new_session"])
        self.assertEqual(options["umask"], 0o022)
        self.assertEqual(result.stdout_bytes, b"raw output\n")
        self.assertEqual(len(options["pass_fds"]), 3)
        process.wait.assert_called_once_with(timeout=5)
        self.assertEqual(result.resource_status["profile"], "agent")
        self.assertEqual(result.resource_status["verified"]["memory_max"], str(64 * 1024 * 1024))
        for property_value in (
            "MemoryHigh=33554432", "MemoryMax=67108864", "MemorySwapMax=0",
            "TasksMax=16", "CPUQuota=100%", "KillMode=control-group",
        ):
            self.assertIn(f"--property={property_value}", command)
        self.assertNotIn("--property=MemoryOOMGroup=yes", command)
        self.kill.assert_not_called()

    def test_namespace_failure_has_no_ambient_fallback_and_keeps_failure_status(self):
        self.process(code=1, stderr="namespace unavailable\n")
        result = self.run_sandbox(["tool"], profile="agent")
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stderr, "namespace unavailable\n")
        self.launch.assert_called_once()

    def test_timeout_kills_the_resource_scope_and_preserves_partial_output(self):
        self.process()
        def timed_out(_process, _limit, _timeout, terminate):
            terminate()
            return b"partial output\n", b"partial diagnostic\n", False, True
        self.communication.side_effect = timed_out
        self.scope_states[0]["ControlGroup"] = "/fixture.scope"
        with mock.patch.object(sandbox.Sandbox, "_scope_populated", return_value=True), \
                self.assertRaises(subprocess.TimeoutExpired) as raised:
            self.run_sandbox(["tool"], profile="agent", timeout=2)
        self.assertEqual((raised.exception.cmd, raised.exception.output, raised.exception.stderr),
                         (["tool"], "partial output\n", "partial diagnostic\n"))
        controls = [call.args[0] for call in self.version.call_args_list
                    if call.args[0][7] == "systemctl"]
        self.assertTrue(any("kill" in command and "--kill-whom=all" in command for command in controls))
        self.kill.assert_not_called()

    def test_pipe_failure_still_kills_and_releases_the_complete_scope(self):
        process = self.process()
        process.poll.return_value = None
        self.communication.side_effect = OSError("pipe read failed")
        self.scope_states[0]["ControlGroup"] = "/fixture.scope"
        with mock.patch.object(sandbox.Sandbox, "_scope_populated", return_value=True), \
                self.assertRaisesRegex(OSError, "pipe read failed"):
            self.run_sandbox(["tool"], profile="agent")
        controls = [call.args[0] for call in self.version.call_args_list
                    if call.args[0][7] == "systemctl"]
        self.assertGreaterEqual(sum("kill" in command for command in controls), 2)
        self.assertTrue(any("reset-failed" in command for command in controls))

    def test_keyboard_interrupt_still_kills_and_releases_the_complete_scope(self):
        process = self.process()
        process.poll.return_value = None
        self.communication.side_effect = KeyboardInterrupt()
        self.scope_states[0]["ControlGroup"] = "/fixture.scope"
        with mock.patch.object(sandbox.Sandbox, "_scope_populated", return_value=True), \
                self.assertRaises(KeyboardInterrupt):
            self.run_sandbox(["tool"], profile="agent")
        controls = [call.args[0] for call in self.version.call_args_list
                    if call.args[0][7] == "systemctl"]
        self.assertGreaterEqual(sum("kill" in command for command in controls), 2)
        self.assertTrue(any("reset-failed" in command for command in controls))

    def test_resource_cleanup_uncertainty_quarantines_the_active_lease(self):
        self.process()
        instance = sandbox.Sandbox(self.config)
        lease = self.lease_for(instance)
        with mock.patch.object(
                instance, "_cleanup_scope",
                side_effect=sandbox.SandboxError("cgroup cleanup uncertain")), \
                self.assertRaisesRegex(sandbox.SandboxError, "resource cleanup failed"):
            instance.run(["tool"], lease, profile="agent")
        self.assert_quarantined(lease)

    def test_output_overflow_remains_primary_when_resource_cleanup_is_uncertain(self):
        self.process()
        self.communication.return_value = (b"bounded output", b"bounded diagnostic", True, False)
        instance = sandbox.Sandbox(self.config)
        lease = self.lease_for(instance)
        with mock.patch.object(
                instance, "_cleanup_scope",
                side_effect=sandbox.SandboxError("cgroup cleanup uncertain")), \
                self.assertRaises(sandbox.SandboxResourceExceeded) as raised:
            instance.run(["tool"], lease, profile="agent")
        self.assertEqual(raised.exception.stdout_bytes, b"bounded output")
        self.assertEqual(raised.exception.stderr_bytes, b"bounded diagnostic")
        self.assertIn("resource cleanup failed", "\n".join(raised.exception.__notes__))
        self.assertIsInstance(raised.exception.__cause__, sandbox.SandboxError)
        self.assert_quarantined(lease)

    def test_timeout_remains_primary_when_resource_cleanup_is_uncertain(self):
        self.process()
        self.communication.return_value = (b"bounded output", b"bounded diagnostic", False, True)
        instance = sandbox.Sandbox(self.config)
        lease = self.lease_for(instance)
        with mock.patch.object(
                instance, "_cleanup_scope",
                side_effect=sandbox.SandboxError("cgroup cleanup uncertain")), \
                self.assertRaises(subprocess.TimeoutExpired) as raised:
            instance.run(["tool"], lease, profile="agent")
        self.assertEqual(raised.exception.stdout_bytes, b"bounded output")
        self.assertEqual(raised.exception.stderr_bytes, b"bounded diagnostic")
        self.assertEqual(raised.exception.output, "bounded output")
        self.assertIn("resource cleanup failed", "\n".join(raised.exception.__notes__))
        self.assertIsInstance(raised.exception.__cause__, sandbox.SandboxError)
        self.assert_quarantined(lease)

    def test_nonzero_exit_remains_primary_when_resource_cleanup_is_uncertain(self):
        self.process(code=7, stdout="bounded output", stderr="bounded diagnostic")
        instance = sandbox.Sandbox(self.config)
        lease = self.lease_for(instance)
        with mock.patch.object(
                instance, "_cleanup_scope",
                side_effect=sandbox.SandboxError("cgroup cleanup uncertain")), \
                self.assertRaisesRegex(sandbox.SandboxError, "command exited 7") as raised:
            instance.run(["tool"], lease, profile="agent")
        self.assertEqual(raised.exception.returncode, 7)
        self.assertEqual(raised.exception.stdout_bytes, b"bounded output")
        self.assertEqual(raised.exception.stderr_bytes, b"bounded diagnostic")
        self.assertIn("resource cleanup failed", "\n".join(raised.exception.__notes__))
        self.assertIsInstance(raised.exception.__cause__, sandbox.SandboxError)
        self.assert_quarantined(lease)

    def test_stop_uncertainty_quarantines_the_active_lease(self):
        process = self.process()
        process.poll.return_value = None
        instance = sandbox.Sandbox(self.config)
        lease = self.lease_for(instance)

        def stopped(_process, _limit, _timeout, terminate):
            terminate()
            return b"", b"", False, False

        self.communication.side_effect = stopped
        with mock.patch.object(
                instance, "_terminate_scope",
                side_effect=sandbox.SandboxError("scope stop uncertain")), \
                self.assertRaisesRegex(sandbox.SandboxError, "resource termination failed"):
            instance.run(["tool"], lease, profile="agent")
        self.assert_quarantined(lease)

    def test_launcher_wait_failure_still_checks_scope_cleanup(self):
        process = self.process()
        process.poll.return_value = None
        process.wait.side_effect = [subprocess.TimeoutExpired("launcher", 5),
                                    subprocess.TimeoutExpired("launcher", 5)]
        self.scope_states[0]["ControlGroup"] = "/fixture.scope"
        instance = sandbox.Sandbox(self.config)
        lease = self.lease_for(instance)
        with mock.patch.object(sandbox.Sandbox, "_scope_populated", return_value=True), \
                self.assertRaisesRegex(sandbox.SandboxError, "launcher did not stop"):
            instance.run(["tool"], lease, profile="agent")
        self.assert_quarantined(lease)
        controls = [call.args[0] for call in self.version.call_args_list
                    if call.args[0][7] == "systemctl"]
        self.assertTrue(any("reset-failed" in command for command in controls))
        self.kill.assert_called_once_with(process.pid, sandbox.signal.SIGKILL)

    def test_invalid_utf8_is_retained_in_raw_streams(self):
        self.process()
        self.communication.return_value = (b"\xff\x00\n", b"\xfe\n", False, False)
        result = self.run_sandbox(["tool"], profile="agent")
        self.assertEqual(result.stdout_bytes, b"\xff\x00\n")
        self.assertEqual(result.stderr_bytes, b"\xfe\n")

    def test_output_limit_kills_scope_and_preserves_bounded_prefix(self):
        self.process()
        def exceeded(_process, _limit, _timeout, terminate):
            terminate()
            return b"x" * 16, b"diagnostic", True, False
        self.communication.side_effect = exceeded
        with self.assertRaises(sandbox.SandboxResourceExceeded) as raised:
            self.run_sandbox(["tool"], profile="agent")
        self.assertEqual(raised.exception.stdout_bytes, b"x" * 16)
        self.assertEqual(raised.exception.resource_status["profile"], "agent")

    def test_resource_evidence_keeps_only_bounded_controller_fields(self):
        outcome = subprocess.CompletedProcess([], -9, "", "")
        outcome.resource_status = {
            "profile": "agent", "unit": "fixture.scope", "LoadState": "loaded",
            "Result": "oom-kill",
            "MemoryPeak": "67108864", "untrusted": "secret",
            "verified": {"memory_max": "67108864", "unknown": "secret"},
        }
        self.assertEqual(sandbox.resource_evidence(outcome), {
            "profile": "agent", "unit": "fixture.scope", "Result": "oom-kill",
            "MemoryPeak": "67108864", "verified": {"memory_max": "67108864"},
        })

    def test_internal_build_hardlinks_do_not_grant_external_file_access(self):
        first = self.workspace / "artifact"
        second = self.workspace / "artifact-copy"
        first.write_text("build output")
        os.link(first, second)
        self.process()
        self.assertEqual(self.run_sandbox(["tool"], profile="agent").returncode, 0)
        outside = self.directory / "outside-link"
        os.link(first, outside)
        self.launch.reset_mock()
        with self.assertRaises(sandbox.SandboxError):
            self.run_sandbox(["tool"], profile="agent")
        self.launch.assert_not_called()


class OutputControlTests(unittest.TestCase):
    def test_output_is_bounded_without_limiting_workspace_file_sizes(self):
        self.assertNotIn("RLIMIT_FSIZE", sandbox._RESOURCE_GUARD)
        process = subprocess.Popen(
            [sys.executable, "-I", "-B", "-c",
             "import os\nwhile True: os.write(1, b'x' * 65536)"],
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        stopped = []

        def terminate():
            stopped.append(True)
            process.kill()

        stdout, stderr, exceeded, timed_out = sandbox._communicate_bounded(
            process, 1024, 5, terminate)
        process.wait(timeout=5)
        self.assertEqual((len(stdout), stderr, exceeded, timed_out), (1024, b"", True, False))
        self.assertEqual(stopped, [True])

    def test_stdout_and_stderr_share_one_combined_byte_cap(self):
        process = subprocess.Popen(
            [sys.executable, "-I", "-B", "-c",
             "import os,time; os.write(1,b'o'*768); os.write(2,b'e'*768); time.sleep(5)"],
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        stopped = []

        def terminate():
            stopped.append(True)
            process.kill()

        stdout, stderr, exceeded, timed_out = sandbox._communicate_bounded(
            process, 1024, 5, terminate)
        process.wait(timeout=5)
        self.assertEqual(len(stdout) + len(stderr), 1024)
        self.assertTrue(exceeded)
        self.assertFalse(timed_out)
        self.assertEqual(stopped, [True])

    def test_zero_cap_rejects_the_first_emitted_byte(self):
        process = subprocess.Popen(
            [sys.executable, "-I", "-B", "-c",
             "import os,time; os.write(1,b'x'); time.sleep(5)"],
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        stopped = []

        def terminate():
            stopped.append(True)
            process.kill()

        stdout, stderr, exceeded, timed_out = sandbox._communicate_bounded(
            process, 0, 5, terminate)
        process.wait(timeout=5)
        self.assertEqual((stdout, stderr, exceeded, timed_out), (b"", b"", True, False))
        self.assertEqual(stopped, [True])


if __name__ == "__main__":
    unittest.main()
