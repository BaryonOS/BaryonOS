# SPRINT-VELD-REVIEW — begrensde regressiefixes #157

**Implementatie:** reviewfixes vanaf bewijshead
`92f5972b81532d931df04ed17017881c215f6ede`;
productreferentie `f1998bdbd8fccd0b26eaf897116b1dd93cb7637e`.
Geen nieuwe feature, main-merge, app-API of productinterfacewijziging.
Alleen `tools/run_window_session.py`, bijbehorende tegenproeven,
`sdk/examples/window-session/tests/notifications.rs` en eigen receipts.
Geen centrale README/progress/PROJECT-STATE/WORK-QUEUE/DECISIONS-writes.

## Bevindingen en afbakening

| Reviewcomment | Actie |
|---|---|
| 4457643877, app-GEN-wait | Verwijderd. Werkelijke compositorinputbevestiging plus paneelpixels bepalen de shell-wait. De claim dat GEN nooit kon stijgen is te absoluut: de bestaande Resync annuleert appinput en invalideert de controller. Dat maakt GEN nog geen geschikte shell-wachtvoorwaarde. |
| 4457643865, sluitpixels | Volledige paneel-plus-schaduwregio moet overeenkomen met de echte vooraf geverifieerde opname; header met de shellfixture. Geen vergelijking van echte apppixels met lege hostbacking. |
| 4457643893, muisisolatie | Echte niet-activerende paneelklik boven de calculatorbody, bestaande KEYS/INPUT_ACKS-tellers en model-/focusbehoud; extra native fase. Openings-Resync wordt eerst bevestigd en als enige niet-toetsinput toegestaan; de volgende echte paste controleert op vertraagde input. Geen MICE-protocol toegevoegd. |
| 4457643856, muisdamage | Eerste volledige paint en na iedere muisgestuurde overgang incremental/referencepixelvergelijking, op vijf displays. |
| 4457643822/4457643830/4457643846, receipts | Relatieve log-artifactroot, gescheiden status/meetwaarden, commando-exits en precieze historische hostdekking. |
| Centrale status-/score-/besluitdocumentatie | Bij General gemeld; geen write-eigenaarschap overgenomen. |

Op beide oude sluitcaptures bleven werkelijk gedeeltelijke paneelpixels
staan toen alleen de header-wait groen werd. Die oude 60-fasengates worden
niet als bewijs voor deze sterkere runner gebruikt.
De nieuwe tegenproef weigert ook een fout in de schaduw of het midden van
de onderliggende app, en gebruikt bewust verschillende host-/appkleuren.

## Testbewijs: host uitgevoerd

Na de bestaande geisoleerde `veld-env/env.sh`:

```sh
export CARGO_TARGET_DIR="$VELD_ENV/host-target"
cargo +1.90.0 fmt --manifest-path sdk/examples/window-session/Cargo.toml
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml --test notifications
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml --all-targets -- -D warnings
python3 -B tools/test_window_session.py
```

Elk commando exit 0. De drie notification-tests slagen; de muis- en
keyboardpixelproeven gebruiken nu ieder vijf displays. De bounds/modifiertest
blijft een afzonderlijke proef op vier displays, zonder pixelclaim.
Runner: 21 tegenproeven. Log `veld-pr157-targeted-host.log` onder
`$VELD_ENV/logs`; hashes worden bij native archivering vastgelegd.
Geen nieuwe volledige scenehostrun geclaimd.

## Mislukte nieuwe native poging; acceptatie nog open

Runnerbron `114053122e198b5b4ec62c86663ae5bed0f931e6`:
100%-probebuild exit 0 (1,98 s), runner exit 1 (155,29 s).
Runnerduur 131,33 s; fout direct na `workspace-notice-copied`:
`IndexError: list index out of range`. De runner las een niet-gepubliceerd
`WINDOW:<role>:MICE`-veld. De apps houden intern wel muisgebeurtenissen bij,
maar de bestaande kernelprobe publiceert alleen KEYS en INPUT_ACKS.
Dit is een runnerfout, geen groen native resultaat of productregressieclaim.
Log `veld-pr157-native-scale1-attempt1.log`; ruwe uitkomst:
`build/veld-pr157-20261103/scale1/run/window-2bcdc528cc42446cb5ef3942acf28e72/result.json`.

Vervolgfix gebruikt alleen bestaande KEYS/INPUT_ACKS: precies een
openingsannulering per app, eventueel een reeds verwerkte Alt-press voor
Materiaal; daarna geen paneelinput naar apps. Extra tegenproeven weigeren
ontbrekende velden, extra afleveringen en een ontbrekende annulering.
Geen kernel-/ABI-writes. Vervolgcommand
`python3 -B tools/test_window_session.py`: exit 0, 23 tegenproeven, 1,30 s;
log `veld-pr157-runner-followup.log`. Rust bleef sinds de gerichte hostgate
ongewijzigd. Nieuwe native-uitkomsten nog open.

Verwacht 61 fasen per schaal, inclusief de nieuwe overlapklik.
Na dit checkpoint, op dezelfde bevroren runnerbron:

```sh
export NOS_BUILD_DIR="$PWD/build/veld-pr157-20261103/scale1"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context"
python3 -B -u tools/build.py --kernel-feature native-window-session-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 1 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
export NOS_BUILD_DIR="$PWD/build/veld-pr157-20261103/scale2"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context-hidpi"
python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 2 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
```

**Openstaand/onbekend:** nieuwe native uitkomsten, actuele gezamenlijke
main-integratie en CI. Geen oude groene gate herlabeld.
Geen menselijke/fysieke/releaseclaim. Scorebeheer blijft bij General.
