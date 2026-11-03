# SPRINT-VELD-REVIEW — begrensde regressiefixes #157

**Implementatie:** reviewfixes vanaf bewijshead
`92f5972b81532d931df04ed17017881c215f6ede`;
productreferentie `f1998bdbd8fccd0b26eaf897116b1dd93cb7637e`.
Fixcommits `114053122e198b5b4ec62c86663ae5bed0f931e6` en
`b31173128d4c0004ada0bfef9de89de4265833a3`.
**Geaccepteerde runnerbron:** laatstgenoemde commit; fingerprint v3
`11d5637f00d4d91eda1e238762f1d1ba47fd03eee9c7b47723ed1bfb5a995b17`.
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
Geen kernel-/ABI-writes. Vervolgcommando
`python3 -B tools/test_window_session.py`: exit 0, 23 tegenproeven, 1,30 s;
log `veld-pr157-runner-followup.log`. Rust bleef sinds de gerichte hostgate
ongewijzigd.

## Testbewijs: nieuwe native deelacceptatie uitgevoerd

Sequentieel uitgevoerd op dezelfde bevroren runnerbron `b3117312`,
met eigen wegwerpimages per run en dezelfde geldige ontwikkelomgeving:

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

| Gate | Exit | Meetwaarde | Log |
|---|---|---|---|
| 100%-probebuild | 0 | 2,61 s wandtijd | `veld-pr157-native-scale1-attempt2.log` |
| 100%-runner | 0; ruwe QEMU-exit 33 | 61 fasen; 406,23 s runnerduur; 428,96 s wandtijd | hetzelfde log |
| 200%-probebuild | 0 | 2,12 s wandtijd | `veld-pr157-native-scale2-attempt1.log` |
| 200%-runner | 0; ruwe QEMU-exit 33 | 61 fasen; 727,20 s runnerduur; 771,49 s wandtijd | hetzelfde log |

Beide rapporten geven `passed`, `native_gate`, `notification_panel_gate`
en de overige toepasselijke gates als `true`. `native_scale200_gate`
is uitsluitend voor de 200%-run `true`. De nieuwe fase
`workspace-notice-mouse-isolated` behoudt het hele paneelbeeld, appmodellen,
focus en inputbevestigingen. De sluitfase vergelijkt nu alle pixels van
paneel plus schaduw met de echte reeds geverifieerde appachtergrond.
Alle bestaande appbody-, hoogcontrast-, verplaatsings-, vensterstatus-,
herstel-, clipboard- en schijforakels blijven behouden.
Geen oude groene gate herlabeld; geen productcodefix nodig gebleken.

[100%-rapport](native-scale1.json), [200%-rapport](native-scale2.json) en
[mislukte eerste poging](native-scale1-attempt1-failed.json) bewaren alle
runnergegevens behalve de omvangrijke `qmp_trace`-lijst.
`raw_report` bevat repository-relatief pad en SHA-256 van het originele
rapport; `qmp_trace_summary` bevat aantal en hash van de canonieke JSON-lijst
(`sort_keys=True`, `separators=(',', ':')`, UTF-8).
Rapporten binden bron, firmware, QEMU, fasebeelden, serial en schijforakel.
Geen gastnetwerk, hostdisk of productiedata gebruikt.

## Logcontrole en open integratie

[logs.sha256](logs.sha256) bevat de vijf ongewijzigde sessielogs: de
gerichte hostgate, de vervolg-runnertegenproeven, de mislukte native poging
en beide geslaagde native pogingen. Paden zijn relatief aan de
log-artifactroot, lokaal `$VELD_ENV/logs`. Met de logbundle:
`cd <log-artifactroot> && sha256sum -c <checkout>/docs/verification/veld-pr157-20261103/logs.sha256`.
Alle vijf hashes zijn lokaal gecontroleerd. De ruwe logs en runimages zijn
niet in Git aanwezig; een schone checkout alleen heeft dus niet de
volledige lokale artifactbundle. De compacte rapporten en hashes wel.

**BLOCKED-BY: General / centrale documenteigenaar**, volgens de
reviewthreadsnapshot na `b3117312`; niet gewijzigd of opgelost door Veld:

| Thread | Comment | Open punt |
|---|---|---|
| `PRRT_kwDOVZC91s6zOtDW` | 4457643730 | Statuslabels in `NATIVE-WINDOW-SESSION.md`. |
| `PRRT_kwDOVZC91s6zOtDh` | 4457643742 | Bewijs-/implementatielabels in `progress.json`. |
| `PRRT_kwDOVZC91s6zOtDq` | 4457643753 | Focusafhankelijke Enter-beschrijving in `PROJECT-STATE.md`. |
| `PRRT_kwDOVZC91s6zPYM1` | 4457929868 | Spelling in centrale handoffkop. |
| `PRRT_kwDOVZC91s6zPYNA` | 4457929883 | Native-status beperken tot Veld-07-deelacceptatie. |
| `PRRT_kwDOVZC91s6zPYNU` | 4457929923 | Afzonderlijke schaalresultaten/exits in centrale status. |

**Niet uitgevoerd op deze fixbron:** nieuwe volledige scenehostrun,
normale non-probebuilds, gezamenlijke main-integratie en CI.
Alleen de hierboven genoemde gerichte host-/probebuild-/VM-gates zijn
nieuw bewijs. Geen menselijke/fysieke/onafhankelijke audit-/releaseclaim.
Volledige Veld-acceptatie blijft open; features blijven gepauzeerd.
Geen centrale writes of scoreimport; scorebeheer blijft bij General.
