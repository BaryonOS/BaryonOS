# DOCX to plain text

New separate import consumer for existing text DOCX files. No editor, viewer,
Word replacement or renderer. Formatting is explicitly lost; tables, images,
fields, tracked changes, hyperlink content, direct hidden properties, section
headers/footers and unknown content are rejected rather than returned as
incomplete successful text. Referenced styles are not visually resolved;
no visibility claim. The source is read only and remains unchanged.

```sh
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/document-import/Cargo.toml --features host-cli
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/document-import/Cargo.toml --features host-cli --bin docx-text -- --plain-text < bron.docx > tekst.txt
```

`--plain-text` is required as an explicit choice to lose formatting.
Stdin/stdout are the only data interfaces. No network/file opening, extraction
to paths, macros, field-code execution, entity resolver or automatic following
of relationships. CLI errors produce stderr/exit 1, not successful stdout.
A write failure may leave partial stdout; no atomic storage claim.

## Bounded format

OPC package with ZIP32, one disk, UTF-8/ASCII part names, stored or raw DEFLATE.
Local records and the central directory must have the same order and contents,
without a self-extracting prefix, gaps or overlapping records. Data descriptors
with/without signature and ZIP comments are allowed; ZIP64/encryption and
other methods are not. CRC-32 and declared output length are checked per part;
CRC is not authentication. No general ZIP API. Case collisions and duplicate
names are rejected. No path is created on disk.

Mandatory namespace and type binding through `[Content_Types].xml` and
`_rels/.rels` to ordinary Transitional `word/document.xml`. No macro-enabled
main type. External root relationships are rejected. Other parts undergo
CRC/length checks but are not interpreted; no external relationship from
such a part is followed. Only the main body is imported. The supported body
consists of paragraphs/runs/text/tabs/ordinary line breaks and limited
formatting properties. Lists/numbering and unknown structures are rejected.
Empty and trailing paragraphs are preserved as LF; no additional final LF.
Text entities and XML line endings are interpreted according to XML.
The namespace URI, not the chosen prefix, determines whether an element
contains Word text.

## Limits

At most 16 MiB package, 512 parts, 1 MiB per decompressed part, 16 MiB total,
32768 XML nodes per XML part, 4096 paragraphs and 64 KiB text. Owned output
buffers reserve fallibly. Upstream XML/DEFLATE allocations remain infallible.
zune-inflate's limit is best effort and may exceed it by a bounded decode/
buffer margin; final length/CRC are checked again. Its API does not report
exact raw-DEFLATE bits consumed: this slice does not claim to detect padding/
junk inside a declared compressed record. No hard native heap/time quota or
OOM-recovery claim without a parser process. Adding a native parser process/
capability boundary is General/K05 work and is not bypassed by this host consumer.

Upstream choices/licences/pins are in `THIRD-PARTY.md` and Cargo.lock.
Base standards: Microsoft, *Structure of a WordprocessingML document*,
12 January 2024,
<https://learn.microsoft.com/en-us/office/open-xml/word/structure-of-a-wordprocessingml-document>;
PKWARE, APPNOTE 6.3.10, 1 November 2022, sections 4.3/4.4,
<https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT>.
Additional primary format sources, matched to the individual claims:

- Ecma International, *ECMA-376: Office Open XML file formats*,
  Part 2 *Open Packaging Conventions*, 5th edition, December 2021
  (OPC package parts, relationships); Part 4 *Transitional Migration Features*,
  5th edition, December 2016 (Transitional references):
  <https://ecma-international.org/publications-and-standards/standards/ecma-376/>.
- Deutsch, L. P., *DEFLATE Compressed Data Format Specification version 1.3*,
  RFC 1951, RFC Editor, May 1996 (raw DEFLATE):
  <https://www.rfc-editor.org/rfc/rfc1951.html>.
- World Wide Web Consortium, *Extensible Markup Language (XML) 1.0
  (Fifth Edition)*, W3C Recommendation, 26 November 2008
  (XML entities and line endings): <https://www.w3.org/TR/xml/>.

These references define the bounded format; they do not establish complete
OPC/OOXML/XML implementation conformance. No Office, LibreOffice, schema
validation or full OOXML-profile claim.

## Acceptance

Targeted Rust tests, actual Python-zipfile/ElementTree DOCX inputs through
the CLI, Clippy/fmt and a no_std library build. The evidence receipt is added
only after acceptance has actually run. Host file functionality and compilation
are not native open/save/launch/window acceptance. No physical hardware,
human usability, independent security audit or release claim.
