import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from run_materiaal_preview import BOOT, READY, verify_frame, verify_ready_transcript


class MateriaalPreviewTests(unittest.TestCase):
    def test_requires_boot_before_manifest_and_each_native_ready_marker(self):
        transcript = "\n".join(("KS-01:PASS", *BOOT, *READY)).encode()
        self.assertEqual(verify_ready_transcript(transcript), list((*BOOT, *READY)))

    def test_accepts_independent_service_readiness_order(self):
        transcript = "\n".join((*BOOT, READY[0], READY[2], READY[1], *READY[3:])).encode()
        self.assertEqual(
            verify_ready_transcript(transcript),
            list((*BOOT, READY[0], READY[2], READY[1], *READY[3:])),
        )

    def test_rejects_missing_or_repeated_ready_markers(self):
        missing = "\n".join((*BOOT, *READY[:-1])).encode()
        duplicate = "\n".join((*BOOT, *READY, READY[2])).encode()
        with self.assertRaises(ValueError):
            verify_ready_transcript(missing)
        with self.assertRaises(ValueError):
            verify_ready_transcript(duplicate)

    def test_rejects_readiness_before_manifest_or_reordered_boot(self):
        for markers in (
            (*BOOT, READY[1], READY[0], *READY[2:]),
            (BOOT[1], BOOT[0], BOOT[2], *READY),
        ):
            with self.assertRaises(ValueError):
                verify_ready_transcript("\n".join(markers).encode())

    def test_rejects_guest_failure_even_after_readiness(self):
        transcript = "\n".join((*BOOT, *READY, "MATERIAAL:FAIL:TEST")).encode()
        with self.assertRaises(ValueError):
            verify_ready_transcript(transcript)

    def test_requires_a_nonblank_native_frame(self):
        pixels = bytearray(800 * 600 * 3)
        for index in range(0, len(pixels), 3):
            color = (index // 3) % 16
            pixels[index:index + 3] = bytes((color, color * 2, color * 3))
        frame = b"P6\n800 600\n255\n" + pixels
        self.assertEqual(verify_frame(frame)["distinct_colors"], 16)
        blank = b"P6\n800 600\n255\n" + bytes(800 * 600 * 3)
        with self.assertRaises(ValueError):
            verify_frame(blank)


if __name__ == "__main__":
    unittest.main()
