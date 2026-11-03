from pathlib import Path
import hashlib,json
base=Path('/mnt/c/Users/simon/Documents/Codex/2026-11-02/native-storage-gate/openzfs')
skip={'artifact-sha256.json','SHA256SUMS.txt'}
def digest(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        for c in iter(lambda:f.read(1024*1024),b''): h.update(c)
    return h.hexdigest()
items=[]
for p in sorted(base.rglob('*')):
    if p.is_file() and p.name not in skip:
        items.append({'path':p.relative_to(base).as_posix(),'bytes':p.stat().st_size,'sha256':digest(p)})
(base/'artifact-sha256.json').write_text(json.dumps({'algorithm':'SHA256','excludes':sorted(skip),'files':items},indent=2)+'\n')
(base/'SHA256SUMS.txt').write_text(''.join(f"{i['sha256']}  {i['path']}\n" for i in items))
check=json.loads((base/'artifact-sha256.json').read_text())
assert all(digest(base/i['path'])==i['sha256'] for i in check['files'])
evidence=json.loads((base/'evidence.json').read_text())
assert all(p['matches_apt_metadata'] for p in evidence['packages'])
assert all(digest(base/p['path'])==p['sha256'] for p in evidence['baseline']['immutable_windows_copies'])
print(json.dumps({'files_hashed_and_rechecked':len(items),'raw_logs_hashed':sum(i['path'].startswith('logs/') for i in items),'evidence_sha256':digest(base/'evidence.json'),'readme_sha256':digest(base/'README.md'),'packages_verified':len(evidence['packages']),'overall':evidence['overall_decision']},indent=2))