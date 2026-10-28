"""Copy only checker inputs and change one archived seed for a negative test."""
import hashlib
import json
from pathlib import Path
import shutil
import sys

root = Path(__file__).resolve().parent
source, destination = (root / argument for argument in sys.argv[1:])
for path in (source, destination):
    path.resolve().relative_to(root)
checker = Path('docs/verification/m6b-review/merge-followup/verify_m5_archive.py')
archive = Path('docs/verification/regression-m6b/m5')
shutil.copytree(source / 'tools', destination / 'tools')
for relative in (checker, archive / 'guest-runs.json', archive / 'm5-two-image-check.json'):
    target = destination / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source / relative, target)
guest_file = destination / archive / 'guest-runs.json'
before = guest_file.read_bytes()
guests = json.loads(before)
guest = next(guest for guest in guests if guest['label'] == 'm5-read-a')
original_seed = guest['disk_oracle']['seed']
guest['disk_oracle']['seed'] = original_seed + 1
guest_file.write_text(json.dumps(guests, indent=2) + '\n')
record = {'scope': 'Synthetic negative fixture; original committed guest records remain unchanged.',
          'source': str(source.resolve()), 'destination': str(destination.resolve()),
          'mutation': {'record_label': 'm5-read-a', 'field': 'disk_oracle.seed',
                       'before': original_seed, 'after': original_seed + 1},
          'original_sha256': hashlib.sha256(before).hexdigest(),
          'modified_sha256': hashlib.sha256(guest_file.read_bytes()).hexdigest(),
          'fixture_builder_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
(root / 'records' / (destination.name + '-mutation.json')).write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps(record, indent=2))
