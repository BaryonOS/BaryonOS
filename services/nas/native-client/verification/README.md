# TASK-NAS-RESUME-1 — eigen broncheckpoint

## Actuele stacked PR-checkpoint

De huidige PR-branch is `simon/nas-resume-client-stack`, gestapeld op de
actuele NAS-core/probe-branch `simonvanieperen-legendary-broccoli` en diens
head `a9cf10970889ef10b4fc67df47316b7fc66fa2f2`. Alleen de nieuwe
`native-client` crate en de vierde `nas-core.yml` matrixentry vormen de PR-diff;
de core/probe/oracle commits worden niet opnieuw meegestuurd.

De resume-client source en runtime commits zijn overgenomen zonder broncode-
aanpassing. Het source manifest is opnieuw aan de workflowbytes op deze
stacked basis gebonden; alle andere vermelde bronhashes zijn bevestigd. Exact
op deze stacked checkout zijn opnieuw gedraaid:

| Gate | Resultaat |
|---|---|
| Client `cargo fmt --check` | exit 0 |
| Client debug tests | exit 0, 16 tests |
| Client release tests | exit 0, 17 tests, inclusief 64-KiB/max-queue-herstel |
| Client debug/release Clippy met `-D warnings` | exit 0 |
| Client `x86_64-unknown-none` release library build | exit 0; geen guestrun |
| NAS probe/volume ELF build en inspectie | exit 0; rapport zegt `native_execution=false` |
| Source-manifestcontrole en `git diff --check` | exit 0 |

Exacte output staat in `stacked-gates.log`; de nieuwe ELF-inspectie staat in
`stacked-build-report.json`. Deze rapporteert `nas-probe` 95.784 bytes
(SHA-256 `8f438d9c6b29f74404a9ef8ddb4cfe85e342f5ae667ac9cd83bac869ee1fd56e`)
en `nas-volume` 98.176 bytes
(SHA-256 `6b8c743866630960fba38c568c0b6fcbb90c31984b39d838e241f81aa04a8b9f`).
De eerdere `checks.log`, `build-report.json` en `stack-gates.log` blijven
historische outputs van de oorspronkelijke oracle86a/ce52-bron; ze worden niet
herlabeld naar deze stacked head.

De zelfstandige Python disk-oraclebestanden zitten niet in de PR-basis/diff en
zijn daarom niet opnieuw uitgevoerd of als stacked-PR bewijs geclaimd. De
historische 8/8 oracle-uitkomst bleef gebonden aan de eerdere bron. Draft PR
#95 is geopend op de stacked client-head. De GitHub-checks zijn gestart maar
stonden bij deze statusopname nog pending; bekijk de PR voor hun actuele
resultaat.

## NEXT AGENT HANDOFF — eigen werk gereed, integratiegrens open

Laatste geteste broncheckpoint:
`26c88292b9e4775f51598488037dee78ed70bd51`, remote bevestigd op
`simon/nas-resume-client`; runtime nog exact
`733da709383ce314cde63dea950125fe859f8c2a`.
Eigen worktree: `target/nas-client-work` onder de oorspronkelijke
`simonvanieperen-legendary-broccoli`-worktree. Oorspronkelijke sessiebranch
en concept-PR71 blijven ce52, geen root/SDK/andere schrijver overgenomen.
De commit met dit handoffrecord wijzigt uitsluitend bewijsdocumentatie.

Actueel scoped bronmanifestfingerprint:
`d0968cb11b30c18e3d224aeed8060b7afb4aa98096194a766f62e5f0e7a328fd`.
Runtime en tests zijn beschreven en gehasht, geen open lokale codewijzigingen.
Vier crategates en eigen ELF-inspecties: 33 commando's exit0 op733da70;
daarna client16debug/17release en overige gerichte gates: 8 commando's exit0.
Ruwe logs staan in `target/nas-client-logs/`; controleerbare kopieen hier.
Geen NAS-client-QEMU, echte TLS/E2EE, duurzame clientcheckpoint of GitHub-CI
uitgevoerd. Score blijft 34/100, NAS0/5; centrale bestanden zijn manager-owned.

Oorspronkelijke integratiestap voor intake/lifecycle (vervangen door de
stacked branch hierboven): maak de afzonderlijke concept-PR met deze head en
`pr-body.md`, bevestig de behouden basisinhoud en voer de echte NAS-CI-jobs op
die PR-head uit. De huidige app-PRtool kan alleen de oorspronkelijke
sessiebranch selecteren; gebruik hem niet om per ongeluk PR71 bij te werken.
Geen oude groene CI van ce52 als acceptatie van een nieuwe head gebruiken.

De echte productketen kan niet met de huidige interfaces worden aangesloten:
NET bevestigt geen geauthenticeerde native TLS-peerprincipal, Security geen
geautoriseerde NAS-seal/device/restore-interface. Security's afzonderlijke
contractbron `c5fbb21a0e46e010d5e9b4748ae9b9307a67cef4` verduidelijkt die
grens, maar is geen runtimegrant of reeds geintegreerde release-interface.
Materiaalbericht is niet afgeleverd; geen UI-afstemming claimen.

Een exact boot1-tokenreplayprofiel is **niet toegewezen**. Release bevestigt
dat zijn huidige rootprofiel geen 80-byte export/coldinputbinding heeft.
Een nieuw concreet verzoek is vastgelegd bij de teamleider: uitsluitend
testfixture Session40+Upload40 uit echte normalrol/stage, private artefacten
zonder tokens in logs/Git, oorsprong/volledigheid/lengte/hash gebonden en
immutable coldinput. De bevoegde rootwriter en een nieuw leafslot zijn eerst
nodig. Geen eigen wire/bootstrap of clientpersistentie daarvoor verzinnen.
Bij onbereikbare coördinatie niet herhaald toestemming vragen: deze exacte
hervatgrens bewaren en geen actief schrijfgebied overnemen.

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
de fingerprint, de ongewijzigde commandologs en het ELF-bouwrapport.

## Vervolg: maximale queue op 64 KiB hoststack

Boven checkpoint `87a04d48b6443a33fa62fb20dfcda1679f29b1a8` is uitsluitend
een geoptimaliseerde test toegevoegd aan `tests/resume.rs`. De runtime,
manifests, workflow en alle drie eerdere crates zijn bytegelijk aan733da70;
de expliciete git-diffcontrole staat in `stack-gates.log`.

Nieuwe bron is gebonden door het actuele `source.sha256` en de daarvan
afgeleide fingerprint. De oorspronkelijke 33-commandolog blijft historisch
ongewijzigd. `stack-gates.log` voegt **8 commando's, alle exit 0** toe:
clienttests debug **16**, release **17**, beide Clippyprofielen, fmt-check,
unknown-none-librarybuild en twee diffchecks. Deze commando's draaien echt;
de ongewijzigde librarybuild kan bestaande compilatieartefacts hergebruiken.

De nieuwe release-only test gebruikt 64 KiB hoststack met caller-owned
heaparena. Vier edits van ieder 1.024 bytes worden gelijktijdig staged
(4.096 bytes reservering), door een typed RAMcheckpoint/core-recovery gehaald,
read-only gereconcilieerd en daarna echt via de core gecommit. Resultaat:
client 9.088 bytes, checkpoint 8.608 bytes; vier commits, used4.096/reserved0.
Dit is geen native caller/grant-, piekstack-, diskcheckpoint- of E2EE-bewijs.
De bestaande ELF-buildhashes gelden voor ongewijzigde bron, niet voor een
nieuw uitgevoerde VM. Nieuwe GitHub-CI blijft nog open.

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
