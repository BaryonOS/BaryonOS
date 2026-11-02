"""Sequential, frozen-source Windows/WHPX A/B; full native gate on every arm."""
from pathlib import Path
import datetime,hashlib,json,statistics,subprocess,sys,time
ROOT=Path(__file__).resolve().parent
SOURCE=ROOT/'source'
ARMS={
 'A':('337f2f29f1833ef4a8d2cd640b09419f85dbb83b','windows-probe-337f2f2','3919403542c923bc86467db3184345159a5ce6881567a0e11baa85f2d39f1995'),
 'B':('1f2ac0ae7cf8944d77ca0656affb01fe9b086f1b','windows-idle-1f2ac0a','8faf3f7f47e3f26409e446b2311673416ddd70d5735ef106d94c9f30d2c6847a')}
REFERENCE='6ed1645b503ba3957005cfd984c2bb5031b2f6788e9ee0b9040ae21ecef07fd7'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def git(*args):return subprocess.check_output(['git','-C',str(SOURCE),*args],text=True,encoding='utf-8',timeout=15).strip()
def main():
 out=ROOT/'measurements'/datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ-formal-ab')
 out.mkdir(parents=True)
 fixed={name:sha(ROOT/name) for name in ['run_windows_window.py','windows_qmp.py','windows_inputs.py','windows_idle_measure.py']}
 report={'passed':False,'kind':'windows-whpx-frozen-full-native-idle-ab','sequence':['A','B','B','A','A','B'],
         'reference_frame_sha256':REFERENCE,'adapter_files_sha256':fixed,'runs':[],
         'scope':'Three 15s intervals per arm on one vCPU; headless WHPX only. Same full endpoint frame; no input or pause during interval. Every arm continues through actual full native recovery/save/oracle/cleanup. No proof of no intermediate redraw, energy consumption or physical-hardware acceptance.'}
 try:
  for index,arm in enumerate(report['sequence'],1):
   if git('status','--porcelain','--untracked-files=no'):raise ValueError('Frozen checkout has tracked changes')
   commit,build_name,source_hash=ARMS[arm]
   git('switch','--detach',commit)
   if git('rev-parse','HEAD')!=commit:raise ValueError('Wrong frozen HEAD')
   build=SOURCE/'build'/build_name
   fixture=list(build.glob('window-fixture-*'))
   if len(fixture)!=1:raise ValueError('Expected exactly one fixture')
   argv=[sys.executable,'-X','utf8',str(ROOT/'run_windows_window.py'),'--source',str(SOURCE),
         '--expected-commit',commit,'--build-dir',str(build),'--fixture-dir',str(fixture[0]),
         '--ovmf-code',str(ROOT/'firmware/OVMF_CODE.fd'),'--ovmf-vars',str(ROOT/'firmware/OVMF_VARS.template.fd'),
         '--bios-rom',str(ROOT/'firmware/bios-256k.bin'),'--vga-rom',str(ROOT/'firmware/VGA.rom'),
         '--acceleration','whpx','--timeout','180','--measure-idle','--reference-frame-sha256',REFERENCE]
   print(f'FORMAL {index}/6 arm {arm} start',flush=True)
   log=out/f'{index}-{arm}.txt'
   with log.open('xb') as stream:r=subprocess.run(argv,stdout=stream,stderr=subprocess.STDOUT,timeout=210)
   result=build/'window-session-report.json';data=json.loads(result.read_text(encoding='utf-8'))
   native=Path(data['run_directory'])/'result.json'
   measurement=data.get('windows_idle_measurement',{})
   if (r.returncode!=0 or not data.get('passed') or not data.get('native_gate') or data.get('raw_qemu_exit')!=33
       or not measurement.get('measurement_passed') or data['windows_adapter']['source_sha256']!=source_hash
       or measurement.get('source_initial_sha256')!=REFERENCE):raise ValueError(f'Failed formal native arm {index}/{arm}: {data.get("error")}')
   if any(sha(ROOT/name)!=expected for name,expected in fixed.items()):raise ValueError('Adapter changed during A/B')
   record={'index':index,'arm':arm,'source_commit':commit,'source_sha256':source_hash,'native_report':str(native),
           'native_report_sha256':sha(native),'log':str(log),'log_sha256':sha(log),
           'native_gate':True,'raw_qemu_exit':33,'duration_s':data['duration_s'],
           'process_percent_one_logical_cpu':measurement['cpu']['process']['percent_one_logical_cpu'],
           'vcpu_percent_one_logical_cpu':measurement['cpu']['vcpu']['percent_one_logical_cpu'],
           'process_wall_s':measurement['cpu']['process']['wall_seconds_midpoint'],
           'initial_s':data['phases']['initial']['seconds'],'measurement':measurement}
   report['runs'].append(record)
   (out/'result.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
   print(f'FORMAL {index}/6 {arm} native PASS; CPU {record["process_percent_one_logical_cpu"]:.3f}%; vCPU {record["vcpu_percent_one_logical_cpu"]:.3f}%',flush=True)
  report['statistics']={}
  for metric in ['process_percent_one_logical_cpu','vcpu_percent_one_logical_cpu']:
   stats={}
   for arm in ARMS:
    values=[x[metric] for x in report['runs'] if x['arm']==arm]
    stats[arm]={'values':values,'median':statistics.median(values),'min':min(values),'max':max(values)}
   stats['median_reduction_percent']=100*(1-stats['B']['median']/stats['A']['median'])
   report['statistics'][metric]=stats
  report['passed']=True
 except Exception as error:
  report['error']=type(error).__name__+': '+str(error)
 finally:
  (out/'result.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
  print(json.dumps({'passed':report['passed'],'report':str(out/'result.json'),'statistics':report.get('statistics'),'error':report.get('error')}),flush=True)
 return 0 if report['passed'] else 1
if __name__=='__main__':raise SystemExit(main())
