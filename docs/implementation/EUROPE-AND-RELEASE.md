# Europese eisen en publicatie

Onderzocht: **26 oktober 2026**. Status: onderbouwde ontwerp- en releasecriteria; geen implementatie, certificaat of juridische conformiteitsverklaring. Deze afspraken laten de eerste QEMU-boot doorgaan. Beoordeel wettelijke toepasselijkheid opnieuw zodra doelgroep, distributie, rechtspersoon en verdienmodel concreet zijn.

Er bestaat geen enkele overkoepelende “Europese OS-norm” die privacy, cybersecurity, toegankelijkheid en interoperabiliteit gezamenlijk aftekent. Behandel deze als afzonderlijke eisen met eigen bewijs. De lokale privacygrens is bovendien strenger dan een algemene eis om persoonsgegevens rechtmatig te verwerken: ook een Europese externe cloud is voor ingebouwde OS-diensten uitgesloten.

## Toepasselijkheid: welke situatie verandert de verplichtingen?

| Onderwerp | Voorwaardelijke toepassing | Gevolg voor dit project |
|---|---|---|
| AVG/GDPR | Rollen volgen de daadwerkelijke verwerking: wie doel en essentiële middelen bepaalt is verwerkingsverantwoordelijke; wie namens die partij verwerkt kan verwerker zijn. Alleen software leveren maakt iemand niet automatisch verwerker. Geen toegang tot data hebben sluit verantwoordelijkheid niet automatisch uit. [1–2] | Registreer per gegevensstroom wie beslist en verwerkt. Het OS-ontwerp, eventuele supportgegevens en repositoryaccounts zijn verschillende activiteiten. “Alles lokaal” is geen algemene AVG-vrijstelling. |
| Persoonlijk gebruik | De huishoudelijke uitzondering betreft uitsluitend persoonlijke/huishoudelijke verwerking zonder beroeps- of commerciële verbinding. [1] | Trek die uitzondering niet door naar de OS-aanbieder, een werkgever, school of zakelijke NAS-gebruiker. |
| Cyber Resilience Act: fabrikant | FOSS die buiten commerciële activiteit wordt ontwikkeld/aangeboden valt buiten de gewone productverplichtingen. Open broncode of een gratis download alleen bewijst die uitzondering niet: het feitelijke distributie- en verdienmodel telt. [3] | Leg vóór een distributierelease vast: wie brengt wat op de EU-markt, onder welke naam, met welke commerciële relatie? OS-producten kunnen onder de belangrijke-productenregeling vallen; bepaal dan de concrete classificatie en conformiteitsroute. |
| CRA: open-source-steward | Een rechtspersoon die duurzaam en systematisch specifieke FOSS ondersteunt, voor commerciële activiteiten bedoeld, en de levensvatbaarheid waarborgt, kan steward zijn. Hiervoor geldt een afzonderlijk regime, waaronder beveiligingsbeleid en samenwerking bij kwetsbaarheden. [3] | Een toekomstige stichting is niet automatisch vrijgesteld en ook niet automatisch fabrikant. Documenteer haar feitelijke rol. |
| Toegankelijkheid | De European Accessibility Act omvat onder meer algemene consumentencomputers en hun besturingssystemen die na **28 juni 2025** op de markt komen. Nationale omzetting, product, rol en eventuele uitzonderingen bepalen de concrete plichten. [6–7] | Neem toegankelijkheid vanaf het eerste interactieprototype mee. Een intern bootexperiment is geen bewijs van een toegankelijk consumentenproduct. |
| Normen en aanbestedingen | EN 301 549 omvat ICT-eisen, waaronder software en samenwerking met hulptechnologie. WCAG alleen dekt geen volledig native OS. Harmonisatie geldt voor een specifieke versie en rechtsgrond; de Web Accessibility Directive betreft publieke websites/apps. [8–9] | Gebruik EN 301 549 V3.2.1 hoofdstuk 11 als gedocumenteerde ontwerpbaseline; WCAG 2.2 AA als aanvullend doel voor webprototypes. Controleer bij release de dan aangewezen normversies en de concrete aanbesteding. Open formaten zijn een productkeuze, geen algemeen certificaat. |

**CRA-datums:** in werking sinds 10 december 2024; de belangrijkste productverplichtingen gelden vanaf **11 december 2027**. Voor fabrikanten geldt melden van actief uitgebuite kwetsbaarheden en ernstige beveiligingsincidenten al sinds **11 september 2026**. Voor stewards begint hun meldplicht op **11 december 2027**. [4–5] Een toepasselijke fabrikant moet onder meer op 24-uurs- en 72-uursmeldingen zijn ingericht; dit is geen automatische OS-telemetriedienst.

## Technische acceptatiecriteria

Dit zijn **eigen projecteisen**, geen opsomming van letterlijk voorgeschreven wettelijke implementaties. Bouw ze zodra het betreffende subsysteem ontstaat; claim ze pas na de test.

| ID | Te bouwen en aantoonbaar te controleren |
|---|---|
| EU-01 | Datastromenregister met dataset, doel, locatie, bewaartermijn, bevoegdheid en verwijderpad. Apparaat of end-to-end versleutelde opslag op expliciet gekoppelde eigen NAS of NAS van vrienden/familie; internet uitsluitend na peergebonden toestemming. Beheerder mag opslag intrekken/verwijderen maar krijgt geen inhoudssleutel. Geen leverancieraccount, stille relay/fallback, sleutelbewaring of externe telemetrie. Voer alle toepasselijke LP-tests uit LOCAL-PRIVACY-CONTRACT.md revisie 3 uit zodra de functies bestaan; dit is geen wettelijke conformiteitsclaim. |
| EU-02 | AI-inferentie zonder netwerkbevoegdheid; modeldownloader zonder persoonlijke context. Modelgebrek, geheugenfout en wegvallend LAN activeren nooit cloudfallback. Test ook helperprocessen en lokale brokers. |
| EU-03 | Appinstallatie verleent geen algemene bevoegdheden. Test afzonderlijk bestanden, NAS-datasets, LAN, internet, camera en microfoon; test intrekking en omwegen via tussendiensten. Maak gecombineerde lees- en internetrechten zichtbaar. |
| EU-04 | Exporteer documenten, instellingen en geselecteerde AI-geschiedenis in gedocumenteerde formaten zonder account. Verwijder tevens indexen, caches en embeddings. Toon afzonderlijk wat in NAS-herstelpunten blijft bestaan; beweer geen gewiste externe kopieën terug te halen. |
| EU-05 | Toets onboarding, aanmelding, rechten, installatie en herstel volledig met toetsenbord; ontwerp zichtbare focus, schaalbare tekst, contrast, verminderde beweging en een semantische toegankelijkheidsboom voor screenreaders. Bewaar scenario's en resultaten per build. |
| EU-06 | Genereer per release een machineleesbare SBOM met directe/transitieve componenten, versies, hashes en licenties; pin buildgereedschap. Documenteer reproduceerbaarheid, bronherkomst en uitzonderingen. Controleer ook firmware, fonts en modellen. |
| EU-07 | Ondertekende updateketen, sleutelrotatie/intrekking, bescherming tegen ongeautoriseerde downgrade en herstel na onderbroken updates. Test foute handtekening, oud pakket, beschadiging en stroomuitval. Onderhoud een offline updatepad. |
| EU-08 | Publiceer bij echte distributie ondersteunde versies, ondersteuningsduur, beveiligingscontact en disclosureproces. Leg triage, correctie, advisories en conditionele wettelijke melding bij verantwoordelijke mensen neer. SBOM en issue-tracker zijn geen bewijs dat alle kwetsbaarheden bekend zijn. |

Gebruik synthetische gegevens in CI en Codex. Releasebewijs bevat versie, configuratie, uitgevoerde tests en resterende beperkingen. Deze matrix is niet uitputtend voor latere sectoren zoals zorg, voertuigen of een specifieke AI-toepassing.

## Voorgestelde licentie

**MPL-2.0** is de aanbevolen standaard voor eigen OS-code: copyleft op bestandsniveau houdt verspreide wijzigingen aan gedekte bestanden beschikbaar, terwijl afzonderlijke componenten andere compatibele licenties kunnen hebben. Bij distributie van binaries moet bijbehorende gedekte broncode beschikbaar zijn; bestaande notices blijven behouden. [10] Dit document past de licentie nog niet toe. Vóór publiceren: controleer rechthebbenden en afhankelijkheden, voeg de officiële tekst en passende notices toe en leg asset-/modellicenties afzonderlijk vast. MPL maakt derdenapps niet vanzelf open source en regelt geen privacygarantie.

## Primaire bronnen

Geraadpleegd op 26 oktober 2026; links verwijzen naar officiële teksten/uitleg. De projectcriteria hierboven zijn eigen afleidingen.

1. Europese Commissie. (z.d.). *Data protection explained*. https://commission.europa.eu/law/law-topic/data-protection/data-protection-explained_en
2. European Data Protection Board. (2022). *Guidelines 07/2020 on the concepts of controller and processor in the GDPR* (versie 2.1). https://www.edpb.europa.eu/system/files/2023-10/EDPB_guidelines_202007_controllerprocessor_final_en.pdf
3. Europese Commissie. (2026, 15 oktober). *Cyber Resilience Act — Open source*. https://digital-strategy.ec.europa.eu/en/policies/cra-open-source
4. Europese Commissie. (2026, 7 september). *Cyber Resilience Act*. https://digital-strategy.ec.europa.eu/en/policies/cyber-resilience-act
5. Europese Commissie. (2026, 11 september). *Cyber Resilience Act — Reporting obligations*. https://digital-strategy.ec.europa.eu/en/policies/cra-reporting
6. Europese Commissie. (z.d.). *European Accessibility Act (EAA)*. https://commission.europa.eu/strategy-and-policy/policies/justice-and-fundamental-rights/disability/european-accessibility-act-eaa_en
7. Europees Parlement & Raad. (2019). *Richtlijn (EU) 2019/882*, artikel 2. https://eur-lex.europa.eu/eli/dir/2019/882/oj
8. ETSI. (2021). *EN 301 549 V3.2.1: Accessibility requirements for ICT products and services*. https://www.etsi.org/deliver/etsi_en/301500_301599/301549/03.02.01_60/en_301549v030201p.pdf
9. Europese Commissie. (2025, 5 mei). *Web Accessibility Directive — Standards and harmonisation*. https://digital-strategy.ec.europa.eu/en/policies/web-accessibility-directive-standards-and-harmonisation
10. Mozilla. (z.d.). *Mozilla Public License, version 2.0*, §§ 1.10, 3.1–3.4. https://www.mozilla.org/en-US/MPL/2.0/

