# NAS-IPC-PROBE-1 / replace-Unknown / gerichte CI

Datum: 30 oktober 2026, Europe/Amsterdam. Branch:
`simonvanieperen-legendary-broccoli`, basis
`109c9f0217a163c8836e5788a1a12a941a2e2627`; geteste bron is het
checkpoint met deze bundle. `source.sha256` bindt alle betrokken NAS-bron,
lokale afhankelijkheden, linkers, SDK-bron en de nieuwe workflow.
De oudere bundles onder native-core/native-volume blijven historisch
bewijs voor hun **eigen** bronversies.

## Resultaat en bewijsgrens

Alle onderstaande lokale controles eindigden met exit **0**, zie
`checks.log`. Tests zijn werkelijk uitgevoerd, ook bij een gecachte build.
Een GitHub-jobresultaat bestaat pas na de push en wordt afzonderlijk
gerapporteerd; de lokale loop is niet als GitHub-CI gelabeld.

| Workspace | Debugtests | Releasetests | Aanvullend |
|---|---:|---:|---|
| native-core | 21 | 22 | alle 13 replace-foutcodes vóór/na commit; initialisatie met Io/Denied/Corrupt/Unknown vóór/na commit; 64-KiB-stack |
| native-volume | 9 | 10 | 210 muterende diskfoutpunten × twee crashbeelden; 64-KiB-stack |
| native-probe | 5 | 5 | caller/ingress, gehele wire-workload/herstel, ongeldige envelopes/replies, PIT-reset |

`Storage::replace` heeft geen “Io betekent zeker niet gecommit”-garantie.
Daarom worden **alle** vervangingsfouten `Unknown`, zowel bij initialize als
persist. Persist blijft poisonen; de oorspronkelijke oorzaak blijft via
`last_storage_error` zichtbaar als foutcode, niet als gevoelige payload.
Negatieven bewijzen geen succes-ACK, geen replay, ongewijzigd aantal writes
na weigering/heropenen, en alleen de mogelijke oude/nieuwe duurzame toestand.
Pre-write/read/validatiefouten krijgen hierdoor geen verzonnen commitstatus.

Geen nieuw kernel-/SDK-ABI, syscall, capabilityrecht, netwerkprotocol,
TLS-library of cryptografie. De lokale kernelprincipal in deze probe is
**niet** een cryptografisch geauthenticeerd peeraccount. Synthetische
opaquebytes zijn **geen E2EE-bewijs**.

## Exacte uitgevoerde commando's

Read-only bestaande binaries van de door Simon verstrekte toolprefix,
geen install/fetch of gedeelde targetwriter:

```sh
P=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools/nieuw-os
export PATH="$P/rustup/toolchains/1.90.0-x86_64-unknown-linux-gnu/bin:$P/cargo/bin:$PATH"
export LD_LIBRARY_PATH="$P/host-cc/usr/lib/x86_64-linux-gnu"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$P/cargo/bin/cc"
export CARGO_TARGET_DIR="$PWD/target/nas-core"
export NOS_BUILD_DIR=build/nas-probe
```

Voor elk van `native-core`, `native-volume`, `native-probe`, met
`manifest=services/nas/$crate/Cargo.toml`:

```sh
cargo fmt --manifest-path "$manifest" --all
cargo test --offline --locked --manifest-path "$manifest" -- --nocapture
cargo test --offline --locked --release --manifest-path "$manifest" -- --nocapture
cargo clippy --offline --locked --manifest-path "$manifest" --all-targets -- -D warnings
cargo clippy --offline --locked --release --manifest-path "$manifest" --all-targets -- -D warnings
cargo fmt --manifest-path "$manifest" --all -- --check
cargo build --offline --locked --release --manifest-path "$manifest" --target x86_64-unknown-none
```

Ook uitgevoerd:

```sh
cargo clippy --offline --locked --release --manifest-path services/nas/native-probe/Cargo.toml --features native --bins --target x86_64-unknown-none -- -D warnings
python3 -B services/nas/native-probe/build.py
git diff --check
python3 tools/progress.py --write
python3 tools/progress.py --check
```

Alle exit 0. `--write` is in de eerdere controleloop uitgevoerd; de
bewaarde definitieve log herhaalt alleen `--check`.
De voortgangsgenerator leverde **geen gewijzigde centrale
bestanden**: 34/100 bleef consistent. De manager bezit de centrale
assessment/status; deze bundle levert daarvoor **P01–P04 0/5 → 0/5**,
geen nieuwe acceptatiecriteria. Geen gewijzigde gewichten of eindscope.

`build-nas-*.json` bevat werkelijke buildduur, commandoregel en exitstatus.
`build-report.json` bevat de door bestaande `inspect_user` gecontroleerde
ELF-segmenten en hashes. `checks.log` bevat runnerresultaten en
testlooptijden; alleen trailing whitespace is uit deze tekstkopie verwijderd.
De ruwe uitvoer blijft lokaal in `target/nas-native/logs/final-checks.log`.
Deze tijden zijn geen end-to-end productperformanceclaim.

## Footprint op deze bron

| Artefact | ELF-bestand | Gemapte ELF-pagina's | `.data` | `.bss`/zero-fill |
|---|---:|---:|---:|---:|
| nas-probe.elf | 91.688 B | 81.920 B | 1.296 B | 0 B |
| nas-volume.elf | 93.568 B | 81.920 B | 1.304 B | 0 B |

Beide images vragen de bestaande gereserveerde 64-KiB-userstack en
64-KiB-private heap. De NAS-coreworkspace is op de host **62.336 B**,
`Service<Memory>` 16 B; geen allocator. Native arena-size/alignment wordt
bij startup opnieuw gecontroleerd. De andere probe-rollen gebruiken de
heap niet. Het oorspronkelijke volume-ELF heeft geen nieuw arenaontwerp.

Disassembly van de geoptimaliseerde server laat directe constructorstores
naar `0x60000000` en een proloog met `sub $0x948,%rsp` plus zes registerpushes
zien, **geen** 62-KiB-stacktemporary voor de constructor. Dit bewijst niet de
totale geneste native stackpiek. Startupwoord 448 levert na echte
probe-uitvoering de aangeraakte stack-high-water; **nog niet uitgevoerd**.
Ook arena-/stackruntimepiek en foutpaden in QEMU blijven open.

Native ELF-hashes staan in `build-report.json`; er zijn geen binaries
gecommit. Volledige startup-/endpoint-/wire-/recoverybeschrijving:
[CONTRACT.md](../CONTRACT.md).

## CI, integratie en resterende risico's

Eén nieuwe exclusief aan NAS toegewezen workflow:
`.github/workflows/nas-core.yml`. Vooraf gemeld aan buildteam/REL;
manager heeft opname van **alle drie** werkelijk aanwezige standalone
crates expliciet vrijgegeven. Gepinde checkout en Rust 1.90.0 volgen het
bestaande patroon. Drie afzonderlijke host/targetjobs, zonder
bestaat-het-bestand-/missing-tool-skips; de extra ELF-stap hoort alleen bij
de probejob. Bestaande root-CI is niet gewijzigd.

**Niet uitgevoerd:** QEMU-manifest-/endpointintegratie, normale/koude
guestassertions, diskhash/readonly-recovery in de VM, native crashes,
twee-VM-transport, TLS/peeridentiteit, clientencryptie, automatische
sync/offlineconflicten, apparatenbeheer/rotatie/herstel en menselijke audit.
Die grenzen blijven open; de testnamen of geslaagde crosscompile sluiten
P01–P04 noch FR-P01–FR-P06.

TL-REL bezit de rootintegratie en krijgt dit bevroren artefactcontract.
NET bezit TCP-listen/TLS/authenticated-peer; geen veilige native
NAS-transport-API is hier verzonnen. Lifecycle beheert #71-integratie;
de PR blijft concept. Een reviewthread wordt pas na geteste push opgelost.

Rollback: laat deze probe buiten het kernelmanifest; de nieuwe core en
workflow zijn afzonderlijk terug te draaien. Geen migratie, persoonlijke
disk, permanente NAS, productiekeys of extern opslagaccount gebruikt.
