"""Development VM check with explicit temporary integration hooks; not final root proof."""
import importlib.util
import json
import os
import subprocess
import sys
sys.path.insert(0, 'tools')
from common import ROOT, BUILD, run_logged, save_json, sha256
from build import build
selected = sys.argv[1]
assert selected in ('normal', 'nx', 'guard', 'peer', 'oom')
p = ROOT / 'kernel/src/m2/m3.rs'
cargo = ROOT / 'kernel/Cargo.toml'
saved, csaved = p.read_bytes(), cargo.read_bytes()
try:
    subprocess.run(['git','apply','build/memory-fixture/m3-hooks.patch'],check=True)
    features = 'process-memory = ["monotonic-clock"]\nnative-memory = ["process-memory", "fpu-context"]\n' + ''.join(f'native-memory-{case} = ["native-memory"]\n' for case in ['nx','guard','peer','oom'])
    cargo.write_bytes(csaved.replace(b'[features]\n',b'[features]\n'+features.encode()))
    run_logged([sys.executable,'-B',str(ROOT/'sdk/examples/memory/build.py')],'build-memory-app',extra_env={'CARGO_TARGET_DIR':str(ROOT/'target/memory-app')})
    app = ROOT / 'target/memory-app/x86_64-unknown-none/release/memory-probe'
    os.environ['NOS_MEMORY_ELF'] = str(app)
    feature = 'native-memory' + ('' if selected == 'normal' else '-' + selected)
    build(kernel_feature=feature,label=feature)
    manifest = json.loads((BUILD/'manifest.json').read_text())
    spec = importlib.util.spec_from_file_location('memory_local_runner',ROOT/'build/memory-fixture/run_root_helper.py')
    run = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(run)
    markers = ['BOOT:EBS_OK','KERNEL:ENTRY','BOOTINFO:OK','M2:PASS','MEMORY:MANIFEST:PASS','MEMORY:GROWTH:PASS','MEMORY:ISOLATION:PASS']
    if selected == 'oom': markers.append('MEMORY:OOM:PASS')
    markers += ['MEMORY:CLEANUP:PASS','MEMORY:CASE:'+selected.upper()+':PASS','M1:PASS']
    result = run.run_guest(feature,required_markers=markers,forbidden_markers=['M1:FAIL','M2:FAIL','M3:FAIL','MEMORY:FAIL'],expected_kernel_sha256=manifest['kernel']['sha256'],expected_loader_sha256=manifest['loader']['sha256'],memory_mib=128 if selected=='oom' else 256)
    result['memory_app_sha256'] = sha256(app)
    result['scope'] = 'Development fixture gate using recorded temporary m3/Cargo hooks and copied root memory_mib helper. Canonical root integrated gate remains separate.'
    result['temporary_m3_hooks_sha256'] = sha256(ROOT/'build/memory-fixture/m3-hooks.patch')
    result['runner_helper_sha256'] = sha256(ROOT/'build/memory-fixture/run_root_helper.py')
    save_json(BUILD/'development-report.json',result)
    if not result['passed']: raise SystemExit(1)
finally:
    p.write_bytes(saved)
    cargo.write_bytes(csaved)
