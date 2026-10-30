# Native crypto-, identiteit-, tijd- en entropygrens

Datum: 30 oktober 2026. **TASK-SEC-CRYPTO-AUTHORITY: afnemerscontract en
broninventaris; geen nieuwe runtime-interface of integrale acceptatie.**
Repositorybasis: `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`.
Dit document verleent geen capability en introduceert geen wire, signer,
calendarbron, cryptoprovider, KDF of NAS-herstelprotocol.

## Bronbinding en eigenaarschap

| Onderdeel | Gecontroleerde bron | Eigenaar / bewijsgrens |
|---|---|---|
| Key Service, RANDOM, records, persistentie en SDK-Drop | `7f674719d5aa90523c15a5a2b0472e15a923fb5f` (#72) | SEC; bestaande core en afzonderlijke native proeven, geen volledige NAS-keten |
| TLS-calendarpoort | `9afabbd8266cd1230b7b89b6c70a56caa22f88ce` | CORE; alle huidige bronnen worden geweigerd |
| TLS-portadapters | `4a0ce99ff6b88adcd0e2c6be0571b30390d8d1c8` | NET; adapter/crosscompile, geen werkende native TLS-provider of positieve handshake |
| Releasekeypolicy | `d7b7fccec6624da42c38bf6850335d9b56ef9417` (#74) | SEC / lifecycle; eigen releaseversiedomein, geen TLS-root |

Dit zijn afzonderlijke bronversies, **niet samen een geïntegreerde release**.
De aanwezige `records.rs` en `persistent.rs` zijn op de basis en #72 identiek.
SDK-Drop en de andere branchinterfaces mogen niet door deze verwijzingen als
reeds geïntegreerd in `ca45a7d` worden beschouwd.

NET bezit TLS/transport; CORE bezit de tijd-API; NAS bezit onboarding,
wachtrij en opslagintegratie. SEC bezit gedeelde crypto- en sleutelgrenzen.
SDK-, bootstrap-, loader-, kernel- en manifestwijzigingen vragen vooraf
afstemming met hun eigenaar en manager. Centrale beoordeling en punten blijven
manager-owned. Dit document tekent B01–B06 of gekoppelde releasecriteria niet af.

## Autoriteitsdomeinen blijven gescheiden

RANDOM-toegang geeft geen sleutel-, decryptie-, herstel-, signer- of netwerkrecht.
Een TLS-verbinding geeft geen NAS-objectrecht. Een geldig certificaat geeft geen
bootstraprecht. Een appinstallatie of geldige pakket-/releasehandtekening geeft
geen algemeen bestands-, netwerk-, NAS- of beheerrecht.

Releaseankers, pakketuitgevers, TLS-identiteiten en symmetrische
gebruikerssleutels zijn afzonderlijke autoriteitsdomeinen. B03 `TrustPolicy`
beoordeelt releaseversies; die versies zijn geen kalender of PKI-truststore.
Een proces-PID of zelf opgegeven identityveld is geen duurzame device-identiteit.
Er bestaat hier nog geen vastgesteld multi-device-token/restorecontract.

Een geautoriseerde normale herhaling binnen bestaande rechten hoeft niet
opnieuw om toestemming te vragen. Een nieuwe identiteit, vertrouwensgrens of
bevoegdheid vraagt wel een expliciete beslissing. Geen impliciete regrant
na intrekking, herstel of een onzekere uitkomst.

## Entropy: uitsluitend geautoriseerde RANDOM-lease

De bestaande publieke client is
`nieuw_os_keyring_random_client::Session<T>`, native met `NativeTransport`
(`sdk/keyring-random-client/src/lib.rs`). Bootstrap moet een exclusief
endpointpaar, de juiste service-PID en het caller/principalslot aan het bedoelde
proces binden. De huidige proefbinding autoriseert niet automatisch een
toekomstig TLS-proces. De lease is RANDOM-only, geen algemene Key Service-ingress.

`Session::begin(count)` publiceert niet; `poll()` doet begrensd werk.
Na enqueue alleen receive, geen automatische resend. Alleen bewezen
niet-gepubliceerde EAGAIN mag dezelfde aanvraag opnieuw aanbieden.
Annulering/onzekerheid retireert de lease; geen verborgen regrant.
`Unknown` is geen succes en geen toestemming voor een retry.
De client controleert afzender, tag, caps en de canonieke antwoordvelden;
capabilitycleanup gaat in #72 vóór normale zeroiserende `Received`-Drop.
De exacte request-/quotum-/readinessregels blijven in
[`KEYRING-RANDOM.md`](../implementation/KEYRING-RANDOM.md).

NET `KeyServiceRandom<T,C>` (`services/net-tls/src/random.rs`, NET-bron boven)
gebruikt die publieke Session-API, een monotone deadline en `try_lock`, niet
rechtstreeks `EntropySource`. De adapter levert volledige output of bij fout
een volledig gezeroiseerde doelbuffer; dat laatste is geen succesvolle
entropyrespons. Hij gebruikt geen host-RNG, seedcache of fallback.
Lege uitvoer vergt geen entropy; ongeldige lengte/Busy geven geen nieuwe grant.
Busy annuleert niet de lease van de actieve andere aanroep.
Een eigen begonnen mislukte aanvraag wordt geannuleerd en de lease retired,
met cleanupfout afzonderlijk. De rustls-trait reduceert dit naar
`GetRandomFailed`; gebruik `try_fill` waar foutdiagnostiek nodig is.
Log geen entropy, sleutelbytes, plaintext of providercredentials.

Deze adapter bewijst niet dat een toekomstige provider, KEM of signer dezelfde
geautoriseerde bron gebruikt. Iedere RNG-consumer in die backend moet apart
worden gebonden. Geen firmware-, hosttijd- of fixtureseed als stille vervanger.

## Tijd: deadline is geen certificaatkalender

CORE `sdk/runtime/src/tls_time.rs`:
`certificate_time(Option<WallClockReading>) -> Result<CertificateTime, TimeUnavailable>`.
`None` geeft `MissingSource`; `UnauthenticatedRtc` wordt geweigerd.
`CertificateTime` heeft geen publieke succesconstructor, parser, `Default`
of numerieke conversie. De huidige kalenderpoort heeft dus geen positief pad.

NET `checked_time` en `CertificateClock::checked_now`
(`services/net-tls/src/time.rs`) behouden die weigering; de rustls
`TimeProvider` levert bij ontbreken/fout `None`, niet een verzonnen
Unix-timestamp. Een caller moet de fout zichtbaar laten eindigen en mag geen
datechecks uitschakelen om een handshake te laten slagen.

PIT-ticks kunnen deadlines begrenzen, maar zijn geen kalenderautoriteit.
RTC, buildtimestamp, hostklok en ontvangen netwerktijd worden niet stil
gepromoveerd. Een toekomstige bron vereist apart beoordeelde herkomst,
onzekerheidsmarge, geldigheid, uitval- en rollbacksemantiek voordat zij een
`CertificateTime` mag uitgeven. Hier wordt geen vertrouwd interval ingevoerd.
Een beschermde ondergrens alleen geeft geen actuele bovengrens; een
ondertekende tijdwaarde bewijst op zichzelf geen tijdige ontvangst.
Een NTS-client heeft nog steeds een bootstrapvertrouwensvraag bij de
certificaattijd van zijn TLS-verbinding, zie RFC 8915 §8.5–8.6.

Raw public keys (RFC 7250) zijn geen stil alternatief voor de gevraagde
PKIX-certificaatsemantiek: identiteit, rotatie en intrekking vragen dan een
afzonderlijk beschermde binding. Er is hier geen raw-key-provider ingevoerd.

## TLS-identiteit en cryptoprovider

Een normale outbound TLS-client die uitsluitend de server authenticeert,
heeft niet automatisch een lokale signer nodig. Lokale certificaatauthenticatie
(zoals mTLS of een NAS-serveridentiteit) vraagt wel een geautoriseerde signer
voor de bijbehorende identiteit. Er is nog geen concrete native signer-API,
productiecredentialuitgifte of signergrant voor die rollen vastgesteld.

Een toekomstige signer moet proces, identiteit, gebruiksdoel, algoritme en
actuele grant controleren; geen sleutelbytes exporteren en geen algemene
cross-protocol-signingservice worden. Exacte ABI, sleutelopslag en binding
van het TLS-transcript blijven een afzonderlijk te beoordelen implementatie.

De huidige rustls-port heeft geen bewezen onderhouden native provider voor
het vereiste hybride profiel. Alleen rustls-core crosscompileren bewijst
geen KEM, certificaatvalidatie, handshake of transport. Geen klassieke
fallback, hostprovider of std/AVX2-aanname stil toevoegen.
Hybride sleuteluitwisseling is bovendien niet automatisch
postkwantumauthenticatie van een klassiek certificaat.

## Gedeelde beveiligde objectlaag

`services/keyring/src/records.rs` bevat `Service::seal_record` en
`unseal_record` met `RecordContext`: volume, domein, object, revisie, chunk,
lengte, recordsoort en schema. Deze context wordt geauthenticeerd.
Dit is een core-API met interne `EntropySource`, geen NAS-appgrant.

De vertrouwde adapter haalt domein, verwachte revisie en toegestane actie uit
geautoriseerde toestand. Contextvelden van caller, provider of ciphertext
zijn geen bewijs van objecttoegang. Naam-/metadata-/manifest-/indexrecords
vereisen hun eigen toegestane actie; dezelfde lengte maakt ze niet uitwisselbaar.
Een nieuwe revisie vraagt opnieuw geautoriseerd sealen onder nieuwe context;
ciphertext herlabelen is geen rebase. De bestaande formaten en API's staan in
[`B02-SECURE-RECORDS.md`](../implementation/B02-SECURE-RECORDS.md).

`Service::export_persistent`/`restore_persistent`
(`services/keyring/src/persistent.rs`) zijn geen wachtwoord-KDF,
devicekoppeling of NAS-tokenformaat. Het 32-byte geheim en 120-byte blob uit
de fixture mogen niet als productcredentials worden gebruikt.
`minimum_generation` werkt alleen met een onafhankelijk vertrouwde ondergrens.
Een naastliggend bestand of herstelde VM-snapshot levert dat bewijs niet.

Open blijven echte volume-identiteit, onafhankelijke lineage/generatieankering,
write-ahead bij allocatie/intrekking en herstel na onderbroken I/O. Geen
bevestigde handle/revocation/objectcommit onder nog niet duurzaam vastgelegde
sleuteltoestand; onzekere uitkomst blijft expliciet onzeker.
Key Service-intrekking, device-intrekking en NAS-adminblokkade zijn niet
dezelfde handeling. Intrekking kan eerder rechtmatig ontvangen gegevens
niet terughalen en verleent geen beheerderrecht op gebruikerssleutels.

Een expliciet gekozen externe provider ontvangt uitsluitend opaque ciphertext
en gescheiden opslagcredentials, geen inhoudssleutel of documentplaintext.
Geen stille relay/fallback. Dit verbergt niet automatisch omvang,
toegangspatroon of transportmetadata. AI-inferentie blijft lokaal zonder netwerkrecht.

## Fout- en herstelcontract voor afnemers

| Situatie | Vereist gedrag / bestaande grens |
|---|---|
| Caller, identiteit, grant of context niet geautoriseerd | Weiger vóór beschermde handeling; een payloadclaim verleent geen recht |
| Entropy niet gereed of gedegradeerd | Geen fallbackkeys/output; bestaande NotReady/Degraded en lease-afhandeling behouden |
| Busy | Geen spin of annulering van de actieve andere aanroep; geen nieuwe bevoegdheid |
| Clock-/deadline-/IPC-fout na begin | Eigen aanvraag sluiten/retiren; cleanupfout afzonderlijk, geen stille retry |
| Unknown na publicatie | Geen succes, ACK, automatische herhaling of regrant veinzen; expliciete reconcile vereist |
| Certificaattijd ontbreekt/onbetrouwbaar | TimeUnavailable/None, geen datecheck-bypass of hosttijd |
| AEAD/signature/context ongeldig | Geen vrijgave van plaintext of geslaagde authenticatie |
| Koud herstel zonder vertrouwde generatie/lineage | Geen antirollbackclaim; authentiek oud materiaal kan nog oud zijn |
| Nieuwe verbinding of herstelde instantie | Nieuwe autorisatie/binding waar vereist; oude handles/slots geven geen duurzame identiteit |

Dit is geen generiek retryprotocol en verandert bestaande statuscodes niet.
Afwijzing van crypto wist niet automatisch de caller-owned input; gebruik de
bestaande expliciete output- en geheugenhygiënecontracten per API.

## Acceptatie vóór positieve integratieclaims

De betrokken eigenaar levert bron-/artifactbinding en echte negatieve proeven:
verkeerde sender/caller/caps, RNG-degradatie, timeout/cancel/Unknown zonder retry,
onbetrouwbare/ontbrekende tijd, verkeerde identiteit/signature, ingetrokken
grants, contextswaps en oude snapshots. Native positieve TLS-, NAS- en
herstelclaims vergen hun eigen geïntegreerde proeven op de actuele bron.

SEC72 heeft 36 hosttests, een geoptimaliseerde echte envelope-Destructorproef,
20 locked-graafchecks en 12 eigen QEMU-gasten op bron
`df1ea1d10e50345bb410c985e5994c12f699350b`, vastgelegd in
[`B02-IPC-ZEROIZATION.md` op de #72-bron](https://github.com/simonvanieperen/nieuw-os/blob/7f674719d5aa90523c15a5a2b0472e15a923fb5f/docs/verification/B02-IPC-ZEROIZATION.md).
Dat bewijst geen TLS-provider, vertrouwde tijd, NAS-onboarding of alle LP01–31.
Menselijke review en eventuele formele invarianten blijven aparte gates.
Ruimere privacykeuzes schakelen de Security Floor niet uit.

## Primaire referenties

- IETF, RFC 8446, E. Rescorla, augustus 2018, §4.4.3:
  CertificateVerify voor certificaatauthenticatie en rolgebonden context;
  client alleen bij een niet-leeg Certificate-bericht.
  https://www.rfc-editor.org/rfc/rfc8446#section-4.4.3
- IETF, RFC 5280, D. Cooper e.a., mei 2008, §6.1.3:
  certificaatgeldigheid omvat de huidige tijd; dit document levert zelf geen
  geauthenticeerde klokbron.
  https://www.rfc-editor.org/rfc/rfc5280#section-6.1.3
- IETF, RFC 8915, D. Franke e.a., september 2020, §8.5–8.6:
  certificaattijd bij bootstrap en vertragingsaanvallen.
  https://www.rfc-editor.org/rfc/rfc8915#section-8.5
- IETF, RFC 7250, P. Wouters e.a., juni 2014:
  TLS-authenticatie met raw public keys; geen ingevoerd projectalternatief.
  https://www.rfc-editor.org/rfc/rfc7250
- Project: de brongebonden documenten hierboven,
  [`LOCAL-PRIVACY-CONTRACT.md`](../LOCAL-PRIVACY-CONTRACT.md), en
  `docs/contracts/TLS-TIME-QUALITY.md` op CORE `9afabbd`.
