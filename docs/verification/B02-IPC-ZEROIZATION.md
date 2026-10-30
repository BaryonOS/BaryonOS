# B02-IPC-HYGIENE-1 — IPC-envelopegeheugenhygiëne

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

Dit checkpoint gaat vóór de langdurige native integratieproeven. Nieuwe
native system/B01/RANDOM/WAIT-proeven en actuele-head-CI zijn **nog niet
uitgevoerd**; de oude onderliggende buildresultaten worden niet naar de
nieuwe bron omgelabeld. Geen nieuw rubricpunt:34→34/security4of12.

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
