import hashlib
import json
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import package_windows_qemu as package


class WindowsQemuPackageTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.build = self.root / "build"
        self.esp = self.build / "esp"
        self.esp_boot = self.esp / "EFI/BOOT"
        self.esp_boot.mkdir(parents=True)
        self.apps_dir = self.build / "users/materiaal"
        self.apps_dir.mkdir(parents=True)
        self.apps = {}
        kernel = bytearray(b"kernel")
        for name in package.APP_NAMES:
            data = ("app:" + name).encode()
            kernel.extend(data)
            path = self.apps_dir / (name + ".elf")
            path.write_bytes(data)
            self.apps[name] = {
                "name": name,
                "sha256": hashlib.sha256(data).hexdigest(),
                "elf": {"name": name},
            }
        (self.esp / "kernel.elf").write_bytes(kernel)
        (self.esp_boot / "BOOTX64.EFI").write_bytes(b"loader")
        self.manifest = {
            "profile": "normal",
            "kernel_feature": package.PROFILE,
            "loader_feature": "framebuffer",
            "source_fingerprint_version": 3,
            "source_sha256": "a" * 64,
            "kernel": {"sha256": hashlib.sha256(kernel).hexdigest()},
            "loader": {"sha256": hashlib.sha256(b"loader").hexdigest()},
            "materiaal": {
                "passed": True,
                "native_runtime_performed": False,
                "probe_controller": False,
                "apps": list(self.apps.values()),
            },
        }
        (self.build / "manifest.json").write_text(json.dumps(self.manifest))
        with patch.object(package, "inspect_user", side_effect=lambda path: {"name": path.stem}):
            self.validated = package.validate_build(self.build, verify_source=False)

    def tearDown(self):
        self.temporary.cleanup()

    def test_whpx_probe_disables_user_configuration(self):
        helper = package.ROOT / "tools/windows-qemu/Acceleration.ps1"
        assignments = [
            line.strip().split(" = ", 1)[1]
            for line in helper.read_text().splitlines()
            if line.strip().startswith("$start.Arguments = ")
        ]
        self.assertEqual(len(assignments), 1)
        arguments = assignments[0].strip('"').split()
        self.assertIn("-no-user-config", arguments)
        self.assertIn("-nodefaults", arguments)
        self.assertIn("-S", arguments)
        self.assertEqual(arguments[arguments.index("-nic") + 1], "none")

    def window_profile(self):
        kernel = bytearray(b"window-kernel")
        apps = []
        for name in package.WINDOW_APP_NAMES:
            data = ("app:" + name).encode()
            kernel.extend(data)
            (self.apps_dir / (name + ".elf")).write_bytes(data)
            apps.append({"name": name, "sha256": hashlib.sha256(data).hexdigest(), "elf": {"name": name}})
        (self.esp / "kernel.elf").write_bytes(kernel)
        self.manifest.update(kernel_feature=package.WINDOW_PROFILE, loader_feature="desktop-mode")
        self.manifest["kernel"] = {"sha256": hashlib.sha256(kernel).hexdigest()}
        self.manifest["materiaal"]["apps"] = apps
        self.save_manifest()

    def save_manifest(self):
        (self.build / "manifest.json").write_text(json.dumps(self.manifest))

    @staticmethod
    def graphics_fixture(repo):
        graphics = repo / "ui/graphics"
        (graphics / "assets").mkdir(parents=True)
        (graphics / "licenses").mkdir()
        atlas = b"synthetic atlas, not a native font"
        (graphics / "assets/open-sans-atlas.rle").write_bytes(atlas)
        (graphics / "assets/font-provenance.json").write_text(json.dumps({
            "font": "Open Sans", "data_bytes": len(atlas), "data_sha256": hashlib.sha256(atlas).hexdigest(),
        }))
        for name in ("THIRD-PARTY.md", "README.md", "licenses/Apache-2.0.txt"):
            (graphics / name).write_text("synthetic license fixture")
        native = repo / "ui/native"
        (native / "licenses").mkdir(parents=True)
        for name in ("THIRD-PARTY.md", "licenses/NotoSansMono-OFL.txt",
                     "licenses/noto-sans-mono-bitmap-MIT.txt"):
            (native / name).write_text("synthetic native notice: " + name)

    def test_window_profile_requires_eight_ordered_real_embedded_elves(self):
        self.window_profile()
        with patch.object(package, "inspect_user", side_effect=lambda path: {"name": path.stem}):
            _, result = package.validate_build(self.build, verify_source=False)
        self.assertEqual(result["kernel_feature"], package.WINDOW_PROFILE)
        self.assertEqual(result["loader_feature"], "desktop-mode")
        self.assertEqual([app["name"] for app in result["materiaal"]["apps"]], list(package.WINDOW_APP_NAMES))

    def test_window_profile_rejects_old_loader_missing_app_and_wrong_order(self):
        for mutation in ("loader", "missing", "order"):
            with self.subTest(mutation=mutation):
                self.window_profile()
                if mutation == "loader":
                    self.manifest["loader_feature"] = "framebuffer"
                elif mutation == "missing":
                    self.manifest["materiaal"]["apps"].pop()
                else:
                    self.manifest["materiaal"]["apps"].reverse()
                self.save_manifest()
                with patch.object(package, "inspect_user", side_effect=lambda path: {"name": path.stem}):
                    with self.assertRaises(ValueError):
                        package.validate_build(self.build, verify_source=False)

    def test_window_probe_and_hidpi_are_not_interactive_packages(self):
        for feature in ("native-window-session-probe", "native-window-session-hidpi", "native-window-session-hidpi-probe"):
            self.window_profile()
            self.manifest["kernel_feature"] = feature
            self.save_manifest()
            with self.assertRaisesRegex(ValueError, "supported interactive"):
                package.validate_build(self.build, verify_source=False)

    def test_window_probe_controller_or_changed_calculator_is_rejected(self):
        self.window_profile()
        self.manifest["materiaal"]["probe_controller"] = True
        self.save_manifest()
        with self.assertRaisesRegex(ValueError, "non-probe"):
            package.validate_build(self.build, verify_source=False)
        self.manifest["materiaal"]["probe_controller"] = False
        self.save_manifest()
        (self.apps_dir / "calculator-app.elf").write_bytes(b"different calculator")
        with patch.object(package, "inspect_user", side_effect=lambda path: {"name": path.stem}):
            with self.assertRaisesRegex(ValueError, "not bound"):
                package.validate_build(self.build, verify_source=False)

    def test_window_bundle_labels_public_research_and_carries_font_provenance(self):
        self.window_profile()
        output = self.root / "window.zip"
        self.package_to(output)
        with zipfile.ZipFile(output) as archive:
            manifest = json.loads(archive.read("manifest.json"))
            self.assertEqual(manifest["kernel_feature"], package.WINDOW_PROFILE)
            self.assertEqual(manifest["loader_feature"], "desktop-mode")
            self.assertEqual(manifest["session"], {
                "classification": "public-workspace-research", "authentication_integrated": False,
                "initial_processes": 8, "width": 1280, "height": 720,
            })
            for source, name in package.GRAPHICS_FILES:
                data = archive.read(name)
                self.assertEqual(data, (package.ROOT / source).read_bytes())
                self.assertEqual(hashlib.sha256(data).hexdigest(), manifest["files"][name]["sha256"])
            font = json.loads(archive.read("licenses/OpenSans-font-provenance.json"))
            self.assertEqual(font["data_sha256"], manifest["graphics"]["atlas_sha256"])

    def test_both_desktop_bundles_include_the_original_noto_notices_and_hashes(self):
        # Keep this required set independent of GRAPHICS_FILES: removing an
        # entry from the packager must not silently change the test's contract.
        notices = (
            ("ui/native/THIRD-PARTY.md", "licenses/NotoSansMono-NOTICE.md"),
            ("ui/native/licenses/NotoSansMono-OFL.txt", "licenses/NotoSansMono-OFL.txt"),
            ("ui/native/licenses/noto-sans-mono-bitmap-MIT.txt", "licenses/noto-sans-mono-bitmap-MIT.txt"),
        )
        for profile in (package.PROFILE, package.WINDOW_PROFILE):
            with self.subTest(profile=profile):
                if profile == package.WINDOW_PROFILE:
                    self.window_profile()
                output = self.root / (profile + ".zip")
                self.package_to(output)
                with zipfile.ZipFile(output) as archive:
                    manifest = json.loads(archive.read("manifest.json"))
                    for source, name in notices:
                        expected = (package.ROOT / source).read_bytes()
                        self.assertEqual(archive.read(name), expected)
                        self.assertEqual(manifest["files"][name], {
                            "sha256": hashlib.sha256(expected).hexdigest(),
                            "size_bytes": len(expected),
                        })

    def test_noto_notice_links_or_missing_files_refuse_the_entire_bundle(self):
        repo = self.root / "notice-fixture"
        self.graphics_fixture(repo)
        for source, _ in package.PACKAGE_FILES:
            target = repo / source
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes((package.ROOT / source).read_bytes())
        notice = repo / "ui/native/licenses/NotoSansMono-OFL.txt"
        original = self.root / "outside-notice"
        notice.rename(original)
        notice.symlink_to(original)
        with patch.object(package, "ROOT", repo):
            linked_output = self.root / "linked-notice.zip"
            with self.assertRaisesRegex(ValueError, "regular, single-link"):
                self.package_to(linked_output)
            self.assertFalse(linked_output.exists())
            notice.unlink()
            missing_output = self.root / "missing-notice.zip"
            with self.assertRaises(FileNotFoundError):
                self.package_to(missing_output)
            self.assertFalse(missing_output.exists())

    def test_rejects_changed_or_linked_font_atlas_before_creating_package(self):
        repo = self.root / "font-fixture"
        self.graphics_fixture(repo)
        atlas = repo / "ui/graphics/assets/open-sans-atlas.rle"
        with patch.object(package, "ROOT", repo):
            package.validate_font_provenance()
            original = atlas.read_bytes()
            atlas.write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError, "provenance"):
                package.validate_font_provenance()
            atlas.write_bytes(original)
            linked = self.root / "atlas-source"
            atlas.rename(linked)
            atlas.symlink_to(linked)
            with self.assertRaisesRegex(ValueError, "regular, single-link"):
                package.validate_font_provenance()

    def test_validates_exact_seven_app_esp_and_embedded_images(self):
        self.assertEqual(self.validated[1]["kernel_feature"], package.PROFILE)

    def test_accepts_session_preview_only_with_the_same_exact_app_and_esp_binding(self):
        self.manifest["kernel_feature"] = package.SESSION_PROFILE
        (self.build / "manifest.json").write_text(json.dumps(self.manifest))
        with patch.object(package, "inspect_user", side_effect=lambda path: {"name": path.stem}):
            _, result = package.validate_build(self.build, verify_source=False)
        self.assertEqual(result["kernel_feature"], package.SESSION_PROFILE)

    def test_package_preserves_the_exact_session_preview_profile(self):
        self.manifest["kernel_feature"] = package.SESSION_PROFILE
        (self.build / "manifest.json").write_text(json.dumps(self.manifest))
        output = self.root / "session-preview.zip"
        self.package_to(output)
        with zipfile.ZipFile(output) as archive:
            manifest = json.loads(archive.read("manifest.json"))
        self.assertEqual(manifest["kernel_feature"], package.SESSION_PROFILE)
        self.assertIn(package.SESSION_PROFILE, manifest["scope"])

    def test_rejects_extra_esp_payload(self):
        (self.esp / "unexpected.txt").write_text("extra")
        with patch.object(package, "inspect_user", side_effect=lambda path: {"name": path.stem}):
            with self.assertRaisesRegex(ValueError, "exactly"):
                package.validate_build(self.build, verify_source=False)

    def test_rejects_changed_userspace_elf(self):
        (self.apps_dir / (package.APP_NAMES[0] + ".elf")).write_bytes(b"changed")
        with patch.object(package, "inspect_user", side_effect=lambda path: {"name": path.stem}):
            with self.assertRaisesRegex(ValueError, "not bound"):
                package.validate_build(self.build, verify_source=False)

    def test_rejects_ambiguous_duplicate_manifest_keys(self):
        (self.build / "manifest.json").write_text('{"kernel_feature":"native-system-materiaal","kernel_feature":"other"}')
        with self.assertRaisesRegex(ValueError, "duplicate JSON key"):
            package.validate_build(self.build, verify_source=False)

    def test_rejects_stale_source(self):
        with patch.object(package, "source_hash", return_value="b" * 64):
            with self.assertRaisesRegex(ValueError, "source fingerprint"):
                package.validate_build(self.build)

    def test_rejects_symlink_in_esp(self):
        target = self.esp / "kernel.elf"
        target.unlink()
        target.symlink_to(self.apps_dir / (package.APP_NAMES[0] + ".elf"))
        with self.assertRaisesRegex(ValueError, "regular"):
            package.validate_build(self.build, verify_source=False)

    def package_to(self, output):
        firmware = self.root / "test-firmware"
        firmware.write_bytes(b"synthetic firmware and license")
        with (
            patch.object(package, "git_commit", return_value="1" * 40),
            patch.object(package, "inspect_user", side_effect=lambda path: {"name": path.stem}),
        ):
            return package.package_bundle(
            build_directory=self.build,
            output=output,
            ovmf_code=firmware,
            ovmf_vars=firmware,
            seabios_bios=firmware,
            vga_rom=firmware,
            ovmf_license=firmware,
            seabios_license=firmware,
            ovmf_version="test-ovmf",
            seabios_version="test-seabios",
            verify_source=False,
            )

    def test_normalizes_parent_components_before_creating_output(self):
        output = self.root / "unused" / ".." / "packages" / "bundle.zip"
        result = self.package_to(output)
        expected = self.root / "packages" / "bundle.zip"
        self.assertEqual(result["output"], str(expected))
        self.assertTrue(expected.is_file())
        self.assertFalse((self.root / "unused").exists())

    def test_rejects_symlink_before_creating_missing_output_parent(self):
        target = self.root / "target"
        target.mkdir()
        link = self.root / "linked"
        link.symlink_to(target, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "must not traverse symlinks"):
            self.package_to(link / "missing" / "bundle.zip")
        self.assertFalse((target / "missing").exists())

    def test_rechecks_output_parent_after_mkdir(self):
        target = self.root / "target"
        target.mkdir()
        parent = self.root / "packages"
        original_mkdir = Path.mkdir

        def redirected_mkdir(path, *args, **kwargs):
            original_mkdir(path, *args, **kwargs)
            if path == parent:
                path.rmdir()
                path.symlink_to(target, target_is_directory=True)

        with patch.object(Path, "mkdir", autospec=True, side_effect=redirected_mkdir):
            with self.assertRaisesRegex(ValueError, "must not traverse symlinks"):
                self.package_to(parent / "bundle.zip")
        self.assertEqual(list(target.iterdir()), [])

    def test_creates_hash_bound_bundle_and_refuses_overwrite(self):
        source = self.root / "firmware"
        source.mkdir()
        inputs = {}
        for name, data in {
            "ovmf-code.fd": b"ovmf-code",
            "ovmf-vars.fd": b"ovmf-vars",
            "seabios.bin": b"bios",
            "vga.rom": b"vga",
            "ovmf-license": b"ovmf license",
            "seabios-license": b"seabios license",
        }.items():
            path = source / name
            path.write_bytes(data)
            inputs[name] = path
        repo = self.root / "repo"
        (repo / "tools/windows-qemu").mkdir(parents=True)
        (repo / "tools/windows-qemu/Start-NieuwOS.ps1").write_text("launcher")
        (repo / "tools/windows-qemu/Acceleration.ps1").write_text("acceleration helper")
        (repo / "tools/windows-qemu/README.md").write_text("instructions")
        self.graphics_fixture(repo)
        output = self.root / "bundle.zip"
        with (
            patch.object(package, "ROOT", repo),
            patch.object(package, "git_commit", return_value="1" * 40),
            patch.object(package, "inspect_user", side_effect=lambda path: {"name": path.stem}),
        ):
            result = package.package_bundle(
                build_directory=self.build,
                output=output,
                ovmf_code=inputs["ovmf-code.fd"],
                ovmf_vars=inputs["ovmf-vars.fd"],
                seabios_bios=inputs["seabios.bin"],
                vga_rom=inputs["vga.rom"],
                ovmf_license=inputs["ovmf-license"],
                seabios_license=inputs["seabios-license"],
                ovmf_version="test-ovmf",
                seabios_version="test-seabios",
                verify_source=False,
            )
            with zipfile.ZipFile(output) as archive:
                manifest = json.loads(archive.read("manifest.json"))
                self.assertEqual(manifest["qemu"]["interactive_acceleration"], "auto-whpx-then-tcg")
                self.assertEqual(manifest["qemu"]["acceptance_acceleration"], "tcg,thread=single")
                self.assertIn("Acceleration.ps1", manifest["files"])
                for name, record in manifest["files"].items():
                    payload = archive.read(name)
                    self.assertEqual(len(payload), record["size_bytes"])
                    self.assertEqual(hashlib.sha256(payload).hexdigest(), record["sha256"])
            self.assertEqual(result["source_commit"], "1" * 40)
            with self.assertRaises(FileExistsError):
                package.package_bundle(
                    build_directory=self.build,
                    output=output,
                    ovmf_code=inputs["ovmf-code.fd"],
                    ovmf_vars=inputs["ovmf-vars.fd"],
                    seabios_bios=inputs["seabios.bin"],
                    vga_rom=inputs["vga.rom"],
                    ovmf_license=inputs["ovmf-license"],
                    seabios_license=inputs["seabios-license"],
                    ovmf_version="test-ovmf",
                    seabios_version="test-seabios",
                    verify_source=False,
                )
            raced_output = self.root / "raced.zip"
            original_record = package.file_record

            def changed_record(path, label, limit):
                if label == "esp/kernel.elf":
                    path.write_bytes(b"Changed after initial validation")
                return original_record(path, label, limit)

            with patch.object(package, "file_record", side_effect=changed_record):
                with self.assertRaisesRegex(ValueError, "changed while packaging"):
                    package.package_bundle(
                        build_directory=self.build,
                        output=raced_output,
                        ovmf_code=inputs["ovmf-code.fd"],
                        ovmf_vars=inputs["ovmf-vars.fd"],
                        seabios_bios=inputs["seabios.bin"],
                        vga_rom=inputs["vga.rom"],
                        ovmf_license=inputs["ovmf-license"],
                        seabios_license=inputs["seabios-license"],
                        ovmf_version="test-ovmf",
                        seabios_version="test-seabios",
                        verify_source=False,
                    )
            self.assertFalse(raced_output.exists())


if __name__ == "__main__":
    unittest.main()
