# Issue #43 — terminale ontvangst na verbindingstime-out

Origineel issue: https://github.com/simonvanieperen/nieuw-os/issues/43.
Basis: `380111b68688c981eec6abb8fab29174a29f5601` op `main`.
Scope: één fout in het N01-netwerkspoor; geen afronding van de 50/100-route.

## Invariant en reparatie

Na een afgebroken TCP-verbindingsopbouw moet een geldige `ReceiveTcp` eindigen
met `Status::Closed`. `Again` betekent dat een volgende poging nog zinvol is.
`Service::poll` annuleert de socket bij de connectdeadline en bewaart
`TimedOut` voor diagnose. De `InvalidState`-tak bij ontvangst herkende alleen
`Closed`, waardoor de gemaskeerde `TimedOut` blijvend `Again` opleverde.

De reparatie herkent beide terminale toestanden uitsluitend in deze fouttak.
Normale ontvangst en het uitlezen van eventueel gebufferde data lopen eerst
via de bestaande stack. Wireformaat, API, rechten en dependencies veranderen niet.

De regressietest controleert `Again` vóór de deadline, herhaald `Closed` na de
deadline, behoud van `State = TimedOut`, lege uitvoer en ongewijzigde
ontvangst-/weigertellers, weigering van verkeerde/verouderde tokens en
`Connecting`/`Again` na opnieuw openen met een nieuw token.

## Uitgevoerde hostcontroles

Toolchain: `rustc 1.91.1 (ed61e7d7e 2025-11-07)`. Alle commando's vanaf de
repositoryroot. Logs: `/tmp/nieuw-os-issue43-logs/` tijdens uitvoering;
duurzaam bewaard onder `outputs/issue-43/` in de automatiseringsworkspace.
[Compacte bron-, log- en native hashes](issue-43-receive-timeout.json).

| Commando | Exit | Resultaat |
|---|---:|---|
| `cargo +1.91.1 test --locked --manifest-path services/network-service/Cargo.toml receive_after_connect_timeout_reports_terminal_closed` (nieuwe test, oude productiecode) | 101 | Rood: `left: Again`, `right: Closed` |
| `cargo +1.91.1 test --locked --manifest-path services/network-service/Cargo.toml` | 0 | 10 servicetests geslaagd |
| `cargo +1.91.1 fmt --manifest-path services/network-service/Cargo.toml --all -- --check` | 0 | Geslaagd |
| `cargo +1.91.1 clippy --locked --manifest-path services/network-service/Cargo.toml --all-targets -- -D warnings` | 0 | Geslaagd |
| `cargo +1.91.1 build --locked --manifest-path services/network-service/Cargo.toml --release --target x86_64-unknown-none` | 0 | Allocatorvrije library gebouwd; geen runtimebewijs |

## Aanvullende host- en native regressiegates

Alle onderstaande commando's eindigden met exit **0**:

```sh
for crate in net-broker net-stack; do
  cargo +1.91.1 test --locked --manifest-path "services/$crate/Cargo.toml"
  cargo +1.91.1 clippy --locked --manifest-path "services/$crate/Cargo.toml" --all-targets -- -D warnings
  cargo +1.91.1 fmt --manifest-path "services/$crate/Cargo.toml" --all -- --check
done
cargo +1.90.0 test --offline --locked --manifest-path services/network-service/protocol/Cargo.toml
cargo +1.90.0 test --offline --locked --manifest-path sdk/net-client/Cargo.toml
python3 -B -m unittest discover -s tools -p test_network_service_peer.py
```

Resultaat: 31 brokerintegratietests + 4 doctests; 26 stacktests; 4 protocoltests;
6 clienttests; 11 Python-peerharnastests. De service zelf heeft 10 groene tests.

Native broncommit: `5925c915f6b8d02842fffd89a94f21c0e018689a`.
Beide runs hebben bronvingerafdruk v3
`9a8b521f4b2e5a24b279142d8554538c5cf618e3e55a9cac3e7070e12ad4fc39`.
Kernel/loader: Rust 1.90.0; netwerk-apps: Rust 1.91.1. QEMU 10.2.1,
TCG, vast machineprofiel `pc-q35-8.2`, OVMF 4M. Uitsluitend verse wegwerpimages
en een synthetische localhost-Ethernetpeer, geen fysieke disks/internetdoelen.

Na het installeren van de CI-toolchains en `cargo fetch --locked` voor de
manifests uit `.github/workflows/net-broker-quality.yml`:

```sh
export NOS_QEMU_MACHINE=pc-q35-8.2
export NOS_TARGET_DIR="$PWD/target/issue43-native"
export CARGO_TARGET_DIR="$PWD/target/issue43-host"
export OS_OVMF_CODE=/usr/share/OVMF/OVMF_CODE_4M.fd
export OS_OVMF_VARS=/usr/share/OVMF/OVMF_VARS_4M.fd
export CARGO_NET_OFFLINE=true
NOS_BUILD_DIR="$PWD/build/issue43-normal" python3 -B -u tools/run_network_service.py --case normal
NOS_BUILD_DIR="$PWD/build/issue43-client-exit" python3 -B -u tools/run_network_service.py --case client-exit
```

| Gate | Harnas-exit | QEMU-exit | Duur | Resultaat |
|---|---:|---:|---:|---|
| `normal` | 0 | 33 | 8,27 s | 44 TX-completions, 35 RX-deliveries, peer compleet, cleanup groen |
| `client-exit` | 0 | 33 | 7,79 s | 25 TX-completions, 18 RX-deliveries; client A eindigt eerder, B voltooit; cleanup groen |

Beide gates rapporteren `passed = true`, `network_service_gate = true`,
geen timeout/uitvoeroverloop en lege stderr. Dit herhaalt bestaande native
netwerkregressiedekking; het specifieke `ReceiveTcp`-na-time-outscenario is
rood/groen op de host aangetoond, niet als nieuwe native scenarioassertie.
De laatste wijzigingen voegen alleen documentatie/bewijs toe; de uiteindelijke
bronvingerafdruk moet gelijk blijven aan deze geteste bron.

## Open grenzen

Concept-PR: https://github.com/simonvanieperen/nieuw-os/pull/134.
Nieuwe exacte-head hosted CI en review volgen op de PR. Een tweede agent heeft
de diff alleen-lezen beoordeeld zonder bevindingen; geen menselijke audit.
Geen hardware-/Windows-/TLS-/volledige releaseacceptatie. De voortgang blijft
**41/100 → 41/100**; #43 blijft open. Terugdraaien kan met een normale revert
van deze gerichte reparatie; geen datamigratie.
