# G01 — begrensde native app-runtime

Deze eigen `no_std`-crate bouwt op de bestaande Nieuw OS-ABI. Zij gebruikt geen
heap, libc, host-I/O of netwerk. De externe dependency `zeroize` is vastgezet
op versie 1.9.1 zonder standaardfeatures en wist de eigen IPC-envelopes bij
opruimen. De pure berichtlaag bevat geen `unsafe`; `native.rs` bevat de kleine
`int 0x80`-grens. De hostbuild biedt die native module niet aan, zodat een
hosttest niet per ongeluk een Linux-syscall met Nieuw OS-argumenten uitvoert.

De runtime is een bouwbare SDK-basis. Zij voegt geen appinstallatie,
compositor of opslagrecht toe. Sinds K05 (ADR-042) kan een proces mét een
`LaunchImage`-capability kinderen starten, observeren en beëindigen en kan elk
proces binnen het quotum eigen endpoints maken; zonder die capability blijft
een app afhankelijk van een geautoriseerde bootstrap. `ChildProcess`-handles
kunnen niet worden afgeleid, overgedragen of ingetrokken.

## API en autoriteit

`Handle::from_raw` controleert uitsluitend de bestaande generatie/slotvorm.
`SendEndpoint` en `ReceiveEndpoint` helpen vergissingen in aanroepen voorkomen.
Een nummer of Rust-type geeft nooit rechten: de kernel controleert bij iedere
aanroep proces, objecttype, generatie en bevoegdheid. `Rights` beperkt zich tot
de bestaande bits; `derive` en capoverdracht laten de kernel iedere poging tot
rechtenverruiming weigeren. Dit is dezelfde authoritygrens als vóór de SDK.

| Functie | Gedrag |
|---|---|
| `native::abi_info/check_abi` | Lees ABI en accepteer uitsluitend bestaande compatibele profielen 1.0 t/m 1.3 |
| `native::yield_now/exit` | Bestaand proces afstaan/beëindigen |
| `native::close/derive/revoke` | Bestaande capabilityoperaties; geen nieuwe objectaanmaak |
| `native::send` | Eén nonblocking send; `Ok` betekent alleen dat het bericht is gequeued |
| `native::receive` | Eén nonblocking receive naar een eigen stagingbuffer, daarna gewiste envelope-overdracht |
| `native::close_received_caps` | Expliciete cleanup van ontvangen caps; probeert alle handles en bewaart de oorspronkelijke berichtbytes |
| `native::endpoint_create` | K05 (ABI 1.3, profiel `native-supervisor`): eigen endpoint binnen het kernelquotum van 4 per proces |
| `native::spawn` | K05: start de image van een `LaunchImage` met één argumentwoord en maximaal twee verminderde capabilities (nooit RECV); levert de enige `ChildProcess`-handle |
| `native::kill/status` | K05: beëindig of observeer een eigen kind; `ProcessStatus` decodeert alleen canonieke statuswoorden |

`Outgoing::new` bouwt een volledig nulgevulde Message320 met maximaal 256
payloadbytes. `Outgoing::from_bytes` accepteert een bestaand protocolbericht
alleen met geldige lengtes, handles en canonieke padding. Hoogstens twee
capabilities kunnen worden toegevoegd; RECV-overdracht blijft geweigerd.

`Received` bewaart sender, tag, payload en alle oorspronkelijke envelopebytes.
De constructor controleert ook dat de lengte gelijk is aan het syscallresultaat.
Native IPC gebruikt één gewiste overdracht naar de envelope; de stagingbuffer
wordt bij succesvolle overdracht direct gewist en zowel `Received` als
`Outgoing` wissen hun bytes bij opruimen. Hun `Debug`-weergave bevat geen
payload. Dit is geheugenhygiëne, geen garantie dat compiler-/registerkopieën,
crashgeheugen of elders gekopieerde bytes worden gewist.
De afnemende dienst/app moet nog zijn verwachte peer, tag, request-ID, epoch,
domein, inhoud en toegestane capabilities controleren. Een geslaagde
`Received::from_bytes` bewijst op zichzelf geen kernelherkomst. `native::receive`
haalt de bytes wel rechtstreeks uit de gecontroleerde kernelkopie.

Er is geen RPC-retry. Een send met `EAGAIN` heeft volgens het bestaande
kernelcontract niets gepubliceerd; een gequeued bericht kan daarna wel door
de dienst verwerkt zijn. Na een verloren antwoord of `Unknown` mag de app een
mutatie dus niet stil opnieuw verzenden. De afzonderlijke catalogusclient kan
deze sessie- en onzekerheidsregels toepassen op de ongewijzigde envelope.

## Bouwen en controleren

Vanuit de repositoryroot, met de vastgelegde Rust 1.90.0-toolchain beschikbaar:

```sh
cargo test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime
cargo clippy --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime --all-targets -- -D warnings
cargo build --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime --target x86_64-unknown-none
python3 -B sdk/examples/catalog-read/build.py
```

Gebruik bij parallel werk een eigen `CARGO_TARGET_DIR`. De SDK is een afzonderlijke
workspace met eigen lockfile. De voorbeeldbuilder overschrijft alleen voor zijn
eigen subprocess de linkerflags, zodat het kernel-linkerscript niet wordt gebruikt.

De hosttests vergelijken alle payloadlengtes 0–256 met de echte
`capability-core`-parser, controleren corrupte envelopes en syscallresultaten,
en voeren send/receive/intrekking uit met het bestaande pure capabilitymodel.
Dat is hostbewijs. Crosscompilatie levert een native ELF maar bewijst geen
boot, isolatie, device-I/O of gebruikershandeling. De native integratie van
de SDK in een echte gastproef heeft een afzonderlijke bewijsbundel.

## Contractbronnen

- Eigen [kernel- en syscallcontract](../../docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md),
  26 oktober 2026, ABI 1.0 en Message320; actuele implementatie
  [`m4.rs`](../../kernel/src/m2/m3/m4.rs) en
  [`capability-core`](../../crates/capability-core/src/lib.rs).
- Eigen [M5-contract](../../docs/implementation/M5-BLOCK-SERVICE.md) en
  [M6a-contract](../../docs/implementation/M6A-WRITE-RECOVERY.md): ABI minor 1/2
  behouden de registerconventie en voegen blokoperaties toe. Deze SDK biedt
  gewone apps geen blokinterface aan.
- [Privacycontract revisie 3](../../docs/LOCAL-PRIVACY-CONTRACT.md): appinstallatie
  verleent geen algemene document-, apparaat- of netwerkrechten.

De tests bewijzen geen menselijke audit of algemene veiligheid. Native uitvoering,
bronhashes en beperkingen staan afzonderlijk onder
[`docs/verification/parallel-g01`](../../docs/verification/parallel-g01/).

