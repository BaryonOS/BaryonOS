# R06 — gerichte read-only harnasreview

**Delta-review: de vier gemelde bevindingen zijn bronmatig gesloten. De
oorspronkelijke tekst blijft als reviewgeschiedenis staan; zie het addendum
onderaan. De latere suite-orchestrator en definitieve gastgate blijven apart.**

Basis `0d33990001c2c974a19a6b265a410f210d0a641d`, 27 oktober 2026.
Scope: tools/m6_disk.py, tools/run_m6.py, tests/test_m6_disk.py,
tests/test_m6_harness.py; uitsluitend de is_success-helper in tools/run.py
als directe afhankelijkheid. Geen brede kernel-/driverreview.

Geen bronwrites, builds, hosttests of QEMU uitgevoerd door deze reviewer.
Parent rapporteert dertien geslaagde hosttests; de bron van die acht oracle-
en vijf childproces-tests is gelezen. Hun strings zijn expliciet hostbewijs,
geen guest-/deviceproef. tools/test_m6.py bestond tijdens deze review nog niet;
de volledige orchestrator voor cold recovery is nog apart te beoordelen.

## Concrete bevindingen, direct aan root gemeld

1. **Recovery kan stil een nieuwe image initialiseren.** run_m6.py:142 maakt
   bij volume=None ook voor case recover een nieuw volume. Dus
   run_m6('recover') en CLI --case recover kunnen een groene nieuwe-VM-recovery
   opleveren zonder het volume uit een eerdere stage. Voorkeur: recover vereist
   het expliciete Volume-object uit de voorafgaande stage; ontbrekend object
   faalt vóór create_volume/QEMU. Geen algemeen padargument toevoegen.
   Test: recover zonder object weigeren; met object identiteit/pad en before
   exact vergelijken met eerdere aftersnapshot.

2. **Toegestane gedeeltelijke tail wordt bij cold recovery afgewezen.**
   run_m6.py:201 kiest allow_torn voor Unknown/cuts2..4, maar niet voor recover.
   Na een toegestane partial-write kan dezelfde ongewijzigde image bij recover
   daarom falen op m6_disk.py:195: tail is niet volume.initial en ook geen
   compleet expected record. De eigenlijke hoogste geldige waarde is correct
   oud, en before==after kan tegelijk waar zijn. Behoud de aantoonbaar toegestane
   before-tail en eis bytegelijke recovery; niet repareren door herinitialisatie.
   Test: accepted crashsnapshot met prefix200/256, recovery GET oud en volledige
   image ongewijzigd; verandering tijdens recovery blijft afwijzen.

3. **Normale PASS-markers zijn niet uniek/volledig begrensd.** run_m6.py:203
   gebruikt run.is_success, dat alleen de vereiste exacte lijnen in volgorde
   zoekt. Tweemaal de complete normale PASS-sequence wordt nog geaccepteerd;
   een extra verkeerde CASE:OTHER:PASS eveneens. VMcuts zijn voor CUT-lijnen
   wel uniek gecontroleerd. Voeg een M6-eigen markercontrole toe die elke
   verplichte lijn exact eenmaal en alleen de verwachte CASE accepteert.
   Tests: duplicate BOOT/CASE/finale PASS, extra andere CASE, CUT in normale run
   en verkeerde volgorde afwijzen, normale unieke sequence accepteren.

4. **Full-oracle controleert niet de volledige committed prefix.**
   verify_volume slaat current==original per slot over. Slot0 geldig init plus
   correct slot3/gen4, met slot1/2 nog nul, voldoet daardoor aan allowed_gen4.
   Dit bewijst niet dat de eerdere PUT2/3-records behouden zijn. De kernelobserver
   kan aanvullend calls bewijzen, maar de externe full-imagecheck hoort voor
   case full alle vier verwachte records te eisen. Recovery blijft conform het
   Rustcontract hoogste-geldige-recordsemantiek gebruiken. Test: alleen gen4
   toevoegen faalt full-oracle; volledige verwachte prefix slaagt.

Dit zijn bronmatige concrete tegenvoorbeelden, geen door deze reviewer
uitgevoerde failing runs. Herstel en gerichte nieuwe tests zijn eigendom van
root; deze review verklaart de harnasgate nog niet afgerond.

## Correct begrensde onderdelen in de gelezen versie

- Onafhankelijke Python-parser correspondeert met Rust: magic NOSM6A01,
  little-endian versie1/header64, flags0, slotgebonden generatie, nonzero
  request-id, lengte128, reserved40..63/192..503nul en checksum504. FNV rekent
  over512 met de laatste8 bytes nul. Parser importeert geen Rustcodec.
- Verwachte waarden volgen uitsluitend uit de echte initiële128 bytes en de
  afgesproken wrapping transformatie. Een aangepast record moet exact het
  verwachte record zijn; scope/canaries en scratch512 worden bytegewijs bekeken.
  De geaccepteerde waarde komt uit een daadwerkelijk geldig geselecteerd record;
  guestmarkers moeten generation/request-id/digest exact daarmee overeenkomen.
- Prefix256-tornvariant is vóór boot expliciet geïnjecteerd en wordt op
  ongeldigheid gecontroleerd; corrupt-all houdt geen geldige initialisatie over.
  allow_torn laat voor slot1 alleen per-byte oude/nieuwe bytes toe en kan een
  ongeldige tail naast een oude geldige waarde accepteren. Het is geen algemene
  hardwaretear- of checksumcollisiongarantie.
- create_volume heeft geen bestaand-imagepad, valideert label/seed en maakt
  via xb een nieuwe64KiB-image onder build/run. Volume.guard controleert
  resolved directory, containment, symlink en matching identity/seed/init-hash/
  omvang. Dit is een lokale harnasgrens, geen bescherming tegen een kwaadwillige
  host die Pythonobjecten/metadata of hardlinks zelf vervalst.
- Bij expliciet doorgegeven Volume gebruikt run_m6 exact dat imagepad en
  initialiseert het niet opnieuw. Het bewaart daadwerkelijke before/aftercopies
  en hashes, en recover verplicht gehele-imagegelijkheid. Nieuwe ESP/firmware-
  kopieën veranderen die rawimage niet.
- De child wordt via eigen Popen-proceshandle beëindigd; geen PIDzoekactie of
  systeemwijd killcommando. Eerst50ms levend blijven, dan expliciet Windows
  exit0xE06A0001, daarna process.wait en pipe-drain. Zelfexit, normale33, timeout,
  verkeerde/duplicate CUT of PASS/FAIL bij de cut voldoen niet. ACCESS_DENIED-
  exitrace wordt niet tot een succesvolle eigen kill opgewaardeerd.
- De runner gebruikt cache=writeback en geen unsafe/no-flush-optie. De no-flush
  case is de contractueel test-only featuremask-injectie; readonly is echt
  readonly=on, no-device koppelt geen target. Bron/ESP/firmware/imagehashes en
  echte commandoregel worden bewaard. QEMUkill laat hostcaches bestaan en is
  geen powerloss- of fysiekehardwareclaim.

Microsofts primaire API-documentatie is gericht gelezen: TerminateProcess
neemt een proceshandle en expliciete exitcode, is voor een ander proces
asynchroon en vereist wachten voor beëindigingsbewijs; een beëindigd proces
kan ERROR_ACCESS_DENIED geven. Het gebruikte wait/exitcodepatroon sluit daarop
aan. Bron: https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminateprocess

Kleine auditmetadata-opmerking: het cutpad weigert ook M6A:PASS/M1:PASS, maar
report.forbidden_markers bewaart alleen FAILURES. Bewaar bij voorkeur exact
de werkelijk gebruikte forbidden-set in het cutrecord.

## Gelezen bronhashes

- tools/m6_disk.py: ca60e1fbc936c4b5ca4e83e82e6caeeed9a52ebceb6313677db0c29b2a7805f7
- tools/run_m6.py: e4a8d008a6b07a3e6eb3b66f9221c5a1f27488ed552d20c5d4d2ac1a10fe1fbc
- tests/test_m6_disk.py: 1e9dee72f6a0a6eb544464bbf276df69618c3e326c2f53e38b0842b5b757122d
- tests/test_m6_harness.py: e388b5a854eb3543834b716a3d28a6d25db24077929484bf1098b8f3d5d75314

Werkelijke reviewcommando's: gerichte Get-Content/rg op bovenstaande bestanden,
git rev-parse HEAD en Get-FileHash; geen testargv uitgevoerd. De verkenningscall
naar het toen nog ontbrekende tools/test_m6.py eindigde exit1; dat is alleen
een ontbrekend reviewbestand, geen uitgevoerde failing test.

## Delta-review van de vier herstellingen

Op dezelfde HEAD0d339900, met nog niet gecommitteerde rootdelta, uitsluitend
de afgesproken harnasfixes en vier nieuwe hosttests herlezen. Root rapporteert
zeventien daadwerkelijke hosttests exit0 (10 oracle,7 crash/parser). Deze
reviewer heeft geen tests opnieuw uitgevoerd en kent op basis hiervan geen
gast-/duurzaamheidsresultaat toe.

- **Punt1 gesloten:** run_m6('recover') weigert zonder expliciet Volume vóór
  create_volume. De CLI biedt recover niet meer aan. previous_stage vereist
  een bestaand last-stage.json, werkelijke result.json en aftersnapshot onder
  build/run. Resultaat moet passed=true zijn en een niet-recoverstage betreffen;
  exact volume-pad, identity, actuele imagehash en volledige snapshotbytes moeten
  overeenkomen. Mutatorreplay op een reeds gebruikt volume wordt geweigerd.
  De nieuwe test bewijst de no-previous-volume-afwijzing; identiteit/hash/
  snapshotguards zijn bronmatig gecontroleerd, nog geen eigen tegenvoorbeeldrun.
- **Punt2 gesloten:** recover accepteert dezelfde toegestane slot1-tail als
  de eerdere crash, maar verplicht before==after voor de hele image. De nieuwe
  oracle-test accepteert de bewaarde prefix200 en wijst een recoverymutatie af.
  De vorige geslaagde stage/snapshot is vooraf gebonden; geen nieuwe init.
- **Punt3 gesloten:** unique_markers projecteert alle gecontroleerde BOOT-,
  CASE-, progress-, resource-, final-PASS- en CUT-lijnen en vereist exact de
  verwachte lijst. Daardoor zijn extras, duplicates, suffixvarianten en verkeerde
  volgorde fout. Beide normale en gecrashte paden gebruiken deze controle.
  De nieuwe hosttest omvat normale unieke sequence, dubbele complete sequence,
  verkeerde CASE, PASS-suffix en CUT in normale run.
- **Punt4 gesloten:** require_full_prefix wordt alleen voor full geëist en
  vergelijkt alle vier complete records met de onafhankelijke expected bytes.
  De nieuwe test weigert uitsluitend slot3/gen4 en accepteert alle vier records.
  Hoogste-geldige-recordsemantiek voor recovery blijft behouden.

De nietblokkerende auditopmerking is eveneens hersteld: cutresultaten bewaren
M6A:PASS/M1:PASS in de werkelijk gebruikte forbidden-set. Serial/stderr worden
nu als de originele pipebytes met write_bytes bewaard; de tekstdecoder wordt
uitsluitend voor markeranalyse/JSON gebruikt.

userspace/M6-PROBE.md adverteert geen kale CLI-recoveryopdracht, dus geen
documentbronwijziging nodig. Userruntime en handleiding blijven bevroren.
tools/test_m6.py wordt later gericht op de echte stageketen beoordeeld.

Delta-bronhashes:

- tools/m6_disk.py: 5c9cecd953db4a74944710082da72c239746678c7a91b0723700265a3092a8bb
- tools/run_m6.py: 2ee9c730209d0d913e01bd44f70b60879d506fc0c234a634c374f7c278cea9fe
- tests/test_m6_disk.py: 75360e394839ccfa0b34ef272109788a8c9c1630e3c5f34908260c387313f14c
- tests/test_m6_harness.py: 3daf85d834c574f3c44c40a06f3c2b2cd27f0d15f11049089620e0be88e11abf

Binnen deze beperkte delta geen resterende concrete blocker. Geen nieuwe
tests/builds/QEMU of bronwrites; alleen dit genegeerde rapport bijgewerkt.

## Gerichte orchestrator-/capture-review

Alleen tools/test_m6.py en tools/capture_m6.py plus hun directe run_m6-
feature/oraclebinding herlezen. Geen tests/builds gestart. Gelezen hashes:

- test_m6.py: 43b6e69e7a19cd3213cebfe9d77cbd826987822b3afad8d3f43306362395d3d9
- capture_m6.py: 1fb3a70a110c49a73544fd2a99ef8848dce4f282bbd8268e0c19242a8e8f9ca6
- run_m6.py: 0ebac5b0d4df9f8be5d6f563eef3beff55a0af079697380fb5ba2e2647a9c281

### Correcte stageketen in de orchestrator

De dertig testfeatures worden daadwerkelijk aangeroepen: eerste zeventien
negatieven, zes servicecuts, zes VMcuts en recover. Normaal draait op twee
verschillende images met kernel-/loadergelijkheid en verschillende ontvangen
waarden; beide krijgen een aparte cold-recovery-VM. Full en de acht Unknown-
varianten krijgen eveneens koude recovery. Alle zes VMcuts krijgen een nieuwe
recovery-VM; zes servicecuts bewijzen interne userfault/herbinding. Daarna
build() plus gewone M1-run. Bronmatig totaal54 groepen/49 VM-runs inclusief M1;
dit is verwachte dekking, geen uitgevoerde telling.

Iedere run moet report.passed=true en de bevroren sourcehash hebben; een
falende groep schrijft passed=false en stopt de suite. Alleen na de laatste
bronhashcontrole wordt completed/passed=true gezet. Exceptpad houdt de suite
failed en probeert alleen M1 te herstellen; herstel wordt niet als succesvolle
M6-groep toegevoegd.

Recovery bouwt exact test-m6a-recover. run_m6 controleert kernel_feature,
loader_feature=None, huidige sourcehash en echte kernel-/loaderbytes vóór
QEMU. De recovery gebruikt het eerdere Volume-object en vergelijkt identity,
path, nieuwe invocatiemap, previous_stage, before==eerdere after en onveranderde
aftersnapshot. Geen nieuwe image of PUT-feature dient hierdoor als recoverybewijs.

Capture controleert echte rawfiles en hashes, alle artifactbytes, feature/
source/binarymanifest, opnieuw berekende journal-/canary-oracle, raw exits,
timeouts, unieke markers, expliciete Unknown en echte hostterminationmetadata.
Een individuele failed groep/run of onjuiste recoverylink wordt afgewezen.
M1 krijgt gewone baselinecontrole, zonder M6-markers of volume_path.

### Twee concrete capture-restpunten, aan root gemeld

1. **Suitevolledigheid niet onafhankelijk gecontroleerd.** Capture accepteert
   momenteel iedere subset van suite.results zolang status completed en passed
   true blijven. Zelfs een lege resultslijst wordt opgeslagen als captured.
   De orchestrator produceert de juiste reeks, maar capture hercontroleert
   niet de verplichte groepen/cases en zeventien specifieke cold-recoverylinks
   (2normaal+9full/Unknown+6VMcuts).
   Vereis exact de verplichte groepen en bijbehorende primaire/recoveryrecords;
   tel niet alleen passed-flags. Test verwijdering van een VMcut/pair/normal/M1
   en ontbrekende/dubbele groep als captureafwijzing.

2. **Tweebeeldbewijs niet opnieuw afgeleid.** m6a-two-image-check.json wordt
   als een willekeurig bestand gekopieerd, zonder passed/checks te valideren of
   de vier vergelijkingen opnieuw uit gevalideerde runrecords te berekenen.
   Een na de suite gewijzigd comparisonbestand met passed=false wordt dus toch
   captured. Herbereken same kernel/loader, verschillende initialimages en
   verschillende ontvangen waarden vanuit de twee echte normale records;
   controleer tevens dat het JSON-bewijs daarmee overeenkomt.

Deze punten zijn bronmatige tegenvoorbeelden, niet opnieuw uitgevoerde tests.
Root heeft al afzonderlijk aangekondigd bad-descriptor strenger op gen1 en
gehele-imagegelijkheid te controleren: Prepared-reject kan conservatief
Unknown zijn, maar mag geen publicatie of devicewijziging hebben. Dat bekende
pending herstel wordt hier niet als nieuwe bevinding dubbel gerapporteerd.

Geen uitbreiding naar driver/pagingbeleid of nieuwe userbron. Alleen dit
genegeerde reviewverslag gewijzigd. Vier eerdere harnasbevindingen blijven
gesloten; bovenstaande twee capturepunten wachten op herstel/delta-inspectie.

## Laatste capture-delta: beide restpunten gesloten

Op HEAD0d339900 met rootdelta alleen de twee afgesproken capturefixes,
de directe oraclebinding en bijbehorende test gelezen. Geen builds/tests/QEMU.
Root rapporteert negentien hosttests exit0; deze reviewer heeft die niet herhaald.

- required_groups bouwt de volledige contractreeks van54 groepen. validate_coverage
  vergelijkt de namenlijst exact inclusief volgorde, vereist49 verschillende
  runmappen, controleert per groep de echte case en per recovery de expliciete
  juiste parentmap. Ontbrekende/extra/dubbele/verplaatste groepen en ontbrekende
  VM-records worden geweigerd. De nieuwe hosttest omvat deze rejectgevallen.
  De individuele raw-/feature-/imagecontroles vinden vóór deze dekkingcontrole
  plaats, zodat een correct getelde lijst niet op zichzelf groen bewijs is.
- De twee normale runs worden uit de reeds gevalideerde records gekozen.
  Same kernel/loader, verschillende echte initialimages en verschillende
  ontvangen waarden worden opnieuw berekend. Alle vier moeten waar zijn;
  comparisonJSON.checks moet exact daarmee overeenkomen en passed moet true
  zijn. Het eerder beschreven false/stale vergelijkingrecord wordt afgewezen.
- De opnieuw berekende volledige disk_oracle moet nu gelijk zijn aan het
  opgeslagen object; een los aangepast opgeslagen digest/identity/boolean
  kan niet achter een enkele oude passed-flag worden verborgen. Loaderfeature
  moet ook bij capture None zijn. bad-descriptor eist generatie1 en volledige
  imagegelijkheid, ondanks het conservatieve appresultaat Unknown.

Laatste gelezen hashes:

- capture_m6.py: bb0fa9605deb16b056662a9fcfb90191a96cc26bff3f9d3c288f6fd48cfe61ee
- tests/test_m6_harness.py: 4aa84eaf3d655b74a60e015431bbc8b0a3e8ca7ce7f4dc54e27752e9f93ae93b
- run_m6.py: 1f00e6fd780431d05dd276280a47c20900d34fb576e61f97ca90937c11cf89a4

Alle zes harnas-/capturebevindingen zijn binnen deze gerichte review gesloten.
Geen resterend concreet tegenvoorbeeld binnen deze delta. De volledige
bevroren-bronpoort met werkelijke crash-/coldrecoveryrecords en regressies
blijft uit te voeren en onafhankelijk op de uiteindelijke artifacts te controleren.
Alleen dit genegeerde rapport is gewijzigd; userbron en handleiding blijven stil.
