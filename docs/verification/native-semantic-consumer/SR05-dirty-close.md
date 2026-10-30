# TASK-UI-SR05-REGRESSIONS — dirty-close-focus en Escape

Assessment 30 oktober 2026, 19:38 CET (Europe/Amsterdam). Expliciete toewijzing
queue `82664b56a03dad5f2cdd64314234c406dd74a299`, blob
`b54f17507a9e8b536a97c87837ab7fa88fc81325`.
Afzonderlijke branch `codex/native-sr05-regressions`, werkboom
`/tmp/nieuw-os-sr05`, exacte basis
`70be43d63b50294a2f40bc2ba431aab557498019`.
Frozen70, hover29b79 en PR #68 zijn niet gewijzigd.

## Resultaat en invariant

De verwachte positieve matrix slaagt direct op de bestaande controller.
Er is **geen rode productreproductie gevonden of kunstmatig gemaakt**.
Daarom zijn uitsluitend bestaande `ui/native/tests/interaction.rs` en dit
nieuwe bewijsdocument gewijzigd; geen controller- of rendererfix.

Escape in de dirty-close-dialoog betekent altijd verder bewerken, ongeacht
welke dialoogknop focus heeft. Een toetsenbordfocusverplaatsing is geen Save-
of Discard-admissie. Alleen de drie zichtbare modale acties zijn in de
actuele semantiekboom beschikbaar; oude snapshots zijn niet geldig.

Twee nieuwe tests gebruiken de bestaande host-capability-/volumeharnashelpers
en dezelfde `scenario!`-stack als de overige interactietests:

1. `sr05_escape_preserves_dirty_document_from_every_dialog_focus`: zes
   gevallen, alle drie focusdoelen `KeepEditing`, `Discard`, `SaveAndBack`
   op zowel 100% als 200%, verkregen met echte controller-Tabevents.
   De boom bevat exact de drie ingeschakelde activeerbare dialoogkinderen,
   geen set-value-actie en precies één focus op het verwachte kind.
   New/Refresh/Back/Save/Document/Object ontbreken uit de boom en hun actuele
   activatie wordt geweigerd met `InvalidInput`; set-value op Document
   eveneens. SaveAndBack met de oude editorgeneratie wordt `StaleInput`.
   Deze weigeringen veranderen focus/generatie niet.
   Vervolgens levert Escape Editor/Document-focus, dezelfde dirty draft
   `New Original`, geen saving, dezelfde originele diskbytes `Original`,
   revision1 en exact hetzelfde aantal opslagrequests. Geen COMMIT.
   De nu oude dialooggeneratie mag achteraf evenmin Discard activeren.
2. `sr05_dialog_tab_and_shift_tab_wrap_without_leaving_modal`: volledige
   voorwaartse en achterwaartse cyclus over alle drie doelen, inclusief beide
   wrapgrenzen; iedere stap houdt precies één ingeschakeld dialoogkind
   gefocust, dezelfde dirty draft en geen saving/opslagrequests/COMMIT.
   Dit tweede geval gebruikt de standaard100%-fixture; 200%-dekking voor
   focusverplaatsing en Escape zit in de zes gevallen hierboven.

De tests controleren echte controller-/semantiekuitvoer in een hostproces,
geen native inputtransport of werkelijk uitgevoerd hulpmiddel.
Er zijn geen nieuwe API's, bevoegdheden, trust boundaries, dependencies,
manifests, locks of root-/ABIwijzigingen.

## Uitvoering en exacte commando's

Bestaande toolchain Rust1.90.0; eigen Cargo-targetmap
`/tmp/nieuw-os-sr05-target`. Omgeving:

```sh
P=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools/nieuw-os
export PATH="$P/cargo/bin:$P/rustup/toolchains/1.90.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld:$PATH"
export CARGO_HOME="$P/cargo" RUSTUP_HOME="$P/rustup" RUSTUP_AUTO_INSTALL=0
export CARGO_TARGET_DIR=/tmp/nieuw-os-sr05-target
export LD_LIBRARY_PATH="$P/host-cc/usr/lib/x86_64-linux-gnu"
cd /tmp/nieuw-os-sr05
```

In de tabel is `cargo` exact `cargo +1.90.0`.
Vooraf is `cargo fmt --manifest-path ui/native/Cargo.toml` uitgevoerd, exit0.
Volledige logs:
`/home/simon/.copilot/session-state/1b87793863ad4dcc8f16a0bd10884654/files/sr05/`.
Tijden zijn gemeten wandkloktijden inclusief Cargo, geen UI-prestatietest.

| Exact commando | Exit | Tijd | Resultaat/log |
|---|---:|---:|---|
| `cargo test --offline --locked --manifest-path ui/native/Cargo.toml --test interaction sr05_` | 0 | 2,244 s | `targeted.log`: 2 nieuwe tests groen, 21 bestaande gefilterd |
| `cargo test --offline --locked --manifest-path ui/native/Cargo.toml` | 0 | 0,639 s | `suite.log`: 23 interactietests groen (21 bestaande + 2 nieuwe); geen actieve unit-/doctests |
| `cargo fmt --manifest-path ui/native/Cargo.toml -- --check` | 0 | 0,061 s | `fmt.log` |
| `cargo check --offline --locked --manifest-path ui/native/Cargo.toml --tests` | 0 | 1,653 s | `check.log` |
| `cargo clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings` | 0 | 0,459 s | `clippy.log` |
| `cargo build --offline --locked --manifest-path ui/native/Cargo.toml --target x86_64-unknown-none` | 0 | 1,886 s | `none.log`: library-crosscompile, geen boot |
| `git diff --check` | 0 | 0,193 s | `diff.log` |

De basis70 bevat 21 interactietests, niet de twintig van de andere0818leaf.
De uitvoer van deze exacte bron is leidend; oude suiteaantallen zijn niet
overgenomen. De R68-renderfix en hoverfix zitten niet automatisch in deze
aparte basis of dit bewijs.

## Hashbinding

SHA-256 van de geteste testbron en ongewijzigde productbronnen:

```text
668dcc0072f1f86434cdce08585f35157854191dafd2d135e0b1e1bc114f2ac6  ui/native/tests/interaction.rs
3bd3cd316218b72661275b9e956132c46bc72f93db917c39faf76125884bf11f  ui/native/src/controller.rs (ongewijzigd)
fc2682ebedafe92e4c99501f7d2a12a49f145c7bec92208db2e2fc789fea9434  ui/native/src/render.rs (ongewijzigd)
```

SHA-256 van de persistente uitvoerlogs:

```text
9eac66a7b7f2ecea8f152d01d6039cf0e39ef837c9fc4f6668ebc31006bed5aa  targeted.log
edd977f2327ee9bdea2f4d08f1004fe7da5a3447f67f46029f6f0752ad51fd0d  suite.log
39b61b7187c938d962d2443e38247f48d790ce47977930897a8e5848dcf0a0cb  fmt.log
ebd5e361c06c6a884132efff8571935346f357961bcfd0bbd6beb44e3660d654  check.log
d2217173a0735a3913cdb463ab34f9a0941381241fc5bffc13f20327a397c5e0  clippy.log
7bccc8a11594b023ac5d08a6101da4052fa59c09f225b6eb86fa32edfbc1ff6d  none.log
d5ebd19b2c86483d13ad7264c4031c633df213893ff7b764e0bbcf8a28606fd7  diff.log
```

## Bewijsgrens en overdracht

Dit sluit de beschreven **hostregressiematrix**, niet SR05-nativeacceptatie.
Niet uitgevoerd: VM, native200%-input, screenreader/externe hulpmiddelen,
menselijke review, diskduurzaamheid/koude recovery, gecombineerde image/CI.
De RAM-diskoracle is een hostfixture, geen native persistentiebewijs.
Geen VMslot gebruikt, geen nieuwe runtimeframes of oud bewijs herlabeld.
De native externe consumer blijft afhankelijk van REL voor IPC/protocol/
rootbootstrap; er is geen tweede rootwriter gestart.

Bronfreeze voor gerichte7f/Appsreview; eventuele concrete bevinding eerst
binnen dezelfde twee-file-scope herstellen. Bij benodigde API-/trustwijziging
wordt alleen dat deel geblokkeerd en naar de eigenaar gerouteerd.
Terugdraaien betreft uitsluitend tests/documentatie, geen productmigratie.

Assessmentvoorstel aan centrale eigenaar: **34 -> 34 / 100; U 0 -> 0 / 15**.
U04 en betrokken FR-U05/productkwaliteit blijven open zonder passend
native/menselijk bewijs. De centrale score-/README-/PROJECT-STATEbestanden
zijn buiten dit exclusieve schrijfgebied; hun eigenaar ontvangt dit
assessment met datum, basis, branch, taak en bewijs.
