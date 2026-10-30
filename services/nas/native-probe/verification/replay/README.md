# TASK-NAS-EXACT-REPLAY-PROBE — leafpayload en root-handoffgrens

## Toewijzing

Taak door teamleider werkqueue definitief toegewezen in `queueb58925f9ce1a9a4d045ba0753f007e7d070be7c2`.
Branch `simon/nas-exact-replay-probe`, basis de bevroren probe
`ce52dd6165ba7af3bdcd1533308b556d1927443d`.
Toegestane files: `services/nas/native-probe/src/main.rs`,
`CONTRACT.md`, `tests/**`, `verification/replay/**`. Release houdt alle
root/runner/manifest-/VM-writes; diens bestaande #83-profiel is bevroren.
Geen workflow, core, volume, oracle, kernel, SDK, crypto, identity, durable
restore- of productwirewijziging.

## Leaf startupinput

Op de normale guest-run zijn bytes `[256,336)` nul. De guest cold-run vereist
een exact 80-byte-paar:

| Offset op de bestaande private guest-startuppagina | Bytes |
|---|---|
| 256..296 | exact 40-byte Session reply uit de normale boot |
| 296..336 | exact 40-byte Upload reply uit de normale boot |

De leaf controleert uitsluitend canonieke tokenvorm en voert daarna READ met
de echte oude Session uit (`Stale`) en COMMIT met beide oude bytes
(`Stale`). De tweede native status is een weigering van de combinatie via de
core, omdat de core eerst de oude Session afwijst. De hosttest isoleert de
Upload-controle afzonderlijk door die met een verse actieve Session te
gebruiken na core recovery. Geen productie-API of autoriteitsclaim.

De offsets zijn een concreet verzoek aan Release voor zijn toegewezen
bootstraprunner; zij zijn nog niet in diens manifest/runner gebonden. Geen
huidige runner voert de cold exact-replay assertions uit. Ontbrekend of
ongeldig input faalt de cold guest op foutstap 47/48, zodat een koude
succesmelding zonder herkomstbewijs niet mogelijk is.

## Vereiste private host-handoff

- Een voltooide **normale native** run, exacte broncommit/profile, guest-PID
  uit manifest, Stage geslaagd, capture na Stage en origin-run-ID.
- Ruwe payload exact `[Session40 || Upload40]`, 80 bytes.
- Apart JSON-manifest: schema 1, source commit, origin run ID, role `guest`,
  origin stage `normal`, capture stage `normal-after-stage`, `completed=true`,
  length 80, SHA-256 van payload. Geen ruwe tokenbytes of geheimen in manifest.
- Alleen private runmap buiten repository: directory `0700`, files `0600`,
  reguliere files, geen symlink/hardlink, bounded read.
- Host maakt een afzonderlijke `0400` cold-inputkopie; vergelijkt payload-hash
  voor injectie en na cold run; mismatch, ontbrekende file, verkeerde bron/rol/
  stage/volledigheid/lengte weigert het cold profiel.
- Artifact/input nooit serial, normale logs, standaard CI artifact of Git.
  Geen user-provided/environment token fallback. De input wordt byte-voor-byte
  op offsets 256..336 in de bestaande cold guest startup-pagina gezet.

De helper `tests/replay_handoff.py` bindt manifestbron en run-ID aan de
verwachte waarden die de rootrunner uit zijn geverifieerde normal-run selecteert,
controleert de minimale metadata-/file-/mode-/hashvoorwaarden en maakt een
read-only cold input. Tests gebruiken
uitsluitend expliciete structurele placeholderbytes; ze parsen die niet als
tokens en sturen ze nooit naar NAS-core. De Rusthostproef genereert daarentegen
echte core-issued Session/Upload handles in geheugen en bewijst afzonderlijk
de core recoveryweigeringen. Geen tokenbytes worden afgedrukt of persistent
opgeslagen door die proef.

## Status en bewijsgrens

Dit leaf levert de bounded startup-consumer en hostnegatieven. Het maakt niet
zelf het private hostartifact aan, kan de guest RAM niet uit QEMU halen en
wijzigt geen rootrunner. Release heeft bevestigd dat huidige tools geen
80-byte normal-output/cold-inputbinding bieden. Daarom is een nieuwe
native exact-replay-run expliciet **geblokkeerd** tot een afzonderlijk
gefrozen Release-profile/hostbinding en passende token-handoffimplementation.
Geen VM-slot is vrijgegeven en geen native PASS wordt geclaimd.

**Root-counterwijziging:** cold guest doet nu twee verwachte `Stale`-calls:
`READ` met de boot-1 Session en `COMMIT` met de boot-1 Session+Upload.
Daarom stijgt alleen de cold-gast `error_replies`-verwachting met één
(historische fixturebaseline 4, nieuwe leafverwachting 5). De Commit-uitkomst
is nog steeds de bestaande core-short-circuit op de stale Session; zie boven
voor de afzonderlijke hosttest die de oude Upload met een verse Session toetst.

Oude cold run met `[42;40]` blijft alleen fixturetest. Dit werk bewijst niet
dat uploadtokens buiten de bijbehorende oude sessie kunnen worden gevalideerd
door de native cold COMMIT; die core-call short-circuitt op stale Session.
De zelfstandige host-coreproef dekt stale Upload met een nieuwe geldige
sessie. Beide claims blijven gescheiden.

Gerichte controls:

```sh
cargo test --offline --locked --manifest-path services/nas/native-probe/Cargo.toml
cargo test --offline --locked --release --manifest-path services/nas/native-probe/Cargo.toml
cargo clippy --offline --locked --manifest-path services/nas/native-probe/Cargo.toml --all-targets -- -D warnings
cargo clippy --offline --locked --release --manifest-path services/nas/native-probe/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path services/nas/native-probe/Cargo.toml --all -- --check
cargo build --offline --locked --release --manifest-path services/nas/native-probe/Cargo.toml --target x86_64-unknown-none
python3 -B -m unittest discover -s services/nas/native-probe/tests -p test_replay_handoff.py -v
```

Finale exacte logs/hashes en resultaten staan naast dit bestand. Host/core-/
crosscompile-resultaten zijn geen QEMU-, identity-, E2EE-, persistentie- of
P01–P04-acceptatie.

## Leaf-gates op de replaybranch

Uitgevoerd op de kandidaatbron in branch `simon/nas-exact-replay-probe`,
basis `ce52dd6165ba7af3bdcd1533308b556d1927443d`, met Rust 1.90.0. Elke
onderstaande gerichte gate eindigde met exit 0:

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

Debug- en release-hosttests: elk 7 tests geslaagd (5 bestaande IPC,
2 replay/core). Handoffnegatieven: 9 tests geslaagd. Debug/release/native
Clippy en de bare-metal library-/binary-builds slaagden. De builder
inspecteerde beide ELF's; report `build-report.json` was gemarkeerd
`native_execution=false`:

| ELF | Bytes | SHA-256 |
|---|---:|---|
| `nas-probe.elf` | 99.352 | `95d01c74608decf387eed326f521082340a1ebea0ed0b61d426cf691198fa7a9` |
| `nas-volume.elf` | 97.976 | `251a1caf4e1e3f6879856e62b9fee0372a1fb6c4fa0e62f3f85d12e46c450989` |

Volledige gate-uitvoer staat in `leaf-gates.log`, SHA-256
`f29099425cc894ee82155e18340ea80386482ffdc0acecceb9cfda5eedf550c0`.
Een eerste verkennende unittest-selectie voor `test_disk_oracle.py` eindigde
met exit 5 omdat deze leaf geen tests met die naam bevat; er is daarom geen
disk-oracletest geclaimd. **Niet uitgevoerd:** QEMU/native exact-replay-run,
omdat de rootrunner de normale geheime RAM-bytes niet exporteert en geen
cold-input bij startup bindt. Deze gates sluiten geen P01–P04- of releasepoort.
