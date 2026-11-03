# PR127-TCP-FIX — TCP connect-deadline en SendTcp-terminaliteit

Bron: PR #127 head `798706c2a53d0062ce0474e5f84f294b2c37b825`, branch
`fix/pr127-tcp-deadline-send`. Bewijsniveau: **alleen broncode + rustfmt**.
Tests en clippy zijn **niet uitgevoerd** (geen host-`cc`/libc-dev; geen
packages of nep-linker geïnstalleerd).

## Bevindingen en herstel (`src/lib.rs`)
1. `poll()` annuleerde iedere niet-`Closed` socket na `owner.deadline`, dus ook
   een actieve `Established`-verbinding werd `TimedOut`. Nu alleen tijdens
   opbouw (`SynSent`/`SynReceived`). Na de handshake blijft de bestaande
   smoltcp-inactiviteitstimeout (`set_timeout`, gereset door peerverkeer)
   leidend; token-, eigenaar-, retire- en revocationcontroles ongewijzigd.
2. `SendTcp` vertaalde `InvalidState` altijd naar `Again`. Nu, analoog aan
   `ReceiveTcp`: `Closed` wanneer de verbinding `Closed` is (reset of
   stack-timeout), anders `Again` (bijv. nog `SynSent`).

## Regressietests (`tests/service.rs`)
- `established_connection_with_traffic_outlives_connect_deadline`: deadline 30
  ticks, echo-verkeer t/m tick 200 moet `Established` blijven (oude code:
  `TimedOut` vanaf tick 30); daarna idle → stack-timeout meldt `Closed`, en
  `SendTcp` geeft `Closed` (oude code: `Again`).
- `send_after_peer_reset_reports_terminal_closed`: `SendTcp` in `SynSent` =
  `Again`; na peer-RST geven `SendTcp` en `ReceiveTcp` `Closed` (oude code:
  `SendTcp` = `Again`); vreemd token blijft `Stale`; tellers ongewijzigd.
Negatieve reproductie is daardoor alleen door code-analyse vastgesteld, niet
door een uitgevoerde rode testrun.

## Commando's (in `services/network-service`, Rust 1.90.0, aparte targetdir)
| Commando | Exit | Resultaat |
|---|---|---|
| `cargo check --tests --locked --ignore-rust-version` (baseline) | 101 | build script: linker `cc` ontbreekt |
| `cargo test --locked --ignore-rust-version` | 101 | niet uitgevoerd: linker `cc` ontbreekt |
| `cargo clippy --locked --ignore-rust-version --all-targets -- -D warnings` | 101 | niet uitgevoerd: linker `cc` ontbreekt |
| `cargo fmt --check` | 0 | geslaagd |

Manifest vraagt Rust 1.91; alleen 1.90 beschikbaar (vandaar
`--ignore-rust-version`). Logs buiten Git:
`../pr127-tcp-fix-logs/{baseline-check,baseline-test-norun,fixed-test,fixed-clippy,fixed-fmt}.log`
(relatief t.o.v. de worktree-ouder).

## Open
Integrator moet in CI/toolchain 1.91 met linker `cargo test` en `cargo clippy
--all-targets -- -D warnings` draaien, en de tests eerst tegen `798706c`
rood zien.
