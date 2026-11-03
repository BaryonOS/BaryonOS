# RANDOM raw: begrensde fixturecorrectie

## Scope en bron

- Basis: `a32eadb1e2fff42b7af4b783fcbb9db42b5c6e0e` (`origin/main` bij start).
- Werkbranch: `fix/random-raw-window-20261103`, eigen geisoleerde worktree.
- Alleen `sdk/examples/random-probe/src/{main,contract}.rs`,
  `sdk/examples/random-probe/tests/quota.rs` en dit ontvangstbewijs.
- Geen wijziging aan productiequota, entropy, rechten, keyring-service, kernel,
  runnerassertions, centrale status/score of de bevroren #156.
- Score blijft ongewijzigd; centrale voortgangsbestanden zijn eigendom van de
  integratiesessie en vallen expliciet buiten deze opdracht.

## Rood bewijs en oorzaak

Run [41671957624, job 126042385200](https://github.com/simonvanieperen/nieuw-os/actions/runs/41671957624/job/126042385200),
CI-head `16eefda390cce7d71a35ddf35c6e58dcb44a3938`: de gast faalt op stap 15,
request 1064, expected count 0, status 0, bodylengte 17. Burst start 131,
eind 245, failure tick 247. De 64 sequentiele volledige RPC's duren 114 ticks,
terwijl het productiequotavenster 100 ticks duurt. Een success na vernieuwing
van het venster is toegestaan; dit is geen aangetoonde productquotafout.

Primaire bron voor deze CI-waarneming: de genoemde GitHub Actions-job
`native-foundation` in workflow **Native RANDOM and image foundation
acceptance** van GitHub (3 november 2026; run 41671957624, job 126042385200; head
`16eefda390cce7d71a35ddf35c6e58dcb44a3938`). Dit is bestaand rood CI-bewijs,
geen herhaalde run in deze extractie.

`git diff 16eefda3 a32eadb1 -- sdk/examples/random-probe
services/keyring/src/random.rs tools/run_random.py
sdk/examples/keyring/src/service.rs` is leeg. Deze betrokken bronnen zijn dus
gelijk, zonder overige resultaten van beide branches te combineren.

De ongewijzigde lokale native raw-run slaagde: exit 0, QEMU-exit 33,
`gate=True`. Dat is nadrukkelijk **geen lokale rode reproductie**.
Het rode VM-bewijs komt uit de bestaande CI-run; de 114-tick-schedule wordt
daarnaast deterministisch als hostregressie nagebootst.

## Correctie en invarianten

De succesvolle requests 1000..1063, elk een byte, gaan in vier batches van 16. Request 1064 volgt afzonderlijk als de 65e request en moet `Quota` opleveren.
Beide bestaande IPC-endpointwachtrijen hebben capaciteit 16. De primaire
bron is [Nieuw OS, `capability-core` endpointqueue (`QUEUE_CAPACITY`), commit
`78ff7c03b015ad16fe665ae7df9fe239d078e08d`, 3 november 2026](https://github.com/simonvanieperen/nieuw-os/blob/78ff7c03b015ad16fe665ae7df9fe239d078e08d/crates/capability-core/src/lib.rs#L72);
de capaciteit geldt per endpoint in de gedeelde capability-engine.
Het ongewijzigde productiebeleid staat in [Nieuw OS, keyring RANDOM-quotavenster
en calllimiet, commit `78ff7c03b015ad16fe665ae7df9fe239d078e08d`,
3 november 2026](https://github.com/simonvanieperen/nieuw-os/blob/78ff7c03b015ad16fe665ae7df9fe239d078e08d/services/keyring/src/random.rs#L17-L19):
100 ticks en maximaal 64 calls per venster. Iedere batch wordt volledig
ontvangen voordat de volgende wordt verstuurd. De service behoudt haar
bestaande verwerking, scheduler en pending-replygedrag.

Iedere reply blijft strikt gecontroleerd op afzender, tag, capabilities,
request-ID, status, lengte en canonicalisatie; buffers behouden zeroizing.
Request 1064 moet nog steeds echt `Quota` opleveren met een lege body.
Daarnaast moet de volledige burst inclusief deze denial volgens de gastklok
strikt binnen de ongewijzigde 100 ticks vallen. Geen skip, retry, kunsttijd
of groter venster. Een te trage uitvoering blijft rood.

De hosttests controleren beide volledig gevulde echte capability-wachtrijen,
FIFO, ongewijzigde 64 succesvolle calls/64 bytes/een seed en de echte
65e-call-denial. De CI-schedule-test toont dat success na venstervernieuwing
nooit als quota-denial geaccepteerd wordt. Synthetische hosttijd en testentropy
zijn geen bewijs van native timing of entropykwaliteit.

## Eerste controles voor checkpoint

Werkdirectory:
`/home/simon/.copilot/session-state/edf11e7c8b3e4c9ba1a91b7ef0779695/files/random-raw-window`.
Omgevingsinstellingen staan buiten Git in `../random-environment.sh`.
Rust 1.90.0, bestaande geisoleerde GCC-sysroot, QEMU 10.2.1, OVMF 4M,
TCG, alleen wegwerpimages; geen installatie of fysieke schijf.

| Commando | Resultaat |
|---|---|
| `python3 -B tools/build.py --kernel-feature native-system-random` (ongewijzigd) | exit 0 |
| `python3 -B tools/run_random.py --case raw` (ongewijzigd) | exit 0, raw 33, gate True, 6,8 s |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/random-probe/Cargo.toml` | exit 0, zes tests |
| `python3 -B tools/build.py --kernel-feature native-system-random` (eerste correctie) | exit 0 |
| `python3 -B tools/run_random.py --case raw` (eerste correctie) | exit 0, raw 33, gate True, 6,6 s |

Eerste buildpogingen mislukten door ontbrekende hostlinker, daarna een
ontbrekende sysroot in een buildscript. Hersteld door de al aanwezige
`/tmp/nieuw-os-c-toolchain/cc-wrapper` te gebruiken; geen dependencywijziging.
Eerste nieuwe hosttestcompile gaf een usize/u64-verschil; gecorrigeerd.
Eerste Clippy-run wees een gewone assertion op constanten af; vervangen door
een compile-time assertion. Finale controles volgen hieronder.

Logs buiten Git: `build/random-window-evidence/`. Baseline native run:
`build/random-window-before/run/system-normal-61f0a4c9f0204216baa9a57eb08ed505/`.
Eerste gecorrigeerde run:
`build/random-window-after/run/system-normal-31a91166d513451f9753fa94c53e6efe/`.

## Finale brongebonden controles

Product-/fixturecommit: `a30046316cbeda5484e920fb819ef34bbe152f5d`.
Alle zes finale VM-resultaten bevatten dit commit-ID en bronfingerprint v3
`cf9b1d3032c3da5af3bf26f2ae52dc6c919a586ce847f9effe88ef4a6445a02b`.
Alle zes: `passed=true`, `random_gate=true`, `status=completed`,
`raw_qemu_exit=33`, `timed_out=false`. Iedere start gebruikt een nieuw
wegwerpvolume, ESP-kopie en VARS-kopie. Exacte kernel-/runnercounters bleven
ongewijzigd en worden door iedere gate gecontroleerd.

De gebruikte omgeving, vanuit bovengenoemde werkdirectory:

```sh
export PATH=/tmp/nieuw-os-c-toolchain/bin:/home/simon/.cargo/bin:/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools/nieuw-os/qemu:$PATH
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/tmp/nieuw-os-c-toolchain/cc-wrapper
export COMPILER_PATH=/tmp/nieuw-os-c-toolchain/root/usr/libexec/gcc/x86_64-linux-gnu/15
export LIBRARY_PATH=/tmp/nieuw-os-c-toolchain/root/usr/lib/x86_64-linux-gnu:/tmp/nieuw-os-c-toolchain/root/usr/lib/gcc/x86_64-linux-gnu/15
export LD_LIBRARY_PATH=/tmp/nieuw-os-c-toolchain/root/usr/lib/x86_64-linux-gnu
export OS_OVMF_CODE=/tmp/nieuw-os-qemu/root/usr/share/OVMF/OVMF_CODE_4M.fd
export OS_OVMF_VARS=/tmp/nieuw-os-qemu/root/usr/share/OVMF/OVMF_VARS_4M.fd
export NOS_TARGET_DIR="$PWD/target/random-window-native"
export CARGO_NET_OFFLINE=true
```

Finale commando's, allemaal exit 0:

| Commando | Resultaat / log onder `build/random-window-evidence/` |
|---|---|
| `cargo +1.90.0 fmt --manifest-path sdk/examples/random-probe/Cargo.toml -- --check` | groen, `final-host.log` |
| `CARGO_TARGET_DIR="$PWD/target/random-window-host" cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/random-probe/Cargo.toml` | 6 tests, `final-host.log` |
| `CARGO_TARGET_DIR="$PWD/target/random-window-host" cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/random-probe/Cargo.toml --all-targets -- -D warnings` | groen, `final-host.log` |
| `CARGO_TARGET_DIR="$NOS_TARGET_DIR/random-probe-app" python3 -B sdk/examples/random-probe/build.py clippy` | no_std native Clippy groen, `native-clippy.log` |
| `NOS_BUILD_DIR="$PWD/build/random-window-a3004631-raw" python3 -B tools/build.py --kernel-feature native-system-random` | groen, `final-raw-build.log` |
| `NOS_BUILD_DIR="$PWD/build/random-window-a3004631-raw" python3 -B tools/run_random.py --case raw` (3 maal) | 3 echte gates groen, `final-raw-{1,2,3}.log` |
| `NOS_BUILD_DIR="$PWD/build/random-window-a3004631-sdk" python3 -B tools/build.py --kernel-feature test-system-random-sdk` | groen, `final-sdk-build.log` |
| `NOS_BUILD_DIR="$PWD/build/random-window-a3004631-sdk" python3 -B tools/run_random.py --case sdk` | gate groen, `final-sdk-run.log` |
| `NOS_BUILD_DIR="$PWD/build/random-window-a3004631-unknown" python3 -B tools/build.py --kernel-feature test-system-random-unknown` | groen, `final-unknown-build.log` |
| `NOS_BUILD_DIR="$PWD/build/random-window-a3004631-unknown" python3 -B tools/run_random.py --case unknown` | gate groen, `final-unknown-run.log` |
| `NOS_BUILD_DIR="$PWD/build/random-window-a3004631-wrong-caller" python3 -B tools/build.py --kernel-feature test-system-random-wrong-caller` | groen, `final-wrong-caller-build.log` |
| `NOS_BUILD_DIR="$PWD/build/random-window-a3004631-wrong-caller" python3 -B tools/run_random.py --case wrong-caller` | gate groen, `final-wrong-caller-run.log` |
| `python3 -B -m unittest discover -s tools/tests -p test_random.py` | 21 tests, `runner-tests.log`; getoonde `gate=False`/`ValueError` zijn verwachte negatieve fixtures |
| `git diff --check` | groen |

Finale runpaden volgen het patroon
`build/random-window-a3004631-<profiel>/run/system-normal-<ID>/`.
De gate bewaart per run `serial.txt` en `result.json`; manifest en laatste
`random-report.json` staan in de profiel-buildmap.

| Profiel | ID | Duur (s) | SHA-256 `serial.txt` |
|---|---|---:|---|
| raw 1 | f95e5b14d94444ea9cd6c4e8572b5ba9 | 6,636 | `d263d665109d9e9e371cc6180a6346fdaabef15dc3b46d93cce59c3cd24a60d8` |
| raw 2 | 1e1d6953eae74c31ad7399e5a78bca41 | 6,538 | `59c72fc0c67f8d3bea5b65e6ca5f809336b0f32b150b87ef565eb3241402961a` |
| raw 3 | 4f4667f6055c47298a66b7683da5cd0f | 6,474 | `b7b5e5b76c342a5ad1137d67da78088a526a6299fa2dd10ac12ccfafa9e63439` |
| sdk | 802649dd45924c739cd4d6abd639d926 | 6,185 | `dc169acbe345dad689101184d95f4ade5006e36969c52723e753d1f5ab0fd416` |
| unknown | 9e39e23d9846468e957fee9493b871c3 | 6,266 | `a322fb31d797fd0435c79dfd36de66beff899090f7da3fc93108232af8c23cd5` |
| wrong-caller | 1d7583154e414170a4673570d2dd8b06 | 6,226 | `e3b010f760497c49623b570ab952130d47fe43208138363130c556d08cbdb124` |

SHA-256 `result.json`, in dezelfde volgorde:

```text
b111b81f5d67999a159a709d9e49bbb25cef3a126e8f5fa12ed4f5161a929dcc raw1
cb0cf0ec0d5f029d122f589f9e3b0b5dbf0fb0de9b74332ea9085fc5bb8a50b6 raw2
15fff3820709cf37abd90660541c7bd274f2cd760faa90601d576f41101be217 raw3
6ba2d984d62ea9cefe7dfa0a04ef939f93acabd429b615a812768d93f6c1fbdd sdk
81f31024d88caa247f0688e625fc10f3b0f70de1161e9a759db734a84fdd37e0 unknown
9367eb5d707a3ea55464a44c7be54e685acd6833cdfacdfd73278e199fd7e8e6 wrong-caller
```

Bron- en binaire binding voor raw:

```text
08596fbe56424b0ca6ed5507f42440ad9d1a2819d03164cc3d3f6005e2396e1f random-probe ELF
87c9b8dd3db818aa6873754201d72eafdf4dd175cc581ca88ac0c4f90346cc21 kernel ELF
20c6560f6f426d3bf921391a870746749757758c6cd49dd3ae9776db852177eb UEFI loader
c0ec393c23f7ba42844585c53fcc8983d399e0280eee8675b0da999b46c6fd4d manifest.json
874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7 OVMF CODE
5d2ac383371b408398accee7ec27c8c09ea5b74a0de0ceea6513388b15be5d1e OVMF VARS-template
09b7d8ab4a583935d449882fcc5fbd73ca76398c4be3a8c0d46108424d5c4b5d QEMU-wrapper (runnerhash)
fa552a45c337a88fe71e1e1af9ebace875f30e88685a6986d9406c92efaf44cb QEMU executable
```

Loghashes (SHA-256), onder `build/random-window-evidence/`:

```text
fb660ca3d6eba4f1fd373a2d2baad90c1e9b0acaa9a8887a38e52104563c6ef5 final-host.log
e75dc925bbb4cf12d0c04ddac5a0ae8d0996cedde00afda02070af2be2a73803 native-clippy.log
77641e7014a439ad2c5d9e154f8f1eb76075278105c0b93602694b95f3f990ab runner-tests.log
945ec9f6ce6ae645e151a327788838f0931997bc9bd140fbdec9791bd389868a final-raw-build.log
18349e01ffc4f2041dfc279bed9dfb93039c2b40e59b0189f04e041e0c0bd18b final-raw-1.log
db0d02800c8c2191d01646a1006b0365bf7c65acfe14ae42cf569214c58ba09e final-raw-2.log
7376a8fa6f7b7c20e52df7b42a8349884d016d9608eb3d42e9e4c4838ea345c9 final-raw-3.log
8add9d61de7c79c7b90dd5c45e6d983e8c7b672b2897a3f4e57c3fa8bc6dcbd6 final-sdk-build.log
7fdd176f452a81a4333abf6fda9ad591b239ed6404735f6b0e100281d2600d01 final-sdk-run.log
491efa671c7dd91ccd582e6133ce77b72513c9fdafb1d6ad0520fc8a77c1fc4f final-unknown-build.log
887a514dd09ba350a447ebf9e4074012e005144afafc7460be668ed98f37ab42 final-unknown-run.log
ad00bc564ba251c04c0589ade5f46bf9e654c4fc3e56aab33072101435da76a4 final-wrong-caller-build.log
c3948f7dbb51d376dc5b3a635f5fde144ab37bd8240947993ae3275aa1e22b3f final-wrong-caller-run.log
```

Rood CI-log: `/tmp/nieuw-os-sprint-pr146-foundation-failure.log`,
SHA-256 `da3223d9cad53289928a211014cacaaf016c9767b5f3149e868b028c83f2058b`.
Rode gastserial:
`/home/simon/.copilot/session-state/0fc9ee51f1e94fc69b3031941f626af0/files/sprint-pr146-ci-evidence/random-raw/run/system-normal-6fe6ad0d73994e9ca25ccc38703c5e52/serial.txt`,
SHA-256 `749d2475761548d0db0a362f46c82bb92cf0cb8ebac9df17ceb53976fa3ac6bf`.

## Resterende bewijsgrenzen

De oorspronkelijke CI-run is **niet opnieuw uitgevoerd**. De fix levert
drie groene echte lokale rawgates tegenover de bestaande rode CI-schedule;
dit is geen claim van lokale rood-naar-groenreproductie of universeel
timinggedrag. Een omgeving die ook met batching de 100 ticks overschrijdt,
faalt nog steeds gesloten. Alle finale rawgates hebben zowel de werkelijke
quota-afwijzing als de nieuwe gehele-burst-tijdgrens doorlopen.

Geen hardwaretest, onafhankelijke audit, merge of releasevalidatie.
Score ongewijzigd; centrale beoordeling blijft bij de integratiesessie.
Rollback: alleen deze fixturecommit terugdraaien; er is geen productpolicy-
of opslagformaatwijziging.

## Extractiecontrole op actuele main

De beperkte extractie is na ontvangst van de nieuwe main opnieuw gebaseerd op
`1c6872063fab055211eff48fce11496db39728be`. De fixturecode staat in commit
`c584f61a4bb0a38fe02be2d3d84420b3f84875ff`; tijdens de controles was de
volledige bron-HEAD `51d9467bcb60f4281302110cf3562602784b3825`. De native
runner bond de run aan die HEAD en bronvingerafdruk v3
`bd99bdb03ae667b491e7ad050360ac6f7fcfc078c50d99d04b0b3e061ac326fe`.

Toolchain: Rust/Cargo 1.90.0, QEMU 10.2.1, machine `pc-q35-8.2`, OVMF 4M.
QEMU gebruikte uitsluitend de bestaande wegwerpimage. De volgende controles
zijn op de extractie uitgevoerd:

Voor de hostcontroles:

```sh
export PATH=/home/simon/.cargo/bin:/tmp/sm-oom-qemu-root/usr/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
```

Voor de native build en run:

```sh
export PATH=/home/simon/.cargo/bin:/tmp/sm-oom-qemu-root/usr/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export CC=/tmp/nieuw-os-c-toolchain/bin/cc
export LD_LIBRARY_PATH=/tmp/sm-oom-qemu-root/usr/lib/x86_64-linux-gnu
export OS_OVMF_CODE=/tmp/sm-oom-qemu-root/usr/share/OVMF/OVMF_CODE_4M.fd
export OS_OVMF_VARS=/tmp/sm-oom-qemu-root/usr/share/OVMF/OVMF_VARS_4M.fd
export NOS_QEMU_MACHINE=pc-q35-8.2
export NOS_BUILD_DIR="$PWD/build/random-extract-raw"
export NOS_TARGET_DIR="$PWD/target/random-extract-native"
export CARGO_NET_OFFLINE=true
```

| Commando | Resultaat |
|---|---|
| `CARGO_TARGET_DIR="$PWD/target/random-extract-host" cargo +1.90.0 fmt --manifest-path sdk/examples/random-probe/Cargo.toml -- --check` | exit 0 |
| `CARGO_TARGET_DIR="$PWD/target/random-extract-host" cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/random-probe/Cargo.toml` | exit 0; 6 hosttests |
| `python3 -B -m unittest discover -s tools/tests -p test_random.py` | exit 0; 21 harnastests |
| `python3 -B tools/build.py --kernel-feature native-system-random` | exit 0 |
| `python3 -B tools/run_random.py --case raw` | exit 0; `passed=true`, `random_gate=true`, QEMU-exit 33, `timed_out=false` |
| `git diff --check origin/main...HEAD` | exit 0 vóór deze receipt-aanvulling |

De native run staat in
`build/random-extract-raw/run/system-normal-4d14d5fdf3124b5e97ea9545851b0088/`.
SHA-256 `serial.txt`:
`878ecf82053e1618147ec08be10fe48fad8d371bddfba34e36406b11feaab611`.
SHA-256 `result.json` en `random-report.json`:
`cd9ba380c5820e3f5d14d52f547f624c0ba865d4566705a3d7a4dc62208efd80`.
De runmap, VM-/diskimages, ruwe serial- en JSON-resultaatbestanden en
commandologs zijn lokale buildartefacten buiten Git; alleen de hierboven
vermelde compacte uitkomsten, paden en hashes staan in deze receipt. De hashes
maken de niet-gecommitte lokale artefacten niet zelfstandig beschikbaar.

Logs zijn buiten Git bewaard:

| Log | SHA-256 |
|---|---|
| `/tmp/random-extract-host.log` | `125c80979030976738467f484b8f699645b8eca2a526c3d9e3db4b0c21f21a0d` |
| `/tmp/random-extract-harness.log` | `c7c7b5765cf996863d49b45efaa3cc56ab6c3491abb646eddb0aec401db54a5d` |
| `/tmp/random-extract-native-build.log` | `7acad9d0544f44fd67aef277e38765ecee38e28ef95b78c193b2913d4eb09da7` |
| `/tmp/random-extract-native-raw.log` | `3771e55e88c21f64d3c9241668ce13b77227d506b1007397b4c8180d8095f4b3` |

De CI-run is niet opnieuw uitgevoerd; ook Clippy, andere RANDOM-profielen,
hardwaretest, onafhankelijke audit en releasevalidatie zijn niet uitgevoerd.
Deze controle valideert uitsluitend de fixturewijziging en de genoemde lokale
host-/harnas-/VM-gates; zij verruimt geen productiegrenzen.
