"""Tests for host command logging failure paths."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import common


class QuantumSourceBinding(unittest.TestCase):
    def test_windows_and_posix_path_order_bind_the_same_source_bytes(self):
        # Model pathlib's Windows case-insensitive ordering while reading the
        # same actual files. This reproduced a real Windows/WSL mismatch.
        def windows_less(left, right):
            return tuple(p.casefold() for p in left.parts) < tuple(p.casefold() for p in right.parts)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative in ("boot/Z.rs", "boot/a.rs", "boot/A/file.rs", "boot/a.txt"):
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(relative.encode("ascii"))
            with patch.object(common, "ROOT", root):
                posix = common.source_hash()
                with patch.object(type(root), "__lt__", windows_less):
                    self.assertLess(root / "a", root / "Z")
                    windows = common.source_hash()
                    self.assertEqual(posix, windows)
                    (root / "boot/a.rs").write_bytes(b"changed")
                    self.assertNotEqual(posix, common.source_hash())

    def test_inherited_package_manifest_and_job_codec_are_source_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            parent = root / "experiments/quantum/Cargo.toml"
            parent.parent.mkdir(parents=True)
            parent.write_text('[workspace.package]\nedition="2021"\n')
            codec = root / "experiments/quantum/job-protocol/src/lib.rs"
            codec.parent.mkdir(parents=True)
            codec.write_text("pub const VERSION: u16 = 1;\n")
            with patch.object(common, "ROOT", root):
                initial = common.source_hash()
                parent.write_text('[workspace.package]\nedition="2024"\n')
                changed_manifest = common.source_hash()
                self.assertNotEqual(initial, changed_manifest)
                codec.write_text("pub const VERSION: u16 = 2;\n")
                self.assertNotEqual(changed_manifest, common.source_hash())


class SourceReceipt(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        source = self.root / "boot/source.rs"
        source.parent.mkdir()
        source.write_text("source\n")
        self.receipt = self.root / common.SOURCE_RECEIPT_NAME

    def write(self, *, commit="a" * 40, source=None):
        self.receipt.write_text(json.dumps({"schema": 1, "source_commit": commit,
                                            "source_sha256": source or common.source_hash(self.root)}) + "\n")
        self.receipt.chmod(0o444)

    def identity(self):
        return common.source_identity(self.root, {"NOS_SOURCE_RECEIPT": str(self.receipt)})

    def test_read_only_receipt_binds_commit_to_current_source(self):
        self.write()
        self.assertEqual(self.identity(), ("a" * 40, common.source_hash(self.root)))

    def test_source_change_and_noncanonical_receipt_fail_closed(self):
        self.write()
        (self.root / "boot/source.rs").write_text("changed\n")
        with self.assertRaisesRegex(ValueError, "does not bind"):
            self.identity()
        other = self.root / "other.json"
        other.write_bytes(self.receipt.read_bytes())
        other.chmod(0o444)
        with self.assertRaisesRegex(ValueError, "fixed private"):
            common.source_identity(self.root, {"NOS_SOURCE_RECEIPT": str(other)})

    def test_duplicate_or_writable_receipt_fails_closed(self):
        self.receipt.write_text('{"schema":1,"schema":1,"source_commit":"' + "a" * 40
                                + '","source_sha256":"' + common.source_hash(self.root) + '"}\n')
        self.receipt.chmod(0o444)
        with self.assertRaisesRegex(ValueError, "duplicate"):
            self.identity()
        self.receipt.chmod(0o644)
        self.write()
        for mode in (0o644, 0o666):
            with self.subTest(mode=oct(mode)):
                self.receipt.chmod(mode)
                with self.assertRaisesRegex(ValueError, "fixed private"):
                    self.identity()

    def test_host_fallback_keeps_the_existing_git_identity_path(self):
        with patch.object(common.subprocess, "check_output", return_value="b" * 40 + "\n") as git:
            self.assertEqual(common.source_identity(self.root, {}),
                             ("b" * 40, common.source_hash(self.root)))
        git.assert_called_once()

    def test_controller_commit_disables_lazy_fetch_and_ambient_credentials(self):
        environment = {"PATH": "/usr/bin:/bin", "GIT_NO_LAZY_FETCH": "1"}
        with patch.object(common, "controller_environment", return_value=environment), \
                patch.object(common.subprocess, "check_output", return_value="c" * 40 + "\n") as git:
            self.assertEqual(common.controller_source_commit(self.root), "c" * 40)
        self.assertEqual(git.call_args.args[0],
                         [common.CONTROLLER_GIT, "--no-lazy-fetch", "rev-parse", "HEAD"])
        self.assertEqual(git.call_args.kwargs["env"], environment)

    def test_controller_commit_failure_is_a_controlled_refusal(self):
        failure = subprocess.CalledProcessError(128, [common.CONTROLLER_GIT])
        with patch.object(common, "controller_environment", return_value={}), \
                patch.object(common.subprocess, "check_output", side_effect=failure), \
                self.assertRaisesRegex(ValueError, "unavailable"):
            common.controller_source_commit(self.root)


class MachineSelection(unittest.TestCase):
    def test_only_explicit_versioned_profiles_and_no_silent_fallback(self):
        with patch.dict(os.environ, {}, clear=True):
            self.assertEqual(common.qemu_machine(), "pc-q35-10.0")
        for name in common.QEMU_MACHINES:
            with patch.dict(os.environ, {"NOS_QEMU_MACHINE": name}):
                self.assertEqual(common.qemu_machine(), name)
        for name in ("q35", "", "pc-q35-unknown", "pc-q35-7.2,accel=kvm"):
            with self.subTest(name=name):
                with self.assertRaises(ValueError):
                    common.qemu_machine(name)
        with patch.dict(os.environ, {"NOS_QEMU_MACHINE": "invalid"}):
            with self.assertRaises(ValueError):
                common.qemu_machine()
            self.assertEqual(common.qemu_machine("pc-q35-7.2"), "pc-q35-7.2")


class AccelerationSelection(unittest.TestCase):
    def test_tcg_is_the_only_implicit_default(self):
        with patch.dict(os.environ, {}, clear=True):
            self.assertEqual(common.qemu_accel(), "tcg,thread=single")
        for value in ("", "tcg", "tcg,thread=single", "auto", "kvm:tcg"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                common.qemu_accel(value)

    def test_kvm_opt_in_requires_a_readable_writable_character_device(self):
        device = Path("/dev/kvm")
        with patch.object(common.Path, "is_char_device", return_value=False), \
                patch.object(common.os, "access", return_value=True):
            with self.assertRaisesRegex(RuntimeError, "KVM was requested"):
                common.qemu_accel("kvm", device=device)
        with patch.object(common.Path, "is_char_device", return_value=True), \
                patch.object(common.os, "access", return_value=False):
            with self.assertRaisesRegex(RuntimeError, "KVM was requested"):
                common.qemu_accel("kvm", device=device)
        with patch.object(common.Path, "is_char_device", return_value=True), \
                patch.object(common.os, "access", return_value=True) as access:
            self.assertEqual(common.qemu_accel("kvm", device=device), "kvm")
            access.assert_called_once_with(device, os.R_OK | os.W_OK)


class RunLoggedTimeout(unittest.TestCase):
    def test_timeout_saves_partial_output_and_metadata_before_reraising(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            build = root / "build"
            logs = build / "logs"
            logs.mkdir(parents=True)
            (logs / "timeout.stdout.txt").write_text("stale stdout", encoding="utf-8")
            (logs / "timeout.stderr.txt").write_text("stale stderr", encoding="utf-8")
            (logs / "timeout.json").write_text('{"returncode": 0}', encoding="utf-8")
            error = subprocess.TimeoutExpired(
                [sys.executable], 3, output=b"partial stdout\xff", stderr=b"partial stderr"
            )
            with patch.object(common, "BUILD", build), \
                    patch.object(common, "environment", return_value={"PATH": os.environ["PATH"]}), \
                    patch.object(common.subprocess, "run", side_effect=error):
                with self.assertRaises(subprocess.TimeoutExpired) as raised:
                    common.run_logged([sys.executable, "-c", "pass"], "timeout", cwd=root, timeout=3)

            self.assertIs(raised.exception, error)
            self.assertEqual((logs / "timeout.stdout.txt").read_text(encoding="utf-8"), "partial stdout\ufffd")
            self.assertEqual((logs / "timeout.stderr.txt").read_text(encoding="utf-8"), "partial stderr")
            metadata = json.loads((logs / "timeout.json").read_text(encoding="utf-8"))
            self.assertIsNone(metadata["returncode"])
            self.assertTrue(metadata["timed_out"])
            self.assertEqual(metadata["timeout_seconds"], 3)


class SharedArtifactHelpers(unittest.TestCase):
    def test_hash_helpers_preserve_sha256_format_and_validation(self):
        data = b"bounded synthetic artifact"
        digest = hashlib.sha256(data).hexdigest()
        self.assertEqual(common.sha256_bytes(data), digest)
        self.assertEqual(common.require_sha256(digest), digest)
        for value in (None, 1, "A" * 64, "a" * 63, "g" * 64):
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, "malformed SHA256"):
                common.require_sha256(value)

    def test_tree_listing_guards_each_path_before_returning_regular_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "nested").mkdir()
            first = root / "first.bin"
            second = root / "nested/second.bin"
            first.write_bytes(b"first")
            second.write_bytes(b"second")
            checked = []

            def validate(path):
                checked.append(path)
                return path

            self.assertEqual(
                common.regular_files_below(root, validate_path=validate),
                {first, second},
            )
            self.assertIn(first, checked)
            self.assertIn(second, checked)
            self.assertIn(root / "nested", checked)

    def test_tree_listing_preserves_relative_paths_for_callers(self):
        with tempfile.TemporaryDirectory(dir=Path.cwd()) as directory:
            absolute_root = Path(directory)
            relative_root = absolute_root.relative_to(Path.cwd())
            expected = relative_root / "artifact.bin"
            (absolute_root / expected.name).write_bytes(b"fixture")

            files = common.regular_files_below(
                relative_root,
                validate_path=lambda path: path.absolute(),
            )

            self.assertEqual(files, {expected})

    def test_tree_listing_refuses_a_link_before_including_its_target(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "root"
            root.mkdir()
            outside = Path(directory) / "outside"
            outside.write_bytes(b"preserve")
            self.addCleanup(outside.unlink, missing_ok=True)
            link = root / "linked"
            link.symlink_to(outside)

            def validate(path):
                if path.is_symlink():
                    raise ValueError("linked artifact path")
                return path

            with self.assertRaisesRegex(ValueError, "linked artifact"):
                common.regular_files_below(root, validate_path=validate)
            self.assertEqual(outside.read_bytes(), b"preserve")


if __name__ == "__main__":
    unittest.main()
