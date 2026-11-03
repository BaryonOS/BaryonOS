from pathlib import Path
import argparse, hashlib, json
parser = argparse.ArgumentParser(description='Hash artifacts in an existing evidence directory')
parser.add_argument('base', type=Path, help='existing artifact directory')
base = parser.parse_args().base
if not base.is_dir():
    parser.error(f'artifact directory does not exist: {base}')
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
evidence=json.loads((base/'evidence.json').read_text())
if not evidence['packages'] or not all(digest(base/p['local_path']) == p['apt_metadata_sha256'] for p in evidence['packages']):
    raise SystemExit('Package bytes do not match apt metadata')
if not all(digest(base/p['path']) == p['sha256'] for p in evidence['baseline']['immutable_windows_copies']):
    raise SystemExit('Baseline image hash mismatch')
if not all(digest(base/i['path']) == i['sha256'] for i in items):
    raise SystemExit('Artifact changed while hashing')
(base/'artifact-sha256.json').write_text(json.dumps({'algorithm':'SHA256','excludes':sorted(skip),'files':items},indent=2)+'\n')
(base/'SHA256SUMS.txt').write_text(''.join(f"{i['sha256']}  {i['path']}\n" for i in items))
print(json.dumps({'files_hashed_and_rechecked':len(items),'raw_logs_hashed':sum(i['path'].startswith('logs/') for i in items),'evidence_sha256':digest(base/'evidence.json'),'readme_sha256':digest(base/'README.md'),'packages_verified':len(evidence['packages']),'overall':evidence['overall_decision']},indent=2))
