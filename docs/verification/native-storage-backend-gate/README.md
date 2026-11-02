# Native backend- en herstelbaarheidstoets — Btrfs en OpenZFS

**Uitgevoerd: geen van beide kandidaten is toegelaten als native backend van
Nieuw OS; bestaande NOSVOL02-controles slagen, hostherstel levert afzonderlijke
positieve resultaten én concrete beperkingen.**

Datum: 2 november 2026, Europe/Amsterdam. Eerste broncommit:
`e08fb0e48c1bb0781e98357f588c2f73b6ae8500`; bevestigende herproef op actuele
main `74a34d05ef5591b782a0da9da34e4669910d0722`. De kernel en drivers zijn niet
gewijzigd. Dit bewijs betreft uitsluitend synthetische regular-file-images,
opnieuw gebouwde eigen-kernelbinaries, koude QEMU-boots en bestaande offline
hostengines. Geen fysieke schijven, kernelmodules of systeemwijde installaties.

## Besluit van deze toets

- Native backendadoptie: **NIET GEACCEPTEERD**, voor Btrfs en OpenZFS.
- Native bestandsherstel uit deze formaten: **NIET AANGETOOND**.
- Bestaande eigen-volumecontrole: **GESLAAGD**, normaal en koud herstel.
- Veilige weigering van ieder vreemd formaat: **NIET GEACCEPTEERD** in de
  bestaande synthetische volumeproef: een Btrfs-prefix kan als leeg gelden.
- Herstelcontrole moet inhoud en vereiste metadata onafhankelijk controleren;
  een exitcode 0 is niet voldoende.
- Strategie B blijft onderzoeksrichting. Geen nieuw diskformaat, driverport,
  Linux-onderlaag of productie-backendkeuze ingevoerd.

De vaste productscore blijft **34/100 → 34/100**. S09, back-up/herstel/export,
blijft open. De beperkte vreemde-formaatproef verandert de kernel niet en
is geen nieuwe acceptatie van S01–S08 of algemene externe-schijfondersteuning.
Er is geen verlies van bestaande NOSVOL02-controledata waargenomen.

## Native uitvoering op de eigen kernel

Rust 1.90.0; targets `x86_64-unknown-none` en `x86_64-unknown-uefi`.
QEMU 11.1.0, `pc-q35-10.0`, TCG, één CPU, 256 MiB RAM. Nieuwe firmwarestatus
per run, geen netwerk, geen oude previewbinary. Bestaande locked dependencies
zijn zo nodig uitsluitend naar een lokale buildcache opgehaald; lockfiles
zijn bytegelijk gebleven. De uiteindelijke builds waren offline.

| Run | Raw QEMU-exit | Waarneming |
|---|---:|---|
| Eigen NOSVOL02 normaal | 33 | PASS; onafhankelijke schijforakelgeneratie 84 |
| Eigen NOSVOL02 koude recovery | 33 | PASS; exacte normale eindimage, dezelfde generatie en beide domeindigests |
| Btrfs 256 MiB, readonly | 35 | APP_RESULT bij stap 20; geen Btrfs-document gelezen |
| OpenZFS mirrorlid 256 MiB, readonly | 35 | SERVICE_STARTUP, Corrupt (-18); geen poolimport |

Beide vreemde bronimages én hun proefkopieën hebben voor/na de VM dezelfde
volledige SHA256. QEMU-datadrive `readonly=on`, READ-only guestcapabilities en
`-nic none` zijn gecontroleerd. Alle vier runs bereikten de bootmarkers en
bleven binnen hun timeout. [Native samenvatting](native/N1-result.json),
[exacte uitvoering en reproduceerroute](native/README.md),
[runner-snapshot](native/native_probe.py).

De Btrfs-proef is geen geslaagde formaatafwijzing. De bestaande volumedienst
leest 68 tabelsectoren vanaf fysieke sector 4. Twee nultabellen gelden in
`services/volume/src/table.rs:828-846` als `Table::EMPTY`. De echte Btrfs-image
bevatte daar nulbytes. Startup ging daardoor verder; pas het ontbrekende
verwachte NOSVOL02-document liet de app falen. Dit is gecombineerd native
bewijs, headerinspectie en bronlezing. Er is geen Btrfs-parser uitgevoerd.

Dit betreft een als vertrouwde testdisk aangeboden fixture, geen algemene
productautoformatter. Geen fysieke of writable vreemde schijf is getest.
Voordat deze route generiek wordt gebruikt, is positieve formaatidentificatie
nodig; een lege onderzochte prefix is geen toestemming om een bestaand volume
als nieuw te gebruiken. De parser is in deze opdracht niet gerepareerd.

De OpenZFS-proef bood één lid van een twee-bestandsmirror aan de bestaande
single-devicebroker aan. Die proef bewijst omgang met vreemde bytes, geen
native multi-vdevimport. De hostpool bevat DMU-testobjectsets, geen normale
ZPL-bestandsboom. Native map-/bestandsherstel is voor deze input niet getest.

## Herproef na nieuwe main-integratie

Tijdens afronding bevatte remote main integratie #125. De bewijsbranch heeft
alle upstreaminhoud behouden en uitsluitend documentatieconflicten opgelost.
De vier native proeven zijn opnieuw uitgevoerd op een aparte schone source-
worktree van `74a34d05ef5591b782a0da9da34e4669910d0722`, met opnieuw gebouwde
binaries en fingerprint
`8fd739dd015b9d01908ccc0e385f5b584985cade44702f6e6662483f4be0b6d3`.

Normal en cold recover slagen opnieuw (exit 33, 62,72 s en 32,52 s).
Btrfs herhaalt de nul-header/appstap-20-uitkomst; het OpenZFS-mirrorlid wordt
weer met Corrupt (-18) geweigerd (beide exit 35). Geen timeout; bronworktree
voor/na schoon; bron- en proefimagehashes onveranderd. De eigen eindimage en
koude kopie zijn exact gelijk, generatie 84 en beide domeindigests matchen.

Dit zijn **acht echte VM-runs over twee bronversies**. De oorspronkelijke
bewijzen zijn niet herlabeld of vervangen. [Actuele native samenvatting](native-current/N1-result.json)
en [uitvoering](native-current/README.md) bewaren de tweede bewijsset. Hosttools,
hostimages en de begrensde hostconclusies zijn ongewijzigd. De PR-diff ten
opzichte van de nieuwe main bevat geen runtimebron- of driverwijziging.

## Btrfs: offline herstel en corruptie

Ubuntu 26.04.1/WSL2, Linux `6.18.54.1-microsoft-standard-WSL2`;
`btrfs-progs 6.17.1-1build1`, lokaal uit officieel Ubuntu-pakket uitgepakt.
Geen mount, loopdevice of Btrfs-kerneldriver gebruikt. Image 256 MiB,
CRC32c, data=single, metadata/system=DUP, 4 KiB sectoren en 16 KiB nodes.

| Geval | Checker-exit | Restore-exit | Onafhankelijke vergelijking |
|---|---:|---:|---|
| Gezond | 0 | 0 | 132 bestanden en één symlink inhoudelijk exact |
| Primair superblock gewist | 1; reservesuperblock 0 | 0 | Alle 133 items inhoudelijk exact |
| Eén bit in payload gewijzigd | 1, checksummismatch | **0** | Eén bestand corrupt; exact byte 12345/XOR 1 |
| Afgekapt naar 32 MiB | 1 | 1 | Geen volledig herstel |

Het reserve-superblockgeval toont herstel van deze specifieke beschadiging.
De dataproef toont foutdetectie maar geen self-healing: data=single heeft geen
tweede payloadkopie. `btrfs restore` kopieerde de corrupte file zonder melding
en met succesexit. De herstelbevestiging moet dus op een onafhankelijk
integriteitsmanifest steunen. Alle readonly bronimages bleven bytegelijk.
[Resultaten](btrfs/evidence.json), [commando's](btrfs/commands.json),
[corpusmanifest](btrfs/expected-corpus.json), [toelichting](btrfs/README.md).

De eerste restore had een nog niet bestaande doelmap en faalde toen een
symlink als eerste werd verwerkt. Die mislukking is behouden. Alleen de
doelmapvoorbereiding is daarna gecorrigeerd; dit geldt niet als FS-fout.

### Metadata: import en salvage apart gecontroleerd

Een aparte nieuwe image bevat acht padvermeldingen met gecontroleerde modes,
mtime-nanoseconden, user-xattrs, lege map, hardlinks en sparse inhoud.
`dump-tree` onderscheidt verlies tijdens `mkfs --rootdir` van verlies tijdens
`restore -m -x -S`. Alle toolcommando's gaven exit 0; de bron bleef bytegelijk.

| Eigenschap | Image na import | Na herstel |
|---|---|---|
| Gewone filemode/user-xattr, inhoud en symlinktarget | Behouden | Behouden in dit corpus |
| Nanoseconden in mtime | Al op hele seconden afgerond | Fractie ontbreekt |
| Sparse 8 MiB file | Al volledig gealloceerd | Inhoud juist, holes niet behouden |
| Hardlinkpaar | Eén inode, twee links | Twee losse bestanden |
| Lege map: mode0700/xattr/oude mtime | In image aanwezig | 0755, xattr weg, nieuwe mtime |

Dit zijn beperkingen van deze import-/salvageroute, geen bewezen beperking
van het Btrfs-diskformaat. ACLs, security-xattrs, gebruikersremapping, ADS en
resource forks zijn niet getest. [Metadataresultaat](btrfs/metadata/metadata-evidence.json),
[nadere afbakening](btrfs/metadata/README.md).

## OpenZFS: userspace-engine, import en crashproef

De exacte resultaten, resourcebegrenzing, toolversies en mislukte preflights
staan in [OpenZFS-resultaat](openzfs/evidence.json) en [toelichting](openzfs/README.md).
OpenZFS 2.4.1-1ubuntu5.1 is lokaal uitgepakt; `ztest` gebruikt de bestaande
`libzpool`-engine op regular-file-vdevs. Geen `/dev/zfs`, module of mount
aangemaakt. De standaard meteen ge-unlinkte `/tmp/ztest.data.XXXXXX`-runtimefile
is als begrensde uitzondering vastgelegd.

- Een normale echte engineproef slaagt; een nieuw `zdb`-proces kan de
  quiescente pool importeren en controleren. Dit is hostpool-/objectbewijs.
- `ztest -E` faalt bij pool discovery: de hostactive-check verwacht `/dev/zfs`.
  Dit is een brononderbouwde verklaring die past bij de waarneming; er is geen
  syscalltrace vastgelegd. `zdb -e` gebruikt een andere importoptie en slaagt. De tool is niet gepatcht
  en de mislukking is niet vervangen door een fictieve geslaagde schrijfrecovery.
- Een afzonderlijke create/crashproef voert drie SIGKILLs uit. Ondanks
  `ztest`-exit 0 meldt `zdb` een allocatieverschil/leak van 9216 bytes.
  De strenge no-leak-herstelvoorwaarde is niet gehaald; oorzaak en betekenis
  voor deze userspace-harnasconfiguratie zijn niet opgehelderd. Dit is geen
  aangetoonde productie-kernel-ZFS-bug.
- Een raw exportpoging op de versleutelde DMU-testdata abortte en leverde
  geen bruikbare backup. Normale bestanden, namen, mappen en metadata
  terugzetten zijn dus niet aangetoond.

Een proceskill is geen fysieke powercut. De twee kandidaten gebruiken
verschillende workloads; tijden/RSS zijn uitvoeringsgegevens, geen eerlijke
prestatie- of minimum-RAMvergelijking. Persistent self-healing, native
encryptie/snapshots, volledige back-uprestore en SSD-slijtage zijn niet
geaccepteerd door deze proef.

## Controle, reproductie en herstel

Volledige lokale bewijzen staan onder
`C:/Users/simon/Documents/Codex/2026-11-02/native-storage-gate/`.
Deze repositorybundel bewaart compacte tekst, runner-snapshots, commando's,
exitcodes en hashes; geen binaries, packages, caches of schijfimages.
`archive-manifest.json` verbindt compacte bestanden met hun oorspronkelijke
SHA256. Herhaalde per-file-manifestvelden zijn alleen in de compacte
Btrfs-samenvatting verwijderd; het corpusmanifest is apart bewaard.
De subdossiers zijn snapshots van de oorspronkelijke lokale proefmappen.
Paden naar images, packages, sources en licenses daarin verwijzen naar die
lokale bron; die omvangrijke binaire artefacten zijn bewust niet opgenomen.
Een mirrorlid kan in principe alle mirrordata dragen; zelfstandige degraded
import van één lid is in deze toets niet onderzocht.

Een afzonderlijke verificatie controleert artefacthashes, read-only/datadrive-
grenzen, cold-imagegelijkheid en de verwachte negatieve uitkomsten. Het is
bewijsintegriteit, geen extra OS- of filesystemacceptatie. Mislukte
preflights/parserruns zijn in de oorspronkelijke logs behouden. Exacte
reproduceercommando's staan bij iedere subproef; kies voor herhaling een
nieuwe eigen wegwerpmappenstructuur en controleer de toolhelp van de
vastgepinde versie.

De hoofdcheckout is ongewijzigd, inclusief de al aanwezige untracked
`.github/github-app.yml`. Alleen deze bewijsbranch wijzigt documentatie en
voortgangsassessment. Herstel is de branch niet integreren of de bewijscommit
reverten; later kunnen uitsluitend de genoemde wegwerpartifacten worden
opgeruimd. Er is geen hostdisk-/firmwareconfiguratie terug te draaien.

## Bronnen en nog open portabiliteit

Geraadpleegd op 2 november 2026. De flags zijn aan de werkelijk uitgevoerde
toolversies getoetst; latest/master-handleidingen vervangen die pin niet.

- [Btrfs mkfs](https://btrfs.readthedocs.io/en/latest/mkfs.btrfs.html)
- [Btrfs check](https://btrfs.readthedocs.io/en/latest/btrfs-check.html)
- [Btrfs restore](https://btrfs.readthedocs.io/en/latest/btrfs-restore.html)
- [OpenZFS ztest](https://openzfs.github.io/openzfs-docs/man/master/1/ztest.1.html)
- [OpenZFS zdb](https://openzfs.github.io/openzfs-docs/man/master/8/zdb.8.html)
- [OpenZFS-platformlaag](https://openzfs.github.io/openzfs-docs/Developer%20Resources/Building%20ZFS.html)

De runtimebron en Cargo-lock bevatten geen Btrfs-/OpenZFS-engine. Hun bestaande
Linux-/libc/SPL-interfaces passen niet rechtstreeks op de huidige bare-metal
app-ABI en begrensde blokcapabilities. Dat vraagt een afzonderlijke bestaande-
engineport met licentie-, API-, resource- en herstelreview. Er is geen
native engine gecompileerd of geporteerd: de eigen kernel is alleen met zijn
al bestaande profielen herbouwd. De volgende stap is zo'n begrensde
portabiliteitsbeslissing, niet een nieuw bestandssysteem ontwerpen.
