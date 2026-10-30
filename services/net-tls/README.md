# Native TLS-port: randvoorwaarden, nog geen TLS-dienst

Taak `TL-NET-TLS-PORT-1`, basis `94c8908` plus de ongewijzigde CORE-tijdpoort
`9afabbd` (cherry-pick `00608c4`). Schrijfgebied uitsluitend
`services/net-tls/**`; de CORE-bestanden zijn een bestaande afgestemde
afhankelijkheid, geen tweede implementatie.

Doel: de onderhouden rustls-core zonder `std` compileren en haar bestaande
random-/tijdinterfaces aansluiten op de bestaande SDK-grenzen. Geen zelfgeschreven
crypto, provider, recordprotocol, TLS-handshake, native ELF, manifestgrant,
certificaatroot, signer of NAS-principal in deze stap.

## Invarianten en interfaces

- `KeyServiceRandom<T, C>` implementeert rustls `SecureRandom` met een exclusief
  bezeten bestaande `keyring-random-client::Session<T>`. Native `T` is de bestaande
  `NativeTransport`; `NativeClock` gebruikt de echte monotone SDK-klok. De
  proces-/account-/generatiebinding moet al door vertrouwde bootstrap zijn
  aangebracht. Deze adapter verleent geen rechten.
- Rustls vereist `fill(&self)` en `Send + Sync`; een onderhouden `spin::Mutex`
  beschermt de mutable SDK-sessie. Alleen `try_lock`, geen spinlus of nieuwe
  `unsafe`. Busy/reentrantie faalt direct zonder de lopende operatie aan te raken.
- Een aanvraag levert maximaal de bestaande 128 bytes, volledig of niets.
  Geen chunking, cache, getrandom/RDRAND, herpublicatie of automatische regrant.
  Nul bytes aanvragen publiceert niets. Ongeldige lengte/busy wist de uitvoer
  zonder een andere lopende operatie te annuleren.
- Een absolute monotone deadline geldt voor de gehele aanvraag, ook na ontvangst
  van een compleet antwoord. Rollback, overflow, klokfout, SDK-pollbudget,
  protocolfout en dienstweigering zijn fouten. De adapter annuleert dan de
  lease; ook een `Unknown`-publicatie wordt niet herhaald. De uitvoer is vooraf
  zeroized, pas na alle controles gekopieerd. De SDK wist haar huidige
  ontvangen antwoordbuffer; dit is geen claim over historische compilerkopieen.
- `try_fill` behoudt de SDK-fout, delivery-state en cleanupfout. Rustls'
  voorgeschreven `fill` vertaalt dit naar `GetRandomFailed`, geen succesfallback.
  `Debug` bevat geen bytes, handles of sleutelinhoud.
- `CertificateClock` en `checked_time` gebruiken uitsluitend de CORE-poort.
  `checked_now` bewaart bronfouten afzonderlijk van `TimeUnavailable`.
  Rustls `TimeProvider` krijgt `None` bij onbeschikbaarheid, nooit RTC, monotone
  ticks, hosttijd of buildtijd als vervanging. Een toekomstige TLS-consument moet
  `checked_now` al voor certificaatvalidatie controleren. Iedere huidige
  kloklezing faalt; er is geen gefingeerde vertrouwde provider.

**Belangrijk:** rustls `SecureRandom` dekt niet automatisch de RNG van de
cryptobackend, KEM of signer. Een toekomstige provider moet ook die routes aan de
echte geautoriseerde Key-Service binden. Deze adapter alleen bewijst dat niet.

## Dependencycontrole voor toevoeging

Direct gepind: rustls `=0.23.46`, spin `=0.12.3`, bestaand zeroize `=1.9.1`;
transitieve versies/checksums staan in de zelfstandige `Cargo.lock`.

| Bibliotheek | Herkomst / licentie | Geselecteerd / buildscript |
|---|---|---|
| rustls 0.23.46 | rustls/rustls; Apache-2.0 OR ISC OR MIT, onderhouden uitgave oktober 2026 | Alleen `custom-provider`; geen `std`, TLS 1.2, logging, AWS-LC of ring. `build.rs` heeft zonder `read_buf` een lege main; bron vooraf gelezen. |
| spin 0.12.3 | codeberg.org/zesterer/spin; MIT, uitgave augustus 2026 | Alleen `mutex,spin_mutex`; `no_std`, geen dependencies of buildscript in deze selectie. |
| rustls-webpki 0.103.16 | rustls/webpki; ISC | `alloc`, geen cryptobackend/buildscript. Manifest van de daadwerkelijk opgeloste patch gecontroleerd, niet alleen dat van 0.103.14. |
| rustls-pki-types 1.15.1 | rustls/pki-types; MIT OR Apache-2.0 | `alloc`, geen buildscript. |
| once_cell 1.21.4 | matklad/once_cell; MIT OR Apache-2.0 | `alloc,race`, geen `std` of buildscript. |
| subtle 2.6.1 | dalek-cryptography/subtle; BSD-3-Clause | Zonder default-features, geen buildscript. |
| untrusted 0.9.0 | briansmith/untrusted; ISC | Geen `std` of buildscript. |
| zeroize 1.9.1 | RustCrypto/utils; Apache-2.0 OR MIT | Bestaande SDK-pin, `alloc` door rustls; geen buildscript. |

Primaire bronnen: crates.io-versiemetadata en de gepubliceerde Cargo-manifests
op `https://docs.rs/crate/<naam>/<versie>/source/Cargo.toml`.
Rustls-interfaces: `https://docs.rs/rustls/0.23.46/rustls/crypto/trait.SecureRandom.html`
en `https://docs.rs/rustls/0.23.46/rustls/time_provider/trait.TimeProvider.html`.
De registryarchieven zijn via Cargo opgehaald, niet door externe scripts
uit te voeren. Cargo kan ongekozen optionele/andere-platformpackages in de
lockfile opnemen/ophalen; alleen de expliciete featureboom en compileruitvoer
bewijzen wat in deze selectie daadwerkelijk wordt gebouwd.

Rustls' unbuffered API ondersteunt `no_std` maar gebruikt nog `alloc`.
Een echte native ELF vereist een begrensde private allocator en OOM-proef;
een librarybuild bewijst geen heap-, stack- of runtimegedrag.

Niet gekozen als heimelijke productfallback: AWS-LC (geen aangetoonde baremetal-
RNG-/platformport), rustls-rustcrypto 0.0.2-alpha (oude experimentele provider),
rustls-graviola 0.4.0 (forceert `std`/TLS 1.2; graviola vraagt onder meer
AVX/AVX2, niet het huidige VM-profiel). Geen CPU-profielwijziging gedaan.
Het afgesproken proefprofiel blijft TLS 1.3, X25519MLKEM768,
TLS_AES_256_GCM_SHA384 en vast ALPN, zonder klassieke fallback/sessiehergebruik/
0-RTT-mutaties. Geen geschikte native provider in deze oplevering.

## Acceptatie en bewijsgrens

Voor deze stap: gerichte hosttests, Clippy, formattering, `x86_64-unknown-none`
librarybuild en inspectie van de werkelijk gekozen dependencyfeatures.
Geplande commando's, nog niet afgetekend in dit broncheckpoint:

```sh
python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path services/net-tls/Cargo.toml
python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked --manifest-path services/net-tls/Cargo.toml --all-targets -- -D warnings
python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path services/net-tls/Cargo.toml -- --check
python3 -B tools/dev.py cargo +1.90.0 build --offline --locked --manifest-path services/net-tls/Cargo.toml --lib --release --target x86_64-unknown-none
python3 -B tools/dev.py cargo +1.90.0 tree --offline --locked --manifest-path services/net-tls/Cargo.toml --target x86_64-unknown-none -e features
```

Tests gebruiken synthetische SDK-berichten en modelklokken, geen echte entropy
of TLS-tegenpartij. Native TLS/Key-Service-aanvraag vanuit een TLS-proces,
certificaatvalidatie, handshake, NAS-peerbinding en native normal/cold: **niet
uitgevoerd en niet aanwezig**. Geen ondertekendienst of vertrouwde kalenderbron
geautoriseerd door SEC; REL bevestigt dat er geen TLS-proces/bootstrapgrant
bestaat. Die grenzen blijven open; FR-N03/N03 krijgt geen punten.
Voorstel aan centrale manager: netwerk **0/10 -> 0/10**, totaal **34 -> 34**.

Rollback: deze zelfstandige leaf niet integreren/verwijderen; de andere
netwerkdiensten hebben hierop geen dependency. Centrale status/scorebestanden
blijven manager-owned.
