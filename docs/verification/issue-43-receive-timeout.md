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
compacte resultaten worden hieronder vastgelegd.

| Commando | Exit | Resultaat |
|---|---:|---|
| `cargo +1.91.1 test --locked --manifest-path services/network-service/Cargo.toml receive_after_connect_timeout_reports_terminal_closed` (nieuwe test, oude productiecode) | 101 | Rood: `left: Again`, `right: Closed` |
| `cargo +1.91.1 test --locked --manifest-path services/network-service/Cargo.toml` | 0 | 10 servicetests geslaagd |
| `cargo +1.91.1 fmt --manifest-path services/network-service/Cargo.toml --all -- --check` | 0 | Geslaagd |
| `cargo +1.91.1 clippy --locked --manifest-path services/network-service/Cargo.toml --all-targets -- -D warnings` | 0 | Geslaagd |
| `cargo +1.91.1 build --locked --manifest-path services/network-service/Cargo.toml --release --target x86_64-unknown-none` | 0 | Allocatorvrije library gebouwd; geen runtimebewijs |

## Nog uit te voeren

Bestaande native `normal`/`client-exit`-QEMU-gates, nieuwe exacte-head hosted CI
en review. Geen hardware-/Windows-/volledige releaseacceptatie. De voortgang
blijft **41/100 → 41/100**; #43 blijft open. Terugdraaien kan met een normale
revert van deze gerichte reparatie; geen datamigratie.
