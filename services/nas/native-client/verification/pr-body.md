## Doel

TASK-NAS-RESUME-1: een begrensde allocatorvrije no_std clientqueue boven de
bestaande NAS-core, zonder een tweede wire/identity-/crypto-ABI. Offline edits,
CAS-conflicten en onzekere commits behouden lokale data. Herstel stript
RAMtickets en vereist expliciete geautoriseerde reconciliatie; een waargenomen
exacte remote versie is `Observed`, niet de oorspronkelijke commit-ACK.

Aparte branch `simon/nas-resume-client`, basis oracle86a/runtimece52, geteste
bron `733da709383ce314cde63dea950125fe859f8c2a`. PR71 en de root-NAS-integratie
blijven behouden. Nieuwe vierde standalone crate staat echt in nas-core.yml.
Deze concept-PR is niet mergeklaar zonder nieuwe-head CI/integratiebesluit.

## Architectuur- en securitygrens

- [x] Relevante `AGENTS.md`-bestanden gelezen.
- [x] Geen bestaande OS-kernel als basis toegevoegd.
- [x] Capability-/privilege-/IPC-/drivergrenzen niet ongemerkt verruimd.
- [x] Nieuwe `unsafe` of dependency is expliciet gemotiveerd en gecontroleerd.

Geen unsafe; enige dependency is de bestaande lokale no_std NAS-core exact
v0.1.0, zonder extern package/buildscript. Echte core-Session/Upload en
namespace-open/CAS/revoke blijven verplicht. De opgeslagen stagingcaller
voorkomt handleverlies door misrouting, maar verleent zelf geen autoriteit.
Calleridentiteit blijft afkomstig uit vertrouwde ingress, niet een payload.
Geen SDK-/kernel-/keyring-/TLS-/provider-/centrale voortgangswrites.

## Verificatie

Alle onderstaande commando's lokaal exit 0 op bron733da70. Voor elk van
`native-core`, `native-volume`, `native-probe`, `native-client` afzonderlijk:

```sh
cargo test --offline --locked --manifest-path "services/nas/$crate/Cargo.toml"
cargo test --offline --locked --release --manifest-path "services/nas/$crate/Cargo.toml"
cargo clippy --offline --locked --manifest-path "services/nas/$crate/Cargo.toml" --all-targets -- -D warnings
cargo clippy --offline --locked --release --manifest-path "services/nas/$crate/Cargo.toml" --all-targets -- -D warnings
cargo fmt --manifest-path "services/nas/$crate/Cargo.toml" --all -- --check
cargo build --offline --locked --release --manifest-path "services/nas/$crate/Cargo.toml" --target x86_64-unknown-none
```

Debug/release: core21/22, volume10/11, probe5/5, client16/16. De volledig
uitgeschreven commando's en resultaten staan in `verification/checks.log`.
De 16 clienttests omvatten 52 before/after-commit/checkpointfoutbeelden,
CAS-conflictbewaring, caller/namespacebinding, cancel/quota/limieten en
coreherstel na revoke. Twee aparte 32-cycli-regressies bewaken de geteste en
gerepareerde AI-reviewbevinding: verkeerde caller consumeert geen tickets.

Ook alle exit 0:

```sh
python3 -B -m unittest discover -s services/nas/native-probe/tests -p test_disk_oracle.py -v
cargo clippy --offline --locked --release --manifest-path services/nas/native-probe/Cargo.toml --features native --bins --target x86_64-unknown-none -- -D warnings
python3 -B services/nas/native-probe/build.py --target-dir target/nas-client-native --output-dir build/nas-client-check/users
python3 -B tools/progress.py --check
git diff --check
git diff --exit-code
git diff --exit-code 86a9600e4f1366dbd2d9e95f50477ea353fa6dfa -- services/nas/native-core services/nas/native-volume services/nas/native-probe
```

Python 8/8; ELF-build/inspectie is geen uitvoering. Bron-/bewijshashes en het
eigen ELF-rapport staan onder `services/nas/native-client/verification/`.

- [x] Relevante unit/integratietests
- [x] Formatting/linting
- [x] Native build indien van toepassing
- [x] Negatieve test voor nieuwe trust- of bevoegdheidsgrens
- [x] Niet-uitgevoerde checks expliciet als zodanig genoemd

**Niet uitgevoerd:** GitHub-CI op de nieuwe PR-head, QEMU met deze client,
durable clientqueue, echt TLS/E2EE, multi-deviceherstel en Materiaalbediening.
De checkbox native build betekent hier uitsluitend freestanding library/
bestaande probe-ELFbuild, geen guestuitvoering. AI-codecontrole is geen
onafhankelijke menselijke audit; Release/B72's native bewijs is afzonderlijk.

## Bewijsgrens en rollback

Uitsluitend hostmodel met echte bestaande core-autorisatie/CAS en synthetische
opaque bytes. Checkpoint is een typed RAMkopie, geen on-disk-formaat of durable
ACK. Geen latest-discovery-API, crypto-authority/contextmapping, native
geauthenticeerde NAS-peer of productready syncclaim. Een nieuwe revisie moet
later opnieuw geautoriseerd worden geseald; geen ciphertext herlabelen.
Intrekking behoudt lokale clientbytes maar verleent geen NAS-toegang.
Optionele providerkeuze blijft expliciet, gescheiden van inhoudssleutels en
zonder fallback; deze queue implementeert geen provider.

Rollback: verwijder uitsluitend de nieuwe clientcrate en vierde matrixentry
via een gewone gerichte revert. Bestaande core/volume/probe/oracle, PR71 en
Release's aparte rootintegratie blijven ongewijzigd.

## Voortgang

- [x] `docs/progress.json`/README alleen aangepast wanneer de vaste acceptatiecriteria dit rechtvaardigen.

Geen centrale voortgangsbestanden gewijzigd. Assessment aan TL:
**34/100 -> 34/100, NAS 0/5 -> 0/5**. P02/P03/S09 en FR-P03/P04/P05 blijven
open; extra hosttests/CI/librarybuilds leveren geen native punten.
