# Bindend privacycontract: internet, persoonlijke NAS en gedeelde opslag
Revisie 3, 27 oktober 2026. Deze expliciete gebruikerswijziging vervangt de eerdere beperking tot een eigen NAS binnen het eigen LAN. De lokale AI-eis en het verbod op leverancierscloud/fallback blijven gelden.

**Bewijsstatus:** dit zijn bindende product- en integratie-eisen. `services/nas/` bevat een uitvoerbare hostproef met echte encryptie en opslagbeheer; `ui/prototype/` bevat gesimuleerde productflows. Een native netwerkstack, NAS-distributie, transportauthenticatie en integratie met kernelcapabilities zijn hiermee niet aangetoond. Concrete scope en acceptatie: [NAS-SHARING-AND-INTERNET.md](implementation/NAS-SHARING-AND-INTERNET.md).

## 1. Internet en gegevenslocatie

Nieuw OS moet internet kunnen gebruiken. Netwerklocatie of eigendom schept geen automatisch vertrouwen: authenticatie, autorisatie en isolatie gelden ook thuis (Rose et al., 2020). Internettoegang maakt een app of dienst niet bevoegd om documenten, NAS-data of sleutels te lezen.

Persoonlijke gegevens blijven lokaal of gaan optioneel **end-to-end versleuteld** naar:
- een eigen, expliciet gekoppelde NAS;
- ruimte op een expliciet gekoppelde NAS van vrienden of familie.

Deze NAS kan op hetzelfde LAN of bij iemand anders thuis staan. LAN is de standaardroute. Toegang via internet vereist afzonderlijke toestemming voor de gekoppelde NAS-identiteit en een werkelijk geauthenticeerd transport. Geen stilzwijgende verruiming bij netwerkverandering. Geen leverancierscloud, verplicht leverancieraccount, sleutelbewaring door leverancier of beheerder, stille relay/fallback of externe OS-telemetrie.

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
| Leverancier | Gekozen update-/modelbestanden aanbieden via afgescheiden pad | Persoonlijke opslag, verplichte accounts, herstelkeys of telemetrie |

### 2.1 Koppeling en netwerk

- Discovery, een hostnaam, privé-IP of hetzelfde wifi-netwerk zijn geen authenticatie.
- Koppeling bindt een eenmalige, vervallende uitnodiging aan de NAS-identiteit, namespace en afgesproken capaciteit. Een zichtbare naam is geen autorisatie-ID.
- Controleer de identiteit via QR of een reeds vertrouwd kanaal. Een korte vergelijkcode alleen mag geen herbruikbaar wachtwoord worden.
- De toekomstige transportadapter verifieert TLS 1.3 en clientidentiteit. Mutaties gebruiken geen TLS 0-RTT; die route heeft replayrisico's (Rescorla, 2018). Koppel herhaalde requests aan operatie, identiteit en generatie.
- De trusted adapter levert netwerk-/identiteitsfeiten; een request mag zichzelf niet authenticeren met een boolean als `verified: true`.
- Internetrecht is peergebonden. Identiteitsafwijking, een redirect of DNS-wijziging naar een ongeautoriseerde peer geeft een fout, geen fallback.
- Geen automatische UPnP/portforwarding, open beheerpoort of afhankelijkheid van een commerciële relay als bijwerking. Directe bereikbaarheid of een afzonderlijk expliciet gekozen eigen netwerkroute moet eerst zijn aangetoond.

### 2.2 Inhoud blijft onder de sleutel van de gebruiker

- Versleutel vóór upload de inhoud, documentnamen en gevoelige manifests/metadata. Gebruik onderhouden authenticated-encryption-implementaties; geen zelfgemaakte cryptoprimitives.
- Bind schema, namespace, object en versie cryptografisch aan de ciphertext. Onjuiste sleutel, gewijzigde tag/data/header of verkeerde namespace geeft geen geaccepteerde plaintext.
- Alleen gebruikersapparaten houden inhoudssleutels; een NAS-adminreset geeft geen decryptierecht. Er is geen verborgen recoverykey bij de host.
- Sleutelherstel verloopt via een eigen offline herstelkopie of vertrouwd eigen apparaat. Zonder alle sleutelkopieën kan inhoud onherstelbaar zijn. Een nieuwe client zonder vertrouwde versiegeschiedenis krijgt geen onbewezen rollbackgarantie.
- De sleutelverwerkende client moet uit de vertrouwde OS-distributieketen komen. Een NAS mag geen willekeurige clientcode aanleveren die daarna gebruikerssleutels verwerkt.
- De host ziet noodzakelijke beheermetadata: toegewezen ruimte, gebruik, opaque object-ID's en activiteit/verbindingsgegevens. Bestandsgroottes en timing worden niet automatisch verborgen door encryptie.
- Encryptie beschermt inhoud, niet beschikbaarheid. Een gecompromitteerd gebruikersapparaat of kwaadaardige clientupdate valt buiten een garantie dat de NAS-beheerder de inhoud niet kan lezen.

### 2.3 Beheerder kan ruimte altijd terugnemen

De beheerder hoeft **geen toestemming, handtekening, verbinding of decryptiesleutel van de gast** te verkrijgen om de NAS-kopie te verwijderen. Quota, blokkeren en verwijderen zijn afzonderlijke beheercapabilities.

Intrekking blokkeert eerst duurzaam verdere toegang en publicatie van uploads; daarna worden beheerde ciphertext, onafgemaakte uploads en beheerde herstelkopieën opgeruimd. Een bevestiging mag alleen de werkelijk voltooide stap beschrijven. Een I/O-fout mag geen valse succesvolle verwijdering opleveren.

Oude sessies, tickets en uploadcommits worden geweigerd. Verwijderde namespaces blijven als ingetrokken bekend; synchronisatie mag ze niet stilzwijgend opnieuw aanmaken. Een nieuwe uitnodiging krijgt een nieuwe namespace. Reserveringen voor lopende uploads tellen mee in quota. Een volle gastquota verhindert de beheeractie niet.

Dit verwijderrecht betreft de kopieën op de betreffende NAS. Het geeft geen recht om lokale bestanden of kopieën op andere apparaten te wissen. De host kan niet garanderen dat een kwaadwillige beheerder nergens een ciphertextbackup heeft behouden; fysieke secure erase en cryptografisch wissen zijn afzonderlijke eigenschappen.

De ontvanger ziet vóór koppelen: de beheerder kan de opslag beëindigen, inhoud is voor de beheerder versleuteld, en een eigen kopie/back-up blijft nodig. Sync is geen zelfstandige back-up. Beschermde versies mogen gewone synccredentials weerstaan, maar mogen de hardware-eigenaar niet permanent de controle over zijn ruimte ontnemen.

## 3. AI uitsluitend op het apparaat
Inference draait lokaal op CPU/GPU/NPU die op dat apparaat wordt ondersteund. Persoonlijke prompts, embeddings, zoekindexen, chatgeschiedenis, resultaten en inference-logs blijven lokaal. De gebruiker mag hiervan een end-to-end versleutelde backup op een expliciet gekoppelde persoonlijke NAS bewaren, inclusief ontvangen ruimte bij vrienden/familie. De NAS voert geen inferentie uit.

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

Hosttests, browsermocks en native OS-/netwerktests worden afzonderlijk gerapporteerd. Een hosttest of policyboolean bewijst geen veilig netwerk op de eigen kernel. Leg versie, netwerkopstelling, observatiepunt en ontbrekende coverage vast.

## 6. Veiligheidsdoel en bewijsgrenzen

Het doel is minimale bevoegdheid, beperking van schade, gecontroleerde verbindingen en herstelbaarheid. Internet brengt een bereikbaar aanvalsoppervlak mee; er is geen absolute garantie 'niet te hacken'. Authenticatie/encryptie vervangen geen parsergrenzen, kernelisolatie, veilige updates, onderhoud of fysieke-hardwareonderzoeken.

## 7. Ontwikkelmiddelen zijn geen OS-runtime

Codex, Git-host en de Node-ontwikkelhost zijn bouw-/testmiddelen. Zij worden geen verplichte diensten van het geïnstalleerde OS. Gebruik synthetische testdata. Deze wijziging configureert geen echte NAS, router, externe account of productiekey.

## Primaire bronnen

- OpenJS Foundation. (2026). *Crypto: Node.js v24.22.1 documentation*. https://nodejs.org/docs/latest-v24.x/api/crypto.html — maintained crypto-API, randomness, AAD en authenticated decryption; geraadpleegd 27 oktober 2026.
- Rescorla, E. (2018). *The Transport Layer Security (TLS) protocol version 1.3* (RFC 8446). IETF. https://www.rfc-editor.org/rfc/rfc8446.html — transport en 0-RTT/replay; geraadpleegd 27 oktober 2026.
- Rose, S., Borchert, O., Mitchell, S., & Connelly, S. (2020). *Zero trust architecture* (NIST SP 800-207). NIST. https://doi.org/10.6028/NIST.SP.800-207

Quota, rolverdeling en beheerder-verwijderrecht zijn projectbesluiten op basis van de gebruikersopdracht; de bronnen certificeren dit project niet.

