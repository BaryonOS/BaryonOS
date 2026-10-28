"""Screenshot-parser checks only; these synthetic pixels are not VM evidence."""
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1]/'tools'))
from run_graphics import ppm, changed_bounds


def frame(width=4, height=3, pixels=None):
    return f'P6\n{width} {height}\n255\n'.encode() + (bytes(width*height*3) if pixels is None else pixels)


class FramebufferEvidence(unittest.TestCase):
    def test_exact_bounded_pixels_required(self):
        self.assertEqual(ppm(frame()), (4,3,bytes(36)))
        for bad in (frame()[:-1], frame()+b'x', frame().replace(b'255',b'256'),
                    b'P6\n0 3\n255\n', b'P3\n4 3\n255\n'+bytes(36),
                    b'P6\n1921 1\n255\n'+bytes(1921*3)):
            with self.assertRaises(ValueError): ppm(bad)

    def test_color_channels_and_inclusive_changed_pixel_bounds(self):
        pixels = bytearray(36)
        pixels[(1*4+2)*3+1] = 7
        self.assertEqual(changed_bounds(frame(),frame(pixels=pixels)), (2,1,3,2))
        pixels[0] = 1
        self.assertEqual(changed_bounds(frame(),frame(pixels=pixels)), (0,0,3,2))
        self.assertIsNone(changed_bounds(frame(),frame()))

    def test_resolution_substitution_and_truncated_images_fail(self):
        with self.assertRaises(ValueError): changed_bounds(frame(),frame(3,4))
        with self.assertRaises(ValueError): changed_bounds(frame(),frame()[:-1])


if __name__ == '__main__': unittest.main()
