"""Run the actual x86-64 kernel intrinsics against guard pages on Linux hosts."""
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]

@unittest.skipUnless(platform.system() == 'Linux' and platform.machine() == 'x86_64',
                     'guard-page/assembly harness requires Linux x86-64')
class KernelMemoryBoundaryTests(unittest.TestCase):
    def test_actual_intrinsics_optimized_guard_page_harness(self):
        rustc = shutil.which('rustc')
        if rustc is None:
            self.skipTest('existing Rust toolchain unavailable')
        with tempfile.TemporaryDirectory(prefix='nieuw-os-memory-') as directory:
            binary = Path(directory) / 'boundaries'
            built = subprocess.run([rustc, '+1.90.0', '--edition', '2021', '--test',
                '-C', 'opt-level=3', str(ROOT / 'tools/tests/native_memory_boundaries.rs'),
                '-o', str(binary)], cwd=ROOT, text=True, capture_output=True, timeout=60)
            self.assertEqual(built.returncode, 0, built.stdout + built.stderr)
            ran = subprocess.run([str(binary), '--test-threads=1'], cwd=ROOT,
                text=True, capture_output=True, timeout=60)
            self.assertEqual(ran.returncode, 0, ran.stdout + ran.stderr)
            self.assertIn('5 passed', ran.stdout)

if __name__ == '__main__':
    unittest.main()
