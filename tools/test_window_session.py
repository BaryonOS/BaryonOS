"""Adversarial host checks for the native runner, never OS execution."""
import unittest
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
    def test_result_transfer_cannot_be_combined_with_other_gate_modes(self):
        for flag in ("smoke", "recovery_handshake", "clipboard"):
            with self.assertRaises(ValueError):
                window.run(timeout=30, bios_rom=None, vga_rom=None,
                           calculator_result=True, **{flag: True})

    def test_result_transfer_requires_actual_answer_expression_and_original_binding(self):
        baseline = dict(window.snapshots(block("CALCULATOR"), "CALCULATOR")[0])
        state = dict(baseline, EXPRESSION_BYTES=3, EXPRESSION_SUM=window.volume_disk.fnv(b"7/2"),
                     CARET=3, CURRENT=1, EVALUATIONS=1, FAILURES=0, ERRORS=0, HAS_ANSWER=1,
                     ANSWER_LOW=3_500_000_000, ANSWER_HIGH=0, ACTIVE=1)
        self.assertTrue(window.calculator_result_matches(state, baseline, b"7/2", 1, 3_500_000_000))
        for field in state:
            if field not in ("GEN", "FRAMES"):
                damaged = dict(state)
                damaged[field] += 1
                self.assertFalse(window.calculator_result_matches(damaged, baseline, b"7/2", 1, 3_500_000_000), field)

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
