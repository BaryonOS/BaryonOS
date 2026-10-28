# Versleutelde objectopslag en Security Floor

**Besluitdatum:** 28 oktober 2026.  
**Status:** bindende architectuur- en producteis; nog **niet native geïmplementeerd of geaccepteerd**.  
**Scope:** lokale opslag, persoonlijke NAS, optioneel gekozen externe opslagproviders, sleutelbeheer, synchronisatie en privacy-/securitygrenzen.

Dit document vult [LOCAL-PRIVACY-CONTRACT.md](../LOCAL-PRIVACY-CONTRACT.md),
[VERIFIABLE-SECURITY-AND-PQC.md](VERIFIABLE-SECURITY-AND-PQC.md) en
[PRODUCT-SPEC.md](PRODUCT-SPEC.md) aan. Het verandert de eigen Rust-microkernel
niet in een bestaande OS-kernel en introduceert geen zelfbedachte cryptografische
primitieven.

## 1. Productprincipe: privacy is configureerbaar, de Security Floor niet

Een gebruiker mag bewust minder privacy kiezen voor functionaliteit, bijvoorbeeld
door een game internettoegang te geven, een externe clouddienst te koppelen of
telemetrie van een derdenapp toe te staan. Zo'n keuze vergroot uitsluitend de
expliciet verleende gegevens- of servicebevoegdheid.

Zij schakelt **niet** automatisch uit:

- kernel-/procesisolatie;
- capability-controle en rechtenvermindering;
- scheiding tussen apps en diensten;
- geheugenbescherming en W^X/NX;
- bescherming van niet-gedeelde bestanden en secrets;
- sleutelisolatie;
- authenticiteit van pakketten en updates;
- DMA/IOMMU-beleid zodra dat op het ondersteunde hardwareprofiel is gerealiseerd;
- herstel- en rollbackgrenzen.

**Invariant:** toestemming vergroot functionaliteit, niet vertrouwen. Een app met
internetrecht krijgt daardoor geen document-, sleutel-, camera-, microfoon-,
NAS-, proces- of kernelrecht dat niet afzonderlijk is verleend.

Privacyuitleg en daadwerkelijke bevoegdheid zijn verschillende instellingen.
Educatieve privacyuitleg mag de gebruiker uitschakelen. Het uitschakelen daarvan
mag nooit impliciet rechten verruimen. Een nog niet verleende capability blijft
een afzonderlijke beslissing of vooraf gekozen regel.

## 2. Opslagmodellen

Nieuw OS ondersteunt op productniveau drie opslagklassen:

1. **Lokaal:** plaintext is alleen binnen de daarvoor bevoegde lokale OS-grens
   beschikbaar; opslagversleuteling en sleutelbeheer volgen de native
   beveiligingsarchitectuur.
2. **Persoonlijke/vertrouwde NAS:** inhoud en gevoelige metadata worden vóór
   verzending end-to-end versleuteld; de NAS-beheerder bezit geen inhoudssleutel.
3. **Onvertrouwde externe objectopslag:** een expliciet gekozen dienst zoals een
   algemene cloudopslagprovider mag uitsluitend opaque ciphertextobjecten bewaren.
   De provider is opslagtransport, niet de vertrouwensanker voor inhoud.

Een externe provider is **nooit** een stille fallback. De gebruiker kiest hem
expliciet en ziet dat account-, netwerk-, volume-, timing- en andere noodzakelijke
provider-metadata buiten de inhoudsencryptie kunnen vallen.

De provider mag voor het eigen account authenticatie vereisen. Nieuw OS mag dat
niet presenteren alsof de provider daarmee decryptierechten of OS-brede toegang
krijgt.

## 3. Dreigingsmodel voor externe opslag

Het ontwerp moet ervan uitgaan dat een externe provider of een aanvaller met
toegang tot de provideropslag:

- alle opgeslagen bytes kan lezen en kopiëren;
- bestanden/objecten kan verwijderen, herschikken, dupliceren of terugrollen;
- grootte, timing, account- en verbindingsmetadata kan observeren voor zover die
  niet afzonderlijk wordt verhuld;
- oude ciphertext onbeperkt kan bewaren;
- geen vertrouwde code of sleutel aan de Nieuw-OS-client mag leveren.

Onder dit model mag de provider zonder gebruikerssleutels geen geaccepteerde
plaintext, documentnaam, gevoelige manifestinhoud of inhoudssleutel verkrijgen.

Dit is geen claim dat de cryptografie absoluut "onkraakbaar" is. De garantie
geldt onder de aannames van de geselecteerde standaardalgoritmen, correcte
implementatie, voldoende entropy, correcte sleutelbescherming en een niet-
gecompromitteerd gebruikersendpoint.

Encryptie beschermt **vertrouwelijkheid en authenticiteit**, niet beschikbaarheid.
Een provider kan ciphertext verwijderen of toegang blokkeren. Daarom zijn
onafhankelijke lokale/back-upkopieën, herstel en versie-/rollbackdetectie
afzonderlijke eisen.

## 4. Eigen object-/containerformaat, geen eigen cryptografie

Nieuw OS mag een eigen, versieerbaar **Encrypted Object Format** definiëren,
maar gebruikt daarvoor alleen onderhouden, openbaar gespecificeerde en
beoordeelde cryptografische bouwstenen.

Geen eigen cipher, hash, KDF, KEM, handtekeningsalgoritme of ad-hoc hybride
protocol invoeren.

Conceptueel formaat:

```text
SecureObject vN
├── minimale publieke header
│   ├── format_version
│   ├── cipher_suite_id
│   ├── opaque_object_id
│   └── padded_size_class / chunking metadata
├── encrypted + authenticated manifest
│   ├── documentnaam
│   ├── type
│   ├── collectie/relaties
│   ├── gevoelige timestamps/metadata
│   ├── revisie
│   └── key/reference metadata
├── authenticated encrypted chunk 0
├── authenticated encrypted chunk 1
└── ...
```

De publieke header bevat alleen wat noodzakelijk is voor veilig parsen,
synchroniseren en migreren. Alles wat niet publiek hoeft te zijn hoort in het
versleutelde manifest.

### 4.1 Binding en integriteit

Authenticated encryption moet minimaal de volgende context cryptografisch
binden:

- formaat- en suiteversie;
- namespace/accountcontext;
- opaque object-ID;
- revisie/generatie;
- chunkindex en totale lengte;
- relevante schema-/typeversie.

Een chunk uit object A mag niet stil als geldige chunk van object B kunnen worden
geaccepteerd. Verkeerde sleutel, tag, versie, namespace, volgorde of lengte geeft
geen gedeeltelijk geaccepteerde plaintext.

De concrete AEAD-suite wordt pas voor native P04/B02 vastgezet na dependency-,
platform- en implementatiecontrole. AES-256-GCM-SIV (RFC 8452) is een kandidaat
vanwege nonce-misuse-resistance; de architectuur schrijft het algoritme nog niet
onherroepelijk voor.

## 5. Sleutelhiërarchie

Doelmodel:

- ieder object krijgt een willekeurige data-encryption key (DEK);
- gewone apps ontvangen geen exporteerbare master key;
- een afzonderlijke Key Service voert wrap/unwrap/decrypt-operaties uit binnen
  capabilities;
- root-/devicekeys gebruiken hardwarebescherming zoals TPM/secure hardware waar
  een ondersteund profiel dat betrouwbaar aanbiedt;
- per-objectkeys kunnen afzonderlijk worden geroteerd, gedeeld of ingetrokken
  zonder de hele opslagnamespace plaintext te maken.

Een externe provider bezit geen master key, objectkey of herstelkey.

Wachtwoordgebaseerd herstel gebruikt een onderhouden password KDF. Argon2id
(RFC 9106) is de voorkeurskandidaat; exacte parameters worden per ondersteund
hardwareprofiel gemeten en versieerbaar opgeslagen. Een wachtwoord is niet de
voorkeursopslag voor een rechtstreeks gebruikte master key.

Herstel kan bestaan uit een offline recovery secret en/of een al vertrouwd
apparaat. Verlies van alle geldige sleutels kan bewust onherstelbaar zijn;
er komt geen verborgen leveranciers- of provider-escrow.

## 6. Crypto-agility en postkwantumroute

Het objectformaat hardcodeert geen enkel algoritme zonder suite-ID en
migratiepad. Iedere suite heeft:

- expliciete versie;
- algoritmen en parameters;
- canonieke serialisatie;
- sleutel-/noncevereisten;
- migratie- en deprecatiebeleid.

Postkwantumalgoritmen zoals ML-KEM (FIPS 203) horen bij sleuteluitwisseling en
sharing waar het dreigingsmodel dat vereist; zij vervangen niet automatisch
symmetric authenticated encryption van bulkdata. Hybride transport blijft
afzonderlijk gespecificeerd in VERIFIABLE-SECURITY-AND-PQC.md.

Nieuwe suites worden niet stil geactiveerd. Migratie maakt nieuwe ciphertext
onder de nieuwe suite en behoudt een controleerbare herstelroute totdat de
oude versie veilig kan worden uitgefaseerd.

## 7. Metadata en privacyprofielen

Encryptie van inhoud verbergt niet automatisch:

- accountidentiteit;
- bron-IP en providerlogs;
- aantal objecten;
- objectgrootte;
- upload-/downloadtiming;
- verkeersvolume.

Nieuw OS moet dit eerlijk tonen en mag geen anonimiteitsclaim afleiden uit
client-side encryptie.

Voor gevoeliger gebruik kan een optioneel privacyprofiel extra mitigaties
onderzoeken, zoals:

- willekeurige opaque objectnamen;
- size-class padding;
- encrypted directory/manifeststructuren;
- batch-/vertraagde synchronisatie waar bruikbaar;
- scheiding tussen providercredentials en inhoudssleutels.

Padding en batching kosten opslag, bandbreedte en/of latency en zijn daarom
geen gratis universele standaard zonder meting.

## 8. Multi-provider en provider-onafhankelijkheid

De objectlaag staat boven de concrete provideradapter. Eenzelfde encrypted
object moet, waar de providerlimieten dat toelaten, kunnen worden opgeslagen op:

- eigen NAS;
- NAS van vertrouwde derden;
- generieke WebDAV/S3-achtige opslag;
- expliciet gekoppelde algemene cloudopslagproviders.

Provideradapters krijgen alleen de capabilities voor de toegewezen
ciphertextnamespace en netwerkbestemming. Zij hoeven geen documentinhoud,
lokale zoekindex of AI-context te zien.

Het ontwerp moet kunnen ondersteunen dat dezelfde versleutelde gegevens
redundant bij meerdere providers staan zonder aan ieder daarvan plaintext te
geven. Dit is een toekomstige beschikbaarheidsfunctie, geen reeds werkend
multi-cloudbewijs.

## 9. Rollback, verwijderen en herstel

Een aanvaller kan oude geldige ciphertext terugplaatsen. Daarom zijn AEAD-tags
alleen onvoldoende voor actuele-versiegaranties.

Het ontwerp moet een vertrouwde, monotone of anderszins rollback-bestendige
versieankering hebben voor de relevante threat model-scope. Test minimaal:

- replay van een oude objectversie;
- swap van chunks tussen objecten;
- truncatie;
- duplicatie;
- gedeeltelijke upload;
- providerdelete;
- providerrollback;
- lokale crash tijdens keyrotatie;
- verloren apparaat;
- verloren providercredentials.

"Verwijderen uit de cloud" betekent alleen dat de actuele provider geen
toegankelijke kopie meer aanbiedt. Het OS mag niet beloven dat een provider
nooit historische ciphertext heeft behouden. Cryptografisch wissen betekent
het vernietigen/intrekken van de relevante keys binnen de gegarandeerde scope;
dit vervangt geen providerretentieclaim.

## 10. App- en netwerkgrens

De syncdienst bezit geen algemene documentcapability voor alle apps. De
object-/Key Service en provideradapter blijven afzonderlijke principals.

Voorbeeld:

```text
Bewerker ──cap: object A lezen/schrijven──> Object Service
Object Service ──begrensde crypto-operatie──> Key Service
Sync Service ──cap: ciphertext A──> Provider Adapter
Provider Adapter ──cap: provider X + namespace Y──> netwerk
```

Een derdenapp die zowel plaintext mag lezen als vrij internet mag gebruiken kan
toegestane inhoud zelf versturen. Nieuw OS moet die gecombineerde bevoegdheid
begrijpelijk tonen, maar kan geen absolute non-exfiltration beloven nadat de
gebruiker beide rechten bewust heeft verleend.

## 11. Privacyuitleg is uitzetbaar, rechten niet impliciet

De systeem-UI onderscheidt:

1. **security/capability prompt:** noodzakelijke beslissing over nog niet
   verleende toegang;
2. **privacyuitleg:** extra begrijpelijke toelichting, voorbeelden en kritische
   vraag over gevolgen.

De gebruiker kan privacyuitleg globaal of per app uitschakelen. Dat verandert
geen bestaande grants en verleent geen nieuwe grants. Een reeds geldige grant
wordt niet bij iedere normale handeling opnieuw gevraagd.

Tijdens een game of andere focusmodus worden niet-urgente educatieve meldingen
niet midden in de activiteit opgedrongen. Een door de gebruiker geactiveerde
nieuwe gevoelige functie kan wel een compacte noodzakelijke grantvraag geven.

## 12. Acceptatie vóór een productclaim

Minimale toekomstige gates voor deze architectuur:

- **EO-01:** provideropslag bevat geen plaintext documentnaam, inhoud of
  inhoudssleutel in een gecontroleerde fixture.
- **EO-02:** gewijzigde ciphertext/tag/header/chunkvolgorde wordt geweigerd.
- **EO-03:** object-/namespace-/revisieswap wordt geweigerd.
- **EO-04:** providerdelete of outage geeft geen stille alternatieve provider of
  dataverliesclaim.
- **EO-05:** providercredentials alleen geven geen decryptierecht.
- **EO-06:** verlies van provideraccount tast een onafhankelijke lokale/back-up
  herstelroute niet stil aan.
- **EO-07:** app met alleen netwerkrecht kan geen beschermd document via de
  gecontroleerde OS-grens lezen.
- **EO-08:** app met alleen documentrecht kan geen provideradapter/netwerk
  gebruiken.
- **EO-09:** privacyuitleg uitzetten wijzigt geen capabilities.
- **EO-10:** keyrotatie en crashpad geven geen oude, nieuwe of gemengde plaintext
  als stil succes.
- **EO-11:** rollback/replay binnen de gekozen threat-modelscope wordt gedetecteerd.
- **EO-12:** multi-device toevoeging/intrekking verandert alleen de bedoelde
  sleutel-/objectscope.
- **EO-13:** een provideradapter kan geen tweede niet-geautoriseerde namespace
  of provider gebruiken.
- **EO-14:** metadata die nog zichtbaar blijft is gedocumenteerd en in
  gebruikerstaal terug te vinden.
- **EO-15:** onafhankelijke crypto-/integratiereview voor de daadwerkelijk
  gekozen suite en implementatie vóór brede veiligheidsclaim.

Hostprototypes, browsermocks en formaten op papier tellen niet als deze native
acceptatie.

## 13. Productervaring: beveiliging zonder ouderwetse UX

Security- en privacygrenzen mogen niet leiden tot een beheerconsole als
dagelijkse interface. Providerkeuze, sleutelbeheer, herstel en grants krijgen
een hoogwaardige, consistente Materiaal-ervaring met:

- normale menselijke termen als primaire laag;
- technische details op verzoek;
- één consistente interaction language;
- snelle hervatting binnen reeds verleende rechten;
- zichtbare locatie en herstelstatus zonder permanent dashboardlawaai;
- toegankelijke bediening en toetsenbordroute;
- geen imitatie van Windows/macOS/Office als productidentiteit.

De exacte native renderer en compositor zijn afzonderlijke implementatiebesluiten.
De producteis is een moderne, responsieve en eigen interface; geen claim dat dit
al gerealiseerd is.

## Primaire technische bronnen

- National Institute of Standards and Technology. (2024). *Module-Lattice-Based
  Key-Encapsulation Mechanism Standard (FIPS 203).* https://csrc.nist.gov/pubs/fips/203/final
- Gueron, S., Langley, A., & Lindell, Y. (2019). *AES-GCM-SIV: Nonce
  Misuse-Resistant Authenticated Encryption* (RFC 8452).
  https://www.rfc-editor.org/rfc/rfc8452
- Biryukov, A., Dinu, D., Khovratovich, D., & Josefsson, S. (2021). *Argon2
  Memory-Hard Function for Password Hashing and Proof-of-Work Applications*
  (RFC 9106). https://www.rfc-editor.org/rfc/rfc9106
- National Institute of Standards and Technology. (2026). *Considerations for
  Achieving Crypto Agility: Strategies and Practices.* 
  https://csrc.nist.gov/pubs/cswp/39/upd1/final

Deze bronnen specificeren bouwstenen en migratieprincipes; zij certificeren
Nieuw OS of deze integratie niet.
