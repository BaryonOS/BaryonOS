# Nieuw OS input service core

Allocatorvrije `no_std`-core met `forbid(unsafe_code)`. Hergebruikt bestaande
`ui-core`-PS/2-decoders en `InputRouter`; geen device-I/O, syscalls, nieuwe
kernelrechten, native service-loop of compositor. Vier bootstrapgebonden
RECV-ingresses scheiden bron, focuscontroller, lifecycle en appverzoeken.

`Service::handle(endpoint, &Received)` valideert endpoint, kernel-afzender,
tag/caps en de canonieke INP1-payload. De native adapter moet `Received` direct
uit de echte kernel halen en onverwacht gedelegeerde caps expliciet sluiten.
`Received::from_bytes` in een hosttest bewijst zelf geen herkomst.

Een `Reply` kiest uitsluitend een vooraf bekende `Route`; de caller maps die
naar de vooraf verleende reply-SEND-capability. `reply_is_current` wordt in
dezelfde servicethread vlak vóór send gecontroleerd. PEEK bewaart input totdat
een geldige ACK volgt; focus/death/reset/overflow maken oude event-/cancelreplies
ongeldig. Empty en commandobevestigingen zijn observaties bij afhandeling en
verlenen geen focus; zij wissen geen later wachtende cancel.
Een afzonderlijke cancel-watermark gaat vóór gewone events. Fouten kunnen
met `error_reply` als canonieke negatieve status worden afgehandeld; er is
geen fallback naar een succes- of Empty-antwoord.

Wire v1: 96 little-endian payloadbytes; header `INP1`, versie, opcode,
requestnummer, epoch, arg0 en arg1. Antwoorden gebruiken opcode|0x8000,
status in arg0 en de gebonden client-ID in arg1. Status 0=Ok, 1=Empty,
2–18=`Error`. Het event begint op byte 40: soort/action/key of buttons op
40–43, modifiers of signed delta op 44–47, eventnummer op 48, focusgeneratie
of cancel-watermark op 56, pointer-X/Y op 64/68. Bytes 72–95 blijven nul.
Een PEEK zonder event heeft een nulgevulde eventbody. Alle requestbytes vanaf
40 zijn nul; ongebruikte arguments zijn eveneens nul.

```sh
cargo +1.90.0 test --manifest-path services/input-core/Cargo.toml --locked --offline
cargo +1.90.0 clippy --manifest-path services/input-core/Cargo.toml --all-targets --locked --offline -- -D warnings
cargo +1.90.0 build --manifest-path services/input-core/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline
```

Gebruik een eigen `CARGO_TARGET_DIR` en de bestaande toolomgeving.
[Contract en native afhankelijkheden](../../docs/implementation/NATIVE-INPUT-SERVICE.md).
[Controleerbaar host-/targetrapport](verification/host-target.json).
