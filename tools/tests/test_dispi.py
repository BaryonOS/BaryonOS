"""Host checks of the H03 DISPI runner parser and pixel oracle; no QEMU."""
from pathlib import Path
import re
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_dispi as dispi

HEAD = ['BOOT:EBS_OK', 'KERNEL:ENTRY', 'BOOTINFO:OK', 'KS-01:PASS', 'KS-02:PASS', 'KS-03:PASS',
        'DISPI:GOP_BASE:0000000080000000', 'DISPI:GOP_BYTES:0000000000384000']
DEVICE = ['DISPI:BAR0_RAW:0000000080000008', 'DISPI:COMMAND:0000000000000003',
          'DISPI:ID:000000000000b0c5',
          'DISPI:VRAM:0000000001000000']


def positive_lines():
    lines = HEAD + DEVICE
    for width, height in dispi.MODES:
        lines += [f'DISPI:READBACK:{width}:{height}:32:65:{width}:{16 * 1024 * 1024 // (width * 4)}:0:0',
                  f'DISPI:MODE:{width}x{height}:SET']
    return lines + ['DISPI:PASS', 'M1:PASS']


def screendump(mode, width=None, height=None):
    width, height = (width, height) if width else dispi.MODES[mode]
    pixels = bytearray()
    for y in range(height):
        for x in range(width):
            pixels.extend(dispi.pattern(mode, x, y))
    return b'P6\n%d %d\n255\n' % (width, height) + bytes(pixels)


class Oracle(unittest.TestCase):
    def test_pattern_matches_the_kernel_formula(self):
        self.assertEqual(dispi.pattern(0, 0, 0), (0, 0, 0))
        self.assertEqual(dispi.pattern(1, 2, 3), (0x46, 0x34, 2 ^ 3))
        self.assertEqual(dispi.pattern(3, 1279, 719),
                         ((1279 * 3 + 0xc0) & 0xff, (719 * 5 + 0x6f) & 0xff,
                          (4 * 0x40 + 2 * 0x10 + (1279 ^ 719)) & 0xff))

    def test_exact_screendump_passes_for_every_mode(self):
        for mode in range(len(dispi.MODES)):
            result = dispi.check_screen(screendump(mode), mode)
            self.assertTrue(result['dimensions'] and result['pixels'], result)

    def test_one_tampered_pixel_is_rejected_with_its_location(self):
        raw = bytearray(screendump(1))
        header = len(b'P6\n800 600\n255\n')
        at = header + (5 * 800 + 7) * 3 + 2
        raw[at] ^= 1
        result = dispi.check_screen(bytes(raw), 1)
        self.assertTrue(result['dimensions'])
        self.assertFalse(result['pixels'])
        self.assertEqual(result['mismatched_pixels'], 1)
        self.assertEqual((result['first_mismatch']['x'], result['first_mismatch']['y']), (7, 5))

    def test_wrong_dimensions_and_stale_mode_frames_are_rejected(self):
        self.assertFalse(dispi.check_screen(screendump(0, 1280, 720), 0)['dimensions'])
        self.assertFalse(dispi.check_screen(screendump(0, 640, 481), 0)['dimensions'])
        # Right geometry, previous mode's pattern: a leftover frame must fail.
        stale = screendump(0, 1024, 768)
        self.assertFalse(dispi.check_screen(stale, 2)['pixels'])

    def test_malformed_screendump_is_rejected(self):
        raw = screendump(0)
        for bad in (raw[:-1], b'P5' + raw[2:], b''):
            result = dispi.check_screen(bad, 0)
            self.assertFalse(result['dimensions'] or result['pixels'])


class Serial(unittest.TestCase):
    def check(self, lines, case='positive'):
        return dispi.validate_serial('\n'.join(lines) + '\n', case)

    def test_exact_positive_and_negative_transcripts_pass(self):
        self.assertEqual(self.check(positive_lines()), (True, None))
        self.assertEqual(self.check(HEAD + ['DISPI:REFUSED:DEVICE', 'M1:FAIL:DISPI_REFUSED'], 'negative'),
                         (True, None))
        noisy = ['UEFI noise'] + positive_lines()
        self.assertTrue(self.check(noisy)[0])

    def test_bar0_must_be_a_32bit_memory_bar_at_the_gop_base(self):
        for raw in ('0000000090000008', '0000000080000001', '000000008000000c'):
            lines = positive_lines()
            lines[8] = 'DISPI:BAR0_RAW:' + raw
            self.assertFalse(self.check(lines)[0], raw)

    def test_command_register_must_enable_io_and_memory_decode(self):
        for raw in ('0000000000000001', '0000000000000002', '0000000000000000', '0000000000010003'):
            lines = positive_lines()
            lines[9] = 'DISPI:COMMAND:' + raw
            self.assertFalse(self.check(lines)[0], raw)
        lines = positive_lines()
        lines[9] = 'DISPI:COMMAND:0000000000000007'
        self.assertEqual(self.check(lines), (True, None))

    def test_refusal_tokens_follow_the_closed_grammar(self):
        decode = HEAD + DEVICE[:2] + ['DISPI:REFUSED:DECODE', 'M1:FAIL:DISPI_REFUSED']
        self.assertEqual(self.check(decode), (False, 'refused: DECODE'))
        self.assertEqual(self.check(HEAD + ['DISPI:REFUSED:BOGUS', 'M1:FAIL:DISPI_REFUSED'], 'negative'),
                         (False, 'unknown refusal token'))
        kernel = (dispi.ROOT/'kernel/src/dispi.rs').read_text()
        self.assertEqual(tuple(re.findall(r'refuse\(b"([A-Z0-9_]+)"\)', kernel)), dispi.REFUSALS)

    def test_id_vram_readback_and_markers_are_exact(self):
        for index, value in ((10, 'DISPI:ID:000000000000b0bf'),
                             (11, 'DISPI:VRAM:0000000000200000'),
                             (12, 'DISPI:READBACK:640:480:24:65:640:6553:0:0'),
                             (12, 'DISPI:READBACK:640:480:32:193:640:6553:0:0'),
                             (12, 'DISPI:READBACK:640:480:32:65:800:6553:0:0'),
                             (12, 'DISPI:READBACK:640:480:32:65:640:479:0:0'),
                             (12, 'DISPI:READBACK:640:480:32:65:640:6553:0:1'),
                             (13, 'DISPI:MODE:800x600:SET'),
                             (-2, 'G0:PASS')):
            lines = positive_lines()
            lines[index] = value
            self.assertFalse(self.check(lines)[0], value)

    def test_missing_duplicate_or_extra_mode_markers_fail(self):
        lines = positive_lines()
        self.assertFalse(self.check(lines[:13] + lines[15:])[0])
        self.assertFalse(self.check(lines[:14] + lines[12:])[0])
        self.assertFalse(self.check(lines + ['M1:PASS'])[0])

    def test_negative_must_refuse_without_any_mode(self):
        for tail in (['DISPI:REFUSED:BAR0', 'M1:FAIL:DISPI_REFUSED'],
                     ['DISPI:REFUSED:DECODE', 'M1:FAIL:DISPI_REFUSED'],
                     DEVICE[:2] + ['DISPI:REFUSED:DECODE', 'M1:FAIL:DISPI_REFUSED'],
                     ['DISPI:REFUSED:DEVICE'],
                     DEVICE + ['DISPI:REFUSED:DEVICE', 'M1:FAIL:DISPI_REFUSED'],
                     ['DISPI:MODE:640x480:SET', 'DISPI:REFUSED:DEVICE', 'M1:FAIL:DISPI_REFUSED']):
            self.assertFalse(self.check(HEAD + tail, 'negative')[0], tail)
        self.assertFalse(self.check(positive_lines(), 'negative')[0])


if __name__ == '__main__':
    unittest.main()
