# TASK-PLATFORM-M7-DOCX-TEXT-EXPORT

Basis `6d523964ab85638d4dd981bcdb19580d18646fab` (gefetched main).
Branch `platform/document-docx-export-20261103`, geteste code/testbron
`79ba4cfef11b11c6e217d99b0f86b445b77eb8b4`.
De vervolgcommit voegt uitsluitend deze receipt toe.
Scope: nieuwe `sdk/examples/document-export/**` plus dit bewijs;
geen oude Platform-slices, General-editor/Work, gedeelde workspace,
document-session, kernel, drivers, launch, rechten of centrale status/score.

## Concreet resultaat

Werkelijke CLI UTF-8-tekst stdin -> DOCX stdout, met no_std-exportfunctie
voor callerbytes. De uitvoer is een ZIP/OPC-package met exact drie vaste
parts en WordprocessingML-alinea's/runs/tekst. Unicode, spaties, tabs en
lege/afsluitende alinea's worden onafhankelijk exact teruggelezen.
Invoermarkup wordt geescaped, niet als veld/link of andere actieve inhoud
geinterpreteerd. Geen auteurmetadata, externe relationships of macro's.
De uitvoer bevat plaintext, geen nieuwe versleutelings-/opslagclaim.

Deze bestandsinteroperabiliteit had geen launch/window/child-I/O nodig.
Alleen de toekomstige native exportdialoog/savekoppeling is lokaal daarvan
afhankelijk; geen globale Platformblokkade. Andere native editorwrites
blijven bij General. Geen afhankelijkheid van overgedragen branches.

Bronprofiel en primaire bronnen staan in de subtree-README: Microsoft
*Structure of a WordprocessingML document* (12 januari 2024) en PKWARE
APPNOTE 6.3.10 (1 november 2022), ZIP-secties 4.3.7/12/16.
Geen algemene DOCX-parser, norm-/Office-appvalidatie of roundtrippariteit.
ECMA-landingspagina HTTP 403, dus niet als gelezen/geverifieerd aangeduid.
Geen dependency of unsafe.

## Exacte uitgevoerde controles

Werkdirectory
`/home/simon/.copilot/session-state/edf11e7c8b3e4c9ba1a91b7ef0779695/files/platform-docx`.
Rust 1.90.0, bestaande `/tmp/nieuw-os-c-toolchain/cc-wrapper` en sysroot,
`CARGO_TARGET_DIR="$PWD/target/platform-docx-host"`.
Geen installatie/download of wijzigingen aan bestaande lockfiles.

| Exact commando | Resultaat |
|---|---|
| `cargo +1.90.0 generate-lockfile --offline --manifest-path sdk/examples/document-export/Cargo.toml` | exit 0; alleen nieuwe dependencyvrije workspace |
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/document-export/Cargo.toml --features host-cli` | exit 0; 1 CRC-unit + 4 exporttests + 1 CLI-procestest, 0,424 s |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/document-export/Cargo.toml --features host-cli --all-targets -- -D warnings` | exit 0, 0,145 s |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/document-export/Cargo.toml -- --check` | exit 0, 0,041 s |
| `cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/document-export/Cargo.toml --target x86_64-unknown-none` | exit 0, 0,146 s; bibliotheek, geen uitvoering |
| `python3 -B sdk/examples/document-export/tests/interop.py target/platform-docx-host/debug/text-docx build/platform-docx-evidence/example.docx` | exit 0, 0,174 s; 133 echte CLI-exports onafhankelijk teruggelezen |
| `file build/platform-docx-evidence/example.docx` | exit 0; Microsoft Word 2007+ |
| `git diff --check` | exit 0 |

Eerste uitgevoerde keten groen; geen rood-naar-groenregressie geclaimd.
Negatieven: malformed UTF-8, XML-ongeldige tekens, CR/CRLF, NUL, exacte
outputbudgetgrens, 64 KiB/64 KiB+1, 4096/4097 alinea's, ongewenste CLI-opties.
Geen successtdout op validatie-/exportfout. Originele invoer blijft gelijk.
Python gebruikt ZIP-CRC/structuur en XML-parser, geen productreader of
gedeelde exporterlogica. Het controleert drie parts, interne relatie,
contenttype, namespaces en exacte Unicode/tab/alinea-inhoud, ook adversariale
markup, BOM/bidi-tekst, grote tekst en maximale lege-alineareeks.
Dit is geen Office-/LibreOffice-open-/renderproef.

Logs/artifacts buiten Git onder `build/platform-docx-evidence/`.
SHA-256:

```text
c51aa07967b0a8d726bc3e059e450346bcfcc04de5100530554efed006ac80c3 gates.log
cf5a06e6a898e86defe9df3f476645485ba4aa57f6702f056d0c058225f05507 example.docx
ac09299726090d537d74c67d3bdfa7435b7a6653b754da5f7c839094cd813eef target/platform-docx-host/debug/text-docx
312581f613ff9688b7864f7fcb64f0dddaccb8e5bf359f51048b4bd2ffd9bec0 target/platform-docx-host/x86_64-unknown-none/release/libnieuw_os_document_export.rlib
d73f111f22b368fcc102a6f8f4b984ddf9fa291e453f98bdb4901a421ba4709c src/lib.rs
cbd559051ade77c1c9c70b3124c90d7e672d4afc5f79edfbed77d8ac311bc964 src/main.rs
971a67907d589071584d1c3e7ce0a2fae5e0366fb7db82911692c312973dbd77 tests/cli.rs
d8acabdde20362907b8344f9b61bd6b5a323d397428b357b6b42fffdd360b011 tests/export.rs
68a0f57060e0f7ff45132a4dbc58a74a9278d5a520955621233f29db40f08420 tests/interop.py
```

## Queue, bewijsgrens en rollback

**QUEUED-FOR-INTEGRATION**, geen nieuwe PR zonder OPEN-PR-LEASE.
Eventuele app-autocreatie wordt als bestaande PR gemeld, geen duplicaat.
Geen oude slices hertest of aanvullende oude receipts gemaakt.

Hostfunctionaliteit plus no_std-compilatie. Niet uitgevoerd/niet bewezen:
native executable, exportdialoog, geautoriseerde volume-save, GUI, DOCX-import,
Word/LibreOffice-rendering, volledige OOXML-schemavalidatie, fonts/paginering/
opmaakpariteit, native hardheapquota/OOM-herstel, QEMU, hardware,
menselijke gebruikerstest, onafhankelijke audit, releaseacceptatie.
Stdoutwrite is niet atomair; geen silent retry of fout-als-succes.

FR-A02/interop draagt deze beperkte functie, geen native aftekening.
Scoreadvies **41 -> 41/100**; centrale beoordeling/status bij General.
Rollback: verwijder uitsluitend nieuwe document-export-subtree en receipt;
geen bestaande ABI, editor of opslagformaat wijzigt.
