import json,sys,re,glob,os,collections,statistics as st
root,out1,out2,namesf=sys.argv[1:5]
names=[l.strip() for l in open(namesf) if l.strip()]
def mmm(v):
    return (min(v),st.median(v),max(v)) if v else ('','','')
def f(x,nd=3):
    return '' if x=='' else (('%.*f'%(nd,x)) if isinstance(x,float) else str(x))
rows1=[];rows2=[];noresult=[];bad=0
for n in names:
    fs=sorted(glob.glob(os.path.join(root,n,'**','result.json'),recursive=True))
    if not fs: noresult.append(n); continue
    groups=collections.defaultdict(list)
    for p in fs:
        try: d=json.load(open(p))
        except Exception as e: bad+=1; continue
        base=re.sub(r'-r\d+-of-\d+$','',d.get('label',''))
        groups[base].append(d)
    for base,ds in sorted(groups.items()):
        dur=[d['duration_s'] for d in ds]; to=sorted({d['timeout_s'] for d in ds})
        mx=max(dur); tmax=max(to)
        rows1.append([n,base,len(ds),sum(1 for d in ds if d.get('passed')),sum(1 for d in ds if d.get('timed_out')),
            sum(1 for d in ds if d.get('raw_qemu_exit')!=d.get('expected_exit')),'/'.join(map(str,to)),
            min(dur),st.median(dur),mx,max(d['duration_s']/d['timeout_s'] for d in ds)])
        ratio=[];rb=0;wait=[];defer=[];lim_ms=set();lim_t=set();fb=0;wake=[];wb=0;load=collections.defaultdict(list);lb=0
        oob=0;kw=collections.Counter();kwb=collections.Counter()
        for d in ds:
            lines=re.split(r'[\r\n]+',d.get('stdout') or '')
            r=[int(m.group(1)) for l in lines for m in [re.fullmatch(r'SMP:TIMER:RATIO:(\d+)',l)] if m]
            if r: rb+=1; ratio+=r; oob+=sum(1 for x in r if not 50000<=x<=75000)
            fa=[tuple(map(int,m.groups())) for l in lines for m in [re.fullmatch(r'SMP:FAIR:CPU=(\d+):WAIT_MS=(\d+):DEFERRED=(\d+):LIMIT_MS=(\d+):LIMIT_TICKS=(\d+)',l)] if m]
            if fa:
                fb+=1; wait.append(max(x[1] for x in fa)); defer.append(max(x[2] for x in fa))
                lim_ms|={x[3] for x in fa}; lim_t|={x[4] for x in fa}
            w=[int(m.group(1)) for l in lines for m in [re.fullmatch(r'SMP:IDLE:WAKE_US=(\d+)',l)] if m]
            if w: wb+=1; wake.append(max(w))
            lo=[(int(m.group(1)),int(m.group(2))) for l in lines for m in [re.fullmatch(r'SMP:LOAD:WORKERS=(\d+):ELAPSED_MS=(\d+)',l)] if m]
            if lo:
                lb+=1
                for a,b in lo: load[a].append(b)
            seen=set()
            for l in lines:
                if re.search(r'TIMEOUT|DRIFT|STUCK|FAIL',l):
                    kw[l]+=1; seen.add(l)
            for l in seen: kwb[l]+=1
        allload=[x for v in load.values() for x in v]
        rows2.append([n,base,len(ds),rb,*mmm(ratio),oob,
            fb,*mmm(wait),'/'.join(map(str,sorted(lim_ms))),*mmm(defer),'/'.join(map(str,sorted(lim_t))),
            wb,*mmm(wake),lb,'/'.join(map(str,sorted(load))),*mmm(allload),
            '; '.join('%s x%d (boots=%d)'%(k,v,kwb[k]) for k,v in sorted(kw.items()))])
h1=['artifact','case','boots','passed','timed_out','exit_mismatch','timeout_s','dur_min_s','dur_median_s','dur_max_s','max_dur_over_timeout']
h2=['artifact','case','boots','timer_ratio_boots','timer_ratio_min','timer_ratio_median','timer_ratio_max','timer_ratio_out_of_50000_75000',
 'fair_boots','fair_wait_ms_min','fair_wait_ms_median','fair_wait_ms_max','fair_limit_ms','fair_deferred_min','fair_deferred_median','fair_deferred_max','fair_limit_ticks',
 'idle_wake_boots','idle_wake_us_min','idle_wake_us_median','idle_wake_us_max','load_boots','load_workers','load_elapsed_ms_min','load_elapsed_ms_median','load_elapsed_ms_max','timeout_drift_stuck_fail_lines']
for p,h,rows in ((out1,h1,rows1),(out2,h2,rows2)):
    with open(p,'w') as o:
        o.write('\t'.join(h)+'\n')
        for r in rows: o.write('\t'.join(f(x) for x in r)+'\n')
        if p==out1:
            for n in noresult: o.write('\t'.join([n,'NOT_PROCESSED: no result.json']+['']*9)+'\n')
print('artifacts',len(names),'with results',len(names)-len(noresult),'noresult',noresult,'bad json',bad,'rows',len(rows1),'boots',sum(r[2] for r in rows1))
print('multi-case artifacts:',[n for n,c in collections.Counter(r[0] for r in rows1).items() if c>1])
print('not all passed:',[(r[0],r[1],r[2],r[3],r[4],r[5]) for r in rows1 if r[3]!=r[2]])
print('TOP12');
for r in sorted(rows1,key=lambda r:-r[10])[:14]: print(r[1],r[2],r[3],r[6],'%.2f %.2f %.2f %.3f'%tuple(r[7:11]))
tr=[r for r in rows2 if r[4]!='']
a=min(tr,key=lambda r:r[4]);b=max(tr,key=lambda r:r[6]);print('RATIO min',a[1],a[4],'max',b[1],b[6],'oob total',sum(r[7] for r in rows2))
print('ratio lows:',sorted((r[4],r[1]) for r in tr)[:5]);print('ratio highs:',sorted((r[6],r[1]) for r in tr)[-5:])
fr=[r for r in rows2 if r[11]!='']
a=max(fr,key=lambda r:r[11]);b=max(fr,key=lambda r:r[15]);print('FAIR wait max',a[1],a[11],'limit',a[12],'| deferred max',b[1],b[15],'limit',b[16],'| limits seen',{r[12] for r in fr},{r[16] for r in fr})
wr=[r for r in rows2 if r[20]!=''];a=max(wr,key=lambda r:r[20]);print('WAKE max',a[1],a[20])
lr=[r for r in rows2 if r[25]!=''];a=max(lr,key=lambda r:r[25]);print('LOAD max',a[1],a[25])
for r in rows2:
    if re.search(r'FAIL|DRIFT|SYNC',r[26]): print('FAILLINES',r[1],r[26])
