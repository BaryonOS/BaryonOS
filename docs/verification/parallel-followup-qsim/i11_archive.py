"""Archive I11's existing single VM without modifying the build artifacts."""
from __future__ import annotations

import datetime
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path('/workspace/scratch/debaa3b6a81c/nieuw-os')
BUILD = ROOT / 'build/parallel-followup-qsim'
PROOF = ROOT / 'docs/verification/parallel-followup-qsim'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, data):
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n')


def main():
    if PROOF.exists():
        raise RuntimeError('Refusing to overwrite an existing I11 proof bundle')
    capture = json.loads((BUILD / 'i11-run-checks.json').read_text())
    acceptance = json.loads((BUILD / 'native-qsim-result.json').read_text())
    if capture.get('passed') is not True or acceptance.get('passed') is not True:
        raise RuntimeError('No successful I11 run to archive')
    sys.path.insert(0, str(ROOT / 'tools'))
    from common import source_hash
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    if head != capture['expected_head'] or source_hash() != capture['expected_source_sha256']:
        raise RuntimeError('Source or HEAD changed before archiving I11')
    guest = acceptance['guest']
    run = Path(guest['run_directory']).resolve(strict=True)
    if not run.is_relative_to(BUILD / 'run'):
        raise RuntimeError('VM directory lies outside the exclusive I11 build root')
    if (run / 'serial.txt').read_bytes().decode('utf-8', 'replace') != guest['stdout']:
        raise RuntimeError('Preserved raw serial differs from the accepted guest output')
    if (run / 'stderr.txt').read_bytes().decode('utf-8', 'replace') != guest['stderr']:
        raise RuntimeError('Preserved raw stderr differs from the accepted guest output')
    for name, expected in guest['artifact_sha256'].items():
        if digest(run / name) != expected:
            raise RuntimeError('Guest artifact changed before archiving: ' + name)
    for name, expected in capture['current_artifacts'].items():
        if digest(BUILD / name) != expected:
            raise RuntimeError('Build artifact changed before archiving: ' + name)
    PROOF.mkdir(parents=True)
    copied = {
        'execution.json': BUILD / 'i11-run-checks.json',
        'acceptance.json': BUILD / 'native-qsim-result.json',
        'result.json': run / 'result.json',
        'build-manifest.json': BUILD / 'manifest.json',
        'quantum-build-report.json': BUILD / 'users/qsim/build-report.json',
        'loader-comparison.json': BUILD / 'loader-comparison.json',
        'serial.txt': run / 'serial.txt',
        'stderr.txt': run / 'stderr.txt',
        'rust-tools.json': Path('/tmp/nieuw-os-par-tools/rust-tools.json'),
        'qemu-tools.json': Path('/tmp/nieuw-os-par-tools/qemu-tools.json'),
    }
    for name, original in copied.items():
        shutil.copyfile(original, PROOF / name)
    # Keep exact UTF-8 log streams, including terminal blank lines, as JSON data.
    # No binary artifact is copied into the versioned proof bundle.
    logs = {}
    paths = list((BUILD / 'logs').glob('*'))
    paths.extend(path for path in (BUILD / 'users/qsim').iterdir() if path.suffix in ('.txt', '.json'))
    paths.extend(BUILD / (label + '.' + stream + '.txt') for label in ('build', 'accept') for stream in ('stdout', 'stderr'))
    for path in sorted(set(paths)):
        raw = path.read_bytes()
        logs[path.relative_to(BUILD).as_posix()] = {
            'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw),
            'utf8': raw.decode('utf-8'),
        }
    save(PROOF / 'raw-streams.json', logs)
    for name in ('i11_capture.py', 'i11_archive.py', 'i11_compare_loader.py'):
        shutil.copyfile(Path(__file__).parent / name, PROOF / name)
    hashes_equal = all(item['identical_bytes'] for item in capture['artifact_comparison'].values())
    loader_comparison = json.loads((BUILD / 'loader-comparison.json').read_text())
    explained_loader = (loader_comparison.get('changes_only_coff_debug_timestamps_and_codeview_guid') is True
                        and loader_comparison.get('all_other_file_bytes_identical') is True
                        and loader_comparison.get('artifacts_unchanged_after_inspection') is True)
    comparison = ('De herbouwde kernel, qsim-user-ELF en UEFI-loader zijn byte voor byte gelijk aan '
                  'de artifacts uit de eerdere native acceptatie op 3e41bbf. De bredere bronvingerafdruk '
                  'is veranderd door andere geïntegreerde bron- en harnesswijzigingen; de 18 qsim-inputs '
                  'zijn ongewijzigd. Er is geen artifacthashverschil dat verklaard moet worden.'
                  if hashes_equal else 'De kernel en qsim-user-ELF zijn byte voor byte gelijk aan de artifacts '
                  'uit de eerdere native acceptatie op 3e41bbf. Alleen de loaderhash verschilt. Directe '
                  'vergelijking vindt precies 12 afwijkende bytes, uitsluitend in de COFF- en debugtimestamps '
                  'en de CodeView RSDS-buildidentifier. De timestamps zijn 2026-10-28 15:36:50 UTC en '
                  '2026-10-28 16:21:30 UTC, passend bij beide builds. De volledige uitvoerbare `.text`-sectie '
                  'en alle overige bytes buiten die benoemde metadatavelden zijn identiek. `loader-comparison.json` '
                  'bewaart de offsets, sectiehashes, volledige onafhankelijke `objdump -x`-uitvoer en de controle '
                  'dat beide echte bestanden ongemoeid bleven. Alleen kopieën in het geheugen zijn voor deze '
                  'vergelijking van metadata ontdaan; geen gehasht artifact is gewijzigd. De nieuwe loaderhash '
                  'is rechtstreeks gebonden aan de actuele VM. De bredere bronvingerafdruk is veranderd door '
                  'andere geïntegreerde bron- en harnesswijzigingen; de 18 qsim-inputs zijn ongewijzigd.')
    if not hashes_equal and not explained_loader:
        raise RuntimeError('Loader hash difference has not been fully explained')
    pids = acceptance['observed']['processes']
    observed = acceptance['observed']
    readme = f'''# I11 — actuele native qsim-smoke

## Uitkomst en bronbinding

Eén echte QEMU-VM slaagt met ruwe exitcode `{guest['raw_qemu_exit']}` en de volledige strikte reeks van 28 regels vanaf `BOOT:EBS_OK`. De runner accepteert beide gescheiden ring-3-processen, Bell- en zesqubittest, de bestaande annulering- en deadlinezelftests, resourcegrenzen, procesisolatie en werkelijke framevrijgave.

- Commit: `{capture['expected_head']}`.
- Tree: `{capture['before']['tree']}`.
- Source fingerprint v3: `{capture['expected_source_sha256']}`.
- HEAD, tree en source fingerprint zijn vóór build en na VM gelijk; iedere fase controleert opnieuw de bron.
- Werkelijke native VMs in deze bundel: **1**. Er is geen volledige M3-suite herhaald.
- Run: `{run.relative_to(ROOT).as_posix()}`.
- Profiel: `{guest['profile']}`.
- PID 1: {pids[0]['checkpoints']} checkpoints, {pids[0]['useful_arena']} nuttige arenabytes, maximale werkkloof {pids[0]['max_work_gap']}.
- PID 2: {pids[1]['checkpoints']} checkpoints, {pids[1]['useful_arena']} nuttige arenabytes, maximale werkkloof {pids[1]['max_work_gap']}.
- {observed['timer_irqs']} echte timerinterrupts, {observed['syscall_calls']} syscalls en {observed['cleanup_frames']} vrijgegeven frames.

Alle zeven strict-runnercontroles slagen. De controle bindt de werkelijk gebruikte kernel, loader, OVMF CODE/VARS-template, qsim-user-ELF, scalarbuildrapport en disassembly aan bron en hashes; het exacte user-ELF is in de kernel aanwezig. De privésnapshot behoudt de onveranderde inputbestanden. `serial.txt` en `stderr.txt` zijn ruwe kopieën van de VM-uitvoer en zijn bij archivering opnieuw met het geaccepteerde rapport vergeleken.

## Vergelijking met de eerdere acceptatie

{comparison}

| Artifact | SHA-256 huidige run |
| --- | --- |
| Kernel | `{capture['current_artifacts']['esp/kernel.elf']}` |
| Qsim-user-ELF | `{capture['current_artifacts']['users/qsim/qsim-native-probe.elf']}` |
| UEFI-loader | `{capture['current_artifacts']['esp/EFI/BOOT/BOOTX64.EFI']}` |

`execution.json` bevat zowel de oude als nieuwe hashes en de 18 afzonderlijke qsim-inputvergelijkingen. De eerdere artifacts zijn uitsluitend gelezen en zijn na deze run opnieuw onveranderd bevonden.

## Reproduceren

Gebruik Rust 1.90.0 met de targets `x86_64-unknown-none` en `x86_64-unknown-uefi`, de gepinde selectorcompatibele launchers en de vastgelegde QEMU/OVMF-installatie. De launchercompatibiliteit is nodig omdat de bestaande buildketen `+1.90.0` gebruikt.

```bash
source /tmp/nieuw-os-par-tools/env.sh
export CARGO_HOME=/tmp/nieuw-os-par-tools/cargo-home
export CARGO_BUILD_JOBS=2
export NOS_BUILD_DIR={BUILD}
export NOS_TARGET_DIR={TARGET}
export CARGO_TARGET_DIR={TARGET}
export NOS_QEMU_MACHINE=pc-q35-7.2
export NOS_SDK_RUNTIME=0
python3 -B tools/build.py --kernel-feature native-qsim
python3 -B tools/run_qsim.py
```

Voer dit uit vanuit de repository op de genoemde commit. Gebruik voor een nieuwe onafhankelijke bewijscapture een eigen leeg build- en targetpad. De oorspronkelijke I11-capture weigert bestaand bewijs te overschrijven; de opgeslagen scripts documenteren die uitvoering en de daaropvolgende archivering.

## Bewijs en grenzen

`acceptance.json` bevat de stricte checks en waarnemingen; `result.json` bevat de echte QEMU-commandoregel, raw exit, timing en artifactbinding. `execution.json` legt de bovenliggende commands, toolwrapper- en firmwarehashes, versie-uitvoer, broncontrole en artifactvergelijking vast. `raw-streams.json` bewaart de exacte UTF-8-bytes van buildlogs, disassembly en symbooluitvoer inclusief oorspronkelijke eindregels, met lengte en SHA-256. De native binaries blijven buiten deze Git-bundel.

Dit bewijst de bestaande begrensde qsim-referentietaken in twee native gebruikersprocessen. Het is geen algemene quantum-jobdienst, geen hardware-QPU, geen GUI-flow en geen fysieke hardwaretest. Het nieuwe job-requestcodec wordt hierdoor niet als actieve dienst geaccepteerd. De disassemblycontrole is een gerichte scalarinstructiegate, geen formeel bewijs. Er wordt geen roadmapcriterium of scorewijziging uitsluitend op deze smoke geclaimd.
'''
    (PROOF / 'README.md').write_text(readme)
    index = {
        'task': 'I11', 'captured_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'source_head': head, 'source_sha256': capture['expected_source_sha256'],
        'files': {p.name: {'sha256': digest(p), 'bytes': p.stat().st_size}
                  for p in sorted(PROOF.iterdir()) if p.is_file()},
        'archive_copies_match_original': {name: digest(PROOF / name) == digest(original) for name, original in copied.items()},
    }
    if not all(index['archive_copies_match_original'].values()):
        raise RuntimeError('Archive copy differs from source bytes')
    save(PROOF / 'artifact-index.json', index)
    print(json.dumps({'proof': str(PROOF), 'files': len(index['files']) + 1,
                      'copied_bytes_exact': True, 'all_artifacts_identical_to_previous': hashes_equal}, indent=2))


TARGET = Path('/tmp/nieuw-os-followup-qsim-target')

if __name__ == '__main__':
    main()
