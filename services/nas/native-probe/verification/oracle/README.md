# TASK-NAS-ORACLE-1 — onafhankelijk schijforakel voor de vaste proef

30 oktober 2026, Europe/Amsterdam. Afzonderlijke leaf
`simon/nas-native-oracle`, basis `ce52dd6165ba7af3bdcd1533308b556d1927443d`.
De bevroren #71-bron, NAS-core/service, startupcontract en ELF's worden
hierdoor **niet** gewijzigd. Deze leaf voegt alleen hosttooling/tests en
de bijbehorende stap aan de eigen NAS-workflow toe.

## Resultaat en gebruik door TL-REL

`disk_oracle.py` leest uitsluitend één reguliere synthetische 1-MiB-image,
met vaste volumegrens 580 sectoren vanaf sector 4. Het gebruikt bestaande
`tools/volume_disk.py` voor tabelherstel, checksums, versiegeschiedenis,
chunkrechten/overlap, domeinen en canaries. Geen tweede filesystemdecoder.

```sh
python3 -B services/nas/native-probe/disk_oracle.py IMAGE --admin-pid ADMIN_PID --guest-pid GUEST_PID
python3 -B services/nas/native-probe/disk_oracle.py COLD_IMAGE --admin-pid ADMIN_PID --guest-pid GUEST_PID --normal-sha256 NORMAL_IMAGE_SHA256
```

PID's moeten uit het werkelijke kernelmanifest komen; geen principal uit
een gastpayload vertrouwen. Succes exit 0 met een compact JSON-rapport,
fout exit 2 zonder succes-JSON. Geen raw snapshot, namespace, tokens of
`content_hex` in het rapport. De invoerimage wordt niet beschreven.
De cold SHA is de volledige imagehash ná de normale run; niet de lege
beginfixture. Gebruik de CLI of `check_image(bytes, admin_pid, guest_pid,
normal_sha256=None)` vanuit de rootrunner.

De vaste eindtoestand is volledig gereconstrueerd: `NOSNAS01` v1,
coregeneratie 6, juiste admin-/gastprocesprincipals, één niet-lege
namespace met `Evicted`, epoch 2, quota 128, lege uitnodiging/aflooptijd,
alle objectbytes en overige namespaces/nulvelden nul. Volumeobject 0
heet `nas-authority-v1`, heeft exact één huidige snapshot en geen history;
alle overige objecten/collecties zijn onaangeraakt. Zes snapshotpublicaties
en vijf historypurges geven volumegeneratie/domeinklok 11/`[11,0]`.
Een echte Rust-core+VolumeStorage+volumedienst-hostproef controleert die
telling, dus niet uitsluitend een met Python verzonnen golden fixture.

Een enorme maar canoniek verzegelde revisie wordt vóór de bestaande
fixturezoeklus geweigerd. Onbetrouwbare u64-revisies mogen die hostlus niet
praktisch onbegrensd laten lopen. Dit is een expliciet vast-proeforakel,
geen productiecapaciteitslimiet of algemene NAS-snapshotdecoder.

## Uitgevoerde controles

Read-only bestaande Rust 1.90.0-/cc-binaries, eigen leaf-target
`target/nas-host`; Python 3 standaardbibliotheek, geen nieuwe dependency,
install/fetch, echte NAS, persoonlijke disk of productiekey.
Exacte commands, resultaten en looptijden: `checks.log`.

```sh
cargo test --offline --locked --manifest-path services/nas/native-volume/Cargo.toml
cargo test --offline --locked --release --manifest-path services/nas/native-volume/Cargo.toml
cargo clippy --offline --locked --manifest-path services/nas/native-volume/Cargo.toml --all-targets -- -D warnings
cargo clippy --offline --locked --release --manifest-path services/nas/native-volume/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path services/nas/native-volume/Cargo.toml --all -- --check
python3 -B -m unittest discover -s services/nas/native-probe/tests -p 'test_disk_oracle.py' -v
git diff --check
```

Alles exit **0**: volume **10 debug/11 release**, inclusief nieuwe
Rust→Python-oracleproef, 210×2 diskfoutbeelden en bestaande 64-KiB-stacktest;
**8 Python-tests** met onder andere alle 4.160 objectbytes afzonderlijk
gemuteerd, schema/identiteit, restgeschiedenis, andere objecten, gewijzigde
canaries/chunks/cold SHA, verkeerde lengte en enorme revisies.
CLI-success/failure bewaren de oorspronkelijke bytes; de Rust-hostproef
heropent de echte volume-uitvoer zonder mutaties en laat die onafhankelijk
door Python controleren. Tijdelijke testfiles worden gericht opgeruimd.

`source.sha256` bindt de gewijzigde/nieuwe bron plus hergebruikte helpers.
`evidence.sha256` bindt de bewaarde runneruitvoer.
De workflow gebruikt geen missing-file-skip; Python-hostnegatieven draaien
in de bestaande probejob. GitHub-CI op deze aparte leaf is **niet uitgevoerd**.

## Grenzen en overdracht

Geen bewijs van gastuitvoering, werkelijke IPC/capabilityafdwinging,
geheugenpiek, RAM-reserveringen, sessie-invalidering, TLS, E2EE of sync.
Het rapport zegt daarom expliciet `guest_execution: not-assessed`.
De daadwerkelijke normal/cold-QEMU-run, rolstatus/counters en fingerprints
van de samengestelde rootbron blijven bij TL-REL. Verwijderde ciphertext kan
fysiek in niet-actieve sectoren/tabelkopieën achterblijven; dit is uitsluitend
logische onbereikbaarheid, geen secure erase of antirollback.

Het orakel is een extra onafhankelijke hostimplementatie, geen onafhankelijke
menselijke audit. **P01–P04 0/5 → 0/5; totale score 34/100 → 34/100.**
De manager beheert centrale status/progress; geen centrale files gewijzigd.
Rollback: deze hostleaf/stap weglaten; geen runtimeformaatmigratie.
