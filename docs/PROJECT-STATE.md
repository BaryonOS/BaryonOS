# Actuele projectstatus

## K05 algemeen proces- en dienstbeheer native geaccepteerd, 30% — 29 oktober 2026

Branch `ccr-43832ef7-gi2qlz`, basis `main` `8b60354`. Omgeving: Linux-container,
Rust 1.90.0, QEMU 8.2.2 (Ubuntu), `pc-q35-8.2`, OVMF 4M.

**Gerealiseerd (native, profiel `native-supervisor`, ADR-042):**
- `capability-core`: objecttypen `Launch` (rechten `LAUNCH`=128) en `Process`
  (`OBSERVE`=256, `KILL`=512), zonder `DERIVE`/`TRANSFER`/`REVOKE`; `orphans()`
  voor kinderen van een gestorven starter; 9 nieuwe hosttests.
- Kernel `kernel/src/m2/m3/supervisor.rs`: syscalls `endpoint_create` (0x30),
  `proc_spawn` (0x50), `proc_kill` (0x51), `proc_status` (0x52); alle controles
  vóór toewijzing; dood via het bestaande `cleanup`-pad; wezen worden iteratief
  in dezelfde gebeurtenis beëindigd. ABI-minor 1.3.
- SDK-runtime: `native::endpoint_create/spawn/kill/status`, `LaunchImage`,
  `ChildProcess`, `ProcessStatus`; drie gewone programma's in
  `sdk/examples/supervisor` (supervisor, herstartbare worker, client).
- Gate `tools/test_supervisor.py`: 4 hostcontroles en 5 echte QEMU-runs
  (normal, crash, denied, parent-exit, quota); runner, capture, 7 harnastests;
  workflow `K05 process management acceptance`.

**Uitgevoerd en geslaagd:** zie [K05-ACCEPTANCE](verification/K05-ACCEPTANCE.md)
en `docs/verification/k05/`. Kernelbewijs per run: normal 3 spawns/2 kills/1
herstart/24 antwoorden; crash 1 echte `#UD`, herstart, 24 antwoorden; denied
17/17 foutcodes, 2 processen; parent-exit 2 wezen beëindigd zonder kills;
quota 15 spawns, 16e `EQUOTA` zonder frameverandering, 16 gelijktijdig, 15
kills. Elke run eindigt met lege engine en framebaseline.

**Roadmap rubric v1: 28% → 30%** (K05 +2).

**Open en niet bewezen:** GitHub Actions op de PR-head moet de gate herhalen;
geen SMP, fysieke hardware, gebruikersgeleverde images (A02) of beleid voor wie
supervisor mag zijn. Tijdens de eerste gate-run is de bronvingerafdruk
veranderd door een gelijktijdige README-wijziging onder `sdk/`; die run is
terecht als mislukt afgewezen en volledig herhaald.

## S06 geaccepteerd na merge en onafhankelijke herhaling, 28% — 29 oktober 2026

Branch `ccr-43832ef7-gi2qlz`, basis `main` `8b60354` (squash van
[PR #49](https://github.com/simonvanieperen/nieuw-os/pull/49)). Omgeving: een
tweede Linux-container met Rust 1.90.0, QEMU 8.2.2 (Ubuntu-pakket), `pc-q35-8.2`,
OVMF 4M. Deze container is niet de omgeving waarin PR #49 is gebouwd.

**Uitgevoerd en geslaagd:**
- `NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B -u tools/test_volume.py` op een
  worktree van PR-head `eadba7e`: exit 0. Hostcontroles 28 volume-core-, 14 SDK-
  en 18 Python-tests; daarna 9 echte QEMU-runs. Bronvingerafdruk `528dfe4c…`,
  gelijk aan `docs/verification/s06/volume-gate.json`; alle 9 `after_sha256`-
  imagehashes zijn byte-identiek aan het vastgelegde PR-bewijs (normal/recover
  `19e2fdb3…`, cut-data `48dcfcdd…`, cut-table `2d37c7c1…`, cut-txn `ede3a54c…`,
  corrupt `a19398f1…`).
- GitHub Actions op `eadba7e`: host-quality, qemu-smoke, k07-native,
  m6e-native-volume, s08-native-resilience en CodeRabbit alle geslaagd; de drie
  CodeRabbit-threads zijn opgelost. Codex-review kon niet draaien (gebruikslimiet).
- Baseline in deze container: `tools/preflight.py --strict`, build
  `milestone-m6c` en `run_m6c.py --case normal --smoke` exit 0. Zonder
  `NOS_QEMU_MACHINE=pc-q35-8.2` weigert QEMU 8.2.2 het standaardprofiel
  `pc-q35-10.0`; dat is de bestaande, vastgelegde ontwikkelkeuze, geen fout.

**Roadmap rubric v1: 26% → 28%** (S06 +2). Dit is de bewijsgrens van
[S06-ACCEPTANCE](verification/S06-ACCEPTANCE.md): opt-in profiel, emulator-kill
is geen stroomuitval, geen authenticatie/encryptie (B02), geen back-up/export
(S09), geen Materiaal-UI.

**Open:** PR [#48](https://github.com/simonvanieperen/nieuw-os/pull/48)
(S08-timing, 0 punten) heeft vijf open CodeRabbit-bevindingen en conflicteert na
deze merge met `main`; de eigenaarsessie is inactief. Volgende implementatiestap
richting 50%: K05 (algemeen proces- en dienstbeheer) als nieuw native profiel.

## S06 volume v2: native gate geslaagd, acceptatie wacht op review en CI, 26% — 29 oktober 2026

Branch `ccr-d3fe1610-gl4r8s`, PR [#49](https://github.com/simonvanieperen/nieuw-os/pull/49).
S06 staat als `71a0f9f` direct op `main` `092ee52` (de squash van #47, M6e).
Het huidige bewijs is van `ce181bf`, met de test- en doccorrecties uit de
tweede review. Uitgevoerd in een Linux-container met Rust 1.90.0, QEMU 8.2.2,
`pc-q35-8.2` en OVMF 4M uit het Ubuntu-pakket.

**Gerealiseerd (native):** volume v2 (`NOSVOL02`, IPC v6) in `services/volume`,
de native dienst en app, het kernelgeval `test-volume-cut-txn`, het v2-orakel,
de runner en de gate. Onderdelen:
- collecties (4 per domein, ook leesbaar met COLLECTION_STAT);
- metadata (`tags`, `kind`, collectie) met stempels uit een klok per domein;
- twee historische versies per object, met RESTORE en verwijderen ongedaan maken;
- PURGE;
- TRANSACT over maximaal 4 objecten in één commit.

**Uitgevoerd en geslaagd (op `ce181bf`, schone kloon):**
- `python3 -B -u tools/test_volume.py`, exit 0. Hostcontroles: 28 volume-core-,
  14 SDK- en 18 Python-tests. Daarna 9 echte QEMU-runs op bronvingerafdruk
  `528dfe4c…`. Guest-uitvoer en image-hashes zijn byte-gelijk aan de vorige run
  op `cb20c27`:
  - normal en koude read-only recovery (domeinklokken 57/27, generatie 84,
    62 chunks);
  - cut-data, cut-table en cut-txn, elk met echte hostkill en koude recovery.
    Bij cut-table zijn 4 en bij cut-txn 5 gescheurde sectoren van g+1
    aangetoond; bij cut-data is de andere kopie onaangeroerd;
  - corrupt geweigerd.
- `python3 -B -u tools/mutate_volume.py`, exit 0. Alle 24 mutanten zijn
  gevangen. Elke mutant schakelt één regel uit die de reviews noemden; het
  rapport staat in `verification/s06/volume-mutants.json`.
- `python3 -B tools/capture_volume.py`, exit 0, vastgelegd als
  `docs/verification/s06/`. Uitslagen:
  [S06-ACCEPTANCE](verification/S06-ACCEPTANCE.md); besluit: ADR-041.
- Een onafhankelijke review van de eerste S06-versie (drie AI-reviewers)
  vond vijf inhoudelijke punten:
  - STAT liet een domein de commits van het andere tellen;
  - collecties waren via IPC alleen schrijfbaar;
  - RESTORE verloor historie;
  - er was geen flush na schrijfbare recovery;
  - de gate kon geen gescheurde tabelwrite aantonen.

  Alle vijf zijn opgelost met regressietests.
- Een tweede review (vijf AI-reviewers; drie stopten door een gebruikslimiet)
  vond geen fout in de productiecode. Wel vond ze tests die zonder hun regel
  slaagden, ontbrekende negatieve tests voor COLLECTION_STAT en twee te ruime
  claims:
  - een read-only antwoord op een `Unknown` commit is voorlopig;
  - `Full` en de instantiebrede `Unknown` zijn zichtbaar over domeinen heen.

  Alles is gecorrigeerd in `ce181bf`. CodeRabbit vond drie kleine punten
  (formulering vingerafdruk, niet vastgelegde mutatiecontrole, U+202E in een
  testfixture); ook die zijn verwerkt.
- Bij de integratie bleek dat een eerdere lokale merge van de M6e-branch in de
  S06-branch M6e-inhoud van `main` had teruggezet. Dat ging om het bewijs, de
  voortgangsmetadata en de subpackage-binding uit `d9e4b6b`. Die inhoud is
  hersteld; alle andere M6e-commits zijn per commit gecontroleerd.

**Roadmap rubric v1: blijft 26%.** S06 (+2) wordt pas geaccepteerd als drie
dingen afgerond zijn:
- de derde reviewronde (RESTORE/flush, gate-bewijs, merge/docs en een
  controle van `ce181bf`), die nu loopt;
- GitHub Actions op de PR-head;
- de herbeoordeling door CodeRabbit.

**Open en niet bewezen:**
- Er zijn geen fysieke schijven, encryptie of authenticatie (B02).
- De Materiaal-interface draait niet native; S06 levert alleen de opslaglaag.
- Native I/O-foutinjectie ontbreekt. De flushfout na recovery en het
  `Unknown`-pad zijn alleen hostgetest.
- S08: de CI-job `s08-native-resilience` faalde op `c87f980` één keer in het
  bestaande geval `flush-error` (`USER_OR_PROTOCOL`) en slaagde bij de herhaling.
  Die commit veranderde alleen documentatie. De oorzaak is nog onbekend: de
  onderzoeksagent is door een onderbreking gestopt zonder rapport.
- S09 (back-up/herstel) staat in een aparte lokale branch op S06. De native
  S09-gate (8 QEMU-runs) is daar geslaagd, en de S06-gate als regressie ook
  (9/9, bron `06d46151…`). S09 krijgt een eigen PR na de merge van #49.

## M6e schaalbare documentopslag native geaccepteerd, 26% — 28 oktober 2026

Branch `ccr-d3fe1610-gl4r8s`, broncommit `d9e4b6b` (bevat `main` `78a96ae`), concept-PR
[#47](https://github.com/simonvanieperen/nieuw-os/pull/47). Uitgevoerd in een
Linux-container met Rust 1.90.0, QEMU 8.2.2, `pc-q35-8.2` en OVMF 4M uit het
Ubuntu-pakket.

**Gerealiseerd (native):**
- `services/volume` (`volume-core`, `no_std`, `forbid(unsafe_code)`): een
  copy-on-write-volume met twee per sector verzegelde tabelkopieën en
  4 KiB-chunks.
- SDK-blok-API en een ring-3 `volume-service` met twee geïsoleerde app-instanties
  (`sdk/examples/volume-docs`).
- Generieke kernelbroker `native-volume` zonder formaatkennis.
- Onafhankelijk Python-orakel (`tools/volume_disk.py`), runner met echte VM-kills,
  en de gate `tools/test_volume.py` met capture.

**Uitgevoerd en geslaagd:**
- `python3 -B -u tools/test_volume.py`, exit 0, met 3 hostcontroles (17 volume-core-,
  14 SDK- en 8 orakeltests) en 7 QEMU-runs op bron `14fa4d37…`.
- Reviewcorrectie (CodeRabbit, PR #47): iedere tabelsector draagt nu een
  pogingdigest, zodat sectoren van twee schrijfpogingen voor dezelfde generatie
  nooit samen een geldige kopie vormen. De nieuwe hosttest faalt zonder de
  correctie; het oude bewijs van bron `535d672e…` is vervangen.
- Tweede reviewronde: de gate schrijft zijn verdict pas na de afsluitende build
  (een mislukte build liet eerder `passed: true` achter) en ADR-040 noemt zijn
  eigenaar. Derde ronde: het verdict is nu ook gebonden aan de SHA-256 van de
  hosttestinputs onder `tests/`; vierde ronde: ook in subpackages, zoals
  `unittest discover` ze laadt. Gate herhaald en bewijs opnieuw vastgelegd.
- `python3 -B tools/capture_volume.py`, exit 0, naar `docs/verification/m6e/`.
- Regressie: `milestone-m6c` normal smoke.
- Uitslagen en grenzen: [M6E-ACCEPTANCE](verification/M6E-ACCEPTANCE.md).
  Contract: [M6E-SCALABLE-VOLUME](implementation/M6E-SCALABLE-VOLUME.md).
  Besluit: ADR-040.

**Roadmap rubric v1: 20% → 26%** (S04 +3, S05 +3).

**Open en niet bewezen:**
- GitHub Actions slaagden op `1037ee3`, `f9e22c1` en `bf0bf61` (5/5, inclusief
  `m6e-native-volume`); de herhaling op de eindhead moet nog slagen.
- Geen native I/O-foutinjectie voor dit profiel; I/O-fouten zijn alleen hostgetest.
- Geen fysieke schijf, encryptie of authenticatie (B02), back-up (S09),
  collecties/historie (S06) of migratie van M6c-volumes.

**Ook in deze opdracht:**
- PR #46 (N01-encoder) is na groene CI en een CodeRabbit-review zonder bevindingen
  squash-gemerged (`78a96ae`), zonder punt. De Codex-review kon niet draaien door
  het gebruikslimiet.
- Drie Claude-subagents zijn bevestigd gestart in eigen worktrees, met lokale
  QEMU: G1 (ring-3 display/invoer), K05 (procesbeheer) en N01 (virtio-net/UDP).
  G1 en K05-a (ring-3-supervisor met niet-overdraagbare procescapabilities,
  3 native gevallen volgens het agentrapport) zijn opgeleverd maar nog niet
  geïntegreerd, onafhankelijk herhaald of geaccepteerd; N01 loopt nog.
- S06 (collecties, metadata, revisiegeschiedenis, transacties) is in een aparte
  lokale worktree in ontwikkeling: hosttests groen, native gate nog niet gedraaid.

## N01 voorbereidend netwerkwerk — UDP-uitvoer zonder native NIC (28 oktober 2026)

Ontwikkelbranch `feature/n01-bounded-udp-ipv4-tx-20261028`, begonnen op main `6e12dcbbd4a37d404dd5a60bc2c8a8e32d2d187e`.
De bestaande `no_std`-protocolmodule is uitgebreid met een begrensde
Ethernet II/IPv4/UDP-transmitter. Deze code genereert uitsluitend bytes in een
caller-buffer: expliciete effectieve MTU, TTL-controle, IPv4/UDP-checksums,
zero-checksumcodering volgens RFC 768, zero padding en geen gedeeltelijke
bufferschrijfactie bij geweigerde input. De ingestelde source/destination
blijven data, geen gevalideerde app- of peerrechten. De bestaande parser wordt
als roundtrip-validator gebruikt; vaste onafhankelijk afgeleide UDP-vectoren
blijven het orakel, ook voor foutgevallen.

**Bewijsstatus:** broncode en aanvullende Rust-hosttests staan in deze branch.
De testcommando's `cargo +1.90.0 test/clippy/fmt` en cross-build naar
`x86_64-unknown-none` zijn in deze chatsessie **niet lokaal uitgevoerd**:
Rust en QEMU ontbreken in de beschikbare lokale omgeving. PR-CI kan dit
op een gecontroleerde runner uitvoeren; de uitkomst daarvan moet nog op de
exacte PR-head worden vastgelegd. Geen native NIC-/packettransmissie,
TCP-stack, service-, hardware- of QEMU-acceptatie geclaimd.

De zes aparte Codex-taken voor K05/S04/N01/U01/B03/H02 zijn op 28 oktober
wel aangemaakt, maar direct zonder code afgerond met
`No usable execution environment is available`. Dit is geen lopende
parallelle agentuitvoering. De hoofdchat bouwt onafhankelijk verder via
GitHub-bestanden. **Rubric v1 blijft 20/100 (20% → 20%); N01 is open.**

## Native A01 en S08 geïntegreerd, 20% — 28 oktober 2026

Op `main` commit `b9c7cf8f4e1804f0333e8fba8862941190224155` zijn twee
afzonderlijke functies nieuw opgenomen. [PR #31](https://github.com/simonvanieperen/nieuw-os/pull/31)
voegt de begrensde gewone native SDK-app en gebruikersdienst toe; een echte
QEMU-acceptatiecontrole toetst normale en geweigerde rechten, foutafhandeling,
proceslevenscyclus en resourcecleanup. [PR #34](https://github.com/simonvanieperen/nieuw-os/pull/34)
voegt de S08-resiliencegate toe met dertien aparte QEMU-runs, waaronder zes
koud gestarte recoveries van exact dezelfde bewaarde image. Read-only is
hardware-emulator-, driver-, capability- en userspace-afgedwongen, niet slechts
een gelijke hash voor en na de proef.

De acceptatiebewijzen staan in
[A01-ACCEPTANCE](verification/A01-ACCEPTANCE.md) en
[S08-ACCEPTANCE](verification/S08-ACCEPTANCE.md). Voor A01 en S08
zijn de gespecialiseerde gates groen; [main-CI 40761772907](https://github.com/simonvanieperen/nieuw-os/actions/runs/40761772907)
is eveneens volledig groen. De oorspronkelijke CodeRabbit-bevindingen zijn
gecorrigeerd; oude formele reviews zijn pas na oplossing en nieuwe groene
tests met onderbouwing gesloten. Codex-cloudagents konden niet starten door
het huidige gebruikslimiet, dus er is geen parallelle agentuitvoering geclaimd.

Roadmapacceptatie volgens bestaande rubric v1: **17% → 20%** (A01 +2, S08 +1).
Alle overige criteria blijven open. Zonder fysiek hardwarebewijs, native GUI,
breed app-ecosysteem, netwerkintegratie en updates is geen 50% geclaimd.

## Volledige repository-audit en integratie — 28 oktober 2026

De volledige huidige repository is opnieuw gecontroleerd op architectuurgrenzen,
duplicatie, build-/testconfiguratie, dependencies, gegenereerde rommel, secrets,
documentatie en open pull requests. De relevante inhoud van PR #16 is via
[PR #25](https://github.com/simonvanieperen/nieuw-os/pull/25) geïntegreerd en
PR #16 is daarna als vervangen gesloten. Een later verschenen, verouderde
documentatie-closeout PR #27 is eveneens inhoudelijk vervangen in plaats van
blind gemergd.

De integratie behoudt de eigen Rust-kernel en de bestaande microkernelachtige
scheiding: de Ethernet/IPv4/UDP-parser is een dependencyvrije
`no_std`-userspace/servicebibliotheek zonder driver-, socket- of capabilityrecht.
De DocumentSession-proef gebruikt bestaande catalogus-/IPC-capabilities en
verruimt kernel-, IPC- of opslagformaten niet. Bewuste duplicatie voor afzonderlijke
binaries/security-isolatie is niet samengevoegd.

GitHub Actions bevat nu een read-only CI met gepinde officiële actions. Na de
auditmerge werd op exact dezelfde Git-tree één niet-deterministische normale
M6c-QEMU-fout waargenomen: de client ontving een geldige reply, maar de mutatie
bereikte de bestaande `Unknown`-route doordat de completion niet binnen de
20-tick brokerdeadline werd waargenomen. De onmiddellijke rerun op dezelfde tree
slaagde. [PR #26](https://github.com/simonvanieperen/nieuw-os/pull/26) verhoogt
uitsluitend de M6c-acceptatiebrokerdeadline naar 64 ticks; faultfixtures blijven
completionwaarneming tot de echte deadline onderdrukken, zodat timeout/Unknown
niet wordt weggefilterd of geretryd.

De definitieve push-run op `main`,
[run 40741321766](https://github.com/simonvanieperen/nieuw-os/actions/runs/40741321766),
is volledig groen. `host-quality` slaagt voor Python/DocumentSession, root Rust,
Clippy/rustfmt, SDK, netwerkparser inclusief native cross-build, quantum,
productprototype, NAS-tests, native releasebuild en voortgangs-/clean-treecontrole.
`qemu-smoke` slaagt voor strict preflight, baseline M6c en DocumentSession
normal plus koude recovery in afzonderlijke VM-invocaties.

CodeRabbit leverde op PR #16 vijf concrete reviewbevindingen; de geldige punten
zijn in de integratie hersteld. Op PR #26 was de CodeRabbit-review bij merge nog
asynchroon in uitvoering en waren geen inhoudelijke bevindingen of reviewthreads
gepubliceerd; de volledige CI was wel groen. Codex code/security review op de
verouderde PR #27 kon niet starten doordat het gekoppelde reviewquotum bereikt
was; PR #27 is daarom niet als goedgekeurd beschouwd en juist vervangen.

Repositoryonderhoud is aangescherpt met `CONTRIBUTING.md`, PR-/bugtemplates,
beperkte maandelijkse Dependabot-voorstellen, binary-attributen voor PDF/GZIP,
en gitignore-regels voor gegenereerde SDK-/netwerk-targets. De bestaande
main-ruleset beschermt verwijdering, non-fast-forward en lineaire geschiedenis
en vereist PR's plus opgeloste reviewthreads. Verplichte CI-statuschecks en
GitHub-securityinstellingen konden via de beschikbare integratie niet worden
gewijzigd of volledig uitgelezen; dit blijft een repository-instellingspunt.

**Roadmap: 14% → 14%.** De audit maakt bestaande functionaliteit aantoonbaar
schoner en beter bewaakt, maar DocumentSession is nog een begrensde headless
proef, de netwerkparser is nog geen native netwerkdienst en de Security Floor/
provider-/premium-UX-eisen zijn nog geen volledige native acceptatie. Daarom is
geen nieuw vast criterium toegekend.

## Architectuurbesluit: Security Floor, opaque provideropslag en premium UX — 28 oktober 2026

De nieuwe gebruikerskeuze is als documentatie-/architectuurwijziging vastgelegd.
Privacy is configureerbaar, maar de **Security Floor** niet: internet,
vrijwillige telemetrie, microfoon, externe opslag of andere expliciete
gegevensdeling verruimt alleen de betreffende capability/gegevensstroom en
schakelt kernel-/procesisolatie, sleutelisolatie, geheugenbescherming,
pakket-/update-authenticiteit of bescherming van niet-gedeelde data niet uit.

Het absolute leverancierscloudverbod uit privacycontract revisie 3 is vervangen
door revisie 4. Persoonlijke NAS blijft een eersteklas route. Daarnaast mag een
gebruiker een algemene externe opslagprovider expliciet kiezen als
**onbetrouwbare opaque ciphertextbackend**. De provider krijgt geen
Nieuw-OS-inhoudssleutel, documentplaintext of gevoelige manifestplaintext.
Geen stille providerfallback of key escrow. Metadata, beschikbaarheid,
providerdelete en endpointcompromittering blijven expliciete rest-risico's.

[ENCRYPTED-OBJECT-STORAGE.md](implementation/ENCRYPTED-OBJECT-STORAGE.md)
legt het toekomstige objectformaat, key hierarchy, crypto-agility,
rollback-/replay-eisen, multi-providergrens en EO-acceptatietests vast. Er is
geen zelfbedachte cryptografische primitief gekozen en geen claim dat een
provider “absoluut niet kan kraken”. P05/EO-tests zijn nog open.

Ook de productambitie is aangescherpt: P3/P4/M7 moet een eigen premium
productiviteitsfamilie opleveren met Materiaal als objectmodel, één designsysteem,
contextuele gereedschappen, hoogwaardige typografie/motion, toekomstige
gedeelde native renderlaag en geteste interoperabiliteit. Geen Office-ribbon-
of bestaande-OS-kopie als einddoel. Het huidige browserprototype blijft alleen
ontwerpgereedschap.

Privacyuitleg en capabilities zijn gescheiden. Educatieve privacyuitleg mag
globaal of per app uit; dat wijzigt geen grants. In focus/gaming worden
niet-urgente educatieve meldingen onderdrukt, terwijl een nieuwe gevoelige
functie nog steeds een noodzakelijke grant kan vereisen.

Vastgelegd in ADR-035 t/m ADR-038. Er is in deze stap **geen native runtime-,
crypto-, provider- of UI-implementatie uitgevoerd en geen nieuw
acceptatiecriterium behaald**. De vaste roadmap blijft **14% → 14%**.


## Markt- en consumentenonderzoek — 28 oktober 2026

Twee aanvullende onderzoeken zijn als strategische onderbouwing vertaald naar
`docs/research/`: een markt-/concurrentieanalyse en een wetenschappelijke
consumentenadoptieverkenning. Nieuw is vooral de externe onderbouwing van
overstapfrictie, compatibiliteit, herstel, ondersteuning, Europese alternatieven
en 90-/180-dagenbehoud als latere marktvalidatie. Materiaal, de eigen kernel,
lokale AI, beperkte apprechten en de privacyrichting waren al projectbesluiten.

De synthese maakt expliciet dat doelgroep, kleurpaletten, prestatiedoelen,
steekproefgroottes, retentiedrempels, verkoopkanalen en verdienmodellen nog
onderzoekshypothesen zijn. Zij wijzigen `PRODUCT-SPEC.md`, capabilities,
kernel-ABI of privacycontract niet zonder afzonderlijk besluit.

De inmiddels op `main` opgenomen privacybrowser-aanvulling (#18) blijft
volledig behouden en is een aangrenzend productonderzoek, geen conflict met
deze markt-/adoptiesynthese.

Aanvullend is [CONSUMER-VALUE-2026-10-28.md](research/CONSUMER-VALUE-2026-10-28.md)
opgenomen als diepere productvertaling. Die legt de te onderzoeken kernbelofte
“materiaal terugvinden, ermee verdergaan en controle houden over wijzigingen en
toegang” vast en definieert DV-01 als latere geïntegreerde Materiaal-toets met
twee gereedschappen, bevestigd opslaan, herstel en revocation. P2 blijft
bevroren; DV-01 is geen reeds bewezen product- of native eigenschap.

De consumentenwaarde-verkenning bevat nu ook de ontbrekende bewijsgrenzen rond
ervaren competentie, desktopgewenning, tijdelijke opslag, local-first,
regelbare meldingen en ervaren werklast. PRX1–PRX4 vertalen dit naar vroege
taakobservatie, een DV-01-prototypeproef, boodschaptest en vierweekse veldproef.
Er zijn nog geen deelnemers onderzocht; de 90-/180-dagenmarktvalidatie blijft
een afzonderlijke latere stap.

Geen broncode, native runtime of acceptatiebewijs is in deze documentatiestap
gewijzigd of uitgevoerd. De technische momentopname in de oorspronkelijke
rapporten kan daarom ouder zijn dan deze actuele projectstatus. Roadmap:
**14% → 14%**.

## Actuele vervolgacceptatie — 28 oktober 2026

**Zes extra echte VM-runs slagen op `dbd35b04d742b18d66b40fdd65d8a9b900656cac`**,
bronfingerprint v3 `c5f3d9316433ba09ba4b886985ffa8b28643d67c72a9ebb0e2b0136362dc7a9f`.
G0 draait met vijf echte invoerbeelden en nieuwe atomische capture; SDK/M6c
en M6b bewaren en lezen terug in elk twee afzonderlijke VM's; de qsim-proef
draait in twee geïsoleerde gebruikersprocessen. Alle zes ruwe exits zijn 33.
Eigen builds, targets, volumes, QMP-pipes en logs voorkomen gedeelde output.
De [onafhankelijke gecombineerde controle](verification/parallel-followup-integration/README.md)
herhasht 216 unieke bestanden en bevestigt alle zes bron- en artifactketens.

- [G0: bron, opdrachten, vijf echte beelden en capture](verification/parallel-followup-g0/README.md).
- [SDK/catalogus: normale opslag en koude herstart](verification/parallel-followup-sdk-m6c/README.md).
- [Quantum: actuele strict run en vergelijking van werkelijke binaries](verification/parallel-followup-qsim/README.md).
- [M6b en gerichte hostgrenzen](verification/parallel-followup-root/README.md).

Op `f52ef0a6` zijn de G07-documentsessie, I08-jobcodec en B07-capture al
gepubliceerd. De documentsessie bewaart geladen revisie en lokaal concept bij
CAS-conflicten of onzekere opslag. Deze appbibliotheek staat buiten de kernel.
De 39 SDK-hosttests en native librarybuilds slagen; I10 vond geen blocker
binnen haar reviewscope. De canonieke 256-byte jobcodec heeft 13 tests en een
gerichte onafhankelijke review; dispatch en gebruikersjobs blijven open.

G09 vond een dynamische outputlijst in de capture. B09 gebruikt een vaste
lijst vóór indexering én vóór publicatie. De negatieve test faalde op de oude
bron en slaagt na reparatie; onafhankelijke herhaling en alle 34 grafische
hosttestmethoden slagen. De verse B08-VM gebruikt de gerepareerde capture;
alle originele inputs en lossless PNG's zijn opnieuw gecontroleerd.
[Documentreview](verification/parallel-i10/README.md),
[capturereparatie](verification/parallel-b09/README.md),
[hercontrole](verification/parallel-g09/README.md).

De [gerichte PR14-behoudcontrole](verification/parallel-b10/README.md)
bevestigt dat de zeven eerdere reviewvragen in de integratiebron zijn opgelost
en dat alle bestaande bestandspaden behouden zijn. De volledige bomen zijn
vergeleken omdat de GitHub-compare slechts een deel van de paden retourneerde.

De acht concrete CodeRabbit-bevindingen zijn afgehandeld met bron-/bewijslinks;
de actuele vervolgbeoordeling en PR-status staan op GitHub. De eerdere
volledige acceptatie van 80 VM's hieronder behoudt haar oorspronkelijke bron.
Bestaande kernel-, dienst- en probebron bleef tijdens deze vervolgstap gelijk.
De nieuwe qsim-kernel en app zijn byte-identiek; twaalf gewijzigde loaderbytes
zijn verklaard als PE-tijdstempels/buildidentifier, met originele artifacts.

G11 bouwt nu de documentsessie in de bestaande headless gebruikersproef om
ook openen, bewerken, bewaren en heropenen via deze bibliotheek native te
controleren. Die uitvoering is nog niet geaccepteerd. G1/Materiaal blijft bij
de bestaande eigenaar; publicatie van diens broncheckpoint is de kleinste
integratieafhankelijkheid. Geen grafische documentflow of algemene appstarter
wordt uit de huidige proeven afgeleid. Roadmap: **14% → 14%**.

## Gecombineerde native acceptatie — 28 oktober 2026

Uitvoeringsbron `3e41bbf93822f7044c6020259f4a752925e1966d`, branch
`codex/parallel-native-foundation-20261028`, PR#15 boven de behouden PR#14.
Lokale/GitHub-commit identiek vóór gates; bronfingerprint v3
`307c828e9ce7bf49a4e809859ca77ecd332015572dd37e96fb580de98951aa60` bleef gelijk.
Alle vier werkelijke agentplaatsen zijn benut en hergebruikt met eigen
worktrees/builds/targets/VMs/logs. G1 en de andere M6a-opdracht blijven bij hun
bestaande eigenaars.

**80 geaccepteerde native VM-runs:** M6b32 groepen/28 VM, SDK/M6c35/31,
M3-regressie22/19, native quantum1 en bevestigde G0-pipe1. Volledige M6b- en
M6c-captures slagen. Beide normale SDK-catalogusruns bewaren sequence10 en
lezen na een nieuwe koude VM dezelfde bytes terug zonder writes. Twee quantum-
userprocessen voeren bestaande referentieberekeningen uit, met64 KiB-arena's,
4 timerIRQs/505 syscalls en96 werkelijk vrijgemaakte frames. G0 verwerkt echte
invoer met per-toetsbeelden en exact backspaceherstel. Geen nieuwe syscall,
capabilityverruiming, on-diskformaat of externecrate.

De gecombineerde artifactcontrole bevestigt80 unieke VMs en616 directe
hashvergelijkingen. De oorspronkelijke G0-run is uitgesloten wegens een
later afwijkend bewaard serialartifact; oorzaak onbekend, niets hersteld.
De verse private pipe-run en alle 15artifacts zijn herhaaldelijk gecontroleerd.
Een aparte UNIX-socketprobe faalde op sandbox-EPERM; de bruikbare pipe-route
is de native geverifieerde variant. Oude fouten en ruwe resultaten blijven
behouden. Alle bron-/bewijs-/reproduceerdetails staan in
[PARALLEL-NATIVE-ACCEPTANCE.md](verification/PARALLEL-NATIVE-ACCEPTANCE.md).

Concrete CodeRabbit- en gerichte AI-bevindingen zijn verwerkt voor QMP-
deadlines/ID's, screenshots, ROMpin, WRITE-timeout-oracle, standalone preflight
en prefixed FPU-instructies. De nieuwe PR#15-review op3e41 is aangevraagd;
definitieve review/mergestatus wordt op GitHub bijgehouden. Geen veiligheids-
of fysieke hardwareclaim door de AI-review. QEMU 7.2.22/pc-q35-7.2 is expliciet
vastgelegd en verschilt van het historische10.0-referentieprofiel.

Voortgang **14% → 14%**; vaste criteria/gewichten behouden. A01 algemene appstart,
U03 native Materiaal, S04/S05 bruikbare documentgrootte/ruimtehergebruik en L02
externe native quantumjobs blijven open. De catalogusclient+SDK bestaan nu;
G07 levert een documentsessie, I08 een jobcodec en B07 een strikte G0-capture.
Deze geïntegreerde bouwblokken zijn nog geen volledige productfunctie.
Zie `WORK-QUEUE.md` voor de kleine actuele taakverdeling en blokkades.

### Directe nieuwe native bewijslinks

- G0: [commando, raw33 en alle controles](verification/parallel-final-g0-confirmed/pipe/result.json), [ruwe serial](verification/parallel-final-g0-confirmed/pipe/serial.txt).
- SDK/catalogus: [opdrachten en exitstatussen](verification/parallel-final-sdk-m6c/driver.json), [alle31 ruwe VM-records](verification/parallel-final-sdk-m6c/guest-runs.json).
- Quantum: [opdrachten en bronbinding](verification/parallel-final-qsim/i05-run-checks.json), [native raw33-resultaat](verification/parallel-final-qsim/result.json), [ruwe serial](verification/parallel-final-qsim/serial.txt).

Alle drie nieuwe bundels registreren werkelijk QEMU7.2.22/pc-q35-7.2.
Historische M6c-consolelogs van andere QEMU-profielen zijn hiervoor niet gebruikt.

## Historische faseverslagen

De verslagen hieronder bewaren hun toenmalige bron- en bewijsgrenzen.

## Doorbouw — opslagtimeouts en native grafische basis

Integratiecheckpoint: `main` op `391f02e` (M6C/D, PR #13) is via normale merge
opgenomen. Alle catalogus-/M6d-features, `archive_destination` met direct-child-
grens en Windows `wait(timeout=10)` blijven behouden. De andere chat beheert
haar afzonderlijke M6a-observerbranch; die bron is hier niet overschreven.
Oude captures houden hun oorspronkelijke bronbinding. Gecombineerde eindgates
zijn op dit checkpoint nog niet uitgevoerd. Voortgangsmeter: **14% → 14%**;
de lopende G0- en timeoutstappen sluiten geen nieuw volledig criterium.

28 oktober 2026, vanaf `7b4945f` op `codex/m6b-storage-errors`. De OS-bouw loopt
door; het vorige checkpoint is geen voltooid OS. K08 voegt kerneldeadlines,
resetquarantaine en lokaal afgebroken polls toe. De dienst sluit bij eigen
polluitputting zijn enige blokcapability via de bestaande ABI; de kernel kan
zo echte autoriteitsbeëindiging waarnemen en pending DMA veilig annuleren.
Vijf gerichte echte VM-proeven op bron `9c0c434` slagen met raw-exit 33:
WRITE-/FLUSH-deadline, gemaskeerde resetack/quarantaine en lokaal uitgeputte
WRITE-/FLUSH-polls. De cancelproeven sluiten de blokcap vóór de kerneldeadline;
de resetproef houdt de pool vast. Het harnas heeft 33 geslaagde hosttests.
Afzonderlijk bewijs: `verification/m6b-timeouts-directed/`. De uitgebreide
volledige 32-groepen-/28-VM-poort volgt na de native grafische bronfreeze.

G09 bouwt parallel een opt-in native grafische basis volgens ADR-033 en
`implementation/GRAPHICS-G0.md`: eigen framebufferuitvoer en PS/2-invoer,
geen browser, volledige desktop of opslagadapter. De pure `ui-core` is aanwezig
met tien geslaagde hosttests; de native GOP-/kernelintegratie staat klaar voor
de eerste hardwareproef. Elf bootprotocoltests, drie grafische harnastests,
fmt/Clippy/no_std slagen. Standaard BootInfo 1.1 blijft behouden. Deze
De eerste native G0-run op `f2c4055` slaagt: raw 33, eigen post-EBS/M2-uitvoer,
800×600-framebuffer, echte QMP-toetsen a/b/backspace/c en aantoonbaar alleen
gewijzigde pixels in de invoerregel. Backspace herstelt exact het eerdere beeld.
Dit bewijst RAM-invoer en grafische bootstrap, geen compositor of opslagUI.
Het harnas wordt nog aangescherpt met exacte marker-/geometriecontrole;
nieuwe gecombineerde eindgates volgen. Bewijs: `verification/graphics-g0-first/`.
De eerste G0-start faalde vóór kernelboot door een
ontbrekende VGA-ROM-lookup. De reeds geïnstalleerde officiële SeaBIOS-ROM is nu
uitsluitend in de projectlokale QEMU-datamap gelinkt; de mislukte run blijft
failed bewaard.

## Hervatting — native M6b-opslagfouten, 28 oktober 2026

Basis `1f24f994bff1b2c7cb0a026b600c5c4948140cfe`, branch
`codex/m6b-storage-errors`. De lokale checkout liep achter op GitHub; na fetch
is deze actuele M6b-basis gekozen. De voorlopige, dubbele M6a-planstap op
`codex/m6a-write-flush-core` is niet geïntegreerd en is geen vervolg op M6b.

Afgeronde stap op uitvoeringsbron `592dccfb329c05107866b18f7187e8b340ccab98`:
de observerpanic bij een terminale WRITE-/FLUSH-fout is gerepareerd. De observer
volgt de echte brokerfout naar de ring-3-dienst, verwacht een canoniek Unknown-
antwoord en bevestigt/commit de mutatie niet. Latere GET/PUT/DELETE en dezelfde
PUT opnieuw geven Unknown zonder nieuwe I/O; verkeerde domeintoegang blijft
Access. Geen nieuwe rechten, syscall, dependency, unsafe of schijfformaat.
De foutinjectie
vervangt uitsluitend een lokale statussnapshot ná echte devicecompletion;
geen claim van een fysieke schijffout of stroomuitval. Eisen en markers staan
in `implementation/M6B-OBJECT-STORAGE.md`.

`python3 -B -u tools/test_m6b.py` en
`python3 -B tools/capture_m6b.py --destination docs/verification/m6b-storage-errors`
slagen (exit 0): **22 groepen / 18 echte VM-runs**. Daarin zitten beide nieuwe
foutcases met negen werkelijk ontvangen Unknown-replies, nul mutatie-ACKs,
blijvend sequence 0 in de levende dienst en precies één werkelijk geschreven
record op schijf. Beide koude vervolgruns herstellen sequence 1 zonder writes.
De overige CRUD-/domein-/full-/corrupt-/kill-/recoverycases en M1-regressiestart
slagen eveneens. Hostlaag: 22 Rust-objecttests en 31 harnastests. De gate telt
acht koude recoveries en twee echte externe QEMU-kills.

Vooraf slagen twee gerichte foutVMs; zij worden niet bij de gate opgeteld.
Targeted rustfmt/diffcontrole slagen. Onafhankelijke AI-bronreview heeft geen
blockers; extra negatieve markerproeven slagen. Checkpoint:
`verification/m6b-storage-errors-checkpoint/`. Volledige ruwe gate/capture:
`verification/m6b-storage-errors/`. Exacte aanvullende commando's, review,
hashaudit en normale eindstart: `verification/m6b-errors-review/`.
De verifier controleerde alle 18 gastrecords, 157 artifacthashes, 36 ruwe
serial-/stderrbestanden en acht koude herstelkoppelingen; geen blockers.
De normale `milestone-m6b`-build en een extra smoke-start slagen (exit 0/raw 33);
kernel en loader zijn bytegelijk aan de normale eindgate. Dit is AI-verificatie,
geen menselijke audit. Concept-PR: [#14](https://github.com/simonvanieperen/nieuw-os/pull/14).

Ontwikkelomgeving: projectlokale Rust 1.90.0, QEMU 10.0.13 en OVMF
2025.02-8+deb13u1 uit officiële Debian-pakketten; geen systeembrede installatie.
Dit is een andere ontwikkelvariant dan het historische Linuxbewijs hieronder.

Grenzen: geen nieuwe volledige M1–M6a-regressiesuite, browser-/NAS-/quantumproef
of fysieke-hardwaretest. Echte backend-IOERR, gedeeltelijke write, timeout,
reset-timeout en uitputting van de eigen poll-LIMIT van de dienst zijn niet
nieuw native geïnjecteerd. Die laatste lokale timeout kan nog van de verwachte
observerstatus afwijken als geen terminale brokerfout is teruggegeven. De
snapshotproeven sluiten dat afzonderlijke randgeval niet. Volgende opslagstap:
deze fout-/timeoutgrenzen, daarna ruimtehergebruik en de verdere productadapter;
native netwerk/desktop en updateketen blijven afzonderlijke bouwtaken.
## Reviewgrenzen en historische auditbron — 28 oktober 2026

Basis `322c553f58f11843aab7cf895bcba05564a9c7ac`, branch
`codex/integrate-m6d-main`. De parallel gepubliceerde recorderreparatie en vier
regressietests zijn behouden; dubbel lokaal herstelwerk is apart bewaard.
De documentatie begrenst receiptbehoud nu expliciet tot beëindigde childprocessen:
de bestaande timeout werpt vóór registratie `TimeoutExpired`.

De oorspronkelijke A04-auditor is toegevoegd als `a04_m6b_audit.original.py`,
exact gebonden aan commit `7484d87`, zijn 22802 bytes en de hash van het oude
auditrecord. De zes historische methoderegels blijven onderscheiden van het
latere script met zeven regels; geen volledige rawaudit is herhaald.
M6D-CLOSEOUT beschrijft de huidige R09-bestandsnaam en de vereiste canonieke
oorspronkelijke uitvoeringsroot. De M6c-queue heeft een expliciet historische
subkop. Ongewijzigde uitvoeringsrecords behouden hun eigen hashes.

Exacte uitgevoerde host-/tekstcontroles, exitstatussen en bytevergelijkingen:
[review-provenance.json](verification/m6d-closeout/review-provenance.json).
Geen nieuwe build of VM-run. Native bronbinding en de vaste score blijven
**14% → 14%**. Het aanvullende native scenario dat een normale pending-poll
en het latere clientresultaat samen observeert, blijft open; de bestaande
terminale foutproeven vervangen die gerichte observatie niet.
De uiteindelijke review- en squashmergestatus staat in PR #13.

## Volledige PR-review: recorder en documentatie — 28 oktober 2026

Broncheckpoint `7113f4ddcf3af34918f89f3869aa3889694d47b2`, branch
`codex/integrate-m6d-main`. Drie aanvullende CodeRabbit-bevindingen uit de
volledige review op `4c8ae3c` zijn gericht verwerkt. De ontwikkelhandleiding
kiest voor iedere M6c/M6d- en M6b-capture een nieuwe expliciete UUID-bestemming;
de roadmap vermeldt de bestaande M6d-acceptatie van 35 groepen / 31 VM,
M6b-regressie 18 / 14 en S07, met behoud van de bewijsgrenzen.

De historische capture-recorder schreef fouten en gewijzigde inputs wel in
zijn receipt, maar eindigde daarna ten onrechte met status 0. De recorder
bewaart nu eerst receipt en uitvoer en retourneert daarna status 1 bij
een beëindigd subproces met niet-nul exitstatus of gewijzigde inputs. Succes met
ongewijzigde inputs houdt status 0. De grens van 60 seconden blijft gelden:
bij een timeout werpt `subprocess.run` eerst `TimeoutExpired`, voordat deze
recorder een receipt of uitvoerbestanden schrijft. De gerichte proeven hieronder
testen beëindigde subprocessen, geen timeoutregistratie.

Werkelijk uitgevoerd: twee gerichte hosttestinvocaties met
`python3 -B -m unittest discover -s tests -p test_capture_check_recorder.py -v`.
De identieke vier scenario's controleren succes, een falend subproces,
gewijzigde input en beide fouten samen. Vóór herstel slaagt één scenario en
falen drie op de onterechte recorderstatus 0 (suite-exit 1); na herstel slagen
**4/4** (suite-exit 0). Receiptvelden, inputhashes en ruwe output blijven ook
bij afwijzing controleerbaar. Exacte executable, opdrachten, cwd, bronhashes,
uitvoer en exitstatussen staan in
[capture-check-regressions.json](verification/m6c-review/integration-followup/capture-check-regressions.json).

De oorspronkelijke 29-testsuite, provenance en drie commandorecords blijven
bytegelijk en aan de historische recorder gebonden. De actuele recorder en
nieuwe regressiecontrole hebben afzonderlijke hashes in het bijbehorende
manifest. Deze vier tests worden niet bij de historische 29 of Windows-173
opgeteld. De twee documentwijzigingen zijn tegen de actuele captureargumenten,
links en opgeslagen aantallen gecontroleerd; Bash-syntax en UUID-fragmenten
zijn lokaal gecontroleerd, PowerShell alleen statisch.

De 144 native/build/runnerinputs en het gepubliceerde Windowsbewijs blijven
ongewijzigd. Geen nieuwe native build, VM-uitvoering of extra productpunt.
Voortgang **14% → 14%**; criteria en gewichten blijven gelijk.
`python3 tools/progress.py --write` en daarna `--check` zijn uitgevoerd met
exit 0. De definitieve externe review- en squashmergestatus staat in
[PR #13](https://github.com/simonvanieperen/nieuw-os/pull/13).

## PR-integratie en qsim-herstel — 28 oktober 2026

Broncheckpoint `8d756ed4b85585e9878f440ea537c7875b02da45`, branch
`codex/integrate-m6d-main`. De eerdere squash van PR #12 naar main (`1f24f994`)
was inhoudelijk aanwezig, maar ontbrak als ouder van de integratiebranch.
Commit `e089e929` verbindt beide geschiedenissen zonder bestandswijzigingen.
Het daarna gepubliceerde checkpoint `c8945c6` bewaart die verbinding en voegt
het hieronder beschreven Windowsbewijs toe. Ook het latere Copilot-checkpoint
`f835a061` en zijn integratieverslag blijven behouden. De twee oude M5-logs blijven weg.

Een nog ontbrekende CodeRabbit-reparatie uit PR #2 is gericht overgenomen:
de qsim-JSON-parser accepteert alleen objecten voor requests en gates.
De oude parser faalt op twee nieuwe regressietests; na herstel slagen
**27 hosttests**, `cargo fmt --all -- --check` en
`cargo clippy --offline --locked --workspace --all-targets -- -D warnings`.
Lockfile, dependencies en quantumcore blijven gelijk. Exacte commando's,
bronbinding en ruwe uitvoer: [qsim-request-validation](verification/qsim-request-validation/README.md).

De 144 native/build/runnerinputs zijn bytegelijk aan Windows-uitvoeringsbron
`326d2f6`. Deze integratiestap controleert gepubliceerd bewijs en voert geen
nieuwe native build of VM-test uit. De historische M6b-observerbeperking en
de twee ontbrekende individuele M1-QEMU-hashes blijven expliciet. Voortgang
**14% → 14%**: de hostparserfix en integratie voegen geen native criterium toe.
`python3 tools/progress.py --write` en daarna `--check` zijn uitgevoerd met
exit 0; zij controleren de berekening, bewijsverwijzingen en weergave.
Reviewaanvulling `82c94221f4e50fbcc5ce3a6200372f94f1d63efc` behandelt vijf
CodeRabbit-punten: bytebehoud van de historische archieven, brongebonden
M6c-opdrachten, de actuele R08-receiptnaam, de latere A04-scriptversie en de
actuele FINAL-REVIEW-hash met vorige/tussenliggende commits. Alleen de twee
gecorrigeerde documenthashes veranderen in de actuele M6d-index; historische
test- en auditreceipts blijven bytegelijk. De CodeRabbit-deelclaim dat het
A04-sectorbestand als ontbrekend stond vermeld, bleek niet reproduceerbaar;
het bestand stond al in Git en in de overdrachtstabel.

Een afzonderlijke lokale Git-controle met `core.autocrlf=true` en
`core.eol=crlf` bevestigt `text: unset` voor alle 26 betrokken archiefpaden.
Acht CRLF-proefbestanden (beide archieven, direct/genest, JSON/TXT) blijven
bytegelijk in de index en na `git checkout-index --all --force`.
De documenthashes, historische hashherkomst en receiptverwijzingen zijn
eveneens gecontroleerd; geen native uitvoering of nieuwe acceptatiepunten.

Dit verslag legt de bron en uitvoering van de integratie vast. De definitieve
review- en squashmergestatus staat in
[PR #13](https://github.com/simonvanieperen/nieuw-os/pull/13).

## Main- en remote-PR-integratie — conflictresolutie 28 oktober 2026

De actuele `main` (`1f24f994`) is samengevoegd in mergecommit
`6603a6bc`, vanaf gemeenschappelijke basis `5a58a743` op integratiebron
`1068e89c`. De M6c/M6d-
implementatie bleef behouden. De gedeelde kernel-, capability-, virtio- en
dienstbronnen zijn bytegelijk aan zowel de M6d-uitvoeringsbron `88e6c04` als de
overlappende bron op `main`; de M6c-selectie is apart behouden. De twee
eerder uitgesloten, onbewezen M5-corelogs uit de inkomende `main` zijn niet
opnieuw toegevoegd; overig main-bewijs is behouden.

Na conflictresolutie zijn uitgevoerd (allemaal exit 0):

```text
python3 -B tests/test_guest_harness.py                         # 7 tests
python3 -B tests/test_m6b_capture.py                            # 13 tests
python3 -B -m py_compile tools/build.py tools/build_users.py tools/capture_m6b.py tools/run.py tools/run_m6b.py
cargo +1.90.0 test --locked -p capability-core -p virtio-core -p block-protocol -p object-core -p catalog-core -p kernel-core # 139 tests
rustup +1.90.0 target add x86_64-unknown-none x86_64-unknown-uefi
python3 -B tools/build.py --kernel-feature milestone-m6c
```

De eerste buildpoging eindigde met exit 1 omdat `x86_64-unknown-none` niet was
geïnstalleerd (`E0463`). Na installatie van de twee officiële Rust-targets
slaagde de build; twee bestaande dead-codewaarschuwingen bleven staan. Rustup
installeerde automatisch de ontbrekende 1.90.0-toolchain en Cargo ververste de
index zonder lockfilewijziging.

De M1–M6d-QEMU-gates zijn **niet opnieuw uitgevoerd** op de mergecommit.
De succesvolle M6c-crossbuild bewijst alleen compilatie, geen boot of VM-gedrag.
De eerder opgeslagen M6d-gates blijven aan hun exacte uitvoeringsbron gebonden;
de native bronvergelijking bevestigt dat de merge geen runtimecode van die bron
veranderde. Voortgang blijft **14% → 14%**: geen nieuw native criterium of
bewijsclaim.

De bijgewerkte remote PR-branch `c8945c6` brengt het M6d-closeoutbewijs mee:
opgeslagen M6c/M6d- en M6b-captures, plus een Windows-hostsuite van 173 tests.
Dit zijn ingekomen, brongebonden rapporten op checkpoint `326d2f6`, geen nieuwe
lokale VM-runs. De volledige hostregressie is ook lokaal op de gecombineerde
werkboom uitgevoerd: `python3 -B -m unittest discover -s tests -v`, **173/173,
exit 0**. De uitvoer is in deze sessie getoond, maar niet als nieuw
repository-logbestand opgeslagen.

## M6d-reviewherstel — Windowsacceptatie 28 oktober 2026

Aanvullende uitvoering op `326d2f6`, in een afzonderlijke worktree buiten
synchronisatiemappen. Fingerprint v3:
`5c69802a6de7c591207950f311d2e9e4f7eb07628d3b91dc9457f21c0bfa8c75`.
M6c/M6d: **35 groepen / 31 VM**, M6b: **18 groepen / 14 VM**, beide captures
exit 0. Samen 45 VM-starts, 20 koude recoveries en vier externe Windows-kills.
De twee gewone M1-runs missen een eigen per-run-QEMU-executablehash; die
meetgrens blijft open. Native kernel/service/probe zijn bytegelijk aan 88e6c04.

Host: M6c 30 Rust- en 48 Python-tests; M6b 22 Rust- en 30 Python-tests.
De bredere definitieve Python-hostsuite slaagt met **173/173**, inclusief
22 auditor- en 19 capturechecks. Deze overlappende aantallen niet optellen.
De eerdere brede suite faalde door LF→CRLF-conversie in synthetische fixtures;
alleen fixturebytes zijn gecorrigeerd. Exacte productiebytevergelijking en
single-linkguard blijven intact. Vier bronmutanten toetsen de auditbypass;
geen algemene formele Pythoncontrole.

De mislukte gesynchroniseerde run blijft failed bewaard. Een onmiddellijke
`fsutil hardlink list` toonde een echte tweede naam onder `.tmp.driveupload`;
het veroorzakende proces is niet vastgesteld. Identieke bron buiten die map
slaagt. Oorspronkelijk Linuxbewijs blijft aan 88e6c04 gebonden; de volledige
historische rawaudit is niet opnieuw uitgevoerd. Een tweede agent controleerde
alle 45 rawrecords, 395 artifacthashes en de werkelijke imagebytes; geen
menselijke audit of fysieke stroomuitvalclaim.

Exacte opdrachten en grenzen: [M6D-CLOSEOUT.md](verification/M6D-CLOSEOUT.md).
Voortgang **14% → 14%**; volgende native stap blijft catalogusherstart.
PR #11 is tussentijds extern gemerged op checkpoint 326d2f6 naar de M6c-branch
(`baeaa7bf`); #12 is extern naar main gemerged (`1f24f994`). Beide nieuwe trees
zijn vergeleken: zij voegen geen main-only broncorrectie toe. PR #13 behoudt
deze actuele main als ouder en voegt M6c/M6d plus het definitieve reviewbewijs
toe. Twee eerder verwijderde oude M5-logs blijven uitgesloten. Bij het schrijven
van dit closeoutverslag was de merge van #13 nog niet geclaimd; de actuele
integratiestatus staat in het bovenstaande checkpoint.

### Vooraf bewaard checkpoint

Hervat vanaf GitHub-head `7484d87`, afzonderlijke checkout `nieuw-os-review`.
De oude M6a-werkmap en ongetrackte bewijsbestanden zijn behouden. De vijf
CodeRabbit-punten op PR #11 worden gericht afgehandeld: exact QEMU-contract
ook voor M1, verplichte 164-bestandsinventaris, reproduceerpaden, historische
bestandsnaam en een nieuwe expliciete bronreview van de definitieve native bytes.
De historische R08-review en receipts blijven bytegelijk; hashwijzigingen van
de bijgewerkte audit/docs staan bewust in `verification/m6d-summary.json`.

De eerste Windows-hostcontrole faalde bij de natuurlijke exit van een child
met WinError 5, zowel in M6b als M6c. Na verlenging van uitsluitend de begrensde
wait bij een geweigerde TerminateProcess-call van 0,25 naar de bestaande
cleanupgrens van 10 seconden slagen 48 M6c- en 30 M6b-Python-tests. Een natuurlijke
exit blijft `kill_requested:false`; werkelijke live kills en timeoutafwijzing
blijven getest. De kernel/service/probe zijn ongewijzigd.

Dit checkpoint gaat vóór de nieuwe volledige Windows-gates naar GitHub.
Die gates, hun captures, externe herreview en merges waren op dit checkpoint
nog **niet** afgerond. De volledige historische Linux-rawaudit is in deze Windowscheckout
niet uitvoerbaar zonder oorspronkelijke ruwe bestanden en QEMU-binary; de
nieuwe audit heeft afzonderlijke negatieve hosttests. Voortgang **14% → 14%**.
Volgende native functie blijft de beperkte catalogusherstart uit WORK-QUEUE.

## M6d — native onzekere I/O gebouwd en geaccepteerd

28 oktober 2026. Uitvoeringsbron `88e6c04be218b8b09ea4f0a226f6b9719da706a6`,
fingerprint v3 `8ca6d5d5291fde62f91fb47dca7df4d058424604449fb32b9e5899b1d4fbef93`.
Werkbranch `codex/m6d-io-recovery-roadmap`, [PR #11](https://github.com/simonvanieperen/nieuw-os/pull/11).
De broncheckpoint is vóór de eindgates gepusht en bewaart zowel M6d-checkpoint
b739183 als upstream M6c-review 5ba238cc. De afsluiting wijzigt documentatie en
bewijs; alle 45 eindruns zijn aan deze ene native uitvoeringsbron gebonden.

De kernelobserver koppelt de eerste Unknown aan een werkelijk teruggegeven
terminale muterende syscallfout. Geen onzekere commit, succes-ACK of levende
revisie; beide appdomeinen blijven actief. Na Unknown leveren geldige
GET/LIST/PUT/DELETE/RENAME-aanvragen geen nieuwe I/O op. Ongeldige of vreemde
verzoeken houden hun eigen validatie-/autorisatiefout. Acht native testfeatures
dekken WRITE/FLUSH-status, time-outs, resetquarantaine, descriptorweigering en
ongeldige used-length. De eigen kernel, capabilities, IPC v4 en journal v2
blijven het fundament; geen nieuwe externe dependency.

| Werkelijk uitgevoerd op deze bron | Resultaat |
|---|---|
| M6d-gate en capture | exit 0; 35 groepen / 31 VM-invocaties, 14 koude recoveries, 8 foutcases, 2 externe cuts |
| Afzonderlijke M6b-regressie en capture | exit 0; 18 groepen / 14 VM-invocaties, 6 koude recoveries, 2 externe cuts |
| Hostcontroles | 30 Rust + 48 Python voor M6c; 22 Rust + 30 Python voor M6b; exit 0 |
| Gerichte rustfmt en normale eindbuild | exit 0; kernel/loader bytegelijk aan beide normale M6d-runs |
| Twee afzonderlijke bewijsaudits | exit 0; 274 + 121 artifacthashes, 16 + 7 werkelijke volumes; bronnen en herstelkoppelingen gecontroleerd |

R09 vergeleek alle 144 bronbestanden en vier M6c-hosttestbestanden met de
uitvoeringscommit; alle 368 M6d-runfiles zijn gehasht. De 16 M6d-images zijn
ook vanuit alleen seeds en sectorhex opnieuw opgebouwd met dezelfde hashes.
A04 controleerde afzonderlijk de 14 M6b-runs en hun werkelijke schijfbytes.
AI-review is geen menselijke audit of formeel bewijs. Exacte commando's,
archieven en grenzen: [M6D-ACCEPTANCE.md](verification/M6D-ACCEPTANCE.md).

De eerste gate stopte vóór een VM door een ontbrekend firmwarezoekpad;
de failed suite is bewaard en het aanwezige passende OVMF-paar is naar de
ondersteunde lokale testmap gekopieerd. Daarna is de volledige gate opnieuw
uitgevoerd zonder bron-/criteriumwijziging. De twee concrete reviewbevindingen
(archiefbestemming en precieze request-ID's) zijn vóór de bronfreeze hersteld.

**Voortgang: 13% → 14% (+1 punt voor S07).** README bevat de zichtbare balk;
`progress.json`, `PROGRESS.md` en `tools/progress.py` houden de vaste 100-punten-
roadmap consistent. AGENTS.md verplicht herbeoordeling na iedere opdracht,
ook als de score gelijk blijft. Geen tijdsraming of veiligheidspercentage.

Grenzen: koude recovery start een nieuwe VM op dezelfde bytes; automatische
herstart in dezelfde sessie is nog open. De historische M6b-observer houdt zijn
foutpadbeperking. Vier objecten, 128 inhoudsbytes en 32 records blijven
proefbudgetten; geen desktop, grotere documenten, encryptie of native NAS.
Status-/completioninjecties zijn geen fysieke schijffout of stroomuitval.
Volgende beperkte stap: geautoriseerde catalogusherstart met nieuwe generatie,
veilige reset en journalherstel zonder replay. K05 en S08 blijven open.

## M6c — native documentnamen en catalogus gebouwd en getest

28 oktober 2026. Basis `bb22fdbe15919edc23baa158a46236195ce314b2`, werkbranch
`codex/m6c-named-catalog`. De afgeronde M6b-basis hieronder blijft behouden.

Gebouwd: dependencyvrije `catalog-core`, IPC v4, journal v2, blijvende UTF-8-namen,
GET/PUT/DELETE/RENAME en één domeingebonden LIST-snapshot. Hernoemen bewaart de
inhoud onder een nieuwe objectrevisie; alleen na WRITE én FLUSH volgt succes.
De vier objecten, twee domeinen, 128 inhoudsbytes en 32 journalrecords blijven
proefbudgetten. Nieuwe formaten zijn expliciet; bezet M6b-v1 wordt vóór READY
geweigerd, zonder migratie of formattering. Bestaande M6b blijft afzonderlijk
bouwbaar. Contract: `implementation/M6C-NAMED-CATALOG.md`; besluit ADR-030.

Definitieve uitvoeringsbron: `bc4cd104e86e2c047b727ae96e07a03fbc810c3c`;
fingerprint v3 `4468b3c18bd7fa57bf3631d634d52ee4c5fe9a76da9f125714d6db95e4407345`.
Deze bron is vóór de lange controles op GitHub bewaard. De oorspronkelijke afsluitende wijzigingen
betreffen documentatie en bewijs. De latere integratie van het M6b-reviewherstel
wijzigt hosttools en tests; de native runtimebron blijft gelijk. Het afzonderlijke
hostbewijs staat in `verification/m6c-review/merge-followup/`. [PR #10](https://github.com/simonvanieperen/nieuw-os/pull/10)
staat boven de M6b-branch; geen merge naar main of release uitgevoerd.

| Scope op deze bron | Resultaat | Bewijs |
|---|---|---|
| Volledige M6c-gate en capture | exit 0; 19 groepen / 15 VM-invocaties | `verification/m6c/` |
| Volledige M6b-regressie en capture | exit 0; 18 groepen / 14 VM-invocaties | `verification/regression-m6c/` |
| Catalog-core en harnas | 30 Rust- en 33 Python-tests; fmt/Clippy/no_std exit 0 | `verification/m6c/`, `verification/m6c-review/core-recheck/` |
| Echte adapter via host-ABI-shim | 15 scenario's, exit 0; geen native devicefoutinjectie | `verification/m6c-review/adapter-result.json` |
| Afsluitende M6c-build | exit 0; kernel en loader bytegelijk aan beide normale eindruns | `verification/m6c-ready/` |

De 15 M6c-gate-invocaties zijn 14 M6c-VM's plus één gewone M1-boot. Ze omvatten
twee normale images, domeingrenzen na populatie, volle journal, corruptie, geldig
legacy-v1, zes koude recoveries en twee echte externe QEMU-kills tijdens RENAME.
Vóór WRITE blijft `Café A` op revisie 1; na FLUSH vóór ACK verschijnt
`Café hernoemd A` op revisie 2. De 128 inhoudsbytes blijven exact gelijk.
Alle zes koude herstelruns veranderen geen schijfbytes. De M6b-regressie omvat
eveneens een eigen gewone M1-boot; dit zijn geen nieuwe volledige M1–M6a-gates.

Een afzonderlijke AI-audit controleerde 130 geregistreerde artifacthashes,
176 runbestanden, acht werkelijke volumes en alle 141 bronbestanden tegen
`bc4cd104`. De 67 niet-lege journal-sectoren en seeds zijn als tekst bewaard;
daaruit zijn alle imagehashes opnieuw opgebouwd. Zes vroege runrecords noemen
nog de toenmalige lokale HEAD `bb22fdb`; hun volledige broninhoud en fingerprint
zijn bytegelijk aan `bc4cd104`. Dit metadataverschil blijft zichtbaar.
De aparte statische integratiecontrole bindt het native gebruikers-ELF aan de
geteste kernel en controleert budgetten en instructies. Reviews zijn AI-reviews,
geen menselijke audit of formeel bewijs. Exacte commando's, afbakening en
provenance: [M6C-ACCEPTANCE.md](verification/M6C-ACCEPTANCE.md).

De vijf GitHub-reviewopmerkingen over bewijsstatus en reproduceerbaarheid zijn
afgehandeld in `verification/m6c-review/REVIEW-FOLLOWUP.md`: actuele native
resultaten gelinkt, portable adapter opnieuw getest en de vier corecontroles
read-only herhaald met HEAD/status en bronhashes vóór/na. Historische evidence
blijft behouden. Dit verandert de native bron of het aantal VM-runs niet.

Het geïntegreerde hosttoolherstel heeft 123 unieke hosttests over twee runs:
121 bestaande tests, gevolgd door vijf preflighttests waarvan drie herhaald
en twee nieuw. Een latere reviewaanscherping bindt de drie M6c-hostlogtrio's
met SHA-256 aan de gate en controleert de binding vóór én na het kopiëren.
Deze wijziging heeft afzonderlijk hosttestbewijs in
`verification/m6c-review/host-log-binding/`; de eerdere 123 tests worden niet
als uitvoering op deze latere toolbron opgevoerd. De oorspronkelijke M6c-suite
mist gate-time-hostloghashes en blijft als historisch bewijs ongewijzigd.

Na opname van het M6b-vervolgherstel zijn de gedeelde capturepaden samen
gecontroleerd: 29 tests geslaagd (vier M6a, tien M6b, vijftien M6c), met
ongewijzigde inputhashes vóór/na. Dit is één afzonderlijke hostinvocatie;
het aantal wordt niet bij eerdere overlappende runs opgeteld. Broncombinatie,
commando en ruwe uitvoer staan in `verification/m6c-review/integration-followup/`.
Geen native build of VM uitgevoerd voor deze integratiecontrole.

Bekende observerfout op de hierboven vastgelegde historische M6c-bron: native
WRITE-/FLUSH-fouten of time-outs kunnen ook in de gewone `milestone-m6c` een
kernel panic veroorzaken voordat de client `Unknown` ontvangt. De host-ABI-
proeven toetsen alleen de serviceadapter. Contract en M6C-ACCEPTANCE vermelden
expliciet dat dit native pad op die bron niet gerepareerd of end-to-end bewezen
is. De afzonderlijke M6d-reparatie en haar geslaagde foutinjectiegate staan bovenaan.
De onderzoeksacceptatie blijft begrensd tot de vastgelegde QEMU/device-fixture.

Vervolg: deze opslaginterface aan native productfuncties koppelen; grotere
documenten, ruimtehergebruik, encryptie, sleutels en native NAS/netwerk vereisen
eigen ontwerp en acceptatie. M6c heeft nog geen interactieve desktop. FNV blijft
uitsluitend corruptiediagnose; QEMU-kills bewijzen geen fysieke stroomuitval.

## M6b — native documentopslag gebouwd en getest

Basis `10b32e31671827f2bd63444113b981ff15bfbb73`, werkbranch `codex/m6b-object-storage`.
De nieuwe `services/objects`-crate bevat een allocatorvrije Rust `no_std`-objectdienst:
vier objecten van maximaal 128 bytes, twee via IPC-endpoints gescheiden domeinen,
GET/PUT/DELETE, revisiecontrole en blijvende verwijderrecords. Alleen de dienst
krijgt blokrechten op de 32 journal-sectoren; apps krijgen hun eigen endpoint.
Succes volgt pas na echte WRITE- en FLUSH-completion. Corruptie en onzekere
mutaties sluiten toegang af; er is geen automatische herhaling van writes.

Definitieve uitvoeringsbron: `1e61a20beac67d685cf4bff5f0f9f588bf2cb56e`;
fingerprint v3 `43e6c06e43e3ba602b49c304328f0c1700bc26dae0a1256f0d5d420421f55ae2`.
Op 28 oktober 2026 (UTC-logdatum 27 oktober) zijn beide volledige eindgates en hun captures met exit 0
afgerond. De resultaten hebben afzonderlijke scopes en worden niet met eerdere
deelruns opgeteld:

| Scope | Bron | Groepen / echte VM-runs | Bewijs |
|---|---|---|---|
| M6b volledige objectopslag | `1e61a20` | 18 / 14 | `verification/m6b/` |
| M1 volledige boot- en foutgate | `1e61a20` | 10 / 8 | `verification/regression-m6b/m1/` |
| M2–M5 volledige regressies | `44892e7` | 67 / 56 | `verification/regression-m6b/m2/` t/m `m5/` |
| M6a volledige ongewijzigde baseline | `10b32e3` | 54 / 49 | `verification/m6a-linux/` |
| M6a normal/herstel op M6b-runtime | `44892e7` | afzonderlijk paar / 2 | `verification/regression-m6b/m6a-pair/` |

M6b omvat twee normale images met dezelfde binaries, CRUD/domeingrenzen, een
volle journal, corruptie, twee echte externe QEMU-kills en zes koude recoveries.
Een aparte AI-audit decodeerde zeven werkelijke volumes en controleerde alle
121 artifacthashes. De M1-audit controleerde acht eigen gastlogs, 31 artifacthashes,
vier buildprofielen en de echte 60-seconden-time-out. De 56 eerdere M2–M5-runs
zijn onder de laatste parser opnieuw gecontroleerd; alle 68 native bron- en
bouwinputs zijn sinds `44892e7` bytegelijk. Dat zijn geen 56 nieuwe runs.

Hostbewijs: 22 Rust-objecttests, 26 M6b-harnastests, 8 snapshotbindingtests,
7 guestparsertests en 9 bestaande capturetests. Zeven aanvullende proeven van
de echte serviceadapter met een host-ABI-shim controleren onzekere WRITE/FLUSH-
uitkomsten; dit is geen native M6b-devicefoutinjectie. Reviews staan in
`verification/m6b-review/`; het zijn AI-reviews, geen menselijke audit of formeel
bewijs. Rust 1.90.0, QEMU 10.0.2 en het bijbehorende OVMF-profiel zijn vastgelegd
in `verification/native-linux-toolchain/`.

Drie eerdere M1-pogingen zijn expliciet afgewezen en behouden. De oude parser
accepteerde eerst een hogere testfase; later blokkeerde de hashguard teruggekeerde
oude artifacts vóór QEMU-start. De vervangingsoorzaak is niet vastgesteld.
M1 gebruikt nu per build een eigen gecontroleerde UUID-snapshot, dezelfde normale
kopie voor alle coldboots/negatieve fixtures, strikte fasecontrole en hashes vóór
iedere start. De afsluitende volledige M1-gate slaagt. De wrapperconsole is niet
overal compleet; de bewijsbasis zijn de afzonderlijke gecontroleerde gastrecords.
Details en de geslaagde capture-hercontrole zonder nieuwe VM-runs:
`verification/regression-m6b/FINAL-ACCEPTANCE.md` en
`verification/m6b-review/capture-recheck/`.

De laatste lokale build is opnieuw `milestone-m6b`; kernel en loader zijn
bytegelijk aan beide normale eindgate-runs. Dit is een buildcontrole zonder
extra VM-start (`verification/m6b-ready/`).

Reviewcorrecties van 28 oktober 2026, basis `bb22fdbe15919edc23baa158a46236195ce314b2`:
historische audits/guards en M1-afwijzingen verduidelijkt, bewijsbeschrijvingen
vertaald en M5-vergelijkingen aan geverifieerde gastrecords gebonden. Preflight
weigert nu ook een niet-nul `probe_exit` en een `probe_error`; de oude `rustup`-probe met `-6` was dus
geen geldige inventariscontrole. De M6b-gate probeert na fouten de normale M1-build
te herstellen en bewaart de oorspronkelijke fout. De 40 onafhankelijk uitgevoerde gerichte hosttests slagen; exacte commando's,
bronhashes en ruwe uitvoer staan in `verification/m6b-review/merge-followup/`.
Deze follow-up betreft hosttools; de oorspronkelijke native uitvoeringsbronnen
in de tabel hierboven blijven behouden.
De vervolgreview scherpt de M5-archiefchecker aan met expliciete controles,
ook onder `python -O`; een gewijzigd seedrecord wordt nu afgewezen. Vier
nieuwe hosttests verifiëren de M6a-stderrverwijzing en volledige
stdout/metadata/stderr-capture. Historische meetbestanden blijven intact;
de M6a-suite heeft een afzonderlijke gecorrigeerde leesindex. Dit nieuwe
hostbewijs staat in `verification/m6b-review/review-followup-2/` en is geen
nieuwe native gate of herhaling van de eerdere 40 tests.
Geen nieuwe native gate uitgevoerd; de M3-hosttestcheckout en de redirectinstelling
van de oude buitenste capturewrapper blijven expliciet onbekend.
Historische native observerbeperking op deze eerdere bron: WRITE-/FLUSH-fouten konden, ook in
`milestone-m6b`, een kernel panic veroorzaken voordat de client `Unknown` krijgt.
Deze oude gates leveren daarvoor geen native foutbewijs. De reparatie en nieuwe
afgebakende injectieproeven op `592dccf` staan in de bovenste hervattingssectie.

Deze begrensde lokale opslagproef heeft geen namen, directoryboom, garbage
collection of encryptie. Verwijderrecords voorkomen het terugkomen van oude
appverzoeken; zij wissen historische schijfbytes niet fysiek. FNV is uitsluitend
een corruptiediagnose. Native NAS/netwerk, sleutels en desktopkoppeling blijven
afzonderlijke stappen. Contract: `implementation/M6B-OBJECT-STORAGE.md`.

## N02 — hybride PQC-transport: echte hostproef getest

27 oktober 2026, basis `ece94a5cfebecaec763eb2855aa9f84e55dcdcbc` (PR #7), werkbranch `codex/pqc-transport-foundation`; eerste checkpoint `3d05f9f8dccb4f515d8522578d20bbf00d8b8ed7`. De aangeleverde beveiligingsblauwdruk is getoetst aan de actuele bronnen en repository. ADR-028 en `implementation/VERIFIABLE-SECURITY-AND-PQC.md` leggen het standaard-TLS-profiel, formele bewijsgrenzen en concrete native vervolgopdrachten vast.

`services/nas/transport.mjs` heeft echte loopback TLS 1.3 met gemeten X25519MLKEM768, AES-256-GCM-SHA384, vereist ALPN, wederzijdse CA-/certificaatpincontrole en serverhostnamecontrole. De client versleutelt vóór transport; de server echoot alleen begrensde ciphertext. Policy controleert toestemming vóór de socketpoging, na authenticatie en bij iedere verzending/response. Een transportfact is gebonden aan app en toestemmingsgeneratie; intrekken en opnieuw toestaan herstelt geen oude geauthenticeerde verbinding. Alleen internet intrekken laat de toegestane LAN-route bestaan. Geen klassieke fallback of sessiehergebruik.

Werkelijk uitgevoerd op Node 24.22.1/OpenSSL 3.5.9/Linux x64: `node --test --test-isolation=none services/nas/tests/*.test.mjs ui/prototype/tests/*.test.mjs` → **124/124**, exit 0, geen skips (85 service: 13 crypto, 19 policy, 23 store, 30 transport; 39 UI). Beide demo's exit 0; 20 JS/mjs-syntaxchecks en diffcontrole geslaagd. Ruwe uitvoer, bronhashes, runtimeversies en review: `verification/pqc-transport/`. De oorspronkelijke sandbox weigerde lokaal luisteren; de expliciete loopback-escalatie is automatisch toegestaan en de tests zijn vervolgens werkelijk uitgevoerd. Geen externe verbinding of echte NAS/router gewijzigd.

Twee afzonderlijke AI-reviews; een concreet randgeval rond herleving van oude idletransportfacts is hersteld en onafhankelijk hertoetst. Dit is geen formeel bewijs of menselijke audit. De testcertificaten zijn klassiek Ed25519; netwerklabels zijn harnasaannames bij loopback. Geen postkwantumauthenticatie, WAN-/LAN-detectie of koppeling van transportecho aan de N01-store. Tijdelijke frame-/tijdsgrenzen zijn geen productbeperkingen. De browserrendering is in deze stap niet opnieuw getest.

Geen native PQC-crate, entropy-syscall, boot-/blokcryptografie of Grover-uitbreiding toegevoegd. De native build heeft concrete ABI-/geheugen-/instructiegrenzen; Rust/Cargo en QEMU waren tijdens die N02-hostsessie niet beschikbaar. Het huidige M6b-traject hierboven heeft inmiddels een werkende native toolchain. Dit verandert geen bestaande M1–M6a-bewijsstatus. N01 hieronder blijft als historische basis bewaard.

## N01 — internet en persoonlijke NAS-ruimte: hostproef getest

27 oktober 2026, gebouwd vanaf ontwikkelbasis `a96ad45adb044177e18670cd38624ea2a671db3d` op `codex/nas-private-sharing`. De nieuwe gebruikersopdracht verruimt de eigen-LAN-beperking naar eigen of vrienden-/familie-NAS, met expliciet peergebonden internet, client-only inhoudssleutels en eenzijdig adminverwijderrecht. Privacycontract revisie 3 en ADR-027 zijn leidend.

Werkende code in `services/nas/`: AES-256-GCM voor inhoud en namen; echte lokale ciphertextstore; tenantgrenzen, quota, eenmalige uitnodigingen, aparte admincapability, revoke/evict, afwijzen van oude sessies en uploads, duurzame tombstones en heropening. Materiaal bevat Mijn NAS, Ruimte ontvangen en Ruimte aanbieden als browsermock. Eenmalig toestaan volstaat voor automatische routekeuze LAN→internet→LAN naar dezelfde geverifieerde peer; planner en UI verlenen daarbij geen nieuwe rechten. Lokale beheerrol is apart van NAS-datatoegang; internet geeft geen beheerrecht.

Werkelijk uitgevoerd op Node 24.22.1/OpenSSL 3.5.9/Linux x64: `node --test --test-isolation=none services/nas/tests/*.test.mjs ui/prototype/tests/*.test.mjs` → **89/89**, exit0 (50 service, 39 UI). `node services/nas/demo.mjs` → passed/exit0. Ruwe uitvoer, bronhashes, review en grenzen: `verification/nas/`. De procescrashproeven gebruiken eigen synthetische tempmappen. Geen echte gebruikersbestanden, NAS of router gewijzigd.

**Niet geleverd/bewezen:** native netwerkstack, TLS-/echte peer-authenticatie, socketafdwinging, NAS-installatiemodus op eigen OS, veilige persistente schrijfkeys/multi-device recovery, fysieke stroomuitval, vijandige-admin snapshotrollback of fysieke gegevensvernietiging. Key recovery is alleen lezen; N01 bevat geen achtergrondsync. Algemene internettoestemming is nog appgebonden, NAS-internet wel peergebonden. Browserrendering van de nieuwe schermen is niet gecontroleerd: de Browser-plugin kon zonder oorspronkelijke browsercontext niet starten; state-/syntaxcontroles wel uitgevoerd.

N01 wijzigt de M6a-runtimecode en bestaande bewijsclaims niet. De M6a-eindpoort hieronder blijft een eigen open stap. Vervolg: M6a afronden, dan native opslag-/netwerkadapter en clientkeybeheer volgens implementation/NAS-SHARING-AND-INTERNET.md.

## Actief: M6a

Nieuwe opdracht 27 oktober 2026 vanaf `448974b`, branch `codex/m6a-write-recovery`. Bouwcontract vóór code: implementation/M6A-WRITE-RECOVERY.md. Gekozen: WRITE512/FLUSH met expliciete rechten, een vierrecords-journal, echte dienstcrashes en QEMU-proceskills plus koude recovery op uitsluitend harnasgegenereerde images. Geen fysieke stroomuitvalclaim. M5-basis hieronder behouden; oorspronkelijke status ook in verification/m5/PROJECT-STATE.md.

Mergecheckpoint: de basisbranch `codex/m2-m4-isolation` (`e816dfd`) is geïntegreerd. De M5-kernel- en archiefvalidatie uit die branch blijft behouden; M6a-rechten, transport, toolroutes en herstelcode blijven actief. Post-merge geslaagd: Python-tool `py_compile`; evidence-/capture-/M6-harnascontroles (9, 1, 4, 4 en 20 tests); M6-ELF-artifacttests (12); Rust-hosttests (`cargo +1.90.0 test --locked -p capability-core -p virtio-core -p block-protocol -p kernel-core`: 30/21/20/16); en `python -B -X utf8 tools/build.py --kernel-feature milestone-m5` plus `--kernel-feature milestone-m6a`. De M1–M6a-QEMU-poorten en gastregressies zijn na deze merge niet opnieuw uitgevoerd.

Eerste samenhangende bronversie: typed WRITE32/FLUSH64, Prepared/Published/Unknown, kernel-inputsnapshot512, modern transport, echte nieuwe M6-userdienst/client/supervisor en Python-volume-/crashharnas. Daadwerkelijk geslaagd: 30 capability- en 21 virtio-tests, 20 protocol/journaltests, 16 kernel-core-tests en 12 M6-ELF-artifacttests; core fmt/Clippy/no_std exit0. Eerste echte M6-crossbuild exit0; latere twee observercorrecties worden opnieuw gebouwd. Het root-harnas heeft 8 schijf-oracle- en 5 childprocescontroles geslaagd, geen guestbewijs. Logs/hashes onder verification/m6a/core/ en service/.

Gerichte echte M6-QEMU-proeven zijn nu geslaagd: normaal gen2, 512B-rechten/snapshot, dienstcrashes vóór WRITE en na ACK, FLUSH-fout en resetquarantaine. Ook een echte externe QEMU-kill na FLUSH (raw3765043201, geen PASS) plus nieuwe VM op exact dezelfde image is geslaagd: gen2, orakel groen, recovery veranderde geen imagebytes. Negen diagnostische records inclusief de eerdere afgewezen getalnotatie blijven onder verification/m6a/diagnostics/. Laatste broncorrecties zijn nog niet door de volledige gate afgedekt.

Transport-/harnasreviews sloten concrete foutvolgorde-/Rust-borrow-/recovery-/coveragebevindingen; rapporten onder verification/m6a/. Root-hostharnas heeft nu 19 geslaagde controles. De volledige poort is geïmplementeerd (verwacht 54 groepen/49 VM, nog niet als geslaagd geclaimd), met 6 dienstcuts, 6 echte VMcut/recoveryparen, Unknown/full/corruptie en twee normale images. Nu: bronfreeze/checkpoint, die hele poort werkelijk uitvoeren, daarna M1–M5-regressie en onafhankelijke raw/hash-audit.

GitHub-e-mailprivacy blokkeerde de eerste push; alleen de ongepushte commitmetadata is naar het GitHub-noreplyadres hersteld. De branchpush en remote-ID-controle slaagden daarna, zonder privacy-/globale configwijziging. M6a staat als concept-PR https://github.com/simonvanieperen/nieuw-os/pull/5 boven op de M5-branch.

Finale kandidaatbron `8c53ea4` bevat ook de twee tussentijdse remote-orakelcorrecties via een gewone merge, zonder force-push. Fingerprint v3: `6fce5c12e87cf6b1a1353743357593de3d8b6843012b12be6deaf7fe17ce3b66`. Volledige M6a-poort gestart; tussentijdse snapshot bij 43 geslaagde groepen bewaart nog nadrukkelijk running/false in verification/m6a/checkpoints/. Normaal schrijven, volle journal en koude recovery zijn daarin werkelijk groen; volledige einduitslag/regressie/audit volgen afzonderlijk. De root-harnascontroles zijn inmiddels 20/20 groen, inclusief behoud van beide remote-correcties.

27 oktober 2026. M5 is geïmplementeerd en getest in het vaste QEMU-profiel.

Repository: https://github.com/simonvanieperen/nieuw-os. Branch `codex/m5-block-service`, concept-PR [#3](https://github.com/simonvanieperen/nieuw-os/pull/3). Basis `c630c59` (M1–M4); definitieve runtime-/harnasbron `7af544aa1b8d731ac27f384f0ba95aa44586a57d`. Latere commits bewaren documentatie en bewijs. Main bevat de startspecificatie; geen merge of release uitgevoerd.
## Reviewfollow-up

De bewijsarchiefcapture valideert een volledige nieuwe opname in een tijdelijke map en laat het bestaande archief bij validatie- of installatiefouten ongemoeid. Alleen stdoutlogs onder `build/logs/` kunnen als hostbewijs worden opgenomen; daarnaast is alleen het gevalideerde `m5-two-image-check.json` toegestaan als gestructureerd M5-hostbewijs. M2–M4-gastresultaten voldoen aan de gedeelde fase-/caseverwachtingen voor markers en ruwe exitstatus; alle gastresultaten doorlopen hun eigen controles. `python -B -X utf8 tests/test_capture_gate.py` (8 tests) slaagt; dit is hosttoolbewijs, geen nieuwe browser- of OS-test. De registratie van vijf `.png`-bestanden als JPEG gebruikt nu hun signatures: `python3 tools/update_evidence_files.py` en `python -B -X utf8 tests/test_evidence_files.py` (4 tests) slagen. Alle genoemde controles exit 0. De grote-probehostcontrole is lokaal geblokkeerd: Rust 1.90.0 en `x86_64-unknown-none` ontbreken; `tools/preflight.py` meldt ook geen QEMU/OVMF.

De capturegate herleidt M2–M4-markers, verboden markers en ruwe exitstatus nu uit één fase-/casecontract dat ook de milestone-runner gebruikt; verwachtingen in een opgeslagen runrecord worden daarvoor niet vertrouwd. `python -B -X utf8 tests/test_capture_gate.py` (8 tests), `python -B -X utf8 tests/test_guest_harness.py` (3 tests), de gedeelde-contractimportcontrole en vergelijking met alle 36 gearchiveerde M2–M4-gastruns slagen (exit 0). Dit is hostvalidatie van de verifier, geen nieuwe QEMU-run.

Na de nieuwste samenvoeging van `codex/m2-m4-isolation` zijn `python -B -X utf8 tests/test_capture_gate.py` (8 tests), `python -B -X utf8 tests/test_evidence_files.py` (4 tests), `python -B -X utf8 tests/test_guest_harness.py` (4 tests) en `python -B -X utf8 -m py_compile tools/build.py tools/capture_gate.py tools/run.py tools/gate_contract.py tools/test_milestones.py tools/test.py` geslaagd (exit 0). Dit zijn gerichte hostcontroles; de QEMU-gates zijn voor deze merge niet opnieuw uitgevoerd.

De reviewfollow-up blokkeert nu ook een gevalideerde hostlog als bewijs voor een M2–M4-gastcase; de fase-/caseverwachting uit `gate_contract.py` bepaalt dat hiervoor een gecontroleerde runmap nodig is. `python -B -X utf8 tests/test_capture_gate.py` (9 tests), `python -B -X utf8 tests/test_common.py` (1 test), `node --test ui/prototype/tests/state.test.mjs` (23 tests) en `python -B -X utf8 -m py_compile tools/capture_gate.py tools/common.py tools/build.py experiments/quantum/verify.py` slagen (exit 0). Dit zijn host-/mockcontroles; geen QEMU-run.

Een alleen-lezen-Webvenstergrant vervangt nu geen nog geldige Notities-schrijftoestemming van de editor; beide grants blijven afzonderlijk begrensd. `node --test ui/prototype/tests/state.test.mjs` slaagt met 24 tests en `node --check ui/prototype/state.js && node --check ui/prototype/app.js` slaagt (exit 0). Dit is mockbroker-/hostbewijs, geen OS-enforcementbewijs.


## Wat werkt

- Eigen UEFI-loader en Rust no_std-kernel, BootInfo 1.1, eigen framebeheer/paging, W^X, bewaakte stacks, ring-3-processen, PIT-preëmptie, capabilities en Message320-IPC. Geen bestaande kernel als basis.
- Echte Rust-blokdienst in ring 3 met client en supervisor. Alleen de dienst krijgt READ op een getypeerd BlockDevice; requestendpoints binden sector 4 en 6. Appvelden kiezen geen fysieke sector of replycap.
- Modern virtio-blk op PCI `00:05.0`: splitqueue van acht plaatsen, maximaal één uitstaande read, vijf gepinde kernel-DMA-pagina's, begrensde submit/poll/reset en supervisor-RW/NX/UC-MMIO. User-MMIO/DMA/PCI-toegang wordt geweigerd.
- Drie echte dienstcrashes met pending software-read, bevestigde reset, nieuwe dienst-PID/epoch en echte IPC-capoverdracht. Client en supervisor behouden PID/adresruimte; epoch 4 levert nieuwe devicebytes. Geen claim dat DMA fysiek tegelijk met de fault liep.
- Alle 13 ongeldige dienstverzoeken veroorzaken aantoonbaar geen blockcall/publicatie/completion. Foutreplies zijn leeg/nulgevuld; onverwachte caps worden gesloten. Time-out/reset/quarantaine en lokale foutafhandeling zijn getest.

M5 is alleen lezen op synthetische wegwerpimages. Nog geen schrijvende opslag, bestandssysteem, interactieve native desktop of algemene spawn-/driverbeheer-API.

## Werkelijk uitgevoerd

| Opdracht / fase | Resultaat | Bewijs onder verification/m5/ |
|---|---|---|
| preflight.py --strict | exit0, geen ontbrekende tools/targets | preflight.json |
| python -u tools/test_m5.py | exit0; 25 groepen/20 VM | suite.json,guest-runs.json |
| python -u tools/test_milestones.py --through m4 | exit0; M2 8/7, M3 22/19, M4 12/10 groepen/VM | regression/{m2,m3,m4}/ |
| python -u tools/test.py | exit0; 10 groepen/8 VM | regression/m1/ |
| Vijf capture_gate.py-opdrachten | exit0; raw/serial/stderr/artifacthashes en M5-schijforacles hercontroleerd | genoemde rapportmappen |
| Normale M5-build + run.py --smoke --milestone m5 | exit0/raw33; extra eindstart | manual-m5.json,final-build-manifest.json |

Definitieve suites: **77 controlegroepen en 64 echte VM-runs**, plus één normale M5-eindstart. Eerdere diagnostische runs zijn niet opgeteld. M1-negatieve guestrecords blijven terecht passed:false; de suite controleert ontbrekende/corrupte kernel, panic en echte 60s-time-out.

M5-hostlaag: 62 Rust-tests (26 capability, 15 kernel-core, 12 virtio, 9 protocol), 11 ELF-artifacttests, 4 harnasparsertests en 5 negatieve schijf-oracletests. Alle 21 eerdere M4-capabilitytests zijn behouden. Extra corecontroles fmt/Clippy-Dwarnings/no_std exit0; exacte scope/argv in core/report.json. Hosttests bewijzen op zichzelf geen device-I/O.

Alle definitieve guestrecords hebben fingerprint v3 c7455ebe2dfcd0e4337d058f440712d60b3ae600d2953c385ee81ba6d69a0307. Twee normale M5-runs gebruiken dezelfde kernel/loader en verschillende echte images; ontvangen sectorchecksums kloppen en de volledige image-SHA256 blijft gelijk. FNV is diagnostiek, geen cryptografie; de guest kent geen seed/fixtureantwoord. Exacte opdrachten/gewijzigde bronbestanden: validation.json. Testmatrix: ACCEPTANCE.md.

## Starten

Vanuit de projectroot in PowerShell:

```powershell
python tools/build.py --kernel-feature milestone-m5
python tools/run.py --smoke --milestone m5
```

De runner maakt zelf een nieuwe alleen-lezen 64KiB-image onder build/run/. De eindige proef leest via de dienst en sluit QEMU na de assertions af. De laatste lokale build staat op het normale M5-profiel. Testsuites herstellen na foutvarianten M1; bouw bij hervatten expliciet het gewenste profiel. Zie DEVELOPMENT.md, drivers/virtio-blk/README.md, services/block/README.md en userspace/M5-PROBE.md.

## Review en grenzen

Herstelreviews: transport-review.md en service/user-review.md. De verifier controleerde alle 65 definitieve VM-runs,279 artifacthashes en werkelijke M5-schijfbytes: m5-verifier-guest.json en m5-verifier-final.json. Finale kernel/loader zijn gelijk aan beide normale gate-runs en de laatste CLI-start. Dit zijn AI-reviews, geen menselijke audit.

Echte tests vonden en herstelden opnieuw resetten tijdens quarantainecleanup en een verkeerde sectorscope-foutcode. De eerste gecombineerde gate bleef rood; failed-records blijven als failed bewaard onder diagnostics/. Andere herstelpunten: BME-clear/reset na initfout, device-id-binding, terminale PID-resultaten, onnodige clientautoriteit, CPU-adresbreedte en harde observatie van alle 13 protocolfouten. DRIVER_OK-readbackfout, device1-scope en exit na resetack vóór foutpoll zijn bronfixes zonder aparte gastinjectie; aanvullende testmogelijkheden.

Kerneltransport en protocolconform QEMU-device/backend zijn vertrouwd. Geen IOMMU, containment van kwaadwillige DMA, SMP- of fysieke-hardwareclaim. Pool blijft gepind; zonder resetack in quarantaine tot VM-stop. Injecties zijn test-only: lokale snapshots na echte completion of suppress-notify/mask-ack. Geen Secure Boot/PQC/productiegeschiktheidsclaim.

Materiaal is browserprototype; qsim is hostsimulator. 9 UI- en 15 quantum-bron-/binaryhashes opnieuw gelijk; eerdere resultaten blijven geldig voor die bestanden, tests niet herhaald. 200%-zoom, schermlezer/menselijke proeven, native UI, OS-quantumport, QPU en OS-afgedwongen lokale AI/NAS-grenzen blijven open.

Geen hostdisk, firmware, NAS, productiekeys, globale Codexconfig of betaalde infrastructuur gewijzigd. Bestaande .codex/environments/ behouden buiten Git. Licentiekeuze vóór publieke publicatie blijft volgens LICENSE-PLAN.md open.

## Gerichte M5-reviewreparaties — 27 oktober 2026

Op basis `448974b1352493fba41158da08daf1ad1937d973`: MMIO-/DMA-testadressen worden uitsluitend aan attackerrol 3 verstrekt in respectievelijk case 14/15; dienst, client, supervisor en PCI-case 16 krijgen ze niet. De succesvolle responsecontrole begrenst lengte, sector en slice vóór bytevergelijking en meldt afwijkingen via `s.unexpected` aan de bestaande `M5:FAIL:USER_OR_PROTOCOL`-afhandeling. ADR-026 heeft primaire bronnen en ontbrekende templatevelden; de capability-README verwijst naar de ingecheckte corekopie. Verouderde `core/tests.txt` en `core/clippy.txt` zijn verwijderd; actuele bewijsbestanden zijn behouden.

Op deze reparaties opnieuw uitgevoerd in Ubuntu 26.04.1 met Rust 1.90.0/QEMU 10.2.1/OVMF 2025.11: `python3 -u tools/test_milestones.py --through m5` en `python3 -u tools/test.py`, beide exit0; samen 77 controlegroepen/64 VM-runs. De 11 hostharnesstests slagen; beide nieuwe regressiegroepen falen terecht tegen de oude bron. Twee aanvullende echte gastproeven met responslengte 0 en `u32::MAX` eindigen gecontroleerd met `M5:FAIL:USER_OR_PROTOCOL`/raw35, zonder panic; de synthetische images blijven bytegelijk. Inventaris, kernel-rustfmt, documentatie-/diffcontroles en de onafhankelijke gerichte codecontrole zijn afgerond. Exacte opdrachten, bron-/artifacthashes en bewijsgrenzen: [review-repairs.json](verification/m5/review-repairs.json). Het eerdere Windowsbewijs hierboven is behouden. Geen contract- of architectuurwijziging.

## Vervolg en herstel

M6a: begrensde WRITE/FLUSH en crashherstel op een synthetische wegwerpimage; de actieve status en nog uit te voeren volledige poort staan hierboven. Werkqueue: WORK-QUEUE.md. Contract: implementation/M6A-WRITE-RECOVERY.md. M1–M5-baseline en eerder bewijs blijven behouden; de volledige QEMU-poort is na deze merge niet opnieuw uitgevoerd. Herstel via aparte consistente checkout c630c59; geen destructieve reset of hostconfigmigratie.
