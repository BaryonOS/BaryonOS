# SR06/SR08: recoveryregressies en rode modale blocker

Status: **gedeeltelijk geblokkeerd, NIET mergeklaar**.
Assessment 30 oktober 2026, Europe/Amsterdam. Taak
TASK-UI-SR06-SR08-REGRESSIONS, queue
`e6e382cb7a02e0f36951842df04993277ee3a6cd`, blob
`60ff361c5c4553d37b112fc14fdb19894a9122a3`.
Afzonderlijke branch `codex/native-sr06-sr08`, werkboom
`/tmp/nieuw-os-sr06-sr08`, basis
`dca00a3ec9eb8eb5120177c4ec4145141c8cd3ee`.
Alleen `ui/native/tests/interaction.rs` en dit bewijsdocument gewijzigd.
Controller/renderer en frozen dca/70/hover/PR68 zijn niet gewijzigd.

## Echte blocker

`sr06_failed_save_blocks_modal_save_actions` reproduceert op de bestaande
conflictfixture: na geweigerde save is `can_save=false` en de editor-Save
terecht disabled. Na `request_back()` blijft SaveAndBack/Bewaren echter enabled
en activeerbaar in de semantiekboom. De test faalt expliciet met
`SR06 unavailable modal SaveAndBack remains advertised for fault 0`.
De volgende revoke-iteratie wordt door deze eerste assertion niet bereikt;
geen geslaagde modale revokeproef claimen.

Dit botst met UX-SR06: geen werkende Bewaren-claim waar de dienst weigert.
Bronoorzaken: `render.rs` schildert modal_save met letterlijk `true` en
publiceert SaveAndBack eveneens met letterlijk `true`; de modale Tab-lijst
in `controller.rs` bevat alle drie doelen zonder beschikbaarheidsfilter.
Alleen een controllerwijziging zou schildering en boom niet consistent maken.
Rendererwrites zijn expliciet uitgesloten. Daarom is alleen deze case
geblokkeerd en blijft haar reproducer **actief/falend**, niet ignored en
niet omgezet naar een verwachting van het foutieve gedrag.

Benodigde begrensde vervolgtoewijzing: renderer modal_save en SaveAndBack
aan bestaande `view.can_save` koppelen; controller modal-Tab dezelfde
beschikbaarheid laten volgen. Geen nieuwe policy/API/rechten, geen tokenwijziging.
Daarna dezelfde rode test plus pointer/focus/semantiekconsistentie en alle gates.
Dit voorstel is nog geen uitgevoerde fix of nieuwe schrijftoewijzing.

## Onafhankelijk groen

- SR06-editorcase hergebruikt conflict en capability-revoke. De draft blijft
  zichtbaar, `can_save=false`, Save is disabled/niet activeerbaar/niet gefocust,
  semantische Save wordt geweigerd. Beide foutiteraties werkelijk groen.
- SR08 verliest daadwerkelijk de COMMIT-reply in het bestaande hostharnas.
  Voor en na inputrebind en terugediten naar oorspronkelijke bytes blijft
  Uncertain zichtbaar; Save is disabled/niet activeerbaar/niet gefocust,
  oude generatie wordt StaleInput, huidige Save InvalidInput en directe Save
  Uncertain. Geen extra opslagrequest of COMMIT; precies één COMMIT, disk
  bevat werkelijk rev2/`Keep Old`, terwijl de lokale uitkomst onzeker blijft.
  Geen retrylabel wordt gepubliceerd. Deze test gebruikt de bestaande
  copy `Opslag onzeker. Je tekst blijft open.`, geen nieuwe doelcopy.

ReadOnly/openen en afzonderlijke IO-/Closed-modale matrix zijn nog niet
toegevoegd; die dekking wordt niet uit deze twee fixtures afgeleid.
Geen native of menselijke acceptatie.

## Uitgevoerde commando's

Bestaande Rust1.90.0-toolchain uit
`/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools/nieuw-os`;
`CARGO_HOME=$P/cargo`, `RUSTUP_HOME=$P/rustup`, `RUSTUP_AUTO_INSTALL=0`,
`PATH=$P/cargo/bin:$P/rustup/toolchains/1.90.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld:$PATH`,
`LD_LIBRARY_PATH=$P/host-cc/usr/lib/x86_64-linux-gnu`,
`CARGO_TARGET_DIR=/tmp/nieuw-os-sr06-sr08-target`.
Werkmap `/tmp/nieuw-os-sr06-sr08`; `cargo` hieronder is `cargo +1.90.0`.
Vooraf fmt zonder --check uitgevoerd, exit0.

| Exact commando | Exit | Wandtijd | Log/resultaat |
|---|---:|---:|---|
| `cargo test --offline --locked --manifest-path ui/native/Cargo.toml` | **101** | 0,844 s | `suite.log`: 25 groen, 1 echte modale fout |
| `cargo test --offline --locked --manifest-path ui/native/Cargo.toml -- --skip sr06_failed_save_blocks_modal_save_actions` | 0 | 0,378 s | `unblocked.log`: 25 groen, **1 expliciet uitgefilterd**, geen volledige groene suite |
| `cargo fmt --manifest-path ui/native/Cargo.toml -- --check` | 0 | 0,065 s | `fmt.log` |
| `cargo check --offline --locked --manifest-path ui/native/Cargo.toml --tests` | 0 | 1,617 s | `check.log` |
| `cargo clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings` | 0 | 0,560 s | `clippy.log` |
| `cargo build --offline --locked --manifest-path ui/native/Cargo.toml --target x86_64-unknown-none` | 0 | 1,729 s | `none.log`, alleen library-crosscompile |
| `git diff --check` | 0 | niet apart gerapporteerd | `diff.log` |

Eerdere gerichte `--test interaction sr06_` gaf exit101 met dezelfde echte
modale fout (`sr06-red.log`). Eigen authoringfout: scenario-invocaties waren
aanvankelijk binnen de macro geplaatst; compilatiefout bewaard in
`authoring-error.log`, daarna hersteld zonder productwijziging.
Eerste SR08-test verwachtte doelwoord “onbekend” terwijl bestaande copy
“onzeker” is; fout in testaanname, bewaard in `status-copy-assumption.log`.
Verwachting nu gebonden aan bestaande status_label en onzekerheid, geen
productpolicy aangepast om groen te worden.

Volledige logs:
`/home/simon/.copilot/session-state/1b87793863ad4dcc8f16a0bd10884654/files/sr06-sr08/`.

SHA-256:

```text
a48183d087c1ad8b563942c8d77282024eb1fbf893531caf56f6112e41aa83f2  ui/native/tests/interaction.rs
8fc4f7e5160a3df90e2c9daff992a9114430da6c0fb586652d1b88b42ffb37f0  suite.log
6b5eebefe5883c2e743671dbc537ebe42bc9d394b36ba35377c7d8b4793d5312  unblocked.log
26fa5995077d8d78fdc8c656c3c8eb7f015093fc4e271448a5a45450a18e9979  fmt.log
0ea89f0a63d850cabb48f940205c6fe967503eb876ecaa0656f54e6a4f321c07  check.log
1e9bee2f548b9d1d5126ca3d921e38344007acfcaccfef71efed9db4b437989d  clippy.log
8ef763572e81d82e497da43a1ba5cff89a883f5e20bc626fbc2dcbd3bc6ed760  none.log
145dfed81d6421ca84b960dafaac8e0e29753969a4dc227edc7e97a038129b33  diff.log
82a244f10f36f98bde179aee390d6db2e65f4a80ad2504921175861cd330cd1e  sr06-red.log
eb273122932d4cbada898832e4e2754f354dd38c9cf04b71aef6d16d474fe5ca  authoring-error.log
88d9ee2972d6fc67d764c45e29b9d7676d12c604562954fb76a5fc620650b097  status-copy-assumption.log
```

Geen VM, IPC/API/root/manifestwijziging, nieuwe provider of rechten.
Host-RAM-disk is geen native duurzaamheid. Geen native200%, screenreader,
menselijke controle, geïntegreerde image of volledige CI uitgevoerd.
Dit is een persistente **rode checkpoint**, geen merge-/reviewgoedkeuring.
Scorevoorstel aan centrale eigenaar **34 -> 34 /100; U 0 -> 0 /15**.
Nieuwe echte blocker verlaagt niet stilzwijgend bewijscriteria; betreffende
native criteria waren al open. Centrale assessment/status/progressbestanden
blijven exclusief van hun aangewezen eigenaar.
