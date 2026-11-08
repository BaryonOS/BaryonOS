# N01 — internet en persoonlijke NAS-ruimte

## Aanvulling P05 — 28 oktober 2026

N01/N03 blijven de **persoonlijke NAS-route**. Privacycontract revisie 4 staat
daarnaast toe dat een gebruiker een algemene externe opslagprovider expliciet
kiest als onbetrouwbare backend voor uitsluitend opaque, client-side
versleutelde objecten. Dat is een afzonderlijke P05-route volgens
[ENCRYPTED-OBJECT-STORAGE.md](ENCRYPTED-OBJECT-STORAGE.md); zij vervangt de NAS
niet, geeft de provider geen inhoudssleutel en is nooit een stille fallback.

Datum: 27 oktober 2026. Projectbesluit: internettoegang, NAS bij de gebruiker thuis als persoonlijke cloud, dezelfde OS-bediening voor client en NAS, en ruimte delen met vrienden/familie zonder hun inhoud te kunnen lezen. De NAS-beheerder mag zijn ruimte eenzijdig terugnemen.

## Status en scope

**Aanvulling N02, 27 oktober 2026:** de oorspronkelijke N01-scope hieronder blijft herkenbaar. De afzonderlijke [TLS-hostproef](../../services/nas/TRANSPORT.md) voegt werkelijk geauthenticeerd hybride loopbacktransport en aan app/generatie gebonden facts toe. Zij levert geen native NAS-netwerkdienst en koppelt de echo nog niet aan de store. Actueel bewijs: `docs/verification/pqc-transport/`; formele/native vervolgcriteria: [VERIFIABLE-SECURITY-AND-PQC.md](VERIFIABLE-SECURITY-AND-PQC.md).

N01 loopt parallel aan M6a. De eigen kernel/netwerkdriver wordt in deze wijziging niet uitgebreid.

| Laag | Levering | Bewijsgrens |
|---|---|---|
| Bindend productcontract | Privacycontract revisie 4, ADR-027 plus ADR-035/036, bijgewerkte architectuur/roadmap/instructies | Eisen en gekozen rolverdeling |
| Versleutelde hostdienst | `services/nas/`: echte clientencryptie, lokale ciphertextopslag, quota, uitnodiging, intrekking en beheer | Node-hostproef; geen native OS-dienst of netwerkserver |
| Netwerkbeleid | Default-deny beslislogica met gescheiden app-/AI-/NAS-rechten | Vertrouwde adapter moet identiteit en verbinding daadwerkelijk verifiëren |
| Materiaal | Mijn NAS, Ruimte ontvangen, Ruimte aanbieden, identiteitscontrole, expliciet internet en hostverwijdering | Browsermock met fictieve data; geen cryptografische veiligheid van de demo |
| Native integratie | Netwerkstack, transportauthenticatie, same-OS-NAS-installatie, persistente sleutels en kernelcapabilities | Nog te bouwen en afzonderlijk te testen |

De daadwerkelijk uitgevoerde testcommando's, getallen en beperkingen staan onder `docs/verification/nas/`. Geen M8-, fysieke-hardware-, internetbereikbaarheids- of productieclaim.

## 1. Gebruikersscenario's

### Productnorm: privacy zonder onnodige bediening

Aanvullende producteis van 27 oktober 2026: de verbonden ervaring en afwerking moeten kunnen overtuigen naast Windows/macOS. Privacy is een standaard onderliggende eigenschap, geen reden om internet, samenwerken, automatisch hervatten of een moderne interface weg te laten. Veiligheid en gebruiksgemak zijn gezamenlijke acceptatiecriteria.

Een gebruiker bevestigt een nieuwe NAS-identiteit en kiest eenmalig de toegestane verbindingsscope. Daarna kiest de routeplanner automatisch het geverifieerde lokale kanaal, of reeds toegestaan direct internet naar dezelfde peer. Geen nieuwe vraag bij iedere wisseling, bestandsbewerking of normale synchronisatie. Mismatch, intrekking en niet-geautoriseerde bestemmingen blijven blokkeren. Offline werken en hervatten mogen geen verborgen dubbele bestanden of dataverlies veroorzaken; echte sync/conflictafhandeling is een nog te bouwen dienst.

De huidige read-only keyrecovery, 8.192-encryptiegrens, kleine testquota en herbevestiging na browserrefresh zijn **tijdelijke proefgrenzen**. Zij zijn geen geaccepteerd dagelijks productgedrag. Voor een productversie moeten veilig blijvend sleutelbeheer, geautomatiseerde rotatie, achtergrondsync, multi-device-toegang en begrijpelijk herstel worden geïmplementeerd en getest. Een feature wordt niet productklaar door een veiligheidsprobleem als permanente gebruikersbeperking te presenteren.

De bronproef demonstreert veilige automatische routeselectie, maar geen echte internetverbinding of achtergrondsync. Onboarding moet toegang, gegevenslocatie en herstel in gewone taal tonen; geavanceerde netwerk-/sleutelinstellingen horen buiten de standaardflow. Vergelijk gebruik met concrete taken en gemeten frictie; claim geen gelijkwaardigheid met bestaande platforms zonder die metingen.

### Eigen NAS instellen

Kies op een tweede geschikt apparaat de NAS-rol van hetzelfde OS. Kies een opslagvolume en maximumcapaciteit; toon welke data verwijderd kunnen worden als het volume opnieuw ingericht zou moeten worden. Deze toekomstige installatie vereist een eigen opdracht voordat een echte schijf wordt gewist. Maak een lokale beheeridentiteit. De client ontdekt een kandidaat of ontvangt een uitnodiging, vergelijkt de identiteit en kiest de gewenste capaciteit. Discovery verleent geen vertrouwen.

### Ruimte ontvangen bij familie of vrienden

De host maakt een eenmalige uitnodiging met een ruimteaanbod. De ontvanger controleert de identiteit en accepteert bewust dat de host de NAS-kopie kan verwijderen. De client genereert lokaal zijn eigen inhoudssleutel; de uitnodiging of beheerder ontvangt die sleutel niet. Koppeling geeft geen toegang tot andere gebruikers of tot het beheer van het apparaat.

Standaard blijft de verbinding lokaal. Gebruik vanaf een ander huis of onderweg vraagt afzonderlijke toestemming voor internet naar die gekoppelde identiteit. Veranderde identiteit onderbreekt de verbinding, ook als hostnaam of adres gelijk blijft.

### Ruimte aanbieden en terugnemen

De host ziet per gast een herkenbare naam of lokale alias, gebruikte ruimte, quotum en status. Geen gastdocumentnamen, voorbeelden of inhoud. 'Toegang intrekken en NAS-kopie verwijderen' vraagt alleen een duidelijke bevestiging van de beheerder. Een offline gast, verloren gastkey, versleuteld object of gastbezwaar blokkeert die beheeractie niet.

Na intrekking werken oude sessies en lopende uploads niet meer. Na herstart blijft de ruimte ingetrokken. Een oude client mag de namespace niet opnieuw aanmaken. Een nieuwe uitnodiging is een nieuw aanbod met een nieuwe namespace.

## 2. Vertrouwensgrenzen

De NAS wordt voor vertrouwelijkheid als onbetrouwbare opslag behandeld. De gebruikersclient en zijn geverifieerde distributieketen houden de sleutel. De NAS moet geen clientupdate of script kunnen opleggen dat inhoudsleutels ontvangt.

De beheercapability staat los van de inhoudssleutel. Een host kan op disk ciphertext lezen, maar dat is geen leesrecht op documenten. De host kan altijd beschikbaarheid beëindigen en noodzakelijke metadata waarnemen. Een lokale apparaatcompromis, verloren sleutel of doelbewuste clientcodevervanging is een andere dreiging.

De browserdemo gebruikt geen echte keys, NAS of netwerk. Haar 'versleuteld'-weergave beschrijft het productcontract en wordt als simulatie gemarkeerd.

## 3. Opslag- en intrekkingsinvarianten

1. Alleen geautoriseerde actieve namespaces mogen eigen objecten publiceren/lezen/verwijderen.
2. NAS-opslag bevat uitsluitend de ciphertextenvelop en noodzakelijke opaque metadata; ook documentnamen zijn versleuteld.
3. Authenticated encryption bindt schema, namespace, object en versie. Volledig verifiëren vóór plaintext teruggeven.
4. Inhoudssleutels worden niet in serverstate, invitations, sessies, administratie of logs opgeslagen.
5. Adminintrekking hoeft geen gebruikerssleutel, gebruikershandtekening of gebruikersconsent.
6. Intrekking en uploadcommit worden onder dezelfde serialisatiegrens beoordeeld. De duurzame ingetrokken toestand gaat vóór cleanup.
7. Staging/sessies/tickets binden aan de huidige namespacegeneratie. Oude autoriteit faalt ook na herstart.
8. Verwijderde namespaces worden niet opnieuw aangemaakt door upload/resume/sync. Nieuw aanbod = nieuwe willekeurige namespace.
9. Quota tellen ciphertext en reserveringen; metadata, objectaantal en aantallen sessies/invites/uploads hebben afzonderlijke grenzen.
10. Gastquota mogen beheerintrekking niet verhinderen. Een echte opslagfout geeft geen onterechte duurzaamheidsbevestiging.
11. Purge is herhaalbaar; administratieve status en werkelijke cleanupstatus blijven onderscheidbaar bij fouten.
12. Verwijderen op deze NAS wist geen gebruikerskopieën op andere apparaten en belooft geen fysieke secure erase of vernietiging van ongecontroleerde backups.

De hostproef moet haar specifieke atomische opslag- en single-writer-aannames documenteren. Een geslaagde processen-herstarttest bewijst geen herstel na fysieke stroomuitval op iedere schijf.

## 4. Netwerkbeleid

| Principal | Standaard | Expliciete verruiming |
|---|---|---|
| Lokale AI-inferentie | Geen LAN/internet of uploadhelper | Geen cloudfallback |
| Nieuwe derdenapp | Eigen appopslag, geen netwerk/documenten/NAS | Afzonderlijke app- en bestemmingsgrants |
| Browser | Geen algemeen document-/NAS-recht | Internetgebruik binnen toegewezen scope |
| NAS-sync | Geen onbekende peer; alleen gekoppelde namespace | Gecontroleerd LAN, vervolgens optioneel internet naar dezelfde identiteit |
| Update-/modeldownloader | Geen persoonlijke context of NAS-keys | Afgebakende downloadbestemming en geverifieerde artifacts |
| NAS-beheer | Lokale beheercapability | Remote beheer is afzonderlijke nog te ontwerpen functie |

Een netwerkgrant is geen documentgrant. Gecombineerde bestands- en internetrechten worden zichtbaar gemaakt; een app die beide terecht bezit kan toegestane inhoud exporteren. Dat risico kan niet met een tekstbelofte worden uitgesloten.

De latere adapter controleert identiteit, TLS 1.3, doel en reële route voordat zij een geautoriseerde principal/verbinding levert. Geen mutaties via TLS 0-RTT. Een extern request mag geen route of authenticatie fabriceren met booleans. Herauthenticatie bij keyrotatie of ingetrokken sessie is verplicht. Geen automatische open beheerpoort, UPnP/portforwarding of relayfallback.

## 5. Cryptografische uitvoering

De hostproef gebruikt Node.js' onderhouden `node:crypto` met AES-256-GCM, cryptografische randomness en authenticated associated data. De precieze sleutel-/noncecyclus staat bij de implementatie; random nonces mogen niet zonder grens als onbeperkt herbruikbaar worden gepresenteerd. Geen hardcoded productiesleutels of handgemaakte cipher/hashconstructies.

TLS beschermt het transport; clientencryptie beschermt documentinhoud ook tegen de NAS-host. Deze functies vervangen elkaar niet. Sleuteloverdracht naar een tweede eigen apparaat, bruikbare offline recovery en bescherming tegen serverrollback vereisen afzonderlijke integratie. Een versleuteld oud object kan nog steeds een geldige cryptografische tag hebben; de client moet geauthenticeerde versiegeschiedenis bijhouden voordat hij versheidsclaims kan doen.

## 6. Nog noodzakelijke native werkzaamheden

- Beperkte netwerkdienst met gecontroleerde socket-/bestemmingscapabilities, parsergrenzen en tests op gecompromitteerde apps.
- TLS-/clientauthenticatie via onderhouden implementatie, sleutelpinning, keyrotatie, geen 0-RTT-mutaties.
- Werkelijke NAS-rol in Nieuw OS, beheeropslag op een herstelbaar bestandssysteem en multi-userquota.
- Native clientkey-opslag en herstel zonder key escrow; veilige koppeling van extra apparaten.
- Directe bereikbaarheid, IPv4/IPv6, DNS-rebinding, NAT/CGNAT, netwerkovergangen en expliciet gekozen eigen netwerkroute. 'Plug & play overal' is niet bewezen.
- Persistente revocation en concurrency bij meerdere processen/devices; quota en disk-full-reserves; crash/fault-injection op de uiteindelijke opslaglaag.
- Fuzzing, sleutel-/updateketenreview en afgebakende onafhankelijke menselijke beveiligingsbeoordeling vóór gebruik voor gevoelige data.
- Taak-, toetsenbord- en schermlezerproeven van dezelfde onboarding op client en NAS.

De eerstvolgende kernelmijlpaal blijft de lopende M6a-eindverificatie. N01 wordt daarop aangesloten zodra opslag en netwerkcapabilities dat toelaten.

## Bronnen en besluitstatus

- OpenJS Foundation. (2026). *Crypto: Node.js v24.22.1 documentation*. https://nodejs.org/docs/latest-v24.x/api/crypto.html
- Rescorla, E. (2018). *The Transport Layer Security (TLS) protocol version 1.3* (RFC 8446). IETF. https://www.rfc-editor.org/rfc/rfc8446.html
- Rose, S., Borchert, O., Mitchell, S., & Connelly, S. (2020). *Zero trust architecture* (NIST SP 800-207). NIST. https://doi.org/10.6028/NIST.SP.800-207

Primaire bronnen gecontroleerd 27 oktober 2026. De functionele rolkeuze en het eenzijdige hostverwijderrecht volgen uit mijn expliciete opdracht. Deze bronnen keuren de implementatie niet goed.
