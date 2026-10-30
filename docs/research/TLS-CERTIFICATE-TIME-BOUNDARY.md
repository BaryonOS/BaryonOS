# TLS-certificaattijd: minimale waarborg en bootstrapgrens

**Beslisadvies, geen architectuurbesluit of implementatiebewijs.**
TASK-RD-TLS-TIME, rol 17, 30 oktober 2026 (Europe/Amsterdam).
Repositorybasis `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`; actuele
port read-only onderzocht op
[`4a0ce99ff6b88adcd0e2c6be0571b30390d8d1c8`](https://github.com/simonvanieperen/nieuw-os/pull/81).
Die port bevat de CORE-tijdpoort
`9afabbd8266cd1230b7b89b6c70a56caa22f88ce`, als cherry-pick `00608c4`.
Geen werkboom van een andere sessie gelezen of gewijzigd.
De gepushte managerqueue/status is via Git gelezen op
`caa2c14da15c5ac82b5493b3b2407167163d00d9`; latere expliciete
managertoewijzing laat rol 18 de bestaande tijdweigering modelleren,
niet capability-revocatie. Dit onderzoek wijzigt dat model niet.

## Vooraf afgebakende vraag en onderzoekscontract

Manager `89f84098` koos deze ene vraag: welke tijdswaarborg is minimaal nodig
voor TLS-certificaatvalidatie, en kan de huidige RTC daarvoor dienen?
Afnemers: SEC2, NET6 en release4. Directe acceptatie: **FR-N03**; rubric
**N03/B04**. Geen onderzoekspunten: voorstel **34% -> 34%, netwerk 0/10**.
De hoofdagent beheert assessment, centrale status en gegenereerde meter.

- **Onzekerheid:** een leesbare UTC-seconde is beschikbaar, maar een
  geautoriseerde, verse en rollbackbestendige certificatentijdbron ontbreekt.
- **Hypothese:** standaard X.509-validatie vereist voldoende betrouwbare
  huidige kalendertijd; alleen een stabiele RTC of opgeslagen minimumwaarde
  voldoet niet aan het projectdreigingsmodel.
- **Serieus alternatief:** vooraf geauthenticeerde ruwe publieke sleutels,
  zonder X.509-datums, voor een expliciet gekoppelde peer.
- **Weerlegging:** een bestaande standaard en concrete implementatie tonen
  dezelfde hostname-, herkomst-, geldigheids-, intrekkings- en
  versheidswaarborgen zonder tijdbron of verplaatste vertrouwensaannames.
- **Verwachte beslissing:** behoud de huidige tijdweigering; specificeer
  eerst de voorwaarden voor een succesvolle bron en de TLS-consument.
- **Middelen/budget:** maximaal 90 minuten, vijf primaire bronnen en gerichte
  code-inspectie; geen nieuwe dependencies, runtime-, SDK-, ADR- of centrale
  writes. Enige schrijfbestand: deze notitie. Stop zodra de vraag beslisbaar
  is; geen algemene literatuurinventarisatie.
- **Bewijscriterium:** normatieve passages, exacte bronpin, concrete
  tegenvoorbeelden en een uitvoerbare vervolgtaak. Een hostexperiment alleen
  indien daarmee nog een beslissende onzekerheid kan worden opgelost.

## Besluitadvies

**De huidige RTC mag geen `CertificateTime` opleveren. Tijdloos TLS kan
authenticatie met een vooraf gebonden sleutel ondersteunen, maar is geen
gelijkwaardige vervanging voor de vereiste X.509-certificaatvalidatie.**
Behoud ADR-050 en `TLS-TIME-QUALITY-1`; geen heropening nodig.

Voor een toekomstige succesvolle bron is onder het projectdreigingsmodel
minimaal nodig: een expliciete bronautoriteit, betrouwbare binding van de
tijdsinhoud aan die bron, begrensde onzekerheid en leeftijd op het moment van
validatie, plus herstart-/rollbackbeleid. Bronondertekening alleen bewijst
niet dat de bron de juiste tijd opgeeft of dat het antwoord tijdig aankwam.
Dit zijn engineeringvoorwaarden uit de projectgrens, geen door RFC 5280
voorgeschreven concreet tijdprotocol.

Een bruikbaar ontwerpmodel is een gesloten interval `[L, U]` dat de werkelijke
UTC bij validatie bevat, onder expliciete aannames. Voor **dezelfde gekozen
certificatieketen** moet iedere te controleren certificaatperiode het hele
interval omvatten: `notBefore <= L <= U <= notAfter`. Dit is een conservatief
ontwerpvoorstel, geen nieuwe vastgestelde API of numeriek productbudget.
Het trust anchor is afzonderlijke vertrouwde invoer; behandel een eventueel
containercertificaat daarvan niet stilzwijgend als een gewoon padcertificaat.
Handtekeningen, namen, key usage, constraints en intrekkingsbeleid blijven
afzonderlijk nodig.

**Geen gekozen vertrouwde tijdbootstrap in deze ronde.** Strikte NTS is een
serieuze standaardkandidaat, maar heft zijn eigen initiële
certificaattijdprobleem niet op. Een uit-band aangeleverde beginwaarde kan
alleen helpen als autoriteit, onzekerheid, levering en veroudering expliciet
zijn geregeld. Een vaste buildtijd of ongecontroleerde handmatige invoer is
dat niet. Een productieprovider vraagt een volgend expliciet besluit; deze
notitie verleent geen nieuw netwerkrecht, leverancierafhankelijkheid of
toestemming om de klokgrens te passeren.

## Bronnen en concrete implementatie

| Bron | Relevant resultaat | Toepassingsgrens |
|---|---|---|
| RFC 5280, 4.1.2.5, 6.1.1, 6.1.3 en 6.3.3 | Huidige datum/tijd is padvalidatie-invoer; geldigheid is inclusief beide grenzen. Handtekening, geldigheid en niet-ingetrokken toestand zijn afzonderlijke controles. CRL-verwerking gebruikt ook actuele tijd. | Specificeert geen betrouwbare klokdienst voor Nieuw OS. |
| RFC 9525, 1.2 en 6.1.1 | Hostname/service-identiteit wordt naast de gehele padvalidatie gecontroleerd; referentie-identiteit wordt onafhankelijk van serveridentifiers opgebouwd. | Een juiste naam bewijst geen actuele geldigheid. |
| RFC 8915, 3, 8.5 en 8.6 | NTS-KE gebruikt TLS; de standaard benoemt expliciet de initiële tijdcirkel en mogelijke mitigaties. Authenticatie voorkomt geen asymmetrische netwerkvertraging. | De in 8.5 besproken versoepeling van datums is geen toegestaan Nieuw-OS-beleid. MAXDIST is geen door ons aangenomen productbudget. |
| RFC 7250, 1 en 6 | Raw public keys kunnen zonder PKIX-pad werken, mits een veilige uit-band identifier/keybinding bestaat en de status daarvan wordt gecontroleerd. | Geen algemene web-PKI-vervanging; lifecycle/rotatie/intrekking moeten elders worden geregeld. |
| rustls 0.23.46, `src/time_provider.rs` | `current_time() -> Option<UnixTime>` levert één tijdpunt; `None` betekent dat tijd niet beschikbaar is. De trait vereist niet eens monotoniciteit. | Deze trait bewaart geen bronkwaliteit, onzekerheidsinterval of antirollbackbewijs. Alleen de traitinspectie bewijst geen volledig handshake-foutpad. |

De vier RFC's zijn Standards Track, met publieke IETF-review en
IESG-goedkeuring; dat is geen wetenschappelijke peer-reviewed evaluatie van
Nieuw OS. Rustls is een controleerbare referentie-implementatie, geen hier
uitgevoerde audit of formeel bewijs. Geen wetenschappelijke publicatie nodig
om deze normatieve vraag te beslissen; daarvoor is geen zesde bron toegevoegd.

Op portbron `4a0ce99` zijn daadwerkelijk gelezen:

- `docs/contracts/TLS-TIME-QUALITY.md`, `sdk/runtime/src/tls_time.rs` en
  `sdk/runtime/tests/tls_time.rs`;
- `services/net-tls/src/{lib,time}.rs`, `tests/time.rs`, `Cargo.toml`,
  relevante `Cargo.lock`-pin en `README.md`.

`certificate_time(None)` geeft `MissingSource`; iedere huidige
`UnauthenticatedRtc` geeft een fout. `CertificateTime` heeft geen publieke
succesconstructor. NET `checked_now` bewaart bron-/kwaliteitsfouten;
`TimeProvider::current_time` vertaalt die naar `None`. Er is **geen** native
TLS-consument, succesvolle vertrouwde bron of handshake in deze leaf.
Een altijd-falende toegangspoort bewijst alleen huidige weigering, niet dat
een toekomstige succesvolle validatie correct zal zijn. De Rust-typegrens
vervangt geen procescapability tegen een kwaadaardig proces.

## Alternatieven en actief gezocht tegenbewijs

| Aanpak | Hostname/herkomst | Validity/intrekking | Rollback/versheid | Advies |
|---|---|---|---|---|
| RTC, buildtijd, hosttijd of opgeslagen floor direct naar `UnixTime` | Kan naast naam-/handtekeningcontrole bestaan. | Kan verlopen of nog niet geldige certificaten ten onrechte goedkeuren. | Floor begrenst hoogstens achteruitgang; bewijst geen actuele bovengrens. | Afwijzen als productiebron. |
| Geautoriseerde tijdbron met onzekerheidsinterval | Onafhankelijke normale naam-/padvalidatie behouden. | Hele interval en toepasselijk statusbeleid controleren. | Leeftijd, drift, delay, suspend, cold boot en ankering expliciet. | Ontwerprichting; bron nog open. |
| NTS met strikte initiële certificaatcontrole | NTS-KE vereist geverifieerde serverketen. | Behoudt controles, maar heeft begintijd nodig. | Requestbinding helpt tegen replay; delay blijft relevant. | Kandidaat na expliciete bootstrap, geen directe oplossing. |
| Raw public key / vooraf geverifieerde keybinding | Kan een specifieke peer authentiseren; hostnamebinding moet uit vertrouwde configuratie komen. | Heeft geen X.509-notBefore/notAfter en vervangt CA-status door eigen lifecycle. | Oude binding kan herleven zonder beschermd status-/rotatiepad; geen automatische versheid. | Alleen apart, expliciet profiel; geen fallback voor FR-N03. |
| Eerste sleutel blind vertrouwen of datumchecks uitschakelen | Geen vooraf verzekerde juiste peer bij eerste contact. | Verlaagt de gevraagde validatie. | Aanvaller kan begin-/oude toestand bepalen. | Geen alternatief binnen het huidige contract. |

De tijdloze optie is dus niet technisch onzinnig: RFC 7250 ondersteunt haar.
Zij **verplaatst** vertrouwen naar enrollment en bindingstatus. Zij levert
niet dezelfde geldigheids- en intrekkingssemantiek voor gewone websites.
Pinning als extra controle bovenop een certificaatketen maakt de datums
evenmin vanzelf overbodig. Dit rechtvaardigt geen beperking van browser-
of NAS-productambitie.

Drie beslissende analytische tegenvoorbeelden, niet als uitgevoerde proeven
gepresenteerd:

1. Een aanvaller bezit de sleutel bij een inmiddels verlopen, vroeger geldig
   certificaat. Een teruggezette maar stabiele RTC valt in de oude periode.
   Juiste naam en geldige handtekening sluiten die fout niet uit.
2. Een authentiek opgeslagen laatste tijd `F` blijft na lang uitgeschakeld
   zijn onveranderd. Werkelijke tijd ligt na `notAfter`, terwijl
   `F < notAfter`. Zelfs een tegen rollback beschermd minimum heeft geen
   bovengrens en lost dit niet op.
3. Twee alternatieve ketens zijn geldig rond respectievelijk `L` en `U`,
   maar laten daartussen een gat. Twee onafhankelijke succesvolle
   rustls-validaties op de eindpunten kunnen verschillende ketens kiezen.
   Zij bewijzen dan geen geldigheid gedurende het hele interval.
   Ook een geldige middenwaarde bewijst geen volledige intervaldekking.

Een actuele tijdbron alleen levert geen actuele intrekkingsinformatie.
Het concrete CRL-/ander statusbeleid, beschikbaarheid en foutgedrag moeten
door SEC/NET apart worden vastgelegd; geen onterechte claim dat rustls'
standaardconfiguratie alle revocationvereisten automatisch afhandelt.

## Een uitvoerbare vervolgtaak

**Voorstel TASK-TLS-TIME-INTERVAL-1, eerst door manager/SEC vast te stellen.**
CORE bezit de tijdinterface; NET bezit de rustls-adapter; SEC bepaalt de
vertrouwens- en intrekkingsvoorwaarden. Eén schrijver per bestand.
Actuele contactpersonen: CORE `fbf60554-e98e-40bd-bd48-ee87a6498abe`,
NET `039dbef8-bc79-4480-b003-55b8eccba736` en
SEC `081d6f31-7de1-4713-b466-d15adf51b2ef`.

**Doel:** leg een reviewbaar contract en host-tegenproef vast voor betrouwbare
tijd met onzekerheid, vóór een productieprovider `CertificateTime` mag maken.
Beoogde bestanden: bestaand `docs/contracts/TLS-TIME-QUALITY.md` (CORE,
besluitroute via hoofdagent), `sdk/runtime/src/tls_time.rs` en directe tests
(CORE), `services/net-tls/src/time.rs` en directe tests (NET).
Geen kernel-/ABI-wijziging of nieuwe cryptoprimitief veronderstellen.

**Interface/invariant:** eventuele tijdsevidence bindt bronautoriteit,
UTC-interval, boot-/provideridentiteit en monotone opname-/vervaltijd.
Onbevoegde callers kunnen geen geaccepteerde evidence maken. Dezelfde gekozen
keten moet het hele toegestane interval dekken. Een scalar `UnixTime` alleen
is geen bewijs; ontwerp de koppeling met de gepinde verifier expliciet.
Er komt geen publieke productieconstructor voor testtijd.

**Voorafgaande acceptatie:** test-only synthetische tijd en certificaatfixtures
op de host; vooraf vaste normale geldigheid en negatieve gevallen voor
ontbrekende/onbevoegde bron, stale/replay/rollback, verkeerde boot, suspend,
overflow, niet-dekkend interval, verlopen tussenliggend certificaat, verkeerde
naam/root en bovenstaande disjuncte-ketentegenproef. Schrijf eerst op welke
aannames en onzekerheids-/versheidsbudgetten gelden; keur geen verzonnen
default goed. Alle huidige RTC-weigeringen blijven groen.

**Verwacht effect:** voorkomt een scalar-/ketenselectiefout bij providerintegratie
en maakt de exacte ontbrekende bronvoorwaarden implementeerbaar. Dit is nog
geen werkende tijdbootstrap. Pas met een afzonderlijk gekozen onderhouden
tijdprotocol, afgedwongen providerautoriteit en echte native positieve én
negatieve TLS-proeven kan FR-N03 verder worden geaccepteerd. Hostresultaten
geven geen productpunten.

## Bronbinding, uitvoering en resterende onzekerheid

Vijf primaire bronnen daadwerkelijk opgehaald; geen build scripts of externe
broncode uitgevoerd. Lokale volledige bronkopieën:
`/home/simon/.copilot/session-state/f9f7a4ffbccb46b78c7de3494496e2f3/files/tls-time-sources/`.
SHA-256 van de opgehaalde bytes:

| Bestand | SHA-256 |
|---|---|
| `rfc5280.txt` | `a2f2628c0a83b873fc4786abd921f9b2c02395954b655d190bf16b831633345d` |
| `rfc9525.txt` | `ed8116c2435376dffe37ec2ca688c3a8515e36820852b29e0b6cdfbe92abd2c5` |
| `rfc8915.txt` | `c92a3ce4d1984b4c13f1cdf74d6f76c42b0fd34b2c1b71f5dd4f7d1cc641f57d` |
| `rfc7250.txt` | `834f7695e5258237efd4638f573203b325dbe8bce66a5690f6ff730fb00538dd` |
| `rustls-0.23.46.crate` | `48e13bd8c0e9365c43cfa5c9e8f9ad49d3c8444926c9aac819e0e4dc503c8fdf` |

Het rustls-archief is bytegelijk aan de checksum in de onderzochte lockfile.
Onderzochte Git-blobs, eveneens SHA-256:
`sdk/runtime/src/tls_time.rs` =
`f135b4b848ba41510662ac25c69e7cc0b1e3629825f36eb697a0b98c82285f79`;
`services/net-tls/src/time.rs` =
`a6f0cc80ca8a1ec1b2c910c9cd38452c07afaea2902134a3181b6d1e52733f1e`;
`services/net-tls/tests/time.rs` =
`796ee4fcce0c85cef535baaca312d2104ce5ea264aee4fa0c262364fcaf9a809`.
Gerichte inspectie gebruikte `git show 4a0ce99:<pad>`, RFC-tekstselecties
en `tar -xOf ... rustls-0.23.46/src/time_provider.rs`. Een zoekpoging naar
`current_time` in `common_state.rs` gaf geen match (exit 1), geen
handshake-uitkomst; daarom wordt daarover geen implementatieclaim gedaan.

**Niet uitgevoerd:** host-/native TLS-handshake, tijdprotocol, modelcheck,
performance-, Windows-, fysieke of menselijke acceptatie. Er is geen
experiment nodig om deze normatieve grens te beslissen. Niet vastgesteld:
welke productiebron en bootstrapketen haalbaar zijn, concrete versheids-/
driftbudgetten, suspend-/snapshotgedrag, persistente ankertransacties,
revocationdistributie en de succesvolle volledige verifierintegratie.
Die open punten zijn geen ontheffing van FR-N03.

**Ronde/checkpoint:** de beslisvraag is met vijf bronnen beantwoord, binnen
het maximum van 90 minuten; geen aanvullend experiment gestart.
`git diff --check` en `python3 tools/progress.py --check` zijn uitgevoerd,
beide exit 0; de laatste bevestigt uitsluitend consistente 34/100.
Geen centrale generatorwrites wegens exclusief managereigenaarschap.
Hervatten: lees deze notitie en laat SEC/CORE/NET de ene voorgestelde taak
accepteren voordat bronkwaliteit of een succesvolle tijdconstructor verandert.
Geen lopend achtergrondwerk, gewijzigde productcode of nieuw releaseartifact.

## Bronnen (APA 7)

Cooper, D., Santesson, S., Farrell, S., Boeyen, S., Housley, R., & Polk, W.
(2008). *Internet X.509 public key infrastructure certificate and certificate
revocation list (CRL) profile* (RFC 5280). RFC Editor.
https://doi.org/10.17487/RFC5280

Franke, D., Sibold, D., Teichel, K., Dansarie, M., & Sundblad, R. (2020).
*Network time security for the network time protocol* (RFC 8915). RFC Editor.
https://doi.org/10.17487/RFC8915

Rustls contributors. (2026). *rustls* (Version 0.23.46) [Computer software].
https://docs.rs/crate/rustls/0.23.46/source/src/time_provider.rs

Saint-Andre, P., & Salz, R. (2023). *Service identity in TLS* (RFC 9525).
RFC Editor. https://doi.org/10.17487/RFC9525

Wouters, P., Tschofenig, H., Gilmore, J., Weiler, S., & Kivinen, T. (2014).
*Using raw public keys in transport layer security (TLS) and datagram
transport layer security (DTLS)* (RFC 7250). RFC Editor.
https://doi.org/10.17487/RFC7250

Alle vijf geraadpleegd op 30 oktober 2026. De RFC-versies hierboven zijn
gepinneerde specificaties; dit onderzoek claimt geen uitputtende
errata-/opvolgersanalyse of upstream securityaudit.
