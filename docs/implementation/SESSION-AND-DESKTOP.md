# Gebruikerskeuze, aanmelden en bureaublad

**Datum:** 2 november 2026. **Status:** onderzocht interactiecontract en
begrensde `no_std`-implementatie; echte accountauthenticatie en sessie-/sleutelrechten
zijn nog niet geïmplementeerd door deze wijziging. Dit sluit FR-S01, FR-S02 of
FR-U01–U05 niet af en verandert de vaste voortgangsscore niet.

## Onderzoek: herkenbare bediening

De officiële Windows-handleiding beschrijft: account kiezen, wachtwoord/PIN of
een ingestelde Windows Hello-methode gebruiken, bevestigen, desktop openen.
Vergrendelen behoudt de applicaties; afmelden beëindigt de sessie; wisselen kan
een andere sessie openen zonder de eerste te beëindigen. Bron: Microsoft,
*User Account Access in Windows*, geraadpleegd 2 november 2026,
https://support.microsoft.com/en-us/accounts-billing/security/user-account-access-in-windows.

Bij macOS kiest de gebruiker een account en authenticeert. Touch ID kan een
eerder met wachtwoord geopende sessie ontgrendelen of wisselen; bij opstarten
blijft het wachtwoord vereist. Apple noemt daarnaast periodieke herauthenticatie
en een wachtwoord na vijf mislukte vingerafdrukpogingen. Bron: Apple,
*Use Touch ID on Mac*, geraadpleegd 2 november 2026,
https://support.apple.com/en-nz/guide/mac-help/mchl16fbf90a/mac.

GNOME laat na het selecteren van een naam het wachtwoordformulier zien. Een
ondersteunde, vooraf geregistreerde scanner kan een vingerafdruk gebruiken;
het wachtwoord blijft beschikbaar. Het vergrendelscherm toont tijd/status en
geeft toegang tot authenticatie. Bronnen: GNOME, *Log in with a fingerprint*,
https://help.gnome.org/gnome-help/session-fingerprint.html en *The lock screen*,
https://help.gnome.org/gnome-help/shell-lockscreen.html,
geraadpleegd 2 november 2026. Dit betreft GNOME, niet iedere Linux-desktop.

Na aanmelden zijn dezelfde taken vindbaar, met verschillende vormen:

| Systeem | Starten, zoeken, wisselen en systeemstatus |
|---|---|
| Windows | Taakbalk met Start, zoeken, geopende/gepinde apps en status |
| macOS | Bureaublad, menubalk, Dock en Spotlight |
| GNOME | Bovenbalk, Activities met zoeken, dash met favoriete/geopende apps |

Bronnen, geraadpleegd 2 november 2026: Microsoft, *Customize the Taskbar in
Windows*, https://support.microsoft.com/en-us/windows/experience/personalization/customize-the-taskbar-in-windows;
Apple, *Get to know your desktop, menu bar, and Dock*,
https://support.apple.com/en-euro/guide/mac-help/mchlws12345m2/27/mac/27;
GNOME, *Visual overview of GNOME*,
https://help.gnome.org/gnome-help/shell-introduction.html.

**Projectkeuze:** gebruik herkenbare namen, accountkaarten, een duidelijk
wachtwoordveld en aanmeldknop, plus een taakbalk en zoekfunctie. De eigen
Materiaal-objecten, gereedschappen en beperkte rechten blijven het inhoudelijke
model. Een nieuwe kleur of launcher vervangt de overige producteisen niet.
Geen broncode van deze systemen is overgenomen.

## Authenticatie is een eigen dienstgrens

Schijf-/sleutelontgrendeling en sessietoegang zijn afzonderlijke beslissingen.
FileVault kan tijdens boot credentials vereisen; BitLocker kan vóór Windows
een PIN of startsleutel vragen. De UI mag de bediening samenbrengen zonder
sleutelvrijgave en accountautorisatie aan elkaar gelijk te stellen. Bronnen:
Apple, *Volume encryption with FileVault in macOS*,
https://support.apple.com/en-au/guide/security/sec4c6dc1b6e/web;
Microsoft, *BitLocker countermeasures*,
https://learn.microsoft.com/en-us/windows/security/operating-system-security/data-protection/bitlocker/countermeasures,
geraadpleegd 2 november 2026.

De bedoelde standaardaanduiding is **FIDO2**. USB-aanwezigheid of een aanraking
alleen is geen algemene accountverificatie. CTAP onderscheidt aanwezigheid
(UP) en gebruikersverificatie (UV, bijvoorbeeld een PIN of vingerafdruk).
De Microsoft-documentatie voor FIDO2-apparaataanmelding behandelt Entra joined
en hybrid joined-apparaten met expliciete inrichting; dit is geen bewijs voor
willekeurige lokale Windows-accounts. Bronnen: FIDO Alliance, *CTAP 2.3 Proposed
Standard*, 26 februari 2026,
https://fidoalliance.org/specs/fido-v2.3-ps-20260226/fido-client-to-authenticator-protocol-v2.3-ps-20260226.html;
Microsoft, *Enable FIDO2 security key sign-in to Windows*,
https://learn.microsoft.com/en-us/entra/identity/authentication/howto-authentication-passwordless-security-key-windows,
geraadpleegd 2 november 2026.

Een latere FIDO2-dienst moet onder meer verse challenges, credential-accountbinding,
handtekening, RP-binding en vereiste UP/UV valideren. Bron: W3C, *Web Authentication
Level 3 Recommendation*, 25 augustus 2026,
https://www.w3.org/TR/2026/REC-webauthn-3-20260825/.
Deze wijziging implementeert geen CTAP, biometrische driver, wachtwoord-KDF,
persistente accountdatabase of sleutelherstel. De standaardbroker weigert al
die methoden; er is geen gedeeld demowachtwoord en geen willekeurig geaccepteerd
wachtwoord.

## Afgebakend implementatiecontract

Doelplatform: de bestaande zelfstandige x86_64/UEFI-QEMU-basis; Rust `no_std`,
geen allocator, nieuwe afhankelijkheden, hardwaretoegang of netwerk in deze
module. Schrijfgebied: `ui/native/src/session.rs`, `ui/native/tests/session.rs`
en dit document. De integrator bezit de native visuele koppeling.

De hoofdroute is `AccountSelection → CredentialEntry → Authenticating →
SessionStarting → Desktop`. `Locked` verbergt iedere actieve identiteit;
`unlock()` toont opnieuw credentials en verleent geen toegang. Generatie- en
requestcounters lopen nooit rond: uitputting eindigt in `Unavailable`.

| Interface | Verantwoordelijkheid |
|---|---|
| `SessionFlow` | UI-toestand, account/methode, begrensde invoer en actuele correlatie |
| `AuthBroker::begin` | Vertrouwde adapter verstuurt verzoek naar authentieke dienst; wachtwoord wordt alleen geleend |
| `BrokerReply` | Antwoord van die adapter, gebonden aan request-ID, UI-generatie, account-ID en methode |
| `BrokerTicket` | Opaque servicehandle; geen `Debug`, `Clone` of `Copy` |
| `AuthBroker::activate` | Sessiedienst valideert ticket opnieuw en start alleen de toegestane sessie |
| `UnavailableBroker` | Veilige standaard zolang de echte diensten ontbreken |

Een toegekend ticket opent nog geen desktop. Pas succesvolle sessieactivatie
zet de gebruikersidentiteit op `Authenticated`. Een fout, timeout of weigering
laat geen actieve identiteit over. Account-/methodewisseling, annuleren,
vergrendelen en afmelden wissen wachtwoordinvoer en maken lopende antwoorden
ongeldig. Een oude reply mag de nieuwe aanvraag niet vervullen.

Wachtwoorden gebruiken maximaal 128 bytes; overloop geeft een fout. De UI kan
alleen de lengte opvragen. Invoer en ticketbytes worden bij verlaten van hun
levensduur met de bestaande gepinde `zeroize 1.9.1`-implementatie gewist.
De native UI gebruikt dezelfde no_std-configuratie zonder extra crate of
buildscript als de SDK. Dit beschermt de expliciete owned buffers tegen
wegoptimaliseren van het wissen; tijdelijke kopieën, registers, crashgeheugen
en data buiten de eigen buffers vallen daarbuiten. De latere secretdienst
vereist nog een gecontroleerde volledige levenscyclus.

De standaard `SessionFlow::new()` gebruikt lokale counters, geen willekeurige
sessie-identiteit. De echte broker moet daarom daarnaast de authentieke
IPC-peer, service-instance en sessiegeneratie binden; correlatie kan niet over
verschillende UI-processen of een herstart heen als authority dienen.
Publieke Rust-constructors controleren alleen vorm en correlatie, **niet de
issuer**. `BrokerReply::granted` mag uitsluitend na echte dienstverificatie door
de trusted adapter worden aangeroepen. Een gecompromitteerde UI kan zijn eigen
geheugen wijzigen; kernelcapabilities en sleutelvrijgave mogen nooit op het
UI-stateveld vertrouwen.

`VmGuest` is een expliciet gekozen wegwerp-VM-profiel zonder wachtwoord en
zonder gebruikerssleutels. `begin_guest` en `activate_guest` vormen een aparte
route en leveren uitsluitend `SessionIdentity::VmGuest`; dit is geen geslaagde
gebruikersauthenticatie. De native adapter mag deze route alleen aanbieden in
het zichtbare VM-previewprofiel en uitsluitend de bijbehorende synthetische
werkruimte gebruiken. Deze module maakt of mount geen opslag.

Werkelijke lock/revoke en input-/surfaceafscherming blijven opdrachten van de
sessie-/compositordienst. `SessionFlow::lock()` kan zelf geen kernelgrants
intrekken. Zoekresultaten blijven bij de toegestane lokale catalogus; zoeken
verleent geen extra bestands- of netwerkrechten.

Aanvullende leerbron: OSDev Wiki, *Security*, W002262, lokale snapshot volgens
de repositoryskill. De passage over systeemgrenzen en onbetrouwbare lengtes is
oriëntatie; primaire specificaties en projectcontracten zijn leidend. Er is
geen tutorialcode overgenomen.

## Acceptatie en bewijsgrens

Gerichte hosttests controleren: expliciete gastidentiteit, alle wachtwoorden
geweigerd door de standaardbroker, geen desktop vóór activatie, weigering en
timeout, mislukte activatie, annuleren/wisselen/lock met verlate replies,
volledige account-/methodebinding, retry met een nieuwe request-ID, invoergrens
en secret-lengteopruiming. Een servicefixture in de tests is geen echte
authenticatiedienst.

Commando voor de gerichte suite vanuit de repository:

```sh
source /workspace/.nieuw-os-env/activate.sh
CARGO_TARGET_DIR=/workspace/nieuw-os/build/session-flow cargo test --manifest-path ui/native/Cargo.toml --test session --offline
```

Uitgevoerd op de werkboom boven bron `e08fb0e48c1bb0781e98357f588c2f73b6ae8500`:
de gerichte suite slaagt met **10 tests, exit 0**; log
`/workspace/.nieuw-os-env/session-tests.log`. `rustfmt --edition 2021 --check`
voor de sessiemodule en de tests en `git diff --check` slagen eveneens, exit 0.
<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:8dff4dfe43779be46cf207ac94fd6ed5a384eab8b3a51682f81527b8c7d826f1 -->

Historisch werd de eerste strikte `cargo clippy --test session --offline -- -D warnings`
geblokkeerd door gelijktijdig gewijzigde integratiecode (exit 101; log
`/workspace/.nieuw-os-env/session-clippy.log`). Een latere integratiecontrole
op bron `344dc0a` rapporteert een geslaagde Clippy-gate voor alle targets met
`-D warnings`; zie het [verificatierapport](../verification/session-desktop-20261102/README.md).
Die historische uitvoering bewijst de huidige werkboom niet.

<!-- /coderabbit-ai-generated:v1 -->
Native UI-acceptatie, echte account-/sleutelpersistentie, FIDO2/biometrie,
kernelrevoke, fysieke hardware en menselijke beoordeling vallen buiten dit
hostbewijs en blijven open.
Rollback: verwijder de nieuwe sessiemodule/tests en de integratorverwijzing;
dit document bewaart het onderzoekscontract. Geen bestaande gebruikersdata
wordt gemigreerd of gewijzigd.

## Native koppeling door de integrator

`native-system-session-preview` is een expliciet opt-in VM-profiel. De kernel
zet uitsluitend startupwoord648=1; geen nieuwe capability of cryptografie.
De native app activeert de shell boven zijn bestaande synthetische appdomein.
De eerste vijfstappenproef op7648a8f slaagde: alle pixels in het appgebied
(16,48,768,536) gelijk aan de gebonden hostrenderer. De uitgebreidere eindproef
controleert ook persoonlijke testinvoer, weigering en wissen vóór expliciete
gastaanmelding. Bewijs en definitieve bron staan onder
[session-desktop-20261102](../verification/session-desktop-20261102/).

De brongebonden hostoracle is een vergelijking, geen zelfstandig native bewijs.
Alleen de echte QMP-invoer en QEMU-screendumps leveren uitvoering aan. De
zichtbare accountkaart 'Eigen gebruiker' is nog geen geregistreerd profiel;
een echte auth-/sessiedienst moet later enrollment, persistente verifier,
herstel, ratelimiting, capability-intrekking en compositor-/input-lock toevoegen.

## Startoppervlak — 3 november 2026

De VM-preview gebruikt nu een hiërarchisch startoppervlak met een herkenbare
werkruimte, expliciete navigatie en één primaire ingang naar Materiaal. De
Materiaalkaart leest de bestaande lijsttoestand van de echte app en toont
niet-geladen, lege, fout- of documenttoestand; zij maakt geen voorbeeldbestanden,
recente-activiteitstatus of verzonnen netwerk-/energiestatus. De compacte
lay-out houdt de kaart boven de navigatiebalk en vervangt een tekstloze
Start-knop door een benoemde actie.

Dit is een visuele en informatiewijziging binnen de bestaande expliciete
VM-gastpreview. Zij voegt geen accountauthenticatie, opslagrechten,
zoekresultaten, netwerkstatus of nieuwe sessieovergang toe. De lege en
gevulde kaarttoestand is als native-rendererhostregressie toegevoegd; lokale
uitvoering vereist een C-linker. Een hostcompile-/lintcontrole is geen
pixelproef; de native QEMU-/screenshotgate moet op de exacte commit slagen
voordat de nieuwe weergave als native uitgevoerd bewijs geldt. De score blijft
41/100 totdat een volledig rubriccriterium met passend bewijs wordt geaccepteerd.
