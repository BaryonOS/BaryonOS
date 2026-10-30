# Verificatie van de artifact-storecore

Status: lokale verificatie van een alleen-lezen contractcrate; geen
acceptatie van een native dienst, installer, loader of release.

- Broncheckpoint: `0816a530bda9006dbb5c96a96061d8fb29e385fd`
- Branch: `simon/native-artifact-store-contract`
- Er is gecontroleerd dat de remote branch naar dezelfde commit wees.
- Toolchain: vastgezette Rust `1.90.0`; Cargo is uitgevoerd via `python3 -B
  tools/dev.py`. Hosttests gebruikten de bestaande alleen-lezen bundel
  hosttools uit een naburige checkout; er zijn geen pakketten geïnstalleerd.
  Deze lokale bundel is geen CI-afhankelijkheid.

| Controle | Exact commando | Resultaat |
|---|---|---|
| Hosttests | `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked -p artifact-store-core` | exit 0; 7 passed |
| Clippy | `python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked -p artifact-store-core --all-targets -- -D warnings` | exit 0 |
| Opmaak | `python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path crates/artifact-store-core/Cargo.toml -- --check` | exit 0 |
| Freestanding bibliotheek | `python3 -B tools/dev.py cargo +1.90.0 build --offline --locked --release --lib --target x86_64-unknown-none -p artifact-store-core` | exit 0 |
| Witruimte in diff | `git diff --check` | exit 0 |

Het freestanding artifact was
`target/artifact-store-core-final-none/x86_64-unknown-none/release/libartifact_store_core.rlib`,
SHA-256 `18dca212ccccda6d34d8ec1400d4b5729607023c1d976792a32e0537e54d659a`.

Lokale commandologs:

- `/tmp/nieuw-os-artifact-store-core-tests.log`
- `/tmp/nieuw-os-artifact-store-core-clippy.log`
- `/tmp/nieuw-os-artifact-store-core-fmt.log`
- `/tmp/nieuw-os-artifact-store-core-none.log`

Ten tijde van deze lokale bewijsregistratie was de GitHub Actions-workflow
toegevoegd, maar nog niet op GitHub uitgevoerd. Deze controles testten of
accepteerden geen QEMU-gast, kernelbinding van de aanroeper, native NAS-/app-
integratie, schrijfbare store, duurzaamheid, cryptografisch vertrouwen of
criterium voor de volledige release.

## Herhaling na main-integratie

Main `ca45a7dabe6a040921de03bdc2fad3e5045a75ba` is zonder conflicten
inhoudsbehoudend gemerged naar
`53c2f147ad299b16d8dc0136afefe7f845cd0117`. De remote branch wees na push naar
dezelfde commit. Alle vier bovenstaande Cargo-commando's en `git diff --check`
zijn op deze mergebron opnieuw uitgevoerd: ieder exit 0, opnieuw 7 hosttests
geslaagd. Dezelfde bestaande host-toolsbundel en eigen targetmappen zijn
hergebruikt; geen installatie of `cargo clean`.

Logs van deze herhaling:
`/tmp/nieuw-os-artifact-store-53c2-{tests,clippy,fmt,none}.log`.
Dit voegt geen native runtimebewijs of rubricpunten toe; advies blijft
34% naar 34%, met A02, L01 en R01-R03 open. Centrale assessment en
PROJECT-STATE blijven bij de manager.
