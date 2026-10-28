"""Screenshot-parser checks only; these synthetic pixels are not VM evidence."""
from pathlib import Path
import io
import json
import os
import sys
import time
import unittest
from unittest.mock import Mock
sys.path.insert(0, str(Path(__file__).resolve().parents[1]/'tools'))
from run_graphics import Qmp, QmpPipe, ppm, changed_bounds, frame_checks, valid_sequence, run_graphics


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

    def test_serial_geometry_order_and_actual_framebuffer_must_agree(self):
        markers = ['BOOT:EBS_OK','KERNEL:ENTRY','BOOTINFO:OK','KS-01:PASS','KS-02:PASS','KS-03:PASS',
                   'G0:FRAMEBUFFER:800:600:800:1','G0:MAPPINGS:PASS','G0:READY',
                   'G0:INPUT:1','G0:INPUT:2','G0:INPUT:1','G0:INPUT:2','G0:PASS','M1:PASS']
        output = '\n'.join(markers)+'\n'
        self.assertTrue(valid_sequence(output,800,600))
        self.assertFalse(valid_sequence(output,1024,768))
        for old,new in [('800:600:800:1','800:600:799:1'),('800:600:800:1','800:600:4097:1'),('800:600:800:1','800:600:800:2'),
                        ('800:600:800:1','foo'),('G0:READY','G0:READY\nM6B:PASS')]:
            self.assertFalse(valid_sequence(output.replace(old,new),800,600))
        for index in range(len(markers)-1):
            altered = markers.copy()
            altered[index],altered[index+1] = altered[index+1],altered[index]
            self.assertFalse(valid_sequence('\n'.join(altered),800,600))
        for index in range(len(markers)):
            self.assertFalse(valid_sequence('\n'.join(markers[:index]+markers[index+1:]),800,600))
            self.assertFalse(valid_sequence('\n'.join(markers[:index]+[markers[index]]+markers[index:]),800,600))

    def test_each_key_must_change_the_real_input_region(self):
        pixels = bytearray(96*224*3)
        pixels[:16] = bytes(range(16))
        initial = frame(96, 224, bytes(pixels))
        def changed(source, x, y, color):
            width, height, data = ppm(source)
            data = bytearray(data)
            offset = (y*width+x)*3
            data[offset:offset+3] = color
            return frame(width, height, bytes(data))
        first = changed(initial, 48, 194, b'abc')
        second = changed(first, 64, 194, b'def')
        final = changed(first, 64, 194, b'ghi')
        frames = [initial, first, second, first, final]
        checks, bounds, steps = frame_checks(*frames)
        self.assertTrue(all(checks.values()))
        self.assertIsNotNone(bounds)
        self.assertTrue(all(rect is not None for rect in steps))
        for index in range(1, 5):
            altered = frames.copy()
            altered[index] = altered[index-1]
            self.assertFalse(all(frame_checks(*altered)[0].values()))
        altered = frames.copy()
        altered[2] = changed(second, 0, 0, b'xyz')
        checks, bounds, steps = frame_checks(*altered)
        self.assertFalse(checks['only_input_row_changes'])
        self.assertEqual(steps[1][:2], (0, 0))


def qmp_stream(*replies):
    """Protocol-only double; never represented as QEMU execution."""
    control = Qmp.__new__(Qmp)
    control.socket = Mock()
    control.reader = io.BytesIO(b''.join(json.dumps(reply).encode()+b'\n' for reply in replies))
    control.sequence = 0
    control.pending = bytearray()
    control.trace = []
    control.deadline = time.monotonic()+10
    return control


class QmpEvidenceBoundary(unittest.TestCase):
    def test_response_is_bound_to_request_and_events_are_not_success(self):
        control = qmp_stream({'event':'STOP'}, {'return':{}, 'id':1})
        self.assertEqual(control.command('query-status'), {})
        self.assertEqual(control.trace[0]['request'], {'execute':'query-status','id':1})
        self.assertEqual(control.trace[0]['response'], {'return':{},'id':1})
        for response in ({'return':{}, 'id':2}, {'return':{}}, ['return'], None):
            with self.subTest(response=response):
                with self.assertRaises((RuntimeError, ValueError)):
                    qmp_stream(response).command('query-status')

    def test_error_partial_oversized_and_endless_event_streams_fail(self):
        with self.assertRaises(RuntimeError):
            qmp_stream({'error':{'class':'GenericError'},'id':1}).command('screendump')
        for raw in (b'', b'{"return":{}}', b'x'*(1024*1024)+b'\n'):
            control = qmp_stream()
            control.reader = io.BytesIO(raw)
            with self.assertRaises(RuntimeError): control.read()
        with self.assertRaises(RuntimeError):
            qmp_stream(*[{'event':'STOP'}]*1024).command('query-status')

    def test_all_qmp_reads_share_the_run_deadline(self):
        control = qmp_stream({'return':{},'id':1})
        control.deadline = time.monotonic()-1
        with self.assertRaises(TimeoutError): control.read()
        self.assertEqual(control.reader.tell(), 0)

    def test_unregistered_machine_is_rejected_before_build_or_vm(self):
        with self.assertRaises(ValueError): run_graphics(machine='q35')

    def test_pipe_transport_handles_buffered_responses_and_eof(self):
        read_fd, write_fd = os.pipe()
        control = QmpPipe.__new__(QmpPipe)
        control.reader = os.fdopen(read_fd, 'rb', buffering=0)
        command_read, command_write = os.pipe()
        control.writer = os.fdopen(command_write, 'wb', buffering=0)
        control.pending = bytearray()
        control.sequence, control.trace = 0, []
        control.deadline = time.monotonic()+5
        os.write(write_fd, b'{"event":"STOP"}\n{"return":{"ok":true},"id":1}\n')
        os.close(write_fd)
        try:
            self.assertEqual(control.command('query-status'), {'ok':True})
            self.assertEqual(json.loads(os.read(command_read, 4096)),
                             {'execute':'query-status','id':1})
            with self.assertRaises(RuntimeError): control.read()
        finally:
            control.close()
            os.close(command_read)

    def test_pipe_rejects_bad_partial_oversize_and_expired_buffered_data(self):
        for raw, exception in ((b'[]\n', ValueError), (b'{', RuntimeError),
                               (b'x'*(1024*1024), RuntimeError)):
            control = QmpPipe.__new__(QmpPipe)
            control.pending = bytearray(raw)
            control.deadline = time.monotonic()+5
            read_fd, write_fd = os.pipe()
            os.close(write_fd)
            control.reader = os.fdopen(read_fd, 'rb', buffering=0)
            try:
                with self.assertRaises(exception): control.read()
            finally:
                control.reader.close()
        control.pending = bytearray(b'{"return":{}}\n')
        control.deadline = time.monotonic()-1
        with self.assertRaises(TimeoutError): control.read()


if __name__ == '__main__': unittest.main()
