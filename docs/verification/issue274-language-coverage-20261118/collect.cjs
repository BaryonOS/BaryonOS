const fs = require('node:fs');
const path = require('node:path');
const cp = require('node:child_process');
const crypto = require('node:crypto');
const {lexer} = require('marked');
const root = path.resolve(__dirname, '../../..');
const out = path.join(root, 'build/english-coverage-20261118');
fs.mkdirSync(out, {recursive:true});
const wikiArgument = process.argv.indexOf('--wiki');
if(wikiArgument < 0 || !process.argv[wikiArgument+1]) throw new Error('Pass --wiki /absolute/path/to/BaryonOS.wiki');
const wikiRoot = path.resolve(process.argv[wikiArgument+1]);
const gitEnvironment = {...process.env, GIT_NO_LAZY_FETCH: "1", GIT_TERMINAL_PROMPT: "0"};
const git = (...args) => cp.execFileSync('git', ['-C', root, ...args], {encoding:'utf8',env:gitEnvironment}).trim();
const sha256 = raw => crypto.createHash('sha256').update(raw).digest('hex');
const head = git('rev-parse', 'HEAD');
const oldAudit = JSON.parse(fs.readFileSync(path.join(root, 'docs/verification/english-translation-20261109/postmerge-link-audit.json')));
const oldLanguage = JSON.parse(fs.readFileSync(path.join(root, 'docs/verification/english-translation-20261109/english-final-readonly-audit.json')));
const oldPaths = new Set(oldAudit.snapshot.changed_active_paths);
function tree(ref) {
  try {const value=cp.execFileSync('git',['-C',root,'ls-tree','-r',ref],{encoding:'utf8',stdio:['ignore','pipe','pipe'],env:gitEnvironment}).trim();return new Map(value.split('\n').map(l => {const [meta,p] = l.split('\t'); return [p,meta.split(' ')[2]];}));}
  catch {return new Map();}
}
const currentTree = tree(head), oldTree = tree(oldLanguage.head);
function bounded(base, relative) {
  const boundary=fs.realpathSync(base);
  let target=path.resolve(boundary,relative);
  if(target!==boundary&&!target.startsWith(boundary+path.sep)) return null;
  if(fs.existsSync(target)) target=fs.realpathSync(target);
  return target===boundary||target.startsWith(boundary+path.sep)?target:null;
}
const protectedReason = p => {
  if (p.includes('/verification/') || p.startsWith('docs/verification/')) return 'Source-bound historical verification record';
  if (p.startsWith('docs/history/')) return 'Historical archive';
  if (p.startsWith('.agents/skills/os-dev/references/corpus/')) return 'Protected original OSDev/book source corpus';
  if (p.startsWith('docs/third-party/licenses/')) return 'Third-party licensed source';
  if (p.includes('/vendor/') && p !== 'services/keyring/vendor/README.md') return 'Vendor source documentation';
  if (p.startsWith('experiments/quantum/third-party/')) return 'Third-party source';
  return null;
};
const docs = [...currentTree.keys()].filter(p=>p.endsWith('.md'));
const inventory = [], excluded = [], links = [], textByPath = new Map(), anchorsByPath = new Map();
const dutch = /\b(?:het|een|zijn|wordt|worden|voor|geen|alleen|ook|deze|dit|moet|moeten|zonder|niet|gegevens|gebruiker|bestanden|veiligheid|mogelijk|beschikbaar|toestemming|verplicht|ondersteund|huidige|volgende|bewijs|onderzoek|ontwerp)\b/giu;
function slug(value) {return value.replace(/<[^>]*>/g,'').replace(/[*`]/g,'').toLowerCase().replace(/[^\p{L}\p{N}_\- ]/gu,'').replace(/ /g,'-');}
function walk(tokens, visit, seen=new WeakSet()) {
  if (!tokens || typeof tokens!=='object' || seen.has(tokens)) return;
  seen.add(tokens);
  if (!Array.isArray(tokens)) visit(tokens);
  for (const value of Object.values(tokens)) if(value && typeof value==='object') walk(value,visit,seen);
}
for (const p of docs) {
  const reason = protectedReason(p);
  if(reason) {excluded.push({path:p,reason,blob:currentTree.get(p)});continue;}
  const raw=fs.readFileSync(path.join(root,p));const text=raw.toString('utf8');textByPath.set(p,text);
  const tokens=lexer(text);const anchors=new Set();const counts=new Map();
  walk(tokens,t=>{
    if((t.type==='link'||t.type==='image') && t.href) links.push({source:p,href:t.href,image:t.type==='image'});
    if(t.type==='html') for(const m of t.raw.matchAll(/\b(?:href|src)\s*=\s*["']([^"']+)["']/gi)) links.push({source:p,href:m[1],html:true});
    if(t.type==='heading') {const base=slug(t.text);const n=counts.get(base)||0;counts.set(base,n+1);anchors.add(base+(n?'-'+n:''));}
  });
  for(const m of text.matchAll(/<a\s+(?:id|name)=["']([^"']+)["']/gi)) anchors.add(m[1]);
  anchorsByPath.set(p,anchors);
  const hits=[];let fence=null;
  text.split(/\r?\n/).forEach((line,index)=>{
    const m=line.match(/^\s*(`{3,}|~{3,})/);
    if(m) {if(!fence)fence=m[1][0];else if(m[1][0]===fence)fence=null;return;}
    if(fence)return;
    const visible=line.replace(/`+[^`]*`+/g,'').replace(/\]\([^)]*\)/g,']').replace(/https?:\/\/\S+/g,'').replace(/<[^>]*>/g,'');
    const words=[...visible.matchAll(dutch)].map(x=>x[0]);
    if(words.length) hits.push({line:index+1,words:[...new Set(words)],quoted:line.trimStart().startsWith('>'),excerpt:visible.trim().slice(0,260)});
  });
  inventory.push({path:p,blob:currentTree.get(p),sha256:sha256(raw),bytes:raw.length,reused_prior_named_english_audit:oldPaths.has(p)&&oldTree.get(p)===currentTree.get(p),dutch_candidate_lines:hits.length,candidates:hits});
}
const edges=new Map(inventory.map(x=>[x.path,new Set()]));const unresolved=[];const legacy=[];const skipped=[];const wikiReferences=[];
for(const l of links) {
  let href=l.href.replace(/&amp;/g,'&');let target,fragment='';
  if(/^(?:mailto|tel|data|javascript):/i.test(href)) continue;
  if(/^https?:\/\//i.test(href)) {
    let u;try{u=new URL(href);}catch{skipped.push({...l,reason:'Unparseable URL'});continue;}
    if(u.hostname!=='github.com')continue;
    const m=u.pathname.match(/^\/(BaryonOS\/BaryonOS|simonvanieperen\/(?:BaryonOS|nieuw-os))\/(blob|tree)\/([^/]+)\/(.*)$/i);
    if(m){if(m[1]!=='BaryonOS/BaryonOS')legacy.push({...l,kind:m[3]==='main'?'legacy-current-main':'legacy-pinned-or-branch'});if(m[3]!=='main')continue;target=decodeURIComponent(m[4]);fragment=decodeURIComponent(u.hash.slice(1));}
    else {const w=u.pathname.match(/^\/(?:BaryonOS\/BaryonOS|simonvanieperen\/(?:BaryonOS|nieuw-os))\/wiki(?:\/(.*))?$/i);if(w){const name=decodeURIComponent(w[1]||'Home')+'.md';const live=bounded(wikiRoot,name);if(!live){skipped.push({...l,reason:'Outside Wiki checkout'});continue;}if(fs.existsSync(live)){wikiReferences.push({...l,wiki_path:name,sha256:sha256(fs.readFileSync(live))});continue;}target='docs/wiki/'+name;}else continue;}
  }else{
    const hash=href.indexOf('#');if(hash>=0){fragment=decodeURIComponent(href.slice(hash+1));href=href.slice(0,hash);}
    const q=href.indexOf('?');if(q>=0)href=href.slice(0,q);
    try{href=decodeURIComponent(href);}catch{skipped.push({...l,reason:'Invalid percent encoding'});continue;}
    const relativeWiki=bounded(wikiRoot,href+'.md');
    if(l.source.startsWith('docs/wiki/')&&href&&!path.posix.extname(href)&&relativeWiki&&fs.existsSync(relativeWiki)){wikiReferences.push({...l,wiki_path:href+'.md',sha256:sha256(fs.readFileSync(relativeWiki))});continue;}
    target=href?path.posix.normalize(path.posix.join(path.posix.dirname(l.source),href)):l.source;
  }
  if(target.startsWith('../')||path.isAbsolute(target)){skipped.push({...l,reason:'Outside repository'});continue;}
  let full=bounded(root,target);
  if(!full){skipped.push({...l,reason:'Resolved target outside repository'});continue;}
  if(fs.existsSync(full)&&fs.statSync(full).isDirectory()){if(fs.existsSync(path.join(full,'README.md'))){target=path.posix.join(target,'README.md');full=path.join(full,'README.md');}else continue;}
  if(!fs.existsSync(full)){unresolved.push({...l,target,fragment,reason:'missing-target'});continue;}
  if(edges.has(target))edges.get(l.source).add(target);
  if(fragment && target.endsWith('.md') && anchorsByPath.has(target) && !anchorsByPath.get(target).has(fragment))unresolved.push({...l,target,fragment,reason:'unresolved-fragment'});
}
const seeds=['README.md','CONTRIBUTING.md','docs/DEVELOPMENT.md','docs/WIKI.md'];const reachable=new Set(seeds.filter(p=>edges.has(p)));let queue=[...reachable];
while(queue.length){const p=queue.pop();for(const t of edges.get(p)||[])if(!reachable.has(t)){reachable.add(t);queue.push(t);}}
for(const i of inventory)i.reachable_from_normal_navigation=reachable.has(i.path);
const candidates=inventory.filter(i=>i.dutch_candidate_lines);
const report={schema_version:1,source_head:head,source_tree:git('rev-parse','HEAD^{tree}'),generated_at_utc:new Date().toISOString(),method:'Tracked first-party Markdown inventory; marked lexer for local/current-main navigation and GitHub-style anchors; Dutch lexical candidates exclude fenced/code-span/URL/HTML metadata and retain quote flags. Candidate detection is not a proof of English quality.',historical_evidence:{language_audit_head:oldLanguage.head,language_audit_named_paths:oldPaths.size,link_audit:oldAudit.head_at_start,old_preexisting_active_findings:oldAudit.preexisting_active_findings},seeds,counts:{tracked_markdown:docs.length,firstparty_candidates:inventory.length,protected_excluded:excluded.length,normal_navigation_reachable:reachable.size,exact_blob_reused_prior_named_audit:inventory.filter(x=>x.reused_prior_named_english_audit).length,documents_with_dutch_candidates:candidates.length,dutch_candidate_lines:candidates.reduce((a,x)=>a+x.dutch_candidate_lines,0),local_links:links.length,unresolved_local_targets:unresolved.length,legacy_url_occurrences:legacy.length,unsupported_local_targets:skipped.length},inventory,protected_exclusions:excluded,unresolved_local_targets:unresolved,legacy_urls:legacy,unsupported_local_targets:skipped,ui_localisation_observation:{source:'67d5859d735836331b6fb971eae89162b7fa8465',file:'ui/native/src/render.rs',symbols:['menu_label','status_label'],finding:'Functions return fixed Dutch user-visible strings; signatures have no locale argument. This bounded observation does not audit every application or establish a complete absence of localisation elsewhere.'},status:'READ-ONLY CANDIDATE AUDIT; manual dispositions and complete DoD verification pending'};
report.historical_evidence.exact_old_tree_available=oldTree.size>0;
report.historical_evidence.exact_old_tree_limit=oldTree.size?'Old snapshot is available; reuse is limited to the explicitly named paths whose Git blobs are identical.':'Old a462e0d snapshot is not available locally. This collector does not fetch and reuses no exact old-blob coverage.';
report.wiki_references=wikiReferences;
report.tool_versions={node:process.version,marked:require('marked/package.json').version};
fs.writeFileSync(path.join(out,'coverage.json'),JSON.stringify(report,null,2)+'\n');
fs.writeFileSync(path.join(out,'candidates.txt'),candidates.map(x=>`${x.path} reachable=${x.reachable_from_normal_navigation} reused=${x.reused_prior_named_english_audit} lines=${x.dutch_candidate_lines}\n`+x.candidates.map(h=>`  ${h.line}${h.quoted?' quote':''}: ${h.excerpt}`).join('\n')).join('\n\n')+'\n');
fs.writeFileSync(path.join(out,'links.txt'),unresolved.map(x=>`${x.source} -> ${x.target}#${x.fragment}: ${x.reason}`).join('\n')+'\n');
console.log(JSON.stringify({counts:report.counts,candidate_files:candidates.map(x=>({path:x.path,reachable:x.reachable_from_normal_navigation,reused:x.reused_prior_named_english_audit,lines:x.dutch_candidate_lines})).slice(0,40),report:path.join(out,'coverage.json')}));
