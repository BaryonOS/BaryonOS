# Veld-07 — echt systeemmeldingenpaneel

**Historisch runnerresultaat; sluit- en muisisolatiebewijs ontoereikend.**
Productbron `f1998bdbd8fccd0b26eaf897116b1dd93cb7637e`, basis
`ea0cd6d19235c5f8575f21d527e5bcc8526056ee`, branch
`simonvanieperen-veld-hc-20261103`. Alleen shell-/fixture-/runner-/CI-writes;
General-/Settingsbestanden blijven ongemoeid.

Review #157 vond dat de oude `notice-closed`-wait alleen de bovenbalk toetste
en geen klik boven een echte app-body deed. De opgeslagen screenshots bij
die sluitfase bevatten op beide schalen nog gedeeltelijke paneelpixels.
De toen groene runner is daarom geen bewijs van volledige paneelverwijdering
of muisisolatie. Rapporten/hashes blijven ongewijzigd historisch bewijs;
de sterkere regressiegate krijgt [een afzonderlijk receipt](../veld-pr157-20261103/README.md).

Het paneel toont de huidige echte compositor-klembordstatus, geen
verzonnen appmelding. Alt+N en zichtbare bovenbalkactie; focus,
Enter/Delete/Escape en echte dismiss-/sluitknoppen. Dismiss wist uitsluitend
presentatie. Exclusieve overlayinput, geen clickthrough/stale releases.
Nieuwe statusovergang kan opnieuw verschijnen; dezelfde statuspoll niet.
Geen algemene appnotificatie-API, historie of persistentie.
[ADR-VELD-006](../../DECISIONS.md).

## Hostbewijs

Na de bestaande geisoleerde `veld-env/env.sh`:

```sh
export CARGO_TARGET_DIR="$VELD_ENV/host-target"
cargo +1.90.0 fmt --manifest-path sdk/examples/window-session/Cargo.toml -- --check
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml --all-targets -- -D warnings
python3 -B tools/test_window_session.py
```

Elk van de vier bovenstaande commando's eindigde met exitstatus 0.
Het gezamenlijke log is `veld-notice-final-host.log`, met SHA-256 in
[logs.sha256](logs.sha256). De paden in dat manifest zijn relatief aan de
log-artifactroot: lokaal `$VELD_ENV/logs`. Met de geleverde logbundle:
`cd <log-artifactroot> && sha256sum -c <checkout>/docs/verification/veld-notice-20261103/logs.sha256`.
De ruwe logs zijn sessie-artifacts en niet in een schone checkout aanwezig.

Historische hostdekking op f1998bdb:
`all_real_notice_states_keyboard_focus_and_dismissal_repaint_every_pixel`
toetste alle statusrollen, focus en volledige incremental/referencepixels op
vijf displays, maar geen muisroute.
`mouse_affordances_and_input_cancellation_cannot_click_or_type_through`
toetste input op alleen 1280x720 en had toen geen pixelasserties.
`panel_controls_labels_and_modifier_scope_are_bounded_on_every_display`
toetste grenzen/modifiers op vier displays, zonder 1920x1080@1 en zonder
pixelasserties. De runner had 19 tegenproeven.

## Native uitgevoerd

Bevroren bronf1998bdb, fingerprint v3
`94c3b156c01013542a50c591d605eca28b574bc0ab039f6a04caddb31b661c76`.
Na checkpoint, sequentieel uitgevoerd:

```sh
export NOS_BUILD_DIR="$PWD/build/veld-notice-20261103/scale1"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context"
python3 -B -u tools/build.py --kernel-feature native-window-session-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 1 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
export NOS_BUILD_DIR="$PWD/build/veld-notice-20261103/scale2"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context-hidpi"
python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi-probe
python3 -B -u tools/run_window_session.py --clipboard --workspace --scale 2 --timeout 1800 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
```

Daarna normal1/normal2-buildmappen en dezelfde respectieve targetmappen:
`python3 -B -u tools/build.py --kernel-feature native-window-session` en
`python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi`.
De twee probe-builds en twee runners eindigden elk met exitstatus 0.
Bij 100%: 60 fasen, 415,49 s; runnerwandtijd 438,63 s.
Bij 200%: 60 fasen, 761,61 s; runnerwandtijd 806,02 s. Beide ruwe QEMU-exit33.
De toenmalige runner markeerde alle toepasselijke gates true, maar zijn
meldingsgate had de hierboven beschreven bewijsleemten.
Beide normale builds exit0,2,84 s en2,50 s; compilatie, geen extra runtimeclaim.
60 fasen per schaal; vier extra fasen: echte gekopieerde melding,
dismiss/lege toestand, muisopenen en muissluiten. Het hele ondoorzichtige
paneelinterieur werd bij openen vergeleken; bij sluiten alleen de bovenbalk.
De oorspronkelijke hele appbodyorakels blijven ongemaskerd; de latere echte
numerieke paste moet clipboarddata na dismiss behouden.
Ook tekst/F6/clear-chord tijdens modaliteit mag geen appinput of clear geven.
Alle oorspronkelijke isolatie-/HC-/venster-/herstel-/schijfgates behouden.

[100%-rapport](native-scale1.json), [200%-rapport](native-scale2.json) en
[loghashes](logs.sha256) binden bron, firmware, pixels, fasen en schijf.
De compacte rapporten behouden alle runnergegevens behalve de omvangrijke
QMP-tracelijst; `raw_report` bewaart pad/hash van het volledige oorspronkelijke
rapport, `qmp_trace_summary` aantal/hash van die lijst.
Geen gastnetwerk, hostdisk of productiedata gebruikt.
Geen nieuwe CI-, fysieke of menselijke acceptatie.

## Integratiepauze

Tijdens de al gestarte200%-run gaf General Simons nieuwe opdracht door:
nieuwe features pauzeren voor een seriele integratiesprint.
Productbronf1998bdb was reeds gepusht en de werkboom was schoon.
Geen nieuwe feature-, merge- of mainwrites uitgevoerd; alleen de bestaande
acceptatierun/normale build afgerond en dit bewijs vastgelegd.
HC-, move- en notification-slices en expliciete General-filevrijgave zijn
overgedragen. Geen score- of inhoudsimport; de volledige Veld-opdracht is
hiermee niet afgerond en blijft gedurende integratie gepauzeerd.
Score **41 -> 41**; complete gebruikersomgeving blijft open.
