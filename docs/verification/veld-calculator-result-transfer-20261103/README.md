# VELD-11: rekenresultaat naar document overnemen

> **Status 5 november 2026 (integrator):** historisch branchbewijs. De functie is
> samen met recall (#184), undo/redo (#186) en resultaatoverdracht (#192) geïntegreerd
> via PR #209. Het native bewijs hieronder hoort bij de bronboom van de losse branch.
> De native gate op de samengevoegde boom is **onbekend** (niet uitgevoerd). De
> gecombineerde semantiek (recall als één undo-stap, redo behoudt de redo-tak, één
> statusregel) staat in de PR-beschrijving van #209 en in `tests/combined.rs`.

Status (3 november 2026): **OPEN PR #192**, in review en nog niet
geïntegreerd. De eerdere fase **QUEUED-FOR-INTEGRATION** en eventuele
OPEN-PR-LEASE betreffen de werkorganisatie, niet het actuele bestaan
van deze PR.
Branch `veld/calculator-result-transfer-20261103`, oorspronkelijke
mainbasis `681f1137e411d6c6fe536bd2bf499c27b34832be`.
Geen import van VELD-09/10/#157; een volgende zelfstandige PR vereist
opnieuw een passende OPEN-PR-LEASE.
De mainintegratie van deze onafhankelijke calculatorhunks vraagt later
behoud van de afzonderlijke undo-/recallwijzigingen en gerichte regressies.

## Complete gebruikersinteractie

Een actueel resultaat kan met een linkermuisklik of Ctrl+R geselecteerd
worden. De selectie is zichtbaar als omlijning en tekstuitleg. Ctrl+C
kopieert precies de getoonde decimale waarde, niet stilzwijgend de formule;
Ctrl+V in een document gebruikt de bestaande klembordketen.
Afronding, negatieve waarden en de Nederlandse decimale komma blijven
gelijk aan de weergegeven waarde. Kopieren rekent niet opnieuw.

Een oude, foutieve of niet-bestaande uitkomst kan niet geselecteerd worden.
Bewerken, formuliernavigatie, nieuwe knoppress of focusverlies heft
resultaatselectie op. Een achtergebleven knoprelease kan niets invoegen.
Plakken terwijl het niet-bewerkbare resultaat geselecteerd is wordt
expliciet geweigerd met bestaande clipboardfeedback; de formule blijft staan.
Ctrl+A selecteert weer de formule. Herhaalde/losgelaten/gecombineerde
Ctrl+R-events selecteren niet onbedoeld.

Geen nieuwe capability, grant, netwerk, opslagrecht, app-ABI, dependency
of algemene notification-/clipboard-API. Alleen actuele expliciete selectie
via de bestaande getypeerde CLP-/INP1-route. De broker blijft de echte
Ctrl+C-intentie, focus, epoch, sequence en request afdwingen.
De gewone selectie heeft geen resultaat-/formulefallback.

## Schrijfclaim

Calculator `src/{app,model,render}.rs`, nieuwe `tests/result_transfer.rs`,
een gerichte nieuwe ketentest en ondersteuning voor copy in de bestaande
uitsluitend hosttest-klembordtegenpartij in calculator `tests/chain.rs`.
Opt-in `--calculator-result` in `tools/run_window_session.py` met eigen
tegenproeven in `tools/test_window_session.py`, plus dit bewijs.
Geen Materiaal-, `scene.rs`/`init.rs`/`main.rs`-, kernel-, centrale status-,
score- of andere workstreamwrites.

## Uitgevoerd voor native checkpoint

Met bestaande `veld-env/env.sh`, Rust 1.90.0, offline/locked en
`CARGO_TARGET_DIR="$VELD_ENV/host-target"`:

```sh
cargo +1.90.0 fmt --manifest-path sdk/examples/calculator/Cargo.toml
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/calculator/Cargo.toml --test result_transfer
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/calculator/Cargo.toml
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/calculator/Cargo.toml --all-targets -- -D warnings
python3 -B sdk/examples/calculator/build.py
python3 -B sdk/examples/calculator/build.py clippy
python3 -B tools/test_window_session.py
git diff --check
```

Laatste volledige Rustreeks exit 0: 47 hosttests, vijf nieuwe selectietests
en een nieuwe daadwerkelijke capability-/INP1-/SUR1-ketentest.
Een bestaande opt-in fixture zonder invoer levert geen native bewijs.
De nieuwe ketentest controleert muis/shortcut op beide schalen, precies
`3,5` in de host-CLP-request, behoud van formule/evaluatieteller en het
weigeren van stale-resultaatfallback. De hosttegenpartij is geen native
broker; daarvoor is de aparte VM-deelgate nodig.
Alle gewijzigde pixels versus retained damage zijn gecontroleerd op
320x240@1, 320x484@1 en 640x608@2.
Suitewandtijd 5,51 s; Clippy, crossbuild/ELF-inspectie en target-Clippy
exit 0. Log `$VELD_ENV/logs/veld11-result-host-target.log`.

Eerste nieuwe pixeltest vond ontbrekende formule-cursordamage bij
resultaatselectie: exit 101, vier van vijf geslaagd.
`veld11-result-targeted-attempt1.log`. Na fix vijf van vijf exit 0:
`veld11-result-targeted-attempt2.log`.
De eerste volledige suite vond de aanvullende statusrepaint bij wisseling
van current: exit 101 in de bestaande damageproef, Clippy niet bereikt.
`veld11-result-host-attempt1.log`. Beide fouten zijn in bovengenoemde
definitieve volledige reeks hersteld; geen oude groene run hergebruikt.

## Uitgevoerd: gerichte native overdracht

```sh
export NOS_BUILD_DIR="$PWD/build/veld11-result-20261103"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context"
python3 -B -u tools/build.py --kernel-feature native-window-session-probe
python3 -B -u tools/run_window_session.py --calculator-result --timeout 600 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
```

De nieuwe route berekent werkelijk 7/2, selecteert met Ctrl+R, kopieert en
plakt exact `3,5` in Materiaal. Daarna 1/3, muisselectie en exact
`0,333333333` in de documentdraft. Save-ACK en normale cleanup volgen
zonder Materiaalcode te wijzigen. Het volledig gecommitte appbeeld moet
bij beide selecties veranderen; model/formule/evaluatieteller blijven correct.
Deze route kan uitsluitend `calculator_result_gate=true`, niet volledige
`native_gate=true`, opleveren. Default-/herstel-/opslaggate ongewijzigd.

Op bron `9c2a994ad8f30500e54ac228e1de0cceff267e16`:
build exit 0 / 2,91 s; runner exit 0 / 121,70 s, twaalf fasen,
ruwe QEMU-exit 33 en normale cleanupmarkers.
`passed=true`, `calculator_result_gate=true`, bewust `native_gate=false`.
Run `window-b79afae6ff7644c287c0dc7a9ce5893b`, log
`$VELD_ENV/logs/veld11-result-native-attempt1.log`.
Bronfingerprint:
`74eb21b6aafb03f16c96f8196468e43453205d7bb6115a256bcc114d8d90fe53`.
Beide waarden zijn werkelijk in de Materiaaldraft aangekomen; geen
hostinvoeging, testbroker of nieuwe apprechten in deze VM-keten.

`native.json` behoudt manifest, images, firmware, serial, frames en
raw-reportpad/hash. Alleen de omvangrijke QMP-trace is vervangen door aantal
entries en SHA-256 van canonieke JSON (`sort_keys=True`, separators `(',', ':')`).
`logs.sha256` bindt alle zes ontwikkellogs, inclusief beide rode hostproeven.
De ruwe artifacts staan in de daarin genoemde lokale buildrun, niet in Git.
Alle loghashes en `git diff --check` zijn exit 0.
`python3 -B tools/test_window_session.py`: exit 0, twaalf runnertegenproeven.

Niet uitgevoerd: gewijzigde-mainintegratie, CI, fysieke
hardware-, menselijke gebruiks-, onafhankelijke audit- en releasegates.
Geen scoreverhoging; geen volwaardige opslag-/herstel-/releaseacceptatie.
Rollback: uitsluitend deze scopedelta reverten.
De milestone is bevroren; nieuwe code volgt uitsluitend op een aparte
opnieuw opgehaalde mainbasis. VELD-09/10 blijven zelfstandige brongebonden
opleveringen, niet automatisch meegeaccepteerd.
