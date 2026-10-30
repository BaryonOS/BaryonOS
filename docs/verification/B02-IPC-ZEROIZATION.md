# B02-IPC-HYGIENE-1 — IPC-envelopegeheugenhygiëne

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
