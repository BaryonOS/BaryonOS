# M6c — finale acceptatie van namen, catalogus en hernoemen

28 oktober 2026, werkbranch `codex/m6c-named-catalog`,
[PR #10](https://github.com/simonvanieperen/nieuw-os/pull/10).
De UTC-uitvoeringslogs van de gates dateren van 27 oktober, 23:48–23:51.
Basis: `bb22fdbe15919edc23baa158a46236195ce314b2` (afgeronde M6b).
Uitgevoerde bron: `bc4cd104e86e2c047b727ae96e07a03fbc810c3c`.
De oorspronkelijke afsluiting wijzigde documentatie en bewijs. De latere
integratie van M6b-reviewherstel wijzigt hosttools en hun tests, met apart
[hostbewijs](m6c-review/merge-followup/README.md). De native runtimebron blijft
ongewijzigd. De fingerprint hieronder hoort bij de oorspronkelijke
uitvoeringsbron; zij wordt niet aan de inmiddels gewijzigde hosttools toegekend.

Fingerprint v3 van alle 141 bron-/bouwinputs:

```text
4468b3c18bd7fa57bf3631d634d52ee4c5fe9a76da9f125714d6db95e4407345
```

## Geleverd gedrag

Een eigen dependencyvrije `no_std`-catalogusdienst draait als gebruikersproces
op de eigen Rust-kernel. Twee apps krijgen ieder hun eigen capability-endpoint.
GET/PUT/DELETE blijven beschikbaar; PUT bewaart nu inhoud en UTF-8-label in één
record. RENAME vergelijkt de verwachte revisie, bewaart inhoud en verhoogt de
revisie. LIST geeft één snapshot van uitsluitend de eigen levende objecten.
Een verwijderd object houdt zijn revisie en verschijnt niet in de catalogus.
Namen verlenen geen rechten en zijn geen bestandspaden.

Alleen geslaagde WRITE- én FLUSH-completion mogen een mutatie bevestigen.
De serviceadapter blokkeert bij onzekerheid verdere geldige verzoeken; dit is
met de host-ABI-shim gecontroleerd. De bekende native observerbeperking onder
‘Grenzen en vervolg’ verhindert momenteel een gecontroleerde `Unknown`-reply
aan de client bij zulke devicefouten. Er is geen automatische herhaling van
writes. Full wordt vóór muterende I/O vastgesteld. Corruptie of een bezet oud M6b-v1-volume wordt vóór READY geweigerd,
zonder formatteren of migreren. De oorspronkelijke M6b-dienst blijft selecteerbaar.
Contract: [M6C-NAMED-CATALOG.md](../implementation/M6C-NAMED-CATALOG.md), ADR-030.

Gewijzigde implementatiegebieden: `services/catalog/`, nieuwe M6c-userspaceprobe
en linker, `kernel/src/m2/m3/m6c.rs`, expliciete feature-/ELF-selectie in de
gedeelde buildconfig, M6c-volume-/run-/test-/capturetools en bijbehorende
harnastests. De normale M1-parser weigert nu ook een M6c-fase als M1-resultaat.
Geen nieuwe externe afhankelijkheid of ruimere appbevoegdheid toegevoegd.

## Werkelijk uitgevoerde controles

Alle onderstaande opdrachten eindigden met **exit 0**. Deze scopes worden niet
opgeteld bij historische runs uit eerdere mijlpalen.

| Opdracht | Gecontroleerd resultaat | Bewijs |
|---|---|---|
| `python3 -B -u tools/test_m6c.py` | 19 groepen; 15 echte VM-invocaties: 14 M6c + 1 gewone M1 | [suite](m6c/suite.json), [gastrecords](m6c/guest-runs.json), [volledige console](m6c-console/gate.stdout.txt) |
| `python3 -B tools/capture_m6c.py` | Volledige gate, bronbinding, gastuitvoer, hashes en schijfuitkomsten opnieuw gevalideerd | [capture](m6c/capture.json) |
| `python3 -B -u tools/test_m6b.py` | 18 groepen; 14 echte VM-invocaties: 13 M6b + 1 gewone M1 | [regressiesuite](regression-m6c/m6b/suite.json), [gastrecords](regression-m6c/m6b/guest-runs.json) |
| M6b-capture met afzonderlijke archiefbestemming | Bestaande validator ongewijzigd; alle nieuwe M6b-regressiegegevens gevalideerd | [exacte opdracht](regression-m6c/m6b/archive-command.json), [capture](regression-m6c/m6b/capture.json) |
| `cargo +1.90.0 test --locked --offline -p catalog-core` | 30 Rust-tests; ook opnieuw uitgevoerd binnen de M6c-gate | [brongebonden corecommando's](m6c-review/core-recheck/results.json), [gatelog](m6c/m6c-host-tests.stdout.txt) |
| `cargo +1.90.0 fmt --package catalog-core -- --check` | Opmaakcontrole die geen bron wijzigt | [corecommando's en logs](m6c-review/core-recheck/) |
| `cargo +1.90.0 clippy --locked --offline -p catalog-core --all-targets -- -D warnings` | Geen resterende Clippy-waarschuwingen | [corecommando's en logs](m6c-review/core-recheck/) |
| `cargo +1.90.0 check --locked --offline -p catalog-core --target x86_64-unknown-none` | Core controleert voor het native target | [corecommando's en logs](m6c-review/core-recheck/) |
| Python M6c-harnastests binnen `test_m6c.py` | 33 tests voor schijforakel, childproces en capture | [exacte argv](m6c/m6c-harness-tests.json), [uitvoer](m6c/m6c-harness-tests.stderr.txt) |
| Echte serviceadapter met host-ABI-shim | 15 scenario's, inclusief WRITE/FLUSH-fouten en time-outs, identiteit, tombstones en legacy | [buildopdracht](m6c-review/adapter-build.json), [resultaat](m6c-review/adapter-result.json) |
| `python3 -B tools/build.py --kernel-feature milestone-m6c` | Afsluitende build bytegelijk aan beide normale eindgate-binaries; geen extra VM | [buildvergelijking](m6c-ready/build-check.json), [console](m6c-ready/m6c-ready-build.stdout.txt) |

Rust-opdrachten gebruiken de lokale toolchainomgeving uit `tools/common.py`;
`python3 -B tools/dev.py` kan deze omgeving aan een losse Cargo-opdracht geven.
Rust 1.90.0, QEMU 10.0.2, OVMF, q35/TCG, één CPU en 256 MiB zijn vastgelegd
in [native-linux-toolchain](native-linux-toolchain/). Native suites gebruiken één
gedeeld buildslot en zijn achtereenvolgens uitgevoerd. De M6b-regressie bevat
ook 22 bestaande object-coretests en 26 bestaande M6b-harnastests.

## Native uitkomsten

De twee normale M6c-runs gebruiken bytegelijke kernel/loader maar verschillende
canary-images. Zij eindigen op sequence 10 met tien mutatiebevestigingen en tien
catalogusantwoorden. Objecten 0 en 2 blijven leven op revisie 3; objecten 1 en 3
zijn verwijderrecords op revisie 2. Beide domeinen tonen uitsluitend hun eigen
namen en revisies. De foutproef test domeinoverstijgende GET/PUT/DELETE/RENAME/LIST
nadat beide domeinen werkelijk objecten bevatten; 80 afgewezen antwoorden
veroorzaken geen muterende I/O. De volle proef vult alle 32 records en controleert
dat een daarna geweigerde andere naam de bestaande naam niet overschrijft.

| Werkelijke externe onderbreking | Schijftoestand bij de volgende koude VM | Inhoud |
|---|---|---|
| Vóór de RENAME-WRITE | `Café A`, revisie 1 | oorspronkelijke 128 bytes |
| Na RENAME-FLUSH, vóór succesvolle reply | `Café hernoemd A`, revisie 2 | exact dezelfde 128 bytes |

Beide QEMU-processen zijn werkelijk van buitenaf beëindigd met SIGKILL, raw exit
−9, met vooraf gecontroleerde live-hold en zonder succesmarker. De koude herstelruns
hebben andere proces-ID's en exact dezelfde volume-identiteit. Alle zes koude
herstelparen behouden de volledige imagehash. De tweede cut toont ook waarom
een ontbrekende reply geen bewijs is dat de mutatie niet is uitgevoerd.

De corrupte fixture en een geldig bezet v1-record leiden tot weigering vóór READY,
zonder WRITE/FLUSH. Een volledige gewone M1-boot wordt apart uitgevoerd; een
M6c-succesmarker geldt niet als gewone M1-acceptatie.

## Afzonderlijke audits en bronbinding

De read-only [native audit](m6c-review/m6c-final-native-audit.json) importeert geen
projectparser, harnas of fingerprinthelper. Zij controleert 130 geregistreerde
artifacthashes, alle 176 werkelijke runbestanden, 12 capture-archiefhashes en alle
141 bronbestanden tegen `bc4cd104`. Acht eindvolumes zijn zelfstandig gedecodeerd:
65 geldige v2-records en twee weigeringsfixtures. De 67 niet-lege werkelijke
sectoren en seeds zijn bewaard in [journal-export](m6c-review/m6c-actual-journal-records.json).
Alle oorspronkelijke en definitieve imagehashes zijn vanuit die tekst opnieuw
opgebouwd. Volledige hashes staan naast het [auditscript](m6c-review/audit_m6c_native.py).
Het script bewaart de uitvoeringspaden van deze sessie en vereist de bijbehorende
runbestanden; een verse checkout heeft die genegeerde buildbestanden niet.
De geëxporteerde schijfgegevens blijven zonder die binaries reconstrueerbaar.

Zes vroege runrecords noemen nog `bb22fdb` als toenmalige lokale Git-HEAD omdat
het reeds op GitHub vastgelegde checkpoint lokaal werd ingelezen terwijl de gate
liep. De bronboom was daarbij al gelijk aan de gepushte tree. Alle vijftien
records en builds hebben dezelfde onafhankelijk nagerekende fingerprint; ieder
van de 141 bronbestanden is bytegelijk aan `bc4cd104`. De overige negen records
noemen die commit ook als HEAD. De oorspronkelijke metadata is niet herschreven.

De [statische integratie-audit](m6c-review/m6c-integration-audit.json) controleert
de bestaande runtimegrenzen en het daadwerkelijk ingebedde M6c-gebruikers-ELF.
De adapterreview gebruikt de echte servicebron met een host-ABI-shim; dat is
hostbewijs van de afhandeling van aangeboden fouten, geen native devicefoutinjectie.
De actuele probe gebruikt relatieve modulepaden en kan vanuit de repositoryroot
opnieuw worden gebouwd met de argv uit `adapter-build.json`. De oorspronkelijke
probe en rapporten met sessiepaden blijven onder `m6c-review/adapter-original/`
bewaard; de nieuwe build en alle 15 scenario's zijn werkelijk opnieuw uitgevoerd.
De eerste diagnostische normalrun en aanvankelijke, later herstelde Clippy-melding
zijn geen extra finale acceptatieruns. De reviews zijn afzonderlijke AI-controles,
geen menselijke audit of formeel bewijs. De native audit dekt M6c; de latere
M6b-regressie en afsluitende build hebben eigen archieven.

De GitHub-review van het broncheckpoint is afzonderlijk behandeld in
[REVIEW-FOLLOWUP.md](m6c-review/REVIEW-FOLLOWUP.md). De vier corecontroles zijn
na de evidencecommit opnieuw read-only uitgevoerd, met HEAD, werkmapstatus,
bronfingerprint en bestandshashes vóór en na. Het eerdere `core-final/` bewaart
de oorspronkelijke voorbereidende formattering; die geschiedenis is niet
achteraf veranderd in een fictieve `--check`-uitvoering. Herhaalde hostproeven
tellen niet als extra scenario's of nieuwe native VM-runs.

## Grenzen en vervolg

### Bekende native observerbeperking

De `Unknown`-afhandeling is met de echte serviceadapter en een host-ABI-shim
gecontroleerd. In `kernel/src/m2/m3/m6c.rs` bewaart de acceptatieobserver bij
een voorbereide mutatie echter de verwachte `Ok`-reply. Na een native WRITE-
of FLUSH-fout of time-out kan de adapter `Unknown` aanbieden, waarna `sent()`
op die verwachting panikeert voordat de client het antwoord ontvangt. Dit
geldt ook voor `milestone-m6c`, niet alleen voor fouttestfeatures. Dit is een
geldige, nog ongerepareerde observerfout.

De huidige native integratie voldoet onder die devicefouten nog niet aan de
bedoelde gecontroleerde `Unknown`-respons en voortzetting. De ontwerpvereisten
blijven staan: geen commit zonder WRITE+FLUSH, geen automatische replay en
geen verdere geldige GET/LIST/mutaties bij onzekerheid. Het geteste
serviceadaptergedrag is geen bewijs dat het defecte native observerpad werkt.

De begrensde native acceptatie gebruikt de vastgelegde QEMU/device-fixture
zonder native devicefoutinjectie; `capture_m6c.py` weigert `M6C:OUTCOME`.
De 15 host-ABI-scenario's vervangen geen toets van de kernelobserver.
Herstel vraagt een afzonderlijke native wijziging met WRITE-/FLUSH-fout- en
time-outinjectie, controle van het werkelijk ontvangen clientantwoord en
nieuwe brongebonden native verificatie.

[Gerichte scopebeoordeling door CodeRabbit](https://github.com/simonvanieperen/nieuw-os/pull/10#issuecomment-6457209539).

### Overige grenzen

De vier objectslots, 128 inhoudsbytes, 32 UTF-8-naambytes en 32 append-only
journalrecords zijn tijdelijke proefbudgetten. Er zijn nog geen directoryboom,
ruimtehergebruik, meerdere schrijvers, native grafische desktop, native NAS of
versleutelde documentopslag. Namen en historische inhoud staan leesbaar in het
testjournal; verwijderen is logisch. FNV diagnosticeert corruptie en biedt geen
cryptografische authenticatie. QEMU-proceskills zijn geen fysieke stroomuitvaltest.
De bestaande vertrouwde virtio-broker en deviceaannames blijven gelden.

De volgende productstap kan deze echte list/open/save/rename-interface gebruiken.
Grotere opslag, sleutels/entropy, compositor/invoer en native netwerk hebben eigen
acceptatiecriteria. De bredere M6-opslag- en updatefase is daarmee nog niet af.
De bron en tekstbewijzen staan op de private werkbranch; main is niet gemerged en
er is geen release of installatie op echte hardware uitgevoerd.
