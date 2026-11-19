#!/usr/bin/env node
'use strict';
const fs=require('node:fs'),path=require('node:path'),cp=require('node:child_process'),crypto=require('node:crypto');
const {lexer,parse}=require('marked');
function option(name,fallback=null){const i=process.argv.indexOf(name);if(i<0)return fallback;if(!process.argv[i+1])throw new Error('Missing '+name);return process.argv[i+1];}
const root=path.resolve(option('--root',process.cwd()));
const wikiRoot=path.resolve(option('--wiki'));
const out=path.resolve(option('--out'));
const reviewInput=path.resolve(option('--review'));
const reviewPath=fs.realpathSync(reviewInput);
const reviewRelative=path.relative(root,reviewPath).split(path.sep).join('/');
if(reviewRelative.startsWith('../')||path.isAbsolute(reviewRelative)||reviewRelative==='')throw new Error('Review manifest escapes repository');
if(!fs.lstatSync(reviewInput).isFile()||reviewInput!==reviewPath)throw new Error('Review manifest must be a canonical regular file');
fs.mkdirSync(out,{recursive:true});
function gitEnvironment(){
  const env={...process.env};
  const exact=new Set(['GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_OBJECT_DIRECTORY','GIT_ALTERNATE_OBJECT_DIRECTORIES','GIT_COMMON_DIR','GIT_CEILING_DIRECTORIES','GIT_PREFIX','GIT_CONFIG','GIT_CONFIG_GLOBAL','GIT_CONFIG_SYSTEM','GIT_CONFIG_PARAMETERS','GIT_NAMESPACE','GIT_REPLACE_REF_BASE','GIT_SHALLOW_FILE','GIT_GRAFT_FILE','GIT_EXEC_PATH']);
  for(const name of Object.keys(env))if(exact.has(name)||/^GIT_CONFIG_(?:COUNT|KEY_\d+|VALUE_\d+)$/.test(name))delete env[name];
  return {...env,GIT_NO_LAZY_FETCH:'1',GIT_NO_REPLACE_OBJECTS:'1',GIT_TERMINAL_PROMPT:'0',GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:'/dev/null'};
}
const environment=gitEnvironment();
function git(repo,...args){return cp.execFileSync('git',['--no-optional-locks','-C',repo,...args],{encoding:'utf8',env:environment,maxBuffer:32*1024*1024}).trim();}
function gitBytes(repo,...args){return cp.execFileSync('git',['--no-optional-locks','-C',repo,...args],{stdio:['ignore','pipe','pipe'],env:environment,maxBuffer:32*1024*1024});}
function commit(repo,ref){if(!/^[0-9a-f]{40}$/.test(ref))throw new Error('A complete immutable commit SHA is required');const value=git(repo,'rev-parse','--verify',ref+'^{commit}');if(value!==ref)throw new Error('Commit identity mismatch');return value;}
function tree(repo,ref){
  const rows=new TextDecoder('utf-8',{fatal:true}).decode(gitBytes(repo,'ls-tree','-r','-z','--full-tree',ref)).split('\0').filter(Boolean);
  return new Map(rows.map(row=>{const tab=row.indexOf('\t');if(tab<0)throw new Error('Invalid Git tree record');const [mode,type,sha]=row.slice(0,tab).split(' ');return [row.slice(tab+1),{mode,type,sha}];}));
}
const regular=entry=>entry&&entry.type==='blob'&&['100644','100755'].includes(entry.mode);
const sha256=value=>crypto.createHash('sha256').update(value).digest('hex');
const utf8=value=>new TextDecoder('utf-8',{fatal:true}).decode(value);
const sourceCommit=commit(root,option('--ref','HEAD'));
const wikiCommit=commit(wikiRoot,option('--wiki-ref','HEAD'));
const sourceTreeId=git(root,'rev-parse',sourceCommit+'^{tree}');
const wikiTreeId=git(wikiRoot,'rev-parse',wikiCommit+'^{tree}');
const sourceTree=tree(root,sourceCommit),wikiTree=tree(wikiRoot,wikiCommit);
function blob(repo,index,relative){const entry=index.get(relative);if(!regular(entry))throw new Error('Non-regular tracked source: '+relative);return gitBytes(repo,'cat-file','blob',entry.sha);}
function protectedReason(p){
  if(p.includes('/verification/')||p.startsWith('docs/verification/'))return 'Source-bound historical verification record';
  if(p.startsWith('docs/history/'))return 'Historical archive';
  if(p.startsWith('.agents/skills/os-dev/references/corpus/'))return 'Protected source corpus';
  if(p.startsWith('docs/third-party/licenses/'))return 'Third-party licensed source';
  if(p.includes('/vendor/')&&p!=='services/keyring/vendor/README.md')return 'Vendor source documentation';
  if(p.startsWith('experiments/quantum/third-party/'))return 'Third-party source';
  return null;
}
const dutch=/\b(?:het|een|zijn|wordt|worden|voor|geen|alleen|ook|deze|dit|moet|moeten|zonder|niet|gegevens|gebruiker|bestanden|veiligheid|mogelijk|beschikbaar|toestemming|verplicht|ondersteund|huidige|volgende|bewijs|onderzoek|ontwerp)\b/giu;
function candidates(text){
  const result=[];let fence=null;
  for(const [index,line] of text.split(/\r?\n/).entries()){
    const marker=line.match(/^ {0,3}((?:\x60){3,}|~{3,})(.*)$/);
    if(marker){const next={character:marker[1][0],length:marker[1].length};if(!fence){if(next.character!=='\x60'||!marker[2].includes('\x60')){fence=next;continue;}}else if(next.character===fence.character&&next.length>=fence.length&&marker[2].trim()===''){fence=null;continue;}}
    if(fence)continue;
    const visible=line.replace(/(?:\x60)+[^\x60]*(?:\x60)+/g,'').replace(/\]\([^)]*\)/g,']').replace(/https?:\/\/\S+/g,'').replace(/<[^>]*>/g,'');
    const words=[...visible.matchAll(dutch)].map(match=>match[0]);
    if(words.length)result.push({line:index+1,words:[...new Set(words)],excerpt:visible.trim().slice(0,260)});
  }
  return result;
}
function slug(value){return value.replace(/<[^>]*>/g,'').replace(/[\x60*]/g,'').toLowerCase().replace(/[^\p{L}\p{N}_\- ]/gu,'').replace(/ /g,'-');}
function visibleHtml(value){for(;;){const next=value.replace(/<!--[\s\S]*?-->/g,'');if(next===value)return value.includes('<!--')?value.slice(0,value.indexOf('<!--')):value;value=next;}}
function walk(tokens,visit,seen=new WeakSet()){if(!tokens||typeof tokens!=='object'||seen.has(tokens))return;seen.add(tokens);if(!Array.isArray(tokens))visit(tokens);for(const value of Object.values(tokens))if(value&&typeof value==='object')walk(value,visit,seen);}
function parseMarkdown(text){
  const links=[],anchors=new Set(),explicitAnchors=new Set(),used=new Set();
  walk(lexer(text),token=>{
    if((token.type==='link'||token.type==='image')&&token.href)links.push({href:token.href,image:token.type==='image'});
    if(token.type==='html'){
      const rendered=visibleHtml(token.raw);
      for(const match of rendered.matchAll(/\b(?:href|src)\s*=\s*["']([^"']+)["']/gi))links.push({href:match[1],html:true});
    }
    if(token.type==='heading'){const base=slug(token.text);let anchor=base,suffix=0;while(used.has(anchor))anchor=base+'-'+(++suffix);used.add(anchor);anchors.add(anchor);}
  });
  const renderedHtml=visibleHtml(parse(text));
  for(const match of renderedHtml.matchAll(/<a\s+(?:id|name)=["']([^"']+)["']/gi)){anchors.add(match[1]);explicitAnchors.add(match[1]);}
  return {links,anchors,explicitAnchors};
}
function decode(value){try{return {value:decodeURIComponent(value),error:null};}catch{return {value:null,error:'Invalid percent encoding'};}}
const documentPaths=[...sourceTree.keys()].filter(p=>p.endsWith('.md'));
const documents=[],protectedExclusions=[],parsedDocuments=new Map(),extractedLinks=[];
for(const relative of documentPaths){
  const reason=protectedReason(relative);if(reason){protectedExclusions.push({path:relative,reason,blob:sourceTree.get(relative).sha});continue;}
  const raw=blob(root,sourceTree,relative),text=utf8(raw),parsed=parseMarkdown(text),hits=candidates(text);parsedDocuments.set(relative,parsed);
  for(const link of parsed.links)extractedLinks.push({source:relative,...link});
  documents.push({path:relative,blob:sourceTree.get(relative).sha,sha256:sha256(raw),bytes:raw.length,candidate_lines:hits.map(hit=>hit.line),candidates:hits});
}
const formPattern=/^\.github\/(?:ISSUE_TEMPLATE|DISCUSSION_TEMPLATE)\/.*\.ya?ml$/i;
const communityForms=[...sourceTree.keys()].filter(p=>formPattern.test(p)).map(relative=>{const raw=blob(root,sourceTree,relative),hits=candidates(utf8(raw));return {path:relative,blob:sourceTree.get(relative).sha,sha256:sha256(raw),bytes:raw.length,candidate_lines:hits.map(hit=>hit.line)};});
const wikiParsed=new Map();
function wikiPage(relative){const name=relative.endsWith('.md')?relative:relative+'.md';if(!regular(wikiTree.get(name)))return null;if(!wikiParsed.has(name)){const raw=blob(wikiRoot,wikiTree,name);wikiParsed.set(name,{raw,...parseMarkdown(utf8(raw))});}return {path:name,...wikiParsed.get(name)};}
function repositoryTarget(relative){if(regular(sourceTree.get(relative)))return relative;const readme=path.posix.join(relative,'README.md');if(regular(sourceTree.get(readme)))return readme;if([...sourceTree.keys()].some(item=>item.startsWith(relative.replace(/\/$/,'')+'/')))return 'directory';return null;}
function validateFragment(source,target,fragment,anchors,unresolved){if(fragment&&!anchors.has(fragment))unresolved.push({source,target,fragment,reason:'unresolved-fragment'});}
const unresolved=[],skipped=[],legacy=[],wikiReferences=[];
const edges=new Map(documents.map(document=>[document.path,new Set()]));let classifiedLocalLinks=0;
for(const item of extractedLinks){
  let href=item.href.replace(/&amp;/g,'&'),target=null,fragment='';if(/^(?:mailto|tel|data|javascript):/i.test(href))continue;
  if(/^https?:\/\//i.test(href)){
    let url;try{url=new URL(href);}catch{skipped.push({...item,reason:'Unparseable URL'});continue;}if(url.hostname!=='github.com')continue;
    const repoMatch=url.pathname.match(/^\/(BaryonOS\/BaryonOS|simonvanieperen\/(?:BaryonOS|nieuw-os))\/(blob|tree)\/([^/]+)\/(.*)$/i);
    if(repoMatch){const dp=decode(repoMatch[4]),df=decode(url.hash.slice(1));if(dp.error||df.error){skipped.push({...item,reason:'Invalid percent encoding'});continue;}if(repoMatch[1]!=='BaryonOS/BaryonOS')legacy.push({...item,kind:'legacy-repository'});if(repoMatch[3]!=='main')continue;target=dp.value;fragment=df.value;classifiedLocalLinks++;}
    else{
      const match=url.pathname.match(/^\/(?:BaryonOS\/BaryonOS|simonvanieperen\/(?:BaryonOS|nieuw-os))\/wiki(?:\/(.*))?$/i);if(!match)continue;
      const dp=decode(match[1]||'Home'),df=decode(url.hash.slice(1));if(dp.error||df.error){skipped.push({...item,reason:'Invalid percent encoding'});continue;}const page=wikiPage(dp.value);classifiedLocalLinks++;
      if(!page){unresolved.push({...item,target:dp.value+'.md',fragment:df.value,reason:'missing-wiki-target'});continue;}validateFragment(item.source,page.path,df.value,page.anchors,unresolved);wikiReferences.push({...item,wiki_path:page.path,blob:wikiTree.get(page.path).sha,sha256:sha256(page.raw),fragment:df.value});continue;
    }
  }else{
    const hash=href.indexOf('#');if(hash>=0){const df=decode(href.slice(hash+1));if(df.error){skipped.push({...item,reason:df.error});continue;}fragment=df.value;href=href.slice(0,hash);}const query=href.indexOf('?');if(query>=0)href=href.slice(0,query);const dp=decode(href);if(dp.error){skipped.push({...item,reason:dp.error});continue;}href=dp.value;
    if(item.source.startsWith('docs/wiki/')&&href&&!path.posix.extname(href)){const page=wikiPage(href);classifiedLocalLinks++;if(!page){unresolved.push({...item,target:href+'.md',fragment,reason:'missing-wiki-target'});continue;}validateFragment(item.source,page.path,fragment,page.anchors,unresolved);wikiReferences.push({...item,wiki_path:page.path,blob:wikiTree.get(page.path).sha,sha256:sha256(page.raw),fragment});continue;}
    target=href?path.posix.normalize(path.posix.join(path.posix.dirname(item.source),href)):item.source;classifiedLocalLinks++;
  }
  if(target.startsWith('../')||path.posix.isAbsolute(target)){skipped.push({...item,reason:'Outside repository'});continue;}const resolved=repositoryTarget(target);if(!resolved){unresolved.push({...item,target,fragment,reason:'missing-target'});continue;}if(resolved==='directory')continue;if(edges.has(resolved))edges.get(item.source).add(resolved);if(fragment&&resolved.endsWith('.md')){const parsed=parsedDocuments.get(resolved);if(parsed)validateFragment(item.source,resolved,fragment,parsed.anchors,unresolved);}
}
const seeds=['README.md','CONTRIBUTING.md','docs/DEVELOPMENT.md','docs/WIKI.md'],reachable=new Set(seeds.filter(seed=>edges.has(seed))),queue=[...reachable];
while(queue.length){const source=queue.pop();for(const target of edges.get(source)||[])if(!reachable.has(target)){reachable.add(target);queue.push(target);}}
for(const document of documents)document.reachable_from_normal_navigation=reachable.has(document.path);
const anchors=[];for(const [relative,parsed] of parsedDocuments)for(const anchor of parsed.explicitAnchors)anchors.push({path:relative,legacy_anchor:anchor});
const reviewRaw=fs.readFileSync(reviewPath);
const counts={tracked_markdown:documentPaths.length,firstparty_candidates:documents.length,protected_excluded:protectedExclusions.length,normal_navigation_reachable:reachable.size,documents_with_dutch_candidates:documents.filter(d=>d.candidate_lines.length).length,dutch_candidate_lines:documents.reduce((n,d)=>n+d.candidate_lines.length,0),extracted_links:extractedLinks.length,classified_local_links:classifiedLocalLinks,unresolved_local_targets:unresolved.length,legacy_url_occurrences:legacy.length,unsupported_local_targets:skipped.length,wiki_references:wikiReferences.length,community_forms:communityForms.length};
const generated=new Date().toISOString(),method='Immutable Git-tree Markdown/YAML inventory; exact repository and Wiki blobs; candidate detection is not language-quality proof.';
const coverage={schema_version:2,source_commit:sourceCommit,source_tree:sourceTreeId,wiki_commit:wikiCommit,wiki_tree:wikiTreeId,generated_at_utc:generated,method,combined_review_manifest:{path:reviewRelative,sha256:sha256(reviewRaw)},counts,documents,community_forms:communityForms,compatibility_anchors:anchors,protected_exclusions:protectedExclusions,full_issue274_acceptance:false,ui_localisation:'OPEN'};
const collector={schema_version:2,source_commit:sourceCommit,source_tree:sourceTreeId,wiki_commit:wikiCommit,wiki_tree:wikiTreeId,generated_at_utc:generated,method,counts,seeds,unresolved_local_targets:unresolved,unsupported_local_targets:skipped,legacy_urls:legacy,wiki_references:wikiReferences,status:'READ-ONLY CANDIDATE AUDIT; issue #274 remains open'};
fs.writeFileSync(path.join(out,'coverage.json'),JSON.stringify(coverage,null,2)+'\n');fs.writeFileSync(path.join(out,'collector.json'),JSON.stringify(collector,null,2)+'\n');console.log(JSON.stringify({source_commit:sourceCommit,wiki_commit:wikiCommit,counts,output:out}));
