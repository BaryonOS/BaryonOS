# VELD-08 — vensterwisselaar met echte appminiaturen

## Taakcontract en bron

Nieuwe kleine branch `veld/next-shell-20261103`, uitsluitend vanaf main
`78ff7c03b015ad16fe665ae7df9fe239d078e08d`. Geen import uit #157 of andere
Veldbranches. #157 is hiermee niet gesupersedeerd: eerdere hoogcontrast-,
menu-, verplaatsings-, meldingencode en bewijs blijven afzonderlijk.
General blijft enige main-/cleanupintegrator en centrale status-/scorewriter.

**Doel:** Alt+Tab toont de twee werkelijk geopende vensters met actuele
miniaturen uit het reeds door de compositor gevalideerde eigen appbacking.
Tab of pijlen kiezen; Alt loslaten of Enter bevestigt; Escape of buitenklik
annuleert. De muis kan een kaart kiezen. Focus verandert pas bij bevestigen.
Inputverlies sluit de wisselaar zonder focuswijziging. De laatste Alt-release,
herhaalde toetsen en muisrelease na sluiten worden niet doorgestuurd.

**Invarianten:** geen nieuwe capability, app-/proces-ABI, opslagrecht,
appsurface, epoch, venstergrootte of netwerktoegang. Geen algemene
notificatie-API, nieuwe app-launcher of General/Maturity/Driver/Platformwerk.
Alleen twee bestaande vensters; geen fictieve apps. Geen animatie of blur:
deze stap heeft geen tijdsafhankelijk effect nodig; de selectie heeft zowel
een contour als een tekstlabel en is niet uitsluitend kleurafhankelijk.
Geen claim van globale high-contrast- of schermlezertoegankelijkheid.

**Schrijfbestanden:** `sdk/examples/window-session/src/{scene,main}.rs`,
nieuwe `tests/window_switcher.rs` en `tests/switcher_native_fixture.rs`;
`tools/run_window_session.py`, `tools/test_window_session.py`, dit receipt.
Noodzakelijke profielbedrading: uitsluitend een feature-alias in
`kernel/Cargo.toml` en de bestaande profiel-/loader-/faultinjectieselecties
in `tools/build.py`. General gaf die begrensde dependency expliciet vrij;
exacte hunks zijn aan Maturity gemeld. Geen kernelruntime-/SMP-/m4-writes.

## Implementatie

Shell-eigen modaliteit consumeert keys voor de bestaande palette-/clearroute.
De bestaande openings-Resync annuleert oude appinput. De compositor houdt
releases vast na sluiten en schermt de volledige mousegesture af.
App-tegelupdates verversen de bijbehorende miniatuur; de fast-renderroute mag
de wisselaar niet overslaan. Miniaturen behouden beeldverhouding, gebruiken
begrensde nearest-neighbor sampling en controleren beide backingextents.
Fouten/overflow blijven fouten; een onmogelijke focuscounter verandert geen
focus voordat de fout is gemeld.

De normale sessierunner blijft standaard op zijn oorspronkelijke 100%-route.
`--switcher --scale 1|2` is een afzonderlijke native deelgate:
`switcher_gate=true` kan niet `native_gate=true` voor de volledige oude
herstel-/opslaggate betekenen. Diagnostiek/clipboard en deze deelgate kunnen
niet worden gecombineerd. `--scale 2` zonder `--switcher` wordt geweigerd.
De nieuwe HiDPI-probe combineert uitsluitend de bestaande normal/probefeatures.

## Uitgevoerd: gerichte hostproeven

In de bestaande geisoleerde ontwikkelomgeving `veld-env/env.sh`:

```sh
export CARGO_TARGET_DIR="$VELD_ENV/host-target"
cargo +1.90.0 fmt --manifest-path sdk/examples/window-session/Cargo.toml
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml --test window_switcher --test switcher_native_fixture
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml --all-targets -- -D warnings
python3 -B tools/test_window_session.py
```

Op de laatste bron elk exit 0: vijf daadwerkelijke nieuwe UX-/pixelproeven
(28,23 s testduur), plus de opt-in fixturetest zonder aangeleverde beelden
(geen gegenereerd oracle of nativeclaim in die lege route), en twaalf
runnertegenproeven. Pixel-/muisketens op vijf displays: 800x500@1,
1280x720@1, 1920x1080@1, 1600x1000@2 en 1920x1080@2.
Na iedere relevante overgang worden de hele incremental/referencepixels
vergeleken; miniatuurpixels worden tevens rechtstreeks tegen hun brondata
getoetst. Negatieve proeven omvatten korte backing, onjuiste index,
modifiercombinaties, dubbele Alt, repeats, inputverlies en counteroverflow.
Log: `$VELD_ENV/logs/veld-switcher-targeted-host-attempt3.log`.

Eerdere ontwikkelpoging 2 had een verkeerd geneste vijfde test; vier tests
slaagden maar Clippy weigerde de niet-uitgevoerde inner-test. Dat is
gecorrigeerd en bovenstaande laatste run voerde alle vijf werkelijk uit.
Log van die mislukking blijft `veld-switcher-targeted-host-attempt2.log`.

## Nog open: nieuwe native deelacceptatie en integratie

Pas na dit broncheckpoint:

```sh
export NOS_BUILD_DIR="$PWD/build/veld-switcher-20261103/scale1"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context"
python3 -B -u tools/build.py --kernel-feature native-window-session-probe
python3 -B -u tools/run_window_session.py --switcher --scale 1 --timeout 900 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
export NOS_BUILD_DIR="$PWD/build/veld-switcher-20261103/scale2"
export NOS_TARGET_DIR="$PWD/target/veld-native-20261103-context-hidpi"
python3 -B -u tools/build.py --kernel-feature native-window-session-hidpi-probe
python3 -B -u tools/run_window_session.py --switcher --scale 2 --timeout 900 --bios-rom "$NOS_BIOS_ROM" --vga-rom "$NOS_VGA_ROM"
```

De deelgate typt echte draft-/rekeninhoud, bewaart de modellen en controleert
de gehele wisselaar tegen een host-shelloracle dat echte appcaptures als
miniatuurbron gebruikt. Annuleren/herstellen vergelijkt het volledige scherm
met de actuele native basis, geen lege appfixture. De inputledger controleert
preciese KEYS/INPUT_ACKS-veranderingen, ook na een volgende echte edit.
Dit is het acceptatiecriterium, nog geen uitgevoerd native resultaat.

## Tussenresultaat op ec231857

De volledige scenehostsuite is op bron
`ec231857951f3aebb081084a704648bd3c3a984e` uitgevoerd:
`cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml`
en de formatcheck met `-- --check`: beide exit 0, tests wandtijd 35,60 s.
Log `veld-switcher-full-scene-host.log`.

Eerste 100%-probebuild exit 0, 7,20 s; de nieuwe native poging is niet
geaccepteerd. Na `switcher-work` (63,09 s) bleef de runner op de Alt-ACK
wachten. De seriële tellerreeks toont Materiaal KEYS 30 -> 31,
INPUT_ACKS 30 -> 31 -> 32: een eerdere F6-FocusLost werd pas na de gekozen
baseline bevestigd. De benodigde Alt-ACK werd daardoor gemist.
De hoofdagent heeft de vastgelopen poging gericht gestopt, geen
runner-exit-0/resultaat gefingeerd. Log
`veld-switcher-native-scale1-attempt1.log`, gedeeltelijk rapport:
`build/veld-switcher-20261103/scale1/run/window-3645fb9e61b0434ea024d302ef088804/result.json`.
Dit is een runner-synchronisatiefout; geen productfout of nativeacceptatie.

Vervolgfix wacht bij iedere voorbereidende F6 eerst expliciet op de enige
FocusLost-ACK van de vorige app. De nieuwe switcher-ACK-waits hebben een
begrensde observatietijd van 90 s; er zijn geen arbitraire sleeps toegevoegd
en geen inputcounters gemaskeerd. Bron-/native-uitkomsten van deze fix nog open.

CI en main-integratie zijn nog niet uitgevoerd. Geen fysieke/menselijke/audit-/
releaseacceptatie, geen scorewijziging. De volledige OS-/Veld-opdracht blijft
open. Na deze ene afgeronde milestone volgt direct een kleine concept-PR voor
integratie, geen extra featurebatch op dezelfde branch.
