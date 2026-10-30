# TASK-UI-PRIMARY-HOVER: primaire stijl behouden bij aanwijzen

## Afbakening en bron

- Assessment: 30 oktober 2026, 19:22 CET (Europe/Amsterdam).
- Expliciete toewijzing: queuecommit
  `16151b9c28a766c47f900e4a3e9f99b642cc3fe5`.
- Afzonderlijke leaf `codex/native-primary-hover`, basis
  `0818f5ec651b72c32b8795ff0d3d9a6ebdc9783d`.
- Schrijfbestanden: uitsluitend `ui/native/src/render.rs`,
  `ui/native/tests/render_regressions.rs` en dit bewijsdocument.
- Bestaande UX-V01 uit rol15: `Nieuw` verloor bij aanwijzen de primaire
  accentvulling. De zes native beelden op 0818 zijn eerder beoordeeld;
  die beelden bewijzen deze nieuwe code **niet**.
- Appwrapper op basis `89e2004` blijft ongewijzigd/geparkeerd.
  Geen wijziging van PR #68, tokens, API, inputroute, capabilities, kernel,
  runner, manifest of centrale score/statusbestanden.

## Invariant en implementatie

Een ingeschakelde primaire knop blijft primair, ook als de aanwijzer erop
staat. De gedeelde `Paint::button` behoudt daarom `ACCENT` als achtergrond
en `SURFACE` als tekstkleur. Alleen de rand verandert bij aanwijzen van
`ACCENT` naar de bestaande `TEXT`-kleur. Zo blijft hoverfeedback bestaan
zonder een nieuwe token, layoutwijziging of verlies van toetsenbordfocus.

De bestaande disabled-tak blijft voorrang houden. Secondary en Danger zijn
niet gewijzigd. Geen gedrags- of architectuurwijziging buiten het schilderen.
Terugdraaien kan door alleen deze leafdelta terug te nemen; geen
gegevensmigratie of nieuw afhankelijkheidspakket.

De pixelregressie rendert echte `Frame::render_tile`-uitvoer bij 100% en
200%, voor `Nieuw`, `Opslaan` en de primaire bewaaractie in de
dirty-close-dialoog. Zij controleert achtergrond, rand, focus en gelijkblijvende
geantialiasde tekstpixels; de samples liggen buiten de aanwijzer-overlay.
Een tweede test bewaakt uitgeschakeld `Nieuw` en secondary-hover `Vernieuwen`.
De bestaande twee semantiekregressies en twintig interactietests blijven groen.

## Uitvoering

Werkmap `/tmp/nieuw-os-primary-hover`; eigen Cargo-targetmap
`/tmp/nieuw-os-primary-hover-target`. Bestaande Rust 1.90.0-toolchain gebruikt:

```sh
P=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools/nieuw-os
export PATH="$P/cargo/bin:$P/rustup/toolchains/1.90.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld:$PATH"
export CARGO_HOME="$P/cargo" RUSTUP_HOME="$P/rustup" RUSTUP_AUTO_INSTALL=0
export CARGO_TARGET_DIR=/tmp/nieuw-os-primary-hover-target
export LD_LIBRARY_PATH="$P/host-cc/usr/lib/x86_64-linux-gnu"
```

In de tabel is `cargo` exact `cargo +1.90.0`. Logmap:
`/home/simon/.copilot/session-state/1b87793863ad4dcc8f16a0bd10884654/files/primary-hover/`.
Tijden zijn gemeten wandkloktijden inclusief Cargo, geen UI-latentiemeting.

| Exact commando | Exit | Tijd | Log / resultaat |
|---|---:|---:|---|
| `cargo test --offline --locked --manifest-path ui/native/Cargo.toml --test render_regressions primary_buttons_preserve_accent_and_focus_when_hovered_at_both_scales` | 101 | 0,278 s | `red.log`: op originele renderer faalt `Material loses primary fill at Percent100`; accent-soft `15199975` in plaats van accent `679509` |
| `cargo fmt --manifest-path ui/native/Cargo.toml -- --check` | 0 | 0,060 s | `fmt.log` |
| `cargo test --offline --locked --manifest-path ui/native/Cargo.toml --test render_regressions` | 0 | 1,797 s | `regressions.log`: 4/4 |
| `cargo test --offline --locked --manifest-path ui/native/Cargo.toml` | 0 | 2,428 s | `suite.log`: 20 interactietests + 4 renderregressies; geen unit-/doctests aanwezig |
| `cargo check --offline --locked --manifest-path ui/native/Cargo.toml --tests` | 0 | 1,962 s | `check.log` |
| `cargo clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings` | 0 | 0,451 s | `clippy.log` |
| `cargo build --offline --locked --manifest-path ui/native/Cargo.toml --target x86_64-unknown-none` | 0 | 1,721 s | `none.log`: crosscompile, geen boot |
| `git diff --check` | 0 | <0,01 s | `diff.log` |

De definitieve regressie is nogmaals tegen de originele 0818-renderer
uitgevoerd voordat de fix opnieuw is aangebracht (`red.log`); daarna zijn
alle bovenstaande groene gates op de gecombineerde fix/testbron uitgevoerd.
Een eerdere eigen testversie verwachtte exact de voorgrondkleur in
geantialiasde glyphs en faalde na de productfix. Dat was een te strenge
testaanname, geen tweede productfout; hersteld naar vergelijking van de
daadwerkelijke ongewijzigde glyphpixels. Het oorspronkelijke foutlog
`label-test-correction.log` is behouden, niet als groen bewijs gebruikt.
Het initiële reproductielog staat in `initial-red.log`.

## Hashbinding

SHA-256 van de geteste bronbestanden:

```text
f4f41f77b7c84425da53fed9042a493a294cae77cde6ada7e29226012a358fce  ui/native/src/render.rs
7ab43691295efd27207dcc2d37464f258c1eb2b3342cae499736cde6f70bb35a  ui/native/tests/render_regressions.rs
0c588509e0cb0f94bbda2c6969da8d79d463359bfc9684687a684210a54aadcf  ui/prototype/tokens.css (ongewijzigd)
```

SHA-256 van de persistente uitvoerlogs:

```text
1b56ed28ebcad527ef72851fc8d374ee421e36c04e976fbfa4cc8108202a1d4b  red.log
2a43320740002fb49852b5c36e99c885e22fc1b2ddd5508fed0618f613682a09  fmt.log
40d71ef35ffd1e38165217a37a8c8d00464fd453a7258768e5ae848c48e03f95  regressions.log
89ba084e3dd11608c95e0f331fdebe25127ab7fd8544d98431b52d80d62cebae  suite.log
45dd85b6689926615e9fb64d7b247c97d8a30669e082eecd3cb56602f71eaf3e  check.log
32efb9fb21fdd3f0dace98598f4b7c83d275bd3c1267520352c66f259cf23ce4  clippy.log
5ab8cc9048888162a2c217dec727a54ad83b2e4a4cd28abe3811dd5e74ddb6a4  none.log
697148e6a166cab3e11df81c9bfcfd7af069e79c45ee64e6608455d9c727702d  diff.log
886a9474f11543f321eaaf817ece658bfec0afd05d5aadebd5540675200b0244  label-test-correction.log
```

## Bewijsgrens en overdracht

Dit is een kleine hostgeteste rendererfix en een geslaagde no_std-crosscompile.
**Niet uitgevoerd:** nieuwe native VM, native 200%-bediening, screenreader,
menselijke gebruiksproef, koude recovery, algemene visuele heracceptatie,
gecombineerde integratie-CI of volledige FR-acceptatie. Er is geen VMslot
toegekend en er is geen VM gestart. De optionele Impeccable-contexthelper kon
niet starten (`node: command not found`, exit 127); bestaande projectinstructies,
ontwerpregels en tokens zijn rechtstreeks gelezen. Geen tooling geïnstalleerd.

Bronfreeze voor gerichte review door 7/15; bestaande 0818-runtimebeelden worden
niet naar deze leaf herlabeld. Integratie vindt uitsluitend plaats op een
gecontroleerde nieuwe basis met de dan vereiste gecombineerde gates.

Scorevoorstel aan de centrale eigenaar: **34 -> 34 / 100; U 0 -> 0 / 15**.
Betrokken criteria U01/U04/U06 en FR-U04/U05 blijven open: deze hostregressies
sluiten niet de vereiste native/menselijke acceptatie. Datum, basis, branch,
taak en bewijs staan hierboven; centrale voortgangsbestanden zijn exclusief
van de manager en zijn door deze leaf niet gewijzigd.
