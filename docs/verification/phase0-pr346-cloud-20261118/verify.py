import hashlib
import json
import subprocess
import sys
from pathlib import Path

pr = int(sys.argv[1])
out = Path(f'docs/verification/phase0-pr{pr}-cloud-20261118/integration.json')
data = json.loads(out.read_text())
def git(*args):
    return subprocess.check_output(['git', *args], text=True).strip()
shared = {'docs/PROJECT-STATE.md', 'docs/progress.json', 'docs/assets/progress.svg', 'docs/PROGRESS.md', 'README.md'}
changed = git('diff', '--name-only', 'origin/main').splitlines()
module_guides = {'sdk/examples/ipc-wait-probe/README.md', 'services/net-stack/THIRD-PARTY.md', 'userspace/IPC-PROBE.md'}
assert all(p.startswith('docs/') or p in {'README.md', '.coderabbit.yaml'} | module_guides for p in changed), changed
main_progress = json.loads(subprocess.check_output(['git', 'show', 'origin/main:docs/progress.json'], text=True))
current = json.loads(Path('docs/progress.json').read_text())
assert {k: v for k, v in main_progress.items() if k != 'assessment'} == {k: v for k, v in current.items() if k != 'assessment'}
data['fixed_progress_register_unchanged'] = True
data['changed_paths_from_main'] = changed
if pr == 346:
    manifest = json.loads(Path('docs/verification/phase0-english-integration-20261117/manifest.json').read_text())
    sources = []
    additive = {'docs/contracts/NATIVE-IPC-WAIT.md', 'docs/product/new-os-installer-spec.md'}
    for item in manifest['source_files']:
        p = item['path']
        raw = Path(p).read_bytes()
        digest = hashlib.sha256(raw).hexdigest()
        same = digest == item['sha256']
        if not same:
            assert p in additive, p
            old = subprocess.check_output(['git', 'show', data['parent'] + ':' + p], text=True).splitlines()
            new = iter(raw.decode().splitlines())
            assert all(any(x == line for x in new) for line in old), p
        sources.append({'path': p, 'source_head': item['source_head'], 'original_sha256': item['sha256'], 'current_sha256': digest, 'byte_identical': same, 'original_lines_preserved_in_order': True})
    data['reviewed_sources'] = sources
    data['source_summary'] = {'files': len(sources), 'byte_identical': sum(x['byte_identical'] for x in sources), 'additive_english_glosses': sorted(additive)}
else:
    expected = subprocess.check_output(['git', 'show', data['parent'] + ':.coderabbit.yaml'])
    assert Path('.coderabbit.yaml').read_bytes() == expected
    data['config_sha256'] = hashlib.sha256(expected).hexdigest()
    data['reviewed_config_byte_identical'] = True
data['preservation_exit_code'] = 0
data['status'] = 'local preservation verified; current-head hosted checks and independent final review required'
# out.write_text(json.dumps(data, indent=2) + '\n')
print(json.dumps({'pr': pr, 'changed_paths': len(changed), 'progress_register_unchanged': True, 'source_summary': data.get('source_summary'), 'config_unchanged': data.get('reviewed_config_byte_identical')}))
