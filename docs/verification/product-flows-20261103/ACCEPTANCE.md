# Dagelijkse editor en terminale app-cleanup

Datum: 3 november 2026 (Europe/Amsterdam). Bronbasis:
`e37989f5b1893dfd1b4a610fe83a043751ae43f1`,
branch `simonvanieperen-animated-garbanzo`.
Geen andere branchcode of native bewijs overgenomen.

## Resultaat

De bestaande dagelijkse editorflow slaagt zonder nieuwe editorarchitectuur:
materiaal openen, via keyboard een meerbyte-UTF-8-teken selecteren en
vervangen, onopgeslagen sluiten annuleren, expliciet bewaren en teruggaan,
editor sluiten, nieuwe engine/service/controller met alleen bewaarde
RAM-sectoren opbouwen en identieke inhoud openen. De oude callerbuffers
worden nul gemaakt. Extra scenario gebruikt een 64-KiB UTF-8-draft:
I/O-fout, ingetrokken endpoint en vervangen revisie behouden het concept;
close weigert verlies, stale save wordt niet herhaald en alleen expliciet
verwerpen/openen haalt de vervangende inhoud op. Undo/redo is niet
geimplementeerd en wordt niet als ondersteund voorgesteld.

Tijdens de volgende echte app-teardownflow is gereproduceerd dat een
volledig gepresenteerde app na `stop_clean` nog Ready kon melden.
Productcode bewaart nu een terminale stopuitkomst. Ready/zichtbaarheid gaan
uit; `step`, interactieve step, semantische acties en presentation-recovery
worden geweigerd. Alle zes eigen endpoints worden eenmaal geprobeerd.
Een fout blijft bij herhaald stoppen dezelfde fout, geen schijnsucces.
Een geweigerde dirty/busy-stop maakt de app juist niet terminaal.
Nieuwe presentationgrants mogen de gestopte app niet doen herleven en
worden beide gepaard gesloten, met expliciete cleanupfout indien nodig.

De capability-engine controleert in de proef dat alle oude handles
daadwerkelijk onbruikbaar zijn; er volgen geen nieuwe input-/volume-/
paintrequests na stop. Een cleanupfout bij elk van de drie kanaaleigenaren
verhindert de cleanup van de andere twee niet. Engine-invarianten blijven
geldig. Dit is app-lokale teardown, GEEN bewijs van algemene dynamic launch,
window removal, OS-procesexit, crash cleanup of sessieafmelding.

## Uitgevoerd op deze productdelta

Cargo-omgeving: `PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH`,
`CC=/tmp/nieuw-os-c-toolchain/bin/cc`. Geen dependencies toegevoegd.

| Exact commando | Resultaat |
|---|---|
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --test chain stop_after_presented_frame_is_terminal` op oude productcode plus regressietest | Verwacht rood: exit 101, gesloten app meldde Ready; log `/tmp/nieuw-os-lifecycle-red.log`. Geen groen resultaat. |
| `cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml --test daily_editor` | Exit 0: 2 scenario's; eerste uitvoering 0,53 s inclusief compilatie. |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --test chain --test semantic_actions` | Exit 0: 41 + 2 tests, laatste uitvoering 1,65 s inclusief compilatie. |
| `cargo +1.90.0 clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings` | Exit 0. |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --all-targets -- -D warnings` | Exit 0. |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/materiaal/Cargo.toml -- --check` | Exit 0. |

Groene logs: `/tmp/nieuw-os-daily-editor.log`,
`/tmp/nieuw-os-lifecycle-green.log`. Compacte bronhashes naast dit document.
Geen native disk- of persoonlijke encryptieclaim door geordende RAM-sectoren.
Native build/ELF-parser volgen na het productcheckpoint. Nieuwe QEMU-runtime,
fysieke hardware, menselijke bediening, volledige CI/releasegate en
onafhankelijke securityaudit zijn niet uitgevoerd.

## Volgende productflows: concrete ontbrekende interfaces

De actuele claims en hervatvoorwaarden staan in WORK-QUEUE; dit is geen
tweede planning. Settings heeft volgens Veld geen actieve schrijver, maar
mist een bestaande instellingendienst, gebruikersnamespace en keyauthority
volgens PROFILES-AND-USERS stap 2. Geen plaintext-document als bypass.
Appselectie in de huidige Scene focust vooraf gestarte apps, geen launch.
K05 spawnt vaste ingebouwde images; app-package-core verifieert alleen.
APP-PACKAGES benoemt de ontbrekende AuthorizedImage-/kernelpreflightprovider.

Netwerk-TLS heeft uitsluitend random/time-adapters, geen native cryptoprovider;
CertificateTime kan door geen huidige bron veilig worden uitgegeven.
Maturity bevestigt dit op eigen bron `ce1c5e7f` (rapportage, niet onze test).
NAS-core/client verwachten reeds ciphertext en een geauthenticeerde Principal;
zij leveren geen pairing, crypto of live peertransport. De lokale AI-map
heeft een modelreadercontract maar geen modelcatalogus/installatie/inference.
Updates hebben hybride verificatie/slotinstallatie, maar geen consumable
health/promote/known-good-statusinterface; Maturity bevestigt dit eveneens.
Geen vervangende kernel-, crypto-, sync-, package- of maturityimplementatie.

Score: **39 -> 39/100**. Geen native criterium gesloten.
Rollback van de productdelta betreft alleen de terminale appstaat/guards en
bijbehorende tests/docs; kernel, volumeformaat, ABI en capabilitysemantiek
zijn ongewijzigd.
