#!/usr/bin/env python3
"""Build a repository-local OSDev/Little OS Book corpus from the official dump or MediaWiki API."""
from __future__ import annotations
import argparse,gzip,hashlib,json,re,shutil,subprocess,tempfile,urllib.error,urllib.parse,urllib.request,xml.etree.ElementTree as ET
from pathlib import Path

WIKI_DUMP='https://files.osdev.org/osdev-wiki.xml.gz'
WIKI_API='https://wiki.osdev.org/api.php'
BOOK='https://littleosbook.github.io/book.pdf'
DATE='2015-01-19'
UA='osdev-mirror'
BOUNDS=[1520,1736,1947,2245,2570,2872,3150,3367,3532,3773,4044,4349,4613,5077,10**9]

def ln(t): return t.rsplit('}',1)[-1]
def kids(e,n): return [x for x in e if ln(x.tag)==n]
def one(e,n): return next((x for x in e if ln(x.tag)==n),None)
def txt(e,n,d=''):
    x=one(e,n); return (x.text or d) if x is not None else d

def revtext(r):
    x=one(r,'text')
    if x is not None: return x.text or ''
    slots=one(r,'slots')
    if slots:
        for slot in kids(slots,'slot'):
            x=one(slot,'text')
            if x is not None and slot.attrib.get('role','main')=='main': return x.text or ''
    return ''

def digest(b): return hashlib.sha256(b).hexdigest()
def headers(): return {'User-Agent':UA,'Accept':'*/*','Referer':'https://wiki.osdev.org/'}

def download(url,path):
    req=urllib.request.Request(url,headers=headers())
    with urllib.request.urlopen(req,timeout=180) as response,path.open('wb') as f:
        shutil.copyfileobj(response,f)

def api(params):
    params=dict(params); params.update({'format':'json','formatversion':'2'})
    url=WIKI_API+'?'+urllib.parse.urlencode(params)
    req=urllib.request.Request(url,headers=headers())
    with urllib.request.urlopen(req,timeout=180) as response:
        return json.load(response)

def nsmap_xml(xml):
    out={0:'Main'}
    with gzip.open(xml,'rb') as f:
        for _,e in ET.iterparse(f,events=('end',)):
            if ln(e.tag)=='namespaces':
                for x in e: out[int(x.attrib.get('key','0'))]=x.text or 'Main'
                return out
    return out

def iter_dump(xml):
    namespaces=nsmap_xml(xml)
    with gzip.open(xml,'rb') as f:
        for _,page in ET.iterparse(f,events=('end',)):
            if ln(page.tag)!='page': continue
            revisions=kids(page,'revision')
            if not revisions:
                page.clear(); continue
            revision=max(revisions,key=lambda x:(txt(x,'timestamp'),int(txt(x,'id','0'))))
            redirect=one(page,'redirect')
            yield {
                'title':txt(page,'title'),'page_id':int(txt(page,'id','0')),'namespace':int(txt(page,'ns','0')),
                'namespace_name':namespaces.get(int(txt(page,'ns','0')),str(int(txt(page,'ns','0')))),
                'revision_id':int(txt(revision,'id','0')),'timestamp':txt(revision,'timestamp'),
                'body':revtext(revision),'redirect':redirect.attrib.get('title') if redirect is not None else None
            }
            page.clear()

def api_siteinfo():
    data=api({'action':'query','meta':'siteinfo','siprop':'namespaces|statistics'})['query']
    namespaces={int(k):(v.get('name') or v.get('canonical') or 'Main') for k,v in data['namespaces'].items()}
    return namespaces,int(data.get('statistics',{}).get('pages',0) or 0)

def iter_api():
    namespaces,expected=api_siteinfo()
    count=0
    for ns in sorted(namespaces):
        cont={}
        while True:
            params={'action':'query','generator':'allpages','gapnamespace':ns,'gaplimit':'max',
                    'prop':'revisions','rvprop':'ids|timestamp|content','rvslots':'main'}
            params.update(cont)
            data=api(params)
            for page in data.get('query',{}).get('pages',[]):
                revisions=page.get('revisions') or []
                if not revisions: continue
                rev=revisions[0]; slot=(rev.get('slots') or {}).get('main') or {}
                body=slot.get('content','')
                m=re.match(r'(?is)^\s*#redirect\s*\[\[([^\]]+)\]\]',body)
                yield {'title':page['title'],'page_id':int(page['pageid']),'namespace':int(page['ns']),
                       'namespace_name':namespaces.get(int(page['ns']),str(page['ns'])),
                       'revision_id':int(rev['revid']),'timestamp':rev['timestamp'],'body':body,
                       'redirect':m.group(1) if m else None}
                count+=1
            if 'continue' not in data: break
            cont=data['continue']
    if expected and count != expected:
        raise RuntimeError(f'MediaWiki API enumerated {count} pages but siteinfo reports {expected}')

def volume(pid):
    for i,b in enumerate(BOUNDS,1):
        if pid<=b: return i

def write_block(path,header,body,anchor):
    path.parent.mkdir(parents=True,exist_ok=True)
    if not path.exists(): path.write_text('# OS Development knowledge corpus\n\n',encoding='utf-8')
    pre=f'\n---\n\n<a id="{anchor}"></a>\n\n{header}\n\n'.encode(); data=body.encode()
    with path.open('ab') as f:
        f.write(pre); start=f.tell(); f.write(data); f.write(b'\n')
    raw=path.read_bytes(); start_line=raw[:start].count(b'\n')+1
    return start,len(data),start_line,start_line+max(1,len(data.splitlines()))-1,digest(data)

def build(root,wiki_iter,book,source_mode,input_sha=None):
    refs=root/'references'; corpus=refs/'corpus'; assets=root/'assets'
    corpus.mkdir(parents=True,exist_ok=True); assets.mkdir(parents=True,exist_ok=True)
    for p in corpus.glob('*.md'): p.unlink()
    rows=[]; newest=''; count=0
    for page in sorted(wiki_iter,key=lambda x:x['page_id']):
        title=page['title']; pid=page['page_id']; sid=f'W{pid:06d}'; anchor=sid.lower()
        rel=f'references/corpus/wiki-{volume(pid):02d}.md'
        start,length,sl,el,h=write_block(root/rel,f'## {sid} — {title}',page['body'],anchor)
        quoted=urllib.parse.quote(title.replace(' ','_'),safe='')
        rows.append({'source_id':sid,'kind':'wiki','title':title,'page_id':pid,'namespace':page['namespace'],
          'namespace_name':page['namespace_name'],'revision_id':page['revision_id'],'timestamp':page['timestamp'],
          'redirect':page['redirect'],'source_url':f'https://wiki.osdev.org/{quoted}',
          'revision_url':f'https://wiki.osdev.org/index.php?title={quoted}&oldid={page["revision_id"]}',
          'local_path':rel,'anchor':anchor,'start_byte':start,'length_bytes':length,'text_sha256':h,
          'start_line':sl,'end_line':el})
        count+=1; newest=max(newest,page['timestamp'])

    shutil.copy2(book,assets/'book.pdf')
    info=subprocess.check_output(['pdfinfo',str(book)],text=True,errors='replace')
    pages=int(re.search(r'^Pages:\s+(\d+)',info,re.M).group(1))
    for pg in range(1,pages+1):
        with tempfile.NamedTemporaryFile(suffix='.txt',delete=False) as tf: temp=Path(tf.name)
        try:
            subprocess.run(['pdftotext','-f',str(pg),'-l',str(pg),'-layout',str(book),str(temp)],check=True)
            body=temp.read_text(errors='replace').replace('\x0c','').rstrip('\n')
        finally: temp.unlink(missing_ok=True)
        sid=f'B{pg:03d}'; anchor=sid.lower(); rel='references/corpus/boek.md'
        start,length,sl,el,h=write_block(root/rel,f'## {sid} — The little book about OS development — page {pg}',body,anchor)
        rows.append({'source_id':sid,'kind':'pdf','title':f'The little book about OS development — page {pg}',
          'page':pg,'timestamp':DATE,'source_url':BOOK,'local_path':rel,'anchor':anchor,
          'start_byte':start,'length_bytes':length,'text_sha256':h,'start_line':sl,'end_line':el})

    (refs/'source-index.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    manifest={'wiki_pages':count,'pdf_pages':pages,'source_records':len(rows),'newest_wiki_revision':newest,
      'source_mode':source_mode,'inputs':{'osdev_wiki_dump':WIKI_DUMP,'osdev_wiki_api':WIKI_API,'book_pdf':BOOK},
      'sha256':{'wiki_dump':input_sha,'book':digest(book.read_bytes())}}
    (assets/'source-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    if count < 3000 or pages != 78 or len(rows) != count + 78:
        raise SystemExit(f'unexpected counts: {count}/{pages}/{len(rows)}')
    print(json.dumps(manifest,indent=2))

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--root',type=Path,required=True)
    ap.add_argument('--xml',type=Path)
    ap.add_argument('--book',type=Path)
    ap.add_argument('--download',action='store_true')
    a=ap.parse_args(); root=a.root.resolve(); cache=root/'.source-cache'; cache.mkdir(parents=True,exist_ok=True)
    xml=a.xml or cache/'osdev-wiki.xml.gz'; book=a.book or cache/'book.pdf'; mode='dump'; wiki_sha=None
    if a.download and not book.exists(): download(BOOK,book)
    if a.download and not xml.exists():
        try:
            download(WIKI_DUMP,xml)
        except urllib.error.HTTPError as e:
            if e.code != 403: raise
            print('OSDev dump returned HTTP 403; falling back to official MediaWiki API',file=__import__('sys').stderr)
            xml.unlink(missing_ok=True); mode='api'
    if not book.exists(): raise SystemExit('missing book source')
    if xml.exists():
        wiki_sha=digest(xml.read_bytes()); wiki_iter=list(iter_dump(xml)); mode='dump'
    else:
        wiki_iter=list(iter_api()); mode='api'
    build(root,wiki_iter,book,mode,wiki_sha)

if __name__=='__main__':
    main()
