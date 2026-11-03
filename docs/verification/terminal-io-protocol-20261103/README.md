# Terminal Message320-I/O-protocol — hostcheckpoint

Datum: 3 november 2026. Branch: `simonvanieperen-vigilant-robot`.
Geteste broncommit: `49e83f56ed083b4f7a9628559d6a37f99b986dca`.

## Contract

De SDK-kern bevat nu een versie-1 terminal-I/O-payload met vaste tag
`0x54454d494f3031`, een header van 28 bytes en maximaal 228 databytes binnen
de bestaande Message320-payloadlimiet van 256 bytes. De header bevat magic,
versie, berichttype, job-ID, volgnummer, datalengte en gereserveerde nulbytes.
Berichttypen zijn stdin, stdin-EOF, stdout, stderr en output-EOF.

Een receiver bindt iedere richting aan één verwachte peer, job-ID, exact
volgnummer en tag; ontvangen capabilities zijn niet toegestaan. Ongeldige
berichten veranderen de volgordestatus niet. EOF sluit die richting en
volgnummers lopen niet terug. De caller moet op fouten ontvangen capabilities
sluiten. De peerwaarde moet rechtstreeks komen uit de door de kernel gevulde
`Received.sender`, nooit uit het payload. Job-ID's zijn labels, geen rechten.
De exit-/fault-/killstatus komt afzonderlijk van de kernel
`native::status`-API; dit protocol draagt geen vertrouwde exitstatus.

Dit is een hosttestbaar SDK-contract, niet de aansluiting op echte endpoints.
Geen Terminal-proces, keyboard/renderpad, launchbroker, per-job bytequota,
native procesuitvoering of echte kernelstatus is getest.

## Controles

Rust `1.90.0`; de hostlinker GCC 15.2 en `libc6-dev` zijn alleen voor deze run
onder `/tmp/nieuwos-terminal-toolchain` uitgepakt, zonder systeembrede
installatie. Met dezelfde tijdelijke hostlinkeromgeving als beschreven in
[het Terminal-corecheckpoint](../terminal-core-20261103/README.md):

| Exact commando | Resultaat |
|---|---|
| `$HOME/.cargo/bin/cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml -p nieuw-os-terminal-core -- --check` | exit 0 |
| `$HOME/.cargo/bin/cargo +1.90.0 check --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-terminal-core` | exit 0 |
| `$HOME/.cargo/bin/cargo +1.90.0 clippy --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-terminal-core --all-targets -- -D warnings` | exit 0 |
| `$HOME/.cargo/bin/cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-terminal-core` | exit 0; 11 tests |
| `$HOME/.cargo/bin/cargo +1.90.0 check --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-terminal-core --target x86_64-unknown-none` | exit 0 |
| `git diff --check` | exit 0 |

Volledige uitvoer staat buiten Git in
`/tmp/nieuwos-terminal-toolchain/terminal-io-protocol-gates.log`.
SHA-256: `821d8ac9a6869ae44f48650dc1df8f38d0aecf856a2ecf9aa995663e0d4e13ca`.

## Bronvingerafdrukken

| Bestand | SHA-256 |
|---|---|
| `sdk/terminal-core/src/lib.rs` | `cf4c9ecaea9bc96486d1ce6057d537961c36f19839a43dd7069ab6f7a418d286` |
| `sdk/terminal-core/README.md` | `a6e2024d4e7712ab6ab55b5d649bf3fd21ea25f69b250a4a45c3d37e7d2f9567` |

De vaste roadmapmeter blijft **41/100**. Deze code sluit geen rubriccriterium
op het vereiste native bewijsniveau.
