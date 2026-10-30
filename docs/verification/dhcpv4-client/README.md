# DHCPv4-clientcore: hostdeelbewijs, geen native configuratie

Basis: `1bb8acfb20c6b43a08bece2d8bbecad43c635c14`.
Branch: `simonvanieperen-dhcp-client`.
De hieronder gehashte bron is getest; dit checkpoint verandert geen kernel,
SDK, manifest, NAS, ABI of capability. Centrale status/assessment zijn sinds
de TL-NET-toewijzing eigendom van teamleider `89f84098`; dit is alleen
brongebonden bewijs. Puntvoorstel N01–N04: **0/10 blijft 0/10**, totaal
**34/100 blijft 34/100**. FR-N01–N04 zijn niet afgetekend.

## Geteste bron

| Bestand | SHA-256 |
|---|---|
| `services/net-protocol/src/lib.rs` | `93bffcf2fc81e13ed74f5015755f218da8e0b0700cb9e0f95127ec0d9722005a` |
| `services/net-protocol/src/dhcp_client.rs` | `1f9e64b272665d1ac43d63ae54fdc539f21d236ef91b636b3c2da39f695fd04c` |
| `services/net-protocol/tests/dhcp_client.rs` | `cbbf859bb7851a281ed7e45dc486546b5b5230a6938e18c949d6a85f5e7d6f19` |
| `services/net-protocol/README.md` | `ccdf87e9713b155b3ddea83b245b7a3569d72f083601bdb1d60885581402e0e7` |

De dependencyvrije `no_std` core bewaakt één directe DHCP-uitwisseling en
lease. Een ACK wordt kandidaat tot de caller een werkelijke conflictcheck
bevestigt. NAK, conflict, linkverlies en exact leaseverval verwijderen de
bruikbare lease. Renewal gebruikt een nieuwe XID, unicast voor T1 en
broadcast vanaf T2. De klok is caller-owned monotone seconden; geen
pollteller, UTC, RTC- of TLS-vertrouwen.

Elf nieuwe modeltests omvatten lifecycle/renew/rebind, server-/XID-/
endpointbinding, malformed input, conflict, linkverlies, exacte deadlines,
klokrollback/-overflow, oneindige lease en toegestane ontbrekende
UDP-checksum zonder verlies van berichtidentiteitscontroles.

## Uitgevoerde commando's

Omgeving: Rust 1.90.0, lokale eerder herstelde GCC-sysroot onder
`/tmp/nieuw-os-dhcp-hostcc/root`, linker
`/tmp/nieuw-os-qemu-102/host-cc`; geen globale installatie of nieuwe dependency.
`LD_LIBRARY_PATH`, `GCC_EXEC_PREFIX`, `COMPILER_PATH`, `LIBRARY_PATH` en
`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER` wijzen naar die sysroot.
`CARGO_TARGET_DIR=target/dhcp-hostcc`; geen globale `RUSTFLAGS`.
Logs zijn compact bewaard naast dit bestand.

| Exact commando | Exit | Uitkomst / log | Tijd |
|---|---:|---|---:|
| `python3 -B tools/dev.py cargo +1.90.0 test --manifest-path services/net-protocol/Cargo.toml --locked --offline` | 0 | 68/68 tests, waarvan 11 clientmodellen; `tests.txt` | 0,59 s |
| `python3 -B tools/dev.py cargo +1.90.0 clippy --manifest-path services/net-protocol/Cargo.toml --all-targets --locked --offline -- -D warnings` | 0 | Geen waarschuwingen; `clippy.txt` | 0,44 s |
| `python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path services/net-protocol/Cargo.toml --all -- --check` | 0 | Schoon; `fmt.txt` | Zie log |
| `python3 -B tools/dev.py cargo +1.90.0 build --manifest-path services/net-protocol/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline` | 0 | Freestanding releasebuild; `none.txt` | 0,33 s |
| `git diff --check` | 0 | Geen whitespacefouten | Niet gemeten |

## Bewijsgrens en eerstvolgende stap

**Niet uitgevoerd:** DHCP over de eigen kernel/NIC, echte ARP-conflictprobes,
adres-/route-/resolverapply, retries/backoff, reconnect, DHCPDECLINE,
leasepersistentie, TLS, luisterende TCP-server, geauthenticeerde NAS-peer,
proces-/bestemmingsgrants en intrekking. `Client` verleent geen rechten.
Een gekopieerde lease vereist ook bij de caller deadline- en
interfacegeneratiecontrole.

Het directe profiel weigert relays, ontbrekende subnetmaskers en oneindige
leases expliciet. Het ondersteunt geen volledige DHCP-interoperabiliteit;
zie component-README. RFC 2131 (R. Droms, maart 1997), §§4.4.1/4.4.5 en
tabel 4: <https://www.rfc-editor.org/rfc/rfc2131.txt>.
De bestaande native ARP/UDP/DNS-pair op **eerdere** bron `9ddc6b6` staat in
`../full-release-network-dev-9ddc6b6/`; deze wordt niet overgeërfd als native
bewijs voor de client.

Eerstvolgend: afzonderlijk codec-reviewherstel voor RFC 1542-minimumpadding
op PR #70, daarna opt-in native DHCP-integratie afstemmen met de eigenaar van
manifest/kernel/build. De bestaande exacte netwerkcountergate blijft intact.

## Samengevoegde codec-reviewfix

De geïsoleerde min-300-bytefix van PR #70 (`5d95ea9`) is als `30d791c`
naar deze branch overgenomen. Op **die samengevoegde bron** zijn dezelfde
vier exacte Cargo-commando's hierboven opnieuw uitgevoerd:
volledige suite **69/69**, Clippy `-D warnings`, fmt-check en
`x86_64-unknown-none` releasebuild alle **exit 0**. De tests duurden 0,60 s;
overige tijden staan in `combined-{clippy,fmt,none}.txt`. Volledig testlog:
`combined-tests.txt`. De elf clientmodellen zijn ongewijzigd.

De bronhashes voor clientmodule/tests/lib hierboven blijven gelijk aan
`35b5ef3`; de codec is nu exact de gehashte paddingbron in
`../dhcpv4-codec/README.md`. De component-README bevat bovendien de uitleg
over de nieuwe 300-bytegrens; de eerdere README-hash beschrijft alleen het
oorspronkelijke clientcheckpoint. Geen native bewijs toegevoegd en geen
punten toegekend.
