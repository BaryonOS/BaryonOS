# Beveiliging, privacy en postkwantumontwerp

Status: beveiligingsontwerp, bijgewerkt 27 oktober 2026. M1–M5 hebben eigen QEMU-testbewijs, M6a heeft nog een open eindpoort en N01 heeft host-/mockbewijs. Er is geen formeel geverifieerde eigen kernel. De eisen hieronder zijn niet automatisch geïmplementeerd of bewezen; de actuele uitvoering staat in PROJECT-STATE.md.

De nieuwe microkernel-/CHERI-/PQC-blauwdruk is gecontroleerd en vertaald in [VERIFIABLE-SECURITY-AND-PQC.md](implementation/VERIFIABLE-SECURITY-AND-PQC.md), met broncorrecties, de werkelijke native buildgrenzen, een N02-proefprofiel voor TLS 1.3/X25519MLKEM768 (RFC 10024) en afzonderlijke bewijs-/integratietaken. Geen eigen hybride KDF/tunnel, stille seL4-kernelwissel of onbewezen CHERI-garantie. Privacy en een soepele verbonden productervaring blijven gezamenlijke eisen.

## 1. Uitgangspunt

Het doel is aantoonbare beperking van schade: een kwaadaardige app mag uitsluitend haar expliciet toegekende middelen gebruiken. Een zelfstandig ontworpen kernel kan dat nastreven, maar wordt niet automatisch veiliger dan bestaande systemen doordat hij nieuw, klein, open source of in Rust geschreven is. Een onbekende kernel begint juist met veel minder praktijkervaring, auditwerk en hardwareondersteuning.

Gebruik geen claims zoals “onkraakbaar”, “AI-proof”, “100% quantumveilig” of “even veilig als macOS” zonder nauwkeurig omschreven meetmethode. Publiceer per beveiligingseigenschap: implementatiestatus, versie, tests, aannames en resterende risico's.

## 2. Wat klopt aan het macOS-voorbeeld?

Apple gebruikt onder meer app-sandboxing: de app krijgt beperkte toegang tot bestanden, netwerk en hardware. Dat verkleint de schade bij compromittering. App Sandbox is verplicht voor distributie via de Mac App Store; de documentatie beschrijft daarnaast hoe bestaande apps het kunnen toevoegen. Dit betekent niet dat alle software op een Mac in volledig gescheiden, ondoordringbare compartimenten draait. System Integrity Protection is een andere laag en beperkt onder meer wijziging van kritieke systeembestanden, ook door processen met beheerdersrechten (Apple, z.d.; Apple, 2021).

Ontwerples: combineer kleine bevoegdheden, procesisolatie, integriteitscontrole, veilig bijwerken en herstel. Een sandbox is geen bewijs dat ontsnappen onmogelijk is. Een microkernel is evenmin synoniem met een veilige sandbox.

## 3. Trust boundaries en trusted computing base

Een trust boundary is een grens waar invoer, code of rechten opnieuw moeten worden gecontroleerd. De trusted computing base (TCB) is alle code en apparatuur waarop een bepaalde beveiligingsgarantie vertrouwt. “Kleine kernel” en “kleine TCB van het hele product” zijn verschillende zaken.

Voorgesteld ontwerp:

| Onderdeel | Vertrouwen en bevoegdheid | Ontwerpeis |
|---|---|---|
| Firmware, bootpad en CPU | Onderliggende vertrouwensbasis | Exacte platformaannames vastleggen; geen bescherming tegen kwaadaardige firmware beloven |
| Kernel | Geheugen, scheduling, interrupts, IPC en capabilitycontrole | Zo klein mogelijk; geen browser, netwerkprotocolparser of complexe bestandsparser in de kernel |
| Initiële beheerder | Verdeelt bij opstart middelen en bevoegdheden | Tijdelijke brede rechten; splitsing en beperking na initialisatie; beleid versioneren |
| Drivers | Alleen toegewezen apparaat en buffers | Afzonderlijke processen waar haalbaar; DMA-beperking vóór echte DMA-apparaten worden vertrouwd |
| Systeemdiensten | Eigen geïsoleerde adresruimte | Alleen specifieke IPC-endpoints en expliciete overdracht van middelen |
| Apps | Standaard geen algemene toegang | Eigen opslag; toestemming via systeemportalen voor extra bestanden/apparaten/netwerk |
| Updater en sleutelbeheer | Zeer gevoelige rollen | Aparte verantwoordelijkheden en sleutels; geen algemene apprechten |
| Eventuele AI-agent | Onbetrouwbare app met delegatie | Hetzelfde rechtenmodel als elke app, plus korte taakgebonden machtigingen |

Een capability is in dit voorstel een door de kernel beheerde verwijzing naar een object met rechten. Een proces mag niet door het raden van een nummer nieuwe rechten krijgen. seL4 is een bruikbare primaire leerbron voor dit concept, maar hier wordt geen seL4-code of verificatiegarantie overgenomen (seL4 Project, z.d.-a).

Voor de eigen capability-API moeten vóór implementatie zijn gespecificeerd: aanmaak, overdracht, beperking van rechten, intrekking, objectlevensduur, generatiecontrole bij hergebruik van handles, opruiming bij procesuitval en bescherming tegen de confused-deputy-fout. Die laatste fout ontstaat wanneer een bevoegde dienst onbedoeld haar eigen rechten inzet namens een onbevoegde aanvrager. Maak duidelijk welke identiteit/rechten bij iedere IPC-operatie bepalend zijn. Beperk ook aantallen objecten, wachtrijen en CPU-tijd: toegangscontrole alleen voorkomt geen uitputting.

## 4. Threat model en toetsbaar bewijs

Beschermde middelen: gebruikersdocumenten, privésleutels, accountgegevens, camera/microfoon, apparaatbesturing, update-integriteit, beschikbaarheid en herstelbaarheid. Tegenstanders: kwaadaardige app, netwerkpartij, besmette dependency, gestolen account of sleutel, kwaadwillende bijdrager, fysiek aanvaller en geautomatiseerde of AI-ondersteunde aanvaller.

| Risico | Voorstel voor beheersing | Minimale bewijsproef |
|---|---|---|
| App leest geheugen van andere app/kernel | Hardwarematige adresruimte-isolatie en gecontroleerde mappings | Proces A probeert lezen/schrijven/uitvoeren in B en kernel; A wordt gestopt, B blijft functioneren |
| App vervalst of hergebruikt capability | Kernelbeheer, type/recht/generatiecontrole | Willekeurige, ingetrokken, verkeerde en opnieuw gebruikte handles geven geen toegang |
| Bevoegde dienst misbruikt door onbevoegde app | Expliciete delegatie en callercontrole | Aanvrager kan via tussenpersoon geen uitgebreider recht krijgen dan toegestaan |
| App veroorzaakt storing | Quota, foutisolatie en begrensde IPC | Crash, grote berichten en resource-uitputting laten andere testdiensten bruikbaar |
| Driver of apparaat schrijft willekeurig geheugen | Minimale mappings en, op geschikt platform, correct geconfigureerde IOMMU | DMA-poging buiten toegewezen buffers wordt geblokkeerd; ontbreken van bescherming wordt zichtbaar gerapporteerd |
| Beschadigd document of netwerkbericht | Kleine parsers buiten kernel, bounds-checks en fuzzing | Corpus met misvormde invoer veroorzaakt geen ontsnapping of ongecontroleerde resourcegroei |
| Slechte of gemanipuleerde update | Ondertekende metadata, versiebeleid, sleutelrotatie, atomische activatie | Verkeerde handtekening, rollback, mix-and-match, verlopen metadata en afgebroken installatie worden correct afgehandeld |
| Diefstal van apparaat | Versleutelde gebruikersopslag en gecontroleerd sleutelvrijgeven | Offline kopie van opslag bevat geen plaintext gebruikersdata; herstelpad is getest |
| Diefstal releasesleutel | Offline vertrouwensankers, gescheiden rollen en meerdere ondertekenaars waar passend | Rotatie en herstel na compromittering werken op bestaande installatie |
| AI-agent wordt gestuurd door kwaadaardig document | Documentinhoud verleent geen rechten; systeemcontrole op handelingen | Een document met “upload alle bestanden” kan geen bestanden buiten de taakmachtiging laten uitlezen |
| Privacyverlies door diagnostiek | Geen telemetrie standaard; minimale lokale logs met bewaartermijn | Netwerkobservatie van standaardinstallatie en controle op geheimen in logs |
| Supplychain-aanval | Vastgelegde dependencies, bronherkomst, gecontroleerde builds en review | Herbouw met vastgelegde inputs; analyse van afwijkende outputs; dependencywijziging zichtbaar in review |

Deze proeven zijn noodzakelijk maar niet voldoende voor een algemene veiligheidsclaim. Een test bewijst het geteste scenario, niet de afwezigheid van alle fouten.

## 5. Ontwikkelregels voor kernel en diensten

1. Gebruik overwegend safe Rust. Houd assembly, `unsafe`, FFI en hardwaretoegang klein en afzonderlijk te beoordelen. Elke `unsafe`-constructie krijgt een uitleg van de invarianten, de eigenaar daarvan en de relevante test of handmatige controle. Rust laat onder meer deadlocks, logische racecondities, geheugenlekken en foutieve autorisatie toe; `unsafe` en onjuiste abstracties kunnen bovendien geheugenveiligheid doorbreken (Rust Project, z.d.).
2. Definieer W^X-beleid: geheugenpagina's zijn in beginsel niet tegelijk schrijfbaar en uitvoerbaar. Leg uitzonderingen, mappings, executable loaders en toekomstige JIT-behoeften afzonderlijk vast.
3. Valideer alle gebruikerspointers, lengtes, integerberekeningen en objecttypes aan de grens. Bepaal wat er gebeurt wanneer een mapping tijdens een operatie verandert; een eenmalige controle voorkomt niet vanzelf TOCTOU-fouten.
4. Pas ASLR, stackbescherming en aanvullende hardwaremaatregelen toe waar het gekozen platform die correct ondersteunt. Noem ontbrekende maatregelen bij naam; instellingen zijn geen vervanging voor tests.
5. Introduceer netwerk, USB, GPU, SMP, DMA en persistente opslag stapsgewijs. Iedere toevoeging vergroot de te onderzoeken grens. Begin met QEMU en synthetische data.
6. Gebruik modelchecking of formele specificaties voor afgebakende invarianten, bijvoorbeeld capability-overdracht en objectlevensduur. Het slagen van een abstract model bewijst niet automatisch dat de implementatie hetzelfde doet.
7. Maak fouten in beveiligingsgevoelige subsystemen zichtbaar en fail closed. Ontwerp tegelijk een herstelpad zodat één onbruikbare update niet permanent alle data ontoegankelijk maakt.

seL4 toont hoe sterk formele verificatie kan zijn, maar documenteert ook aannames rond bootcode, assembly, hardware en DMA. Timingkanalen vallen niet onder de genoemde vertrouwelijkheidsdekking. De eigen kernel heeft eigen specificaties en bewijzen nodig; seL4 noemen of hetzelfde architectuurtype gebruiken draagt geen bewijs over (seL4 Project, z.d.-b).

## 6. Bescherming tegen AI-ondersteunde aanvallen

Behandel AI als mogelijke versneller van bestaande aanvalsklassen: meer pogingen, sneller zoeken naar fouten, geloofwaardige misleiding en aanvallen op ontwikkeltools. Het OS hoeft geen “AI-aanval” te herkennen om een ongeautoriseerde systeemaanroep te weigeren.

Voor een optionele assistent geldt uitsluitend lokale inferentie op het apparaat, met netwerktoegang geweigerd en zonder cloudfallback. Modeldownloads lopen via een afzonderlijke dienst zonder document- of contexttoegang. Zie LOCAL-PRIVACY-CONTRACT.md. Verder:

- Geen permanente adminrechten, algemene shellmachtiging of onbeperkte toegang tot de thuismap.
- Machtiging heeft een concrete scope: objecten, actie, bestemming, duur en eventueel maximumbedrag/hoeveelheid. De kernel/dienst dwingt dit af; de belofte in de modelprompt is onvoldoende.
- Scheid lezen van uitvoeren en lezen van verzenden. Toegang tot een document is geen toestemming om het te uploaden.
- Geef betrouwbare systeemdialogen voor een nieuwe gevoelige bevoegdheid. Een app mag toestemmingstekst niet als systeemmelding kunnen vervalsen.
- Log uitvoeringsbesluiten lokaal zonder documentinhoud of geheimen onnodig te bewaren. Maak toestemming intrekbaar.

Voor Codex tijdens ontwikkeling: geen productiesleutels, echte gebruikersdata of releasebevoegdheden in de werkruimte. Tooluitvoer, issues, comments en dependencydocumenten kunnen onbetrouwbare instructies bevatten. Laat een onafhankelijke reviewer de gewijzigde code en bewijsvoering beoordelen; dezelfde agent nogmaals laten zeggen dat het veilig is, is geen onafhankelijke audit.

## 7. Postkwantumcryptografie: wat wel en niet wordt beschermd

Postkwantumcryptografie (PQC) draait op gewone computers en probeert cryptografische bescherming te bieden tegen aanvallers met een grote quantumcomputer. Het maakt een fout in de kernel, verkeerd ingestelde rechten, een gestolen sleutel of een gelekt wachtwoord niet onschadelijk.

| Standaard | Algoritme | Rol in een mogelijk OS |
|---|---|---|
| FIPS 203 | ML-KEM | Een gedeeld geheim vaststellen; geen losse bestandsversleutelaar en geen handtekening |
| FIPS 204 | ML-DSA | Digitale handtekeningen, bijvoorbeeld in een passend ontworpen updateprotocol |
| FIPS 205 | SLH-DSA | Hashgebaseerde digitale handtekeningen; andere afwegingen voor omvang en snelheid |

Deze standaarden zijn op 13 augustus 2024 gepubliceerd. NIST formuleert de weerstand als gebaseerd op de huidige kennis, geen onvoorwaardelijke garantie. Controleer bij daadwerkelijke implementatie de geldende specificatie, errata, parameterkeuze, libraryversie en protocolondersteuning; de geopende FIPS 203- en 204-pagina's verwijzen naar errata (NIST, 2024a, 2024b, 2024c).

Aanbevolen ontwerpbeleid:

1. **Geen eigen cryptografische algoritmen of zelfbedachte hybride combinaties.** Hergebruik onderhouden, onafhankelijk beoordeelde implementaties die de gekozen standaardversie ondersteunen. Een eigen OS hoeft geen eigen wiskunde te gebruiken.
2. **Maak een crypto-inventaris.** Registreer algoritmen, sleutels, certificaten, protocollen, trust anchors, bewaartermijnen en afhankelijkheden van firmware/TPM/cloud. Begin vóór het eerste versleutelde netwerkprotocol.
3. **Ontwerp crypto-agility.** Geef sleutels en metadata expliciete algoritme- en formaatversies, met een beperkte lijst toegestane opties. Onderhandel nooit onbeveiligd terug naar een zwakker algoritme. Test intrekking en migratie.
4. **Gebruik PQC in een passend, gespecificeerd protocol.** “ML-KEM toegevoegd” is geen bewijs dat authenticatie, kanaalbinding, downgradebescherming en sleutelafleiding kloppen. Hybride inzet kan nuttig zijn, maar vraagt een vastgesteld protocol en onafhankelijke beoordeling; geen algoritmen zelf aan elkaar plakken.
5. **Bescherm langdurig vertrouwelijke data vroeg.** Opgeslagen netwerkverkeer kan later worden aangevallen. Versleuteling van de schijf vervangt daarom geen toekomstbestendig transportprotocol, en omgekeerd.
6. **Blijf symmetrische encryptie gebruiken waar passend.** AES-256 is een kandidaat voor veiligheidsmarge in een geschikt opslag- of berichtformaat. Modus, nonces, authenticatie, sleutelafleiding en sleutelbeheer moeten afzonderlijk kloppen. ML-KEM vervangt die onderdelen niet. NIST behandelt de quantumdreiging voor symmetrische crypto anders dan voor klassieke public-key-systemen (NIST, 2026).
7. **Test de hele keten.** Bekende testvectoren, interoperabiliteit, misvormde sleutels/handtekeningen, entropyfouten, sleutellevensduur en sidechannels. Een algoritmenaam op een checklist is onvoldoende.

Een PQ-ondertekend systeemimage garandeert geen PQ-veilige secure-bootketen als firmware alleen een klassieke kwetsbare handtekening afdwingt. Benoem expliciet welk deel van boot, herstel, firmware en updates beschermd is. Geen end-to-end-claim zolang één vertrouwensanker niet onder de claim valt.

Veilige entropy is een apart ontwerpprobleem. Geen sleutels afleiden uit tijd, vaste seeds of alleen een voorspelbare emulatorbron. Bij gebrek aan geschikte entropy geen cryptografische sleutelgeneratie uitvoeren. Een testseed hoort uitsluitend in duidelijk gemarkeerde tests.

## 8. Updates, rollback en herstel

The Update Framework is een referentie voor gescheiden ondertekenrollen en bescherming tegen onder meer oude updates, bevroren update-informatie en inconsistente metadata. Gebruik de specificatie en een geschikte implementatie; claim geen TUF-compatibiliteit op basis van alleen een vergelijkbaar ontwerp (The Update Framework, z.d.-a, z.d.-b).

Het OS-voorstel combineert atomische activatie met een geteste vorige installatie. Maak onderscheid tussen een geautoriseerde herstelactie en een aanvaller die een kwetsbare oude versie aanbiedt. Definieer de laagst toegestane beveiligingsversie, autorisatie voor uitzonderingen, sleutelrotatie en wat er gebeurt wanneer de klok onbetrouwbaar is. Test stroomuitval vóór, tijdens en na activatie en controleer behoud van gebruikersdata. Kopieer nooit simpelweg een halve systeemupdate over de actieve installatie.

Reproduceerbare builds helpen om bron en output te vergelijken, maar bewijzen niet dat de broncode vrij van kwetsbaarheden is. Ondertekening bewijst herkomst onder de gebruikte sleutel, niet dat de software goed is.

## 9. Privacy en Europese autonomie

Herziene gebruikersvereiste (27 oktober 2026): OS-gegevens blijven lokaal of gaan end-to-end versleuteld naar een expliciet gekoppelde eigen NAS of NAS van vrienden/familie. Remote bereik via internet vereist afzonderlijke toestemming en dezelfde identiteits-/rechtencontrole als LAN. Geen leverancierscloud, leveranciersaccount, stille relay/fallback, sleutelbewaarservice of OS-telemetrie. NAS-beheerder mag capaciteit intrekken en zijn ciphertextkopieën wissen zonder toestemming van de gast; inhoudssleutels en leesrechten blijven bij de gebruiker. Beheerder kan beschikbaarheid niet garanderen en ziet noodzakelijke gebruiksmetadata. Lokale taken blijven zonder NAS werken. Eigen herstelkopieën en sleutelherstel zijn afzonderlijke gebruikersverantwoordelijkheden. Zie LOCAL-PRIVACY-CONTRACT.md revisie 3 en implementation/NAS-SHARING-AND-INTERNET.md.

Europese hosting alleen is geen bewijs van privacy; open source alleen evenmin. Houd ook afhankelijkheden van stores, signingdiensten, buildinfrastructuur, firmware, documentatie en externe authenticatie zichtbaar. Geografie is geen kwaliteitskeurmerk voor cryptografie: beoordeel openbare standaarden op inhoud, implementatie en onafhankelijke analyse.

Bij toekomstig gebruik door instellingen is een aparte beoordeling van concrete gegevensverwerkingen, wetgeving en verantwoordelijkheden nodig. Dit document doet geen claim van AVG- of andere juridische conformiteit.

## 10. Een quantumcomputer waarop het OS draait

De praktische uitbreidingsroute is een klassieke computer die onder dit OS draait en een QPU als gespecialiseerde rekenvoorziening aanstuurt. IBM beschrijft systemen met klassieke runtime-servers, qubitbesturing en gecombineerde klassieke/quantumworkflows (IBM, z.d.).

“De kernel direct op qubits draaien zoals op een x86-CPU” is daarom geen bruikbaar eerste portabiliteitsdoel. Een eventuele latere quantumjobdienst mag onder dit privacycontract uitsluitend een eigen lokaal beheerde QPU aansturen; externe quantumcloudverwerking valt buiten de OS-integratie. Dit is een expliciet onderzoeksspoor vanaf de ontwerpstart, met Q0–Q4 in QUANTUM-RESEARCH.md. De eigen kernel als klassieke besturingslaag van een hybride quantumcomputer is een afzonderlijk doel; hardware-integratie volgt na specifieke interface- en controllerproeven. Toegangsrechten en bescherming van invoer/resultaten blijven vereist. Compatibiliteit met een concrete QPU moet per platform worden aangetoond. Voor dit nog te bouwen OS: dit kan ik niet verifiëren.

## 11. Claims per ontwikkelfase

| Fase | Wat mag worden gezegd na bewijs | Wat nog niet |
|---|---|---|
| Bootdemo | Start in de benoemde emulatorconfiguratie | Veilig voor gevoelige gegevens |
| Procesisolatie | Genoemde negatieve isolatietests slagen op deze commit | Alle sandboxes zijn onontsnapbaar |
| Capabilityprototype | Gespecificeerde overdracht/intrekking werkt in tests | Formeel bewezen autorisatie |
| Crypto-integratie | Benoemde implementatie, vectoren en interoperabiliteit werken | Het gehele OS is quantumveilig of FIPS-gevalideerd |
| Externe review | Genoemde onderdelen zijn op datum X beoordeeld; bevindingen zijn inzichtelijk | Alle toekomstige wijzigingen zijn automatisch veilig |

Gebruik het prototype uitsluitend met wegwerpdata totdat de relevante bescherming aantoonbaar is geïmplementeerd, herhaalbaar getest en beoordeeld. Plan onderhoud, beveiligingsmeldingen en updates als permanente projectonderdelen.

## Bronnen (APA 7)

Apple. (z.d.). *Protecting user data with App Sandbox*. Apple Developer Documentation. Geraadpleegd op 26 oktober 2026, van https://developer.apple.com/documentation/security/protecting-user-data-with-app-sandbox

Apple. (2021, 18 februari). *System Integrity Protection*. Apple Platform Security. https://support.apple.com/en-gw/guide/security/secb7ea06b49/web

IBM. (z.d.). *IBM Quantum hardware and roadmap*. Geraadpleegd op 26 oktober 2026, van https://www.ibm.com/quantum/hardware

National Institute of Standards and Technology. (2024a). *Module-lattice-based key-encapsulation mechanism standard* (FIPS 203). https://doi.org/10.6028/NIST.FIPS.203 — Publicatiepagina en errata: https://csrc.nist.gov/pubs/fips/203/final

National Institute of Standards and Technology. (2024b). *Module-lattice-based digital signature standard* (FIPS 204). https://doi.org/10.6028/NIST.FIPS.204 — Publicatiepagina en errata: https://csrc.nist.gov/pubs/fips/204/final

National Institute of Standards and Technology. (2024c). *Stateless hash-based digital signature standard* (FIPS 205). https://doi.org/10.6028/NIST.FIPS.205

National Institute of Standards and Technology. (2026, 5 augustus). *Post-quantum cryptography: FAQs*. https://csrc.nist.gov/projects/post-quantum-cryptography/faqs

Rust Project. (z.d.). *What unsafe Rust can do*. The Rustonomicon. Geraadpleegd op 26 oktober 2026, van https://doc.rust-lang.org/nomicon/what-unsafe-does.html

seL4 Project. (z.d.-a). *Capabilities*. seL4 Documentation. Geraadpleegd op 26 oktober 2026, van https://docs.sel4.systems/Tutorials/capabilities.html

seL4 Project. (z.d.-b). *What the proofs assume*. Geraadpleegd op 26 oktober 2026, van https://sel4.systems/Verification/assumptions.html

The Update Framework. (z.d.-a). *Roles and metadata*. Geraadpleegd op 26 oktober 2026, van https://theupdateframework.io/docs/metadata/

The Update Framework. (z.d.-b). *Security*. Geraadpleegd op 26 oktober 2026, van https://theupdateframework.io/docs/security/

