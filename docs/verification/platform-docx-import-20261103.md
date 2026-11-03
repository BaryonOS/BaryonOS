# TASK-PLATFORM-M7-DOCX-IMPORT

Basis `6d523964ab85638d4dd981bcdb19580d18646fab` (gefetched main).
Branch `platform/document-docx-import-20261103`.
Geteste code/test/lockfile/documentatiebron:
`73ff2dffd7dc627719b8c91275f91d02f0e4680c`.
Vervolgcommit voegt alleen deze receipt toe.
Scope: nieuwe `sdk/examples/document-import/**` en eigen receipt.
Geen dependency op gequeuede DOCX-export; geen writes of hertests van
EXIF/CSV/BMP/recalc/presentation/WAV, General/Veld/K05 of centrale score/status.

## Resultaat en veiligheidsinvariant

Werkelijke CLI leest DOCX van stdin, valideert ZIP32/OPC-mainbinding,
controleert per part outputlengte/CRC, interpreteert namespaces met
onderhouden roxmltree en schrijft UTF-8-tekst. Verplichte `--plain-text`
maakt verlies van opmaak expliciet. De bron blijft ongewijzigd.
Geen diskextractie, padopening, netwerk, macro-/veldcode, entityresolver
of opvolgen van relaties. Geen algemene XML-/DEFLATE-engine toegevoegd.
Eigen ZIP-admission is beperkt tot dit bestandsprofiel, geen archiefframework.

Ondersteund: gewone Transitional `word/document.xml`, tekstalinea's/runs,
lege/afsluitende alinea's, Unicode, entities, tabs, gewone linebreaks en
beperkte opmaakproperties. Velden/tabellen/afbeeldingen/hyperlinks/nummering,
wijzigingen, onbekende inhoud en header-/footerreferenties worden geweigerd,
niet stil als volledige tekst gepresenteerd. Gekoppelde stijlen worden niet
visueel geïnterpreteerd; geen zichtbaarheid-/Word-renderclaim.
Het volledige profiel, budgets en primaire formaatbronnen staan in README.

Maximaal 16 MiB input, 512 parts, 1 MiB per expanded part, 16 MiB totaal,
32768 XML-nodes per XML-part, 4096 alinea's, 64 KiB tekst.
Stored/raw-DEFLATE, signed/unsigned descriptors; geen ZIP64/encryptie.
Local/CD-records moeten dezelfde volgorde hebben, zonder overlap/gaps.
Geen stille clamp, recoverytekst, schema-validatie of gedeeltelijk succes.

## Dependencycontrole

`THIRD-PARTY.md` bevat daadwerkelijk gelezen primaire herkomst/licentie/
onderhoud/no_std/buildscriptgegevens. Cargo.lock bindt de registrytarballs:

```text
roxmltree 0.21.2  9da7f59328b09405b181fee1018925fad50f157afb43c0c0f5c522c256dd5907
memchr 2.7.6     f52b00d39961fc5b2736ea853c9cc86238e165017a493d1d5c8eac6bdc4cc273
zune-inflate 0.2.54 73ab332fe2f6680068f3582b16a24f90ad7096d5d39b974d1c0aff0125116f02
```

De eerste offline lockpoging faalde exit 101 door ontbrekende roxmltree-cache.
Daarna `cargo +1.90.0 generate-lockfile --manifest-path sdk/examples/document-import/Cargo.toml`
en `cargo +1.90.0 fetch --locked --manifest-path sdk/examples/document-import/Cargo.toml`
met tijdelijk `CARGO_NET_OFFLINE=false`: exit 0.
De aanvankelijke 0.21.1-kandidaat is vervangen na controle van upstream
0.21.2-changelog (24 oktober 2026): diepe XML-nesting veroorzaakte stackoverflow;
0.21.2 parseert die iteratief. Manifest exact bijgewerkt,
`cargo +1.90.0 update --manifest-path sdk/examples/document-import/Cargo.toml -p roxmltree --precise 0.21.2`
en dezelfde locked fetch: exit 0. Geen eigen fix of genegeerde stackfout.

Daadwerkelijke `cargo metadata --offline --locked --format-version 1`
met deze manifestpath bevat geen custom-build-target voor de drie deps;
`cargo tree --offline --locked --manifest-path sdk/examples/document-import/Cargo.toml -e features`
bevat uitsluitend de drie defaultfeatureloze deps. Beide exit 0.
roxmltree-registrybron bevestigt no_std. Registry-manifest SHA-256:
roxmltree `5a361401f9e83ab9ca2a604a303b9e2b871b0ddfa0382b5943e0751405f04a98`;
memchr `5750ca97e8b2643f2ba1d7e98f54dcf54518c0176899e547876a59eb736198a5`.
Aanwezige LICENSE-MIT-hashes:
roxmltree `fffe218c1e349e1d482a05677f2ce850502f88298174d77f6ac7c04821db7899`;
memchr `0f96a83840e146e43c0ec96a22ec1f392e0680e6c1226e6f3ba87e0740af850f`.
Dit is herkomst-/ontwerpcontrole, geen onafhankelijke securityaudit.

## Exacte uitgevoerde acceptatie

Werkdirectory
`/home/simon/.copilot/session-state/edf11e7c8b3e4c9ba1a91b7ef0779695/files/platform-docx-import`.
Rust 1.90.0 en bestaande cc-wrapper/sysroot uit random-environment.sh;
`CARGO_TARGET_DIR="$PWD/target/platform-docx-import-host"`.
Geen nieuw OS-toolchaininstallatie.

| Exact commando | Finale uitkomst |
|---|---|
| `cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/document-import/Cargo.toml --features host-cli` | exit 0; 5 geregistreerde tests, 0,404 s |
| `cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/document-import/Cargo.toml --features host-cli --all-targets -- -D warnings` | exit 0, 0,181 s |
| `cargo +1.90.0 fmt --manifest-path sdk/examples/document-import/Cargo.toml -- --check` | exit 0, 0,047 s |
| `cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/document-import/Cargo.toml --target x86_64-unknown-none` | exit 0, 0,306 s; compilatie, geen native uitvoering |
| `python3 -B sdk/examples/document-import/tests/interop.py target/platform-docx-import-host/debug/docx-text build/platform-docx-import-evidence/input.docx` | exit 0, 1,618 s; 83 werkelijke tekst-exacte imports, 1432 afwijzingen |
| `file build/platform-docx-import-evidence/input.docx` | exit 0, 0,003 s; Microsoft Word 2007+ |
| `git diff --check` | exit 0, 0,142 s |
| `git diff --cached --check` | exit 0 na staging van de nieuwe subtree |

Rust bewijst namespace-URI versus prefix, Unicode/entities/whitespace,
geen verborgen content in opmaakproperties, format-/field-/table-denials,
12000 diep geneste elementen zonder stackoverflow, nodebudget, invalid
XML/UTF-8/DTD/external entities en exacte paragraph-/tekst-/XML-bytebudgetten.
Python maakt DOCX met onafhankelijke zipfile/ElementTree-writers:
80 inhoudsvarianten (stored/deflated, seekable/streaming descriptors),
511/512-entrygrenzen en een werkelijk unsigned-descriptorpakket.
Alle uitvoerbytes worden met verwachte tekst vergeleken.
Negatieven: iedere truncatie van een geldig pakket, CLI-keuze, CRC/header/
filename-mutaties, duplicaten/casecollisions, traversalnamen, ontbrekende
parts, 513-entrygrens, main-type/relatie/namespace-spoof, externe rootrelatie,
macrotype, unsupported inhoud en een forged-small-output DEFLATE-bom.
Afwijzingen vereisen exit !=0, stderr en lege stdout.

SHA-256 van buiten Git bewaarde logs/artifacts onder de workdirectory:

```text
a47b13a19058ee87e974e01d627520979e6bd2b5385217f9ac8a5ad163fd292c build/platform-docx-import-evidence/accepted-gates.log
c48674d5b43b9f877f84cc6e7c3175a5e1792ce9c222bf95df012b24b718aa7b build/platform-docx-import-evidence/input.docx
6b03a1a9d722d04d938802016b9a2f70c822beaebd09e095e5a5befea53588d1 target/platform-docx-import-host/debug/docx-text
1af4b5e53544c652c1436351bf3588ae021264fc31322ad6914b7c6bb927b2d2 target/platform-docx-import-host/x86_64-unknown-none/release/libnieuw_os_document_import.rlib
bc70f7613ee09aa7343e52575bd5aab9b359c25f78b09b52ee547b9b9be15cf5 src/lib.rs
176655b7edba8d46304424bddac7bc7b4792a16ce5fe7b404ce6d398d5c94e23 src/package.rs
356ed956fa91117c7e8ddd7beb7306f7fd81822b8d3548ce06e5ed36541b9bd4 src/main.rs
5d6602d61641c8cfcc4cda17032d10ad48c8e959408892e144dd45dbc77c1687 tests/word.rs
24232dfb4574c97d966f8b8cb0b073a9935faccacb8041ba371517307c421067 tests/interop.py
```

## Queue en resterende grenzen

**QUEUED-FOR-INTEGRATION**, geen nieuwe PR zonder OPEN-PR-LEASE.
Hostbestandsfunctionaliteit en no_std-compilatie, niet native open/save.
Geen General-/K05-launch-/surface-/rechteninterfaces gewijzigd.
Geen Office/LibreOffice-interop, volledige OOXML-/ZIP-conformance,
native hardheapquota/OOM-herstel, VM, fysiek apparaat, menselijke
gebruikerstest, onafhankelijke audit of releasevalidatie uitgevoerd.
XML/DEFLATE-upstreamallocaties blijven infallible; inflaterlimit is best
effort, geen exact raw-DEFLATE-consumptie-API. Geen claim dat interne
compressed padding/junk wordt ontdekt. CRC is geen authenticatie.
Native gebruik vereist afzonderlijke parserproces-/geheugengrens.
Stdoutschrijven is niet atomair.
Scoreadvies **41 -> 41/100**; geen native criterium gesloten.
Rollback verwijdert uitsluitend nieuwe document-import-subtree en receipt.
