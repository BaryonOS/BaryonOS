# TASK-U04-APP-CONSUMER — app-lokale generatiegebonden acties

## Bron en behoud

Assessment 30 oktober 2026, 19:28 CET (Europe/Amsterdam). Expliciet
heractiveerd via queuecommit `6d34fdfe5e8886b8abe65a43650f3347c95f9156`.
Werkbranch `codex/native-semantic-consumer-20261030`, basis
`89e200451066f86a1cffb5a8dab4b3dd4089e587`.

Voor de wijziging zijn HEAD, status en SHA-256 vastgelegd. De bestaande
ongecommitteerde `src/lib.rs` en nog niet getrackte
`tests/semantic_actions.rs` zijn integraal gekopieerd naar:
`/home/simon/.copilot/session-state/1b87793863ad4dcc8f16a0bd10884654/files/u04-app-consumer/`.
Geen reset, reconstructie, branchwissel of wijziging van andermans WIP.
De volledige oorspronkelijke wrapper is behouden.

| Bewaard bestand | SHA-256 |
|---|---|
| `lib.before.rs` | `9683f3a06de8e1e9da2098157ef9fc2499932190ee85918ca47a2c81dae8d692` |
| `semantic_actions.before.rs` | `a6b13a09bb8b3246d8a1c2959025efec5765eb9baa1cbeddd679b61ea8e11370` |

Schrijfgebied: alleen `sdk/examples/materiaal/src/lib.rs`,
`sdk/examples/materiaal/tests/semantic_actions.rs` en deze nieuwe bewijsnote.
Geen manifest/lock/token, gedeelde protocol-API, root, kernel, runner, VM of
wijziging van de bevroren hoverleaf/PR #68.

## Invariant en implementatie

De bestaande controller blijft de enige autoriteit voor node, toegestane actie,
enabled-state en actuele generatie. Een snapshot is een momentopname van
dezelfde app, geen capability en geen toestemming voor een ander proces.

De behouden wrapper voegt een `SemanticSnapshot { node_count, generation }`
toe. `semantic_snapshot` bouwt de boom en generatie uit dezelfde immutable
controller-view; de bestaande `semantics` blijft hetzelfde aantal nodes
teruggeven. App-lokale `activate_semantic` en `set_semantic_value` delegeren aan
de bestaande controller. Fouten worden geteld en als concrete
`Error::Controller` teruggegeven, niet verborgen of als succes behandeld.
Een waardewijziging bewerkt alleen de draft; opslaan blijft een aparte actie.
Geen nieuwe IPC, autorisatie, opslagroute of gewijzigde generatiecontrole.

De heractiveerde positieve test verwachtte ten onrechte succes voor Save met
de generatie van **voor** de edit. Dat faalde terecht met `StaleInput`.
De test controleert nu expliciet deze weigering, dat er geen opslagtransactie
is begonnen en dat de RAM-disk nog leeg is. Daarna haalt hij een nieuwe
snapshot op, controleert de gewijzigde generatie en start daarmee Save.
De errorcounter is vier: onbekende Save-node, oude New-generatie, oude
Document-generatie en oude Save-generatie. De tweede test controleert
read-only New en niet-beschikbare Document-acties; beide geven `InvalidInput`,
geen draft en precies twee fouten.

De opslagtransporten en capability-engine in de test zijn hostfixtures.
Het resultaat bewijst alleen de app-lokale koppeling en Save-admissie,
**niet** een voltooide duurzame save, native semantische IPC of screenreader.
De bestaande twintig adaptertests bewaken daarnaast de ongewijzigde adapter.

## Exacte uitgevoerde controles

Eigen targetmap `/tmp/nieuw-os-u04-app-target`; bestaande Rust1.90.0,
geen installatie en geen nieuwe afhankelijkheden. Omgeving:

```sh
P=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools/nieuw-os
export PATH="$P/cargo/bin:$P/rustup/toolchains/1.90.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld:$PATH"
export CARGO_HOME="$P/cargo" RUSTUP_HOME="$P/rustup" RUSTUP_AUTO_INSTALL=0
export CARGO_TARGET_DIR=/tmp/nieuw-os-u04-app-target
export LD_LIBRARY_PATH="$P/host-cc/usr/lib/x86_64-linux-gnu"
```

In de tabel is `cargo` exact `cargo +1.90.0`; alle commando's zijn uitgevoerd
vanuit de repositoryroot. Logmap is de bovenstaande persistente WIP-map.
Tijden zijn gemeten wandkloktijden inclusief Cargo, geen prestatieresultaat.

| Exact commando | Exit | Tijd | Resultaat/log |
|---|---:|---:|---|
| `cargo test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --test semantic_actions` op bewaarde WIP | 101 | 1,769 s | `red.log`: 1/2; positieve test unwrapt terecht geweigerde `Controller(StaleInput)` |
| `cargo test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --test semantic_actions` na testcorrectie | 0 | 0,232 s | `semantic.log`: 2/2 |
| `cargo test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml` | 0 | 0,764 s | `suite.log`: 20 bestaande adaptertests + 2 semantische tests; ELF-testbestand heeft zonder feature nul actieve tests |
| `cargo fmt --manifest-path sdk/examples/materiaal/Cargo.toml -- --check` | 0 | 0,053 s | `fmt.log` |
| `cargo check --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --tests` | 0 | 1,990 s | `check.log` |
| `cargo clippy --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --all-targets -- -D warnings` | 0 | 0,283 s | `clippy.log` |
| `cargo build --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --target x86_64-unknown-none` | 0 | 1,623 s | `none.log`: library-crosscompile, geen native ELF-/boottest |
| `git diff --check` | 0 | <0,01 s | `diff.log` |

SHA-256 van geteste bron:

```text
9683f3a06de8e1e9da2098157ef9fc2499932190ee85918ca47a2c81dae8d692  sdk/examples/materiaal/src/lib.rs
295ac5cb907345bd9b01090feca61ccd19e7eea3a31df0866649a204f43f6bed  sdk/examples/materiaal/tests/semantic_actions.rs
```

SHA-256 van volledige persistente logs:

```text
f3fdc3953a985dc60540c4eb2ecdabe449192d5e5e37bb1d0b100cdc239059f0  red.log
8bceac206a5ac82c6b044a451cb65326219c1530cd62596647a184a61bab76e4  semantic.log
ee28e85035e47c343e2eb06554e2e85907df5d582e08f0f48cb1706e7e1d0e2f  suite.log
0f23a60028d40376256201efaa1a3feb031957d83a60f62c67900711f97bccc1  fmt.log
6749e8f7475dc5e2c3a413235a3403780941c391741be0aa561048d49f249bd6  check.log
3e17591df60f05a16092a1a6979a9140eaf0a85a5781116f40b4467561baf477  clippy.log
50a168a3a2c190bb932ef2dbf1ca1845241a284a5faa1b218e41ab57a2c4bd69  none.log
31e227dfe7f5688536231066f2582f62824733ce8453107b1407deab46478298  diff.log
```

## Open grenzen en overdracht

Bronfreeze voor gerichte review7/Apps11. **Niet uitgevoerd:** nieuwe native VM,
externe hulptechnologie/screenreader, menselijke acceptatie, duurzame Save via
deze testfixture, native ELF-featuregate of gecombineerde integratie-CI.
Geen toegekend VMslot gebruikt. Oude 0818-beelden en runtimeproeven blijven aan
hun eigen bron gebonden; hover29b79 blijft ongewijzigd.

Voorstel aan centrale voortgangseigenaar: **34 -> 34 / 100; U 0 -> 0 / 15**.
U04/FR-U05 blijven open zonder native/menselijke acceptatie van de vereiste
consumerketen. Centrale README/progress/PROJECT-STATE zijn niet van deze
schrijftaak; datum, bron, branch, taak en bewijs zijn voor hun assessment
overgedragen. Geen nieuwe architectuurkeuze of veiligheidsaanname; wel de
bestaande aanname dat de controller generatie en toegestane acties afdwingt.
Die controller is hier niet gewijzigd.
