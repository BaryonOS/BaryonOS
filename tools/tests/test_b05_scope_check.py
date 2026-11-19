"""Inventory validation fixtures; no security audit evidence."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import b05_scope_check as scope


class ScopeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.data = json.loads((scope.ROOT / scope.SCOPE).read_text())
        (self.root / scope.SCOPE).parent.mkdir(parents=True)
        for row in self.data['boundaries']:
            for key in ('files', 'tests', 'evidence'):
                for name in row[key]:
                    p = self.root / name
                    p.parent.mkdir(parents=True, exist_ok=True)
                    p.write_text('synthetic reference')

    def check(self, data=None):
        (self.root / scope.SCOPE).write_text(json.dumps(self.data if data is None else data))
        return scope.check(self.root)

    def test_complete_proposal_and_required_boundaries(self):
        self.assertEqual(self.check()['boundaries'], 7)
        for mutate in (lambda d: d.update(status='ACCEPTED'), lambda d: d['boundaries'].pop(),
                       lambda d: d['boundaries'][0].update(invariants=[]),
                       lambda d: d['boundaries'][0].update(tests=[]),
                       lambda d: d['boundaries'][0].update(id='unknown')):
            data = copy.deepcopy(self.data)
            mutate(data)
            with self.assertRaises(ValueError):
                self.check(data)

    def test_every_missing_or_escaping_reference_fails(self):
        for field in ('files', 'tests', 'evidence'):
            for name in ('missing/file', '../escape', '/etc/passwd', 'C:\\escape', 'docs//bad'):
                data = copy.deepcopy(self.data)
                data['boundaries'][0][field] = [name]
                with self.assertRaises((ValueError, OSError)):
                    self.check(data)
        with tempfile.TemporaryDirectory() as outside:
            p = Path(outside) / 'reference'
            p.write_text('outside')
            (self.root / 'docs/link').symlink_to(p)
            self.data['boundaries'][0]['evidence'] = ['docs/link']
            with self.assertRaises(ValueError):
                self.check()


if __name__ == '__main__':
    unittest.main()
