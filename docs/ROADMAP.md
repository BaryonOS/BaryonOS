# Route van onderzoeksproject naar bruikbaar systeem

## Vaste productroadmap: 0% tot 100%

De [README](../README.md#voortgang-naar-een-volwaardig-zelfstandig-os) toont de
actuele voortgangsbalk en score per productspoor. De vaste noemer is
**100 gewogen acceptatiepunten**, met alle criteria en bewijs in
[PROGRESS.md](PROGRESS.md) en het bronbestand [progress.json](progress.json).
De eerste meting op M1–M6c was **13%**. De M6d-acceptatie van S07 bracht de
stand op **14%**; de actuele herbeoordeling staat in de README.
Dit is een planningskeuze, geen percentage resterende tijd of veiligheid.

| Productspoor | Punten tot het volledige einddoel |
|---|---:|
| Eigen kern, processen en isolatie | 15 |
| Opslag, documenten en gegevensherstel | 15 |
| Drivers, fysieke hardware en energie | 12 |
| Native Materiaal en toegankelijkheid | 15 |
| Internet en netwerkdiensten | 10 |
| Beveiliging, privacy en updates | 12 |
| Apps, SDK en installatie | 8 |
| Persoonlijke NAS en meerdere apparaten | 5 |
| Lokale AI en native quantum-runtime | 3 |
| Dagelijks gebruik en onderhoudbare release | 5 |
| **Totaal** | **100** |

De sporen kunnen parallel lopen. De M-nummers hieronder zijn technische
mijlpalen, geen gelijke procentstappen: M6c betekent niet dat het OS voor
60% of 6/11 klaar is. Een vier-objectenproef levert alleen haar specifieke
punten op; grotere documenten, ruimtehergebruik en dagelijks herstel blijven
aparte eisen.

**0%** = nog geen criterium gerealiseerd. **100%** = alle criteria voor een
volwaardig zelfstandig dagelijks OS gehaald, met eigen kernel, hoogwaardige
native bediening, nuttige apps, internet, beveiligde opslag, updates, herstel,
ondersteunde hardware en onderhoud. Geen bestaand OS vormt de basis.
De ambitie wordt niet verlaagd tot een testshell of VM-demo. Concrete
hardware- en appondersteuning blijven aantoonbaar afgebakend; volledige
historische featurepariteit met ieder groot OS is geen meetbare release-eis.

Iedere opdracht herbeoordeelt de score en actualiseert datum/bewijs; geen
nieuwe acceptatie betekent **0 procentpunt verschil**, geen fictieve stijging.
De generatie- en controleopdrachten staan in PROGRESS.md en AGENTS.md.

## Technische route en bewijsgrenzen

Status 28 oktober 2026: M1–M6b zijn geïmplementeerd en hebben versiegebonden
native bewijs in het vaste QEMU-profiel. M6c voegt namen, atomisch hernoemen
en een domeingebonden documentcatalogus toe. De oorspronkelijke M6c-gate is
vastgelegd met 19 groepen / 15 VM-invocaties; M6b-regressie op die bron met
18 / 14. M6d voegt gecontroleerd `Unknown` bij native WRITE-/FLUSH-fouten toe,
zonder automatische herhaling van de onzekere write. De geaccepteerde
M6c/M6d-gate omvat **35 groepen / 31 VM-invocaties**, met een afzonderlijke
M6b-regressie van **18 groepen / 14 VM-invocaties**. Dit sluit het afgebakende
criterium **S07** en brengt de vaste productscore op **14%**.

Het oorspronkelijke Linuxbewijs staat in
[M6D-ACCEPTANCE.md](verification/M6D-ACCEPTANCE.md); de aanvullende
Windowsuitvoering blijft afzonderlijk aan bron `326d2f6` gebonden in
[M6D-CLOSEOUT.md](verification/M6D-CLOSEOUT.md). De foutinjecties bewijzen geen
fysieke devicefout of stroomuitval. Herstel start nog via een nieuwe VM;
automatische catalogusherstart en algemeen opslagherstel (K05/S08) blijven
open. De historische M6b-observer krijgt geen nieuwe foutafhandelingsclaim.
De bredere M6-opslag-/updatefase bevat dus vervolgwerk. Actuele ontwikkeling:
[PROJECT-STATE.md](PROJECT-STATE.md) en [WORK-QUEUE.md](WORK-QUEUE.md).
Materiaal, NAS/TLS en qsim behouden hun afzonderlijke host-/prototypegrenzen.
Verdere fasen zijn een ontwikkelroute, geen kalenderbelofte.

## Richting en grenzen

Het project heeft vanaf M0 drie parallelle sporen: een zelfstandig technisch fundament, een oorspronkelijk hoogwaardig productontwerp en quantumonderzoek. De eerste kernel draait in een virtuele machine; tegelijkertijd onderzoekt het ontwerpteam hoe het OS moet voelen en werken. De omvang van de bootproef bepaalt niet de omvang van de productambitie.

Een bootend prototype, een bruikbaar experimenteel OS en een betrouwbaar dagelijks besturingssysteem zijn verschillende resultaten. Dat laatste vraagt langdurig onderhoud, drivers, appondersteuning, onderzoek en onafhankelijke experts. Een beginner met Codex kan de eerste stappen uitvoeren; Codex kan niet geloofwaardig garanderen alleen een veilig alternatief voor alle grote OS’en op te leveren. Daarvoor bestaat geen verifieerbare tijdsinschatting in dit onderzoek.

## Mijlpalen en beslismomenten

| Fase | Te leveren resultaat | Acceptatiebewijs | Stop-/doorgaancriterium |
|---|---|---|---|
| M0 — voorbereiding | Reproduceerbare ontwikkelomgeving, vaste QEMU-/firmware-/Rust-versies, ontwerpbesluiten en threat model | Toolinventaris, buildinstructies, checksums; geen verborgen afhankelijkheid van persoonlijke hostconfiguratie | Alleen door als een schone tweede omgeving dezelfde setup kan uitvoeren of het ontbrekende bewijs expliciet openstaat |
| M1 — zelfstandige boot | Eigen UEFI-loader laadt eigen Rust-kernel; kernel draait na overdracht en logt een vaste succescode | Geautomatiseerde QEMU-run met timeout, echte seriële log en exitstatus; ontbrekende/corrupte kernel levert gecontroleerde fout | Geen uitbreiding van deze boot-slice vóór betrouwbare boot; losstaand productonderzoek loopt parallel |
| M2 — geheugen en exceptions | Eigen framebeheer en pagetables; betrouwbare exceptionafhandeling | Allocatie-/grensgevallen, ongeldige mapping, page-fault-tests; gereserveerd geheugen blijft intact | Geen gebruikersprocessen vóór geheugeninvarianten en herstelpad voldoende onderzocht zijn |
| M3 — processen en isolatie | Twee user-mode-processen met gescheiden adresruimten, minimale scheduler en syscallgrens | Proces A mag B niet lezen/schrijven; A crasht terwijl B voortgang maakt; foutief syscalladres crasht niet de kernel | Gebroken isolatie is blokkerend, ook als alle demonstraties er goed uitzien |
| M4 — capabilities en IPC | Beperkte kernelobjecten en versieerbaar IPC-contract | Ongeldige/stale handle, rechtenverhoging, rechtenvermindering, intrekking, berichtfuzzing en quotatests | Reviewer die implementatie niet schreef probeert garanties te weerleggen |
| M5 — eerste systeemservice | User-space-service en eerste virtio-driver; alleen testimage | Driver/service herstart; malformed descriptors, grenslengtes, foutcodes en buffers getest | Geen echte hardwareclaim op basis van uitsluitend virtuele DMA-tests |
| M6 — opslag en updates | Persistent gebruikersvolume plus controleerbaar update-/herstelontwerp | Fault-injection bij schrijven en updaten; wijziging, rollbackaanval, volle schijf en sleutelrotatie getest | Dagelijks gebruik blijft geblokkeerd zonder herstelbaarheid en onafhankelijke review |
| M7 — native productintegratie | Gekozen interactiemodel en designsysteem integreren in compositor/shell, invoer, instellingen en één nuttige native app | Toegankelijkheids-/taaktests; privacygedrag gemeten; app-rechten zichtbaar en intrekbaar | Productbeslissing op gebruiksproef; geen grote appwinkel vóór basis bruikbaar is |
| M8 — netwerk en harde beveiligingscontrole | Beperkte netwerkdienst/stack, mature cryptobibliotheken, sleutelbeheer | Protocol-/fuzztests, geen standaard luisterpoorten, traffic-capture, keten van updateverificatie en entropiebewijs | Geen ‘veilig voor gevoelige data’-claim zonder afgebakende audit en rest-risico’s |
| M9 — eerste fysieke pc | Eén gedocumenteerd referentiesysteem | Drivers, DMA-begrenzing, slaap/wek, energie, boot en herstel op die exacte configuratie | Ondersteuningsmatrix noemt exact wat wel/niet werkt |
| M10 — tweede ISA | Bijvoorbeeld AArch64 onder QEMU | Zelfde proces-/IPC-contracttests op tweede architectuur | Pas generaliseren nadat echte verschillen zichtbaar zijn |
| M11 — community-release | Gedocumenteerde experimentele versie, SDK, bijdragepad, release- en onderhoudsbeleid | Reproduceerbare build door derde; bekende issues; bron- en licentiecontrole; verantwoord meldpunt | Productieclassificatie volgt bewijs, niet enthousiasme |

M6 kan eerst zonder internet worden ontwikkeld met lokaal aangeleverde ondertekende updatebundels. Een veilige updateketen ontwerpen hoeft dus niet op de netwerkstack te wachten. Productverkenning begint al bij M0. Integratie met de echte kernel volgt zodra de benodigde interfaces bestaan; afspraken over die interfaces worden samen met het productteam ontworpen.

## Parallel productspoor vanaf M0

Dit spoor voorkomt dat een technische demonstratie ongemerkt uitgroeit tot een conventionele desktop. Het gebruikt synthetische gegevens en afzonderlijke prototypebestanden; het vervangt geen kerneltests.

| Stap | Resultaat | Beslisbewijs |
|---|---|---|
| P0 — identiteit en principes | Heldere productbelofte, doelgroep, Europese waarden en eigen ontwerpprincipes | Simon kan verwoorden waarom dit product meer biedt dan een andere desktopstijl |
| P1 — drie interactieconcepten | Materiaal, Werk en Atelier uit PRODUCT-AND-SOVEREIGNTY.md als afzonderlijke scenario's | Zelfde vijf taken per concept; vergelijking met bekende patronen en expliciete nieuwe hypothese |
| P2 — beleefbaar prototype | Hoogwaardig interactief prototype van de sterkste richting, met eerste gebruik en herstel | Gebruikers begrijpen navigatie, rechten en gegevenslocatie; gemeten frictie leidt tot aanpassing |
| P3 — designsysteem en appcontract | Eén premium productfamilie met typografie, contextuele tools, motion, toegankelijkheid, privacyuitleg, SDK-/UI-principes en object-/taakinterfaces; native rendercontract als doel | Consistente tekst-/tabel-/presentatieflows zonder Office-ribbonkopie; requirements voor kernel, graphics, diensten en ontwikkelgereedschappen teruggeleverd |
| P4 — native vertaling | Geselecteerde flows op het eigen OS, gekoppeld aan M7 en relevante services | Dezelfde taken werken werkelijk op het OS; verschillen met het prototype zichtbaar bijgehouden |

Een interactieprototype op een bestaand hostsysteem is ontwerpgereedschap; het wordt geen vervangende OS-kernel. Tests met fictieve NAS- en AI-status tonen begrijpelijkheid, terwijl de echte privacygaranties later de LP-tests moeten doorstaan. Begin vroeg met een hoogwaardige vormgeving; verwissel simulatie en implementatie niet.

Een technische onzekerheid wordt een korte onderzoekstaak met hypothese, experiment en besliscriterium. Bestaande beperkingen zijn input voor onderzoek, geen automatische reden om terug te vallen op een Linux-achtige productervaring. Kies een conventioneel patroon alleen als de gebruikersproef dat ondersteunt. Bewaar verworpen concepten en de reden, zodat ontwerpkeuzes overdraagbaar blijven.

## Parallel quantumspoor vanaf M0

Volg [QUANTUM-RESEARCH.md](QUANTUM-RESEARCH.md): Q0 is voor de eerste proef afgerond in [implementation/QUANTUM-SPEC.md](implementation/QUANTUM-SPEC.md); Q1 bouwt de gekozen lokale simulator en jobinterface; Q2 een lokaal QPU-contract; Q3 de eigen OS-kernel als klassieke host/controller van één hybride quantumplatform; Q4 verdere uitvoeringsmodellen. Het quantumspoor is een expliciete ambitie, geen losse toekomstige wens. Elke fase heeft eigen bewijs; postkwantumcryptografie blijft bovendien onderdeel van de beveiliging van gewone apparaten. Geen externe quantumcloud.

## Aanvullende privacygates na de netwerkbasis

N02 voegt een echte, begrensde hybride TLS-hostproef toe vóór de native port. De gewijzigde bewijs-/PQC-opdrachten staan in [VERIFIABLE-SECURITY-AND-PQC.md](implementation/VERIFIABLE-SECURITY-AND-PQC.md): formeel capabilitymodel, gecontroleerde native cryptobuild, entropy, ondertekende images, geauthenticeerde opslag en daarna N03-integratie. CHERI blijft een expliciete toekomstige platformproef; de huidige x86_64-code erft die hardwaregaranties niet. Qsim en PQC behouden afzonderlijke acceptatiecriteria.

De gebruikerscorrectie van 28 oktober 2026 staat bindend in LOCAL-PRIVACY-CONTRACT.md revisie 4. Internet is een productvereiste. Bouw de persoonlijke NAS-route verder volgens N01/N03 en behandel externe providers als een **afzonderlijke onbetrouwbare ciphertextbackend** volgens [ENCRYPTED-OBJECT-STORAGE.md](implementation/ENCRYPTED-OBJECT-STORAGE.md). De gebruiker mag een provider expliciet kiezen, maar er is geen verplichte leverancierscloud, provider-key-escrow of stille fallback. Lokale AI houdt een geïsoleerde modeldownloader en netwerkloze inferentie.

Naast de bestaande NAS-gates krijgt P05 een provider-onafhankelijke encrypted-objectadapter. Vóór een productclaim moet worden getest dat providercredentials zonder content key geen plaintext geven; namen/manifests beschermd blijven; ciphertextmanipulatie en rollback geen stil succes opleveren; providerdelete/outage geen stille uitwijk activeert; en privacyuitleg uitschakelen geen capability verruimt. Een vals NAS/providerendpoint krijgt geen gegevens buiten zijn scope. Adminintrekking blokkeert oude NAS-sessies/uploads zonder gebruikerssleutel of gasttoestemming. De AI werkt met uitgeschakeld internet en onvoldoende RAM start geen externe inferentie. Een schone installatie of document op papier bewijst deze toekomstige eigenschappen niet.

## Eerste opdracht aan Codex

De voorbereidende onderzoeken hieronder niet opnieuw uitvoeren. De concrete opdracht is [implementation/FIRST-BATCH.md](implementation/FIRST-BATCH.md):

1. **B00:** voer `python3 tools/preflight.py` uit in de echte ontwikkelomgeving en maak de vastgelegde tools beschikbaar.
2. **B01:** implementeer eigen loader, bootprotocol en kernel met de vastgelegde succes- en fouttests.
3. **P02:** bouw het gekozen Materiaal-prototype volgens PRODUCT-SPEC.md, uitsluitend met synthetische gegevens.
4. **Q01:** bouw en test de begrensde lokale Rust-simulator volgens QUANTUM-SPEC.md.
5. Verdeel exclusieve schrijfgebieden; laat de samengevoegde wijzigingen reviewen en passende tests uitvoeren.
6. Leg werkelijk bewijs en concrete blokkades vast in PROJECT-STATE.md. Een ontbrekende QEMU-tool blokkeert geen onafhankelijk productwerk.

De ontwikkelomgeving, eerste build en M2 zijn nu in het vastgelegde Windows/QEMU-profiel getest; zie de actuele status voor bewijs en beperkingen. Een code-review vervangt geen bootbewijs. Verhelp concrete problemen gericht; begin niet opnieuw een brede onderzoeksronde.

## Meetplan voor ‘licht en snel’

Geen getallen als gerealiseerd presenteren voordat ze gemeten zijn. Stel na de eerste bruikbare baseline budgetten vast en leg de meetmethode vast. Eerste voorstellen voor te volgen metrics:

| Metriek | Methode | Beperking |
|---|---|---|
| Bootduur | Firmware-start tot afgesproken kernel-/shell-ready-marker; rapporteer mediaan en p95 over herhaalde runs | TCG, lokale acceleratie en echte hardware apart rapporteren |
| Geheugen | Kernel-/service-/appgeheugen afzonderlijk; committed en resident waar van toepassing | Totale toegewezen VM-RAM is geen werkelijk OS-gebruik |
| IPC | Latentieverdeling voor vast berichtformaat en belasting | Geen vergelijking tussen verschillende beveiligingsgaranties zonder toelichting |
| Interactie | Invoer tot zichtbare verwerking; later framevertraging en stotteren | Alleen meten na een echte invoer-/renderketen |
| Herstel | Tijd en gegevensverlies na gecontroleerde servicecrash/updateonderbreking | Herstart van een proces bewijst geen herstel van alle gekoppelde resources |
| Energie | Idle-/werkbelasting op fysieke referentiehardware | VM CPU-gebruik voorspelt telefoonbatterijduur niet |
| Projectoppervlak | Privilegieerde code, dependencies, ongedocumenteerde unsafe-blokken en open securitybevindingen | Regelaantal is een hulpmiddel, geen directe veiligheidsmaat |

Publiceer meetomgeving, sampleaantal en spreiding. Budgetten zijn projectkeuzes; gebruik geen ononderbouwde vergelijking met Windows, Linux, Android of macOS.

## Ondersteuning verdient niveaus

- **Gepland:** staat in de roadmap; er is nog geen werkend bewijs.
- **Bouwt:** cross-compilatie slaagt, maar boot is niet bewezen.
- **Boot:** overeengekomen bootcheck slaagt op de genoemde configuratie.
- **Experimenteel:** kernfuncties werken; bekende beperkingen en dataverliesrisico’s staan erbij.
- **Ondersteund:** afgesproken testmatrix, herstelroute en onderhoudseigenaar aanwezig.

Gebruik nooit ‘ondersteunt ARM’, ‘draait op alle apparaten’ of ‘quantumveilig’ als een test alleen een compiler, simulator of losse cryptofunctie betreft.

## Onderzoek dat later nog nodig is

Vóór dagelijks gebruik moeten ten minste de concrete crypto-implementatie, opslagintegriteit, updatevertrouwensketen, UEFI-/hardwarevertrouwen, isolatie onder DMA en resource-uitputting, compiler-/dependencyketen en gebruikersherstel door onafhankelijke deskundigen worden onderzocht. Formele modellen zijn vooral zinvol voor kleine kerninvarianten en rechtenovergangen; bewijsstatus altijd per component en property registreren.

De roadmap is een eigen synthese. Technische onderbouwing en primaire bronnen staan in [ARCHITECTURE.md](ARCHITECTURE.md) en [SECURITY-AND-QUANTUM.md](SECURITY-AND-QUANTUM.md). De geselecteerde toolversies staan in implementation/BOOT-AND-TOOLCHAIN.md; hun werkelijke gezamenlijke build wordt tijdens B00/B01 getest. Geselecteerd betekent niet getest.

