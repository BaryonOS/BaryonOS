## Doel

Voeg de toegewezen leaf-consumer toe voor exacte boot-1 replaycontrole: de cold
NAS-gast leest een normale-run Session (40 bytes) en Upload (40 bytes) uit de
bestaande private startup-pagina en vereist dat core-herstel oude autoriteit
weigert. De private-handoffvalidator bindt de payload aan de door de rootrunner
verwachte broncommit en origin-run-ID.

## Architectuur- en securitygrens

- [x] Relevante `AGENTS.md`-bestanden gelezen.
- [x] Geen bestaande OS-kernel als basis toegevoegd.
- [x] Capability-/privilege-/IPC-/drivergrenzen niet ongemerkt verruimd.
- [x] Nieuwe `unsafe` of dependency is expliciet gemotiveerd en gecontroleerd.

Geen syscall, endpoint, wire-versie, capability, crypto-, TLS-, identity- of
productrestore-interface toegevoegd. De helper verleent geen tokenautoriteit.
De cold `COMMIT` wordt door de bestaande core eerst op de stale Session
afgewezen; de hosttest toetst stale Upload afzonderlijk met een verse Session.
De Release-runner moet zijn cold guest `error_replies`-verwachting van 4 naar 5
aanpassen.

## Verificatie

Vermeld exacte commando's en resultaten.

- [x] Relevante unit/integratietests
- [x] Formatting/linting
- [x] Native build indien van toepassing
- [x] Negatieve test voor nieuwe trust- of bevoegdheidsgrens
- [x] Niet-uitgevoerde checks expliciet als zodanig genoemd

Exit 0:

```sh
cargo fmt --manifest-path services/nas/native-probe/Cargo.toml --all -- --check
cargo test --offline --locked --manifest-path services/nas/native-probe/Cargo.toml
cargo test --offline --locked --release --manifest-path services/nas/native-probe/Cargo.toml
cargo clippy --offline --locked --manifest-path services/nas/native-probe/Cargo.toml --all-targets -- -D warnings
cargo clippy --offline --locked --release --manifest-path services/nas/native-probe/Cargo.toml --all-targets -- -D warnings
cargo clippy --offline --locked --release --manifest-path services/nas/native-probe/Cargo.toml --features native --bins --target x86_64-unknown-none -- -D warnings
cargo build --offline --locked --release --manifest-path services/nas/native-probe/Cargo.toml --target x86_64-unknown-none
python3 -B -m unittest discover -s services/nas/native-probe/tests -p test_replay_handoff.py -v
NOS_BUILD_DIR="$PWD/build/nas-replay-leaf" python3 -B services/nas/native-probe/build.py --target-dir "$PWD/target/nas-replay-elf" --output-dir "$PWD/target/nas-replay-elf-report"
git diff --check
```

Debug- en release-Rusttests: 7/7 elk. Private handoffnegatieven: 9/9. ELF's
zijn gebouwd en geïnspecteerd; hashes en volledige uitvoer staan in
`verification/replay/README.md` en `leaf-gates.log`.

Niet uitgevoerd: QEMU/native exact-replay-run, omdat Release nog geen private
normale-run tokenexport of cold-startupbinding heeft. Een eerdere verkennende
selectie voor `test_disk_oracle.py` eindigde met exit 5 (geen tests met die
naam in deze leaf); er wordt geen disk-oracle-resultaat geclaimd.

## Bewijsgrens en rollback

Dit is host-, lint-, crosscompile- en ELF-inspectiebewijs, geen native
replay-pass en geen P01–P04-acceptatie. De rootrunner blijft eigenaar van
private boot-1 RAM-capture, herkomstcontrole, cold-inputinjectie en VM-slot.
Rollback: revert deze leafcommit; geen bestaand productwire of persistent
formaat migreert.

## Voortgang

- [ ] `docs/progress.json`/README alleen aangepast wanneer de vaste acceptatiecriteria dit rechtvaardigen.

Geen criterium is door deze leaftest alleen afgetekend; de centrale score is
niet gewijzigd.
