# Native PNG/JPEG-integratiegate

IMAGE-HARNESS-1, basis `4b638568e4dee62ac35f2a8f6407d7b05c7b770d`.
Deze gate bindt de echte ring3-imageprobe aan de oorspronkelijke geïntegreerde
systeem-/schijfketen. Hij voert geen hostdecoder uit en verandert geen
PNG/JPEG-grens om een groen resultaat te verkrijgen. De
[image-fixture](NATIVE-IMAGE-FIXTURE.md),
[codeccomponent](../implementation/NATIVE-IMAGES.md) en
[FULL-TEST-RELEASE](../FULL-TEST-RELEASE.md) blijven de inhoudelijke contracten.

## Profiel en uitvoering

Het enige profiel is `native-system-images`, de CLI-case heet `normal`.
De root levert de bestaande vier programma's plus een vijfde initiële,
capabilityloze imageprobe, de opt-in geheugen/FPU/klokfuncties, de werkelijke
bootstrap en de gedeelde build-/runhooks. De app vraagt zelf1024 verse eigen
heappagina's aan en draait met de bestaande64KiB-stack. Het harnas levert
geen pixels, heap of succesvolle decode-uitkomst.

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B tools/dev.py python3 -B tools/build.py --kernel-feature native-system-images
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B tools/dev.py python3 -B tools/run_images.py --case normal
```

Gebruik afzonderlijke `NOS_BUILD_DIR`/`NOS_TARGET_DIR` binnen de checkout en
het vastgelegde QEMU-/OVMF-profiel. De gate gebruikt de bestaande guarded
`run_system`/`run_volume.execute`: TCG, één qemu64-CPU,256MiB,geen netwerk of
hostshare, geen snapshot, één nieuwe lege1MiB-wegwerpschijf en de bestaande
virtio-rng voor de Key Service. De imageprobe krijgt geen opslag-, display-,
netwerk-, sleutel- of entropycapability. Er is geen optie om een eigen schijf
of bestaand volume te kiezen. Timeout is expliciet begrensd tot180seconden.

Deze gate is een functionele proef. Gelijktijdige TCG-runs op gedeelde host-CPU
leveren geen vergelijkbare prestatieresultaten. De bestaande FR-I01-normal→
koude readonly-recovery blijft een apart verplicht regressiebewijs; er wordt
geen image-recoveryprofiel of duurzame pixelsnapshot verondersteld.

## Verplicht native bewijs

De oorspronkelijke boot-, proceslifecycle-, privilege-, record-, cleanup- en
schijfasserties blijven actief, inclusief2seals/2unseals/9authweigeringen en de
4096exacte documentbytes. De werkelijke QEMU-exit moet33 zijn en timeoutfalse.
Een ontbrekende probe, fault, panic, onverwachte exit of alleen een PASS-vlag
maakt de gate niet groen.

Vóór het uitvoeren van de probe print de rootbootstrap één twintigcijferig
`SYSTEM:IMAGE_MANIFEST_PID:<actualpid>`, direct na `image_fixture::bind` en vóór
`SYSTEM:MANIFEST:PASS`. De finalewaarneming moet precies hetzelfde echte PID
melden. Het harnas neemt PID5 of de latere gebruiker van slot4 niet als identiteit.
De kernelobserver bewaart PID/bootslot, echte launch-/exitticks en een snapshot
vóór cleanup; hij controleert ook eigenaarschap, RW/NX en mappings/guards.

Daarna zijn precies de20 vastgelegde `SYSTEM:IMAGE_*`-velden in observervolgorde
en één `SYSTEM:IMAGES:PASS` vóór `SYSTEM:BROKERS:PASS` vereist. Alle tellers
zijn exact20decimale cijfers binnen u64. Onbekende, dubbele, ontbrekende,
verkeerd geordende of overlopende velden zijn fouten. De vereiste inhoud is:

| Gegeven | Eis |
|---|---|
| PNG/JPEG |8/3, inclusief succesvolle herdecode na fouten |
| Pixels / samplebytes |149647 /623808 |
| Negatieve uitkomsten / vooraf annuleren |10 /1 |
| Pixelpariteit |548683896226622996 |
| Appgroei / totale kernelheap |1024 /1040pagina's |
| Arenapeak / liveallocaties |werkelijk>0 en≤4194304bytes /0 |
| Stack / guards |16pagina's /1; bestaande64KiB-stack |
| Klok |start≤einde≤exit; exit−start<20000echte ticks |
| Specifieke JPEG-vraag |QUALITY_OPEN=0 voor de onderzochte, herstelde fixture |
| Linked ELF |werkelijke lengte en FNV tegenover het opnieuw gelezen gebouwde ELF |

De echte entropybroker moet de oorspronkelijke1024bytes rapporteren; deze
imageprobe mag geen extra sleutel-/RNG-operaties veroorzaken. Algemeen JPEG-
kwaliteitsbewijs is niet gelijk aan dezelfde-enginepixelpariteit. De concrete
kleuren-/PNG-sampleproeven en de oorspronkelijke rode JPEG-uitkomst blijven
in het codeccontract behouden.

## Artifactbinding en resultaat

`manifest.system` bevat precies `system_init`, `system_app`, `system_worker`,
`keyring_service`, `volume_service` en `image_probe`; de aanvullende
`manifest.image_probe` moet hetzelfde artifact aanwijzen. De builder gebruikt
`TARGET/system-image-probe/x86_64-unknown-none/release/image-probe` en levert
`NOS_IMAGE_PROBE_ELF` aan de kernel. De overige vijf paden zijn de bestaande
systeembuildtargets.

De gate controleert bronvingerafdrukversie3, actuele feature en manifest,
SHA256 en nieuwe inspectie van alle zes echte ELFbestanden, en aanwezigheid
van hun volledige bytes in de geladen kernel. Hij vergelijkt build-ESP én
werkelijke QEMU-ESP met de kernel-/loaderhashes, zonder extra bestanden of
links. De gelinkte ELF-FNV is uitsluitend een parity-/driftfingerprint;
SHA256 bindt de exacte buildbytes. **FNV verleent geen authenticiteit.**

De opgeslagen serial wordt opnieuw gelezen en streng geparseerd. De generieke
systeemparser herkent het cijferdeel in `IMAGE_ELF_FNV64` niet; dit harnas
parseert dat veld zelf en registreert de volledige imagewaarden afzonderlijk
als `image_evidence`. Bestaande `evidence` blijft gelijk aan de opnieuw
geëvalueerde oorspronkelijke systeemgegevens. Oude of gewijzigde gecachte
image-/systeemtellers mogen de werkelijke bytes niet vervangen.

Het commandoprofiel, de QEMU-executablehash, de geselecteerde OVMF-code en
preboot-VARStemplatehash moeten overeenkomen en aan het einde onveranderd
zijn. Werkelijk writableUEFI-state wordt beheerd gelezen, niet onjuist met
zijn oude bytes gelijkgesteld. Een snapshot-/netwerk-/anderecommandoregel
wordt niet geaccepteerd als dezelfde proef.

De onafhankelijke diskparser controleert de livewegwerpimage én identieke
postrun-snapshot: het nieuwe volume begon leeg; het oorspronkelijke document
heet `systeemproef`, bevat4096exacte fixturebytes op revisie1 en SAVErequestID3.
Geschreven extra objecten, collecties, onjuiste canaries of gewijzigde content
kunnen niet door een opgeslagen oracle-succes worden afgedekt.

Symlinks, specialfiles en hardlinked bewijs-/outputbestanden worden geweigerd.
Alleen read-only consumptie van echte Cargo-uitvoer laat diens interne hardlinks
toe. Publicatie gebruikt nieuwe inodes en atomicreplace binnen eigen paden.
De aanvankelijk onware `image_gate` wordt pas na alle controles waar. Iedere
extra fout maakt ook `passed=False`. Resultaten staan onder de eigen buildmap
als `image-report.json` en bij de werkelijke run als `result.json`; het
`last-system-stage.json` met deze andere feature autoriseert geen gewone
FR-I01-coldrecovery. Deze registratie is geen ondertekende attestatie tegen een
kwaadwillende host die alle bron, binaries en bewijsbestanden vervalst.

De actuele `image-report.json` wordt vóór verdere preflight en VM-delegatie
atomair vervangen door `status=starting`, `passed=False`, `image_gate=False`.
Een preflight-/launch-exceptie schrijft `status=failed` met onware flags en wordt
doorgegeven; een ouder groen indexrapport kan daardoor geen nieuwe mislukking
verbergen. Historische per-runresultaten blijven behouden. Onveilig gelinkte
rapportdoelen worden vóór publicatie geweigerd zonder externe bytes te wijzigen.
Vóór sharedcopy/delegatie wordt de bron-ESP zelf op exact de twee gehashte files,
alle voorouders en afwezigheid van links/specialfiles gecontroleerd. De latere
controle van de echte guest-ESP blijft verplicht en vervangt deze vroege grens niet.

## Hosttegenproeven en open acceptatie

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B tools/dev.py python3 -B -m unittest discover -s tools/tests -p test_images.py
```

De expliciet synthetische hostfixtures toetsen alleen het harnas: PIDmix,
verkeerde bootstrapvolgorde, fout/timeout met PASS-vlag, andere pixel-/heap-/
stack-/klokcijfers, onjuiste ELF-FNV/lengte/SHA/inspectie, profiel-/bronfouten,
snapshot-/netwerkcommando's, links, gewijzigde serial/schijf en verouderde
oracle-resultaten. Zij draaien geen beelddecoder en vormen geen native log.

Echte root-VM-uitvoering op de bevroren integratiebron en bestaande gezamenlijke
koude regressies blijven verplicht. Algemene viewer-/formaatacceptatie, object-
RPC, Materiaal-/compositorintegratie, zoom/pan, mid-decodecancel, echte native
allocator-OOM/herstart en volledige memory-safety-/kwaliteitsaudit blijven open.
Een geslaagde sampleproef op64KiB-stack is geen maximum-stackgarantie voor alle
invoer. Deze gate tekent FR-A03 of de volledige testrelease niet zelfstandig af.
