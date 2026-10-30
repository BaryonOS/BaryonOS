# Native Materiaal — app-lokale semantische snapshot

Status: beperkte implementatiestap, geen aftekening van native toegankelijkheid.

## Bron en wijziging

- Basiscommit: `f52dab50af281dab2c94ab2c9e31aaabd299246a`
- Implementatiecommit: `56626baf7f6db7b8a1d0e07787ae4df5997d96f5`
- Branch: `codex/native-materiaal-accessibility-20261030`
- Bronfingerprint v3 vóór documentatie/checkpointupdates:
  `d9e3c57244904e20bf52f95b2b7d80e3d2d93e22a303b86f179e583f53e80185`
- `App::semantics` geeft uitsluitend de huidige semantische view van de eigen
  app terug. Het deelt niets met een ander proces en verleent geen rechten.
- De bestaande `complete_chain`-integratietest controleert kop, focus,
  documentwaarde, bewerkbaarheid, opgeslagen status, heropenen en atomische
  weigering van een te kleine buffer.

## Uitgevoerde controles

| Exact commando | Resultaat |
|---|---|
| `python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path sdk/examples/materiaal/Cargo.toml -- --check` | exit 0 |
| `python3 -B tools/dev.py cargo +1.90.0 check --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --tests` | exit 0 |
| `python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --all-targets -- -D warnings` | exit 0 |
| `python3 -B -m unittest tools.tests.test_materiaal_probe tools.tests.test_materiaal_host_scenarios tools.tests.test_materiaal_host_count` | exit 0; 27 tests |
| `LOCALAPPDATA="$PWD/.tools" python3 -B sdk/examples/materiaal/build.py` | exit 0; echte `x86_64-unknown-none`-ELF gebouwd |
| `sha256sum build/sdk-materiaal/x86_64-unknown-none/release/materiaal-app` | `cb1f73a07e18d35d77db0b0056d4cf4702ef64a9d59b61484a1e9c2ff1d3090c` |
| `python3 -B tools/dev.py cargo +1.90.0 fetch --locked --manifest-path sdk/Cargo.toml` | exit 0; alleen bestaande lockfile-versies opgehaald; geen manifest/lockfilewijziging |
| `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml` met lokale `.tools/nieuw-os/host-tools` GCC/sysroot-omgeving | exit 0; alle 20 `chain`-scenario's slagen |
| `python3 -B tools/build.py --kernel-feature native-system-materiaal-probe` met lokale Rust/QEMU-tooling en `NOS_QEMU_MACHINE=pc-q35-8.2` | exit 0; kernel en acht userspace-ELF's gebouwd en geïnspecteerd |
| `python3 -B tools/preflight.py --strict` met lokale QEMU/OVMF-paden | exit 0; inventory slaagt; geen boot uitgevoerd |
| `python3 -B tools/run_materiaal_probe.py --case normal --build-dir build/materiaal-probe-accessibility --machine pc-q35-8.2 --qmp-transport pipe --vga-rom .tools/nieuw-os/host-tools/usr/share/seabios/vgabios-stdvga.bin --vga-rom-sha256 c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde` | **FAIL**, raw QEMU-exit 35 na 20.000 PIT-ticks; 240 s runnerdeadline niet verruimd; geen save/reopen |

De normale native QMP-proef is brongebonden in
[`qmp-normal-red.json`](qmp-normal-red.json). Zij bereikte drie echte frames
(ready/editor/dirty), vier muis-ACKs en 28 key-events; de draft telde 14 bytes.
Daarna meldde de gast `MATERIAAL:FAIL:TIMEOUT`, met `SAVE_ACKS=0`; de
opslag-/heropenings-/clean-stopfasen zijn niet bereikt. De runnerdiagnostiek
wees bovendien het eerste `MATERIAAL:TIMEOUT_SLOT`-record af als onbekend.
Dat rapportageprobleem wordt apart hersteld; het verandert de timeout nooit in
een geslaagde proef.

Ready-, editor- en dirty-screenshots (800×600) zijn visueel bekeken. Dat toont
de werkelijke eigen-kernel-rendering en muis-/toetsbediening in deze beperkte
run; het bewijst geen keyboard-only taak, screenreader/spraak, menselijke
toegankelijkheidsproef of visuele eindacceptatie. De latere native
IPC-wachtintegratie op een andere bron is apart bewijs en maakt deze oude
run niet groen of bron-equivalent.

De eerstvolgende native probe gebruikt QEMU `10.2.1`,
`OVMF_CODE_4M.fd` SHA-256
`874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7`,
`OVMF_VARS_4M.fd` SHA-256
`5d2ac383371b408398accee7ec27c8c09ea5b74a0de0ceea6513388b15be5d1e`, en
`vgabios-stdvga.bin` SHA-256
`c944f5fd404a6553a32e1e0527081d40e6040d3ce1740d39766bff01871e4fde`.
Deze zijn lokale tool-inputs, geen meegeleverde productonderdelen.

## Bewijsgrens en open werk

De ELF-build, Rust-typecheck en lint vervangen geen uitvoering van de app of
van een accessibility-service. Er is nog geen native schermlezer-/spraakuitvoer,
keyboard-only gebruikersproef, instellingen
voor schaal/contrast/reduced motion, menselijke gebruikerstest of fysieke
hardwaretest uitgevoerd. Deze wijziging sluit geen U01–U06-criterium en
verhoogt de vaste score niet. Zie `docs/PROJECT-STATE.md` voor de eerstvolgende
hervatstap.

## Semantiekboom en native keyboard-baseline

De leaf-UI-semantiekboom staat op implementatiecommit
`f6ec180103779dfdb1f50088c7a9cfd7338fb955`; de brongebonden host/scalar-
verificatie staat op `02766785356de75fc2f766031f79e49c48839ff0`. De volledige
`ui/native`-hosttests (`20/20`), fmt, check, Clippy en scalar-build slaagden;
`ui/native/verification/checks.json` vermeldt `native_execution: false`.

De aparte native keyboard-baseline is uitgevoerd op **andere code**:
`74d15ac09ac61c8d47a882c3c982711af1a9d891`, fingerprint v3
`24d5f4c112924cfe45681de3fcf3a92f62087e29524f15c74e14d83620891abd`.
De werkelijke guest voltooide zes frames, 42 toetsen, nul pointerevents en
save/list/in-sessie-reopen; raw QEMU-exit 33, probe passed, geen timeout.
Het guest-sourcebestand mist de nieuwe stabiele semantic-ID's, boomrelaties en
actievelden. Dus dit is geen native bewijs voor de nieuwe leaf-code of een
screenreaderconsumer. Details en hashes staan in
[`qmp-keyboard-v3.json`](qmp-keyboard-v3.json); de volledige ruwe run is lokaal
bewaard onder de genegeerde `build/native-materiaal-accessibility-v3/`.

Open implementatiepoort: release moet de leaf-UI-code integreren in één actuele
Materiaal-runtime, de sourcefingerprint en gebouwde app/artifacthashes binden,
en de keyboardgate op exact die guest opnieuw uitvoeren. Koude herstart,
screenreader/spraak en menselijke UX-acceptatie blijven aparte open bewijzen.
