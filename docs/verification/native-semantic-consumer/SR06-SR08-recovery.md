# SR06/SR08: recoveryregressies en rode modale blocker

Status: **rode modale blocker hersteld na expliciete scopeuitbreiding;
hostgates groen, gerichte review/integratie/nativeacceptatie nog open**.
De onderstaande rode checkpoints blijven ongewijzigd historisch bewijs;
de actuele fix en hashbinding staan onder V4.
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

ReadOnly/openen en IO-editor zijn in de onderstaande v2 toegevoegd.
Een afzonderlijke IO-/Closed-modale matrix is nog niet toegevoegd;
die dekking wordt niet uit deze fixtures afgeleid.
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

## V2: onafhankelijk ReadOnly/IO-editorwerk voortgezet

Na rode checkpoint `0b0271c63697ce52eacba5bef91ae1c99ffd6697` is binnen
dezelfde testscope een vierde test toegevoegd. ReadOnly opent bestaande
inhoud, weigert semantische Save en Document-set-value zonder nieuwe
opslagrequests. IO tijdens de save behoudt dirty tekst, Failed-status,
geen saving en originele disk/revision1. De daadwerkelijk aangeboden
Save-beschikbaarheid volgt `can_save`; een disabled actie is niet gefocust.
ReadOnly doet geen COMMIT; IO mag één COMMIT **proberen** dat de dienst weigert.
Een eerste eigen testaanname “ook IO geen COMMIT-request” was onjuist;
`io-commit-assumption.log` bewaart die fout. De service-/controllerpolicy
is niet gewijzigd. Geen claim dat een IO-poging automatisch veilig retrybaar is.

De eerdere commandotabel/hashes blijven historische checkpoint0b0271.
Actuele v2-testbron:
`9b21190f1c8125b3c556bb34a23fdc20baee04a056535446c19ebb5a30016909`.
Zelfde exacte commandoprefix/omgeving als hierboven:

| Exact commando | Exit | Tijd | Actuele uitvoer |
|---|---:|---:|---|
| `cargo test --offline --locked --manifest-path ui/native/Cargo.toml` | **101** | 0,729 s | `suite-v2.log`: 26 groen, zelfde 1 actieve modale blocker |
| `cargo test --offline --locked --manifest-path ui/native/Cargo.toml -- --skip sr06_failed_save_blocks_modal_save_actions` | 0 | 0,368 s | `unblocked-v2.log`: 26 groen, 1 expliciet uitgefilterd |
| `cargo fmt --manifest-path ui/native/Cargo.toml -- --check` | 0 | 0,063 s | `fmt-v2.log` |
| `cargo clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings` | 0 | 0,250 s | `clippy-v2.log` |
| `cargo build --offline --locked --manifest-path ui/native/Cargo.toml --target x86_64-unknown-none` | 0 | 0,057 s | `none-v2.log`, uitvoering exit0, cachehergebruik is geen runtimebewijs |

```text
8c0116b5febdf711043d4036228339658608afa71912cdaf8be78ae7ecba1855  suite-v2.log
78192056b38c47dc94a597eb0ee31b7556687bddd976af868de254a63587e818  unblocked-v2.log
156eccae11d6005c3e059dda8a9aad37d2f18a03cf387b481cc4f6cead7262ae  fmt-v2.log
03cb5e8e97c7cc56e71a81ce5a2a4abceb53c1ded7992f5c28910b8744bc2524  clippy-v2.log
2585c9a22accdb18fc58a9f21283521f2b85a29a821f66fb5dac0b1f81ed0e7b  none-v2.log
```

Rendereruitbreiding is aangevraagd, nog niet toegekend bij deze v2.
Er zijn nog steeds geen productbronwrites of native/menselijke punten.

## V3: beide modale fouten werkelijk bereikt

De modale test verzamelt nu de beschikbaarheidsuitkomst voor conflict én
revoke voordat de eindassertie faalt. Op beide gevallen is
`[disabled_after_conflict, disabled_after_revoke] = [false, false]` in plaats
van `[true, true]`. Hiermee is de eerdere bereikbeperking opgeheven, niet de
verwachte bescherming versoepeld. De actieve test blijft rood. Een eigen
Clippy-waarschuwing over indexeren is hersteld naar `iter_mut().enumerate()`;
het eerdere lintlog blijft `both-modal-clippy.log`.

Actuele testbron SHA-256:
`43c660e13ca2544a1c3d84981a40e37aaadcf26c3af6a5b4d48a51db5774a3f0`.
Zelfde exacte commandoprefix/omgeving:

| Exact commando | Exit | Tijd | Actuele uitvoer |
|---|---:|---:|---|
| `cargo test --offline --locked --manifest-path ui/native/Cargo.toml` | **101** | 0,687 s | `suite-v3.log`: 26 groen, 1 echte blocker met beide foutgevallen |
| `cargo test --offline --locked --manifest-path ui/native/Cargo.toml -- --skip sr06_failed_save_blocks_modal_save_actions` | 0 | 0,358 s | `unblocked-v3.log`: 26 groen, 1 uitgefilterd |
| `cargo fmt --manifest-path ui/native/Cargo.toml -- --check` | 0 | 0,061 s | `fmt-v3.log` |
| `cargo check --offline --locked --manifest-path ui/native/Cargo.toml --tests` | 0 | 0,145 s | `check-v3.log` |
| `cargo clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings` | 0 | 0,263 s | `clippy-v3.log` |
| `cargo build --offline --locked --manifest-path ui/native/Cargo.toml --target x86_64-unknown-none` | 0 | 0,053 s | `none-v3.log`, geen runtimebewijs |

```text
a79f65fc2935c07b876974b6c248cd7acfb1196e747f26cc0dc7d1cf4c69e7e9  suite-v3.log
029dd9729181577d6e1e620f92ffa7cdd0279c58cb1cf467cee72f6d307fdd4f  unblocked-v3.log
cc0bb653ab0d1db422e3aae8ab06f89f128cd6a5b5fa0219226de01a3a19d36b  fmt-v3.log
0ae0b17a664e277b01a8baaee648a228007efc0b6ae1b1373a87b56e318909ca  check-v3.log
d9a822a4b05a724b05dd28bdfa50985f6fcd45409576541f7541d624ae6c6f4c  clippy-v3.log
d9549f427a222d626d6199921fe50cd7a11227c6920fac9f27a13f182ec21a8d  none-v3.log
```

Geen renderer- of controllerwrites. Nog geen uitbreiding ontvangen bij
deze v3-checkpoint; oorspronkelijke scope en rode status blijven gelden.

## V4: expliciet vrijgegeven modale consistentiefix

Scopeuitbreiding queue `dea97f9b57c6fda9549a48072119663fe155a563`, blob
`53ac59d53fa7daece42b07925d6ecffe852012f6`, ontvangen vóór productwrites.
Rode bron `7dc64b3a5a58ce95030f5edd2bb5c6eb184efc57` blijft ancestor.
Vier toegestane bestanden: bestaande tests en deze note, plus modal-only
renderer en conditionele controllerfocus/inputconsistentie.

`render.rs` gebruikt nu bestaande `view.can_save` voor zowel het schilderen
van Bewaren als de enabled/action-state van SaveAndBack. Geen hover-, token-,
algemene layout- of nieuwe beleidswijziging. Controller modal-Tab/ShiftTab
beperkt de cyclus tot KeepEditing/Discard wanneer Save niet beschikbaar is.
Pointer-hit-testing sluit dezelfde disabled knop uit. Keyboard en semantische
activatie hergebruiken de ongewijzigde actuele-boom/action/generationguards.
Opslagguards en capabilities blijven ongewijzigd.

De uitgebreide conflict/revoke-test controleert nu naast enabled/action:
werkelijke disabled pixel (`LINE`), beide Tab-richtingen zonder disabled focus,
pointerpress/release op Bewaren zonder activatie of focusverlies, oude generatie
geweigerd, KeepEditing/Discard beide beschikbaar en Enter op KeepEditing
behoudt de draft zonder nieuwe opslagrequests. Beide oorspronkelijke
reprogevallen zijn groen; er wordt geen case meer uitgefilterd.

Een aanvullende echte rode case toonde een nauw verbonden focusfout:
de grant wordt pas **na** het openen van de dialoog ingetrokken; SaveAndBack
was toen terecht beschikbaar, maar de daadwerkelijk geweigerde save liet
focus op de inmiddels afwezige SaveAndBack staan in Editor.
`focus-red.log`, exit101: SaveAndBack in plaats van Document.
De bestaande `failed`-focusherstelroute verplaatst nu ook Save/SaveAndBack
naar Document als er geen save meer loopt en `can_save=false`. Geen fout
wordt verborgen; de oorspronkelijke storagefout blijft geretourneerd.

### Actuele volledige gates

Zelfde exacte toolchain/omgeving/target/logmap. Dit zijn controles op de
werkelijk gecombineerde fix/testbron met onderstaande hashes; oude groene
of rode resultaten worden niet naar de nieuwe bron herlabeld.

| Exact commando (`cargo` = `cargo +1.90.0`) | Exit | Tijd | Log |
|---|---:|---:|---|
| `cargo test --offline --locked --manifest-path ui/native/Cargo.toml --test interaction sr06_` | 0 | 0,430 s | `fix-targeted-v2.log`, 4/4 |
| `cargo test --offline --locked --manifest-path ui/native/Cargo.toml` | 0 | 0,552 s | `fix-suite.log`, **28/28**, geen skip/ignore |
| `cargo fmt --manifest-path ui/native/Cargo.toml -- --check` | 0 | 0,060 s | `fix-fmt.log` |
| `cargo check --offline --locked --manifest-path ui/native/Cargo.toml --tests` | 0 | 0,240 s | `fix-check.log` |
| `cargo clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings` | 0 | 0,420 s | `fix-clippy.log` |
| `cargo build --offline --locked --manifest-path ui/native/Cargo.toml --target x86_64-unknown-none` | 0 | 0,198 s | `fix-none.log`, alleen crosscompile |
| `git diff --check` | 0 | 0,007 s | `fix-diff.log` |

SHA-256 van exacte geteste product-/testbron en logs:

```text
4210c82fd2436f22ec520f0eed8d110fc59af9f1ae8b35b6da84d6a4f1a8615b  ui/native/src/controller.rs
898366f8ea3fe9ad965915c78148a320773a05df8ea0182e00bacfff9b977d16  ui/native/src/render.rs
77c42c0466e87fc781ee172e68ba9d9803c6c9ea7ef0f61a3f1d10d094cc8980  ui/native/tests/interaction.rs
05c7a04e16e590d950628250abc1901b7584d026aeaff4c954031db09f846c57  fix-targeted-v2.log
403cc58dfcb16b64b807036c0d6ccc3be1d0076d90eb28fa1c1ebcbde009308b  fix-suite.log
a0caceb5e4af72c716a4ad0b514482308f3e04935f852d3f933915ff7ba9bbc2  fix-fmt.log
54beadcd3422d92e117cd5ad6777c1fb3309d5d934207fa451f5d0d2fec8bdae  fix-check.log
a3b98d50032906d640fb236d2d9eb1d4de701aa56c42fa7392a881ddd31d36e9  fix-clippy.log
a6f8dca32c4df9c12f970a437b214bf0f09cbe64afd65fcec9d6c75c10fa9293  fix-none.log
aaaec055204f4742ef3b8130164dcc63e629905a1d9cf13f2c05b73a6b6bcd87  fix-diff.log
b08514502e211c952b5bf4ad5fc8f4e30a322d93b6fa968bc747529fb5829972  focus-red.log
```

Bronfreeze voor begrensde7f/Appsreview; hun uitkomst is hier nog niet
verondersteld. Geen merges uitgevoerd. De aparte integratorpauze geldt nog
onafhankelijk van deze domeinfix. Frozen dca/70/hoverheads zijn niet gewijzigd.
**Niet uitgevoerd:** VM, externe nativeconsumer/screenreader, menselijke
acceptatie, koude opslagproef of gezamenlijke image-CI. Nieuwe assessment:
34 -> 34 /100, U0 -> U0 /15. Alleen de hostmatige defect/recoverygrens is
hersteld; geen native criterium gesloten.
