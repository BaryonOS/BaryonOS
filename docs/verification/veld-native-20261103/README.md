# Veld: eerste native uitvoering — 3 november 2026

Basis `d6f8e98178d73d6aeda0abe84ca404dfde03123c`,
branch `simonvanieperen-fictional-disco`. Dit checkpoint bewaart de eerste
code en hostcontroles vóór de langere native build/VM-controle. Geen
volledige Veld-, U02–U06-, fysieke, menselijke of releaseacceptatie.

## Omgeving en uitgevoerde hostcontroles

De oorspronkelijke `python3 tools/preflight.py --strict` faalde met exit 2:
Rust/linker/QEMU/OVMF ontbraken. Daarna uitsluitend binnen de sessie-artifactmap
een geïsoleerde omgeving opgebouwd; geen systeeminstallatie of hostschijfmutatie.
Officiële Rust 1.90.0, beide eigen-OS-targets, rustfmt/Clippy, lokaal uitgepakte
distro-QEMU/firmware, bron-/pakket-/binaryhashes en echte linkchecks:
`files/veld-env/{report,validation,rust-packages,distro-packages,binary-hashes,crate-hashes}.json`.
Strikte preflight met `--strict-host-tests`: exit 0.
Node ontbrak eveneens; Node 22.22.0 ondersteunt de bestaande CI-optie
`--test-isolation=none` niet (exit 9). Officiële Node 24.11.0 met gecontroleerde
SHA-256 werkt; beide downloads bleven buiten Git.

Commandoprefix voor Rust/Python:
`source /home/simon/.copilot/session-state/7f2fb1d137004b68b9bef4b8ca72f938/files/veld-env/env.sh`;
hosttarget `CARGO_TARGET_DIR="$VELD_ENV/host-target"`.
De volledige logs staan in `$VELD_ENV/logs/veld-*`.

| Commando | Resultaat |
|---|---|
| `cargo +1.90.0 test --offline --locked --manifest-path ui/graphics/Cargo.toml` | exit 0; gradient/schaduw-tilegelijkheid, guards, ongeldige/extreme geometrie |
| `cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml` | exit 0 |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/window-session/Cargo.toml` | exit 0; inclusief 9 workspace-v2-tests, exact volledig/incrementeel beeld bij 800×500, 1280×720 en 1920×1080/200% |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml` | exit 0 |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/calculator/Cargo.toml` | exit 0 |
| `cargo +1.90.0 clippy --offline --locked --manifest-path <manifest> --all-targets -- -D warnings` | exit 0 voor alle vijf bovengenoemde manifests |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/window-session/Cargo.toml -- --check` | exit 0; appmanifests eveneens gecontroleerd |
| `cargo +1.90.0 test --offline --locked --manifest-path ui/surface-protocol/Cargo.toml --features client` | exit 0 |
| `python3 -B tools/test_window_session.py` | exit 0; 12 tests, inclusief vervalste workspacepixels en diagnostic-only weigering |
| `python3 -B -m unittest tools.tests.test_package_windows_qemu` | exit 0 |
| `$VELD_ENV/node/node-v24.11.0-linux-x64/bin/node --test --test-isolation=none ui/prototype/tests/*.test.mjs` | exit 0; 39 tests, inclusief echte tokencontrastparen |

Tijdens ontwikkeling zijn echte fouten gevonden en hersteld: een verkeerd
geneste methode/test, een Clippy-lusbevinding, niet bereikbare zoekknop en
onvolledige dockdamage bij focuswisseling. Eerdere niet-geregistreerde tests
zijn niet als uitgevoerd bewijs gebruikt; de uiteindelijke testlijst telt
de nieuwe tests expliciet mee. Optionele fixture-/previewtests zonder
omgevingsvariabele zijn geen visuele of native acceptatie.

## Native en overige grenzen

Eerste native uitvoering op `093d26eb`: build exit 0 (11,34 s), echte
QEMU-workspaceproeven tot en met Finder-selectie en documentinvoer uitgevoerd.
De totale gate gaf **exit 1** na 61,20 s: de oude beginbeeldvergelijking werd
ten onrechte over de nieuwe, bewust focuswijzigende shelltaken heen gebruikt.
Een verse pixelanalyse vond 296 gewijzigde calculator-caretpixels; de
workspace-selectiecapture kon bovendien vóór volledige chrome-repaint vallen.
Dit is geen geaccepteerd native resultaat.

De runner wacht nu na de shelltaken op de volledige exact gedefinieerde
chrome-complementgebieden én het oorspronkelijke volledige Materiaalbeeld.
Dat vastgelegde stabiele beeld is daarna de basis voor dezelfde ongewijzigde
vergelijking van **alle** pixels buiten de appbody; er zijn geen extra
uitzonderingspixels of maskers toegevoegd. Bestaande volledige apporakels,
crash-, schijf- en cleanupgates blijven verplicht. Gerichte scene-/runnertests
en Clippy slagen opnieuw; de volledige native herkansing volgt nog.

De herkansing op `3de934dd` passeerde daarna ook de volledige appisolatie-
en klembordketen tot berekening `2+3`. De bestaande ongewijzigde
verplaatsingsoracle vond een echte regressie: iedere titelbeweging annuleerde
onterecht **alle** appwachtrijen en wijzigde 296 caret-/selectiepixels van de
Rekenmachine (lokaal gebied x24–295/y28–55). De uitvoering is gericht gestopt
in plaats van de gate of oracle te verzwakken. De inputroute annuleert nu
uitsluitend een werkelijk aanwezige appmuiscapture bij layout-/modewisseling.
Titelbeweging zonder grab laat gewone invoer en caret ongemoeid.
Nog geen geslaagde volledige native acceptatie; herbouw/herhaling volgen.

De derde poging op `1915d510` blokkeerde op de seriële ontvangstobservatie:
de laatste gepubliceerde inputteller bleef 14. Dat is achteraf geen bewijs
dat F7 niet ontvangen werd.
De uitsluitend geïdentificeerde QEMU-PID is beëindigd; de runner sloot eerlijk
af met exit 1 / ontbrekende input-ACK. Dit bewijst niet dat de vorige
caretfix faalt. De nieuwe shell-injector gebruikt nu QMP's normale 100 ms
keyhold in plaats van een 1 ms-puls en begrenst ontvangst tot 30 s. Geen
automatische herinjectie van toggletoetsen en geen versoepelde pixel- of
ownershipcontrole. De oorspronkelijke gewone appkeyroute blijft ongewijzigd.

**Oorzaak daarna afgebakend:** ook 100 ms en afzonderlijk geïnjecteerde
press/release-edges bleven op dezelfde seriële observatie wachten; een gewone
Enter-/klemborddiagnose werkte wel (bewust op 90 s afgekapt, geen acceptatie).
De native kernelproef publiceerde compositorrecords alleen bij
app-framecommits, focus, beweging of pointerstatus. F7 verandert uitsluitend
shellpixels; na de caretfix ontstond terecht geen kunstmatige appframecommit
meer. De eerdere verklaring "niet ontvangen" wordt daarom ingetrokken.

De bestaande inputteller wordt nu meegenomen in de **alleen voor
`native-window-session-probe` gecompileerde snapshottrigger**. Geen nieuw
logveld, plaintext, recht, syscall, userpointer, scheduler- of productkernel-
gedrag toegevoegd. De shell-injector observeert echte press en release apart,
zonder herhaling of willekeurige keyhold als synchronisatie. De volledige
native herkansing moet dit samen met de originele regressiepoort toetsen.

**Geslaagde VM-bron:** `52aa1885445b8a2de1f52024adb7abd64de1ab5a`,
source-SHA-256 `c63a4d7a7cdde0998e6d0b8e91eaf65706e57181cd27055b99e193f262aa0c24`.
De onverkorte `--clipboard --workspace`-gate geeft exit 0: 28 fasen in
241,63 s, ruwe QEMU-exit 33, echte compositorcrash en nieuwe procesidentiteit/
epochs, bewaarde draft en berekening, werkfocus/Finder/ordening, volledige
appbeelden en buiten-appisolatie, klembordketen en onafhankelijke diskcontrole.
Disk: revisie 1, 14 bytes, SHA-256
`9a45a00fa9ff69c17e124f54c14bb14cf2c7650e8cf1c272f6631f618a19a5a5`;
canaries intact. Logs: `veld-native-{build,gate}-observed.log`.

De gedeelde-afnemercontrole vond daarna drie oude lichte Home-tekstkleuren
in de afzonderlijke gast/sessiepreview. Deze zijn vervangen door dezelfde
tekstrollen; een nieuwe hosttest controleert 4,5:1 over alle 256 verloopstappen.
Vijf relevante test-/Clippy-/fmt-routes slagen opnieuw. Die bronwijziging
krijgt nog een eigen volledige VM-herhaling; het bovenstaande bronbewijs
wordt niet zonder herbouw aan een gewijzigde fingerprint gekoppeld.

De eerste buildpoging naar de artifactmap buiten de checkout is terecht
geweigerd (exit 1, `FPU artifact escaped checkout`). Geen guard aangepast;
de build gebruikt daarna de genegeerde lokale mappen
`NOS_BUILD_DIR="$PWD/build/veld-native-20261103/probe"` en
`NOS_TARGET_DIR="$PWD/target/veld-native-20261103"`.
CI is aangepast, niet als geslaagd geteld.
Een host-renderoracle is geen VM-test. QEMU is geen fysiek GPU-, driver-,
energie- of timingbewijs. Geen nieuwe gebruikerstest, schermlezerproef,
onafhankelijke menselijke review of volledige releasecontrole uitgevoerd.
Voortgang **41 → 41**; geen criteriumstatus, gewicht of historisch bewijs gewijzigd.
