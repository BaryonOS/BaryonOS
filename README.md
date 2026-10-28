# Nieuw OS — eigen kernel en native documentcatalogus

<!-- OS-PROGRESS:BEGIN -->
## Voortgang naar een volwaardig zelfstandig OS

![Nieuw OS: 14% van de vaste productroadmap](docs/assets/progress.svg)

**14% — 14 van 100 gewogen acceptatiepunten.** Dit is een
planningsmeter voor aantoonbaar werkende functies van het eigen OS, geen
percentage verstreken ontwikkeltijd, veiligheidswaarborg of gemeten gelijkheid
met Windows, macOS of Linux. De eigen kernel blijft het fundament; geen van
die besturingssystemen is de basis.

**0%:** nog geen criterium gerealiseerd. **100%:** een volwaardig, dagelijks
bruikbaar en onderhoudbaar zelfstandig OS op gedocumenteerde ondersteunde
hardware, met native Materiaal, nuttige apps, internet, veilige opslag,
updates, herstel en de afgesproken privacyfuncties. Een bootende demo of
alleen een nette desktop haalt die eindgrens niet.

| Productspoor | Totaal punten | Behaald |
|---|---:|---:|
| Eigen kern en procesisolatie | 15 | 9 |
| Opslag en gegevensherstel | 15 | 4 |
| Hardware, drivers en energie | 12 | 1 |
| Native Materiaal en toegankelijkheid | 15 | 0 |
| Internet en netwerkdiensten | 10 | 0 |
| Beveiliging, privacy en updates | 12 | 0 |
| Apps, SDK en installatie | 8 | 0 |
| Persoonlijke NAS en meerdere apparaten | 5 | 0 |
| Lokale AI en quantum-runtime | 3 | 0 |
| Dagelijks gebruik en onderhoudbare release | 5 | 0 |
| **Totaal** | **100** | **14** |

**Herbeoordeeld:** 2026-10-28 · **Rubric:** v1 ·
**Bron:** [`f2c40551`](https://github.com/simonvanieperen/nieuw-os/commit/f2c4055120b893f004c11bf9b845f0f37c92adaf) ·
**Ontwikkelbranch:** `codex/m6b-storage-errors`.
Deze opdracht: **14% → 14% (+0 procentpunt)**. **Opdracht:** Native G0 start en toetsenbordproef na main-integratie; eindregressies volgen.
G0 voert echte beeldschermuitvoer en PS2-RAMinvoer uit in QEMU na EBS/M2; typen en backspace zijn met echte framebufferbeelden gecontroleerd. Geen compositor/sessie, audio/USB of fysieke-hardwarebewijs, dus U01/H02/H03 blijven open. Vijf gerichte M6b-timeout/cancelproeven geslaagd op eerdere bron; gecombineerde eindgates volgen. Score blijft 14%.

De meter geldt voor de beoordeelde ontwikkelstand; opname in `main` en een
productrelease zijn afzonderlijke gebeurtenissen. Hostprototypes van Materiaal,
NAS/TLS en qsim staan zichtbaar in de projectstatus, maar krijgen geen native
productpunten. Elk criterium telt volledig of niet; onbewezen werk blijft open.

[Criteria, bewijs en berekening](docs/PROGRESS.md) ·
[Volledige roadmap](docs/ROADMAP.md) ·
[Actuele uitvoering](docs/PROJECT-STATE.md).
Bij **iedere opdracht** worden deze meter, datum, bewijs en wijziging opnieuw
beoordeeld; de score mag gelijk blijven of dalen. Instructie: [AGENTS.md](AGENTS.md).
<!-- OS-PROGRESS:END -->

## Laatste integratie en review

[PR #13](https://github.com/simonvanieperen/nieuw-os/pull/13) brengt M6c/M6d,
de actuele main en het [nieuwe Windowsbewijs](docs/verification/M6D-CLOSEOUT.md)
samen. De geschiedenis is zonder codeverlies verbonden. Ook de ontbrekende
qsim-parserfix is opgenomen: positionele JSON-arrays worden geweigerd;
[27 hosttests, rustfmt en Clippy](docs/verification/qsim-request-validation/README.md)
slagen. De actuele review- en squashmergestatus staat in de PR.
De native broninputs blijven gelijk en
de voortgang blijft **14%**.

## Laatste bouwstap: M6d — opslagfouten gecontroleerd afhandelen

De eigen kernel laat onzekere schrijfuitkomsten nu bij de apps aankomen zonder
observerpanic of onterechte bevestiging. Acht foutproeven controleren onder meer
WRITE/FLUSH-fouten, time-outs, descriptorweigering en resetquarantaine. Beide
appdomeinen blijven voortgang maken; na `Unknown` volgen geen nieuwe
blokaanroepen of automatische herhalingen van de onzekere wijziging.

Op bron `88e6c04b` slagen **35 groepen / 31 echte VM-invocaties**, inclusief
14 koude recoveries. De afzonderlijke M6b-regressie slaagt met **18 groepen /
14 VM-invocaties**. Beide bewijsbundels zijn afzonderlijk gecontroleerd; de
normale eindbuild is bytegelijk aan beide normale M6d-gateruns. Dit sluit
criterium S07 en brengt de vaste voortgangsmeter van **13% naar 14%**.

[Acceptatie, commando's en bewijs](docs/verification/M6D-ACCEPTANCE.md) ·
[Vooraf vastgelegd contract](docs/implementation/M6D-IO-UNCERTAINTY.md) ·
[PR #11](https://github.com/simonvanieperen/nieuw-os/pull/11).
Werkbranch: `codex/m6d-io-recovery-roadmap`. Herstel gebruikt nog een nieuwe VM;
automatische dienstherstart in dezelfde sessie is de volgende beperkte stap.
De foutinjecties bewijzen geen fysieke schijffouten of stroomuitval.

## Eerdere bouwstap: M6c — namen en documentcatalogus

De nieuwe native dienst bewaart documentnamen, inventariseert uitsluitend het
eigen appdomein en hernoemt met behoud van inhoud en revisiecontrole. Dit is de
opslaginterface voor de latere Materiaal-integratie. De volledige native gate
slaagt op broncommit `bc4cd104`: **19 controlegroepen / 15 echte VM-invocaties**,
waaronder zes koude recoveries, twee externe onderbrekingen tijdens hernoemen
en één normale M1-boot. De volledige M6b-regressie op dezelfde bron slaagt ook:
**18 groepen / 14 VM-invocaties**. Daarnaast slagen 30 Rust-coretests,
33 M6c-harnastests en 15 afzonderlijke hostproeven van de echte serviceadapter.
[Contract](docs/implementation/M6C-NAMED-CATALOG.md) ·
[Acceptatie en reproduceerroute](docs/verification/M6C-ACCEPTANCE.md) ·
[Actuele projectstatus](docs/PROJECT-STATE.md).

```sh
python3 -B tools/build.py --kernel-feature milestone-m6c
python3 -B tools/run_m6c.py --case normal --smoke
```

Werkbranch: `codex/m6c-named-catalog`, [PR #10](https://github.com/simonvanieperen/nieuw-os/pull/10),
boven op de afgeronde M6b-branch. De proef gebruikt eigen wegwerpimages:
vier objecten, maximaal 128 inhoudsbytes, 32 UTF-8-naambytes en 32 journalrecords.
Bezet v1 wordt zonder schrijven geweigerd. Dit levert native opslaggedrag;
de interactieve desktop, encryptie en native NAS-integratie blijven vervolgstappen.

## Afgeronde basis: M6b

De eigen Rust-kernel heeft nu een native objectdienst in ring 3. Twee geïsoleerde
testapps kunnen via capability-IPC objecten aanmaken, lezen, bijwerken en
verwijderen. De dienst schrijft via echte virtio-I/O; revisies en verwijderrecords
blijven behouden na een koude herstart. Apps krijgen uitsluitend hun eigen
domeinendpoint, geen rechtstreekse blok- of DMA-toegang.

De uitgebreide M6b-gate op broncheckpoint `592dccf` is geslaagd: **22 controlegroepen
en 18 echte VM-invocaties**, inclusief acht koude recoveries en twee externe
QEMU-kills. Twee nieuwe WRITE-/FLUSH-foutproeven controleren dat de dienst
Unknown teruggeeft zonder kernelpanic, valse succesmelding, replay of oude
leesinhoud. Ook 22 Rust-tests en 31 M6b-harnastests slagen. De fouten zijn lokale
statussnapshotinjecties na echte devicecompletion; geen echte defecte backend.
Ruwe uitvoer en hashes: [verification/m6b-storage-errors](docs/verification/m6b-storage-errors/).
Het [eerdere M6b-bewijs](docs/verification/m6b/) is behouden.
Deze afgebakende proef gebruikt vier objecten van maximaal 128 bytes en een
journal van 32 records. M6c hierboven voegt namen toe; ruimtehergebruik, encryptie
en native NAS/desktop volgen als aparte integratiestappen.

```sh
python3 -B tools/build.py --kernel-feature milestone-m6b
python3 -B tools/run_m6b.py --case normal --smoke
```

De runner gebruikt een eigen synthetisch wegwerpvolume en sluit na de
acceptatieproef af. Volledig contract: [M6B-OBJECT-STORAGE.md](docs/implementation/M6B-OBJECT-STORAGE.md).
Voortgang, regressies en resterende grenzen: [PROJECT-STATE.md](docs/PROJECT-STATE.md).

## Nieuwe richting: internet en persoonlijke NAS-opslag

**Vervolg N02:** [hybride TLS-transportproef](services/nas/TRANSPORT.md) met echte wederzijdse authenticatie en X25519MLKEM768 op loopback. De gecontroleerde [beveiligings-/PQC-blauwdruk](docs/implementation/VERIFIABLE-SECURITY-AND-PQC.md) onderscheidt formele eigenschappen, eigen kernel, CHERI-onderzoek en native integratie. Dit is hostbewijs; geen native internetstack, formeel bewezen compleet OS of volledige postkwantumauthenticatie. Uitvoering en actuele testresultaten: [PROJECT-STATE.md](docs/PROJECT-STATE.md).

De gebruikerswijziging van 27 oktober 2026 voegt veilige internettoegang, eigen NAS en gedeelde ruimte bij vrienden/familie toe. Inhoud en documentnamen worden op de gebruikersclient versleuteld; de NAS-beheerder kan ruimte intrekken en zijn kopie verwijderen zonder gebruikerssleutel of toestemming. De uitvoerbare hostproef staat in [services/nas/](services/nas/README.md); het browserprototype blijft een simulatie. Contract en native integratiecriteria: [NAS-SHARING-AND-INTERNET.md](docs/implementation/NAS-SHARING-AND-INTERNET.md). Dit voegt nog geen netwerkstack aan de kernel toe en verandert de M6a-bewijsstatus niet.

De eigen Rust-kernel start via de eigen UEFI-loader in QEMU, draait geïsoleerde gebruikersprocessen en verwerkt capabilities/IPC. M5 voegt een echte ring-3-blokdienst met alleen-lezen virtio-I/O toe. De definitieve M1–M5-suites slagen: **77 controlegroepen en 64 echte VM-runs**, plus een normale M5-eindstart. Materiaal en qsim blijven afzonderlijke hostprototypes. Dit is een experimentele ontwikkelversie met synthetische testprogramma's.

- **Actuele voortgang en open controles:** [PROJECT-STATE.md](docs/PROJECT-STATE.md).
- **Starten en reproduceren:** [DEVELOPMENT.md](docs/DEVELOPMENT.md).
- **Opgeslagen testbewijs:** [docs/verification/](docs/verification/).
- **Productprototype:** [handleiding](ui/prototype/README.md).
- **Quantumsimulator:** [handleiding](experiments/quantum/README.md).

M6b is hervat vanaf `main` (`1f24f99`) op [codex/m6b-storage-errors](https://github.com/simonvanieperen/nieuw-os/tree/codex/m6b-storage-errors). De status staat per spoor in PROJECT-STATE.md. [PR #3](https://github.com/simonvanieperen/nieuw-os/pull/3) bewaart de M5-historie; de oude testmatrix staat in [ACCEPTANCE.md](docs/verification/m5/ACCEPTANCE.md).

De oudere M5-leesproef blijft selecteerbaar via `python tools/build.py --kernel-feature milestone-m5` en `python tools/run.py --smoke --milestone m5`. Er is nog geen interactieve native desktop. Kerneltransport en QEMU-device zijn vertrouwd; geen IOMMU- of fysieke-hardwarebewijs. De uitvoerroute staat in DEVELOPMENT.md. Publieke publicatie en definitieve licentie blijven een afzonderlijk besluit volgens LICENSE-PLAN.md.

## Projectrichting en verder lezen

Nieuw OS bouwt een eigen Rust-kernel met expliciete bevoegdheden en afzonderlijke
systeemdiensten, plus een oorspronkelijke productervaring: **Materiaal**.
Documenten en collecties staan centraal; apps zijn verwisselbare gereedschappen.
De eigen kernel is geen fork van een bestaand besturingssysteem. Rust/LLVM,
QEMU, UEFI, open standaarden en gecontroleerde bibliotheken zijn ontwikkelmiddelen.

Persoonlijke gegevens blijven lokaal of end-to-end versleuteld op een expliciet
gekoppelde eigen NAS of NAS bij vrienden/familie. Internet vraagt passende
app-/peerrechten. Systeem-AI blijft op het eigen apparaat zonder cloudfallback;
installatie van een app verleent geen algemene bestands- of netwerkrechten.
De actuele native implementatiegrenzen staan per spoor hierboven.

- [Architectuur en kernelcontract](docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md).
- [Materiaal en productambitie](docs/implementation/PRODUCT-SPEC.md).
- [Privacycontract](docs/LOCAL-PRIVACY-CONTRACT.md).
- [Quantumspoor](docs/QUANTUM-RESEARCH.md).
- [Beveiliging en postkwantumroute](docs/implementation/VERIFIABLE-SECURITY-AND-PQC.md).
- [Werkinstructies voor agents](AGENTS.md).
- [Ontwerpbesluiten](docs/DECISIONS.md) en [afhankelijkheden](docs/DEPENDENCIES.md).

De startdocumenten van 26 oktober beschrijven de toenmalige voorbereiding.
Zinnen daarin over ontbrekende loader/kernel of niet-uitgevoerde eerste tests
zijn historisch; volg voor de huidige uitvoering PROJECT-STATE.md.
De README en roadmap zijn geen claim van onhackbaarheid, algemene
hardwarecompatibiliteit of reeds bewezen gelijkwaardigheid met bestaande platforms.
