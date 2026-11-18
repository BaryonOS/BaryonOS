"""Host unit controls; mocked launches do not establish namespace isolation."""
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import signal
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
        self.workspace = self.directory / "disposable"
        self.workspace.mkdir()
        self.backend = self.directory / "backend"
        self.backend.write_bytes(b"pinned backend fixture\n")
        self.backend.chmod(0o755)
        self.manager = self.directory / "systemd-run"
        self.manager.write_bytes(b"pinned resource manager fixture\n")
        self.manager.chmod(0o755)
        self.config = {
            "backend": str(self.backend),
            "backend_sha256": hashlib.sha256(self.backend.read_bytes()).hexdigest(),
            "rootfs": str(self.rootfs),
            "rootfs_sha256": sandbox.tree_digest(self.rootfs),
            "resource_manager": str(self.manager),
            "resource_manager_sha256": hashlib.sha256(self.manager.read_bytes()).hexdigest(),
            "scratch_root": str(self.directory),
            "timeout_seconds": 60,
            "memory_max_bytes": 64 * 1024 * 1024,
            "tasks_max": 16,
            "cpu_quota_percent": 100,
            "file_max_bytes": 1024 * 1024,
            "open_files_max": 256,
            "workspace_max_bytes": 4 * 1024 * 1024,
            "workspace_max_entries": 1000,
        }
        self.backend_version = subprocess.CompletedProcess([], 0, "bubblewrap 0.13.1\n", "")
        self.manager_version = subprocess.CompletedProcess([], 0, "systemd 259 (259.9)\n", "")

        def version(command, **_options):
            result = self.manager_version if "systemd-run" in command else self.backend_version
            if isinstance(result, BaseException):
                raise result
            return result

        self.version = self.enterContext(mock.patch.object(
            sandbox.subprocess, "run", side_effect=version))
        self.launch = self.enterContext(mock.patch.object(sandbox.subprocess, "Popen"))
        self.kill = self.enterContext(mock.patch.object(sandbox.os, "killpg"))
        self.tmpfs = self.enterContext(mock.patch.object(sandbox, "_require_private_tmpfs"))
        self.resource_env = self.enterContext(mock.patch.object(
            sandbox, "_resource_environment", return_value={"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8",
                                                              "XDG_RUNTIME_DIR": "/run/user/1000",
                                                              "DBUS_SESSION_BUS_ADDRESS": "unix:path=/run/user/1000/bus"}))
        self.output = self.enterContext(mock.patch.object(sandbox, "_read_output"))

    def process(self, code=0, stdout="output\n", stderr=""):
        process = mock.MagicMock()
        process.pid = 123456789
        process.returncode = code
        process.poll.return_value = code
        process.wait.return_value = code
        process.__enter__.return_value = process
        self.launch.return_value = process
        self.output.side_effect = [(stdout.encode(), False), (stderr.encode(), False)]
        return process

    def test_explicit_complete_configuration_and_bounded_timeout_are_required(self):
        malformed = (None, {}, {**self.config, "extra": True},
                     {**self.config, "backend_sha256": "bad"},
                     {**self.config, "rootfs_sha256": 123},
                     {**self.config, "resource_manager_sha256": "bad"})
        for config in malformed:
            with self.subTest(config=config), self.assertRaises(sandbox.SandboxError):
                sandbox.Sandbox(config)
        for timeout in (0, -1, 7201, True, 1.5, "60"):
            with self.subTest(timeout=timeout), self.assertRaises(sandbox.SandboxError):
                sandbox.Sandbox({**self.config, "timeout_seconds": timeout})
        for field, value in (("memory_max_bytes", 1), ("tasks_max", True),
                             ("cpu_quota_percent", 1001), ("file_max_bytes", 1),
                             ("open_files_max", 8), ("workspace_max_bytes", 1),
                             ("workspace_max_entries", 0)):
            with self.subTest(field=field), self.assertRaises(sandbox.SandboxError):
                sandbox.Sandbox({**self.config, field: value})
        with self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox({**self.config, "file_max_bytes": 8 * 1024 * 1024})
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
                sandbox.Sandbox(self.config).run(["tool"], self.workspace)
            metadata.unlink()
        self.launch.assert_not_called()
        self.version.assert_not_called()

    def test_workspace_may_not_contain_external_links_or_special_files(self):
        link = self.workspace / "outside"
        link.symlink_to(self.tool)
        with self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox(self.config).run(["tool"], self.workspace)
        link.unlink()
        os.link(self.tool, link)
        with self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox(self.config).run(["tool"], self.workspace)
        link.unlink()
        os.mkfifo(link)
        with self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox(self.config).run(["tool"], self.workspace)
        self.launch.assert_not_called()

    def test_overlapping_workspace_is_rejected_before_execution(self):
        for path in (self.rootfs, self.rootfs / "usr", self.directory, Path("/")):
            with self.subTest(path=path), self.assertRaises(sandbox.SandboxError):
                sandbox.Sandbox(self.config).run(["tool"], path)
        self.version.assert_not_called()
        self.launch.assert_not_called()

    def test_command_environment_and_timeout_validation_precede_execution(self):
        instance = sandbox.Sandbox(self.config)
        for command in ([], "tool", [""], ["tool", "\0"], ["tool", 1]):
            with self.subTest(command=command), self.assertRaises(sandbox.SandboxError):
                instance.run(command, self.workspace)
        for timeout in (0, True, 61, "5"):
            with self.subTest(timeout=timeout), self.assertRaises(sandbox.SandboxError):
                instance.run(["tool"], self.workspace, timeout=timeout)
        for env in ([], ["invalid"], {"SSH_AUTH_SOCK": "/run/canary"}, {"LD_PRELOAD": "canary"},
                    {"PYTHONPATH": "/workspace"}, {"HTTPS_PROXY": "canary"}, {"HOME": "/root"}):
            with self.subTest(env=env), self.assertRaises(sandbox.SandboxError):
                instance.run(["tool"], self.workspace, env=env)
        self.version.assert_not_called()
        self.launch.assert_not_called()

    def test_mounts_namespaces_environment_and_raw_output_are_preserved(self):
        process = self.process(code=33, stdout="raw output\n", stderr="raw diagnostic\n")
        extra = {"NOS_BUILD_DIR": "/workspace/build", "MACHINE": "q35"}
        with mock.patch.dict(os.environ, {"PRIVATE_CANARY": "absent", "SSH_AUTH_SOCK": "/host/socket"}):
            result = sandbox.Sandbox(self.config).run(["tool", "arg"], self.workspace, env=extra, timeout=7)
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
        self.assertEqual(command.count("--bind"), 1)
        self.assertEqual(command[command.index("--bind") + 1:command.index("--bind") + 3],
                         [str(self.workspace), "/workspace"])
        self.assertEqual([command[index + 1] for index, item in enumerate(command) if item == "--tmpfs"],
                         ["/tmp", "/run", "/home", "/root"])
        environment = {command[index + 1]: command[index + 2]
                       for index, item in enumerate(command) if item == "--setenv"}
        self.assertEqual(environment, isolated_environment(extra))
        self.assertEqual(options["env"], {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8"})
        self.assertEqual(options["stdin"], subprocess.DEVNULL)
        self.assertTrue(options["close_fds"])
        self.assertTrue(options["start_new_session"])
        self.assertEqual(options["umask"], 0o022)
        self.assertEqual(result.stdout_bytes, b"raw output\n")
        self.assertEqual(len(options["pass_fds"]), 1)
        process.communicate.assert_called_once_with(timeout=7)
        self.kill.assert_not_called()

    def test_namespace_failure_has_no_ambient_fallback_and_keeps_failure_status(self):
        self.process(code=1, stderr="namespace unavailable\n")
        result = sandbox.Sandbox(self.config).run(["tool"], self.workspace)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stderr, "namespace unavailable\n")
        self.launch.assert_called_once()

    def test_timeout_kills_the_process_group_reaps_and_preserves_partial_output(self):
        process = self.process()
        process.communicate.side_effect = [subprocess.TimeoutExpired("launcher", 2),
                                           (b"partial output\n", b"partial diagnostic\n")]
        with self.assertRaises(subprocess.TimeoutExpired) as raised:
            sandbox.Sandbox(self.config).run(["tool"], self.workspace, timeout=2)
        self.assertEqual((raised.exception.cmd, raised.exception.output, raised.exception.stderr),
                         (["tool"], "partial output\n", "partial diagnostic\n"))
        self.kill.assert_called_with(process.pid, signal.SIGKILL)
        self.assertEqual(process.communicate.call_args_list,
                         [mock.call(timeout=2), mock.call(timeout=5)])

    def test_invalid_utf8_is_retained_in_raw_streams(self):
        process = self.process()
        process.communicate.return_value = (b"\xff\x00\n", b"\xfe\n")
        result = sandbox.Sandbox(self.config).run(["tool"], self.workspace)
        self.assertEqual(result.stdout_bytes, b"\xff\x00\n")
        self.assertEqual(result.stderr_bytes, b"\xfe\n")

    def test_internal_build_hardlinks_do_not_grant_external_file_access(self):
        first = self.workspace / "artifact"
        second = self.workspace / "artifact-copy"
        first.write_text("build output")
        os.link(first, second)
        self.process()
        self.assertEqual(sandbox.Sandbox(self.config).run(["tool"], self.workspace).returncode, 0)
        outside = self.directory / "outside-link"
        os.link(first, outside)
        self.launch.reset_mock()
        with self.assertRaises(sandbox.SandboxError):
            sandbox.Sandbox(self.config).run(["tool"], self.workspace)
        self.launch.assert_not_called()


if __name__ == "__main__":
    unittest.main()
