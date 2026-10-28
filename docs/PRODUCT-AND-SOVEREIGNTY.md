# Productvisie, privacy en Europese zelfstandigheid

Status: ontwerpvoorstel, 26 oktober 2026. Dit document beschrijft gewenste eigenschappen; geen daarvan is al geïmplementeerd of gecertificeerd. Architectuur en technische ontwikkelfasen staan in `ARCHITECTURE.md` en `ROADMAP.md`.

## 1. Eén duidelijke productbelofte

Een zelfstandig, open besturingssysteem waarmee mensen hun apparaten en gegevens kunnen gebruiken met lokale accounts, een eigen NAS of vrijwillig aangeboden NAS-ruimte bij vrienden/familie voor end-to-end versleutelde synchronisatie, via LAN of expliciet toegestane internettoegang, zonder leverancierscloud voor OS-gegevens, met begrijpelijke toegangsrechten en herstelbare wijzigingen.

De technische eerste proef draait op de vastgelegde virtuele machine; productontwerp en interactieonderzoek beginnen tegelijk. Ontwerp interfaces zodat later verschillende apparaten mogelijk zijn. Een telefoon, pc en klein ingebouwd apparaat hebben verschillende bediening, energiebehoeften en beveiligingsgrenzen. Eén productfamilie hoeft daarom geen identieke interface of identiek installatiebestand te hebben.

“Veilig als macOS”, “gebruiksvriendelijk als Windows” en “instelbaar als Linux” zijn inspiratie, geen meetbare specificaties. Vergelijk uiteindelijk concrete taken op concrete versies en apparaten. Een claim dat dit OS een bestaand OS overtreft is zonder vergelijkende metingen niet onderbouwd.

## 2. Originaliteit die de gebruiker merkt

Nieuwheid hoeft geen nieuwe programmeertaal, cryptografie of incompatibel bestandsformaat te betekenen. Het onderstaande is een voorstel voor een eigen samenhangend product; niet de claim dat deze ideeën nog nooit zijn toegepast.

| Productidee | Gewenst gedrag | Eerste bewijs |
| --- | --- | --- |
| Werkruimte rondom een taak | Een gebruiker groepeert documenten, apps en relevante instellingen rond een activiteit. | Een prototype waarin deelnemers een taak hervatten zonder instructie van de maker. |
| Begrijpelijke toestemming | Een app krijgt bijvoorbeeld alleen het gekozen document, voor deze handeling; de gebruiker kan die toegang terugzien en intrekken. | Gebruiker kan aangeven wat gedeeld is; negatieve toegangstest slaagt. |
| Veilige aanpasbaarheid | Thema, shell en apps zijn vervangbaar binnen vastgelegde rechten. Kritieke wijziging heeft een herstelroute. | Onjuiste configuratie wordt teruggedraaid zonder gebruikersbestanden te verliezen. |
| Lokaal als normale werkwijze | Zoeken, instellingen en lokale documenten werken zonder account of netwerk. | Taaktest na installatie met netwerk uit. |
| Begrijpelijke foutmeldingen | De melding noemt probleem, gevolgen en één uitvoerbare herstelactie. | Gebruiker herstelt de fout zonder terminalkennis. |

Het OS krijgt een eigen visuele identiteit én een eigen interactiemodel. Originaliteit is een acceptatiecriterium vanaf M0. Een klassieke desktop met andere kleuren is onvoldoende als eindconcept. Een prototype mag nieuwe ideeën onderzoeken voordat de kernel ze uitvoert; registreer productbewijs en systeembewijs afzonderlijk.

### Drie te onderzoeken conceptrichtingen
Deze richtingen zijn hypotheses, geen reeds gekozen product of bewezen wereldprimeur. Werk ze uit als werkelijk verschillende manieren van werken, niet als drie thema's op dezelfde schermindeling.

| Concept | Primair organisatiemodel | Onderscheidende onderzoeksvraag |
|---|---|---|
| **Materiaal** | Mensen beginnen bij een document, foto, verzameling of apparaat; geïnstalleerde apps bieden passende bewerkingen aan. | Kan één object verschillende gereedschappen gebruiken met tijdelijke rechten per bewerking, zonder bestanden steeds tussen apps te verplaatsen? |
| **Werk** | Mensen openen een activiteit met doel, huidige stap, resultaten en hersteltijdlijn; apps verschijnen waar nodig. | Kan werk na onderbreking begrijpelijk worden hervat, ook op een tweede eigen apparaat via een expliciet gekoppelde persoonlijke NAS? |
| **Atelier** | Mensen richten directe bedieningsoppervlakken in: meerdere gereedschappen, materiaal en feedback tegelijk zichtbaar, passend bij taak en scherm. | Kunnen beginners meteen werken terwijl gevorderden bediening ingrijpend aanpassen zonder versnipperde instellingen of breuk in toegankelijkheid? |

Maak voor elk concept dezelfde vijf taakproeven: installeren, iets maken/bewerken, terugvinden, onderbreken/hervatten en een fout herstellen. Vergelijk met een klassieke vensterdesktop, appstartscherm en bestandsbeheer: benoem precies wat verandert, wat bewust herkenbaar blijft en welk voordeel gebruikers ervaren. Kies daarna één samenhangende richting; combineer niet automatisch alle ideeën.

### Hoogwaardige afwerking als producteis
Ontwikkel tegelijk een eigen designsysteem: typografie, spacing, kleur, iconen, materialen, focus, bewegingsregels, fouttoestanden en consistente taal. Beoordeel ook installatie, eerste gebruik, instellingen, updates, zoeken, bestandselectie en herstel; kwaliteit geldt voor de hele ervaring.

Leg meetbare interactiebudgetten vast, test vloeiende animatie en directe feedback, en bied verminderde beweging. Ontwerp toetsenbord, aanraking en hulpmiddelen als volwaardige bedieningsvormen. Het SDK en de UI-toolkit moeten externe ontwikkelaars helpen die kwaliteit te bereiken, met documentatie, voorbeelden en testgereedschappen.

Privacy krijgt een vanzelfsprekende bediening: zichtbare gegevenslocatie, rechten bij concrete handelingen, begrijpelijke NAS-koppeling en herstel zonder leveranciersaccount. AI ondersteunt lokaal en blijft optioneel. Beoordeel een ontwerp ook zonder AI, zodat het gewone product op eigen kracht overtuigt.

### Europese norm als ontwerpopdracht
We interpreteren Simons Europese norm als privacy, zeggenschap, toegankelijkheid, transparantie, open samenwerking en overdraagbaarheid. Maak daarvan zowel merkbare productkwaliteit als toetsbaar gedrag. Specifieke wettelijke of technische normen worden later per toepassingsgebied in een eisenregister gekoppeld aan bewijs; dit begrip is hier geen verwijzing naar één vastgestelde certificering.

Een onbekende technische mogelijkheid krijgt een onderzoeksvraag en een afgebakend experiment. Verlaag de productambitie niet omdat de eerste implementatie klein is; een daadwerkelijk aangetoonde grens leidt tot een expliciete ontwerpkeuze.

## 3. Privacy als observeerbaar gedrag

Dit zijn producteisen voor toekomstige implementatie:

- De basisinstallatie werkt met een lokaal account. Persoonlijke opslag bestaat uit lokale data, optionele E2EE-opslag op een eigen NAS/aangeboden NAS-ruimte en optioneel opaque client-side versleutelde objecten bij een expliciet gekozen externe opslagprovider. LAN blijft standaard voor persoonlijke NAS; internet is expliciet per peer/provider. Geen verplichte leverancierscloud, stille relay/fallback, provider-key-escrow of uitwijkserver. Een provideraccount kan nodig zijn voor de vrijwillig gekozen provider, maar geeft geen Nieuw-OS-inhoudssleutel. Zonder NAS/provider blijven lokale taken werken. De NAS-host beheert quota en mag zijn kopie zonder gasttoestemming wissen; dat geeft geen decryptiesleutel of documenttoegang.
- Geen externe OS-telemetrie of automatische externe crashuploads. Diagnostiek blijft lokaal, is begrensd en inzichtelijk. Een gebruiker mag een expliciete client-side versleutelde back-up daarvan naar een toegestane NAS of opaque provideropslag maken; de opslagbackend krijgt geen plaintext diagnose-inhoud via deze route.
- Crashexports tonen vóór verzending welke gegevens meegaan. Geen automatische geheugendumps met mogelijk documentinhoud of sleutels.
- Geen advertentie-ID, systeemreclame, verplicht platformaccount of vooraf aangevinkte gegevensdeling.
- Netwerkverkeer is verklaarbaar per dienst. Updates en tijdsynchronisatie hebben een zichtbare instelling en mogen niet stilzwijgend als “telemetrie” worden gebundeld.
- AI is optioneel en verwijderbaar, maar inferentie vindt uitsluitend op het apparaat plaats. Prompts, geschiedenis, embeddings en resultaten blijven lokaal; eventuele opslagbackup alleen end-to-end versleuteld naar een expliciet gekoppelde persoonlijke NAS. Geen cloudfallback of uitbestede inferentie. Een aparte modeldownloader heeft geen toegang tot persoonlijke context; de inferentiedienst heeft geen netwerkrecht.
- Offline gebruik mag geen gedwongen cloudregistratie activeren. Gebruikers worden wel duidelijk geïnformeerd over gemiste beveiligingsupdates.
- Export en migratie moeten mogelijk zijn met gedocumenteerde formaten. Intrekken van toegang kan eerder gemaakte kopieën bij een andere partij niet ongedaan maken.

Downloadbare derdenapps blijven mogelijk met expliciete bestands-, NAS-, apparaat- en netwerkrechten. Een app met zowel lees- als internetrecht kan toegestane gegevens naar buiten sturen; maak die combinatie zichtbaar en bied geen absolute niet-lekkenbelofte. Details en toetsen: [LOCAL-PRIVACY-CONTRACT.md](LOCAL-PRIVACY-CONTRACT.md).

Bewijs: test netwerkverkeer op een schone installatie, herhaal na updates en na het in- en uitschakelen van optionele diensten. Zolang het prototype geen netwerk heeft, bewijst “geen verkeer” nog niet dat de toekomstige netwerkversie privacyvriendelijk is.

## 4. Licht en snel: meetplan vóór marketing

Leg voor elke meting vast: commit, buildprofiel, hostprocessor, toegewezen CPU's/RAM, QEMU-versie, acceleratievorm, scherminstelling, dataset en meetprocedure. Vergelijk TCG-emulatie niet rechtstreeks met hardwareversnelling.

| Eigenschap | Voorlopig toetsdoel | Voorwaarde |
| --- | --- | --- |
| Starten | Eerst een reproduceerbare nulmeting, daarna een per fase afgesproken budget. | Meet afzonderlijk firmware, loader, kernel en bruikbare sessie. |
| Geheugen | Meet kernel, diensten, shell en cache apart; behandel een onverklaarde stijging van meer dan 20% als reviewpunt. | Dit percentage is een projectvoorstel, geen bewezen norm of absolute afkeurgrens. |
| Reactie op bediening | Eenvoudige lokale acties geven binnen 100 ms zichtbare terugkoppeling in de afgesproken referentieopstelling. | Ontwerpdoel voor de latere shell; geen claim over bestaande code of alle hardware. |
| Rusttoestand | Geen onbedoelde drukke wachtlussen; leg periodieke achtergrondtaken vast. | CPU-percentages zijn pas zinvol met een reproduceerbare meetopstelling. |
| Energie | Meet slaap, ontwaken en werkelijk energiegebruik op ondersteunde fysieke hardware. | Een kleine kernel of goede QEMU-score bewijst geen lange accuduur. |

Stel per fase een budget vast dat bij de dan bestaande functies past. Een grootheidsclaim als “sneller dan Windows” vereist dezelfde taken, vergelijkbare functionaliteit en een gepubliceerd meetprotocol.

## 5. Toegankelijkheid en Europese bruikbaarheid

Producteisen: volledige toetsenbordbediening, duidelijke focus, schaalbare tekst, voldoende contrast, minder beweging, schermlezerondersteuning en signalen die niet alleen hoorbaar zijn. Vermijd dat veiligheidsvragen uitsluitend op kleur, geluid of snelle reacties steunen.

WCAG 2.2 biedt toetsbare toegankelijkheidscriteria voor webinhoud. Gebruik die rechtstreeks voor eventuele webinterfaces en als inspiratie bij de native shell. De native omgeving heeft daarnaast een eigen toegankelijkheids-API, semantische UI-structuur en praktijktests met hulpmiddelen nodig. Dit is geen verklaring van wettelijke conformiteit (W3C, 2024).

Plan vertaling vanaf de eerste UI: externe tekstbestanden, Unicode, verschillende toetsenbordindelingen, datum- en getalformaten en tekst die langer wordt na vertaling. Onderzoek behoeften met Nederlandse en andere Europese gebruikers; er bestaat niet één uniforme “Europese gebruiker”. Kies voor de eerste proef één heldere doelgroep, bijvoorbeeld privacybewuste desktopgebruikers, en test vijf kerntaken met meerdere deelnemers. Resultaten van een kleine proef zijn richtinggevend, niet representatief voor heel Europa.

## 6. Zelfstandigheid heeft meerdere lagen

| Laag | Betekenis voor dit project |
| --- | --- |
| Tijdens gebruik | OS-gegevens lokaal, E2EE op gekoppelde persoonlijke NAS of na expliciete keuze als opaque client-side versleutelde objecten bij een externe provider; internet per peer/provider expliciet. Host/provider krijgt geen inhoudssleutel. AI-inferentie alleen op apparaat. Geen verplichte leverancierscloud, stille relay/fallback, provider-key-escrow of OS-telemetrie. Updates/modeldownloads mogen via een afgescheiden downloader; derdenapps alleen met toegekende rechten. |
| Tijdens ontwikkeling | Codex kan helpen schrijven en onderzoeken; broncode, besluiten, tests en buildinstructies blijven buiten de chat beschikbaar. |
| Tijdens bouwen | Een derde partij moet uit de broncode kunnen bouwen zonder Codex-account. Controleer alle download- en buildafhankelijkheden. |
| Publicatie | De code blijft via Git exporteerbaar; releasebestanden, issues en documentatie krijgen een overdraagbare kopie. |
| Bestuur | Publiceer beslisregels, onderhoudsrollen, financieringsafhankelijkheden en het proces voor beveiligingsmeldingen. |
| Hardware | Firmware en apparaten kunnen externe afhankelijkheden behouden. Een eigen kernel maakt hardware niet automatisch controleerbaar. |

Codeberg is een platform van een non-profitvereniging in Berlijn. Forgejo is de zelf te hosten software waarop Codeberg draait. Dat maakt ze relevante opties voor Europese zeggenschap; het is geen garantie dat elke verwerking of afhankelijkheid Europees is (Codeberg, z.d.; Forgejo, z.d.). Zelf hosten brengt eigen beheer, updates, backups en hersteltests mee. Voor een beginner verdient een beheerd platform aanvankelijk meestal de voorkeur boven een extra serverproject.

Een GitHub-mirror kan deelname vergemakkelijken. Bepaal één primaire repository en voorkom twee concurrerende issue- en beslissystemen. Controleer de actuele platformvoorwaarden en ondersteuning van de gekozen Codex-werkwijze vóór gebruik; dit document garandeert geen specifieke integratie.

## 7. Open source en bestuur

Een zichtbare repository zonder passende licentie is niet voldoende. Gebruik een bestaande, passende OSI-goedgekeurde licentie en leg de keuze vast vóór publieke publicatie van code. Controleer compatibiliteit van hergebruikte code en leg herkomst, versie, licentie en eventuele wijzigingen vast. Neem ook bijdragen van AI niet zonder herkomstcontrole over.

Het Europese doel betreft zeggenschap en bruikbaarheid, niet een verbod op niet-Europese gebruikers. Een licentie die bepaalde nationaliteiten, bedrijven of gebruiksvelden uitsluit past niet bij de Open Source Definition (Open Source Initiative, z.d.). Leg privacy- en productwaarden vast in governance, standaardgedrag en technische interfaces.

Ontwikkeling mag doorgaan terwijl een licentiebesluit wordt voorbereid; presenteer het project vóór dat besluit niet als reeds juridisch open-source gepubliceerd. Dit startpakket geeft geen verklaring dat het toekomstige product aan AVG, productwetgeving of aanbestedingseisen voldoet. Laat toepasselijkheid beoordelen wanneer doelgroep, distributievorm en dienstverlening concreet zijn.

Minimale governance vóór een openbare proef: benoemde maintainer, bijdragenprocedure, licentie- en herkomstregister, beveiligingscontact, releaseproces, ondersteunde versies en een procedure om fouten te melden zonder openbare publicatie van gevoelige details.

## Bronnen

Codeberg. (z.d.). *What is Codeberg?* Geraadpleegd op 26 oktober 2026, van https://docs.codeberg.org/getting-started/what-is-codeberg/

Forgejo. (z.d.). *Forgejo: Beyond coding. We forge.* Geraadpleegd op 26 oktober 2026, van https://forgejo.org/

Open Source Initiative. (z.d.). *The Open Source Definition*. Geraadpleegd op 26 oktober 2026, van https://opensource.org/osd

Open Source Initiative. (z.d.). *OSI approved licenses*. Geraadpleegd op 26 oktober 2026, van https://opensource.org/licenses

World Wide Web Consortium. (2024, 12 december). *Web Content Accessibility Guidelines (WCAG) 2.2*. https://www.w3.org/TR/WCAG22/

