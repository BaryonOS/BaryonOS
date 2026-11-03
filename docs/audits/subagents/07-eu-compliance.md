# AUDIT-07 - Technische Europese compliance-gap-analyse

**Resultaat:** de bevroren repository bevat conditionele EU-ontwerpcriteria en
bruikbaar, brongebonden historisch updatebewijs, maar geen compleet technisch
release-/conformiteitsdossier. Juridische toepasselijkheid, actuele primaire
EU-bronnen, operationele kwetsbaarhedenafhandeling, release-SBOM en native
toegankelijkheidsacceptatie zijn niet voldoende aangetoond voor aftekening.
Dit is **geen juridische conformiteitsverklaring**, securityaudit of verklaring
dat het onderzoeksproject wetgeving overtreedt.

## 1. Opdracht, bron en grenzen

- Letterlijke opdracht: "Maak een technische Europese compliance-gap-analyse.
  Geef geen juridische conformiteitsverklaring."
- Uitgevoerd op 3 november 2026; roltekst
  `.github/agents/os-eu-compliance-reviewer.agent.md:10-29` gelezen als
  instructies voor een gewone chatsessie, niet als technisch geladen customagent.
- Beginbranch: `simonvanieperen-improved-invention`, niet detached; begin-HEAD
  `69809c2cb53dfb3bd77abe5352892810fba6e82d`. Eigen werkboom was schoon.
- Op expliciete opdracht overgeschakeld naar detached HEAD
  `beafdedfc188154fe7285848fbd411e651013f54`. Lokale bronbranch
  `copilot/add-os-specialist-agents` wees bij controle naar diezelfde commit.
  Geen bewijs uit andere werkbomen gecombineerd; de werkboomlijst is alleen
  voor situering bekeken. De hoofdcheckout is niet gelezen of gewijzigd.
- Alleen deze rapportage wordt geschreven:
  `docs/audits/subagents/07-eu-compliance.md`. Geen productcode, scoreblad,
  projectstatus, commits, pushes, PR's of andere rapporten gewijzigd.
- Alleen lokale repository-inhoud gebruikt. Geen externe bron opgehaald,
  GitHub-status opgevraagd, installatie/download, build, test, nieuwe subagent,
  echte gegevens, netwerkproef of hardwarehandeling uitgevoerd.
- Gelezen instructies: root-`AGENTS.md`, `docs/AGENTS.md`, het rolprofiel en
  de geladen `os-dev`-skill. De lijst gevolgde repositoryinstructies bevat
  geen geneste `AGENTS.md` onder het toegewezen rapportpad.

De specifieke auditopdracht sluit de algemene commit-/voortgangs- en
testwerkstroom hier uit. Deze analyse verleent geen toestemming voor een
release, marktintroductie, fysieke installatie of aanvullende gegevensverwerking.
Owners hieronder zijn **voorgestelde verantwoordelijke rollen**, geen reeds
benoemde mensen, actieve agents of aanvaarde juridische verantwoordelijkheid.

## 2. Bewijsmethode

| Niveau | Betekenis in deze audit |
|---|---|
| E0 | Ontbrekend of alleen ambitie |
| E1 | Ontwerp/documentatie |
| E2 | Hostprototype of simulatie |
| E3 | Native broncode |
| E4 | Native positieve uitvoering |
| E5 | Native negatieve uitvoering |
| E6 | Herhaalde CI-/releasegate |
| E7 | Gedocumenteerde fysieke hardwaretest |
| E8 | Onafhankelijke menselijke audit of formele evaluatie |

Een niveau geldt uitsluitend voor de benoemde eigenschap en bronversie.
Een historisch E5-deelbewijs promoveert niet tot E5 voor de huidige complete
release. Een workflowbestand toont een ingericht gatepad, geen geslaagde
E6-uitvoering. Deze audit levert uitsluitend eigen statische leescontroles;
zij voegt geen E4-E8-uitvoering toe.

### Concrete lokale bewijsankers

| Ref. | Bestand en regels | Wat het wel en niet bewijst |
|---|---|---|
| S1 | `docs/implementation/EUROPE-AND-RELEASE.md:3-18,22-35,39-54` | Conditionele juridische beschrijving, eigen EU-01-08-criteria, licentievoorstel en verwijzingen. E1; geen geldende wetstekst of uitvoering. |
| S2 | `docs/LOCAL-PRIVACY-CONTRACT.md:1-21,54-114,154-183` | Revisie 4; lokale/E2EE-opslag, optionele ciphertextprovider, metadata-, verwijder- en herstelgrenzen, LP-01-31. Bindend ontwerp, geen algemene AVG-acceptatie. |
| S3 | `SECURITY.md:2-8` | Expliciet experimenteel; geen ondersteunde releases, securitycontact, responstermijn of productieondertekeningssleutels. Rechtstreekse verklaring van governancegaten. |
| S4 | `docs/verification/B03-ACCEPTANCE.md:9-41,57-116` | Historische eigen-kernel/QEMU-updateproeven met TEST-ankers, positieve en negatieve gevallen en beperkingen. Geen heruitvoering op auditbron. |
| S5 | `docs/verification/B03-KEY-POLICY-1.md:3-45` | Latere herhaling op `69d5c3acce8a97e38895f284f13d3de3dbf4a5d9`, v3-fingerprint `df5a6f39dc081340d8d40dd6c1ba1eaa7bc40ecf372e728ba1df67120b3a73b9`; rapporteert 18 native stages. Actuele-head-CI en menselijke review blijven afzonderlijk nodig. |
| S6 | `crates/update-core/src/policy.rs:19-63`; `crates/update-core/src/test_policy.rs:1-44`; `boot/loader/src/signed.rs:15,170`; `sdk/examples/update/src/service.rs:12-22` | E3: versievensters, intrekking en beide handtekeningen in gedeeld beleid; loader en native dienst gebruiken expliciete openbare QEMU-testpolicy, niet productieautoriteit. |
| S7 | `.github/workflows/signed-updates.yml:3-10,17-53` | Ingericht PR/main-gatepad met native updateharnas en artifactupload. Geen eigen waarneming van actuele CI-resultaten. |
| S8 | `docs/implementation/B03-SIGNED-UPDATES.md:163-170,200-214` | Geen firmwareverificatie van loader, productieceremonie of geauthenticeerde beleidsdistributie bewezen; versievensters zijn geen kalenderverval. Later sleutelbeleid niet als ontbrekende implementatie wegstrepen. |
| S9 | `docs/audit/os-completeness/accessibility-gaps.md:5-31`; `docs/implementation/UX-MATERIAAL-SAVE-RECOVERY.md:191-193,246,273-274` | E1/afgebakend prototypebewijs; native accessibility-tree/event-IPC, echte schermlezer en menselijke taakacceptatie niet aangetoond. App-lokale semantiek is geen geïntegreerde hulptechnologie. |
| S10 | `docs/FULL-TEST-RELEASE.md:95-125,129-135,172,210-215,225-233` | Expliciete cumulatieve bewijseisen; alle 62 volledige releaserijen open. Inclusief toegankelijkheid, keys, privacy, onafhankelijke controle, duur/hardware en reproduceerbaar pakket. |
| S11 | `docs/third-party/README.md:9-32`; `LICENSE-PLAN.md:2-14` | Concrete r-efi-versie/checksum/licentieroute, afzonderlijke toolchain/firmware en licentievoorstel. Expliciet geen compleet distributie-SBOM of toegepaste projectlicentie. |
| S12 | `docs/DECISIONS.md:615`; `docs/PRODUCT-AND-SOVEREIGNTY.md:81-87,104-112`; `docs/PROGRESS.md:363` | EU-criteria zijn projectbesluit; web-WCAG is geen native conformiteit; governance en R03-releaseonderhoud blijven open. |

Huidige bron, opgeslagen historie en eigen waarneming blijven gescheiden:
S6/S7 zijn bekeken broncode/configuratie van de auditcommit; S4/S5 beschrijven
oudere uitvoeringen; S9 is mede een eerdere repository-audit, hier ondersteund
door de implementatietekst en open FR-U05. Geen binaire hashes, volledige
testtranscripten of historische bronvingerafdrukken opnieuw gevalideerd.

## 3. Primaire bronnen en actualiteitsgrens

Het profiel vraagt actuele primaire EU-bronnen; de opdracht verbiedt externe
raadpleging. Daarom is voor **alle onderstaande juridische verwijzingen de
actualiteit, volledigheid en concrete toepasselijkheid NIET GEVERIFIEERD**.
Een lokaal genoteerde raadpleegdatum is geen eigen raadpleging door deze audit.
De titels, organisaties, datums en URL's hieronder zijn repo-metadata, geen
bevestiging van de inhoud op die URL's.

| Bron-ID | Reeds aanwezige primaire verwijzing | Lokaal aantoonbare grens |
|---|---|---|
| P-AVG | S1:45-46: Europese Commissie, *Data protection explained* (z.d.); EDPB, *Guidelines 07/2020*, versie 2.1 (repo vermeldt 2022), `commission.europa.eu/law/law-topic/data-protection/data-protection-explained_en` en `edpb.europa.eu/system/files/2023-10/EDPB_guidelines_202007_controllerprocessor_final_en.pdf` | Links/uitleg en richtsnoeren; geen in deze audit gecontroleerde geconsolideerde AVG-tekst, nationale regels of verwerkingsbeslissing. |
| P-CRA | S1:47-49: Europese Commissie, *CRA - Open source*, *Cyber Resilience Act*, *Reporting obligations*, repo-datums 15 oktober/7 september/11 september 2026; `digital-strategy.ec.europa.eu/en/policies/cra-open-source`, `/cyber-resilience-act`, `/cra-reporting` | Geen gecontroleerde volledige verordening, classificatiebijlagen, uitvoeringsregels of meldportaalprocedure. |
| P-EAA | S1:50-51: Commissie, *European Accessibility Act* (z.d.); Parlement/Raad, Richtlijn (EU) 2019/882 (2019), artikel 2; `eur-lex.europa.eu/eli/dir/2019/882/oj` | Lokaal staat een verwijzing, geen vastgestelde nationale toepasselijkheid of gecontroleerde volledige tekst. |
| P-ICT | S1:52-53: ETSI, *EN 301 549 V3.2.1* (2021), `etsi.org/deliver/etsi_en/301500_301599/301549/03.02.01_60/en_301549v030201p.pdf`; Commissie, *Web Accessibility Directive - Standards and harmonisation*, repo-datum 5 mei 2025 | Ontwerpbaseline, geen vastgestelde huidige harmonisatie voor dit product/rechtsregime. |
| P-WCAG | `docs/PRODUCT-AND-SOVEREIGNTY.md:124`: W3C, *WCAG 2.2*, repo-datum 12 december 2024, `w3.org/TR/WCAG22/` | Webstandaardverwijzing, geen complete native OS-norm of juridische verklaring. |
| P-LIC | S1:54 en `LICENSE-PLAN.md:15-16`: Mozilla, MPL 2.0/FAQ | Primaire licentie-uitgever, geen EU-wetgever; voorstel is geen licentieverlening. |
| P-OVERIG | NIS2, ePrivacy, RED/CE, consumentenrecht | De gerichte zoekactie vond deze onderwerpen in het rolprofiel maar geen overeenkomstig uitgewerkt primair bronnenregister in de onderzochte actieve EU-/productdocumenten. Dit is een begrensde zoekbevinding, geen bewijs dat nergens in de gehele repository ooit een losse verwijzing staat. |

**Deadlinebron:** S1:18 bevat CRA-datums en meldvensters; S1:15 bevat een
EAA-marktdatum. Ze zijn uitsluitend als historische lokale claims vastgesteld,
niet als geldige deadlines voor Nieuw OS bevestigd. Dit rapport neemt die
datums/termijnen niet over als handelingsdeadline. Voor de overige onderwerpen
is in de onderzochte stukken geen bruikbare deadlinebron vastgesteld.
Een bevoegde menselijke beoordelaar moet eerst primaire teksten, rol,
productvorm en eventuele nationale implementatie controleren.

## 4. Gapmatrix

Prioriteiten: **BLOKKEREND** verhindert de genoemde aftekening/publicatieclaim,
niet verder veilig onderzoekswerk; **KRITIEK** zou een direct zwaar technisch
risico met voldoende bewijs vereisen en wordt hier niet toegekend;
**HOOG/MIDDEL/LAAG** ordenen vervolgwerk; **ONBEKEND** betekent onvoldoende
grond voor een impact- of toepasselijkheidsbesluit. Dit zijn technische
auditprioriteiten, geen wettelijke categorieën of bewezen kwetsbaarheden.

| ID / prioriteit | Onderwerp, bron en toepasselijkheid | Technisch bewijs | Ontbrekend bewijs / acceptatievoorstel | Voorgestelde eigenaar | Menselijke/juridische beoordeling |
|---|---|---|---|---|---|
| EU07-01 **BLOKKEREND** | CRA/productrol; P-CRA, S1:13-14. **Afhankelijk van productvorm**: commercieel aanbod, naam/rechtspersoon en fabrikant/steward/andere rol zijn niet vastgesteld. Geen FOSS-vrijstelling aannemen. | E1 conditionele matrix; S10 open release. | Gedateerde product-/distributiebeschrijving; rol- en classificatiebesluit met primaire grondslag; gekozen beoordelingsroute, technische dossierindex en verantwoordelijke. | Producteigenaar + releasebeheer | Jurist bepaalt rol, bereik, eventuele uitzonderingen en route. Geen automatische fabrikant- of stewardkwalificatie. |
| EU07-02 **HOOG** | Lifecycle security/updates; P-CRA, eigen EU-07; S4-S8. **Waarschijnlijk van toepassing** bij beoogde productdistributie; juridische detailplicht nog te toetsen. | E3 native verificatiebeleid; historisch E4/E5: updates, corruptie, downgrade, onderbreking en latere testkeyrotatie/intrekking. S7 gateconfiguratie; geen nieuw E6-verdict. | Productie-trustanchors, sleutelcustody/ceremonie, geauthenticeerde beleidsupdates en loader-trustketen; herstel/rotatie na compromis; geïntegreerde update-UX en volledige productdatabehoudproef. Exacte finale bron/image opnieuw positief en negatief testen. | Beveiliging/boot + releasebeheer | Beoordeling lifecycle-risico's, verantwoordelijkheden en toepasselijke producteisen; testsleutels mogen geen productiebelofte dragen. |
| EU07-03 **BLOKKEREND** | Vulnerability intake/disclosure/melding; P-CRA, S1:33, S3. **Afhankelijk van productvorm** voor wettelijke melding; technisch meldpunt nodig volgens eigen openbare-testreleasebeleid. | E1 beleid benoemt afwezigheid van contact/responstermijn. Geen operationele keten aangetoond. | Echt privaat kanaal, benoemde triage/vervanging, veilige intake/retentie, severity- en patchbesluit, advisories en bereikbare eigenaar; synthetische oefening van intake tot fix/disclosure en conditionele wettelijke melding. | Maintainer/security response + juridische contactpersoon | Bevestig bevoegde ontvangers, toepasselijke meldtriggers/termijnen en privacy van melders. Geen fictief adres of onbevestigde deadline invoeren. |
| EU07-04 **HOOG** | AVG/GDPR; P-AVG, S1:11-12,26-29, S2. **Juridisch te beoordelen** per verwerking; lokale softwarelevering, support, professionele inzet en NAS-beheer zijn niet dezelfde rol. | E1 dataminimalisatie, capabilities, key-/providergrenzen en lokale AI-eisen. S2 noemt apart hostencryptie/simulaties (E2); geen integrale native AVG-acceptatie. | Concreet gegevensstromenregister met doel, categorie, actor, ontvanger, locatie, retentie, grondslagvraag en verwijder/exportpad; aparte support-/log-/repository-/NAS-administratie. Native negatieve tests op helperroutes, ingetrokken grants en datalekken. | Privacy/product + betrokken dienstowners | Jurist/mens bepaalt rollen, grondslagen, informatieplichten, rechten, verwerkersrelaties en noodzaak van effectbeoordeling. Encryptie is geen automatische vrijstelling of anonimisering. |
| EU07-05 **HOOG** | Verwijderen/export/back-up; P-AVG; eigen EU-04, S2:64-109, S10:210-215. **Juridisch te beoordelen** voor wettelijke rechten; **van toepassing** als eigen producteis. | E1 eerlijke grenzen: NAS-admin wist eigen kopieën, niet lokale/andere kopieën; metadata/timing blijven zichtbaar; sync is geen back-up. | End-to-end native export en verwijdering voor documenten, indexen/caches, AI-afgeleiden, providerobjecten, staging en herstelpunten; retentie-/tombstonebeleid; koude herstart, offline peer, I/O-fout en stale upload zonder valse succesclaim. | Opslag/NAS + privacy | Toets rechten, uitzonderingen/bewaarplichten en ontvangerscommunicatie per actor. NAS-eviction is niet automatisch uitvoering van een AVG-verzoek; geen fysieke secure-eraseclaim. |
| EU07-06 **ONBEKEND** | NIS2; P-OVERIG. **Afhankelijk van productvorm**: sector, entiteit, dienst/bedrijfsgrootte, afnemer en lidstaat onbekend. | E0 voor uitgewerkte NIS2-toepasselijkheidsanalyse; generieke update-/isolatieontwerpen zijn geen NIS2-dossier. | Beschrijf of alleen software wordt geleverd of ook een beheerde dienst/NAS wordt geëxploiteerd; inventariseer sector/organisatie/contractuele keteneisen; maak pas daarna operationele risico-/incidentmatrix. | Product/governance + eventuele dienstexploitant | Jurist bepaalt directe reikwijdte en nationale implementatie. Een NIS2-afnemer maakt iedere softwareleverancier niet zonder meer zelf gereguleerd. Geen onvoorwaardelijk toepasselijk/NVT. |
| EU07-07 **ONBEKEND** | ePrivacy; P-OVERIG, S2:11-21,91-109. **Juridisch te beoordelen** voor feitelijke communicatiediensten, toegang tot eindapparatuur en identifiers; internetfunctie alleen bepaalt dit niet. | E1 geen stille relay/telemetrie, gescheiden downloader; metadatarest expliciet. Geen complete communicatiedienst-/identifierinventaris of eigen verkeersobservatie. | Inventariseer discovery, pairing, credentials/IDs, opslag/toegang op apparaten, update-/providerverkeer, eventuele webinterfaces en derdenappgrenzen; koppel aan doel en keuze-/informatiepad. | Netwerk/NAS + privacy | Bepaal dienstdefinities, uitzonderingen, nationale regels en waar toestemming of andere voorwaarden spelen. Een appcapability is niet automatisch juridisch geldige toestemming. |
| EU07-08 **BLOKKEREND** | Toegankelijkheid; P-EAA/P-ICT/P-WCAG, S1:15-16,30, S9, FR-U05. **Waarschijnlijk van toepassing** voor het beoogde consumenten-OS; concrete wettelijke plicht **afhankelijk van productvorm**. | E1 criteria; beperkte E2-web-/hostchecks en native app-lokale semantiek; geen geïntegreerde schermlezertest aangetoond. | Native semantische boom/events en hulptechnologie; complete setup/login/grants/update/recovery met toetsenbord/schermlezer; tekstschaal/focus/contrast/reduced motion en foutstatus; benoemde gebruikersproeven op finale bron. | Native UI/accessibility + ontwerp | Deskundige/jurist bepaalt vereiste normversie, relevante hoofdstukken, nationale toepassing en beoordeling. Web-WCAG of screenshots vervangen native bewijs niet. |
| EU07-09 **ONBEKEND** | RED; P-OVERIG, S10 fysieke profielgrens. **Afhankelijk van productvorm**: losse onderzoekssoftware versus apparaat/radiofunctie; geen benoemd radioapparaat of fabrikantrol. | E0 voor RED-beoordeling; virtuele netwerkonderdelen zijn geen radio-/RF-test. | Indien radioapparaatproduct: exacte hardware/firmware/radioconfiguratie, rol en wijzigingsimpact, toepasselijke technische eisen en test-/dossierroute laten vaststellen. | Hardware/product + bevoegde externe beoordelaar | Jurist/hardwareconformiteitsdeskundige beslist toepasselijkheid; geen RED-plicht louter uit wifi-ambitie, geen vrijstelling uit QEMU afleiden. Geen hardwaretests toegestaan in deze audit. |
| EU07-10 **BLOKKEREND** voor een CE-/marktclaim | CE, dossier/markttoezicht; P-CRA/P-OVERIG, S1:13, S10:225-233. **Afhankelijk van productvorm** en toepasselijke wetgeving; CE is geen algemene OS-veiligheidsstempel. | E1 technische contracten en opgeslagen deelbewijzen; geen gecontroleerd volledig conformiteitsdossier. | Product-ID/versie/configuratie, toepasselijkheidsregister, traceerbare risicoanalyse, eisen/testmapping, componentdossier, beoordelingsroute, verantwoordelijke marktactoren en documentbeheer; eventuele verklaring/markering pas na bevoegde beoordeling. | Release/product + juridische eigenaar | Bepaal of/waarom markering en verklaring nodig zijn, welke regelgeving geldt en wie bevoegd tekent. Dit rapport is geen dossiergoedkeuring. |
| EU07-11 **BLOKKEREND** voor supportbelofte | Consumentenrecht/support/veiligheidsupdates; P-OVERIG, S1:33, S3:4-8. **Afhankelijk van productvorm**: aanbod, contract, betalingen, eventuele gegevens-tegenprestatie en lidstaat onbekend. | E1 experimentele waarschuwing; expliciet geen ondersteunde releases of duur. Historische B03-functie bewijst geen onderhoudsorganisatie. | Transparant aanbod met werkelijke functie-/hardwaregrenzen, ondersteunde versies, update-/supportbeleid, beëindigingscommunicatie en herstelroute; financiering/owners voor patchdelivery; klachten/foutenproces. | Product/support + releasebeheer | Jurist bepaalt contractuele wettelijke aanspraken en vereiste informatie/updates. Niet aannemen dat gratis of experimenteel automatisch alle consumentenregels uitsluit. Geen zelfbedachte supportduur. |
| EU07-12 **BLOKKEREND** | SBOM/licenties/herkomst; eigen EU-06, S11, FR-R04/R03; P-CRA/P-LIC. **Van toepassing** als eigen releasecriterium; wettelijke vorm/reikwijdte **juridisch te beoordelen**. | E1 concrete r-efi-notice/checksum, lockfiles en dependencyregistraties; geen compleet release-SBOM aangetoond. Git-bestandszoekactie vond `LICENSE-PLAN.md`, geen top-level `LICENSE`/`COPYING` of SBOM-bestandsnaam. | Machineleesbare stuklijst per exact geleverd artifact met directe/transitieve componenten, versies, hashes/licenties en scope; ook loader, firmware waar meegeleverd, fonts, assets, modellen en toolchainprovenance onderscheiden. Bevestigde projectlicentie/notices en reproduceerbare component/artifactbinding. | Release/dependencybeheer + rechthebbenden | Licentiekeuze en distributierechten door bevoegde mensen bevestigen; lockfile is geen complete SBOM, SBOM geen kwetsbaarheidsvrijverklaring. Beoordeel wettelijk vereiste inhoud en ontvangers afzonderlijk van eigen eis. |
| EU07-13 **MIDDEL** | Consistentie EU-acceptatiebasis; S1:5,26 versus S2:1-4,85-109. **Van toepassing** als eigen projectdocumentatie. | E1 aantoonbare drift: EU-tekst sluit externe cloud generiek uit en verwijst naar privacyrevisie 3; actuele revisie 4 staat expliciet gekozen ciphertextprovider toe. | EU-01 en introductie laten aansluiten op revisie 4/LP-26-31 zonder key-/netwerkrechten te verruimen; dataflowregister/acceptatiemapping dezelfde grens geven. Niet in deze audit aangepast. | Documentintegrator + privacy/product | Bevestig dat een nieuwe formulering het leidende productbesluit volgt; behoud onderscheid providercredentials/informatie en inhoudssleutels. Geen nieuwe juridische conclusie nodig om documentdrift te erkennen. |

## 5. Afhankelijkheden en aanbevolen aftekenvolgorde

1. **EU07-01 -> EU07-03/06/07/09/10/11:** productvorm, verantwoordelijke
   rechtspersoon, landen, distributie en diensten vaststellen. Daarna
   primaire juridische bronnen en deadlinebronnen door bevoegde mensen laten
   valideren. Geen nieuwe externe raadpleging uitgevoerd in deze opdracht.
2. **EU07-13 -> EU07-04/05/07:** actieve privacyteksten op revisie 4 uitlijnen
   en echte dataflows/actoren inventariseren. Juridische toestemming,
   productkeuze en technisch capabilityrecht afzonderlijk modelleren.
3. **EU07-03 + EU07-11:** verantwoordelijke mensen, kwetsbaarheidsintake,
   disclosure en supportketen concretiseren voordat het eigen beleid voor
   een openbare testrelease kan worden afgetekend. Juridisch vereiste melding
   is conditioneel, geen reden OS-telemetrie toe te voegen.
4. **EU07-02 + EU07-12 -> EU07-10:** finale artifact-/componentbinding,
   daadwerkelijke distributierechten en update-trustketen samenbrengen.
   Historische TEST-key-proeven niet weggooien, maar niet als
   productie-releaseautoriteit presenteren.
5. **EU07-08 + EU07-04/05 + bovenstaande -> FR-B04/B05/R01-R04:** geïntegreerde
   native positieve/negatieve acceptatie, waar vereist herhaalde gates,
   toegestane hardwareproeven, menselijke taakproeven en onafhankelijke
   deskundigencontrole op de exacte finale bron. De bestaande releasegates
   blijven leidend; deze analyse voegt geen concurrerende roadmap toe.

De onmiddellijke auditblocker voor **juridische** conclusies is dat actuele
primaire teksten en de productrol onder de bronbeperking niet kunnen worden
geverifieerd. Dat verhindert dit technische rapport niet. Technische
releaseblockers zijn de expliciete open eigen gates hierboven, niet een door
deze AI vastgestelde wettelijke overtreding.

## 6. Eigen controles, commando's en uitkomsten

Onderstaande shellcommando's zijn werkelijk uitgevoerd in de eigen werkboom.
Geen test- of buildresultaat wordt aan deze audit toegeschreven. Commando's
met `&&` zijn sequentieel; bij exit 2 is het resterende deel niet uitgevoerd.
`view`, `rg` en `glob` zijn daarnaast als ingebouwde read-only tools gebruikt
voor de in S1-S12 genoemde bestanden, instructies, README, status en relevante
implementatiepassages; die toolreads hebben geen shell-exitstatus.

### C1 - begincontrole: exit 0

```sh
git branch --show-current && git rev-parse HEAD && git status --short && git worktree list && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54
```

Resultaat: beginbranch/HEAD uit sectie 1, lege status en bronobject `commit`.
Geen bestanden uit andere werkbomen gelezen.

### C2 - geautoriseerde bronselectie: exit 0

```sh
git switch --detach beafdedfc188154fe7285848fbd411e651013f54 && git rev-parse HEAD && git status --short && git ls-files '*AGENTS.md' '.github/agents/os-eu-compliance-reviewer.agent.md' && git log -1 --format='%H %s'
```

Resultaat: correcte detached auditbron, schone status, instructielijst.

### C3 - regels en gateconfiguratie: exit 0

```sh
nl -ba docs/implementation/EUROPE-AND-RELEASE.md && nl -ba SECURITY.md && nl -ba docs/audit/os-completeness/accessibility-gaps.md && nl -ba .github/workflows/signed-updates.yml && nl -ba docs/third-party/README.md
```

### C4 - register-/callerszoekactie: exit 2, niet geslaagd

```sh
git ls-files 'LICENSE*' 'COPYING*' '*sbom*' '*SBOM*' '*cyclonedx*' '*spdx*' '*dataflow*' '*datastrom*' && git ls-files 'services/*update*' 'crates/update-core/*' '*key-policy*' && rg -n 'TrustPolicy|verify_manifest|test_policy|revoked|first_version|last_version' crates/update-core/src boot/loader/src services/update* && rg -n 'NIS2|ePrivacy|Radio Equipment|RED/CE|consumentenrecht|2016/679|2022/2555|2024/2847|2002/58' docs/implementation docs/PRODUCT-AND-SOVEREIGNTY.md SECURITY.md .github/agents/os-eu-compliance-reviewer.agent.md
```

De Gitlijsten en gedeeltelijke codezoekactie leverden inhoud, maar
`services/update*` bestaat niet; `rg` gaf exit 2 en de laatste zoekactie werd
overgeslagen. Geen runtime- of codefout. Hersteld met C5 en gerichte read van
de echte dienst `sdk/examples/update/src/service.rs:1-70`.

### C5 - herstel en rapportinstructies: exit 0

```sh
rg -n 'NIS2|ePrivacy|Radio Equipment|RED/CE|consumentenrecht|2016/679|2022/2555|2024/2847|2002/58' docs/implementation docs/PRODUCT-AND-SOVEREIGNTY.md SECURITY.md .github/agents/os-eu-compliance-reviewer.agent.md; git ls-files 'docs/audits/*' '*AGENTS.md' | grep -E 'docs/audits|AGENTS.md'
```

Resultaat: onderwerpen in rolprofiel; geen overeenkomstig uitgewerkt actief
bronregister in deze zoekscope. Geen bestaande tracked rapporten of geneste
auditinstructies in de getoonde lijst.

### C6 - caller-/policy-/privacyregels: exit 0

```sh
rg -n 'test_policy::verify_manifest|test_policy|fn verify' sdk --glob '*.rs' && nl -ba crates/update-core/src/policy.rs && nl -ba crates/update-core/src/test_policy.rs && nl -ba LICENSE-PLAN.md && nl -ba docs/LOCAL-PRIVACY-CONTRACT.md | sed -n '1,22p;54,114p;154,184p'
```

### C7 - historische verwijzingen: exit 0

```sh
rg -n 'source|commit|passed|rotate-install|rotate-boot|key-revoked|key-expired|key-not-yet-valid' docs/verification/B03-KEY-POLICY-1.md docs/verification/b03-key-policy-mainca45/update-gate.json | head -65; rg -n 'FR-B0|FR-P|FR-R0|FR-N0|FR-O0' docs/FULL-TEST-RELEASE.md | tail -32; rg -n 'schermlezer|accessibility|app-lokaal|native_execution|screenreader' docs/implementation/UX-MATERIAAL-SAVE-RECOVERY.md | head -20
```

C6/C7-tooloutput werd wegens omvang als preview weergegeven; daarom zijn
beleid, echte dienst en historisch rapport daarnaast met gerichte `view`-reads
gelezen en zijn relevante verwijzingen in C8 begrensd opgehaald. Geen
volledige bewijshashcontrole uit zoekmatches afgeleid.

### C8 - bronbinding en begrensde regels: exit 0

```sh
rg -n 'FR-B0|FR-P|FR-R0' docs/FULL-TEST-RELEASE.md | cut -c1-370; rg -n 'accessibility|app-lokaal|native_execution|screenreader' docs/implementation/UX-MATERIAAL-SAVE-RECOVERY.md | head -12 | cut -c1-300; git status --short; git rev-parse --verify refs/heads/copilot/add-os-specialist-agents; git symbolic-ref -q --short HEAD; test -z "$(git status --porcelain)"
```

Resultaat: status leeg, bronbranch exact auditcommit, werkboom schoon vlak
vóór rapportaanmaak. `git symbolic-ref -q --short HEAD` gaf afzonderlijk 1,
zoals verwacht bij detached HEAD; het afsluitende schoonheidscommando gaf 0.
De gecombineerde exit 0 is geen claim dat iedere deelopdracht exit 0 had.

### C9 - eerste rapportcontrole: exit 1

```sh
git diff --no-index --check /dev/null docs/audits/subagents/07-eu-compliance.md && git diff --exit-code && git status --short --untracked-files=all && git rev-parse HEAD && wc -l docs/audits/subagents/07-eu-compliance.md && sha256sum docs/audits/subagents/07-eu-compliance.md
```

Geen whitespacefout afgedrukt; de no-index-vergelijking gaf 1 wegens het nieuwe
bestand. Door `&&` zijn de overige controles in deze poging niet uitgevoerd.
Daarom afzonderlijk afgehandeld in C11.

### C10 - controle regelankers: exit 0

```sh
nl -ba docs/implementation/B03-SIGNED-UPDATES.md | sed -n '163,179p;198,217p'; nl -ba docs/verification/B03-ACCEPTANCE.md | sed -n '99,126p'; nl -ba docs/implementation/UX-MATERIAAL-SAVE-RECOVERY.md | sed -n '187,195p;269,277p'
```

Resultaat: bewijsgrenzen bevestigd; S8-regelbereik aangescherpt.

### C11 - afzonderlijke rapport-/scopecontrole: exit 0

```sh
git diff --no-index --check /dev/null docs/audits/subagents/07-eu-compliance.md; printf 'no-index-check-exit=%s\n' "$?"; git diff --check && git diff --exit-code && git status --short --untracked-files=all && git rev-parse HEAD && wc -l docs/audits/subagents/07-eu-compliance.md && sha256sum docs/audits/subagents/07-eu-compliance.md
```

Resultaat: no-index afzonderlijk 1 zonder whitespacebevinding, tracked diff
leeg en exit 0; uitsluitend het toegewezen rapport als untracked bestand,
HEAD exact auditcommit. Hash/lengte uit deze tussentijdse controle zijn geen
hash/lengte van het daarna aangevulde rapport.

### C12 - finale scope-/bestandscontrole

```sh
git diff --check && git diff --exit-code && git status --short --untracked-files=all && git rev-parse HEAD && wc -l docs/audits/subagents/07-eu-compliance.md && sha256sum docs/audits/subagents/07-eu-compliance.md
```

Exitstatus en finale hash worden bij overdracht vermeld; geen testcommando.

### Log-/bewijsplaats en niet uitgevoerde controles

De eigen controle-uitvoer staat in het sessietranscript; dit rapport bewaart
de commando's en relevante uitkomsten. Er is geen nieuw testlog/evidencebundle
in de repository geschreven. Bestaande historische update-logs staan onder
`docs/verification/b03/` en `docs/verification/b03-key-policy-mainca45/`;
zij zijn niet opnieuw uitgevoerd of volledig op hashes gecontroleerd.

**Niet uitgevoerd:** externe primaire EU-broncontrole, juridische beoordeling,
live PR/CI-controle, dependency/advisoryscan, SBOM-generatie, licentieaudit,
builds/hosttests/QEMU-proeven, pakket- of update-installatie, verkeerscapture,
native export-/verwijderproeven, privaat meldkanaaltest, hardware/RF-/CE-test,
screenreadertest, menselijke gebruikerstest, onafhankelijke menselijke
securityaudit en releasevalidatie. Afwezigheid van een gevonden bestandsnaam
bewijst geen repositorybrede afwezigheid van iedere functie.

## 7. Aannames, restrisico's en overdracht

- Aangenomen productrichting: eigen kernel, consumentenclient en NAS-rol,
  optioneel expliciet gekozen ciphertextprovider volgens het leidende
  privacycontract. Dat bepaalt niet de juridische marktrol of exploitant.
- Marktlanden, rechtspersoon, verdienmodel, distributiecontracten, werkelijk
  aangeboden diensten en supportverplichtingen zijn hier niet vastgesteld.
  Geen FOSS-, huishoudelijke of onderzoeksuitzondering als feit aangenomen.
- E2EE en lokale AI beperken blootstelling, maar laten metadata,
  geautoriseerde export, eindpuntcompromis en zelfstandige supportverwerking
  bestaan. Geen absolute privacy- of veiligheidsclaim.
- Oude documenten en eerdere auditbevindingen kunnen achterlopen op losse
  implementatiestappen. Daarom zijn latere key-policycode en brongebonden
  herhalingen expliciet behouden; de volledige finale release blijft open.
- Geen productscore opnieuw vastgesteld of gewijzigd. README vermeldt 41/100;
  deze documentanalyse levert geen nieuwe geaccepteerde productpunten.
- Integrator verzamelt de zestien rapporten en beslist daarna over de
  gezamenlijke vastlegging. Dit rapport is on-gecommit in de detached eigen
  werkboom beschikbaar; geen PR/merge/releasehandeling uitgevoerd.

**Kernoverdracht:** eerst productrol/actuele bronbasis laten beoordelen;
operationele vulnerability response, support en component-/licentiedossier
concretiseren; native toegankelijkheid en geïntegreerde privacy/updateketen
met passend bewijs aftekenen. EU-documentdrift apart herstellen zonder
stilzwijgende wijziging van het privacycontract. Geen juridische
conformiteit, algemene veiligheid, productrijpheid of fysieke
hardwareondersteuning uit deze audit afleiden.
