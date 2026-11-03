import importlib.util
import json
from pathlib import Path
from unittest import TestCase, main, mock
import subprocess
import sys
import tempfile
import types

ROOT = Path(__file__).resolve().parent.parent

def load(name):
    path = ROOT / name / 'native_probe.py'
    spec = importlib.util.spec_from_file_location('native_probe_' + name.replace('-', '_'), path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

class ProbeTests(TestCase):
    def test_help_does_not_import_missing_source(self):
        for name in ('native', 'native-current'):
            with self.subTest(name=name):
                res = subprocess.run([sys.executable, '-B', str(ROOT / name / 'native_probe.py'), '--help'], capture_output=True, text=True, timeout=5)
                self.assertEqual(res.returncode, 0, res.stderr)
                self.assertIn('--source', res.stdout)

    def test_source_state_checks_head_status_and_timeouts(self):
        for name in ('native', 'native-current'):
            m = load(name)
            with self.subTest(name=name), mock.patch.object(m, 'source_hash', lambda: 'digest', create=True), mock.patch.object(m.subprocess, 'check_output', side_effect=[m.EXPECTED_HEAD+'\n', '']) as call:
                self.assertEqual(m.source_state()['git_status'], [])
                self.assertTrue(all(c.kwargs['timeout'] == 15 for c in call.call_args_list))
            with mock.patch.object(m, 'source_hash', lambda: 'digest', create=True), mock.patch.object(m.subprocess, 'check_output', side_effect=[m.EXPECTED_HEAD, ' M somefile\n']):
                with self.assertRaisesRegex(RuntimeError, 'not clean'):
                    m.source_state()
            with mock.patch.object(m.subprocess, 'check_output', return_value='wrong'):
                with self.assertRaisesRegex(RuntimeError, 'source changed'):
                    m.source_state()
            with mock.patch.object(m.subprocess, 'check_output', side_effect=subprocess.TimeoutExpired('git', 15)):
                with self.assertRaises(subprocess.TimeoutExpired):
                    m.source_state()

    def test_actual_git_dirty_status_is_rejected(self):
        for name in ('native', 'native-current'):
            m = load(name)
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temp:
                p = Path(temp)
                subprocess.run(['git', 'init', '-q', str(p)], check=True)
                subprocess.run(['git', '-C', str(p), 'config', 'user.name', 'Test'], check=True)
                subprocess.run(['git', '-C', str(p), 'config', 'user.email', 'test@example.invalid'], check=True)
                (p/'tracked').write_text('baseline')
                subprocess.run(['git', '-C', str(p), 'add', 'tracked'], check=True)
                subprocess.run(['git', '-C', str(p), 'commit', '-qm', 'fixture'], check=True)
                head = subprocess.check_output(['git', '-C', str(p), 'rev-parse', 'HEAD'], text=True).strip()
                with mock.patch.object(m, 'SOURCE', p), mock.patch.object(m, 'EXPECTED_HEAD', head), mock.patch.object(m, 'source_hash', lambda: 'fixture', create=True):
                    self.assertEqual(m.source_state()['git_status'], [])
                    (p/'untracked').write_text('changed')
                    with self.assertRaisesRegex(RuntimeError, 'not clean'): m.source_state()

    def test_source_verified_before_helpers_import(self):
        for name in ('native', 'native-current'):
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temp:
                root = Path(temp); p = root/'checkout'; p.mkdir(); (p/'tools').mkdir()
                marker = root/'imported'
                (p/'tools/common.py').write_text(f"from pathlib import Path\nPath({str(marker)!r}).write_text('imported')\ndef environment(): return {{}}\ndef firmware(): return ()\ndef source_hash(): return 'chosen'\n")
                (p/'tools/volume_disk.py').write_text('')
                (p/'tools/run_volume.py').write_text('def expected_markers(*a): pass\ndef controlled(*a): pass\ndef numbers(*a): pass\ndef check_state(*a): pass\n')
                subprocess.run(['git', 'init', '-q', str(p)], check=True)
                subprocess.run(['git', '-C', str(p), 'config', 'user.name', 'Test'], check=True)
                subprocess.run(['git', '-C', str(p), 'config', 'user.email', 'test@example.invalid'], check=True)
                subprocess.run(['git', '-C', str(p), 'add', 'tools'], check=True)
                subprocess.run(['git', '-C', str(p), 'commit', '-qm', 'fixture'], check=True)
                head = subprocess.check_output(['git', '-C', str(p), 'rev-parse', 'HEAD'], text=True).strip()
                code = "import importlib.util,sys; spec=importlib.util.spec_from_file_location('p',sys.argv[1]); m=importlib.util.module_from_spec(spec); spec.loader.exec_module(m); m.EXPECTED_HEAD=sys.argv[3]; m.load_source(sys.argv[2]); assert str(m.SOURCE)==sys.argv[2]; assert m.source_hash()=='chosen'"
                def run(expected):
                    return subprocess.run([sys.executable, '-B', '-c', code, str(ROOT/name/'native_probe.py'), str(p), expected], capture_output=True, text=True, timeout=5)
                wrong = run('wrong')
                self.assertNotEqual(wrong.returncode, 0)
                self.assertIn('source changed', wrong.stderr)
                self.assertFalse(marker.exists())
                (p/'tools/common.py').write_text((p/'tools/common.py').read_text()+'# dirty\n')
                dirty = run(head)
                self.assertNotEqual(dirty.returncode, 0)
                self.assertIn('not clean', dirty.stderr)
                self.assertFalse(marker.exists())
                subprocess.run(['git', '-C', str(p), 'checkout', '--', 'tools/common.py'], check=True)
                clean = run(head)
                self.assertEqual(clean.returncode, 0, clean.stderr)
                self.assertTrue(marker.exists())

    def test_build_requires_clean_unchanged_source_before_and_after(self):
        for name in ('native', 'native-current'):
            m = load(name)
            clean = {'head': m.EXPECTED_HEAD, 'git_status': [], 'source_sha256': 'before'}
            changed = dict(clean, source_sha256='after')
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temp, mock.patch.object(m, 'ROOT', Path(temp)), mock.patch.object(m, 'build_environment', return_value={}), mock.patch.object(m.subprocess, 'run', return_value=types.SimpleNamespace(returncode=0, stdout='', stderr='')) as build:
                with mock.patch.object(m, 'source_state', side_effect=RuntimeError('source worktree is not clean')):
                    with self.assertRaisesRegex(RuntimeError, 'not clean'): m.build('test-volume-recover')
                    build.assert_not_called()
                with mock.patch.object(m, 'source_state', side_effect=[clean, RuntimeError('source worktree is not clean')]):
                    with self.assertRaisesRegex(RuntimeError, 'not clean'): m.build('test-volume-recover')
                with mock.patch.object(m, 'source_state', side_effect=[clean, changed]):
                    with self.assertRaisesRegex(RuntimeError, 'source changed'): m.build('test-volume-recover')
                    report=json.loads((Path(temp)/'builds/test-volume-recover/build-invocation.json').read_text())
                    self.assertFalse(report['source_unchanged'])
                with mock.patch.object(m, 'source_state', side_effect=[clean, clean]):
                    m.build('test-volume-recover')
                    report=json.loads((Path(temp)/'builds/test-volume-recover/build-invocation.json').read_text())
                    self.assertTrue(report['source_unchanged'])
                    self.assertEqual(report['source']['git_status'], [])

    def test_image_validation_and_short_reads(self):
        for name in ('native', 'native-current'):
            m=load(name)
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temp:
                p=Path(temp); img=p/'img'; img.write_bytes(b'\0'*(4*512+34*512))
                link=p/'link'; link.symlink_to(img)
                with mock.patch.object(m, 'prepare') as prepare:
                    with self.assertRaisesRegex(ValueError, 'regular image'):
                        m.run('x', link)
                    prepare.assert_not_called()
                out=p/'out'; out.mkdir()
                with mock.patch.object(m, 'prepare', return_value=(out, {}, {})):
                    with self.assertRaisesRegex(ValueError, 'short read'):
                        m.run('x', img)
                hard=p/'hard'; hard.hardlink_to(img)
                with self.assertRaisesRegex(ValueError, 'regular image'):
                    m.run('x', img)

    def test_qemu_version_timeout_propagates(self):
        for name in ('native', 'native-current'):
            m=load(name)
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temp:
                p=Path(temp); img=p/'img'; img.write_bytes(b'\0'*(4*512+2*34*512)); out=p/'out'; out.mkdir()
                with mock.patch.object(m, 'prepare', return_value=(out, {}, {})), mock.patch.object(m, 'environment', lambda: {'PATH': ''}, create=True), mock.patch.object(m.shutil, 'which', return_value='qemu'), mock.patch.object(m.subprocess, 'run', return_value=types.SimpleNamespace(returncode=0,stdout=b'',stderr=b'')), mock.patch.object(m.subprocess, 'check_output', side_effect=subprocess.TimeoutExpired('qemu',15)) as version:
                    with self.assertRaises(subprocess.TimeoutExpired): m.run('x', img)
                    self.assertEqual(version.call_args.kwargs['timeout'], 15)
                    self.assertFalse((out/'result.json').exists())

    def test_historical_runs_keep_their_recorded_cleanliness(self):
        # The old native run recorded an untracked file; current run was clean.
        self.assertIsNone(load('native').btrfs_admission_finding())
        self.assertIsNotNone(load('native-current').btrfs_admission_finding())

    def test_admission_requires_measured_tables_and_both_serial_markers(self):
        for name in ('native', 'native-current'):
            m=load(name)
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temp, mock.patch.object(m, 'ROOT', Path(temp)):
                self.assertIsNone(m.btrfs_admission_finding())
                p=Path(temp)/'runs/btrfs-readonly'; p.mkdir(parents=True)
                result=p/'result.json'; serial=p/'serial.txt'
                data={'source': {'head': m.EXPECTED_HEAD, 'git_status': []},
                      'source_unchanged': True, 'timed_out': False, 'raw_qemu_exit': 35,
                      'kind': 'native-foreign-admission-probe', 'label': 'btrfs-readonly',
                      'profile': 'test-volume-recover', 'data_disk_read_only': True,
                      'source_image_unchanged': True, 'probe_image_unchanged': True,
                      'first_table_all_zero': True, 'second_table_all_zero': True,
                      'source_image_sha256_before': 'a'*64, 'source_image_sha256_after': 'a'*64,
                      'probe_image_sha256_before': 'a'*64, 'probe_image_sha256_after': 'a'*64}
                result.write_text(json.dumps(data)); serial.write_text('VOLUME:APP_STEP:00000020\nVOLUME:FAIL:APP_RESULT\n')
                self.assertIn('(step 20)', m.btrfs_admission_finding()['observation'])
                serial.write_text('VOLUME:APP_STEP:00000019\nVOLUME:APP_STEP:00000020\nVOLUME:FAIL:APP_RESULT\n')
                self.assertIn('(step 20)', m.btrfs_admission_finding()['observation'])
                for text in ('VOLUME:APP_STEP:00000020\nVOLUME:APP_STEP:00000021\nVOLUME:FAIL:APP_RESULT\n',
                             'VOLUME:APP_STEP:00000020\nVOLUME:FAIL:APP_RESULT\nVOLUME:APP_STEP:00000021\n',
                             'VOLUME:FAIL:APP_RESULT\nVOLUME:APP_STEP:00000020\n',
                             'VOLUME:APP_STEP:00000020\nVOLUME:APP_STEP:invalid\nVOLUME:FAIL:APP_RESULT\n'):
                    serial.write_text(text)
                    self.assertIsNone(m.btrfs_admission_finding())
                serial.write_text('VOLUME:APP_STEP:00000020\nVOLUME:FAIL:APP_RESULT\n')
                for key, value in [('timed_out', True), ('raw_qemu_exit', 33),
                                   ('source_unchanged', False), ('data_disk_read_only', False),
                                   ('source_image_unchanged', False), ('probe_image_unchanged', False),
                                   ('source_image_sha256_after', 'b'*64), ('probe_image_sha256_before', ''),
                                   ('kind', 'native-control'), ('profile', 'native-volume')]:
                    altered=dict(data, **{key: value}); result.write_text(json.dumps(altered))
                    with self.subTest(field=key): self.assertIsNone(m.btrfs_admission_finding())
                altered=dict(data, source={'head': m.EXPECTED_HEAD, 'git_status': [' M source']})
                result.write_text(json.dumps(altered))
                self.assertIsNone(m.btrfs_admission_finding())
                altered=dict(data, source={'head': 'wrong', 'git_status': []})
                result.write_text(json.dumps(altered))
                self.assertIsNone(m.btrfs_admission_finding())
                result.write_text(json.dumps(data))
                serial.write_text('VOLUME:APP_STEP:00000201\nVOLUME:FAIL:APP_RESULT\n')
                self.assertIsNone(m.btrfs_admission_finding())
                serial.write_text('VOLUME:APP_STEP:00000020\n')
                self.assertIsNone(m.btrfs_admission_finding())
                serial.write_text('VOLUME:APP_STEP:00000020\nVOLUME:FAIL:APP_RESULT\n')
                data['second_table_all_zero']=False; result.write_text(json.dumps(data))
                self.assertIsNone(m.btrfs_admission_finding())
                result.write_text('{')
                self.assertIsNone(m.btrfs_admission_finding())

if __name__ == '__main__': main(verbosity=2)
