"""Focused host checks for portable B03 evidence capture."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
import capture_update
import test_update as update_gate


class Capture(unittest.TestCase):
    def test_capture_normalizes_paths_before_hashing(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            build = root / 'build'
            run = build / 'run' / 'update-example'
            run.mkdir(parents=True)
            destination = root / 'docs' / 'verification' / 'capture'
            report = {'passed': True, 'source_sha256': 'source', 'run_directory': str(run),
                      'state_directory': str(build / 'run' / 'state'),
                      'command': ['/usr/bin/qemu-system-x86_64'], 'stdout': 'cwd ' + str(root)}
            (run / 'result.json').write_text(json.dumps(report))
            for name in ('serial.txt', 'stderr.txt'):
                (run / name).write_text('fixture')
            summary = {'status': 'completed', 'passed': True, 'source_sha256': 'source',
                       'stages': [{'stage': 'install', 'run_directory': run.relative_to(root).as_posix(),
                                   'state_directory': str(build / 'run' / 'state')}],
                       'host': [{'stdout': 'build ' + str(root / 'target')}]}
            (build / 'update-gate.json').write_text(json.dumps(summary))
            with mock.patch.object(capture_update, 'ROOT', root), mock.patch.object(capture_update, 'BUILD', build), \
                 mock.patch('update_paths.ROOT', root), \
                 mock.patch.object(sys, 'argv', ['capture_update.py', '--destination', str(destination)]):
                self.assertEqual(capture_update.main(), 0)
            captured = json.loads((destination / 'update-gate.json').read_text())
            stage = json.loads((destination / '0-install' / 'result.json').read_text())
            self.assertEqual(captured['stages'][0]['run_directory'], 'build/run/update-example')
            self.assertEqual(stage['run_directory'], 'build/run/update-example')
            self.assertEqual(stage['state_directory'], 'build/run/state')
            self.assertEqual(stage['command'], ['qemu-system-x86_64'])
            self.assertNotIn(str(root), json.dumps([captured, stage]))
            self.assertNotIn('/usr/bin/', json.dumps([captured, stage]))
            for line in (destination / 'SHA256SUMS').read_text().splitlines():
                digest, name = line.split('  ', 1)
                self.assertEqual(digest, hashlib.sha256((destination / name).read_bytes()).hexdigest())


class GateConfiguration(unittest.TestCase):
    def test_image_copy_uses_configured_target(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            target = root / 'custom-target'
            kernel = target / 'x86_64-unknown-none' / 'release' / 'os-kernel'
            kernel.parent.mkdir(parents=True)
            kernel.write_bytes(b'kernel from configured target')
            build = root / 'build'
            images_file = build / 'update' / 'images.json'
            fake_build = lambda *a, **kw: {'kernel': {'sha256': hashlib.sha256(kernel.read_bytes()).hexdigest()},
                                          'loader': {'sha256': 'loader'}}
            with mock.patch.object(update_gate, 'BUILD', build), mock.patch.object(update_gate, 'TARGET', target), \
                 mock.patch.object(update_gate, 'KERNELS', {'k1': ('native-update', 1, 'kernel.elf')}), \
                 mock.patch.object(update_gate, 'MANIFESTS', {}), \
                 mock.patch.object(update_gate, 'JUNK_IMAGE', ('junk', 'junk.bin', 1)), \
                 mock.patch.object(update_gate, 'build', side_effect=fake_build), \
                 mock.patch.object(update_gate.run_update, 'IMAGES', images_file), \
                 mock.patch.object(update_gate.run_update, 'update_tool'), \
                 mock.patch.object(update_gate, 'environment', return_value={}):
                images = update_gate.prepare_images('fingerprint')
            self.assertEqual((build / 'update' / 'kernel.elf').read_bytes(), kernel.read_bytes())
            self.assertEqual(images['kernels']['k1'], hashlib.sha256(kernel.read_bytes()).hexdigest())

    def test_host_check_uses_environment_and_stops_on_failure(self):
        with tempfile.TemporaryDirectory() as temporary:
            environment = {'PATH': 'fixture', 'RUSTUP_AUTO_INSTALL': '0'}
            with mock.patch.object(update_gate, 'BUILD', Path(temporary)), \
                 mock.patch.object(update_gate, 'source_hash', return_value='fingerprint'), \
                 mock.patch.object(update_gate, 'test_inputs', return_value={}), \
                 mock.patch.object(update_gate, 'HOST', [['host-check']]), \
                 mock.patch.object(update_gate, 'environment', return_value=environment), \
                 mock.patch.object(update_gate.subprocess, 'run', return_value=SimpleNamespace(returncode=2, stdout='', stderr='')) as run, \
                 mock.patch.object(update_gate, 'prepare_images') as images:
                self.assertEqual(update_gate.main(), 1)
            self.assertEqual(run.call_args.kwargs['env'], environment)
            self.assertEqual(run.call_args.kwargs['cwd'], update_gate.ROOT)
            self.assertTrue(run.call_args.kwargs['capture_output'])
            self.assertTrue(run.call_args.kwargs['text'])
            images.assert_not_called()


if __name__ == '__main__':
    unittest.main()
