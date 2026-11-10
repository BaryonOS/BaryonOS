# Export text to an actual DOCX file

TASK-PLATFORM-M7-DOCX-TEXT-EXPORT, baseline `6d523964`.
A concrete interoperability function: convert UTF-8 text from stdin into a
standalone WordprocessingML/OPC document on stdout. No second editor,
Office parser, ZIP importer or empty format foundation.
The exporter is `no_std+alloc`; the actual CLI is host-only.

```sh
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/document-export/Cargo.toml --features host-cli --bin text-docx < tekst.txt > document.docx
```

## Concrete export profile

- Text remains Unicode, including accents, non-Latin characters and emoji.
  No font substitution or glyph fallback during export; the receiving
  document app determines fonts/layout.
- Every LF becomes a paragraph, including empty and trailing paragraphs.
  Tabs become `w:tab`; spaces are retained with `xml:space="preserve"`.
  No trimming or interpretation of markup, URLs or formula-like content.
- No silent conversion: invalid UTF-8, XML-invalid characters and CR/CRLF
  produce an error, with the source-byte position where possible. The caller
  selects any normalisation beforehand; this exporter does not alter the source.
- At most 64 KiB input, 4096 paragraphs and 4 MiB output; callers may lower
  the output budget. Limits include exact ZIP overhead.
- Three fixed OPC parts: `[Content_Types].xml`, `_rels/.rels`,
  `word/document.xml`. ZIP stored, no compression, ZIP64 or encryption.
  A fixed date of 1980-01-01 makes output deterministic; it is not an actual
  modification date. CRC32 detects corruption; it is not authentication.
- No macros, automatic fields, embedded objects, external relationships,
  hyperlinks, scripts, author metadata or filenames derived from input.

This is a **text export profile**, not a claim of DOCX import/roundtrip,
stylesheets, typography, pagination, tables, images, revisions, screen-reader
support or full Office compatibility. Output contains plaintext; a DOCX
export is not end-to-end encrypted and must use deliberately selected output/
storage. No upload or network fallback.

## Implementation and resource boundary

No unsafe or dependencies. Only the minimum standard ZIP serialiser for the
three fixed parts; no general archive library or parser. The fixed input
limit bounds all lengths before conversion to ZIP u16/u32: document contents
are below 3 MiB; part names are fixed ASCII constants. XML and ZIP output are
each reserved exactly and fallibly in advance; working memory is approximately
twice the output plus input, not a native hard heap quota.

Only `sdk/examples/document-export/**` and its own receipt are new.
No changes to the General editor/document session, Work, shared workspace,
kernel, drivers, launch, permissions or score. The no_std API operates on
caller bytes and grants no file/network capability. A native export action
must be connected by the existing General/portal owner to authorised saving;
that integration does not block this file function.

The CLI writes stdout only after validation and export succeed; failures
produce exit 1 and stderr without document contents. Stdout writes are not
atomic: I/O errors may leave a partial file. Never use the same input/output
path in shell redirection.

## Primary sources and acceptance

Microsoft, *Structure of a WordprocessingML document*, 12 January 2024,
<https://learn.microsoft.com/en-us/office/open-xml/word/structure-of-a-wordprocessingml-document>.
PKWARE, *APPNOTE: ZIP File Format Specification*, 6.3.10, 1 November 2022,
sections 4.3.7, 4.3.12 and 4.3.16,
<https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT>.
Accessed 3 November 2026; no source text copied.
The ECMA-376 landing page could not be retrieved (HTTP 403);
no full standard/schema validation is claimed.

```sh
# Run from the repository root with a prepared Rust toolchain.
set -e
export CARGO_TARGET_DIR="$PWD/target/platform-docx-host"
mkdir -p build/platform-docx-evidence
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/document-export/Cargo.toml --features host-cli
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/document-export/Cargo.toml --features host-cli --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/document-export/Cargo.toml -- --check
cargo +1.90.0 build --offline --locked --manifest-path sdk/examples/document-export/Cargo.toml --features host-cli --bin text-docx
cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/document-export/Cargo.toml --target x86_64-unknown-none
python3 -B sdk/examples/document-export/tests/interop.py target/platform-docx-host/debug/text-docx build/platform-docx-evidence/example.docx
file build/platform-docx-evidence/example.docx
```

Six Rust tests and 133 actual CLI exports are independently checked with
Python `zipfile`/CRC and `ElementTree` for fixed package parts, relationships,
XML namespaces and exact reread Unicode/tab/paragraph contents. `file`
recognises output as Microsoft Word 2007+. This does **not** demonstrate that
Word/LibreOffice actually opened/rendered it. No native executable, GUI/save
portal, QEMU, hardware, human, schema/Office-app, independent security audit
or release acceptance.
