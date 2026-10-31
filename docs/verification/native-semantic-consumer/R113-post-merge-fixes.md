# R113 — UI-herstel na post-merge review van #113

Status: alleen hostregressiebewijs. Deze notitie claimt niets over de native
compositor, QEMU of de assistive service; die acceptatiestappen blijven open.

## Bron en scope

- Reviewbasis: `e08fb0e` (merge van #113)
- Werkbranch: `claude/fix-113-review` (PR #118); in integratie 3 via
  `ccr-4bb5b18c-t9u3cv` naar main
- Gewijzigde runtime: `ui/native/src/controller.rs`
- Regressietests: `ui/native/tests/interaction.rs`. Drie nieuwe
  `scenario!`-tests en één helper, `assert_single_live_focus`, zijn
  toegevoegd. Geen bestaande test is gewijzigd of verwijderd.
- Geen Cargo-manifest, lockfile, dependency, `unsafe`, SDK-, kernel- of
  capabilitypad gewijzigd.
- De DHCP-fix uit dezelfde PR staat beschreven in
  `services/net-config/README.md`, niet in deze notitie.

## Bevindingen en herstel

1. **Major: BEGIN-fout bij `SaveAndBack` terwijl `can_save` waar blijft.**
   `activate(SaveAndBack)` schakelt naar het Editor-scherm vóórdat het
   `save()` aanroept. Bij een quotafout liet `failed()` de focus op
   `SaveAndBack` staan, maar die knop bestaat alleen in het sluitdialoog.
   Gevolg: geen gefocuste semantische node, Enter gaf `InvalidInput` en
   getypte tekens verdwenen stil. Herstel: als er niet wordt opgeslagen, het
   scherm niet `ConfirmClose` is en de focus op `SaveAndBack` staat, zet
   `failed()` de focus nu op `Save`. De bestaande regel gaat voor: als
   `can_save` onwaar is, gaat de focus naar `Document`.
2. **Minor: terug in alleen-lezenmodus focuste de uitgeschakelde New.**
   `request_back()` kiest nu dezelfde beginfocus als de constructor: `New`
   in ReadWrite-modus, anders `Refresh`.
3. **Minor: een lopend pointergebaar overleefde een semantische actie.**
   `activate_semantic` en `set_semantic_value` wissen nu `pressed` en
   `dragging` zodra de live semantische boom de actie heeft geaccepteerd.
   Dat sluit aan bij wat `request_back`, `resize`, `rebind_input` en
   `cancel_pointer` al deden. Een muisrelease kan daardoor niet meer het
   object openen dat na een semantische Refresh op de ingedrukte lijstindex
   terecht is gekomen, en een vastgehouden sleep kan geen selectie meer
   uitbreiden over vervangen tekst.

## Vóór het herstel (rood)

De drie tests zijn gedraaid tegen de ongewijzigde controller op `e08fb0e`:

```
cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml --test interaction r113
exit=101
test r113_read_only_back_focuses_enabled_refresh ... FAILED
test r113_semantic_action_cancels_pending_pointer_gesture ... FAILED
test r113_save_and_back_begin_failure_refocuses_editor_control ... FAILED
  r113_read_only_back...:      assert_eq!(focus, Refresh)        left: New
  r113_semantic_action...:     assert_ne!(contents, Some("zero")) left: Some("zero")
  r113_save_and_back...:       assert_ne!(focus, SaveAndBack)     left: SaveAndBack
test result: FAILED. 0 passed; 3 failed; 28 filtered out
```

Een aparte mutatiecontrole dekte de sleephelft van test 3. Met alle fixes
behalve het wissen in `set_semantic_value` faalde de test met
`stale drag changed the selection  left: 0  right: 13` (exit 101). Daarna is
de volledige fix teruggezet.

## Na het herstel (groen)

Alle onderstaande commando's draaiden met
`CARGO_TARGET_DIR=/home/simon/projects/target-fix113/ui`, tenzij het commando
een andere map noemt. Dit is de rapportage van de oorspronkelijke
uitvoerder; de herhaling in integratie 3 staat in `docs/PROJECT-STATE.md`.

| Commando | Exit |
| --- | --- |
| `cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml --test interaction r113` (3 geslaagd) | 0 |
| `cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml` (interaction 31 geslaagd, render_regressions 4 geslaagd) | 0 |
| `cargo +1.90.0 clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings` | 0 |
| `cargo +1.90.0 fmt --manifest-path ui/native/Cargo.toml -- --check` | 0 |
| `RUSTFLAGS="-C no-redzone=yes -C target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2" cargo +1.90.0 build --offline --locked --release --lib --target x86_64-unknown-none --manifest-path ui/native/Cargo.toml` | 0 |
| `CARGO_TARGET_DIR=/home/simon/projects/target-fix113/mat cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml` (20 + 2 geslaagd) | 0 |

## Niet bewezen

- Voor deze wijziging is geen native QEMU-run, compositorpresentatie of
  consumptie door de assistive service uitgevoerd.
- De `x86_64-unknown-none`-build laat alleen zien dat de bibliotheek voor dat
  target compileert, niet dat ze daar tijdens uitvoering correct werkt.
