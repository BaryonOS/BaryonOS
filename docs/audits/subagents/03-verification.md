# AUDIT-03 - reproduceerbaarheid en bewijsbinding

**Uitkomst: NIET VAST TE STELLEN voor native acceptatie van de actuele
samengestelde bron en de volledige testrelease. GEFAALD voor de gecombineerde
volledigheids- en integriteitscontrole van de SHA256SUMS-archieven.** De beperkte eigen
hostcontroles slagen. Historische native rapporten zijn gedeeltelijk goed
herleesbaar, maar zijn geen opnieuw uitgevoerde VM-tests.

## Opdracht, bron en afbakening

- Opdracht: "Controleer de reproduceerbaarheid van bestaand bewijs, tests,
  CI-resultaten en geclaimde functionaliteit."
- Uitvoerder: gewone chatsessie die `.github/agents/os-verifier.agent.md`
  heeft gelezen en gevolgd; geen technisch geladen custom agent. Dit is
  AI-verificatie, geen onafhankelijke menselijke audit.
- Datum: 3 november 2026, Europe/Amsterdam.
- Bronbranch volgens opdracht en lokale ref:
  `copilot/add-os-specialist-agents`.
- Exact onderzochte commit:
  `beafdedfc188154fe7285848fbd411e651013f54`.
- Eigen oorspronkelijke werkbranch:
  `simonvanieperen-redesigned-funicular`, oorspronkelijk HEAD
  `69809c2cb53dfb3bd77abe5352892810fba6e82d`.
- `git status --short` was leeg. De commit bestond lokaal. Daarna uitsluitend
  in de eigen werkboom `git switch --detach` naar de opgegeven commit, exit 0.
  De werkstand tijdens onderzoek is dus **detached HEAD**, niet de bronbranch.
- Werkpad:
  `/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-redesigned-funicular`.
- Actuele bronvingerafdruk volgens `tools/common.py:68-84`, source-v3:
  `fe0fe328ef3909e2dab4386c0ba77ebf3cf6a8244301e245863b21debfde4c99`.
  Dit is een hash van de door die functie geselecteerde build-/toolbronnen,
  niet van alle documentatie of alle workflows.
- Alleen dit rapport wordt geschreven. Geen productwijzigingen, builds,
  downloads, externe diensten, commits, pushes, PR-acties, scorewijzigingen
  of wijzigingen in projectstatus. Git-objecten van historische bronnen zijn
  alleen gebruikt om expliciet historische ontvangstbewijzen te controleren.

Gelezen instructies: root `AGENTS.md`, `docs/AGENTS.md`, verifierprofiel,
OS-dev-skill, `tests/AGENTS.md`, README, actuele handoff, privacycontract,
releaseacceptatie, teststrategie, relevante workflowafspraken en ADR-063/
ADR-028. Er staan op deze bron geen verdere geneste AGENTS-bestanden onder
`docs/audits/` of `.github/agents/`. De specifieke schrijfbeperking van deze
audit gaat voor het normale bijwerken van score/projectstatus.

De controle omvat alle 102 door Git gevolgde `SHA256SUMS` onder
`docs/verification/`, plus geselecteerde bindings rond U01, N01, GUI-idle,
CI en behoud van historische documentheads. Dit is geen volledige
inhoudelijke audit van ieder JSON-rapport, iedere test of iedere productclaim.
Er is geen algemene product-, veiligheids-, conformiteits- of hardwaregoedkeuring.

## Bewijsniveaus en beslisregel

E0 = ontbrekend/ambitie; E1 = ontwerp; E2 = hostprototype/simulatie;
E3 = native code; E4 = native positief; E5 = native negatief;
E6 = herhaalde CI/releasegate; E7 = gedocumenteerde hardwaretest;
E8 = onafhankelijke menselijke audit/formele evaluatie.

Een bewaarde beschrijving van E4-E6 is niet een door deze audit herhaalde
E4-E6-uitvoering. Een hashmatch bewijst byte-integriteit ten opzichte van het
opgeslagen manifest, niet de waarheid van iedere assertion in het bestand.
`GESLAAGD` hieronder geldt uitsluitend voor de expliciet benoemde eigen
controle; ontbrekend native bewijs blijft `NIET VAST TE STELLEN`.

## Bewijs-, eis- en reproductiematrix

| Eis/claim | Concrete bron | Historische laag en bronbinding | Eigen controle | Oordeel en grens |
|---|---|---|---|---|
| Exacte auditbron | Verifierprofiel:16-25; Git HEAD | Geen runtimeclaim | Schone werkboom, commitobject, detached checkout en HEAD gecontroleerd | GESLAAGD voor bronselectie |
| 41/100 ontwikkelrubric | `README.md:33-80`; `tools/progress.py:33-70,185-211` | Register, geen releasebewijs | `python3 -B tools/progress.py --check`, exit 0 | GESLAAGD voor berekening, lokale verwijzingen en weergaven; geen herbeoordeling van alle criteria |
| Echte eigen boot-/procesbasis, K01-K05 | `docs/verification/full-release-baseline/README.md:3-20`; `docs/progress.json:19-71` | E4/E5 gerapporteerd op `73ebc1e169b893eb5aba4b4f8ac06d7798136de3`; K05 ook oudere specifieke bronnen | Baselinebestanden vallen in de SHA256SUMS-controle | Integriteit van opgenomen bestanden controleerbaar; oorspronkelijke 238 Rust-hosttests niet herhaald en volledige stdout volgens bron niet gearchiveerd |
| Opslag/crashherstel, S06 | `docs/verification/s06/SHA256SUMS`; `docs/verification/s06/volume-gate.json` | E4/E5 gerapporteerd, 9 stages; source-v3 `528dfe4c0535a0f68dc72a9928a9b8494e9a608fa41e3ed2dc41f75241b8f383` | Manifestbestanden bytegewijs gecontroleerd | Geen herhaalde schijf-/VM-oracle; geen nieuw FR-O01/O04-aftekenen |
| U01 compositorherstel en twee apps | `docs/verification/mature-session-20261102/U01-ACCEPTANCE.md:3-55`; gelijknamige JSON:5-14 | E4/E5 en CI gerapporteerd op `337f2f29f1833ef4a8d2cd640b09419f85dbb83b`, source-v3 `3919403542c923bc86467db3184345159a5ce6881567a0e11baa85f2d39f1995` | Volledig resultaat, gecomprimeerde en uitgepakte serial, CI- en pakketreceipt: 5 hashbindingen gelijk | GESLAAGD voor deze archiefbindingen; native gedrag op auditbron NIET VAST TE STELLEN |
| Volledige pixel-/snapshotorakels mogen corruptie niet accepteren | `tools/test_window_session.py:19-89` | Harnastests, geen OS-uitvoering | 10 tests, exit 0; o.a. verkeerde pixels, ontbrekende/duplicaatvelden, partiele snapshot, ongeldige crop, diagnostic-only versus clipboard | GESLAAGD voor actuele hostharnasgevallen, E2; geen levende GUI-/IPC-grens herhaald |
| Historische hosttesttellingen moeten onvolledige logs weigeren | `tools/tests/test_materiaal_host_count.py:15-69` | Hostlogparser, E2 | 5 tests, exit 0; ontbrekende/dubbele/mislukte/ignored resultaten geweigerd | GESLAAGD voor parsergevallen; geen herhaling van de Rust-tests die het historische log maakten |
| N01 native TCP/UDP-service en clientverlies | `docs/verification/mature-session-20261102/n01-integrated.json:5-29,114-116,158-182,269` | E4/E5 gerapporteerd op `ba117d25bf4880ea5ffd302808282d6f1b4dd032` en `e6ccff32fb9c4816cc3ab480c6fca17a4d992c3f`, dezelfde historische source-v3 `bfbb9ff0c2202403e3d4c9979dd5838099697692109f3502e95f2930de5d98a9` | SHA-256 en byteaantal van beide opgenomen serialbestanden gelijk aan receipt | GESLAAGD voor ruwe logbinding; geen actuele packets, netwerkconfiguratie, TLS of FR-N01/N03 bewezen |
| 18 groene workflows | `docs/verification/gui-idle-20261102/ci-2a7b744-workflows.json`; `docs/verification/mature-session-20261102/ci-337-workflows.json` | E6 gerapporteerd voor twee historische heads; CI-mergeheads apart benoemd | Beide lijsten bevatten 18 completed/success-rijen; GUI-idle native gzip/serial en 10 integratiebestandhashes gecontroleerd | Archief intern consistent; live runstatus, volledige paginering, huidige CI en opnieuw draaien NIET VAST TE STELLEN |
| CPU-mediaan 99,37% naar 11,67%, 88,26% lager | `README.md:94-98`; `docs/verification/gui-idle-20261102/windows-ab/result.json:1738-1783` | Zes historische headless Windows/WHPX-VM-proeven; A=`337f2f29`, B=`1f2ac0ae7cf8944d77ca0656affb01fe9b086f1b` | Oude 100ns-tellers met Decimal herberekend; gzip-origineel/JSON-waarden en 18 A/B-indexbindingen gecontroleerd | GESLAAGD voor historische rekenkundige claim; 99,37236696 -> 11,66628786, daling 88,26002820%; geen nieuwe meting, energieclaim of E7-deviceacceptatie |
| Historische documentinhoud behouden | `docs/verification/retained-document-heads-20261103.json:4-14` en volgende snapshots | E1/archief, expliciet geen runtimebewijs | Alle 20 UTF-8-snapshots: SHA-256, Gitblob-ID en `git show <oude-head>:<pad>` exact gelijk | GESLAAGD voor archiefbehoud, geen functionele acceptatie |
| Alle SHA256SUMS moeten herleesbaar zijn | 102 manifests onder `docs/verification/` | Archiefintegriteit, geen bewijsniveauverhoging | 1814 entries: 1788 gelijk, 25 ontbrekend, 1 afwijkend | GEFAALD; zie V03-01/02 |
| Volledige finale release met hardware/mensenpoort | `docs/FULL-TEST-RELEASE.md:96-120,225-233`; `docs/PROJECT-STATE.md:31-44` | Vereist onder meer N/D/V/T/P/H/A/X; huidige deelbewijzen zijn niet hetzelfde finale image | Geen native/CI/hardware/menselijke herhaling toegestaan in deze opdracht | NIET VAST TE STELLEN; geen aftekenen FR-R01/R02/R03/R04 of FR-B05 |

## Geprioriteerde bevindingen

### V03-01 - HOOG - 25 in manifests opgenomen logbestanden ontbreken

**Vaststelling:** de gecombineerde hashcontrole faalt op 25 werkelijk
ontbrekende bestanden na correcte behandeling van rootrelatieve paden.

| Manifest en regels | Ontbrekend |
|---|---|
| `docs/verification/full-release-pr58-v3-components/SHA256SUMS:12-30` | 19 hostlogs: vijf testlogs `10-test-root.log` t/m `14-test-ui-native.log`, vijf fmt-logs `20-fmt-root.log` t/m `24-fmt-ui-native.log`, vijf clippy-logs `30-clippy-capability-core.log` t/m `34-clippy-ui-native.log`, plus `pr58-v3-core-targeted.log`, `pr58-v3-python.log`, `pr58-v3-system-host.log`, `pr58-v3-update-host.log` |
| `docs/verification/native-display-input-bff1ed7/SHA256SUMS:6-7` | `build.log`, `native-run.log` |
| `docs/verification/usb-descriptor-core-f8e7490/SHA256SUMS:2-5` | `host-test.log`, `clippy.log`, `format.log`, `target-build.log` |

De twee laatste READMEs zeggen uitdrukkelijk dat deze logs uitvoer bewaren:
`native-display-input-bff1ed7/README.md:56-60` en
`usb-descriptor-core-f8e7490/README.md:22-26`. In deze bron ontbreken ze.
`git check-ignore -v` bevestigt voor drie representatieve paden de globale
regel `.gitignore:20:*.log`; dat verklaart een concreet archiveringsrisico,
maar bewijst niet hoe de historische omissie is ontstaan.

**Impact:** genoemde historische commando's kunnen niet op hun volledige
opgegeven loguitvoer worden nagelopen. De originele uitvoering is daarmee
NIET VAST TE STELLEN op basis van deze ontbrekende stukken; niet automatisch
een mislukte native uitvoering of bewezen fout in de USB-/displaycode.
**Herstelcriterium:** oorspronkelijke bytes controleerbaar terugbrengen of
de lacune expliciet maken en op een vastgelegde bron opnieuw meten. Geen
succeslogs reconstrueren uit verwachte markers, hashes of samenvattingen.

### V03-02 - MIDDEL - gedocumenteerde RTC-hashcontrole faalt

`docs/verification/rtc-native/SHA256SUMS:1` verwacht voor zijn README:

`ac1eff95f8e879e94145d1e85ead505de3620d0a0907ed1a0c0f646103405fe0`.

De actuele bytes leveren:

`2b33bdb9362a2e8406e068a9f82f1f4774aca6bfe27aa1b524e5e9ecca048d14`.

Het exacte in `docs/verification/rtc-native/README.md:169-173` genoemde
`sha256sum -c`-commando is zelf herhaald met `--quiet`: exit 1, uitsluitend
README `FAILED`. De README bevat ook later CI-checkpointproza op regels
177-184. Die aanwezigheid is geen sluitend onderzoek naar de historische
oorzaak, maar het manifest bindt in ieder geval niet de huidige README.

**Impact:** het gehele RTC-tekstarchief is op deze stand niet integraal groen
te verklaren. De overige 77 entries van dit manifest matchen; geen bewijs
voor een RTC-driverregressie. **Herstelcriterium:** oude en nieuwe
documentversie onderscheiden en het bedoelde archief opnieuw expliciet
verankeren, zonder historisch runtimebewijs naar nieuwe code te promoveren.

### V03-03 - BLOKKEREND voor actuele releaseacceptatie - historische proeven zijn geen gecombineerd image

De actuele source-v3 is anders dan de geselecteerde U01-, GUI-idle- en
N01-bewijzen. `U01-ACCEPTANCE.json:5-6` en
`n01-integrated.json:10-12,163-165` registreren dat zelf correct. De U01-
en N01-runnerprofielen en historische versies zijn niet een en hetzelfde
releaseimage. `git grep` vindt de exacte auditcommit niet in
`docs/verification/`; dat laatste is op zichzelf geen defect, want
`git show --stat HEAD` toont alleen zestien nieuwe agentprofielen.
Het rechtvaardigt ook geen automatische omzetting van oudere native tests
naar deze geïntegreerde bron.

Het releasecontract eist bron-/artifactbinding en herbeoordeling na relevante
wijzigingen (`FULL-TEST-RELEASE.md:98-107`), en een finaal gecombineerd pakket
(`:230-233`). Het checkpoint noemt actuele gezamenlijke CI/review en een
ontbrekend hosted fixtureartifact/run-ID expliciet open
(`PROJECT-STATE.md:31-44`). Die laatste blocker is hier niet extern onderzocht.

**Impact:** geen goedkeuring van actuele native volledigheid of FR-R04.
Dit is een bekende acceptatiegrens, geen aangetoonde nieuwe runtimebug.
**Herstelcriterium:** de relevante poorten op de exacte combinatie uitvoeren,
configuratie en binaries binden en het integrale pakket in de voorgeschreven
schone omgeving laten controleren. Niet binnen deze read-only opdracht doen.

### V03-04 - ONBEKEND - live CI en volledige artifactbeschikbaarheid

De historische lijsten zijn lokaal leesbaar en bevatten ieder 18 successen.
Dat herhaalt geen workflow en haalt geen actuele API-status op.
`ci-2a7b744-workflows.json` omschrijft zelf een eerste pagina; volledigheid
ten opzichte van alle verplichte checks is niet onafhankelijk opnieuw
vastgesteld. `window-337-ci-verification.json:9-13` verwijst naar een ZIP
onder `/workspace/attachments/`; dit is geen bestand in de eigen werkboom.
Het volledige U01-resultaat verwijst naar een runmap onder
`/workspace/nieuw-os/build/window-final/`. Er is niet buiten de eigen
werkboom naar die oude paden gezocht.

De actuele workflow bewaart onder meer ESP, resultaten, serial, volumes en
PPM's, ook bij falen (`.github/workflows/native-window-session.yml:179-203`),
maar hanteert `retention-days: 14` op regel 203. Dit is een bewaarrisico,
geen vaststelling dat de remote artifacts nu verdwenen zijn. Downloaden of
API-bevraging was niet toegestaan. Geen volledige onafhankelijke pixel-/
volumehercontrole uit enkel de compact opgeslagen hashes geclaimd.

### V03-05 - LAAG - historische hostbaseline mist volledige ruwe stdout

`docs/verification/full-release-baseline/README.md:8-11` vermeldt zelf dat
de volledige uitvoer van 238 Rust-tests niet apart is gearchiveerd. De
ontbrekende ruwe uitvoer kan deze audit niet terughalen. De rapportage
beperkt de claim correct; dit is een bewijslek, geen verborgen groen
en geen nieuwe kritieke bevinding.

Er is binnen deze verificatiescope geen onderbouwde bevinding met ernst
KRITIEK vastgesteld. Dit is niet een verklaring dat kritieke productfouten
afwezig zijn.

## Eigen uitgevoerde commando's en resultaten

Alle commando's zijn lokaal uitgevoerd. Geen nieuw blijvend test-/buildlog
in de repository gemaakt; relevante resultaten staan hieronder en in het
tooltranscript van deze sessie. Alleen de twee unittest-runners rapporteren
hier hun gemeten testtijd; overige commandodoorlooptijden zijn niet apart
gemeten. Lees-/zoekcommando's vormden geen producttest.

| ID | Exact commando of hieronder letterlijk opgenomen script | Exitstatus en waarneming |
|---|---|---|
| C01 | `git branch --show-current && git rev-parse HEAD && git status --short && git worktree list && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54 && if test -z "$(git status --porcelain)"; then git switch --detach beafdedfc188154fe7285848fbd411e651013f54; else printf 'STOP: dirty worktree\n'; exit 2; fi` | 0; eigen werkboom schoon, commit gevonden, checkout op exacte bron |
| C02 | `python3 -B tools/progress.py --check` | 0; 41/100 consistent; eigen output zegt expliciet dat geen bewijsinhoud/native tests worden beoordeeld |
| C03 | `python3 -B -m unittest -v tools.tests.test_materiaal_host_count` | 0; 5/5, 0,002 s |
| C04 | `python3 -B tools/test_window_session.py -v` | 0; 10/10, 0,047 s |
| C05 | `sha256sum --quiet -c docs/verification/rtc-native/SHA256SUMS` | 1; README FAILED; 1 checksum mismatch |
| C06 | Manifestscript hieronder | 1; 102 manifests, 1814 entries, 1788 gelijk, 25 ontbrekend, 1 mismatch |
| C07 | Bron-/receipt-/CI-bindingsscript hieronder | 0; actuele source-v3; 20 snapshots, 5 U01-bindingen, 10 integratiehashes, 2 uitgepakte CI-hashes; twee historische lijsten ieder 18 successen |
| C08 | CPU-script hieronder | 0; zes ruwe tellers consistent; originele gzip-hash/JSON-waarden gelijk; mediaandaling 88,26002820% |
| C09 | A/B-indexscript hieronder | 0; 12 uitgepakte native/serial-hashes en 6 genormaliseerde consolehashes gelijk |
| C10 | `git grep -n beafdedfc188154fe7285848fbd411e651013f54 HEAD -- docs/verification` | 1; geen letterlijke verwijzing; geen runtimefout |
| C11 | `git check-ignore -v docs/verification/native-display-input-bff1ed7/build.log docs/verification/usb-descriptor-core-f8e7490/host-test.log docs/verification/full-release-pr58-v3-components/host/10-test-root.log` | 0; alle drie vallen onder `.gitignore:20:*.log` |
| C12 | N01-script hieronder | 0; beide serialhashes en byteaantallen gelijk; historische rawexit 33/passed true uit receipt, niet nieuw uitgevoerd |

Een eerdere verkennende versie van C06 behandelde alle paden als
manifestrelatief: exit 1, 1700 gelijk en 114 ontbrekend. Dat resultaat
was **geen geldige eindtelling**: 108 entries gebruiken expliciet
`docs/verification/...` vanaf de repositoryroot. C06 hieronder corrigeert
alleen die padinterpretatie; het verbergt geen ontbrekende bestanden.
Er zijn geen 114 ontbrekende artifacts als bevinding gerapporteerd.

Bij de mechanische rapportcontrole verwachtte een eerste controlescript ten
onrechte 12 codefences en faalde met AssertionError; er zijn vijf shellblokken
met tien fences. Dat is een fout in de auditcontrole, niet in de producttests.
`git diff --no-index --check -- /dev/null <rapport>` gaf exit 1 zonder
whitespacewaarschuwingen bij het nieuwe bestand; `git diff --exit-code` gaf
exit 0 voor alle gevolgde bestanden. De uiteindelijke scope-/structuurcontrole
hieronder is afzonderlijk herhaald.

### C06 - alle gevolgde SHA256SUMS, read-only

```sh
python3 -B - <<'PY'
import hashlib,re,subprocess
from pathlib import Path
root=Path.cwd().resolve(); names=subprocess.check_output(['git','ls-files','docs/verification/**/SHA256SUMS'],text=True).splitlines()
c={'manifests':len(names),'entries':0,'matched':0,'missing':0,'mismatch':0,'root_relative':0}; issues=[]
for name in names:
 p=Path(name)
 for n,line in enumerate(p.read_text().splitlines(),1):
  if not line.strip():continue
  c['entries']+=1
  m=re.fullmatch(r'([0-9a-f]{64}) [ *](.+)',line); assert m,(name,n)
  expected,rel=m.groups()
  target=(root/rel if rel.startswith('docs/verification/') else p.parent/rel).resolve()
  c['root_relative']+=rel.startswith('docs/verification/')
  assert target.is_relative_to(root)
  if not target.is_file():c['missing']+=1; issues.append((name,n,'MISSING',rel));continue
  got=hashlib.sha256(target.read_bytes()).hexdigest()
  if got!=expected:c['mismatch']+=1;issues.append((name,n,'MISMATCH',rel,expected,got))
  else:c['matched']+=1
print(c)
for i in issues:print(*i,sep=' | ')
raise SystemExit(bool(issues))
PY
```

### C07 - actuele bron en geselecteerde historische bindings

```sh
python3 -B - <<'PY'
import sys,json,hashlib,gzip,subprocess
from pathlib import Path
sys.path.insert(0,'tools')
from common import source_hash
print('CURRENT_SOURCE_V3',source_hash())
p=Path('docs/verification/retained-document-heads-20261103.json');d=json.loads(p.read_text());n=0
for s in d['sources']:
 for x in s['central_source_snapshots']:
  b=x['utf8'].encode(); assert hashlib.sha256(b).hexdigest()==x['sha256']
  blob=hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();assert blob==x['git_blob']
  assert subprocess.check_output(['git','show',s['head']+':'+x['path']])==b
  n+=1
print('RETAINED_SNAPSHOTS_SHA256_GITBLOB_SOURCE_MATCH',n)
p=Path('docs/verification/mature-session-20261102');d=json.loads((p/'U01-ACCEPTANCE.json').read_text())
for name,key in [(d['full_result'],'full_result_sha256'),('window-337-serial.txt.gz','serial_gzip_sha256'),('window-337-ci-verification.json','CI_verification_sha256'),('window-337-package-verification.json','package_verification_sha256')]:
 assert hashlib.sha256((p/name).read_bytes()).hexdigest()==d[key],name
assert hashlib.sha256(gzip.decompress((p/'window-337-serial.txt.gz').read_bytes())).hexdigest()==d['serial_raw_sha256']
full=json.loads((p/d['full_result']).read_text());print('U01_BINDINGS_MATCH 5', 'historical_source',d['source_commit'],'passed',full.get('passed'),'source_match',full['source_sha256']==d['source_sha256'])
p=Path('docs/verification/gui-idle-20261102');d=json.loads((p/'integration-file-hashes.json').read_text())
for name,v in d.items():
 expected=v if isinstance(v,str) else v['sha256']
 assert hashlib.sha256((p/name).read_bytes()).hexdigest()==expected,name
print('IDLE_INTEGRATION_FILE_HASHES_MATCH',len(d))
ci=json.loads((p/'ci-2a7b744-verification.json').read_text())
for name,key in [('ci-2a7b744-native.json.gz','report_sha256'),('ci-2a7b744-serial.txt.gz','raw_serial_sha256')]:
 assert hashlib.sha256(gzip.decompress((p/name).read_bytes())).hexdigest()==ci[key],name
print('IDLE_CI_GZIP_RAW_BINDINGS_MATCH 2')
for name in ['docs/verification/gui-idle-20261102/ci-2a7b744-workflows.json','docs/verification/mature-session-20261102/ci-337-workflows.json']:
 d=json.loads(Path(name).read_text());rows=d['workflow_runs'];print(name,'rows',len(rows),'success',sum(x['status']=='completed' and x['conclusion']=='success' for x in rows),'historical_source',d.get('head_commit',d.get('source_commit')))
PY
```

### C08 - historische CPU-statistiek, geen nieuwe procesmeting

```sh
python3 -B - <<'PY'
import json,gzip,hashlib,statistics
from decimal import Decimal as D
from pathlib import Path
p=Path('docs/verification/gui-idle-20261102/windows-ab');d=json.loads((p/'result.json').read_text());index=json.loads((p/'index.json').read_text());original=gzip.decompress((p/index['aggregate_original_file']).read_bytes())
assert hashlib.sha256(original).hexdigest()==index['aggregate_sha256'];assert json.loads(original)==d
arms={'A':[],'B':[]}
for run in d['runs']:
 cpu=run['measurement']['cpu'];b=cpu['before']['process'];a=cpu['after']['process']
 assert a['creation_100ns']==b['creation_100ns']
 for x in (a,b):assert x['total_100ns']==x['kernel_100ns']+x['user_100ns']
 wall=(D(str(a['read_started']))+D(str(a['read_completed']))-D(str(b['read_started']))-D(str(b['read_completed'])))/2
 value=D(a['total_100ns']-b['total_100ns'])/D(10000000)/wall*100
 assert abs(value-D(str(run['process_percent_one_logical_cpu'])))<D('0.000001')
 arms[run['arm']].append(value)
 print('RUN',run['index'],run['arm'],'process_cpu_percent',round(value,9),'historical_source',run['source_commit'][:8])
a,b=(statistics.median(arms[k]) for k in ('A','B'));reduction=(1-b/a)*100
assert abs(reduction-D(str(d['statistics']['process_percent_one_logical_cpu']['median_reduction_percent'])))<D('0.000001')
print('ORIGINAL_GZIP_HASH_AND_JSON_VALUES_MATCH; medians',round(a,8),round(b,8),'reduction',round(reduction,8),'percent')
PY
```

### C09 - A/B-archiefbindingen

```sh
python3 -B - <<'PY'
import json,gzip,hashlib
from pathlib import Path
p=Path('docs/verification/gui-idle-20261102/windows-ab');d=json.loads((p/'index.json').read_text());count=0
for group in ['native_reports','serial']:
 for name,expected in d[group].items():
  raw=gzip.decompress((p/name).read_bytes());assert hashlib.sha256(raw).hexdigest()==expected,name;count+=1
for name,item in d['artifacts'].items():
 assert hashlib.sha256((p/name).read_bytes()).hexdigest()==item['included_sha256'],name;count+=1
print('WINDOWS_AB_INDEX_BINDINGS_MATCH',count,'(12 raw decompressed, 6 normalized included console files)')
PY
```

### C12 - N01-serialbindingen

De uitgevoerde N01-controle maakte deel uit van een read-only inspectiescript;
dit is het letterlijk uitgevoerde relevante gedeelte, met dezelfde imports
en padinitialisatie zelfstandig herhaalbaar:

```sh
python3 -B - <<'PY'
import json,hashlib
from pathlib import Path
p=Path('docs/verification/mature-session-20261102')
d=json.loads((p/'n01-integrated.json').read_text())
for c in d['cases']:
 f=p/('n01-normal-serial.txt' if c['case']=='normal' else 'n01-client-exit-serial.txt')
 assert hashlib.sha256(f.read_bytes()).hexdigest()==c['stdout_sha256'];assert len(f.read_bytes())==c['stdout_bytes']
 print('N01 stdout hash/length MATCH',c['case'],c['source_commit'],c['raw_qemu_exit'],c['passed'])
PY
```

## Niet uitgevoerd, aannames en resterende risico's

- Geen Cargo-, Clippy-, crossbuild-, QEMU-, Windows/WHPX-, boot-, netwerk-,
  update-, fault-injection-, koude herstel- of langduurproef gestart. De
  historische native successen en mislukkingen zijn niet opnieuw uitgevoerd.
- Geen live GitHub-/CI-/PR-status, artifactdownload of controle van verlopen
  opslag; geen aanname dat de remote bronbranch na aanvang onveranderd blijft.
  Alle conclusies gelden voor de vaste lokale commit.
- Geen fysieke hardwaretest, echte gegevens, credentials, accountauthenticatie,
  NAS/provider of onafhankelijke menselijke review. Een Windows-host die een
  VM uitvoert is geen E7-driveracceptatie voor Nieuw OS.
- De source-v3-hash wordt conform de bestaande helper gebruikt. Gelijkheid
  daarvan is geen complete supply-chain-/reproduceerbare-buildgarantie.
  C07 verifieert documentbytes tegen historische Git-objecten; hij voert die
  historische bron niet uit.
- De manifestcontrole dekt uitsluitend de entries in de 102 gevolgde
  SHA256SUMS-bestanden. Niet-genoemde bestanden, JSON-hashes buiten de
  steekproef en authenticiteit van degene die de oorspronkelijke rapporten
  schreef, vallen buiten die integriteitsclaim.
- CPU-herberekening neemt de geregistreerde tellers en tijdstempels als
  observatiedata; zij bewijst niet zelfstandig het werkelijke hostgedrag.
  Beperkte samplegrootte, headless WHPX, conditionele gastcounters en
  verschillende historische bronnen blijven gelden.
- De voortgangsmeter is uitsluitend read-only gecontroleerd: opgeslagen
  score 41/100, geen nieuwe punten toegekend of ingetrokken. De hoofdagent
  moet gevolgen van bewijsgebreken afwegen zonder historische proeven naar
  de actuele bron om te labelen.

**Overdracht:** herstel eerst de ontbrekende logarchieven en RTC-manifestbinding
of registreer hun niet-reproduceerbaarheid expliciet. Laat daarna de bevoegde
integrator actuele bron-/artifactgebonden native/CI-gates en het finale
releasepakket controleren. Alleen dit rapport is de oplevering van AUDIT-03.

## Eindcontrole van de rapportoplevering

Onderstaand read-only controlescript eindigt met exit 0. Het controleert
alleen de rapportoplevering; geen aanvullende productacceptatie:

```sh
python3 -B - <<'PY'
import subprocess,sys
from pathlib import Path
sys.path.insert(0,'tools')
from common import source_hash
p=Path('docs/audits/subagents/03-verification.md')
lines=p.read_text().splitlines()
fences=[x for x in lines if x.startswith('```')]
assert len(fences)==12 and fences[::2]==['```sh']*6
assert fences[1::2]==['```']*6
assert all(line==line.rstrip() for line in lines)
assert subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()=='beafdedfc188154fe7285848fbd411e651013f54'
assert subprocess.run(['git','diff','--exit-code']).returncode==0
assert subprocess.check_output(['git','status','--porcelain','--untracked-files=all'],text=True).splitlines()==['?? '+p.as_posix()]
assert source_hash()=='fe0fe328ef3909e2dab4386c0ba77ebf3cf6a8244301e245863b21debfde4c99'
print('FINAL_REPORT_SCOPE_STRUCTURE_SOURCE_OK')
PY
```

De eindcontrole voegt zelf een zesde shellblok toe; daarom verwacht zij twaalf
fences, anders dan de vijf blokken bij de eerste mechanische controle.
