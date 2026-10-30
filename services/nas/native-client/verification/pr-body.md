## Doel

Voeg een begrensde, allocatorvrije `no_std` NAS-clientqueue toe boven op de
bestaande native NAS-core. De client bewaart offline edits, houdt beide
conflictkanten vast en herstelt onzekere uitkomsten alleen via expliciete
reconciliatie; een waargenomen revisie wordt niet als oorspronkelijke ACK
voorgesteld. Deze PR is gestapeld op de actuele NAS-core/probe-PR71-head.

## Architectuur- en securitygrens

- [x] Relevante `AGENTS.md`-bestanden gelezen.
- [x] Geen bestaande OS-kernel als basis toegevoegd.
- [x] Capability-/privilege-/IPC-/drivergrenzen niet ongemerkt verruimd.
- [x] Nieuwe `unsafe` of dependency is expliciet gemotiveerd en gecontroleerd.

Er is geen `unsafe` of externe package toegevoegd; de enige dependency is de
bestaande lokale NAS-core. Caller-/namespacebinding en stale Session/Upload
blijven door de core gecontroleerd. Een verkeerde caller kan geen stagingticket
consumeren. De clientqueue is een host/RAM-model, geen duurzame autoriteit.
Geen SDK-, crypto-, TLS-, provider-, key-lifecycle-, rootrunner- of centrale
voortgangswijziging.

## Verificatie

Vermeld exacte commando's en resultaten.

- [x] Relevante unit/integratietests
- [x] Formatting/linting
- [x] Native build indien van toepassing
- [x] Negatieve test voor nieuwe trust- of bevoegdheidsgrens
- [x] Niet-uitgevoerde checks expliciet als zodanig genoemd

Op de gestapelde branch `simon/nas-resume-client-stack`, met Rust 1.90.0:

| Exact commando | Resultaat |
|---|---|
| `cargo fmt --manifest-path services/nas/native-client/Cargo.toml --all -- --check` | exit 0 |
| `cargo test --offline --locked --manifest-path services/nas/native-client/Cargo.toml` | exit 0; 16 tests |
| `cargo test --offline --locked --release --manifest-path services/nas/native-client/Cargo.toml` | exit 0; 17 tests, inclusief 64-KiB hoststack/max-queue-herstel |
| `cargo clippy --offline --locked --manifest-path services/nas/native-client/Cargo.toml --all-targets -- -D warnings` | exit 0 |
| `cargo clippy --offline --locked --release --manifest-path services/nas/native-client/Cargo.toml --all-targets -- -D warnings` | exit 0 |
| `cargo build --offline --locked --manifest-path services/nas/native-client/Cargo.toml --release --target x86_64-unknown-none` | exit 0; freestanding library, geen guestuitvoering |

De exacte probe-/volume-ELF-build en inspectie, bronhashcontrole en volledige
uitvoer zijn vastgelegd in `verification/README.md` en `stack-gates.log`.
De debug-/releasetests omvatten conflicten, offline behoud, quota, revoke,
onzekere commituitkomsten, misrouted callers en bounded queueherstel.

**Niet uitgevoerd / geen resultaat geclaimd:** disk-oracletest op deze stacked
head (de oraclebron is geen onderdeel van deze PR-diff), GitHub-CI (raadpleeg
de live PR-checks voor status), QEMU/native clientuitvoering, duurzame clientqueue,
geauthenticeerde TLS/E2EE, multi-deviceherstel en Materiaalbediening. Geen
historische check van een andere source tree wordt als deze PR-headbewijs
opgevoerd.

## Bewijsgrens en rollback

Dit is hosttest-, lint- en freestanding-buildbewijs, geen native NAS-client,
persistente clientcheckpoint, interoperabel identity-/crypto-/restorecontract
of P01–P04-acceptatie. De helper/core-interface levert geen ontbrekende
geauthenticeerde peer of sleutelautorisatie. De lokale queue is geen
onafhankelijke back-up.

Rollback: revert de clientcrate en de vierde matrixentry. De gestapelde NAS-core,
volumeadapter en IPC-probe zijn in deze PR niet gewijzigd.

## Voortgang

- [x] `docs/progress.json`/README alleen aangepast wanneer de vaste acceptatiecriteria dit rechtvaardigen.

Er is geen criterium afgetekend; rubric blijft 34/100 en NAS P01–P04 blijft
0/5.
