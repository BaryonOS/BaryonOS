# Bindend privacycontract: eigen NAS, lokale AI en geïsoleerde apps
Revisie 2, 26 oktober 2026. Vastgelegd naar aanleiding van Simons expliciete correctie. Dit zijn te implementeren eisen; er is nog geen OS-code of werkend bewijs.

## 1. Productgrens
De ingebouwde OS-diensten bewaren of verwerken persoonlijke gegevens uitsluitend op het apparaat zelf of, voor optionele opslag/synchronisatie/back-up, de eigen NAS binnen het eigen lokale netwerk. Geen externe cloudprovider, leverancieraccount, relay, beheerportaal, sleutelbewaarservice of externe OS-telemetrie. Dit geldt ook voor instellingen, indexen, diagnostiek en AI-context.

Zonder NAS moeten lokale kerntaken blijven werken. Een NAS die ontbreekt of buiten het eigen LAN ligt geeft een zichtbare onderbreking, nooit een externe fallback. NAS-inferentie en toegang vanaf buiten het LAN zijn niet stilzwijgend inbegrepen: het gekozen basisontwerp laat AI op het apparaat zelf draaien en sync uitsluitend binnen het eigen LAN.

Internet is niet algemeen verboden: de gebruiker mag andere programma's downloaden en gebruiken. Pakket-/OS-updates en modeldownloads krijgen afgescheiden, beperkte downloadpaden. Geen persoonlijke documenten, context, identificerende telemetrie of NAS-credentials in die verzoeken. De downloadserver ziet wel de technisch noodzakelijke verbindingsgegevens, zoals een bron-IP; 'geen persoonlijke inhoud uploaden' is geen anonimiteitsgarantie. Offline installeren/updaten blijft een te ontwerpen route.

## 2. Eigen NAS: eigendom is geen bewijs van veiligheid
Het apparaat koppelt expliciet met de identiteit van de gekozen NAS. Controleer sleutel/certificaat via een begrijpelijk, geauthenticeerd koppelproces; het protocol wordt later uit onderhouden standaardimplementaties gekozen. Geen blind vertrouwen op hostnaam, ontdekking via multicast, privé-IP of hetzelfde wifi-netwerk. Een eigen LAN kan onbetrouwbare apparaten bevatten. NIST SP 800-207 maakt onderscheid tussen netwerklocatie en daadwerkelijk geauthenticeerde/geautoriseerde toegang (Rose et al., 2020).

Ontwerpeisen:
- Verifieer peer-identiteit bij elke sessie en handel sleutelwisseling/intrekking af.
- Beperk verbindingen tot expliciet geselecteerde lokale interfaces/netwerken én de gekoppelde NAS-identiteit. Ontwerp voor IPv4, IPv6, multihoming, redirects, DNS-rebinding en netwerkovergangen; een privé-IP-check alleen is onvoldoende.
- Geen UPnP/NAT-PMP/portforwarding, vendorrelay, externe tunnel of internetuitwijking door de OS-NAS-dienst.
- Gebruik versleuteld transport. Voor een backupkluis: ook client-side encryptie met sleutels onder beheer van de gebruiker; geen key escrow bij een leverancier.
- Leg sleutelherstel vast met een eigen offline herstelkopie. Verlies van alle sleutels kan gegevens onherstelbaar maken; verzin geen geheime achterdeur.
- Geef een sync-/backupdienst alleen toegang tot de relevante datasets en bewerkingen. Apps krijgen geen NAS-adminaccount of algemeen gemounte NAS-root.
- Behandel NAS-bestandsnamen, metadata en protocolantwoorden als onbetrouwbare invoer.
- Sync is geen back-up. Bewaar herstelpunten die het syncaccount niet kan wissen en een aanvullende eigen offline kopie. Test herstel. NAS-admincompromis blijft een expliciet restrisico.

De OS-dienst kan niet bewijzen dat een willekeurige bestaande NAS nooit zelf extern synchroniseert of beheertelemetrie verstuurt. Vóór een claim 'niemand anders beheert deze gegevens' moet de gekozen NAS-configuratie op externe accounts, cloudsync, beheerkanalen, toegangsrechten en uitgaand verkeer worden beoordeeld. Er is in dit onderzoek geen echte NAS geïnspecteerd of aangepast.

## 3. AI uitsluitend op het apparaat
Inference draait lokaal op CPU/GPU/NPU die op dat apparaat wordt ondersteund. Persoonlijke prompts, embeddings, zoekindexen, chatgeschiedenis, resultaten en inference-logs blijven lokaal. De gebruiker mag hiervan een versleutelde backup op de eigen gekoppelde NAS bewaren.

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

## 5. Negatieve acceptatietests
| ID | Scenario | Vereist resultaat |
|---|---|---|
| LP-01 | NAS onbereikbaar; internet beschikbaar | Lokale taken blijven werken; sync stopt zichtbaar; geen externe opslag/relay |
| LP-02 | Een ander LAN-apparaat claimt NAS-naam/IP | Geen sessie of gegevens zonder correcte gekoppelde identiteit |
| LP-03 | Apparaat verhuist naar ander netwerk | NAS-sync wordt niet buiten de toegestane lokale netwerkcontext voortgezet |
| LP-04 | Lokale AI; internet uit | Ondersteunde inferentietaak werkt met geïnstalleerd model |
| LP-05 | AI probeert upload, subprocess of helperroute | Uitgaand verkeer/ongeautoriseerde delegatie geweigerd |
| LP-06 | Te weinig RAM of model ontbreekt | Expliciete lokale beperking; geen cloudfallback |
| LP-07 | Modeldownloader vraagt persoonlijke context | Geen toegang tot documenten, historie, indexen of NAS-keys |
| LP-08 | Externe app geïnstalleerd zonder extra rechten | Geen leesrecht buiten appopslag, geen NAS/LAN/internetrechten |
| LP-09 | App heeft internetrecht, maar geen documentrecht | Persoonlijke testbestanden ook niet via helperdienst leesbaar |
| LP-10 | Rechten ingetrokken, oude handle behouden | Verdere geautoriseerde operaties volgen het gespecificeerde revocatiemodel; bestaande kopieën apart benoemd |
| LP-11 | Ransomware via gecompromitteerde syncbevoegdheid | Beschermde backups niet door syncaccount gewist; herstel naar geteste versie mogelijk |
| LP-12 | Gemodificeerd model/pakket/update | Integriteits-/authenticiteitsfout gedetecteerd; geen stilzwijgende uitvoering |
| LP-13 | Schone installatie + periodieke diensten | Alleen beschreven noodzakelijke verbindingen; geen externe telemetrie of inhoudsuploads |
| LP-14 | Kwaadaardige NAS-antwoorden/bestandsmetadata | Begrensde foutafhandeling; geen proces-/bevoegdheidsontsnapping |

Leg platform, versie, netwerkopstelling, observatiepunt en ontbrekende coverage vast. Een lege app zonder netwerkstack bewijst niet dat deze eisen later worden afgedwongen.

## 6. 'Niet te hacken' als ambitie
Geen absolute onhackbaarheidsclaim. Het concrete doel is minimale aanvalsmogelijkheden, technisch afgedwongen rechten, beperking van schade en controleerbaar herstel. Hardware, firmware, kernel, NAS en toegestane apps kunnen fouten bevatten. Formeel bewijs blijft gebonden aan de bewezen eigenschap en aannames. Benoem rest-risico's in plaats van een oncontroleerbare garantie.

## 7. Ontwikkelmiddelen zijn geen OS-runtime
Codex en een Git-host zijn middelen om het project te ontwikkelen; zij worden geen onderdelen van het geïnstalleerde OS. Deze opdracht wijzigt geen bestaande Codex-, account- of NAS-instellingen. Gebruik bij ontwikkeling synthetische testgegevens en geen echte persoonlijke NAS-inhoud. Een eis om ook alle ontwikkeling uitsluitend offline te doen zou een andere workflow vereisen; dat is hier niet automatisch aangenomen.

## Bron
Rose, S., Borchert, O., Mitchell, S., & Connelly, S. (2020). *Zero trust architecture* (NIST Special Publication 800-207). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.SP.800-207

De concrete componenten en tests hierboven zijn eigen ontwerpvoorstellen, geen claims dat NIST dit specifieke OS-ontwerp heeft goedgekeurd.
