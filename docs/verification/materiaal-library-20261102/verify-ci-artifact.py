from pathlib import Path
import argparse, hashlib, io, json, struct, sys, tempfile, zipfile
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'tools'))
import run_window_session as window
import window_library as library
from build_users import inspect_user
from run_materiaal_probe import disk_oracle

ARTIFACT_ID = 13183010575
ARTIFACT_SHA256 = '3c7ca75a8304efb0922b6d6fc3a3beda750a71d294988a663a0711df9af7aede'
parser = argparse.ArgumentParser(description='Verify the original CI artifact ZIP and its contents')
parser.add_argument('download', type=Path, help='original downloaded ZIP for artifact 13183010575')
args = parser.parse_args()
EXPECTED = '069666e5406a893f64aa5d8e00298863f33f8be9cef57b6e0522b8bdf9648670'
def digest(data): return hashlib.sha256(data).hexdigest()
# Verify the exact bytes that will be unpacked, before reading any ZIP entries.
download = args.download.read_bytes()
artifact_sha256 = digest(download)
if artifact_sha256 != ARTIFACT_SHA256:
    raise ValueError('CI artifact SHA-256 mismatch')
artifact_data = tempfile.TemporaryDirectory(prefix='nieuw-os-ci-readback-')
DATA = Path(artifact_data.name)
with zipfile.ZipFile(io.BytesIO(download)) as artifact:
    artifact.extractall(DATA)
source = json.loads((DATA/'evidence/source.json').read_text())
assert source['source_sha256'] == EXPECTED and source['source_fingerprint_version'] == 3
archive = DATA/'evidence/NieuwOS-Venstersessie-Windows-QEMU.zip'
with zipfile.ZipFile(archive) as package:
    payloads = {name: package.read(name) for name in package.namelist()}
    pm = json.loads(payloads['manifest.json'])
    assert pm['source_sha256'] == EXPECTED and pm['kernel_feature'] == 'native-window-session'
    assert set(payloads) == set(pm['files']) | {'manifest.json'}
    for name, entry in pm['files'].items():
        assert entry['sha256'] == digest(payloads[name]) and entry['size_bytes'] == len(payloads[name])

kernel = (DATA/'probe/esp/kernel.elf').read_bytes()
manifest = json.loads((DATA/'probe/manifest.json').read_text())
assert manifest['source_sha256'] == EXPECTED
assert digest(kernel) == manifest['kernel']['sha256']
assert digest((DATA/'probe/esp/EFI/BOOT/BOOTX64.EFI').read_bytes()) == manifest['loader']['sha256']
# Extract exact embedded ELF files, including their section-header tables.
by_hash = {a['sha256']: a for a in manifest['materiaal']['apps']}
extracted, offset = {}, 0
directory = ROOT/'build/library-ci-640/extracted-users'
directory.mkdir(parents=True, exist_ok=True)
while True:
    offset = kernel.find(b'\x7fELF\x02\x01\x01', offset + 1)
    if offset < 0: break
    if len(kernel) - offset < 64: continue
    section = struct.unpack_from('<Q', kernel, offset + 40)[0]
    entry, count = struct.unpack_from('<HH', kernel, offset + 58)
    length = section + entry * count
    if not section or not count or not 64 <= length <= len(kernel) - offset: continue
    data = kernel[offset:offset + length]
    app = by_hash.get(digest(data))
    if app:
        path = directory/(app['name']+'.elf')
        path.write_bytes(data)
        assert inspect_user(path) == app['elf']
        extracted[app['name']] = digest(data)
assert len(extracted) == 8

fixtures = []
for path in (DATA/'probe').glob('window-fixture-*/fixture.json'):
    metadata = json.loads(path.read_text())
    assert metadata['source_sha256'] == EXPECTED and metadata['generator_exit'] == 0
    for name, expected in metadata['files_sha256'].items():
        assert digest((path.parent/name).read_bytes()) == expected
    fixtures.append((path.parent, metadata))

records = {}
for path in (DATA/'probe/run').glob('*/result.json'):
    report = json.loads(path.read_text())
    assert report['passed'] is True and report['raw_qemu_exit'] == 33
    assert not report.get('error') and not report.get('harness_terminated_process')
    assert report['source_sha256'] == EXPECTED and report['source_commit'] == source['source_commit']
    assert report['manifest'] == manifest
    raw = (path.parent/'serial.txt').read_bytes()
    assert digest(raw) == report['serial_sha256'] and b'WINDOW:FAIL:' not in raw and b'PANIC' not in raw
    assert raw.endswith(b'WINDOW:CLEANUP:PASS\nWINDOW:CASE:PASS\nM1:PASS\n')
    snapshots = {role: window.snapshots(raw, role) for role in ('MATERIAL','CALCULATOR')}
    fixture = next(folder for folder, metadata in fixtures if metadata == report['oracle'])
    exact = 0
    for name, phase in report['phases'].items():
        image = (path.parent/(name+'.ppm')).read_bytes()
        assert digest(image) == phase['frame_sha256'] and window.ppm(image)[:2] == (1280,720)
        for role, state in phase['state'].items(): assert state in snapshots[role]
        oracle = phase.get('oracle')
        if oracle:
            prefix = 'calculator-320x484' if oracle in window.CALCULATOR_CLIPBOARD_PHASES else '880x510'
            expected = (fixture/(prefix+'-'+oracle+'.ppm')).read_bytes()
            assert window.oracle_matches(image, expected, oracle)
            exact += 1
    for name, expected in report['immutable'].items():
        if name.startswith('esp/'):
            actual = (DATA/'probe'/name).read_bytes()
        elif name in payloads:
            actual = payloads[name]
        elif name == 'volume.json':
            continue  # Non-guest metadata not uploaded; actual image decoded below.
        else:
            raise AssertionError(('missing immutable', name))
        assert digest(actual) == expected
    image = (path.parent/'volume.img').read_bytes()
    if report['library_requested']:
        assert library.disk_oracle(image) == report['disk_oracle']
        assert digest(image) == report['volume_sha256']
        kind = 'cold' if report['library_cold'] else 'write'
        assert list(report['phases']) == list(library.PHASES[10:] if kind == 'cold' else library.PHASES[:10])
        assert report['library_gate'] and not report['native_gate']
    else:
        kind = 'regression'
        assert len(report['phases']) == 22 and report['native_gate'] and report['clipboard_gate']
        assert disk_oracle(image) == report['disk_oracle']
        assert window.numbers(raw,'WINDOW:FAULT_SLOT') == [4]
        assert window.numbers(raw,'WINDOW:FAULT_VECTOR') == [6]
        assert window.numbers(raw,'WINDOW:RESTARTS') == [1]
        def frame(name): return (path.parent/(name+'.ppm')).read_bytes()
        assert window.crop(frame('calculated'), window.CALCULATOR_BODY) == window.crop(frame('moved'), (772,122,320,484))
        assert window.crop(frame('calculated'), window.CALCULATOR_BODY) == window.crop(frame('calculator-restored'), window.CALCULATOR_BODY)
        for name in ('editor','dirty'): assert not window.changed_outside(frame('initial'),frame(name),window.BODY)
        initial, recovered = report['phases']['initial']['state'], report['phases']['recovered']['state']
        for role in initial:
            assert initial[role]['PID'] == recovered[role]['PID']
            assert initial[role]['PROBES'] == 3 and recovered[role]['PROBES'] == 10
            assert recovered[role]['SURFACE_EPOCH'] > initial[role]['SURFACE_EPOCH']
            assert recovered[role]['INPUT_EPOCH'] > initial[role]['INPUT_EPOCH']
    records[kind] = {'report': report, 'directory': path.parent, 'image': image,
                     'summary': {'passed': True, 'phases': len(report['phases']), 'exact_app_images': exact,
                     'seconds': report['duration_s'], 'report_sha256': digest(path.read_bytes()),
                     'serial_sha256': digest(raw), 'volume_sha256': digest(image)}}
assert set(records) == {'write','cold','regression'}
write, cold = records['write'], records['cold']
assert write['image'] == cold['image'] and write['directory'] != cold['directory']
parent = cold['report']['library_parent']
assert parent['report_sha256'] == write['summary']['report_sha256']
assert parent['volume_sha256'] == write['summary']['volume_sha256']
assert cold['report']['command'] == write['report']['command']
assert cold['report']['qemu_sha256'] == write['report']['qemu_sha256']
assert cold['report']['qemu_implementation']['sha256'] == write['report']['qemu_implementation']['sha256']
result = {'passed': True, 'github_head': '64044318c622eaacab25272611acf856a5fca5d5',
          'ci_merge_source': source, 'artifact_id': ARTIFACT_ID,
          'artifact_sha256': artifact_sha256,
          'runs': {kind: record['summary'] for kind, record in records.items()},
          'embedded_users': extracted, 'normal_package_payloads': len(pm['files']),
          'normal_package_sha256': digest(archive.read_bytes()),
          'limits': ['CI is Linux QEMU/TCG; no Windows/VMware/hardware runtime claim',
                     'volume.json metadata was not archived; actual volume bytes, all images and available immutable firmware/ESP inputs were verified']}
(ROOT/'docs/verification/materiaal-library-20261102/ci-readback.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
