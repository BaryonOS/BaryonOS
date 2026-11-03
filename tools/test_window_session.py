"""Adversarial host checks for the native runner, never OS execution."""
import unittest
import subprocess
import sys
import run_window_session as window


def block(role="MATERIAL", **values):
    fields={key:0 for key in window.SNAPSHOT_FIELDS[role]}
    fields.update(READY=1, GEN=2, PID=4, SERVICE_GENERATION=1, SURFACE_EPOCH=9, INPUT_EPOCH=8)
    fields.update(values)
    prefix="WINDOW:"+role+":"
    body="".join(prefix+key+f":{value:020}\n" for key,value in fields.items())
    return (prefix+"SNAPSHOT:BEGIN\n"+body+prefix+"SNAPSHOT:END\n").encode()


def image(width,height,color):
    return f"P6\n{width} {height}\n255\n".encode()+bytes(color)*(width*height)


class WindowRunnerTests(unittest.TestCase):
    def test_cli_scale_help_is_valid_without_booting_or_touching_images(self):
        result = subprocess.run([sys.executable, window.__file__, "--help"],
                                capture_output=True, text=True, timeout=10, check=False)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("--scale {1,2}", result.stdout)
        self.assertIn("200%", result.stdout)

    def test_scale200_oracles_use_actual_app_and_shell_geometry(self):
        width, height, body, calc = window.profile_geometry(2)
        self.assertEqual((width, height, body, calc), (1920, 1080, (56, 228, 1120, 660), (1224, 280, 640, 608)))
        raw = image(width, height, (1, 2, 3))
        header = len(raw) - width * height * 3
        for rectangle, phase in ((body, "read-tool"), (calc, "clipboard-pasted")):
            expected = image(*rectangle[2:], (1, 2, 3))
            self.assertTrue(window.oracle_matches(raw, expected, phase, 2))
            x, y, w, h = rectangle
            for px, py in ((x, y), (x + w - 1, y + h - 1), (x + w // 2, y + h // 2)):
                changed = bytearray(raw)
                changed[header + (py * width + px) * 3] ^= 1
                self.assertFalse(window.oracle_matches(bytes(changed), expected, phase, 2))
        for phase, regions in window.workspace_regions(2).items():
            self.assertTrue(window.workspace_matches(raw, raw, phase, 2))
            for x, y, w, h in regions:
                window.crop(raw, (x, y, w, h))
                changed = bytearray(raw)
                changed[header + ((y + h // 2) * width + x + w // 2) * 3] ^= 1
                self.assertFalse(window.workspace_matches(bytes(changed), raw, phase, 2))
        for scale in (0, 3, -1):
            with self.assertRaises(ValueError):
                window.profile_geometry(scale)

    def test_document_tool_oracles_require_whole_body_without_masks(self):
        raw = image(1280, 720, (1, 2, 3))
        expected = image(880, 510, (1, 2, 3))
        header = len(raw) - 1280 * 720 * 3
        for phase in ("read-tool", "read-tool-blocked", "write-tool-restored"):
            self.assertIn(phase, window.FIXTURE_PHASES)
            self.assertTrue(window.oracle_matches(raw, expected, phase))
            for x, y in ((28, 114), (907, 623), (180, 160)):
                changed = bytearray(raw)
                changed[header + (y * 1280 + x) * 3] ^= 1
                self.assertFalse(window.oracle_matches(bytes(changed), expected, phase))

    def test_document_menu_oracles_require_whole_body_at_both_scales(self):
        phases = ("document-menu", "document-menu-next", "document-menu-read",
                  "document-menu-blocked", "document-menu-cancelled", "document-menu-write",
                  "document-menu-mouse", "document-menu-mouse-cancelled")
        for scale in (1, 2):
            width, height, body, _ = window.profile_geometry(scale)
            raw = image(width, height, (1, 2, 3))
            expected = image(*body[2:], (1, 2, 3))
            header = len(raw) - width * height * 3
            x, y, w, h = body
            for phase in phases:
                self.assertIn(phase, window.FIXTURE_PHASES)
                self.assertTrue(window.oracle_matches(raw, expected, phase, scale))
                for px, py in ((x, y), (x + w - 1, y + h - 1), (x + w // 2, y + h // 2)):
                    changed = bytearray(raw)
                    changed[header + (py * width + px) * 3] ^= 1
                    self.assertFalse(window.oracle_matches(bytes(changed), expected, phase, scale))

    def test_high_contrast_requires_entire_real_app_without_color_masks(self):
        for scale in (1, 2):
            width, height, body, calc = window.profile_geometry(scale)
            raw = image(width, height, (0, 0, 0))
            for rectangle, phases in (
                (body, ("document-menu-high-contrast", "document-menu-contrast-restored")),
                (calc, ("calculator-high-contrast", "calculator-contrast-restored")),
            ):
                expected = image(*rectangle[2:], (0, 0, 0))
                x, y, w, h = rectangle
                header = len(raw) - width * height * 3
                for phase in phases:
                    self.assertTrue(window.oracle_matches(raw, expected, phase, scale))
                    for px, py in ((x, y), (x + w - 1, y + h - 1), (x + w // 2, y + h // 2)):
                        changed = bytearray(raw)
                        changed[header + (py * width + px) * 3] ^= 1
                        self.assertFalse(window.oracle_matches(bytes(changed), expected, phase, scale))

    def test_keyboard_move_requires_every_shell_and_translated_app_pixel(self):
        for scale in (1, 2):
            width, height, body, _ = window.profile_geometry(scale)
            shell = image(width, height, (1, 2, 3))
            app = image(*body[2:], (4, 5, 6))
            header = len(shell) - width * height * 3
            for phase, (dx, dy) in window.WINDOW_MOVE_OFFSETS.items():
                x, y = body[0] + dx * scale, body[1] + dy * scale
                w, h = body[2:]
                actual = bytearray(shell)
                for row in range(h):
                    start = header + ((y + row) * width + x) * 3
                    actual[start:start + w * 3] = bytes((4, 5, 6)) * w
                self.assertTrue(window.window_move_matches(bytes(actual), shell, app, phase, scale))
                for px, py in ((x, y), (x + w - 1, y + h - 1), (x + w // 2, y + h // 2),
                               (0, 0), (width - 1, height - 1), (0, y + 1)):
                    changed = bytearray(actual)
                    changed[header + (py * width + px) * 3] ^= 1
                    self.assertFalse(window.window_move_matches(bytes(changed), shell, app, phase, scale))
                if dx or dy:
                    self.assertFalse(window.material_matches(bytes(actual), app, body))

    def test_notification_gate_checks_the_whole_opaque_panel_not_only_text(self):
        for scale in (1, 2):
            width, height, _, _ = window.profile_geometry(scale)
            expected_panel = ((width - 460 * scale, 64 * scale, 424 * scale, 216 * scale),)
            for phase in ("notice-copied", "notice-dismissed", "notice-empty-mouse"):
                self.assertEqual(window.workspace_regions(scale)[phase], expected_panel)
            self.assertEqual(window.workspace_regions(scale)["notice-closed"], ((0, 0, width, 40 * scale),))

    def test_workspace_checks_every_pixel_in_declared_shell_regions(self):
        raw = image(1280, 720, (1, 2, 3))
        header = len(raw) - 1280 * 720 * 3
        for phase, regions in window.WORKSPACE_REGIONS.items():
            self.assertTrue(window.workspace_matches(raw, raw, phase))
            for x, y, width, height in regions:
                for px, py in ((x, y), (x + width - 1, y + height - 1), (x + width // 2, y + height // 2)):
                    changed = bytearray(raw)
                    changed[header + (py * 1280 + px) * 3] ^= 1
                    self.assertFalse(window.workspace_matches(bytes(changed), raw, phase))
        with self.assertRaises(KeyError):
            window.workspace_matches(raw, raw, "unknown")

    def test_workspace_cannot_claim_diagnostic_only_success(self):
        for flags in ({"smoke": True}, {"recovery_handshake": True}):
            with self.assertRaises(ValueError):
                window.run(timeout=30, bios_rom=None, vga_rom=None, workspace=True, **flags)

    def test_clipboard_oracle_checks_calculator_body_without_internal_masks(self):
        raw = image(1280, 720, (1, 2, 3))
        expected = image(320, 484, (1, 2, 3))
        self.assertTrue(window.oracle_matches(raw, expected, "clipboard-pasted"))
        for x, y in ((932, 140), (1251, 623), (950, 300)):
            changed = bytearray(raw)
            offset = len(raw) - 1280 * 720 * 3 + (y * 1280 + x) * 3
            changed[offset] ^= 1
            self.assertFalse(window.oracle_matches(bytes(changed), expected, "clipboard-pasted"))

    def test_clipboard_cannot_be_combined_with_diagnostic_only_success(self):
        for flags in ({"smoke": True}, {"recovery_handshake": True}):
            with self.assertRaises(ValueError):
                window.run(timeout=30, bios_rom=None, vga_rom=None, clipboard=True, **flags)

    def test_exact_snapshot_rejects_missing_duplicate_or_oversized_fields(self):
        raw=block()
        self.assertEqual(window.snapshots(raw,"MATERIAL")[0]["PID"],4)
        for bad in (raw.replace(b"WINDOW:MATERIAL:PID:00000000000000000004\n",b""),
                    raw.replace(b"SNAPSHOT:END",b"PID:00000000000000000004\nWINDOW:MATERIAL:SNAPSHOT:END"),
                    raw.replace(b"00000000000000000004",b"99999999999999999999")):
            with self.assertRaises(ValueError):window.snapshots(bad,"MATERIAL")

    def test_unready_or_injected_snapshot_cannot_pass(self):
        with self.assertRaises(ValueError):window.snapshots(block(READY=0),"MATERIAL")
        with self.assertRaises(ValueError):window.snapshots(block()+b"\xff","MATERIAL")
        with self.assertRaises(ValueError):window.snapshots(block(),"UNKNOWN")

    def test_partial_record_does_not_certify_a_frame(self):
        raw=block()
        self.assertEqual(window.snapshots(raw[:-1],"MATERIAL"),[])
        self.assertEqual(window.snapshots(raw[:raw.index(b"SNAPSHOT:END")],"MATERIAL"),[])

    def test_whole_body_oracle_compares_every_pixel_without_masks(self):
        raw=image(1280,720,(1,2,3))
        expected=image(880,510,(1,2,3))
        self.assertTrue(window.material_matches(raw,expected))
        for x,y in ((28,114),(907,623),(100,200)):
            damaged=bytearray(raw)
            header=len(raw)-1280*720*3
            damaged[header+(y*1280+x)*3]^=1
            self.assertFalse(window.material_matches(bytes(damaged),expected))

    def test_crop_rejects_crossing_framebuffer_or_invalid_geometry(self):
        raw=image(32,8,(0,0,0))
        for rectangle in ((-1,0,1,1),(31,0,2,1),(0,7,1,2),(0,0,0,1)):
            with self.assertRaises(ValueError):window.crop(raw,rectangle)

    def test_surface_boundary_check_includes_system_and_other_app(self):
        raw=image(1280,720,(1,2,3))
        header=len(raw)-1280*720*3
        for x,y,expected in ((28,114,False),(907,623,False),(27,114,True),(908,623,True),(950,300,True),(50,30,True)):
            changed=bytearray(raw);changed[header+(y*1280+x)*3]^=1
            self.assertEqual(window.changed_outside(raw,bytes(changed),window.BODY),expected)

    def test_repeated_numeric_records_keep_order_and_require_canonical_width(self):
        raw=b"WINDOW:RESTARTS:00000000000000000001\nother\nWINDOW:RESTARTS:00000000000000000002\n"
        self.assertEqual(window.numbers(raw,"WINDOW:RESTARTS"),[1,2])
        self.assertEqual(window.numbers(b"WINDOW:RESTARTS:1\n","WINDOW:RESTARTS"),[])

    def test_compositor_requires_complete_current_frame_and_input_observations(self):
        prefix="WINDOW:COMPOSITOR:"
        fields={name:0 for name in window.SNAPSHOT_FIELDS["COMPOSITOR"]}
        fields.update(GENERATION=2,PID=9,MATERIAL=12,CALCULATOR=8,POINTER_X=1260,POINTER_Y=50)
        raw=(prefix+"SNAPSHOT:BEGIN\n"+"".join(prefix+name+f":{value:020}\n" for name,value in fields.items())+prefix+"SNAPSHOT:END\n").encode()
        self.assertEqual(window.snapshots(raw,"COMPOSITOR")[-1]["MATERIAL"],12)
        self.assertEqual(window.snapshots(raw[:-1],"COMPOSITOR"),[])
        for bad in (raw.replace(b"WINDOW:COMPOSITOR:PID:00000000000000000009\n",b""),
                    raw.replace(b"WINDOW:COMPOSITOR:FATAL:00000000000000000000",b"WINDOW:COMPOSITOR:FATAL:00000000000000000001")):
            with self.assertRaises(ValueError):window.snapshots(bad,"COMPOSITOR")


if __name__=="__main__":unittest.main()
