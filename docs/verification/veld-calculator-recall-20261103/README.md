# VELD-10: eerdere berekeningen opnieuw gebruiken

Status: **QUEUED-FOR-INTEGRATION**, geen OPEN-PR-LEASE en dus geen nieuwe PR.
Branch `veld/calculator-recall-20261103`, opnieuw opgehaalde mainbasis
`6d523964ab85638d4dd981bcdb19580d18646fab`.
VELD-09 blijft afzonderlijk bevroren op `35475ee6`, queued zonder PR-lease.
Deze branch bevat geen VELD-09-, VELD-08- of #157-import.

## Gebruikersinteractie en scope

Omhoog in het formuleveld haalt de vorige geslaagde berekening terug.
Nogmaals Omhoog gaat verder terug; Omlaag gaat vooruit en herstelt ten slotte
de exacte eigen onaffe formule, cursor en selectie. Geen rondloop.
De huidige appinstantie bewaart maximaal acht formules in eigen vluchtig
geheugen. Aansluitende dubbele formules komen niet dubbel in de lijst;
rekenfouten worden niet als geslaagde berekening toegevoegd.
Dit is een begrensde eerste implementatie, geen permanente productlimiet
of claim van persistente berekeningsgeschiedenis.

Terughalen berekent niets: het vorige antwoord blijft duidelijk verouderd.
Aanpassen maakt de teruggehaalde formule een nieuwe eigen invoer; Enter
rekent uitsluitend op uitdrukkelijk verzoek. C/Escape verlaten het bladeren.
Focusverlies bewaart de eigen onaffe invoer. Gewijzigde, herhaalde of
losgelaten pijltoetsen kunnen geen geschiedenis doorlopen; NumLock-uit
Keypad8/Keypad2 volgen dezelfde Omhoog/Omlaag-route.
De bestaande statusregel geeft de toetsbediening aan.

Exclusieve huidige writes: calculator `src/model.rs`, `src/render.rs`,
nieuwe `tests/recall.rs`, een nieuwe test in calculator `tests/chain.rs`
en dit checkpoint. Aanvullende runnerclaim: uitsluitend de opt-in
`--calculator-recall`-route in `tools/run_window_session.py` en bijbehorende
tegenproeven in `tools/test_window_session.py`.
Geen `scene.rs`/`init.rs`/`main.rs`, app-lifecycle, Settings,
algemene notification-/core-API, kernel, centralestatus of scorewrites.
Geen nieuwe capability, dependency, allocator, opslagrecht of app-ABI.

## Uitgevoerd op dit actieve broncheckpoint

In de bestaande ontwikkelomgeving, met
`CARGO_TARGET_DIR="$VELD_ENV/host-target"`:

```sh
cargo +1.90.0 fmt --manifest-path sdk/examples/calculator/Cargo.toml
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/calculator/Cargo.toml --test recall
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/calculator/Cargo.toml
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/calculator/Cargo.toml --all-targets -- -D warnings
python3 -B sdk/examples/calculator/build.py
python3 -B sdk/examples/calculator/build.py clippy
git diff --check
```

Laatste volledige reeks exit 0: 48 hosttests, waaronder zes nieuwe
recalltests en een nieuwe echte capability-/INP1-/SUR1-ketentest.
Een bestaande opt-in fixture zonder invoer levert geen native bewijs.
Pixels/damage gecontroleerd op 320x240@1, 320x484@1 en 640x608@2.
De ketentest controleert beide schalen, acht exacte key-/ACK-events en
nieuwe actuele frames, plus een uitdrukkelijke herevaluatie.
Suitewandtijd 5,42 s. Crossbuild/ELF-inspectie en target-Clippy ook exit 0.
Volledig log: `$VELD_ENV/logs/veld10-recall-host-target.log`.

Eerste ontwikkelpoging had een verkeerd geplaatste match-arm:
rustfmt stopte met exit 1, geen tests uitgevoerd. Bewaard in
`veld10-recall-targeted.log`. Na correctie vijf toen bestaande recalltests
en Clippy exit 0: `veld10-recall-targeted-attempt2.log`.

## Native deelacceptatie en bewijsgrens

De nieuwe gerichte VM-deelgate is geimplementeerd en uitgevoerd.
Zij gebruikt echte QMP-pijltoetsen, herstelt de oorspronkelijke draft,
controleert actuele/stale-resultaattellers en volledige native appbeelden.
Dat is geen herhaling van VELD-08 of de al geaccepteerde VELD-09-route.
Dit is een afzonderlijke interactiedeelgate, geen volledige OS-acceptatie.

```sh
python3 -B tools/test_window_session.py
export NOS_BUILD_DIR="$PWD/build/veld10-recall-20261103"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context"
python3 -B -u tools/build.py --kernel-feature native-window-session-probe
python3 -B -u tools/run_window_session.py --calculator-recall --timeout 600 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
```

De opt-in route kan uitsluitend `calculator_recall_gate=true` opleveren,
niet volledige `native_gate=true`. Default-/herstel-/opslaggate ongewijzigd.

Eerste VM-poging op `13f0a505`: build exit 0 / 2,89 s; runner exit 1 /
32,97 s. Run `window-28698ca01c2d44abbe91d424e2281349`, log
`veld10-recall-native-attempt1.log`. De eerste berekening was correct,
maar de buitengebiedcontrole vergeleek met een opname direct na F6, terwijl
de compositor zijn eigen focuschrome nog schilderde (58.390 pixels buiten
de appbody, x20..1252/y80..713). Geen appmodelwijziging of productfout
geclaimd. Runnerfix neemt de eerste complete berekening als native
buitengebiedbasis; vanaf daar moet iedere recall-/draftstap exact gelijk
blijven buiten de appbody. Geen pixels gemaskeerd, geen sleeps toegevoegd,
geen geslaagde gate voor de mislukte poging geclaimd.

Tweede poging op `ccf5617f`: build exit 0 / 2,17 s, runner exit 1 /
50,49 s; run `window-e0c3bfab5e144c958e7d72f74721d3a3`,
log `veld10-recall-native-attempt2.log`.
De recall-/draft- en eerste editstappen voldeden, maar daarna veranderden
1.869 pixels in Materiaals statusregel (x221..513/y581..593).
De bestaande Materiaalconsumer schildert zijn FocusLost-status pas na de
volledige achtergrondframeketen; oude READY-snapshots bleven tijdens die
paint zichtbaar. De brontrace bevestigt dat iedere Cancel in deze mainbasis
de Materiaalstatus ongeldig maakt. Dit is geen calculatorpaint.
De runner wacht nu vóór zijn baseline expliciet op de nieuwe Materiaal-
framegeneratie na F6 en de bijbehorende compositorcommit, naast de echte
calculatorcommit. Geen Materiaalcode gewijzigd of statusregel gemaskeerd.

Derde VM-poging op `9e5af2049c93d417c36777309e80fe257456d4e4`
geslaagd: build exit 0 / 2,26 s; runner exit 0 / 63,56 s, veertien fasen,
ruwe QEMU-exit 33 en normale cleanupmarkers.
`passed=true`, `calculator_recall_gate=true`, bewust `native_gate=false`.
Run `window-a44e2f969c164431b0e0b3ce9db95ccb`,
log `$VELD_ENV/logs/veld10-recall-native-attempt3.log`.
Bronfingerprint
`0541b0b0e66a6fb110e1d54a7ceae07aa3a73eb0dda755db9ca8d09a2e60d2e6`.
Product-Rust sinds `51e06fbf` ongewijzigd; beide vervolgfixes betroffen
uitsluitend concrete runner-observatiegrenzen.

`native.json` behoudt manifest, images, firmware, serial, frames en
raw-reportpad/hash. Alleen de omvangrijke QMP-trace is vervangen door aantal
entries en SHA-256 van canonieke JSON (`sort_keys=True`, separators `(',', ':')`).
`logs.sha256` bindt alle zeven ontwikkellogs, inclusief de twee mislukte
VM-pogingen. De ruwe artifacts staan in de daarin genoemde lokale buildrun,
niet als binaries in Git. Alle loghashes en `git diff --check` zijn exit 0.
`python3 -B tools/test_window_session.py`: exit 0, twaalf runnertegenproeven.

Niet uitgevoerd: gewijzigde-mainintegratie, CI, fysieke
hardware-, menselijke gebruiks-, onafhankelijke audit- en releasegates.
Geen punten toegekend; geen volledige product-/opslag-/herstelclaim.

Bij latere gezamenlijke integratie met VELD-09 moeten beide calculatorhunks
behouden blijven: undo-registratie van terughalen en beide sneltoetshints
vragen een gerichte integratiebeslissing en regressietest. Geen bewijs van
de ene branch als runtimebewijs van de andere gebruiken.
Geen PR zonder OPEN-PR-LEASE. Rollback: uitsluitend deze scopedelta reverten.
De milestone is bevroren; nieuwe productcode volgt uitsluitend op een
afzonderlijke opnieuw opgehaalde mainbasis.
