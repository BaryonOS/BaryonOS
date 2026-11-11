# CSV table consumer

TASK-PLATFORM-M7-CSV-TABLE, base `d10b5cda`. A small app-local
`no_std + alloc` table with a **host-only** file consumer:
import CSV, edit cells as text, export and reopen.
This is not a native spreadsheet app, formula engine, window or XLSX compatibility.
No new registry dependency, kernel ABI, runtime or rights layer.

## Usage and format

```sh
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/table-editor/Cargo.toml --features host-cli --bin table-csv -- --set 2 3 '€ 42,50' < invoer.csv > uitvoer.csv
```

`--set RIJ KOLOM TEKST` may be repeated; indices start at 1.
Without an edit the file is checked and written canonically.
All import, argument, edit and export checks happen before stdout
is written. On such an error: exit 1, an error on stderr, no CSV output.
An I/O error during writing can still leave partial output.
Never use the same input/output file name with shell redirection:
the host shell may truncate the input beforehand. This is not an atomic file save path.

Chosen profile based on Y. Shafranovich,
*[RFC 4180, Common Format and MIME Type for CSV Files](https://www.rfc-editor.org/rfc/rfc4180.html)*,
October 2005, section 2 (informational, not a universal CSV standard):
commas, double quotes, doubled quotes, quoted newlines and equal
column counts. Spaces, header text and number-like values remain literal.
No schema/type/header detection, delimiter guessing, platform encoding or trim.
UTF-8, optional UTF-8 BOM and LF records are explicit extensions.
An empty last cell after a comma is preserved; export quotes empty cells.
Export is UTF-8 without BOM with CRLF records and minimally required quotes.
A cell with a leading U+FEFF is quoted, so that this text character does not
disappear as a file BOM on reopening.
An empty file is a zero-row table; an empty record is an empty cell.
Invalid UTF-8, NUL, bare CR outside quotes, ragged rows, unexpected or
unterminated quotes and text after a closing quote are explicitly refused.

`Table::set_cell` is transactional: a failed edit preserves the original
cell and the text budget. `Table::export` checks the total output size and
reserves all output space first. The model allocations are fallible;
this is not a native process arena or evidence of recovery on every host OOM.

## Boundaries and security

Maxima: 4 MiB input and cell text, 8 MiB output, 64 KiB per cell,
4096 rows and 256 columns. `Limits` can only lower these limits.
These are explicit limits of this first consumer, not a product choice for
large spreadsheets. Header/vector overhead comes on top of cell text; native
isolation and hard heap quotas are missing.

Cells with `=`, `+`, `-`, `@`, tabs or formulas remain **literal data**:
this consumer executes nothing and does not silently change such values.
Other spreadsheet programs may still execute that same CSV as a formula.
The output is therefore **not formula-injection-safe for external spreadsheets**.
That requires a separate explicit export choice; quoting alone does not solve this.

The host CLI receives only the stdin/stdout/stderr handles of its caller.
It is not a Nieuw OS resource broker. A native consumer must work via the existing
object portal and authorised volume client; path names or table cells
grant no capability. No network, NAS, shell execution or cloud fallback.
Content is not logged in error messages.

## Checks

```sh
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/table-editor/Cargo.toml --features host-cli
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/table-editor/Cargo.toml --features host-cli --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/table-editor/Cargo.toml -- --check
cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/table-editor/Cargo.toml --target x86_64-unknown-none
```

Host tests cover real CLI processes, import/edit/export/reopen,
Unicode/quotes/multiline/empty/BOM, exact budget limits, error transactions,
literal formulas and deterministic malformed mutations.
The cross-build proves only target compilation; no native app start,
renderer, volume storage, undo/formulas, XLSX/Office parity or release acceptance.
