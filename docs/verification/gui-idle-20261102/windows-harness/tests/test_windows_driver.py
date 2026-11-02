import io
from pathlib import Path
import sys
from types import SimpleNamespace
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_windows_window as driver


class DriverGuards(unittest.TestCase):
    def setUp(self):
        self.qemu = Path(r"C:\Program Files\qemu\qemu-system-x86_64.exe")
        self.source = Path(r"C:\frozen-source")
        self.build = self.source / "build"
        self.machine = "pc-q35-8.2"

    def test_explicit_whpx_keeps_isolation_and_device_contract(self):
        original = driver.expected_command(self.qemu, self.machine)
        frozen = original.copy()
        translated = driver.translate_command(original, self.qemu, self.machine, "whpx")
        self.assertEqual(original, frozen)
        self.assertEqual(translated[translated.index("-accel") + 1], "whpx")
        self.assertEqual(translated[translated.index("-object") + 1], "rng-builtin,id=rng0")
        self.assertEqual(translated[1], "-no-user-config")
        self.assertEqual(translated[translated.index("-nic") + 1], "none")
        self.assertEqual(translated[translated.index("-smp") + 1], "1")
        self.assertEqual([x for x in translated if "virtio-" in x], [x for x in original if "virtio-" in x])

    def test_tcg_never_silently_selects_whpx(self):
        translated = driver.translate_command(driver.expected_command(self.qemu, self.machine), self.qemu, self.machine, "tcg")
        self.assertEqual(translated[translated.index("-accel") + 1], "tcg,thread=single")

    def test_windows_executable_case_is_not_another_binary(self):
        command = driver.expected_command(self.qemu, self.machine)
        command[0] = str(self.qemu).removesuffix(".exe") + ".EXE"
        translated = driver.translate_command(command, self.qemu, self.machine, "whpx")
        self.assertEqual(translated[0], command[0])
        command[0] = r"C:\other-install\qemu-system-x86_64.EXE"
        with self.assertRaises(ValueError):
            driver.translate_command(command, self.qemu, self.machine, "whpx")

    def test_changed_cpu_network_disk_or_extra_device_is_refused(self):
        for option, value in [("-smp", "2"), ("-nic", "user"), ("-cpu", "host"), ("-drive", "file=PhysicalDrive0")]:
            changed = driver.expected_command(self.qemu, self.machine)
            changed[changed.index(option) + 1] = value
            with self.subTest(option=option), self.assertRaises(ValueError):
                driver.translate_command(changed, self.qemu, self.machine, "whpx")
        changed = driver.expected_command(self.qemu, self.machine) + ["-device", "usb-host"]
        with self.assertRaises(ValueError):
            driver.translate_command(changed, self.qemu, self.machine, "whpx")

    def test_unknown_acceleration_and_nonlist_are_refused(self):
        for acceleration in ["auto", "kvm", "whpx,ssd=off"]:
            with self.subTest(acceleration=acceleration), self.assertRaises(ValueError):
                driver.translate_command(driver.expected_command(self.qemu, self.machine), self.qemu, self.machine, acceleration)
        with self.assertRaises(ValueError):
            driver.translate_command(tuple(driver.expected_command(self.qemu, self.machine)), self.qemu, self.machine, "whpx")

    def factory(self, image=None):
        return driver.WindowsProcesses(self.source, self.build, self.qemu, self.machine, "whpx", 30,
            lambda path, **kwargs: Path(path), image or (lambda child, expected: expected))

    def kwargs(self):
        return {"cwd": self.build / "run/owned", "env": {}, "stdin": driver.host_process.PIPE,
                "stdout": driver.host_process.PIPE, "stderr": SimpleNamespace(name=str(self.build / "run/owned/stderr.txt")), "bufsize": 0}

    def test_process_translation_preserves_report_list_and_one_owned_child(self):
        factory = self.factory()
        command = driver.expected_command(self.qemu, self.machine)
        report_reference = command
        child = mock.Mock(pid=123)
        child.poll.return_value = 0
        try:
            with mock.patch.object(driver.host_process, "Popen", return_value=child) as popen:
                self.assertIs(factory.Popen(command, **self.kwargs()), child)
                self.assertIs(command, report_reference)
                self.assertEqual(command, factory.actual_command)
                self.assertEqual(factory.original_command, driver.expected_command(self.qemu, self.machine))
                self.assertEqual(popen.call_args.kwargs["creationflags"], driver.host_process.CREATE_NO_WINDOW)
                self.assertEqual(factory.readlink("/proc/123/exe"), str(self.qemu))
                with self.assertRaises(ValueError):
                    factory.Popen(command, **self.kwargs())
                with self.assertRaises(ValueError):
                    factory.readlink("/proc/124/exe")
        finally:
            factory.close()
        child.kill.assert_not_called()

    def test_unexpected_shell_or_text_pipe_never_starts_child(self):
        for change in [{"shell": True}, {"text": True}, {"bufsize": 1}, {"stdin": None}]:
            factory = self.factory()
            kwargs = self.kwargs() | change
            with mock.patch.object(driver.host_process, "Popen") as popen, self.assertRaises(ValueError):
                factory.Popen(driver.expected_command(self.qemu, self.machine), **kwargs)
            popen.assert_not_called()

    def test_image_query_failure_still_leaves_owned_handle_for_cleanup(self):
        def fail_image(child, expected):
            raise OSError("identity query failed")
        factory = self.factory(image=fail_image)
        child = mock.Mock(pid=123)
        child.poll.return_value = None
        child.stdin = io.BytesIO()
        child.stdout = io.BytesIO()
        with mock.patch.object(driver.host_process, "Popen", return_value=child), self.assertRaises(OSError):
            factory.Popen(driver.expected_command(self.qemu, self.machine), **self.kwargs())
        factory.close()
        child.kill.assert_called_once()
        child.wait.assert_called_once_with(timeout=5)
        self.assertTrue(factory.watchdog_terminated)
        self.assertTrue(child.stdin.closed)
        self.assertTrue(child.stdout.closed)

    def test_already_exited_child_still_closes_original_pipes(self):
        factory = self.factory()
        child = mock.Mock(pid=123)
        child.poll.return_value = 0
        child.stdin, child.stdout = io.BytesIO(), io.BytesIO()
        factory.child = child
        factory.close()
        factory.close()
        self.assertTrue(child.stdin.closed and child.stdout.closed)
        child.kill.assert_not_called()

    def test_failed_transport_constructor_remains_in_cleanup_report(self):
        class FailingTransport:
            def __init__(self, process, deadline):
                self._workers = [object()]
                self.cleanup_errors = []
                self.cleanup_pending = True
                raise TimeoutError("greeting did not arrive")
            def close(self):
                self.cleanup_pending = False
        child = object()
        records = []
        control_type = driver.tracked_transport(FailingTransport, records, SimpleNamespace(child=child))
        with self.assertRaises(TimeoutError):
            control_type(child, 10)
        self.assertEqual(len(records), 1)
        status = driver.cleanup_transports(records)
        self.assertFalse(status[0]["initialized"])
        self.assertFalse(status[0]["pending"])
        self.assertIn("greeting did not arrive", status[0]["initialization_error"])

    def test_git_dependency_is_read_only_and_bounded(self):
        factory = self.factory()
        with mock.patch.object(driver.host_process, "check_output", return_value="a" * 40) as query:
            factory.check_output(["git", "rev-parse", "HEAD"], cwd=self.source, text=True)
            self.assertEqual(query.call_args.kwargs["timeout"], 10)
        with mock.patch.object(driver.host_process, "check_output") as query:
            for command, kwargs in [(["git", "reset", "--hard"], {"cwd": self.source, "text": True}),
                                    (["git", "rev-parse", "HEAD"], {"cwd": self.build, "text": True})]:
                with self.assertRaises(ValueError):
                    factory.check_output(command, **kwargs)
            query.assert_not_called()

    def test_preloaded_other_checkout_is_refused(self):
        fake = SimpleNamespace(__file__=str(Path(r"C:\other-source\tools\common.py")))
        for name in ["common", "run_display_probe"]:
            with self.subTest(module=name), mock.patch.object(Path, "glob", return_value=[Path("common.py"), Path("run_display_probe.py")]):
                with mock.patch.dict(sys.modules, {name: fake}), self.assertRaises(ValueError):
                    driver.import_runner(self.source)


    def test_idle_hook_precedes_exactly_one_original_first_key(self):
        calls = []
        class Base:
            def __init__(self, process, deadline):
                self.deadline = deadline
            def command(self, name, arguments=None):
                calls.append((name, arguments))
                return name
        child = object()
        def observe(control, original, name, arguments):
            self.assertEqual(calls, [("query-status", None)])
            self.assertEqual((name, arguments), ("send-key", {"first": True}))
            original("query-cpus-fast")  # Bypasses hook; cannot recurse.
        wrapped = driver.tracked_transport(Base, [], SimpleNamespace(child=child), observe)(child, 100)
        wrapped.command("query-status")
        wrapped.command("send-key", {"first": True})
        wrapped.command("send-key", {"second": True})
        self.assertEqual(calls, [("query-status", None), ("query-cpus-fast", None),
                                ("send-key", {"first": True}), ("send-key", {"second": True})])

    def test_failed_idle_observation_never_sends_the_first_key(self):
        sent = []
        class Base:
            def __init__(self, process, deadline): pass
            def command(self, name, arguments=None): sent.append(name)
        child = object()
        def reject(*args): raise ValueError("not a verified initial frame")
        wrapped = driver.tracked_transport(Base, [], SimpleNamespace(child=child), reject)(child, 100)
        with self.assertRaises(ValueError):
            wrapped.command("send-key", {})
        self.assertEqual(sent, [])

    def test_no_opt_in_keeps_original_key_path_without_observation(self):
        sent = []
        class Base:
            def __init__(self, process, deadline): pass
            def command(self, name, arguments=None): sent.append((name, arguments))
        child = object()
        wrapped = driver.tracked_transport(Base, [], SimpleNamespace(child=child))(child, 100)
        wrapped.command("send-key", {"keys": []})
        self.assertEqual(sent, [("send-key", {"keys": []})])

    def test_missing_or_failed_idle_evidence_cannot_validate(self):
        for used, passed in [(False, False), (False, True), (True, False)]:
            with self.subTest(used=used, passed=passed), self.assertRaises(ValueError):
                driver.validate_idle_evidence(SimpleNamespace(used=used, record={"measurement_passed": passed}))


if __name__ == "__main__":
    unittest.main()
