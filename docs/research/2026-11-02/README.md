# OS-gebruikersbehoeften, overstap en ontwerp — fase 2–4

Onderzoeksdatum: **2 november 2026**, Europe/Amsterdam. Status: vastgelegde deskresearch; geen eigen interviews, geen productcode, geen geaccepteerde nieuwe OS-functies. Opdracht: de eerdere repositoryanalyse aanvullen met actueel extern onderzoek, de oorspronkelijke Eurobarometerclaim controleren en dit onderzoek in de repository bewaren.

## Kernconclusie

[ANALYSE] Voor nieuw-os is de onderbouwde kans een samenhangende dagelijkse workflow met controle, herstel en een aantoonbaar haalbare overstap. Een nieuwe stijl, Europese herkomst of AI-functie op zichzelf is onvoldoende onderbouwd als overstapreden. De bronnen laten zowel verbeteringen bij gevestigde platformen als blijvende afhankelijkheden zien; “niemand lost dit op” is geen verdedigbaar uitgangspunt.

**Belangrijkste cijfercontrole:** 58% instemming met overstappen naar een EU-gevestigde digitale dienstverlener, zelfs bij iets hogere kosten, is bevestigd in Special Eurobarometer 572/QB6.4. Dit meet geen OS-migratie. De oorspronkelijke vraag, antwoordverdeling, methode en beperkingen staan in [het adoptierapport](european-adoption-barriers.md) en [de broncontrole](eurobarometer-check.json).

## Onderzoeksvragen

1. Welke dagelijkse fricties en gewenste functies zijn daadwerkelijk beschreven bij Windows, macOS, Linux-desktops en Android?
2. Welke signalen komen in onafhankelijke bronnen terug, en welke zijn slechts een losse melding of leveranciersclaim?
3. Welke functionele, sociale, organisatorische en economische barrières verhinderen overstappen?
4. Wat meet de Eurobarometer precies over Europese diensten en betalen, en wat blijft onbekend over OS-keuze?
5. Welke OS-ontwerppatronen ontwikkelden zich tussen circa 2011 en 2026, en welk bewijs bestaat voor hun praktische waarde?
6. Welke bevindingen kunnen op de bestaande onafhankelijke nieuw-os-architectuur aansluiten, zonder een productbesluit of implementatie te veronderstellen?

## Dossiers en leesvolgorde

**Uitbreiding fase 4:** [volledige installatie- en day-one-journey](journey/README.md), met 31 installatieaspecten, 28 dagelijkse taken, mentale modellen en afzonderlijk gelabelde installer-/onboarding-/interactievoorstellen. Geen implementatie. De vergelijking is eerst vastgelegd; daarna is het bestaande productplan begrensd aangevuld.

| Document | Inhoud |
|---|---|
| [Repositorybasis](repository-baseline.md) | Fase 1 vastgelegd: main, nieuwere ontwikkelbranch en prototype/native-bewijsgrenzen |
| [Gebruikersbehoeften en synthese](os-user-needs.md) | Vergelijking, herhaling, impact, Europese relevantie en technische ruimte |
| [Windows en macOS](windows-macos-needs.md) | Aspecten per OS, oorspronkelijke gebruikerssignalen, actuele oplossingen en beperkingen |
| [Linux en mobiel](linux-mobile-needs.md) | GNOME/KDE/Ubuntu, Android en relevante iOS/ChromeOS-vergelijkingen |
| [Europese adoptie en barrières](european-adoption-barriers.md) | Eurobarometer, intentie versus gedrag, compatibiliteit, migratie en Europese alternatieven |
| [OS-ontwerptrends](future-os-design.md) | Tijdlijn, bruikbare patronen, decoratieve risico's en expliciete toekomsthypothesen |
| [Bronnenregister](sources.md) | Bronfamilies, datums, primaire bronnen en controlebeperkingen |
| [Eurobarometercontrole](eurobarometer-check.json) | Originele PDF-hashes, paginanummers, cijfers en rekencontrole |

Dit is een aanvulling op de [verkenningen van 28 oktober](../README.md), geen vervangende productroadmap. Materiaal, privacycontract, volledige releasecriteria en vaste acceptatiegewichten blijven gelden. Fase 5–8 moet eventuele gewijzigde productkeuzes, prioriteiten en implementaties afzonderlijk motiveren. Er is hier geen nieuwe desktoprichting aangenomen.

## Zoek- en selectiestrategie

Doelgerichte triangulatie met drie parallelle onderzoeksdossiers en centrale bron-/claimcontrole. Actualiteit: huidige platformdocumentatie en recente bronnen, doorgaans 2024–2026. Oudere originele studies zijn alleen gebruikt waar zij een mechanisme of ontwerphistorie onderbouwen; hun uitkomsten worden niet naar 2026 geëxtrapoleerd. De tijdlijn omvat expliciet oudere releases.

Gezocht via webzoekmachine, vervolgens de oorspronkelijke pagina, uitgave, issue of rapport geopend. Zoekclusters omvatten:

- `Windows 2026 search taskbar updates user feedback`, Microsoft OneDrive bestands-/back-upgedrag, PowerToys-vindbaarheid en NV Access-regressies;
- macOS window tiling, Spotlight, iPhone Mirroring EU, AppleVis VoiceOver en onderzoek naar OS-interfacegewoonte;
- GNOME/KDE toegankelijkheid, window management, pakketportalen, oorspronkelijke issues, Ubuntu-hardwarecertificatie;
- Android privacykeuzes, notificaties, background-processen, adaptive interfaces, update-/back-up- en integriteitsgrenzen;
- `Eurobarometer 2026 58 EU digital service provider`, oorspronkelijke SP572-vragenlijst, QB6.4, rapport en data-annex;
- CMA mobile ecosystems switching, originele vzbv-Windows-10-tabellen, Office-VBA, Adobe-vereisten, Valve-Proton en Europese migratiecases;
- oorspronkelijke introducties van flat/Material/designmaterialen, dynamische kleur, adaptive layouts, semantiek, AI-richtlijnen en studies naar aandacht, contrast en energie.

Voorkeur voor oorspronkelijke studies en uitgevers, officiële specificaties/support/releasebesluiten, regulatoronderzoek, gevolgd door oorspronkelijke communitymeldingen. Communitybronnen helpen concrete mislukte handelingen en verwachtingen vinden; ze dragen geen algemene marktclaim. Geen automatische telling van zoekresultaten, stemmen of duplicerende nieuwsartikelen. De selectie omvat bewuste tegenvoorbeelden: bestaande oplossingen, verbeterde uitrol en tevredenheid als reden om te blijven.

Opname vereist een benoembare bevinding, directe URL, datum of expliciet onbekende datum, bronsoort en beperking. Afgewezen als algemene onderbouwing: onherleidbare percentages, marketing zonder noemer, een rant als volledige doelgroepstem, onbereikbare fulltext alsof die gelezen was, en een featureaankondiging als bewezen gebruikerseffect. Beta/Insider/roadmap blijft apart van algemene beschikbaarheid. In elk dossier staan de werkelijk gebruikte bronnen; geen volledige zoekgeschiedenis of PRISMA-claim.

## Labels en bewijssterkte

- **[BRON]**: wat een gelezen bron beschrijft, meet of officieel ondersteunt; de reikwijdte blijft erbij.
- **[SIGNAAL]**: een concrete oorspronkelijke melding of communitywens; geen representativiteit.
- **[ANALYSE]**: onze synthese, vergelijking of technische gevolgtrekking.
- **[HYPOTHESE]**: een te toetsen behoefte, ontwerpvertaling of toekomstverwachting.
- **[REPOSITORY]**: aangetroffen broncode of versiegebonden projectbewijs; geen externe marktbevinding.

Voor de synthese betekent **sterk**: rechtstreeks empirisch bewijs of een expliciet controleerbare technische grens voor de afgebakende claim. **Middel**: meerdere bruikbare primaire aanwijzingen, beperkte doelgroep of overdracht naar nieuw-os. **Zwak/open**: losse signalen, alleen productvoorzieningen of ontbrekende meting. Sterk technisch bewijs kan samengaan met onbekende gebruiksfrequentie.

“Herhaling” betekent uitsluitend terugkeer in deze doelgerichte bronset. Meerdere Microsoftpagina's tellen bijvoorbeeld als één leveranciersfamilie; persbericht en tabel van dezelfde enquête als één onderzoek. We publiceren geen rangorde van klachtfrequentie onder alle Europese gebruikers. Impact, haalbaarheid en onderscheid zijn redactionele inschattingen; geen onderzoekspercentages of fase-6-score.

## Doelgroepen en ontbrekende eigen data

Onderzoekssegmenten: mensen met dagelijkse browser-/documenttaken; gevorderde gebruikers met automatisering of maatwerk; gamers; creatieve/professionele gebruikers; mensen die hulptechnologie nodig hebben; beheerders/organisaties. Dit zijn analytische groepen, geen geïnterviewde persona's. Leeftijd, land, technische kennis en beperkingen zijn niet betrouwbaar uit platformkeuze af te leiden.

Er zijn geen fictieve namen, citaten, interviews, feedbackscores, mockdata of marktaandeelprognoses toegevoegd. Toekomstig onderzoek moet daadwerkelijke taken en noodzakelijke software/apparaten verzamelen, met deelnemers uit meer dan één vaardigheids- en toegankelijkheidscontext. Aantrekkelijkheid, taakprestatie, proberen, werkelijk overstappen en blijven gebruiken afzonderlijk meten.

## Grenzen en veilige vervolgroute

De bronset is niet uitputtend, niet preregistreerd en niet representatief voor alle Europeanen. Engelstalige/Nederlandse/Duitse bronnen en technisch actieve communities zijn oververtegenwoordigd. Leveranciers kennen hun product, maar hebben commerciële belangen; gebruikersissues oververtegenwoordigen problemen. Veel platformfeatures verkleinen inmiddels eerdere gaten. Gebrek aan een gevonden bron bewijst geen gebrek aan een behoefte.

De onderzoeken sluiten geen native, fysieke, menselijke of releasegate. De documentatiebranch op main behoudt 34/100; de apart gecontroleerde nieuwere ontwikkelbranch rapporteert 41/100. Fase 5–6 kan nu probleemgerichte concepten en expliciete experimenten vergelijken. Productcode volgt pas na vastgelegd ontwerp-/implementatieplan binnen de gebruikersopdracht.
