# NAS-VOLUME-1 — historisch volume- en stackcheckpoint

30 oktober 2026, Europe/Amsterdam. Branch
`simonvanieperen-legendary-broccoli`; oorspronkelijke basis
`f52dab50af281dab2c94ab2c9e31aaabd299246a`, oudercheckpoint
`de8ff67ffbd44207024895afeb470c43fbced298`. Dit historische rapport en
`source.sha256` horen uitsluitend bij checkpoint
`560507ff6ccc6ae9bebdd4f0f4c8521a6321c270`, inclusief de hergebruikte
volume-/catalogcode en blockdevice-testhelper. Controleer de hashes tegen
die commit, niet tegen een latere PR-head. De hashes en onderstaande
18/19 core-testresultaten blijven ongewijzigd historisch bewijs; zij zijn
niet opnieuw gelabeld voor de latere replace-Unknown-fix.
Er zijn op dit checkpoint uitsluitend bestanden onder `services/nas/**`
gewijzigd. De latere controles, 21/22 core-tests en gerichte GitHub-CI staan in
[het main94-checkpoint](../../native-probe/verification/main94/README.md).

## Resultaat en uitgevoerde controles

Rust 1.90.0, host `x86_64-unknown-linux-gnu`. De bestaande read-only
toolchain en cc-wrapper uit de gedeelde buildomgeving zijn gebruikt; zie de
[exacte omgeving](../../native-core/verification/README.md). Eigen
`CARGO_TARGET_DIR=$PWD/target/nas-core`; geen install/fetch, systeemwijziging,
gedeelde buildmap of gedeelde bronwriter.

Onderstaande commando's zijn letterlijk uitgevoerd voor beide manifesten.
Volledige gemeten uitvoer: `native-core-final.log`, `native-volume-final.log`.
Dezelfde lokale logs staan onder `target/nas-core/logs/`.

| Commando (`M=services/nas/<crate>/Cargo.toml`) | Core: exit / wandtijd | Volume: exit / wandtijd |
|---|---|---|
| `cargo test --offline --locked --manifest-path "$M"` | 0 / 0,367 s; 18 tests | 0 / 1,167 s; 9 tests |
| `cargo test --offline --locked --release --manifest-path "$M" -- --nocapture` | 0 / 0,840 s; 19 tests | 0 / 1,346 s; 10 tests |
| `cargo clippy --offline --locked --manifest-path "$M" --all-targets -- -D warnings` | 0 / 0,278 s | 0 / 0,275 s |
| `cargo clippy --offline --locked --release --manifest-path "$M" --all-targets -- -D warnings` | 0 / 0,205 s | 0 / 0,402 s |
| `cargo fmt --manifest-path "$M" --all -- --check` | 0 / 0,034 s | 0 / 0,454 s |
| `cargo build --offline --locked --manifest-path "$M" --release --target x86_64-unknown-none` | 0 / 0,175 s | 0 / 0,169 s |

`python3 tools/progress.py --check` en `git diff --check`: exit 0.
De voortgangscontrole meldt 34/100, rubric v1 consistent; zij valideert geen
OS-gedrag. **Op dit historische checkpoint** waren de twee nieuwe workspaces
nog niet aangesloten op CI: `.github/**` viel toen buiten het afgesproken
schrijfgebied. Later is `.github/workflows/nas-core.yml` toegevoegd voor drie
NAS-crates; de bijbehorende brongebonden uitvoering staat in het hierboven
genoemde main94-checkpoint. Dat latere bewijs vervangt de historische
controles in dit rapport niet.

De volume-integratie hergebruikt de echte `volume_core::client::Client`,
volumedienst, canonieke wirevalidator en het bestaande blockdevice-model.
De tests controleren cold recovery van daadwerkelijk geschreven
modelsectoren, quota-/domeinweigering, verloren COMMIT/PURGE-antwoorden,
corruptie, stale writers, cleanup zonder automatische herhaling en herstel
zonder oude gastautoriteit. Alle **210** muterende schijffoutpunten in een
eviction zijn geïnjecteerd met twee crashbeelden per punt (420 beelden),
inclusief verloren/onvolledige writes. Dat zijn modelproeven, geen 420
onafhankelijke hardwaretests of bewijs voor iedere denkbare crashvolgorde.

## Werkelijk gevonden en verholpen blokkade

De eerste geoptimaliseerde 64-KiB-stackproef van de by-value-core eindigde
met exit 101 / SIGABRT: `thread '<unknown>' has overflowed its stack`.
Command:

```sh
cargo test --offline --locked --release --manifest-path services/nas/native-core/Cargo.toml --test authority optimized_core_fits_the_existing_64_kib_native_stack -- --exact
```

Die eerste fout staat in de sessietooluitvoer, niet in een achteraf
gefabriceerd raw log. Grote state-/transactiekopieën en snapshotbuffers leven
nu in een exclusief geleende, caller-owned `Workspace`, niet op de stack.
De snapshotdecoder schrijft rechtstreeks in de aangewezen state. Geen
kleinere opslagcaps, verhoogde native stacklimiet, allocator of `unsafe`
toegevoegd om het resultaat groen te maken.

De uiteindelijke geoptimaliseerde hostproeven slagen op daadwerkelijk
64-KiB-threadstacks: core plus geheugenstore, en core plus volumeclient met
een aparte volumedienstthread. Gemeten workspace: **62.336 bytes**,
`Service<Memory>`: **16 bytes**. Dat bewijst het onderzochte host-callpad,
niet het uiteindelijke native adapter-/IRQ-/IPC-stackmaximum.

## Assessment en resterende grenzen

**P01–P04: 0/5 → 0/5; totaal 34/100 → 34/100.** Dit bouwt aan P04 maar
sluit geen acceptatiecriterium. De gedeelde status-/voortgangsbestanden zijn
op expliciet scopeverzoek niet gewijzigd; dit +0-assessment is overdracht
aan hun eigenaar, geen alternatieve roadmap.

**Niet uitgevoerd:** native NAS-proces, afzonderlijk kernelgebonden
beheer-/gastingress, native QEMU/OVMF-run/koude image/crash, verkeer/TLS,
productie-entropy of betrouwbare pairingtijd, E2EE-client, automatisch
offline/conflictsync, LAN/internetwisseling, apparaten toevoegen/intrekken,
sleutelrotatie/herstel, UI, CI en menselijke audit. De publieke principal
is nog een vertrouwde adapterinvoer; het lokale invitation-tuple is geen
interoperabel token of herstelcontract en gebruikt het B02-fixture niet.

Het dedicated volumeobject moet exclusief blijven en voldoende COW-ruimte
houden. Gewone `load` schrijft nooit; achtergebleven geschiedenis blokkeert
herstel tot een expliciete `finish_cleanup` de huidige canonieke versie
heeft gevalideerd en oude versies heeft vrijgegeven. Geen fallback op een
oudere autoriteit. Logische purge is geen fysieke secure erase; checksum
is geen MAC en diskrollback is niet opgelost.

Voor verdere native acceptatie is gecoördineerde manifest-/IPC-/build- en
transportintegratie buiten `services/nas/**` nodig. Zonder verruiming van
het huidige schrijfgebied wordt die grens niet stil overschreden.
Rollback van dit checkpoint: revert de NAS-commits; er is nog geen
productiedata, native manifest, ABI of bestaand diskformaat gemigreerd.
