# Welke kennis Codex en de ontwikkelaar nodig hebben

Status: leer- en onderzoeksplan, 26 oktober 2026. Een taalmodel heeft geen aantoonbare bevoegdheid als kernelontwikkelaar. Goede uitkomsten vereisen actuele specificaties, concrete interfaces, uitvoerbare tests en controle van aannames. Dit bestand is een routekaart, geen vervanging voor de genoemde handleidingen.

## 1. Kennis per werkgebied

Gebruik de matrix om een taak voor te bereiden. De architectuurkeuzes en de volgorde van bouwen staan in `ARCHITECTURE.md` en `ROADMAP.md`; hier staat welke kennis vóór die taken aantoonbaar beschikbaar moet zijn.

| Werkgebied | Wat de uitvoerende agent moet begrijpen | Te leveren bewijs of kennisbestand |
| --- | --- | --- |
| Git en samenwerken | Commits, branches, worktrees, conflicts, review, rollback en bestandseigenaarschap. | Kleine reviewbare wijziging, diff en herstelinstructie. |
| Rust | Ownership, borrowing, lifetimes, generics, foutafhandeling, `no_std`, layout, FFI en voorwaarden van `unsafe`. | Voor elke onveilige grens een contract en gerichte test; verwijzing naar geldende taalregels. |
| CPU en geheugen | Registers, privilege, virtueel/fysiek geheugen, paginatabellen, uitzonderingen, atomics en geheugenordening. | Specifiek hoofdstuk uit de gekozen ISA-handleiding; invariant en test op de gekozen target. |
| Bouwen en opstarten | Crosscompilatie, objectformaat, linker, bootprotocol, firmware-overdracht en debug-symbolen. | Vastgelegde toolchain, bouwcommando's, artefactidentiteit en herhaalbaar opstartlog. |
| Kernelmechanismen | Allocatie, taken, contextwisseling, scheduling, IPC, capability-rechten en objectlevensduur. | Interfacebeschrijving plus toegestane en geweigerde scenario's. |
| Concurrency | Interrupt-context, races, deadlocks, lockvolgorde, annulering en multicoregedrag. | Vastgelegde lock- en levensduurregels; stress- en gerichte regressietests. |
| Drivers | Device discovery, MMIO, interrupts, DMA, descriptorgrenzen, time-outs en herstel na fouten. | Precieze apparaat-/protocolspecificatie en ongeldige-inputtests. |
| Opslag | Cachegedrag, persistente metadata, crashconsistentie, corrupte media, herstel en toegang. | Gedocumenteerd on-diskformaat, foutinjectie en hersteltest met wegwerpimages. |
| Netwerken | Framing, parsing, adressering, routing, naamresolutie, tijd, transportbeveiliging en resourcegrenzen. | Protocolversies en grenzen; tests voor misvormde pakketten en uitputting. |
| Beveiliging | Vertrouwensgrenzen, authenticatie versus autorisatie, sleutels, updates, herkomst en aanvallersmodellen. | Actueel threat model; elke claim gekoppeld aan test, review of bewijs. |
| Apps en SDK | ABI/API, versiebeheer, packages, rechtenmanifest, install/uninstall en ontwikkelgereedschap. | Een kleine externe voorbeeldapp die alleen de gedocumenteerde SDK gebruikt. |
| Grafische omgeving | Compositie, rendering, input, fonts, Unicode, accessibility, vensters en gebruikersverwachtingen. | Interactieprototype én later integratietest in het echte OS; duidelijk gescheiden. |
| Portability en energie | Architectuurafhankelijke code, boardondersteuning, firmware, slaap/ontwaken en batterijen. | Hardwarematrix met gemeten ondersteunde functies; geen generieke apparaatbelofte. |
| Verificatie | Unit/integratietests, property tests, fuzzing, foutinjectie, modelchecking en eventueel formele bewijzen. | Testbereik en beperkingen; reproduceerbare resultaten, geen alleen gegenereerde succesmelding. |
| Product en beheer | UX, toegankelijkheid, lokalisatie, privacy, releaseonderhoud en communitybijdragen. | Takenlijst, gebruikerstests, privacygedrag en onderhoudsbeleid. |

De matrix is bewust breed: een bruikbaar OS is ook een appplatform, driverproject en onderhoudsorganisatie. Codex kan kennis ontsluiten en voorstellen doen. Het ontbrekende empirische bewijs moet uit builds, tests en onafhankelijke beoordeling komen.

### Productonderzoek en consumentenadoptie

Voor UX-, onboarding-, migratie- en positioneringstaken zijn
[MARKET-AND-ADOPTION-2026-10-28.md](research/MARKET-AND-ADOPTION-2026-10-28.md)
en [CONSUMER-VALUE-2026-10-28.md](research/CONSUMER-VALUE-2026-10-28.md)
de actuele onderzoeksingangen. Houd **aandacht, proberen, overstappen en blijven
gebruiken** als verschillende uitkomsten. Privacy, lokale AI, vormgeving en
Europese herkomst zijn geen zelfstandige adoptiegaranties. Onderzoek
taakprestatie, leerlast, noodzakelijke apps/bestanden/apparaten, herstel en
vrijwillig hergebruik; ontwerpvoorstellen blijven hypotheses totdat passende
gebruikersproeven ze ondersteunen.

## 2. Wat vóór iedere ontwikkeltaak in de context hoort

Geef een agent niet alle documenten tegelijk. Laat hem eerst de relevante repository-instructies lezen en daarna alleen de bronstukken die voor de taak nodig zijn.

1. **Doel en grens:** één observeerbare uitkomst, toegestane bestanden en expliciete niet-doelen.
2. **Huidige toestand:** commit, doelplatform, versies, bestaande API's en bekende defecten.
3. **Normatieve bron:** documentnaam, versie, hoofdstuk en URL; markeer of iets een standaard, fabrikantregel of projectkeuze is.
4. **Invariant:** wat altijd waar moet blijven, bijvoorbeeld “een ingetrokken recht verleent geen nieuwe toegang”.
5. **Acceptatie:** concrete positieve en negatieve gevallen, exacte commando's en betekenis van exitcodes.
6. **Bewijs:** bewaar uitgevoerde commando's, relevante logs, uitkomst en beperkingen.
7. **Overdracht:** wat veranderd is, wat nog onzeker is en wat de volgende taak veilig kan aannemen.

Een verwijzing naar een document betekent niet dat de hele inhoud in het modelgeheugen aanwezig is. Laat de agent de relevante passage ophalen of een gecontroleerde lokale kopie lezen. Leg van gedownloade specificaties versie en checksum vast wanneer de build of review ervan afhankelijk is. Bewaar geen volledige beschermde boeken in de repository zonder passende rechten.

## 3. Leerroute voor Simon

Deze route loopt naast het OS-project en heeft geen vaste kalenderduur. Ga verder zodra de oefening zelfstandig kan worden uitgelegd en herhaald.

| Stap | Oefening | Klaar wanneer |
| --- | --- | --- |
| A. Wijzigingen beheersen | Maak een klein tekst-/Rustproject, verander het op een branch en herstel een fout met Git. | Een commit, diff en herstelactie zijn begrijpelijk zonder de hele projectmap opnieuw te kopiëren. |
| B. Rust-basis | Bouw een klein lokaal programma dat invoer leest, valideert en fouten afhandelt. | Ownership, borrowing en `Result` zijn aan eigen code uit te leggen. |
| C. Denken in grenzen | Ontwerp een berichtformaat met maximale lengte en valideer het in een gewone hosttest. | Misvormde invoer veroorzaakt een gecontroleerde fout en geen onbeperkte allocatie. |
| D. Van bron naar machine | Laat Codex één functie compileren en objectbestand, symbolen en relevante assembly uitleggen. | Het verschil tussen broncode, machinecode, linker en runtime is duidelijk. |
| E. Opstartproef | Volg de eerste mijlpaal uit de roadmap in QEMU. | De exacte build en boot zijn herhaalbaar; een foutlog kan worden teruggevonden. |
| F. Isolatie begrijpen | Gebruik een diagram en tests om twee processen met verschillende rechten te vergelijken. | Een geslaagde legitieme toegang en geweigerde onrechtmatige toegang zijn aantoonbaar. |
| G. Reviewen | Laat een andere agent een kleine wijziging beoordelen en beoordeel zelf de testlog. | “De agent zegt dat het klopt” is te onderscheiden van reproduceerbaar bewijs. |

Vraag bij een onbekend begrip om drie dingen: een uitleg van maximaal tien zinnen, een minimaal voorbeeld en wat er misgaat als de aanname onjuist is. Laat uitleg de huidige bouwstap ondersteunen; probeer niet eerst alle OS-literatuur uit te lezen.

## 4. Actieve kennis bijhouden

Houd naast de broncode bij:

- **Besluiten:** ADR's met probleem, opties, gekozen route, gevolgen en heroverwegingscriterium.
- **Interfaces:** syscall-/IPC-contracten, datastructuren, versiebeleid en eigenaarschap.
- **Aannames:** hardwaremogelijkheden, firmware, compiler, emulator en externe diensten.
- **Bewijsregister:** claim, test-ID, target, commit, resultaat en resterende beperking.
- **Status:** laatste werkende commit, huidige blokkade en eerstvolgende kleine taak.
- **Afhankelijkheden:** naam, versie, herkomst, licentie, onderhoud en reden van opname.

Werk deze documenten bij wanneer de werkelijkheid verandert. Een oude chat is geen gezaghebbende bron voor het huidige systeem. Een uitvoerende agent mag een ontbrekend resultaat als “niet uitgevoerd” registreren; hij mag geen geslaagde test invullen om een mijlpaal af te ronden.

## 5. Eerste primaire studiebronnen

Lees gericht. Voor Rust is het *Book* een leerbron; de *Reference* beschrijft taalregels en is niet bedoeld als eerste introductie. Intel's systeemhandleiding behandelt onder meer geheugenbeheer, privilege, interrupts en multiprocessorondersteuning. Die bronnen bepalen geen productontwerp; ze begrenzen wat technisch correct kan zijn (Klabnik et al., z.d.; The Rust Project, z.d.; Intel, 2026).

| Onderwerp | Beginpunt | Gebruik |
| --- | --- | --- |
| Rust leren | https://doc.rust-lang.org/book/ | Eerst ownership, borrowing, enums, fouten en tests. |
| Rust-regels | https://doc.rust-lang.org/reference/ | Taakgericht opzoeken; koppel aan de gepinde compiler. |
| x86_64-systeemprogrammering | https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html | Volume 3 voor systeemmechanismen; relevante versies vastleggen. |
| Open source | https://opensource.org/osd | Begrijpen wat de term wel en niet betekent. |
| Toegankelijke webinterface | https://www.w3.org/TR/WCAG22/ | Toetsbare webcriteria; niet gelijkstellen aan volledige OS-toegankelijkheid. |

De geraadpleegde bronnen zijn geen complete technische bibliotheek. Boot-, driver-, netwerk-, cryptografie- en hardwarestandaarden worden per mijlpaal geselecteerd in de andere kennisbestanden. Voor een ongelezen of niet te openen bron schrijft de agent: “Dit kan ik niet verifiëren.”

## Bronnen

Intel. (2026, 9 oktober). *Intel 64 and IA-32 architectures software developer manuals*. https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html

Klabnik, S., Nichols, C., & Krycho, C. (z.d.). *The Rust programming language*. Geraadpleegd op 26 oktober 2026, van https://doc.rust-lang.org/book/

Open Source Initiative. (z.d.). *The Open Source Definition*. Geraadpleegd op 26 oktober 2026, van https://opensource.org/osd

The Rust Project. (z.d.). *The Rust Reference*. Geraadpleegd op 26 oktober 2026, van https://doc.rust-lang.org/reference/

World Wide Web Consortium. (2024, 12 december). *Web Content Accessibility Guidelines (WCAG) 2.2*. https://www.w3.org/TR/WCAG22/


## Aanvulling na privacycorrectie
Lees LOCAL-PRIVACY-CONTRACT.md voor de bindende scope. Aanvullende kennis: lokale NAS-identiteitskoppeling en beperkte share-rechten, versleuteld transport en client-side backupkluis, sleutelherstel, netwerkovergangen/IPv6/DNS-rebinding, isolatie van AI-runtime en modeldownloader, modelbestandsvalidatie, gecombineerde bestand-/netwerkrechten en herstelpunten buiten de bevoegdheid van het syncaccount. Iedere implementatietaak noemt de relevante LP-acceptatietests.
