# B02-IPC-HYGIENE-1 — IPC-envelopegeheugenhygiëne

## RANDOM-consumerintegratie na exclusieve eigenaarsafspraak

30 oktober 2026: netwerkowner039dbef8 gaf exclusief
`sdk/keyring-random-client/src/lib.rs` en
`sdk/examples/random-probe/src/main.rs` vrij voor de gekoppelde Drop-fix.
Beide oude `wipe_received`-helpers en hun foutieve `!needs_drop`-aanname zijn
verwijderd. Bestaande afzender-/capabilityvalidatie en **alle capcleanup
vóór `drop(message)`** blijven staan, ook wanneer een close mislukt.
Unknown/no-retry, retirement, lease-/caller-/wirebeleid zijn ongewijzigd.
Geen nieuwe productie-unsafe; twee oude unsafe-vervangingen verwijderd.

De oude helpertest is vervangen door een echte destructortest voor zowel
Received als Outgoing. Deze bewaart de allocatie met `MaybeUninit`, roept
eenmaal de werkelijke destructor aan en leest daarna de huidige320
bufferbytes via volatile reads. De bufferlocatie komt uit `as_bytes`,
niet een veronderstelde structlayout. Uitsluitend test-unsafe, onderbouwd
bij de test; geen tweede destructor of lezen van vrijgegeven opslag.
Dit controleert daadwerkelijke zeroisatie, niet alleen compilatie of
`needs_drop`; geen claim over eerdere compiler-/registerkopieën.

Gericht uitgevoerd in de bestaande lokale hostlinkeromgeving:
`cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime -p nieuw-os-keyring-random-client`
exit0/36 tests:25 SDK-runtime en11 RANDOM-client, waaronder capcleanup bij
een mislukte close en de nieuwe echte destructortest.
Dezelfde selectie met
`cargo +1.90.0 clippy --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime -p nieuw-os-keyring-random-client --all-targets -- -D warnings`
exit0. Logs `/tmp/nieuw-os-security-ipc-random-drop-{host,clippy}.log`.

Nieuwe echte native herhaling op
`d4677211edc6b380f2e4f2d5f57ad9d8f0a75e72`, v3
`8d908a3b08bd99628e6485493600aaec707aa11f8d21da216d761019d249e38d`:

| Exacte opdracht | Resultaat |
|---|---|
| `python3 -B -u tools/test_system.py` | exit0,2/2 gasten normaal+koud |
| `python3 -B -u tools/test_keyring.py` | exit0,5/5 B01-gasten |
| `python3 -B -u tools/run_ipc_wait.py` | exit0,1/1 WAIT-gast,raw33 |
| `python3 -B -u tools/build.py --kernel-feature native-system-random` en `python3 -B -u tools/run_random.py --case raw` | beide exit0,1 echte gast,raw33 |
| `python3 -B -u tools/build.py --kernel-feature test-system-random-sdk` en `python3 -B -u tools/run_random.py --case sdk` | beide exit0,1 echte gast,raw33 |
| `python3 -B -u tools/build.py --kernel-feature test-system-random-unknown` en `python3 -B -u tools/run_random.py --case unknown` | beide exit0,1 echte gast,raw33 |
| `python3 -B -u tools/build.py --kernel-feature test-system-random-wrong-caller` en `python3 -B -u tools/run_random.py --case wrong-caller` | beide exit0,1 echte gast,raw33 |

Alle12 gasten gebruiken eigen wegwerpimages/schone firmware; de RANDOM-runner
hercontroleert de werkelijke IPC/servicecounters, afzenderbinding,
capabilityweigeringen en Unknown/no-retry vóór een geslaagde gate.
Geen tijd-/signer-/TLS-grant of entropykwaliteitsclaim toegevoegd.

Brongebonden capturecommands, beide exit0:
`python3 -B tools/capture_system.py --destination docs/verification/full-release-ipc-drop-system`
en
`python3 -B tools/capture_keyring.py --destination docs/verification/ipc-drop-keyring`.
De prefix `full-release-` is verplicht door de bestaande capturetool,
geen integrale releaseclaim. WAIT/RANDOM-rapporten en ongewijzigde echte
serialbytes/logs staan in `ipc-drop-random/`, per geval; de huidige buffers
en alle12 native-uitkomsten hebben uitsluitend bovenstaande nieuwe bron.

Aanvullend
`cargo +1.90.0 test --offline --locked --release --manifest-path sdk/Cargo.toml -p nieuw-os-keyring-random-client --lib sdk_owned_envelopes_are_zeroized_by_their_destructors`
exit0/1: echte320-bytezeroisatie ook met release-optimalisatie.
Gerichte fmt-checks voor client en probe slagen. Ruwe logs
`/tmp/nieuw-os-ipc-drop-{system,keyring,wait,random-<case>[-build],release-destructor}.log`
en de eerder vermelde host-/Clippylogs zijn in de capture opgenomen.
Alle vier RANDOM-serialorakels zijn bij capture opnieuw berekend en
vergeleken met de opgeslagen servicecounters. Alle66 capturehashes zijn
tegen Git-blobs gecontroleerd. De whitespacecontrole vond een lege
EOF-regel in de ongewijzigde ruwe Cargo-teststdout; die hostlogs behouden
hun bytes met een uitsluitend capturelokale whitespace-attribuutregel.

De eerdere82b-system/B01/WAIT-uitkomsten en mislukte RANDOM-compile
hieronder blijven historisch eigen bron; niet omgelabeld of verzwegen.
PR blijft concept/WIP totdat actuele-mainintegratie/head-CI en lifecycle-
review zijn afgerond; die controles, volledige overige native matrix,
fysieke hardware en menselijke externe review zijn **niet uitgevoerd**.
#74 frozen, geen scopeoverdracht. Score34→34/security4of12, TL eigenaar.

## Vervolg op main94 — WAIT-behoud en sterkere redactietest

30 oktober 2026: main `94c8908ee363d06a5be51417c5cae635549d566f`
inhoudsbehoudend in de eigen IPC-branch geïntegreerd. De twee gevoelige
runtimebestanden zijn per diff beoordeeld: alle ABI1.10-IPC-WAIT-code blijft
behouden; Key Service gebruikt idle_wait uitsluitend als random én admin
ontbreken. De enige servicedelta tegenover main is het verwijderen van
handmatige envelopevervanging, vervangen door de SDK-destructor.
De native receive-delta gebruikt Zeroizing-staging en dezelfde bestaande
capabilitycleanupfoutweg. Geen blind bestand van één branch gekozen.

De concrete reviewbevinding bij `message.rs` was terecht: alleen zoeken
naar `"secret payload"` detecteert een afgeleide Debug-implementatie met
decimale byteweergave niet. De test bewaart nu beide tekstcontroles, weigert
ook de exacte decimale byteprefix en vereist de volledige geredigeerde
Outgoing/Received-vorm. Een positieve controle toont dat de onderliggende
bytearray de bedoelde decimale fixture werkelijk bevat.

Gerichte hostcontrole met de hieronder beschreven lokale linkeromgeving:
`cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime`
exit0, nu 8 unit +14 message +3 process =25 tests, inclusief de nieuwe
main-WAIT-tests. Log `/tmp/nieuw-os-security-ipc-main94-host.log`.
De SDK-formattering en scoreconsistentiecheck slagen; centrale score-/status-
bestanden zijn exact main94 behouden in overleg met TL89f84098.

Eerste native integratie op `4de7e5644bdb1e72e1a5e3bb8453b0851729f39d`:
system normaal/koud2/2 en B01 vijf gevallen5/5 slagen. De volgende echte
IPC-WAIT-build faalde met exit101: de zelfstandige probe-lockfile miste de
nieuwe SDK-runtime→zeroize-edge; `--locked` weigerde terecht te bouwen.
De runner eindigde daardoor exit1; RANDOM was door de commandoketen nog
**niet uitgevoerd**. Logs `/tmp/nieuw-os-security-ipc-{system,keyring,wait}.log`.

Alle20 getrackte lockfiles die SDK-runtime consumeren zijn vervolgens met
`cargo +1.90.0 update --offline --manifest-path <consumer>/Cargo.toml -p nieuw-os-sdk-runtime`
gecontroleerd;19 veranderden, de SDK-workspace had de edge al. Na concrete
ontbrekende registrymetadata voor bestaande dependencies is dezelfde
gerichte update zonder `--offline` herhaald; geen brede update of versoepeling
van `--locked`. Een gestructureerde vergelijking van alle20 voor-/nagraaf-
versies bevestigt uitsluitend zeroize1.9.1 en de nieuwe runtime-edge; geen
andere packageversie/checksum/edge gewijzigd. Logs
`/tmp/nieuw-os-security-sdk-lock-update{,-retry}.log`.

Nieuwe echte herhaling op lockfilebron
`82b65d86d41a499b5697c999ae9007d6bafa627f`, v3
`0c324daafefe5b73537771d70c7297a3f868b7568bfe120a25bb1387d5229960`:
`python3 -B -u tools/test_system.py` exit0/2 native gasten,
`python3 -B -u tools/test_keyring.py` exit0/5 native gasten,
`python3 -B -u tools/run_ipc_wait.py` exit0/1 native gast, raw33.
Brongebonden captures: `full-release-ipc-main94-system82/`,
`ipc-main94-keyring82/`, `ipc-main94-wait82/`. De verplichte prefix van
de system-capturetool betekent **geen volledige releaseacceptatie**.

RANDOM blijft **mislukt,0/4 native uitgevoerd**: eerst werd de runner
terecht afgewezen wegens het nog geladen WAIT-profiel. Daarna faalde
`python3 -B -u tools/build.py --kernel-feature native-system-random` exit1
(onderliggende Cargo exit101/E0080): de oude handmatige
`wipe_received` in `sdk/keyring-random-client/src/lib.rs:350` veronderstelt
`!needs_drop::<Received>()`, maar de nieuwe SDK heeft juist een destructor.
De probe heeft dezelfde verouderde aanname. Dit is een gekoppelde
consumerintegratiefout, geen geslaagde RNG-test. De noodzakelijke
consumerfix is afzonderlijk af te stemmen; lease/callerbeleid mag niet
veranderen. Log plus concrete compilerdiagnostiek zijn vastgelegd in de
WAIT-capturedirectory. Actuele-head-CI en RANDOM-herhaling blijven open.

Voor system-capture is het normale native-system-buildprofiel hersteld met
`python3 -B tools/build.py --kernel-feature native-system` exit0; de
bestaande capturevalidator hercontroleerde vervolgens beide echte eerdere
gasten en hun artefacten. De eerste capturepoging op een ander huidig
profiel werd terecht geweigerd. Geen nieuwe gast of acceptatie verzonnen.
De eerdere4de-nativeuitkomst wordt niet naar deze fingerprint omgelabeld.
Geen nieuw rubricpunt:34→34/security4of12.

## Historisch bewijs vóór WAIT-integratie

Datum: 30 oktober 2026. Implementatiebron: `89d8b0941795d179fa8bb90d692aff0fb74f1f87`
op branch `simonvanieperen-solid-waffle`; source fingerprint v3:
`53fc1cdd5448d2bc50aaee32a8fcf0df95d0ca40679eb83a1cf32eb093cb5f4a`.

## Resultaat en grens

`Outgoing`- en `Received`-Message320-envelopes gebruiken `zeroize 1.9.1`
zonder default features en wissen hun bytes bij opruimen. De native receiveweg
houdt een `Zeroizing`-stagingbuffer aan en draagt een geldige envelope over
terwijl de stagingbytes direct worden gewist. De foutweg behoudt bytes lang
genoeg om maximaal twee ontvangen capabilityhandles te sluiten; daarna wist de
stagingbuffer bij opruimen. `Debug` toont geen payload. Gerichte tests
controleren overdracht, foutbehoud voor cleanup en redactie.

Dit is geheugenhygiëne, geen volledige sleutelisolatie en geen B02-acceptatie.
Caller-owned aanvraagarrays, compiler-/registerkopieën, crashgeheugen,
persistente antirollback, write-ahead per GENERATE/REVOKE en een echte
volume-identiteit blijven buiten deze wijziging.

## Uitgevoerde controles

| Exacte controle | Resultaat |
|---|---|
| `LOCALAPPDATA="$PWD/.tools" CARGO_TARGET_DIR="$PWD/target/security-ipc" GCC_EXEC_PREFIX="$PWD/.tools/bootstrap/host-linker/usr/lib/gcc/" LIBRARY_PATH="$PWD/.tools/bootstrap/host-linker/usr/lib/x86_64-linux-gnu" RUSTFLAGS="-C linker=$PWD/.tools/bootstrap/host-linker/bin/cc" python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime` | exit 0; 5 unit-, 14 message-, 3 process-tests geslaagd; testlog-SHA256 `1f870b3949902b9cc33a4ac6d39873ecaad33c0581243dd48969cd0892446195` (`/tmp/nieuw-os-security-sdk-runtime-tests.log`) |
| `LOCALAPPDATA="$PWD/.tools" CARGO_TARGET_DIR="$PWD/target/security-ipc" python3 -B tools/dev.py cargo +1.90.0 check --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime --target x86_64-unknown-none` | exit 0; no_std-targetcontrole |
| `LOCALAPPDATA="$PWD/.tools" python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml --all -- --check` | exit 0 |
| `git diff --check` | exit 0 |

De hosttests zijn gelinkt met Ubuntu GCC 13.4.0 en lokaal uitgepakte
glibc-ontwikkelbestanden; er is geen systeembreed pakket geïnstalleerd.
QEMU/OVMF-runtimeproeven en CI op deze commit zijn niet uitgevoerd. Dit rapport
sluit geen rubriccriterium en verhoogt de roadmapscore niet.

## Vervolg — native Key Service-integratie, 30 oktober 2026

Na de SDK-wijziging bleek dat `sdk/examples/keyring/src/service.rs` de
`Outgoing`-envelope handmatig overschreef met volatile writes en statisch
eiste dat het type geen destructor heeft. De nieuwe SDK-destructor maakte die
oude aanname ongeldig. De workaround is verwijderd; normale scope-exit voert
nu de SDK-zeroisatie uit. Er is geen keyring-wire-/TPM-/restorecontract
gewijzigd. Geïntegreerde broncommit `aec3034bb370f290b54331d3ef9694e4d1b7e680`;
source fingerprint v3 `0ba8e5f110c7c8225f88beabe6f25d8ca0d464726101c4b102059df36ce76aa2`.

De eerste native buildpoging faalde met exit 101 doordat `cc` niet in PATH
stond. Met de lokaal uitgepakte GCC 13.4.0 vond de compiler vervolgens de
statische `needs_drop::<Outgoing>()`-assertie (exit 101). Na verwijdering van
de verouderde handmatige workaround slaagde de native build (exit 0). De
eerste foutuitvoer is niet als afzonderlijk raw-log bewaard; de foutstatus en
oorzaak worden hier expliciet vastgelegd. De uiteindelijke logs zijn lokaal
onder `/tmp` bewaard en niet in Git opgenomen.

De uiteindelijke builds gebruikten de lokale omgeving:

```sh
export PATH="$PWD/.tools/bootstrap/host-linker/bin:$PWD/.tools/bootstrap/host-linker/usr/bin:$PATH"
export LD_LIBRARY_PATH="$PWD/.tools/bootstrap/host-linker/usr/lib/x86_64-linux-gnu:/tmp/nieuw-os-dhcp-hostcc/root/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export C_INCLUDE_PATH="$PWD/.tools/bootstrap/host-linker/usr/include:$PWD/.tools/bootstrap/host-linker/usr/include/x86_64-linux-gnu"
export LIBRARY_PATH="$PWD/.tools/bootstrap/host-linker/usr/lib/x86_64-linux-gnu"
LOCALAPPDATA="$PWD/.tools" python3 -B sdk/examples/keyring/build.py
LOCALAPPDATA="$PWD/.tools" python3 -B sdk/examples/system/build.py
```

Beide uiteindelijke commands eindigden met exit 0 en bouwen release-binaries
voor `x86_64-unknown-none`; logs:
`/tmp/nieuw-os-security-keyring-build.log` SHA-256
`391f37453d56823716d8f24816b8d256ee2bc0bd0fe96a837d68e393df796359` en
`/tmp/nieuw-os-security-system-build.log` SHA-256
`3f8a6f007c804fb0cbb5f50a3632b1dc99c9e0a79aea14483acf159a05a19a8e`.
Runtime-Clippy op SDK-runtime exit 0; log
`/tmp/nieuw-os-security-sdk-runtime-clippy.log`, SHA-256
`6648f70bbf70e1e45eb73f26f9eea5fc9fca0964ede7bdd39fe818ba18555104`.
De opnieuw uitgevoerde preflightinventaris exit 0; rapport
`/tmp/nieuw-os-security-preflight.json`, SHA-256
`ff55bfc97f2c5601477ee6266a06485fc7cbcaf613cda29829b2ecd9f45f6090`.
De inventaris ziet `x86_64-unknown-none`, maar geen QEMU, OVMF of
`x86_64-unknown-uefi`; dit is geen runtime-/bootbewijs.

Dit vervolg herstelt native buildcompatibiliteit maar sluit geen rubricpunt.
B02 blijft open: een vertrouwd koud rollbackanker, write-ahead per
GENERATE/REVOKE, echte volume-identiteit, native cold-recovery-/manipulatietests
en een vastgesteld multi-writer-/multi-device-contract ontbreken nog.
