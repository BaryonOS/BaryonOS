# KEY1 SDK-client

De zelfstandige crate `sdk/keyring-client` levert een `no_std`, allocatorvrije
client voor de bestaande geïsoleerde sleuteldienst. Dit is het herbruikbare
appkanaal voor sleutelcreatie, intrekking en het versleutelen/ontsleutelen van
kleine records. Sleutelmateriaal verlaat de sleuteldienst niet via deze API.
De systeemapp gebruikt deze nieuwe client nog niet; native uitvoering van de
client en de volledige versleutelde opslagketen blijven open acceptatiepoorten.

De implementatie bouwt voort op
[B01](B01-ENTROPY-AND-KEYS.md), [B02](B02-SECURE-RECORDS.md),
[het opslagcontract](ENCRYPTED-OBJECT-STORAGE.md), `services/keyring/src/wire.rs`
en de bestaande SDK-runtime. Er zijn geen kernel-, grant-, dienst- of
workspacewijzigingen. De eigen `[workspace]` en lockfile laten de integrator de
gedeelde SDK later bewust aansluiten.

## Invariant en eigenaarschap

Een `Session<T>` krijgt een reeds verleend, **uitsluitend aan deze sessie
toegewezen** zend-/ontvangstpaar, de verwachte dienst-PID, `expected_domain` en
een begrensd pollbudget. De caller mag hetzelfde paar niet delen, buiten de
sessie gebruiken of een volgende sessie op dezelfde oude handles openen.
`close()` en beëindiging na protocolverlies sluiten beide eigen handles; er
zijn geen verborgen syscalls in `Drop`. De caller moet `close()` uitvoeren
voordat hij een ongebruikte sessie laat vallen. Een mislukte constructor neemt
de handles niet over. Closefouten worden teruggegeven; er is geen claim dat een
geweigerde kernel-close het object heeft vrijgegeven.

`expected_domain` controleert alleen de lokale context en antwoordtellingen.
Het is geen bewijs van rechten. De server bepaalt het echte domein via zijn
door de kernel geautoriseerde ingress. Bij recordopcodes 8/9 staat **geen
domeinveld op de wire**; de bestaande serverdecoder vult dat domein in. Ook
object, revisie en `RecordKind` zijn gevraagde context, geen toestemming. Een
native broker moet het object en de handeling autoriseren voordat hij een
geschikt kanaal verleent. De client maakt die ontbrekende productbroker niet
overbodig.

De client exposeert uitsluitend de bestaande opcodes 1–5 en 8–9. Er is geen
API voor recovery-/snapshotexport of restore (6/7), geen nieuwe grant en geen
zelfbedacht cryptografisch algoritme. Deze types vormen geen servergrens:
kwaadwillende apps kunnen zelf wirebytes maken. De bestaande brede
ontwikkel-endpoints moeten door de native broker/server worden begrensd voordat
derdenapps toegang krijgen. Toegang tot verschillende recordsoorten moet daar
eveneens worden afgedwongen.

## Interface

`begin(Operation)` kopieert en valideert het verzoek; de geretourneerde `u64`
is uitsluitend een correlatie-ID. `poll()` doet hoogstens één IPC-poging en
één yield, en levert `Poll::Pending` of een volledig gevalideerde `Reply`.
`exchange(Operation)` gebruikt hetzelfde begrensde pad synchroon.

| Bewerking | Getypeerd antwoord | Grenzen |
| --- | --- | --- |
| `Status` | `ServiceStatus` | Canonieke state/reason, beide keytellers ≤16, entropy en tellers dalen niet binnen deze sessie. |
| `Generate` | `Generated(KeyIndex)` | Index 0–15; geen hergebruik van een al waargenomen index binnen deze dienstlevensduur. |
| `Revoke(key)` | `Revoked` | Lege succesvolle body; verloren antwoord is geen bevestigde intrekking. |
| `Seal` / `Unseal` | `Sealed` / `Plaintext` | Bestaande legacy context; plaintext 0–128 bytes, sealed exact plaintextlengte +40. |
| `SealRecord` / `UnsealRecord` | `Sealed` / `Plaintext` | B02-context: volume, domein, object, revisie, chunk, lengte, soort en schema; exacte lengtes. |

Voor namen gebruikt de caller `RecordKind::Name`; voor inhoud
`RecordKind::Content`. De sleuteldienst authenticeert deze verschillende
contexten. Een naamrecord kan daardoor niet als inhoud worden ontsleuteld.
De 128-byte grens is de bestaande KEY1-recordgrens, geen productlimiet voor
documenten. De geïntegreerde opslagdienst moet grotere objecten in records
verdelen en de verwachte context uit geauthenticeerde metadata verkrijgen.
Dit levert op zichzelf geen bescherming tegen replay van een volledig oud
volume; een onafhankelijke actuele revisieanker en duurzaam sleutelherstel
blijven onderdeel van het opslagcontract.

Native gebruik na een door de broker verleende exclusieve verbinding:

```rust,ignore
use nieuw_os_keyring_client::{NativeTransport, Operation, Reply, Session};

let mut keys = Session::new(
    NativeTransport, key_service_pid, expected_domain,
    granted_send, granted_receive, 4096,
)?;
let key = match keys.exchange(Operation::Generate)? {
    Reply::Generated(key) => key,
    _ => unreachable!("typed operation/reply contract"),
};
// Gebruik key uitsluitend via deze geautoriseerde dienstverbinding.
keys.close()?;
```

`KeyIndex` bevat een publieke domeinlokale index, geen ruwe sleutel of
zelfstandig toegangsrecht. Een nieuwe dienstlevensduur vereist een nieuwe
brokerverbinding en sessie; er is geen automatische herverbinding.

## Fouten, verlies en annulering

De parser eist dienst-PID, KEY1-tag, precies 256 bytes en nul meegestuurde
capabilities. Daarna controleert hij opcode, index, correlatie-ID,
lengte, nulpadding, bekende status en de exacte succesvolle antwoordvorm.
Een canonicale dienstweigering wordt `Error::Service(Status)` en kan geen
plaintext of ACK opleveren. Onbekende status, vervalste afzender, extra grant,
verkeerde ID of misvormde body beëindigt de verbinding. Onverwachte ontvangen
grants worden afzonderlijk gesloten; closefouten blijven zichtbaar.

Er is één lopende bewerking. Een tweede `begin` geeft `Busy`. IDs lopen
monotoon vanaf 1; bij uitputting sluit de sessie vóór verzending, zonder wrap.
Het pollbudget blijft over alle polls behouden en begint afzonderlijk voor de
zend- en ontvangstfase. `EAGAIN` bij send betekent volgens de SDK-runtime dat
niets in de kernelqueue is geplaatst. Andere onduidelijke senduitkomsten
krijgen `Delivery::Unknown`.

- Vóór enqueue: annuleren wist de eigen aanvraagbuffer en geeft
  `NotPerformed`; het exclusieve kanaal blijft bruikbaar.
- Na enqueue of bij onduidelijke publicatie: annuleren, timeout of
  protocolverlies sluit de verbinding en geeft `MayHaveCompleted`. Er bestaat
  geen KEY1-abortopcode; een al gepubliceerde Generate/Revoke kan nog worden
  uitgevoerd. De client verzendt niet opnieuw en meldt geen succes zonder
  geldig antwoord.
- Een verloren antwoord op Generate of Revoke vereist herstel door de hogere
  laag. Een timeout opnieuw als dezelfde handeling starten kan een tweede
  sleutel maken of een andere toestand treffen en gebeurt hier nooit stil.

`Plaintext` heeft geen `Clone`/`Copy`, toont in `Debug` alleen de lengte en wist
zijn eigen buffer bij drop. Eigen verzoekbuffers worden met `zeroize` gewist
bij vernietiging en direct na een geslaagde send. Dit bewijst **niet** dat alle
copies zijn gewist: callerbuffers, tijdelijke moves, SDK `Outgoing`/`Received`
en kernel-/dienstbuffers hebben hun eigen levensduur. Sleutelinhoud wordt door
deze client nooit ontvangen.

## Afhankelijkheden en uitgevoerd bewijs

De lokale dependencies zijn de bestaande SDK-runtime en `keyring-core`; de
laatste levert uitsluitend hergebruikte protocol-/contexttypes aan de
productclient. De hosttests gebruiken daarnaast de echte `capability-core` en
sleuteldienstcore met herkenbare deterministische testentropy. Dat is geen
bewijs van native entropy of native clientuitvoering.

De directe externe dependency is de reeds aanwezige RustCrypto-crate
`zeroize = 1.9.1`, default features uit, Apache-2.0/MIT. De gepinde registry
checksum is `e13084392c5e4bc371903e2935a5eaeed24905a7511356b883835e18a78f6879`.
De standalone lockfile voegt tegenover basiscommit
`3879c3288f3b005d9df3ed5e1e40aa99c3791521` geen registryversie/checksum toe.
Er is geen nieuwe crypto-implementatie of nieuwe buildscriptcode. De herkomst
is dezelfde eerder gebruikte upstream RustCrypto/zeroize-release:
<https://crates.io/crates/zeroize/1.9.1>, repository
<https://github.com/RustCrypto/utils>. De lokale package verklaart Rust ≥1.85,
geen buildscript en Apache-2.0 OR MIT; deze build gebruikt Rust 1.90.0.

Op 30 oktober 2026 uitgevoerd met Rust 1.90.0, offline en een eigen targetmap:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-keyring-client-target \
  python3 -B tools/dev.py cargo +1.90.0 test \
  --manifest-path sdk/keyring-client/Cargo.toml --locked --offline
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-keyring-client-target \
  python3 -B tools/dev.py cargo +1.90.0 clippy \
  --manifest-path sdk/keyring-client/Cargo.toml --locked --offline --all-targets -- -D warnings
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-keyring-client-target \
  RUSTFLAGS='--cfg chacha20_force_soft --cfg chacha20_backend="soft" --cfg poly1305_backend="soft" --cfg sha2_backend="soft"' \
  python3 -B tools/dev.py cargo +1.90.0 build \
  --manifest-path sdk/keyring-client/Cargo.toml --locked --offline \
  --release --target x86_64-unknown-none --lib
```

Resultaat: 3 unittests en 12 hostintegratietests geslaagd, geen overgeslagen
tests; Clippy en freestanding librarybuild exit 0. Tests omvatten werkelijke
keycreatie/intrekking, 0/1/128 bytes, 7 foutieve recordcontext/authenticatiegevallen,
malformed replies, afgedwongen capability-intrekking, domein-injectie via echte
ingress, tellergrenzen/-regressie, verloren Generate/Revoke-replies en beide
annuleringsfasen. De beperkte checks staan met bron-/artifacthashes in
`sdk/keyring-client/verification/`.

Niet uitgevoerd: native appintegratie, een nieuwe QEMU-run van deze client,
langdurige opslag/herstart, fysieke hardware, volledige objectbroker en
persistente sleutel-/snapshot-/rollbackintegratie. Die blijven zichtbaar open
en worden door deze SDK-stap niet afgetekend. De hoofdagent beheert de
releaseacceptatie en voortgangsmeter.
