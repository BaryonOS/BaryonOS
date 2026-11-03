"""Synthetic tests for the archive's fail-closed evidence checks; no VM runs."""
import copy
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

DIR = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('archive_native', DIR / 'archive_native.py')
archive = importlib.util.module_from_spec(spec)
spec.loader.exec_module(archive)

def digest(b):
    return hashlib.sha256(b).hexdigest()

def put(path, obj):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj), encoding='utf8')

def manifest(proof):
    entries = []
    for path in sorted(proof.rglob('*')):
        if path.is_file() and path.name != 'ARTIFACTS.json':
            b = path.read_bytes()
            entry = dict(path=path.relative_to(proof).as_posix(), bytes=len(b), sha256=digest(b))
            if path.name.endswith('.gz'):
                raw = gzip.decompress(b)
                entry.update(uncompressed_bytes=len(raw), uncompressed_sha256=digest(raw))
            entries.append(entry)
    put(proof / 'ARTIFACTS.json', {'files': entries})

class ArchiveChecks(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.proof = self.root / 'proof'
        self.proof.mkdir()
        (self.proof / 'root-verification.json').write_text('previous report')
        (self.proof / 'verify_archive.py').write_bytes((DIR / 'verify_archive.py').read_bytes())
        self.source = self.root / 'source'
        (self.source / 'tools').mkdir(parents=True)
        (self.source / 'tools/common.py').write_text('def source_hash(): return "frozen-hash"\n')
        for cmd in (['git','init','-q'], ['git','config','user.name','Test'], ['git','config','user.email','test@example.invalid'], ['git','add','.'], ['git','commit','-qm','fixture']):
            subprocess.run(cmd, cwd=self.source, check=True, capture_output=True)
        head = subprocess.check_output(['git','rev-parse','HEAD'], cwd=self.source, text=True).strip()
        put(self.proof / 'native/freeze.json', {'head':head, 'source_sha256':'frozen-hash'})
        put(self.proof / 'native/matrix.json', {'fixture':'matrix'})
        put(self.proof / 'native/regression/build/volume-gate.json', {'fixture':'gate'})
        for name, field, path in [('matrix','matrix_sha256','native/matrix.json'), ('regression','gate_sha256','native/regression/build/volume-gate.json')]:
            put(self.proof / f'independent-native-review/{name}-review.json', {'passed':True, field:digest((self.proof / path).read_bytes())})
        original = self.root / 'original'
        original.write_bytes(b'fixture data')
        stored = gzip.compress(original.read_bytes(), mtime=0)
        (self.proof / 'trace.gz').write_bytes(stored)
        put(self.proof / 'native-import.json', {'files':[dict(path='trace.gz', stored_sha256=digest(stored), stored_bytes=len(stored), gzip_lossless=True, source_sha256=digest(original.read_bytes()), source_bytes=len(original.read_bytes()), source_path=str(original))]})
        manifest(self.proof)

    def run_verifier(self, mode=None, optimized=False):
        cmd = [sys.executable, '-B'] + (['-O'] if optimized else []) + [str(DIR / 'verify_archive.py')]
        if mode: cmd.append(mode)
        return subprocess.run(cmd + ['--source',str(self.source),'--proof',str(self.proof)], capture_output=True, text=True)

    def test_manifest_and_full_optimized_reject_corruption_before_report(self):
        self.assertEqual(self.run_verifier('manifest', True).returncode, 0)
        # The committed location must work without a source checkout.
        standalone = subprocess.run([sys.executable, '-B', '-O', str(self.proof / 'verify_archive.py'), 'manifest'], capture_output=True, text=True)
        self.assertEqual(standalone.returncode, 0, standalone.stderr)
        run = self.run_verifier(optimized=True)
        self.assertEqual(run.returncode, 0, run.stderr)
        self.assertEqual(self.run_verifier('manifest', True).returncode, 0)
        before = (self.proof / 'root-verification.json').read_bytes()
        (self.proof / 'trace.gz').write_bytes(b'broken')
        for mode in ('manifest', None):
            self.assertNotEqual(self.run_verifier(mode, True).returncode, 0)
        self.assertEqual((self.proof / 'root-verification.json').read_bytes(), before)

    def test_gzip_metadata_is_checked_under_optimization(self):
        original = json.loads((self.proof / 'ARTIFACTS.json').read_text())
        for key, value in [('bytes', 0), ('sha256', 'wrong'),
                           ('uncompressed_bytes', 0), ('uncompressed_sha256', 'wrong')]:
            changed = copy.deepcopy(original)
            next(entry for entry in changed['files'] if entry['path'] == 'trace.gz')[key] = value
            put(self.proof / 'ARTIFACTS.json', changed)
            self.assertNotEqual(self.run_verifier('manifest', True).returncode, 0, key)
        for key in ('uncompressed_bytes', 'uncompressed_sha256'):
            changed = copy.deepcopy(original)
            del next(entry for entry in changed['files'] if entry['path'] == 'trace.gz')[key]
            put(self.proof / 'ARTIFACTS.json', changed)
            self.assertNotEqual(self.run_verifier('manifest', True).returncode, 0, key)

    def test_manifest_rejects_missing_extra_or_duplicate_entries(self):
        original = json.loads((self.proof / 'ARTIFACTS.json').read_text())
        for entries in ([], original['files'][1:], original['files'] + [original['files'][0]]):
            put(self.proof / 'ARTIFACTS.json', {'files': entries})
            self.assertNotEqual(self.run_verifier('manifest', True).returncode, 0)
        (self.proof / 'native/matrix.json').unlink()
        manifest(self.proof)
        self.assertNotEqual(self.run_verifier('manifest', True).returncode, 0)

    def test_first_full_verification_adds_generated_outputs(self):
        (self.proof / 'root-verification.json').unlink()
        (self.proof / 'verify_archive.py').unlink()
        manifest(self.proof)
        result = self.run_verifier(optimized=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.run_verifier('manifest', True).returncode, 0)

    def test_source_and_import_mismatches_stop_before_report(self):
        report = (self.proof / 'root-verification.json').read_bytes()
        (self.root / 'original').write_bytes(b'different')
        self.assertNotEqual(self.run_verifier(optimized=True).returncode, 0)
        self.assertEqual((self.proof / 'root-verification.json').read_bytes(), report)
        (self.root / 'original').write_bytes(b'fixture data')
        (self.source / 'tools/common.py').write_text('def source_hash(): return "wrong"\n')
        self.assertNotEqual(self.run_verifier(optimized=True).returncode, 0)
        self.assertEqual((self.proof / 'root-verification.json').read_bytes(), report)

    def test_changed_and_untracked_source_are_rejected_under_optimization(self):
        common = self.source / 'tools/common.py'
        common.write_text(common.read_text() + '# tracked runtime change\n')
        self.assertNotEqual(self.run_verifier(optimized=True).returncode, 0)
        subprocess.run(['git', 'checkout', '--', 'tools/common.py'], cwd=self.source, check=True)
        (self.source / 'unexpected.py').write_text('unexpected\n')
        self.assertNotEqual(self.run_verifier(optimized=True).returncode, 0)

    def test_receipt_hash_missing_or_wrong_fails_even_with_valid_manifest(self):
        receipt_path = self.proof / 'independent-native-review/matrix-review.json'
        original = json.loads(receipt_path.read_text())
        before = (self.proof / 'root-verification.json').read_bytes()
        for value in (None, 'wrong'):
            changed = dict(original)
            if value is None: del changed['matrix_sha256']
            else: changed['matrix_sha256'] = value
            put(receipt_path, changed)
            manifest(self.proof)
            self.assertNotEqual(self.run_verifier(optimized=True).returncode, 0)
            self.assertEqual((self.proof / 'root-verification.json').read_bytes(), before)
        put(receipt_path, original)
        regression = self.proof / 'independent-native-review/regression-review.json'
        gate_receipt = json.loads(regression.read_text())
        gate_receipt['gate_sha256'] = 'wrong'
        put(regression, gate_receipt)
        manifest(self.proof)
        self.assertNotEqual(self.run_verifier(optimized=True).returncode, 0)
        regression.unlink()
        manifest(self.proof)
        self.assertNotEqual(self.run_verifier(optimized=True).returncode, 0)

    def test_archive_requires_confirmed_gate_before_copy(self):
        native = self.root / 'native'
        source = {'head':'a', 'source_sha256':'b'}
        labels = ['own-normal', 'own-cold-recover'] + [
            f'{candidate}-{mode}' for candidate in ('btrfs', 'zero', 'zero-prefix-payload-tail', 'openzfs-leaf')
            for mode in ('readonly', 'writable')]
        matrix = {'passed':True, 'source_unchanged':True, 'source':source,
                  'runs':[{'label': label, 'passed': True} for label in labels]}
        regression = {'passed':True, 'returncode':0, 'source_unchanged':True, 'source':source, 'gate_passed':True, 'stage_count':9}
        plan = [('normal',None),('recover','normal'),('cut-data',None),('recover-cut','cut-data'),('cut-table',None),('recover-cut','cut-table'),('cut-txn',None),('recover-cut','cut-txn'),('corrupt',None)]
        gate = {'passed':True,'status':'completed','source_sha256':'b','host':[{'exit':0}]*3,'stages':[{'case':c,'parent':p,'passed':True} for c,p in plan]}
        (self.root / 'independent-native-review').mkdir()
        archive.ROOT, archive.DEST = self.root, self.root / 'dest'
        for field, target in [('source_unchanged','matrix'), ('source_unchanged','regression'), ('gate_passed','regression'), ('passed','gate'), ('stages','gate')]:
            m, r, g = copy.deepcopy(matrix), copy.deepcopy(regression), copy.deepcopy(gate)
            del {'matrix':m, 'regression':r, 'gate':g}[target][field]
            put(native / 'matrix.json',m); put(native / 'regression/invocation.json',r); put(native / 'regression/build/volume-gate.json',g)
            with self.assertRaises(RuntimeError, msg=f'{target}.{field}'): archive.archive()
            self.assertFalse(archive.DEST.exists())
        duplicate = copy.deepcopy(matrix)
        duplicate['runs'][0]['label'] = duplicate['runs'][1]['label']
        put(native / 'matrix.json',duplicate); put(native / 'regression/invocation.json',regression); put(native / 'regression/build/volume-gate.json',gate)
        with self.assertRaises(RuntimeError): archive.archive()
        self.assertFalse(archive.DEST.exists())
        put(native / 'matrix.json',matrix); put(native / 'regression/invocation.json',regression); put(native / 'regression/build/volume-gate.json',gate)
        archive.archive()
        self.assertTrue((archive.DEST / 'native-import.json').exists())

if __name__ == '__main__':
    unittest.main()
