# Bindend privacycontract: internet, persoonlijke NAS en gedeelde opslag
Revisie 4, 28 oktober 2026. Dit expliciete projectbesluit vervangt zowel de eerdere beperking tot een eigen NAS binnen het eigen LAN als het absolute verbod op leverancierscloud. De lokale AI-eis blijft gelden. Externe opslagproviders zijn uitsluitend optioneel en mogen als onbetrouwbare opslaglaag voor client-side versleutelde opaque objecten worden gebruikt; zij worden geen sleutelhouder, OS-vertrouwensanker of stille fallback.

**Bewijsstatus:** dit zijn bindende product- en integratie-eisen. `services/nas/` bevat N01 met echte hostencryptie/opslagbeheer en de afzonderlijke N02-loopbackproef met werkelijk geauthenticeerde hybride TLS. `ui/prototype/` bevat gesimuleerde productflows. Native netwerkstack, NAS-distributie, pairing tussen echte apparaten, externe provideradapters, sleutelservice en kernelafdwinging blijven open. Scope: [NAS-SHARING-AND-INTERNET.md](implementation/NAS-SHARING-AND-INTERNET.md), [ENCRYPTED-OBJECT-STORAGE.md](implementation/ENCRYPTED-OBJECT-STORAGE.md) en de [verificatie-/PQC-aanvulling ADR-028](implementation/VERIFIABLE-SECURITY-AND-PQC.md). De bestaande NAS-beheerdersrechten blijven gelden; de externe provider krijgt geen vergelijkbare beheer- of decryptiebevoegdheid over lokale data.

## 1. Internet en gegevenslocatie

Nieuw OS moet internet kunnen gebruiken. Netwerklocatie of eigendom schept geen automatisch vertrouwen: authenticatie, autorisatie en isolatie gelden ook thuis (Rose et al., 2020). Internettoegang maakt een app of dienst niet bevoegd om documenten, NAS-data of sleutels te lezen.

Persoonlijke gegevens blijven lokaal of gaan optioneel **client-side/end-to-end versleuteld** naar:
- een eigen, expliciet gekoppelde NAS;
- ruimte op een expliciet gekoppelde NAS van vrienden of familie;
- een expliciet gekozen externe opslagprovider die uitsluitend opaque ciphertextobjecten ontvangt.

Een NAS kan op hetzelfde LAN of bij iemand anders thuis staan. LAN is de standaardroute voor persoonlijke NAS-opslag. Toegang via internet vereist afzonderlijke toestemming voor de gekoppelde NAS-identiteit en een werkelijk geauthenticeerd transport. Een externe provider wordt afzonderlijk gekoppeld en mag een eigen account vereisen, maar Nieuw OS behandelt die provider als onbetrouwbare opslag en geeft hem geen inhoudssleutel. Geen stilzwijgende verruiming bij netwerkverandering, geen verplichte leverancierscloud, geen provider- of beheerder-key-escrow, geen stille relay/fallback en geen externe OS-telemetrie.

Lokale kerntaken blijven zonder NAS of internet werken. Een onbereikbare of ingetrokken opslagplaats wordt zichtbaar gepauzeerd; bestanden worden niet naar een andere host doorgestuurd. Lokale kopieën blijven van de gebruiker. NAS-inferentie is niet toegestaan als alternatief voor de lokale systeem-AI.

Pakket-/OS-updates en modeldownloads krijgen afgescheiden, beperkte downloadpaden zonder persoonlijke context of NAS-credentials. De bestemming ziet noodzakelijke verbindingsmetadata zoals het bron-IP. Dit is geen anonimiteitsgarantie.

## 2. Eenvoudige persoonlijke cloud met gescheiden bevoegdheden

Privacy en gebruiksgemak zijn gezamenlijke producteisen: één keer koppelen en de scope kiezen, daarna automatische verbinding en synchronisatie binnen die autoriteit. Extra bevestiging is bedoeld voor een nieuwe vertrouwensrelatie, uitgebreid recht of ingrijpende handeling. De dagelijkse ervaring mag geen systeembeheerkennis vereisen; tijdelijke host-/browserproefbeperkingen zijn geen blijvende productkeuzes. Zie DESIGN-AMBITION.md voor de concrete taakcriteria.

Hetzelfde OS-product krijgt een clientrol en een NAS-rol. De gewenste bediening is: kies opslagrol, stel ruimte in, koppel via een eenmalige uitnodiging en verifieer de identiteit. Geen verplicht leveranciersaccount. 'Haast plug & play' is een productdoel, geen bewijs dat willekeurige routers/NAT/CGNAT zonder verdere inrichting direct bereikbaar zijn.

| Actor | Wel toegestaan | Niet automatisch toegestaan |
|---|---|---|
| Gebruiker/client | Eigen inhoud versleutelen/ontsleutelen; eigen gekoppelde namespace gebruiken; eigen sleutelherstel beheren | Andere namespaces, algemene NAS-adminrechten of netwerk voor alle apps |
| NAS-beheerder | Uitnodigen, quota bepalen, toegang intrekken, eigen opgeslagen gastkopieën en staging verwijderen zonder toestemming van de gast | Gastinhoud, documentnamen, decryptiesleutels of key escrow |
| Sync-/transportdienst | Toegestane versleutelde objecten naar de gekoppelde identiteit vervoeren | Willekeurige documenten ontsleutelen of een andere provider kiezen |
| Leverancier van OS-/modelbestanden | Gekozen update-/modelbestanden aanbieden via afgescheiden pad | Persoonlijke plaintextopslag, verplichte OS-accountkoppeling, herstelkeys of telemetrie |
| Expliciet gekozen externe opslagprovider | Opaque versleutelde objecten bewaren/vervoeren binnen de toegekende provider-namespace | Documentplaintext, gevoelige manifestinhoud, Nieuw-OS-inhoudssleutels, lokale zoekindex, AI-context of rechten op andere providers/apps |

### 2.1 Koppeling en netwerk

- Discovery, een hostnaam, privé-IP of hetzelfde wifi-netwerk zijn geen authenticatie.
- Koppeling bindt een eenmalige, vervallende uitnodiging aan de NAS-identiteit, namespace en afgesproken capaciteit. Een zichtbare naam is geen autorisatie-ID.
- Controleer de identiteit via QR of een reeds vertrouwd kanaal. Een korte vergelijkcode alleen mag geen herbruikbaar wachtwoord worden.
- De native transportadapter moet een onderhouden standaardprotocol voor hybride postkwantum-sleuteluitwisseling gebruiken, met echte peer-/clientauthenticatie en zonder stille klassieke fallback. N02 onderzoekt TLS 1.3/X25519MLKEM768 volgens RFC 10024 op loopback; de Ed25519-testauthenticatie blijft klassiek. Mutaties gebruiken geen TLS 0-RTT wegens replayrisico's. Koppel herhaalde requests aan operatie, identiteit en generatie; bind bestaande transportautoriteit aan app en intrekkingsgeneratie.
- De trusted adapter levert netwerk-/identiteitsfeiten; een request mag zichzelf niet authenticeren met een boolean als `verified: true`.
- Internetrecht is peergebonden. Identiteitsafwijking, een redirect of DNS-wijziging naar een ongeautoriseerde peer geeft een fout, geen fallback.
- Geen automatische UPnP/portforwarding, open beheerpoort of afhankelijkheid van een commerciële relay als bijwerking. Directe bereikbaarheid of een afzonderlijk expliciet gekozen eigen netwerkroute moet eerst zijn aangetoond.

### 2.2 Inhoud blijft onder de sleutel van de gebruiker

- Versleutel vóór upload de inhoud, documentnamen en gevoelige manifests/metadata. Gebruik onderhouden authenticated-encryption-implementaties; geen zelfgemaakte cryptoprimitives.
- Bind schema, namespace, object en versie cryptografisch aan de ciphertext. Onjuiste sleutel, gewijzigde tag/data/header of verkeerde namespace geeft geen geaccepteerde plaintext.
- Alleen gebruikersapparaten houden inhoudssleutels; een NAS-adminreset geeft geen decryptierecht. Er is geen verborgen recoverykey bij de host.
- Sleutelherstel verloopt via een eigen offline herstelkopie of vertrouwd eigen apparaat. Zonder alle sleutelkopieën kan inhoud onherstelbaar zijn. Een nieuwe client zonder vertrouwde versiegeschiedenis krijgt geen onbewezen rollbackgarantie.
- De sleutelverwerkende client moet uit de vertrouwde OS-distributieketen komen. Een NAS mag geen willekeurige clientcode aanleveren die daarna gebruikerssleutels verwerkt.
- Een NAS of externe provider ziet noodzakelijke opslag-/verbindingsmetadata binnen zijn rol: bijvoorbeeld toegewezen ruimte of accountgebruik, opaque object-ID's en activiteit/verbindingsgegevens. Bestandsgroottes, aantallen en timing worden niet automatisch verborgen door encryptie.
- Encryptie beschermt inhoud, niet beschikbaarheid. Een gecompromitteerd gebruikersapparaat of kwaadaardige clientupdate valt buiten een garantie dat de NAS-beheerder de inhoud niet kan lezen.

### 2.3 Beheerder kan ruimte altijd terugnemen

De beheerder hoeft **geen toestemming, handtekening, verbinding of decryptiesleutel van de gast** te verkrijgen om de NAS-kopie te verwijderen. Quota, blokkeren en verwijderen zijn afzonderlijke beheercapabilities.

Intrekking blokkeert eerst duurzaam verdere toegang en publicatie van uploads; daarna worden beheerde ciphertext, onafgemaakte uploads en beheerde herstelkopieën opgeruimd. Een bevestiging mag alleen de werkelijk voltooide stap beschrijven. Een I/O-fout mag geen valse succesvolle verwijdering opleveren.

Oude sessies, tickets en uploadcommits worden geweigerd. Verwijderde namespaces blijven als ingetrokken bekend; synchronisatie mag ze niet stilzwijgend opnieuw aanmaken. Een nieuwe uitnodiging krijgt een nieuwe namespace. Reserveringen voor lopende uploads tellen mee in quota. Een volle gastquota verhindert de beheeractie niet.

Dit verwijderrecht betreft de kopieën op de betreffende NAS. Het geeft geen recht om lokale bestanden of kopieën op andere apparaten te wissen. De host kan niet garanderen dat een kwaadwillige beheerder nergens een ciphertextbackup heeft behouden; fysieke secure erase en cryptografisch wissen zijn afzonderlijke eigenschappen.

De ontvanger ziet vóór koppelen: de beheerder kan de opslag beëindigen, inhoud is voor de beheerder versleuteld, en een eigen kopie/back-up blijft nodig. Sync is geen zelfstandige back-up. Beschermde versies mogen gewone synccredentials weerstaan, maar mogen de hardware-eigenaar niet permanent de controle over zijn ruimte ontnemen.

## 2.4 Externe provider als opaque opslaglaag

Een algemene cloudopslagdienst mag alleen na een expliciete gebruikerskeuze als
onbetrouwbare opslagbackend worden gekoppeld. Dit verandert het trustmodel niet:
de provider is geen onderdeel van de Security Floor en krijgt geen inhoudssleutel.

- Versleutel inhoud, documentnamen en gevoelige manifests **vóór** de provideradapter ze ontvangt.
- Gebruik een eigen versieerbaar object-/containerformaat, maar geen zelfbedachte cryptografische primitief; zie [ENCRYPTED-OBJECT-STORAGE.md](implementation/ENCRYPTED-OBJECT-STORAGE.md).
- Providercredentials en inhoudssleutels blijven gescheiden. Compromittering van alleen het provideraccount mag geen plaintext opleveren.
- De provideradapter krijgt alleen de bedoelde ciphertextnamespace en netwerkbestemming, niet automatisch lokale documenten, zoekindexen of AI-context.
- Een provider mag ciphertext verwijderen, blokkeren of terugrollen. Encryptie is dus geen back-up- of beschikbaarheidsgarantie; onafhankelijke herstelkopieën en rollbackdetectie blijven nodig.
- De gebruiker kan meerdere opslaglocaties kiezen. Nieuw OS schakelt nooit stil over naar een andere provider wanneer de gekozen opslag faalt.
- Claims als “Microsoft/Google kunnen dit absoluut nooit kraken” zijn niet toegestaan. De toetsbare claim is dat de provider de decryptiesleutel niet bezit en dat gecontroleerde ciphertext zonder zo'n sleutel geen geaccepteerde plaintext oplevert onder het gedocumenteerde dreigingsmodel.
- Account-, IP-, timing-, volume- en andere noodzakelijke provider-metadata kunnen zichtbaar blijven; client-side encryptie is geen anonimiteitslaag.

## 3. AI uitsluitend op het apparaat
Inference draait lokaal op CPU/GPU/NPU die op dat apparaat wordt ondersteund. Persoonlijke prompts, embeddings, zoekindexen, chatgeschiedenis, resultaten en inference-logs blijven lokaal tijdens gebruik. De gebruiker mag hiervan een client-side versleutelde back-up bewaren op een expliciet gekoppelde persoonlijke NAS, ontvangen ruimte bij vrienden/familie of een expliciet gekozen externe provider volgens ENCRYPTED-OBJECT-STORAGE.md. Een NAS of provider voert geen systeeminferentie uit en ontvangt geen plaintext AI-context als onderdeel van deze back-uplaag.

- De inferentiedienst krijgt geen directe LAN-/internettoegang.
- Ontbrekend model of onvoldoende RAM/rekenkracht geeft een fout, wachtrij, kleiner lokaal model of uitgeschakelde functie; nooit externe inference.
- Een aparte modeldownloader haalt gekozen modelbestanden op zonder toegang tot prompts, documenten, indexen of NAS-credentials. Pin herkomst/versie/integriteit en controleer licentie.
- Behandel modelbestanden en converters als onbetrouwbaar: geen willekeurige modelcode/installatiescripts uitvoeren in de inferenceomgeving; gecontroleerd formaat en begrensde parser.
- AI krijgt alleen de expliciet gekozen bestanden/context. Acties verlopen via een beperkte lokale broker met taakgebonden capabilities. De broker mag geen verkapte algemene shell- of uploaddienst zijn.
- Ook bij model- of toolfouten blijft netwerk geweigerd. Test subprocessen, gedelegeerde diensten en lokale sockets als mogelijke omweg.
- Lokale AI is optioneel en verwijderbaar; het OS mag er niet van afhankelijk zijn.

Een derde-partijprogramma dat online diensten gebruikt wordt niet automatisch onderdeel van de ingebouwde systeem-AI. Het krijgt uitsluitend expliciet toegekende apprechten; het OS biedt zo'n dienst niet als alternatief voor de lokale systeem-AI aan.

## 4. Open app-ecosysteem met expliciete grenzen
Programma's mogen uit meerdere bronnen komen; geen verplichte centrale appwinkel. 'Downloadbaar' betekent niet automatisch 'compatibel met dit nieuwe OS'. De native API/ABI en eventuele compatibiliteitslagen blijven aparte ontwikkeltaken.

Installeren kent geen algemene machtiging toe. Standaard: eigen appopslag, geen toegang tot andere apps, persoonlijke mappen, NAS, internet, camera, microfoon of beheerfuncties. Geef afzonderlijke, zichtbare en intrekbare rechten voor:
- geselecteerde bestanden en concrete acties (lezen, schrijven, exporteren);
- een benoemde NAS-dataset of een systeemportaal;
- LAN en internet afzonderlijk, met begrensde bestemmingen waar haalbaar;
- apparaten en gevoelige systeemhandelingen.

Een ondertekening toont herkomst onder de betreffende sleutel, geen betrouwbaarheid van de app. Pakketinstallatie mag niet vanzelf onbeperkte beheerscripts uitvoeren. Compatibiliteitslagen vallen binnen hetzelfde isolatiemodel.

**Cruciale bewijsgrens:** een app die een bestand mag lezen en een server mag benaderen kan de inhoud mogelijk naar die server sturen. Het OS kan niet tegelijk algemene lees-/netwerkvrijheid geven en beloven dat toegestane data nooit weggaan. Toon gecombineerde rechten en gebruik voor gevoelige gegevens bij voorkeur netwerkloze verwerking met een gecontroleerd deel-/exportportaal. Een browser kan internet nodig hebben zonder automatisch toegang tot de gehele thuismap of NAS.

Intrekken stopt toekomstig gebruik via de gecontroleerde grens, maar wist geen kopie die een app eerder mocht maken. Test ook gegevenslekken via tussendiensten, klembord, gedeelde opslag, logs en foutmeldingen. Volledige informatieflowbeveiliging vergt meer dan enkele schakelaars.


### 4.1 Security Floor en uitzetbare privacyuitleg

Privacykeuzes en systeembeveiliging zijn verschillende lagen. Een gebruiker mag
bijvoorbeeld een app internet, microfoon, cloudsync of vrijwillige telemetrie
toestaan. Dat verruimt uitsluitend de daarbij horende expliciete capability en
schakelt kernel-/procesisolatie, geheugenbescherming, sleutelisolatie,
pakket-/update-authenticiteit of bescherming van niet-gedeelde data niet uit.

De UI onderscheidt een noodzakelijke toegangsbeslissing van aanvullende
educatieve privacyuitleg. De gebruiker mag die uitleg globaal of per app
uitschakelen. “Privacyuitleg uit” betekent nooit “alles toestaan”. Bestaande
grants blijven ongewijzigd en nieuwe rechten worden afzonderlijk verleend of
geweigerd. Een eenmaal geldige grant hoeft niet voor iedere normale herhaling
opnieuw te worden gevraagd.

## 5. Acceptatietests

Bestaande LP-identifiers blijven behouden; LP-03 is expliciet herzien voor peergebonden internettoegang.

| ID | Scenario | Vereist resultaat |
|---|---|---|
| LP-01 | NAS onbereikbaar; internet beschikbaar | Lokale taken werken; geen andere provider/relay als fallback |
| LP-02 | Ander apparaat claimt NAS-naam/IP | Geen gegevens zonder gekoppelde geverifieerde identiteit |
| LP-03 | Netwerk wisselt van LAN naar internet | Geen sync zonder expliciet internetrecht voor dezelfde geverifieerde peer |
| LP-04 | Systeem-AI; internet uit | Ondersteunde lokale taak werkt met geïnstalleerd model |
| LP-05 | AI probeert socket, subprocess of helperupload | Directe en gedelegeerde netwerktoegang geweigerd |
| LP-06 | RAM of model ontbreekt | Expliciete lokale beperking, geen cloudinferentie |
| LP-07 | Modeldownloader vraagt persoonlijke context | Documenten, historie, indexen en NAS-keys geweigerd |
| LP-08 | Nieuwe app zonder grants | Geen NAS/LAN/internet of algemene documenttoegang |
| LP-09 | App heeft alleen internetrecht | Geen persoonlijk document via bestand-, NAS- of helperroute |
| LP-10 | Recht ingetrokken, oude handle behouden | Toekomstige operaties geweigerd; eerdere externe kopieën apart benoemd |
| LP-11 | Synccredentials gecompromitteerd | Afgescheiden herstelkopie blijft bruikbaar; admin mag zijn eigen NAS-ruimte wel wissen |
| LP-12 | Gewijzigd model/pakket/update | Geen uitvoering zonder geslaagde authenticiteit/integriteit |
| LP-13 | Schone installatie en periodieke diensten | Alleen beschreven noodzakelijke verbindingen; geen OS-telemetrie |
| LP-14 | Kwaadaardige NAS-antwoorden/metadata | Begrensde fouten, geen code-uitvoering of bevoegdheidsontsnapping |
| LP-15 | NAS-state plus admincredentials uitgelezen | Geen documentnamen/plaintext/gebruikerssleutels; noodzakelijke metadata expliciet |
| LP-16 | Admin verwijdert ruimte; gast offline/weigert/key kwijt | Intrekken en verwijderen zonder gastconsent of decryptiesleutel |
| LP-17 | Oude sessie/upload na eviction of herstart | Geen verdere publicatie of namespaceherleving; nieuwe invite = nieuwe namespace |
| LP-18 | Meerdere uploads, volle quota, abort/herhaling | Reserveringen begrensd en correct vrijgegeven; admin niet door gastquota geblokkeerd |
| LP-19 | Invite hergebruikt/verlopen/verkeerde peer | Koppeling geweigerd; geen automatische vertrouwenstoekenning |
| LP-20 | NAS-adminwachtwoord gereset | Geen nieuwe decryptiebevoegdheid op gastdata |
| LP-21 | Beheerscherm | Alleen quota/gebruik/identiteit/status; geen gastdocumentlijst of inhoud |
| LP-22 | Ciphertext/header/object/versie gemanipuleerd | Authenticatiefout; geen gedeeltelijk geaccepteerde plaintext |
| LP-23 | Internet aan voor browser, niet voor AI of NAS | Alleen expliciete appbestemming toegestaan; geen rechtenovererving |
| LP-24 | Client- en NAS-rol op hetzelfde OS | Geauthenticeerd koppelen/ontvangen/aanbieden met begrijpelijke fout- en herstelroute |
| LP-25 | Crash tijdens revoke/purge of echte schijffout | Oude autoriteit herleeft niet; geen onterechte duurzaamheids- of verwijderclaim |
| LP-26 | Alleen providercredentials buitgemaakt | Geen documentplaintext, gevoelige manifestinhoud of Nieuw-OS-inhoudssleutel beschikbaar |
| LP-27 | Providerobject/tag/header/chunk/revisie gemanipuleerd | Authenticatie-/rollbackfout volgens het gekozen threat model; geen gedeeltelijke plaintext |
| LP-28 | Externe provider offline/verwijdert data | Lokale werking en expliciete herstelstatus; geen stille providerfallback of valse back-upclaim |
| LP-29 | Privacyuitleg globaal/per app uitgeschakeld | Geen educatieve popup; capabilityset blijft byte-/semantisch gelijk en nieuwe toegang blijft afzonderlijk |
| LP-30 | App krijgt internet en vrijwillige telemetry, geen documentgrant | Telemetriepad mag de verleende scope gebruiken maar krijgt geen document-/NAS-/keycapability |
| LP-31 | App krijgt document én internet na bewuste keuze | UI toont gecombineerde exfiltratieruimte; overige documenten/apps/secrets blijven afgeschermd |

Hosttests, browsermocks en native OS-/netwerktests worden afzonderlijk gerapporteerd. Een hosttest of policyboolean bewijst geen veilig netwerk op de eigen kernel. Leg versie, netwerkopstelling, observatiepunt en ontbrekende coverage vast.

## 6. Veiligheidsdoel en bewijsgrenzen

Het doel is minimale bevoegdheid, beperking van schade, gecontroleerde verbindingen en herstelbaarheid. De **Security Floor** blijft van kracht wanneer een gebruiker bewust minder privacy kiest: een ruimere gegevens- of netwerkgrant geeft niet automatisch meer systeemautoriteit. Internet brengt een bereikbaar aanvalsoppervlak mee; er is geen absolute garantie 'niet te hacken'. Authenticatie/encryptie vervangen geen parsergrenzen, kernelisolatie, veilige updates, onderhoud of fysieke-hardwareonderzoeken.

Client-side encryptie van externe opslag betekent niet dat een provider niets kan observeren of verwijderen. Zonder provider-held decryptiesleutel moet inhoud binnen het geteste dreigingsmodel vertrouwelijk blijven; absolute onkraakbaarheid, anonimiteit en beschikbaarheid worden niet geclaimd.

## 7. Ontwikkelmiddelen zijn geen OS-runtime

Codex, Git-host en de Node-ontwikkelhost zijn bouw-/testmiddelen. Zij worden geen verplichte diensten van het geïnstalleerde OS. Gebruik synthetische testdata. Deze wijziging configureert geen echte NAS, router, externe account of productiekey.

## Primaire bronnen

- OpenJS Foundation. (2026). *Crypto: Node.js v24.22.1 documentation*. https://nodejs.org/docs/latest-v24.x/api/crypto.html — maintained crypto-API, randomness, AAD en authenticated decryption; geraadpleegd 27 oktober 2026.
- Rescorla, E. (2018). *The Transport Layer Security (TLS) protocol version 1.3* (RFC 8446). IETF. https://www.rfc-editor.org/rfc/rfc8446.html — transport en 0-RTT/replay; geraadpleegd 27 oktober 2026.
- Rose, S., Borchert, O., Mitchell, S., & Connelly, S. (2020). *Zero trust architecture* (NIST SP 800-207). NIST. https://doi.org/10.6028/NIST.SP.800-207

Quota, rolverdeling en beheerder-verwijderrecht zijn projectbesluiten op basis van mijn opdracht; de bronnen certificeren dit project niet.

