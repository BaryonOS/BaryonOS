# TASK-NAS-RESUME-1 — eigen broncheckpoint

Basis `86a9600e4f1366dbd2d9e95f50477ea353fa6dfa`
(oracle-leaf, runtime ce52). Werkbranch `simon/nas-resume-client`.
Exclusieve managerclaim: nieuwe `services/nas/native-client/**` en vierde
matrixentry in `.github/workflows/nas-core.yml`; vooraf aan build afgestemd.
Geen centrale status/queue/score-, SDK-, crypto-, transport- of rootwrites.

Uitgevoerd voor dit eerste broncheckpoint, Rust 1.90.0, eigen target
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

Bij dit eerste checkpoint zijn release/lint/fmt-check/none en vierde CI-job
**nog niet uitgevoerd**. Volgende stap: beide test-/Clippyprofielen,
fmt-check en bare-metal-librarybuild; daarna bron-/loghashes en exact-head CI.
Progressassessment aan TL: **34 -> 34, NAS 0/5 -> 0/5**; geen native punten.
Open gedeelde interfaces en bewijsgrenzen staan in de bovenliggende README.
