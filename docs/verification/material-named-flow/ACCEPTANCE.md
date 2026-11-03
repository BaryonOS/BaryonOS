# Materiaal — documentnaam, bewaren, sluiten, zoeken en heropenen

Datum: 3 november 2026 (Europe/Amsterdam). Bronbasis:
`21d14592a54a80da2debf7cfad139d37264078c7`, productcommit
`34d23e8f5a23077e9f92d416efcb44615d7e9858`, branch
`simonvanieperen-animated-garbanzo`.

## Productresultaat en grens

De bestaande native app toont nu een bewerkbare documentnaam en lokale
naamzoeker. Een gebruiker kan maken, benoemen, typen, bewaren, teruggaan,
zoeken en dezelfde naam/inhoud heropenen. F2 selecteert de naam, Enter gaat
naar inhoud; Ctrl+F focust zoeken, Enter opent het eerste resultaat.
Er zijn geen nieuwe grants, opslagprotocollen of afhankelijke authwijzigingen.

De nieuwe app-hostproef gebruikt echte PS2 Set1-bytes, INP1/input-core,
capability-engine, volume-v7-service en SUR1/displayrecorder. Na save/open
sluit de app alle zes eigen endpoints; een nieuwe app en volume-service
starten met uitsluitend gekopieerde bewaarde RAM-sectoren. De gereconstrueerde
volumetabel, gezochte naam en inhoud zijn identiek; de tweede app heeft geen
savebevestiging nodig en sluit schoon. Dit is een hosttegenpartij, GEEN
native disk-, encrypted-storage-, hardware- of menselijke gebruikstest.

## Uitgevoerd

Hostomgeving voor onderstaande Cargo-commando's:
`PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH`
en `CC=/tmp/nieuw-os-c-toolchain/bin/cc`. Bestaande tijdelijke linkerwrapper;
geen nieuwe dependencies of systeeminstallatie.

| Commando | Resultaat |
|---|---|
| `cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml --test named_documents --test interaction --test render_regressions` | Exit 0: 49 tests op gepushte productbron, 6,32 s wall time. |
| `cargo +1.90.0 test --offline --locked --manifest-path ui/native/Cargo.toml --test named_documents` | Exit 0: 5 tests, inclusief aanvullende revoke-tegenproef. |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --test chain --test semantic_actions` | Exit 0: 39 + 2 tests. |
| `cargo +1.90.0 clippy --offline --locked --manifest-path ui/native/Cargo.toml --all-targets -- -D warnings` | Exit 0. |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --all-targets -- -D warnings` | Exit 0. |
| `cargo +1.90.0 fmt --manifest-path ui/native/Cargo.toml -- --check` | Exit 0. |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/materiaal/Cargo.toml -- --check` | Exit 0. |
| `PATH=/home/simon/.cargo/bin:$PATH python3 -B sdk/examples/materiaal/build.py` | Exit 0: scalar/no_std `x86_64-unknown-none` executable, eerste build compileerde de geraakte crates; Cargo meldde 2,35 s. |
| `NOS_MATERIAAL_ELF="$PWD/build/sdk-materiaal/x86_64-unknown-none/release/materiaal-app" cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/materiaal/Cargo.toml --features verify-native-elf --test elf -- --nocapture` | Exit 0: 1 test, 0,46 s wall time. De echte kernelparser accepteert 201 pagina's, vijf begrensde W^X LOAD-segmenten en drie private 64-KiB BSS-buffers. |

De vijf nieuwe UI-scenario's behandelen ook readonly-search, filter versus
slotallocatie, UTF-8/veldgrenzen op 100/200%, stale semantiek, discard,
onbekende/ongeldige naam, busy save, I/O en revoke zonder publicatie.
De bestaande 37 interactie- en 7 renderregressies blijven groen.
Aanvankelijke rode proeven vonden veldinvoer onder een verkeerde Ctrl-branch,
oude nodetellingen en een onjuiste tabvolgorde; deze zijn hersteld.

Logs: `/tmp/nieuw-os-named-material-ui-final.log`,
`/tmp/nieuw-os-named-material-final.log`,
`/tmp/nieuw-os-named-material-app-final.log`,
`/tmp/nieuw-os-named-material-native-build.log`,
`/tmp/nieuw-os-named-material-elf.log`.
Deze lokale logs zijn geen duurzaam releasebewijs.
Compacte bronhashes en executablehash/ELF-headers staan naast dit document.

## Open bewijsgrenzen

Native scalar-appbuild en host-ELF-parser zijn uitgevoerd, maar bewijzen
geen uitvoering van de nieuwe bediening in de VM.
Nieuwe QEMU-runtime, fysieke hardware, koude versleutelde persoonlijke opslag,
menselijke bediening, onafhankelijke audit en volledige releasepoort zijn
NIET uitgevoerd. B02/persoonlijke keyauthority en profielnamespace blijven
externe dependencies. HC-leaf `16c26c1c` is niet geintegreerd.
`command -v qemu-system-x86_64` vond hier geen executable op PATH.
De legacy-appwrapper `sdk/examples/materiaal/verify.py` is niet gedraaid:
zijn reeds bestaande vaste verwachting van 20 hostscenario's is achterhaald
(de huidige chain heeft 39 tests). De gerichte Cargo-commando's hierboven zijn
wel uitgevoerd; dit is geen groene wrapper-/CI- of releaseclaim.

Score: **39 -> 39/100**. Geen rubriccriterium is door hostbewijs gesloten.
Rollback van deze slice betreft uitsluitend de betrokken product-/testdelta;
bestaande volume-v7- en capabilitycontracten blijven intact.
