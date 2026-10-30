# TASK-NAS-RESUME-1 — eigen broncheckpoint

Basis `86a9600e4f1366dbd2d9e95f50477ea353fa6dfa`
(oracle-leaf, runtime ce52). Werkbranch `simon/nas-resume-client`.
Exclusieve managerclaim: nieuwe `services/nas/native-client/**` en vierde
matrixentry in `.github/workflows/nas-core.yml`; vooraf aan build afgestemd.
Geen centrale status/queue/score-, SDK-, crypto-, transport- of rootwrites.

Historisch eerste broncheckpoint `bada7de2d128fa6df9efdc4ff9cb44498797ac45`,
Rust 1.90.0, eigen target
`target/nas-client-host`:

```text
cargo generate-lockfile --offline --manifest-path services/nas/native-client/Cargo.toml => exit 0
cargo fmt --manifest-path services/nas/native-client/Cargo.toml --all => exit 0
cargo test --offline --locked --manifest-path services/nas/native-client/Cargo.toml => exit 0; 15 hosttests
git diff --check => exit 0
```

Ruwe lokale uitvoer: `target/nas-client-logs/development.log`.
De 15 tests bevatten 13 oorspronkelijke replace-errorcodes x voor/na commit
x huidige/pre-dispatchcheckpoint (52 gevallen), echte core-autorisatie/CAS,
conflictbewaring, geen replay, revoke/evict/coreherstel, stagingcancel,
quota/sessioncleanup, namespacebinding, limieten en redactie.
Dit is een hostmodel, geen daadwerkelijke diskpersistente client of QEMU-test.

Op die bron volgden alle vier crategates debug/release/Clippy/fmt/none met
exit 0; die resultaten gelden niet als bewijs voor latere bronwijzigingen.

## Callerfix na gerichte AI-codecontrole

De reviewer vond op bada7de: een afwijkende caller kon via commit/cancel de
lokale tickets consumeren terwijl de core beide operaties weigerde. Daarmee
bleven RAMreservaties zonder lokale cleanuphandle achter. Dit is opgelost
door de oorspronkelijke caller naast de echte coretickets te bewaren en
misrouting **voor** ticketconsumptie af te wijzen, zonder staged stateverlies.
De core voert de werkelijke autorisatie nog steeds uit. Twee afzonderlijke
32-cycli-regressies bewaken owner-commit en owner-cancel, nul achterblijvende
reserveringen/slotuitputting en geen false ACK/write voor de verkeerde caller.

`cargo test --offline --locked --manifest-path services/nas/native-client/Cargo.toml`
is na de fix herhaald: exit 0, **16 hosttests**, log
`target/nas-client-logs/caller-fix.log`. De reviewer bevestigt de concrete
bevinding opgelost; geen tests door de reviewer herhaald, geen menselijke audit.

## Finale lokale gates op bron 733da70

Bron `733da709383ce314cde63dea950125fe859f8c2a` was schoon tijdens de controles.
`checks.log` bevat alle exacte commando's, compiler, datum Europe/Amsterdam,
doorlooptijden en **33 expliciete exitstatuses, alle 0**, zonder skips.

| Zelfstandig workspace | Host debug/release | Clippy debug/release | fmt-check | unknown-none release |
|---|---|---|---|---|
| native-core | 21 / 22 | exit 0 / 0 | exit 0 | library exit 0 |
| native-volume | 10 / 11 | exit 0 / 0 | exit 0 | library exit 0 |
| native-probe | 5 / 5 | exit 0 / 0 | exit 0 | library exit 0 |
| native-client | 16 / 16 | exit 0 / 0 | exit 0 | library exit 0 |

Ook Python-oracle 8/8, native-probe-bin-Clippy, bestaande leafbuilder met
twee ELF-inspecties, progresscheck en diffchecks: alle exit 0.
Eigen hosttarget `target/nas-client-host`; ELF-target `target/nas-client-native`;
buildoutput `build/nas-client-check/users`. Geen gedeelde writers/downloads.

`build-report.json` bindt uitsluitend deze nieuw gebouwde ELF's:

- NAS 95.784 bytes, mapped 86.016 bytes, SHA-256
  `19479515dc99d002dad3c7b9e43c17a24f223a053193fe2b6834f8e670d28647`.
- Volume 97.976 bytes, mapped 86.016 bytes, SHA-256
  `e6be26f54ac02e2df33a00817910d2eb3d06a86f6e89cbedbdd31fb5934aca94`.

Dit zijn **niet** de historische ce52-ELFhashes of Release's QEMU-artefacts.
De probe is hier niet uitgevoerd; deze component krijgt geen native bewijs
van de afzonderlijke Release-run. De client zelf is alleen een library.

`source.sha256` is het expliciete scoped client/core/workflow-bronmanifest;
`source-fingerprint.sha256` hasht dat manifest, geen globaal imagefingerprint.
`evidence.sha256` bindt dit verslag, de PR-conceptbody, het bronmanifest,
de fingerprint, de ongewijzigde commandolog en het ELF-bouwrapport.

```sh
sha256sum --check services/nas/native-client/verification/source.sha256
cd services/nas/native-client/verification
sha256sum --check evidence.sha256
```

Git-diff tegen oraclebasis86a bevestigt dat core, volume en probe volledig
ongewijzigd zijn. Hoofdbranch/PR71 blijft ce52. Nieuwe CI-matrix heeft
daadwerkelijk vier entries, geen conditionele skip voor de nieuwe crate.
**GitHub-CI op deze branch is nog niet uitgevoerd**: nog geen nieuwe PR/run.
`pr-body.md` volgt de repositorytemplate voor integrator/intake; definitieve
PR-basis en exact-head CI worden niet met oude groene #71-jobs vervangen.

Progressassessment aan TL: **34 -> 34, NAS 0/5 -> 0/5**; geen native punten.
Open gedeelde interfaces en bewijsgrenzen staan in de bovenliggende README.

Hervatten: open de aparte concept-PR op `simon/nas-resume-client` volgens
bijgevoegde body, kies met lifecycle de inhoudsbehoudende integratiebasis en
controleer vier echte CI-jobs op die nieuwe head. Voor de native keten zijn
de door NET/Security bevestigde ontbrekende interfaces eerst noodzakelijk.
