# NAS op actuele main94 — inhoudsbehoud en herhaalde gates

30 oktober 2026, Europe/Amsterdam. Geteste bron:
`675d4f042871e7925a412bcfb700637e434d575f`, een inhoudsbehoudende merge
van NAS-checkpoint `ba557473f5d6cce0ee944e6f19345b4056b2f157` met main
`94c8908ee363d06a5be51417c5cae635549d566f`.
De commit met deze aanvullende bewijsmap verandert geen uitvoerbare bron.
De oudere bundle één map hoger blijft bewijs voor **ba55747**, niet main94.

Lifecycle vroeg deze actualisatie; TL-manager en TL-REL zijn vooraf
geïnformeerd. Geen mergeconflict, handmatige root-/SDK-/WAITvervanging of
nieuwe centrale assessment. Na integratie bleef buiten NAS en zijn eigen
nieuwe workflow alles bytegelijk aan main94:

```sh
git diff --exit-code 94c8908ee363d06a5be51417c5cae635549d566f HEAD -- . ':(exclude)services/nas' ':(exclude).github/workflows/nas-core.yml'
```

Exit **0**. Dit is inhoudsbehoud, geen nieuwe review van alle main-code.
Het SDK-/volumedienstverschil is gericht gelezen: SDK accepteert nu ABI
1.10; de bestaande volumedienst gebruikt op dat profiel zijn bestaande
IPC-wachtset. Op oudere ondersteunde ABI's blijft zijn poll/yield-pad
bestaan. NAS behoudt zijn eigen begrensde poll/yield-probe en dezelfde
endpoints/callerbinding. IPC WAIT, TPM, rootbuild en centrale status zijn
niet door NAS vervangen. Het wachtcontract staat in
`docs/contracts/NATIVE-IPC-WAIT.md`.

## Werkelijk opnieuw uitgevoerd

`checks.log` begint met het geteste commit-ID en compilerinformatie.
Dezelfde read-only Rust-1.90.0-/cc-omgeving als de eerdere bundle,
eigen target `target/nas-core`, geen install/fetch of gedeelde writer.
Voor elk van native-core/native-volume/native-probe zijn opnieuw
locked/offline debug- en releasetests, beide Clippy-profielen met
`-D warnings`, fmt-check en `x86_64-unknown-none`-librarybuild uitgevoerd.
Alle commando's, gehele runneruitvoer, duur en exitstatus staan in de log.

Resultaten: core **21/22**, volume **9/10**, probe **5/5**, alles exit **0**.
Beide 64-KiB-stacktests, alle replace-foutcodes vóór/na commit en
210 diskfoutpunten × twee crashbeelden opnieuw groen.
De Unknown-/diagnose-/poison-/geen-ACK-/geen-replaysemantiek is ongewijzigd.

Ook opnieuw uitgevoerd, exit **0**:

```sh
cargo clippy --offline --locked --release --manifest-path services/nas/native-probe/Cargo.toml --features native --bins --target x86_64-unknown-none -- -D warnings
NOS_BUILD_DIR=build/nas-probe-main94 python3 -B services/nas/native-probe/build.py --output-dir build/nas-probe-main94/users
git diff --check
python3 tools/progress.py --check
```

De builder hergebruikt de eigen targets; afzonderlijke output bewaart de
oudere ba55747-ELF's. Hashes, segmentvalidatie en buildduur staan in
`build-report.json` en `build-nas-*.json`.

| ELF op main94 | Bestand | Gemapte ELF-pagina's | `.data` | `.bss` |
|---|---:|---:|---:|---:|
| nas-probe | 91.688 B | 81.920 B | 1.256 B | 0 B |
| nas-volume | 97.816 B | 86.016 B | 1.304 B | 0 B |

Workspace blijft **62.336 B**; bestaande 64-KiB-stack en 64-KiB-heap per
proces, geen nieuwe allocator. Totale native stackpiek en startupwoord
448 blijven **niet gemeten**. Oude ELF-hashes zijn niet opnieuw gelabeld.

## CI en overdracht

Drie afzonderlijke NAS-jobs op head **675d4f0** zijn live `SUCCESS`:
run https://github.com/simonvanieperen/nieuw-os/actions/runs/41099226351.
Jobmetadata staat in `github-nas-jobs.json`. Ook de drie banen op ba55747
waren groen; dat oudere resultaat vervangt geen gate op een nieuwe head.
De uiteindelijke PR-head wordt na het bewijscheckpoint opnieuw bekeken.

`source.sha256` bindt deze bron plus actuele SDK/volume/capabilityinputs.
`source-v3.txt` is daarnaast de bestaande roottooling-fingerprint van deze
gecombineerde bron, niet een nieuwe definitie van v3. `evidence.sha256`
bindt de bewaarde logs en rapporten.

TL-REL krijgt de nieuwe frozen source/ELF-fingerprints en bezit de nog te
maken rootmanifest-/QEMU-integratie. Geen guestcaller-/endpointbewijs
door deze host- of targetgates. **Niet uitgevoerd:** normale/koude
NAS-VM-run, native crashes, rawdisk-assertions, TLS/peeridentiteit,
E2EE, automatische sync, meerdere apparaten, rotatie/herstel en audit.
De PR blijft een component-/integratievoorbereidings-PR, geen P04-release.

Assessment voor de centrale eigenaar: **34/100 → 34/100**;
**P01–P04 0/5 → 0/5**. Geen gewijzigd criterium, gewicht of eindscope.
