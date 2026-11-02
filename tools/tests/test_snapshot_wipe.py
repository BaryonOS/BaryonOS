"""Compile and exercise the real kernel wipe with real capability-core types."""
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]

@unittest.skipUnless(platform.system() == 'Linux' and platform.machine() == 'x86_64',
                     'guard-page harness requires Linux x86-64')
class SnapshotWipeTests(unittest.TestCase):
    def test_schema_guards_reject_growth_and_copy_types_with_invalid_zero(self):
        rustc = shutil.which('rustc')
        if not rustc:
            self.skipTest('existing Rust toolchain unavailable')
        env = dict(os.environ, RUSTUP_AUTO_INSTALL='0')
        schema = '''#[derive(Clone,Copy)]
pub struct Cap { pub handle:u64, pub rights:u64 CAP_EXTRA }
#[derive(Clone,Copy)]
pub struct Message { pub sender_pid:u64, pub tag:TAG, pub len:u32,
pub cap_count:u32, pub reserved:u64, pub caps:[Cap;2], pub data:[u8;256] MSG_EXTRA }
'''
        with tempfile.TemporaryDirectory(prefix='nieuw-os-wipe-schema-') as directory:
            directory = Path(directory)
            probe = directory/'probe.rs'
            module = ROOT/'kernel/src/m2/m3/snapshot_wipe.rs'
            probe.write_text(f'#[path="{module}"] mod snapshot_wipe;\n')
            for name, tag, cap_extra, msg_extra in [
                ('current', 'u64', '', ''),
                ('nonzero-message', 'core::num::NonZeroU64', '', ''),
                ('message-growth', 'u64', '', ', pub future:u64'),
                ('cap-growth', 'u64', ', pub future:u64', ''),
                ('nonzero-cap', 'u64', '', ''),
                ('coercing-wrapper', 'Word', '', ''),
            ]:
                fixture = schema.replace('TAG', tag).replace('CAP_EXTRA',cap_extra).replace('MSG_EXTRA',msg_extra)
                if name == 'nonzero-cap':
                    fixture = fixture.replace('pub rights:u64', 'pub rights:core::num::NonZeroU64')
                if name == 'coercing-wrapper':
                    fixture += '''
#[derive(Clone,Copy)] pub struct Word(pub u64);
impl core::ops::Deref for Word { type Target=u64; fn deref(&self)->&u64 { &self.0 } }
impl core::ops::DerefMut for Word { fn deref_mut(&mut self)->&mut u64 { &mut self.0 } }
'''
                source = directory/'fixture.rs'
                source.write_text(fixture)
                library = directory/'libfixture.rlib'
                built = subprocess.run([rustc,'+1.90.0','--edition','2021','--crate-name',
                    'capability_core','--crate-type','rlib',str(source),'-o',str(library)],
                    env=env,text=True,capture_output=True,timeout=30)
                self.assertEqual(built.returncode,0,built.stdout+built.stderr)
                checked = subprocess.run([rustc,'+1.90.0','--edition','2021','--crate-type',
                    'rlib','--extern',f'capability_core={library}',str(probe),'-o',str(directory/'probe.rlib')],
                    env=env,text=True,capture_output=True,timeout=30)
                if name == 'current':
                    self.assertEqual(checked.returncode,0,checked.stdout+checked.stderr)
                else:
                    self.assertNotEqual(checked.returncode,0,name)
                    self.assertIn('snapshot_wipe.rs',checked.stderr)

    def test_actual_word_wipe_optimized_and_guarded(self):
        cargo, rustc = shutil.which('cargo'), shutil.which('rustc')
        if not cargo or not rustc:
            self.skipTest('existing Rust toolchain unavailable')
        env = dict(os.environ, RUSTUP_AUTO_INSTALL='0', CARGO_PROFILE_RELEASE_PANIC='unwind')
        with tempfile.TemporaryDirectory(prefix='nieuw-os-snapshot-wipe-') as directory:
            directory = Path(directory)
            built = subprocess.run([cargo, '+1.90.0', 'build', '--offline', '--locked',
                '--release', '-p', 'capability-core', '--target-dir', str(directory/'target')],
                cwd=ROOT, env=env, text=True, capture_output=True, timeout=60)
            self.assertEqual(built.returncode, 0, built.stdout+built.stderr)
            library = directory/'target/release/libcapability_core.rlib'
            binary = directory/'wipe-tests'
            compiled = subprocess.run([rustc, '+1.90.0', '--edition', '2021', '--test',
                '-C', 'opt-level=3', '--extern', f'capability_core={library}',
                str(ROOT/'tools/tests/native_snapshot_wipe.rs'), '-o', str(binary)],
                cwd=ROOT, env=env, text=True, capture_output=True, timeout=60)
            self.assertEqual(compiled.returncode, 0, compiled.stdout+compiled.stderr)
            result = subprocess.run([str(binary), '--test-threads=1'], cwd=ROOT,
                env=env, text=True, capture_output=True, timeout=60)
            self.assertEqual(result.returncode, 0, result.stdout+result.stderr)
            self.assertIn('4 passed', result.stdout)

if __name__ == '__main__':
    unittest.main()
