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

Voor deze gewijzigde bron zijn release/lint/fmt-check/none en vierde CI-job
**nog niet uitgevoerd**. Volgende stap: volledige vier-cratematrix opnieuw,
bare-metal-library/ELF-gates en bron-/loghashes; CI op de uiteindelijke head.
Progressassessment aan TL: **34 -> 34, NAS 0/5 -> 0/5**; geen native punten.
Open gedeelde interfaces en bewijsgrenzen staan in de bovenliggende README.
