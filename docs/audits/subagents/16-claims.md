# AUDIT-16 - documentatie en bewijsclaims

## Resultaat en opdrachtgrens

Op bron `beafdedfc188154fe7285848fbd411e651013f54` zijn twee concrete
archiefproblemen bevestigd: 25 ontbrekende manifestbestanden en een afwijkende
RTC-README-hash. Dit bevestigt de vooraf gemelde verifierbevindingen; het zijn
geen nieuwe runtimebugs. Daarnaast bestaan al bekende tegenstrijdigheden door
historische tekst die nog als actuele toestand leesbaar is. Geen aangetoonde
inflatie naar een dagelijks bruikbare release, fysieke hardwareacceptatie of
onafhankelijke menselijke audit in de onderzochte hoofdclaims.

De vaste meter telt 21 geaccepteerde criteria met samen **41/100 punten**.
Alle **62 volledige releasecriteria staan open**. Dit onderscheid is consistent:
ontwikkelpunten betekenen niet dat 41% van een dagelijks bruikbaar systeem of
41% van het finale releaseimage is geaccepteerd. Geen scorewijziging voorgesteld.

Opdracht: "Controleer README, roadmap, projectstatus, release notes en
bewijsclaims op inconsistenties en bewijsinflatie." Gewone chatsessie;
`.github/agents/os-claims-auditor.agent.md:10-28` expliciet gelezen, niet
technisch als customagent geladen. Geen subagents gebruikt. Alleen dit rapport
geschreven; geen claims gerepareerd, productbestanden aangepast, voortgang
gegenereerd, commit/push/PR/merge uitgevoerd of gevraagd.

## Identiteit en instructies

| Controle | Waarneming |
|---|---|
| Werkdirectory | `/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-shiny-parakeet` |
| Beginbranch | `simonvanieperen-shiny-parakeet`, dus niet de gevraagde bronbranch |
| Begin-HEAD | `69809c2cb53dfb3bd77abe5352892810fba6e82d` |
| Beginstatus | `git status --short`: leeg; geen gebruikerswerk overschreven |
| Worktree | Eigen gekoppelde worktree; de hoofdcheckout kwam uitsluitend voor in Git-metadata, is niet geopend of gewijzigd |
| Beschikbaarheid bron | `git cat-file -t beafdedfc188154fe7285848fbd411e651013f54`: `commit` |
| Lokale bronrefs | Zowel `refs/heads/copilot/add-os-specialist-agents` als `refs/remotes/origin/copilot/add-os-specialist-agents` wezen naar de exacte auditcommit; geen fetch uitgevoerd |
| Bronselectie | `git switch --detach beafdedfc188154fe7285848fbd411e651013f54`: exit 0; daarna HEAD exact gelijk en werkboom schoon |
| Eindtoestand | Detached op de auditbron; uitsluitend het nieuwe auditrapport toegestaan |

Gelezen instructies: `AGENTS.md:1-100`, `docs/AGENTS.md:1-5`, het eigen profiel
en de lokale os-dev-reviewroute
`.agents/skills/os-dev/references/workflows/06-debugging-and-review.md:1-47`.
De inventaris van tracked geneste `AGENTS.md` bevatte geen extra instructies
onder het rapportpad. README, actuele handoff, privacycontract, releasecontract,
roadmap, voortgangsregister en relevante ADR's zijn gelezen. ADR-058
(`docs/DECISIONS.md:118-145`) laat herkenbare desktopbediening expliciet toe.
Het os-dev-skill is gebruikt als leesroute; CodeRabbit is niet uitgevoerd:
de opdracht staat geen externe reviewdienst of codeoverdracht toe.

## Bewijslegenda en methode

Gedeelde orchestratorlegenda: E0 ontbrekend/ambitie, E1 ontwerp, E2
hostprototype/simulatie/hosttest, E3 native bron, E4 native positief, E5 native
negatief, E6 herhaalde CI/releasegate, E7 fysieke hardware, E8 onafhankelijke
menselijke audit/formele evaluatie. Deze labels vervangen niet de eigen
matrixcodes N/D/V/T/P/H/A/X uit `docs/FULL-TEST-RELEASE.md:103-121`.
Een modelspecificatie alleen is geen formeel bewijs.

Deze audit produceert statische repositorywaarnemingen. Bestaande E4/E5/E6
worden uitsluitend als historische, brongebonden registraties beoordeeld,
niet opnieuw uitgevoerd of aan de auditcommit toegeschreven. Een intacte hash
bewijst integriteit ten opzichte van een manifest, niet de waarheid van de
runregistratie. Een tweede AI of Python-schijforakel is geen E8.

## Concrete bevindingen

### C16-01 - P1: archieven verwijzen naar 25 niet aanwezige bestanden

**Claim en locatie.** `docs/verification/native-display-input-bff1ed7/README.md:52-60`
noemt `build.log` en `native-run.log` als relevante archiefuitvoer;
`docs/verification/usb-descriptor-core-f8e7490/README.md:22-26` zegt dat vier
logs de uitvoer bewaren. De bijbehorende manifests registreren die bestanden.
Ook het componentmanifest registreert 19 ontbrekende hostlogs.

**Bewijs.** De read-only controle van alle 102 tracked
`docs/verification/**/SHA256SUMS` leest 1.814 entries: 1.788 hashmatches,
25 ontbrekende bestanden, 1 mismatch. Ontbrekend:

| Exacte manifestlocatie | Ontbrekende inhoud |
|---|---|
| `docs/verification/full-release-pr58-v3-components/SHA256SUMS:12-30` | 19 hostlogs: vijf testlogs, vijf fmt-logs, vijf Clippy-logs en `pr58-v3-{core-targeted,python,system-host,update-host}.log` |
| `docs/verification/native-display-input-bff1ed7/SHA256SUMS:6-7` | `build.log`, `native-run.log` |
| `docs/verification/usb-descriptor-core-f8e7490/SHA256SUMS:2-5` | `host-test.log`, `clippy.log`, `format.log`, `target-build.log` |

**Probleem en impact.** De genoemde volledige ruwe uitvoer is niet vanuit deze
repositorysnapshot te inspecteren. Dit beperkt herleidbaarheid van specifieke
historische E2/E4/E5-claims; het bewijst geen mislukte native proef, defecte
USB-runtime, vervalste logs of afwezigheid van de overige JSON-/serialgegevens.
De USB-README begrenst de implementatie terecht tot een descriptorlibrary,
zonder USB-controller of device-I/O (`:3-6`).

**Toegestaan alternatief.** "De samenvattingen en manifests zijn opgeslagen;
de volgende 25 genoemde logbestanden ontbreken in deze snapshot."

**Vereiste verificatie / afhankelijkheid.** Laat de bewijseigenaar oorspronkelijke
bytes uit een geautoriseerd archief leveren en tegen de bestaande hashes
controleren, of markeer het archief expliciet onvolledig. Een nieuwe run mag
niet als vervanging van de historische bytes worden voorgesteld. Geen directe
scoreverlaging uit alleen deze archiefgaten afgeleid: daarvoor is een
criteriumspecifieke analyse nodig. Niet dubbel als nieuwe verifierbevinding tellen.

### C16-02 - P2: RTC-manifest dekt de huidige begeleidende tekst niet

**Claim en locatie.** `docs/verification/rtc-native/SHA256SUMS:1` bindt
`docs/verification/rtc-native/README.md` aan
`ac1eff95f8e879e94145d1e85ead505de3620d0a0907ed1a0c0f646103405fe0`.
De huidige bytes hashen naar
`2b33bdb9362a2e8406e068a9f82f1f4774aca6bfe27aa1b524e5e9ecca048d14`.

**Bewijs en oorzaakafbakening.** Git-historie laat op
`c0b7147b601dd3d8fdfe266bf5fa1a74a4c93534` precies de README-verwijzing
`ADR-049` naar `ADR-057` wijzigen; het manifest verandert daar niet mee.
De betrokken actuele verwijzing staat in
`docs/verification/rtc-native/README.md:33`. In deze scan matchen de overige
77 RTC-manifestentries wel.

**Probleem en impact.** Een volledige checksumcontrole van deze snapshot faalt.
Er is een concrete documentatieverklaring voor de mismatch, geen aanwijzing
uit deze hashcontrole voor gewijzigde RTC-binaries of corrupte runtime-uitvoer.
De beperkte RTC-claim blijft bovendien apart van de productie-TimeProvider
(`docs/verification/rtc-native/README.md:3-18`).

**Toegestaan alternatief.** "Het oorspronkelijke manifest bindt de oude
README; de latere ADR-hernummering is afzonderlijk vastgelegd."

**Vereiste verificatie / afhankelijkheid.** Bewaar beide documentversies of leg
de exacte document-only wijziging en nieuwe hash vast zonder het oorspronkelijke
runbewijs te herlabelen. Geen RTC-herbouw nodig om deze tekstafwijking te
verklaren; actuele native acceptatie is een andere vraag. Reeds bekend,
hier zelfstandig bevestigd.

### C16-03 - P2: bekende historische passages zijn als actuele status leesbaar

**Claims en locaties.**

- `README.md:193-211`: "Laatste bouwstap: M6d", "nu" en een volgende stap voor
  automatische dienstherstart, terwijl de native venstersessie en U01 eerder
  op dezelfde pagina staan (`README.md:86-110`).
- `README.md:274-284`: "Materiaal en qsim blijven afzonderlijke hostprototypes"
  en "Er is nog geen interactieve native desktop."
- `docs/PROGRESS.md:79-84` zegt na de eerste meting zonder aanvullende
  tijdsaanduiding dat K05/S08 open blijven. De passage onder de gedateerde
  roadmapstatus is duidelijker historisch (`docs/ROADMAP.md:53-73`).
- De contractkop `docs/LOCAL-PRIVACY-CONTRACT.md:4` noemt nog ontbrekende native
  netwerkstack en sleutelservice.

**Bewijs.** `docs/verification/mature-session-20261102/U01-ACCEPTANCE.md:3-20`
beschrijft brongebonden native vensters en compositorherstel;
`docs/verification/L02-ACCEPTANCE.md:3-15` beschrijft de native software-qsim-port.
Het vaste register heeft K05/S08/U01/N01/L02 geaccepteerd; hun aangehaalde
bewijsbestanden bestaan. De actuele projecthandoff
`docs/PROJECT-STATE.md:13-20,31-45` bewaart juist brononderscheid en open
integratiegrenzen.

**Probleem en impact.** Dit is onderrapportage en chronologische verwarring,
niet bewijsinflatie of bewijs van ontbrekende runtime. Dat de hostprototypes
nog bestaan, betekent niet dat de native deelports ontbreken. Andersom
bewijzen de deelports geen complete NAS, TLS, authenticatie of release.
Deze klasse was vooraf al bekend en wordt niet als nieuwe runtimebevinding
geteld. De historische roadmaptekst is niet zelfstandig fout doordat zij
een oudere gedateerde bron beschrijft.

**Toegestaan alternatief.** Markeer deze passages als historische stand met
datum/bron en verwijs voor huidige deelimplementaties naar de actuele
handoff en geaccepteerde criteriumrapporten.

**Vereiste verificatie / afhankelijkheid.** Eerst onderscheid per historisch
profiel/bron vastleggen; pas daarna afzonderlijk documentatie bijwerken.
Geen oude gate promoveren naar de huidige bron en geen volledige NAS- of
gebruikersauthenticatieclaim toevoegen. Binnen deze audit niets aangepast.

## Onderzochte kernclaims zonder nieuwe tegenbevinding

| Claim en exacte locatie | Bewijssoort en bronafbakening | Oordeel / toegestane formulering |
|---|---|---|
| 41/100 ontwikkelpunten, geen veiligheid/tijd/pariteit: `README.md:33-80`; `docs/PROGRESS.md:7-37` | Register `docs/progress.json`, 21 accepted-criteria; alle genoemde accepted-evidencepaden aanwezig | Rekenkundig consistent; geen heracceptatie van alle 21 criteria. Niet als dagelijkse bruikbaarheid lezen. |
| Volledige release open: `docs/FULL-TEST-RELEASE.md:1-5,37-44,128-140`; `docs/audit/os-completeness/README.md:5-17,70-77` | E1 acceptatieregister; 62/62 matrixrijen open | Geen volledige release, fysieke/human gate of onderhoudsacceptatie geclaimd. |
| U01 geaccepteerd: `README.md:86-110`; `docs/verification/mature-session-20261102/U01-ACCEPTANCE.md:3-6,22-56,67-70` | Historisch E4/E5/E6 op `337f2f29...`; 18 success-conclusies in lokale CI-snapshot; werkelijke CI-merge expliciet apart | Native compositor-deelacceptatie, geen echte accounts, volledige toegankelijkheid of productrelease. |
| Klembordroutes en CI: `README.md:112-128`; `docs/verification/native-clipboard-20261102/README.md:13-23,44-56,88-102` | Historisch E4/E5/E6; d0ae-rootbron, e318-trigger, 5754763-CI-merge en test-only binaire brug afzonderlijk | Lokale snapshot bevat 18 success-conclusies. Geen huidige-head CI-check; p95/dagelijkse bediening blijft open. |
| CPU-daling 88,26%: `README.md:94-99`; `docs/verification/gui-idle-20261102/INTEGRATION.md:17-26,45-71` | Historische zes Windows/WHPX-VM-proeven, drie per arm; bronnen 337/1f2/2a7 afzonderlijk | Beperkte QEMU-procesCPU-meting, niet fysieke energie, universele snelheid of gelijke binaries tussen hosts. Snapshot bevat 18 successes op 2a7. Ruwe CPU-tellers hier niet opnieuw nagerekend. |
| Changelog met 14 commits: `CHANGELOG.md:3-18,20-41` | Expliciet historisch bereik `ad2b981..380111b6`; `git rev-list --count` geeft 14 | Geen release van de auditcommit gesuggereerd. Ontwikkelproeven en beperkte geluidscode worden terecht niet tot hardware-/audioacceptatie verheven. |
| Sleutelrotatie/intrekking: `CHANGELOG.md:14-18` versus oud `docs/verification/B03-ACCEPTANCE.md:109-115` | Oud rapport sluit rotatie uit; later `docs/verification/b03-key-policy-1/update-gate.json:1-9` registreert completed/passed en bevat 18 stages | Geen nieuwe ontbrekende-rotatiebug. Later TEST-policybewijs bestaat; productieankers/B04 blijven apart open (`docs/progress.json:398-404`). |
| Privacy en providers: `README.md:294-302`; `docs/LOCAL-PRIVACY-CONTRACT.md:2,8-19` | E1 bindende productkeuze revisie 4; native privacygate B06 open (`docs/progress.json:415-420`) | Expliciet gekozen opaque provider toegestaan, geen stille fallback of cloud-AI. Ontwerpcontract is geen end-to-end implementatiebewijs. |
| Quantum: `docs/verification/L02-ACCEPTANCE.md:3-19`; `docs/ROADMAP.md:128-130` | Historische native simulatieport 433bab9f / integratie 7fb8c722, geen fysieke QPU | Niet terugzetten naar alleen hostprototype, maar ook geen quantumversnelling/E7 claimen. |
| Securityreview: `docs/verification/security-audit-20261103.md:5-24,27-57`; `docs/PROGRESS.md:114-120` | Statische AI-review zonder opgeleverde bestandsdekking; toenmalige alertstatus; geen volledige secrets-/advisoryscan | Correct begrensd, geen E8 of "geen kwetsbaarheden"-garantie voor de codebase. |
| Releaseclassificatie: `docs/audit/os-completeness/release-profiles.md:3-12,29-31`; `docs/ROADMAP.md:169-177` | E1 profielcriteria, technische preview t/m consumer release, fysieke voorwaarden apart | Geen alpha/beta/consumerstatus uit groene CI afgeleid. |

Een opgeslagen native archief blijft een runregistratie, geen onafhankelijk
gemeten eigenschap op deze auditcommit. De 18-workflowuitspraken hierboven
zijn getoetst aan lokale JSON-snapshots, niet aan de actuele GitHub-status.
De handoff verlangt zelf nieuwe gecombineerde CI/review
(`docs/PROJECT-STATE.md:31-45`); dat is consistent, geen aangetoonde regressie.

## Uitgevoerde commando's, resultaten en reproduceerbaarheid

Alleen lokale leescontroles. Geen build/testharnas gestart dat repo-output
maakt. Geen externe diensten, downloads, installaties, credentials, echte
gegevens, devices of destructieve proeven gebruikt. Leestijd en afzonderlijke
commandoduren zijn niet gemeten; geen snelheidsoordeel uit deze audit.

| Exact commando | Exit / resultaat |
|---|---|
| `pwd && git branch --show-current && git rev-parse HEAD && git status --short && git worktree list --porcelain && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54` | 0; schone eigen beginboom en bestaande broncommit |
| `git switch --detach beafdedfc188154fe7285848fbd411e651013f54` | 0 |
| `git for-each-ref --format='%(refname) %(objectname)' refs/heads/copilot/add-os-specialist-agents refs/remotes/origin/copilot/add-os-specialist-agents` | 0; beide refs exact auditbron |
| `git rev-list --count ad2b981fe1f2bcccb03cb776b2281d32529d2f00..380111b68688c981eec6abb8fab29174a29f5601` | 0; 14 |
| `git log -3 --format='%H %s' -- docs/verification/rtc-native/README.md docs/verification/rtc-native/SHA256SUMS` | 0; wijzigingsherkomst RTC-tekst |
| `git show c0b7147b601dd3d8fdfe266bf5fa1a74a4c93534 -- docs/verification/rtc-native/README.md docs/verification/rtc-native/SHA256SUMS` | 0; ADR-hernummering zonder manifestaanpassing |
| `git status --porcelain=v1 --untracked-files=all` en `git diff --stat` voor schrijven | 0; leeg |

Daarnaast zijn `nl -ba`, `sed`, `head`, `rg`, `git ls-files` en `view` gebruikt
voor bovengenoemde citaten. Enkele verkennende paden bestonden niet:
`docs/verification/b03-key-policy-1/README.md`,
`docs/verification/full-release-pr58-v3-components/{README.md,INDEX.json}`
en de zoekglob `rtc-clock-*`. Dat waren lees-/zoekfouten, geen producttestfouten;
samengestelde shellregels konden ondanks een eerdere `nl`/`rg`-fout exit 0
geven. Deze ontbrekende verkennende namen zijn niet bij de 25 manifestgaten
opgeteld.

De definitieve archiefcontrole was exact:

```sh
python3 -B - <<'PY'
import pathlib,subprocess,re,hashlib,collections,sys
root=pathlib.Path.cwd(); counts=collections.Counter(); manifests=subprocess.check_output(['git','ls-files','docs/verification/**/SHA256SUMS'],text=True).splitlines()
for name in manifests:
 p=pathlib.Path(name)
 for n,line in enumerate(p.read_text().splitlines(),1):
  m=re.fullmatch(r'([0-9a-fA-F]{64})\s+\*?(.+)',line)
  if not m: counts['unparsed']+=1; continue
  counts['entries']+=1
  q=(root/m[2] if m[2].startswith('docs/') else p.parent/m[2]).resolve()
  if not q.is_relative_to(root): counts['outside']+=1; continue
  if not q.is_file(): counts['missing']+=1; continue
  counts['ok' if hashlib.sha256(q.read_bytes()).hexdigest().lower()==m[1].lower() else 'mismatch']+=1
print('manifests',len(manifests),dict(counts))
sys.exit(1 if any(counts[x] for x in ['unparsed','outside','missing','mismatch']) else 0)
PY
```

Resultaat **exit 1**: `manifests 102 {'entries': 1814, 'ok': 1788,
'missing': 25, 'mismatch': 1}`. De scriptlogica leest uitsluitend bestanden
binnen deze worktree. Rootrelatieve `docs/...`-entries en archiefrelatieve
entries zijn onderscheiden. Een eerdere verkennende telling behandelde alles
als archiefrelatief en gaf 114 schijnbaar ontbrekende paden; die telling is
verworpen, niet als bevinding gebruikt. De uiteindelijke foutstatus is een
archiefintegriteitsresultaat, geen runtime-testresultaat.

Aanvullende read-only Python-stdincontroles (`python3 -B - <<'PY'`, exit 0)
hebben JSON met `json.loads` gelezen en:

- alle `tracks[*].gates[*]` met `status == "accepted"` opgeteld:
  21 criteria, 41 punten; hun `evidence`-paden met `Path.is_file()` gecontroleerd;
- matrixregels uit FULL-TEST-RELEASE geselecteerd met
  `line.startswith('| FR-')`: 62 regels, alle 62 met `| open |`;
- `workflow_runs[*].conclusion` geteld uit
  `native-clipboard-20261102/ci-e318-workflows.json`,
  `gui-idle-20261102/ci-2a7b744-workflows.json` en
  `mature-session-20261102/ci-337-workflows.json` onder `docs/verification/`:
  ieder 18 keer `success`;
- de 18 `stages` en `status == "completed"`, `passed == true` uit
  `docs/verification/b03-key-policy-1/update-gate.json` vastgesteld;
- Markdown-links volgens `r'\]\(([^)\s]+)\)'` in README, CHANGELOG, ROADMAP,
  PROGRESS, PROJECT-STATE en FULL-TEST-RELEASE gecontroleerd, met externe
  URL's/mailto/fragment-only links overgeslagen en fragmenten afgesplitst:
  respectievelijk 53/0, 0/0, 19/0, 56/0, 15/0, 14/0 lokale paden/ontbrekende
  paden. Geen controle van ankers, bare paden of externe URL-bereikbaarheid.

De uitvoer staat in het sessietooltranscript; er is conform de opdracht geen
tweede log-/bewijsbestand aangemaakt. Bovenstaand checksumcommando is volledig
opgenomen om de belangrijkste negatieve bevinding te kunnen reproduceren.

Rapportcontroles: `git diff --check` en `git diff --exit-code` exit 0 voor de
bestaande tracked bestanden; `git status --porcelain=v1 --untracked-files=all`
toont uitsluitend `?? docs/audits/subagents/16-claims.md`. Een Python-controle
van de 38 exact geformatteerde pad-regelcitaten gaf exit 0, alle paden/ranges
bestaand; inhoudelijke controle corrigeerde daarna twee verwijzingsranges.
Een aparte Python-witruimtecontrole gaf exit 0, geen trailing whitespace en
wel een afsluitende newline. `git diff --no-index --check /dev/null
docs/audits/subagents/16-claims.md` gaf exit 1 zonder witruimtediagnostiek;
dit is niet als geslaagde producttest gebruikt.

## Niet uitgevoerd, aannames, risico's en prioriteiten

**Niet uitgevoerd:** Rust/Python-producttests, builds, QEMU, actuele CI-opvraging,
artifactdownloads, fysieke hardware, gebruikersproeven, onafhankelijke
menselijke audit, formele evaluatie, volledige releasevalidatie en nieuwe
native regressies op beafdedf. Geen genoemde historische exitstatus geldt als
verse uitvoering door deze audit.

**Aannames:** de lokale Git-objecten/refs identificeren de bedoelde bron; de
werkelijk uitgevoerde historische runs worden niet onafhankelijk gereconstrueerd.
De claimcontrole is begrensd tot hoofddocumenten, relevante acceptatiearchieven
en alle tracked SHA256SUMS onder docs/verification; geen volledige inventaris
van iedere zin, JSON-hashveld, binaire input, changelogimplementatie of de
volledige PROJECT-STATE-historie. De oorspronkelijke runimages/binaries zijn
bewust niet integraal in Git aanwezig.

**Prioriteiten en afhankelijkheden:** eerst C16-01 archiefbeschikbaarheid en
C16-02 manifestprovenance door de bewijseigenaar verklaren, daarna C16-03
historische labels gericht reconciliëren. Dat zijn vervolgvoorstellen, geen
uitgevoerde reparaties. Voor een actuele native claim is vervolgens relevante
gecombineerde bron-/imagegebonden verificatie nodig; voor dagelijks gebruik
blijven de 62 volledige releasevoorwaarden inclusief fysieke en menselijke
poorten leidend. Ontbrekende archiefbytes blokkeren volledige lokale
hercontrole van die specifieke archieven, niet automatisch al het onafhankelijke
bronwerk. Er was geen blocker voor het afronden van deze audit.

**Score en overdracht:** bestaand 41/100, geen nieuwe punten of geaccepteerde
criteria; scorebestanden bewust niet geschreven wegens het exclusieve
auditmandaat. Lever dit rapport vanaf detached bron beafdedf aan de orchestrator;
de SHA-256 wordt na de laatste rapportwijziging buiten het rapport berekend,
zodat geen zelfrefererende hash ontstaat.
