# Numeric spreadsheet consumer

TASK-PLATFORM-M10-RECALC-CONSUMER, base `d10b5cda`. A small concrete
host consumer of the **existing** `calculator::decimal` arithmetic core:
set a cell, query a formula with cell references, change input,
recalculate the same formula. No third arithmetic core, float or macro language.
The model is `no_std + alloc`; the CLI is exclusively a host program.

```sh
cargo +1.90.0 run --offline --locked --manifest-path sdk/examples/spreadsheet/Cargo.toml --features host-cli --bin sheet-calc -- --set A1 2 --set B1 '=A1*3' --get B1 --set A1 4 --get B1
# stdout: 6, then 12 (each on its own line)
```

`--set ADRES INVOER` and `--get ADRES` are executed in order.
An empty input `--set A1 ''` clears that cell. A `--get` is required.
Errors give exit 1/stderr; stdout is only written after **all**
actions succeed. No partial success values on a later formula error.
A stdout I/O error can still produce a partial write.
Use quotes around formulas so that the host shell does not interpret them itself.
No storage, CSV import or coupling to a not-yet-integrated branch.

## Explicitly supported profile

- Canonical addresses `A1` to `Z64`; at most 64 simultaneously occupied cells.
- Numeric literal with optional sign, point or comma; no text cell.
- A formula starts with `=`: existing `+ - * / ( )`, numeric literals and
  canonical cell references. No functions, ranges, absolute refs, names,
  external files, macros, date/money types or network.
- Existing decimal semantics: nine fractional places, nearest
  value with ties-to-even (on a tie, to even). Absolute maximum
  `9999999999,999999999`. No IEEE/Excel/XLSX parity claim.
- Rounding remains explicit in `Evaluation::rounded`, even if it took place in an
  earlier dependency. The CLI shows `[afgerond]` (rounded).
- A missing/cleared cell is an error, **not** implicitly zero.
  A cycle, division by zero, range overflow, invalid literal or
  reference yields no stored/fabricated result.

Input at most 256 ASCII bytes without control characters; spaces preserved.
Bounded invalid formula text is retained so it can be corrected;
evaluation gives an explicit error with the address of the failing cell.
Non-numeric text cells have not yet been implemented.
An invalid admission/allocation does not change the previous cell.

Every query starts a new recalculation with a temporary cache per
cell and cycle detection. References are passed to the existing evaluator
as parenthesised **values**, never again as source code/formulas.
As a result, negative values do not change operator precedence.
No persistent cache that could become stale after edit/clear/error repair.
At most 32 simultaneous dependency frames; the existing evaluator
additionally keeps its own 32-level nesting limit. An expanded formula is
at most 256 bytes and can therefore be refused earlier than the source formula.
Error positions of the decimal evaluator refer to that expanded formula,
not to a claimed cursor position in the original cell.

## Resource and integration boundary

Cell sources use fallible reserves: at most 16 KiB of source bytes plus
String/cell overhead; temporary formula buffers at most approx. 8 KiB along
a dependency path, besides stack and cache. The host CLI limits actions
to 256. This is no native hard heap quota/OOM/performance evidence.

No unsafe or new registry dependency. The path dependency reuses the
existing calculator package; the lockfile brings in its existing runtime,
UI and exactly pinned font/zeroize closure. No calculator, renderer,
workspace, kernel, IPC, B02, launch or package writes.
The numeric core grants no file, network or process rights.
The host CLI inherits exclusively the ordinary host process/stdio environment.

A native table window, shared Work document model, text/typed cells,
CSV/XLSX/ODS import/export, authorised saving, undo, selection and
dependency invalidation optimisation remain open. No complete
spreadsheet app or original M10 acceptance claimed.

## Targeted acceptance

```sh
# Run from the repository root on a prepared Rust toolchain.
set -e
export CARGO_TARGET_DIR="$PWD/target/platform-sheet-host"
cargo +1.90.0 test --offline --locked --manifest-path sdk/examples/spreadsheet/Cargo.toml --features host-cli
cargo +1.90.0 clippy --offline --locked --manifest-path sdk/examples/spreadsheet/Cargo.toml --features host-cli --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path sdk/examples/spreadsheet/Cargo.toml -- --check
cargo +1.90.0 build --offline --locked --manifest-path sdk/examples/spreadsheet/Cargo.toml --features host-cli --bin sheet-calc
cargo +1.90.0 build --offline --locked --release --lib --manifest-path sdk/examples/spreadsheet/Cargo.toml --target x86_64-unknown-none
python3 -B sdk/examples/spreadsheet/tests/interop.py target/platform-sheet-host/debug/sheet-calc
```

Ten tests cover real processes, recalculation after edits/clearing, cycles,
missing references, error repair, reused dependency DAG,
operator precedence, rounding propagation and exact limits.
Python `Decimal` independently checks 32 actual CLI recalculations
and five rounded dependency results. No native execution,
GUI/hardware/human or release acceptance.
