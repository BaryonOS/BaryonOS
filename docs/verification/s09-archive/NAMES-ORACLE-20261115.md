# S09 ustar name check: independent oracles and mutation checks (15 November 2026)

**Scope.** This note records evidence for the name check in `services/volume/src/ustar.rs` (`reasons`, `collide`, `judge` and the re-judge in `from_archive`). The check belongs to the approved lossy user export, decision D6(a): POSIX ustar with pax `path` records. This is host-level evidence only. No native or QEMU stage runs the name check, and S09 acceptance stays open. The design and the refusal rules are in [S09-BACKUP-ARCHIVE.md](../../implementation/S09-BACKUP-ARCHIVE.md), section "User export to ustar".

**Source.**
- Branch `feat/s09-ustar-name-check`, task base `14d8da04f88568c3356b24ab704fd8cdc7cbe030`.
- The oracles were added in `ac06aeac`, and that tree was tested.
- `ustar.rs` was last changed in `1b1ef0fb2014798893ee3076637102884e4f1473`, with sha256 `56cfb8377eb8346aa40d5509c2bd89189ecfdb95585283be15062dc86d7c36ac`. This hash is the same at HEAD and in the working tree after the mutation run.

**Toolchain.** rustc 1.90.0 uses Unicode 16.0.0 (`char::UNICODE_VERSION`). Python is 3.14.4, with `unicodedata` 16.0.0. Both sides therefore use the same Unicode version.

## Host checks (local)

| Command | Exit | Result |
|---|---|---|
| `cargo +1.90.0 test --locked -p volume-core` | 0 | 92 passed. The `archive` binary has 37 tests, 14 of them ustar tests. |
| `cargo +1.90.0 clippy --locked -p volume-core --all-targets -- -D warnings` | 0 | No warnings. A fresh check: the sources were touched first, without any content change. |
| `python3 -B -m unittest discover -s tests -p 'test_volume_ustar*.py'` | 0 | 3 tests passed. |
| `cargo +1.90.0 fmt -p volume-core -- --check` | 0 | — |
| `cargo +1.90.0 build --locked -p volume-core --target x86_64-unknown-none` | 0 | Freestanding `no_std` build, recompiled. |

## Oracles

**Shared fixture.** [`services/volume/tests/ustar-names.txt`](../../../services/volume/tests/ustar-names.txt) holds 107 hex-encoded cases:
- 82 `name` lines, each with the expected refusal bits;
- 15 `collide` pairs;
- 10 `distinct` pairs.

Both oracles below read this file. Python does not run the Rust code, so the two implementations agree only transitively, through this fixture.

**Rust tests** (`services/volume/tests/archive.rs`):
- `ustar_case_fold_is_at_least_as_coarse_as_core_case_mapping` is exhaustive over the 510 characters of the checked set: U+0020–007E, U+00A0–017F except U+00AD, and U+2700–27BF. The key from `core` is `c.to_lowercase().to_uppercase().to_lowercase()`.
  - For every pair whose `core` keys are equal, `ustar::collide` is true.
  - Every character whose key lies inside the set collides with its key spelt as a name; this covers `ß`/`ss`.
  - The only characters whose keys leave the set are `µ`, `İ` and `ŉ`. The test asserts that this list is exact. Their keys (`μ`, `i̇`, `ʼn`) are names the check already refuses as UNCHECKED.
- `ustar_names_agree_with_the_shared_fixture` asserts `ustar::reasons` and `ustar::collide` on all 107 fixture lines.

**Python oracle** ([`tests/test_volume_ustar_names.py`](../../../tests/test_volume_ustar_names.py)) uses only the standard library. Its rules are written from the specification with `unicodedata`, not from the Rust code.
- (a) The set holds 510 characters, and none of them has a combining-mark (`M*`) or control/other (`C*`) category.
- (b) NFD is injective on the set.
  - Every character is NFC and decomposes into one starter followed by non-starters.
  - On single characters, and exhaustively on all 260 610 names of one or two characters, NFD maps no two names to the same string.
  - Folding after NFD merges no names that the case key keeps apart.
  - The case key is full Unicode upper case, with U+0130 taken as `I`.
- (c) The oracle's own `reasons` and `collide` agree with the expected values on all 107 fixture lines.

## Mutation checks

**Procedure.** The script `mutate.py` lives in the ignored runner folder `build/s09-ustar-notes/runner/runs/S09-ORACLE/1/`. It is not committed, and its logs stay there.
1. The script refuses to start unless `ustar.rs` equals `git show HEAD:services/volume/src/ustar.rs` and `git diff --quiet -- services/volume/src/ustar.rs` exits 0.
2. Every mutant replaces one source snippet. Each snippet must occur exactly once in the file.
3. For each mutant, the script writes the mutant and runs:
   - Build: `cargo +1.90.0 test --locked -p volume-core --test archive --no-run`
   - Test: `cargo +1.90.0 test --locked -p volume-core --test archive ustar`
4. A build exit of 0 shows that the test exit of 101 comes from failing tests, not from a compile error.
5. After each mutant, a `finally` block writes the HEAD bytes back and asserts that they are equal. The script turns SIGTERM into a normal exit, so the restore also runs if the script is stopped.
6. At the end, the script runs `git diff --quiet` and stores the result.

**Results.**
- The unmutated baseline exited 0, with 14 ustar tests passed.
- All 38 mutants built with exit 0, and every test run exited 101. Every mutant was killed.
- The whole run took about 16 s.
- After the run, `git diff --quiet` exited 0, both in the script and in a separate check. The sha256 of `ustar.rs` matched HEAD.

Test labels:

| Label | Test |
|---|---|
| FX | `ustar_names_agree_with_the_shared_fixture` |
| CO | `ustar_case_fold_is_at_least_as_coarse_as_core_case_mapping` |
| UC | `ustar_case_fold_matches_unicode_case_within_the_checked_set` |
| WC | `ustar_export_refuses_windows_characters` |
| WD | `ustar_export_refuses_windows_device_names` |
| TS | `ustar_export_refuses_trailing_dots_and_spaces` |
| CF | `ustar_export_refuses_names_equal_under_case_folding` |
| FD | `ustar_export_refuses_a_file_where_a_directory_is` |
| OS | `ustar_export_refuses_characters_outside_the_checked_set` |
| EV | `ustar_export_reports_every_refused_name_at_once` |
| RJ | `ustar_export_judges_names_again_and_refuses_another_source` |

| Mutant | Refusal rule disabled | Change in `ustar.rs` | Build exit | Test exit | Failing tests |
|---|---|---|---|---|---|
| C1 | CHARACTER: control characters | `if c.is_control() \|\| "<>:\"\|?*/\\".contains(c) {` → `if "<>:\"\|?*/\\".contains(c) {` | 0 | 101 | FX, WC |
| C2 | CHARACTER: Windows special characters | `if c.is_control() \|\| "<>:\"\|?*/\\".contains(c) {` → `if c.is_control() {` | 0 | 101 | EV, FX, WC |
| C3 | CHARACTER: colon alone | `"<>:\"\|?*/\\"` → `"<>\"\|?*/\\"` | 0 | 101 | FX, WC |
| U1 | UNCHECKED: characters outside the set | `} else if !allowed(c) {` → `} else if !allowed(c) && false {` | 0 | 101 | FX, OS, UC, WC |
| U2 | UNCHECKED: invalid UTF-8 | `return UNCHECKED;` → `return 0;` | 0 | 101 | FX, OS |
| U3 | UNCHECKED: soft hyphen excluded | `&& c != '\u{AD}'` → (removed) | 0 | 101 | FX, OS, UC |
| U4 | UNCHECKED: Latin Extended-A upper bound | `'\u{A0}'..='\u{17F}'` → `'\u{A0}'..='\u{180}'` | 0 | 101 | FX, UC |
| U5 | UNCHECKED: Dingbats lower bound | `'\u{2700}'..='\u{27BF}'` → `'\u{26FF}'..='\u{27BF}'` | 0 | 101 | FX, UC |
| T1 | TRAILING: final space | `text.ends_with(['.', ' '])` → `text.ends_with(['.'])` | 0 | 101 | FX, TS, WD |
| T2 | TRAILING: final dot | `text.ends_with(['.', ' '])` → `text.ends_with([' '])` | 0 | 101 | EV, FX, TS |
| R1 | RESERVED: whole rule | `if reserved(text) {` → `if false && reserved(text) {` | 0 | 101 | EV, FX, RJ, WD |
| R2 | RESERVED: CON | `["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"]` → `["PRN", "AUX", "NUL", "CONIN$", "CONOUT$"]` | 0 | 101 | EV, FX, RJ, WD |
| R3 | RESERVED: PRN | `["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"]` → `["CON", "AUX", "NUL", "CONIN$", "CONOUT$"]` | 0 | 101 | FX, WD |
| R4 | RESERVED: AUX | `["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"]` → `["CON", "PRN", "NUL", "CONIN$", "CONOUT$"]` | 0 | 101 | EV, FX, WD |
| R5 | RESERVED: NUL | `["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"]` → `["CON", "PRN", "AUX", "CONIN$", "CONOUT$"]` | 0 | 101 | FX, WD |
| R6 | RESERVED: CONIN$ | `["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"]` → `["CON", "PRN", "AUX", "NUL", "CONOUT$"]` | 0 | 101 | FX, WD |
| R7 | RESERVED: CONOUT$ | `["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"]` → `["CON", "PRN", "AUX", "NUL", "CONIN$"]` | 0 | 101 | FX, WD |
| R8 | RESERVED: COM ports | `stem.eq_ignore_ascii_case("COM") \|\|` → (removed) | 0 | 101 | FX, WD |
| R9 | RESERVED: LPT ports | `\|\| stem.eq_ignore_ascii_case("LPT")` → (removed) | 0 | 101 | FX, WD |
| R10 | RESERVED: port number 0 | `Some('0'..='9' \| '¹' \| '²' \| '³')` → `Some('1'..='9' \| '¹' \| '²' \| '³')` | 0 | 101 | FX, WD |
| R11 | RESERVED: superscript port numbers | `Some('0'..='9' \| '¹' \| '²' \| '³')` → `Some('0'..='9')` | 0 | 101 | FX, WD |
| R12 | RESERVED: trailing spaces before the extension | `.trim_end_matches(' ');` → `;` | 0 | 101 | FX, WD |
| R13 | RESERVED: base ends at the first dot | `.split_once('.')` → `.split_once('\0')` | 0 | 101 | EV, FX, WD |
| K1 | CASE: objects in one collection | `object.reasons \|= CASE;` → `object.reasons \|= 0;` | 0 | 101 | CF, RJ |
| K2 | CASE: collections | `collection.reasons \|= CASE;` → `collection.reasons \|= 0;` | 0 | 101 | CF |
| K3 | CASE: fold disabled (exact bytes only) | `folded(a).eq(folded(b))` → `a == b` | 0 | 101 | CF, CO, FD, FX, RJ, UC |
| D1 | FILE_AND_DIRECTORY: object side | `object.reasons \|= FILE_AND_DIRECTORY;` → `object.reasons \|= 0;` | 0 | 101 | FD |
| D2 | FILE_AND_DIRECTORY: collection side | `collection.reasons \|= FILE_AND_DIRECTORY;` → `collection.reasons \|= 0;` | 0 | 101 | FD |
| J1 | Re-judge in from_archive: verdict ignored | `if judged.refused() {` → `if false && judged.refused() {` | 0 | 101 | CF, EV, FD, OS, RJ, TS, WC, WD |
| J2 | Re-judge in from_archive: judge skipped | `judge(&mut judged);` → (removed) | 0 | 101 | RJ |
| F1 | Fold: sharp s = ss | `'ß' => return once('s').chain(Some('s')),` → (removed) | 0 | 101 | CF, CO, FX, UC |
| F2 | Fold: dotted and dotless i | `'İ' \| 'ı' => 'i',` → (removed) | 0 | 101 | CF, CO, FX, UC |
| F3 | Fold: long s | `'ſ' => 's',` → (removed) | 0 | 101 | CF, CO, FX, UC |
| F4 | Fold: Y with diaeresis | `'Ÿ' => 'ÿ',` → (removed) | 0 | 101 | CF, CO, FX, UC |
| F5 | Fold: ASCII capitals | `'A'..='Z' \| '\u{C0}'..='\u{D6}' \| '\u{D8}'..='\u{DE}' =>` → `'\u{C0}'..='\u{D6}' \| '\u{D8}'..='\u{DE}' =>` | 0 | 101 | CF, CO, FD, FX, RJ, UC |
| F6 | Fold: Latin-1 capitals | `'A'..='Z' \| '\u{C0}'..='\u{D6}' \| '\u{D8}'..='\u{DE}' =>` → `'A'..='Z' =>` | 0 | 101 | CF, CO, FX, UC |
| F7 | Fold: Latin Extended-A, capital on even code point | `char::from_u32(code \| 1).unwrap_or(c)` → `c` | 0 | 101 | CO, FX, UC |
| F8 | Fold: Latin Extended-A, capital on odd code point | `char::from_u32(code + (code & 1)).unwrap_or(c)` → `c` | 0 | 101 | CF, CO, FX, UC |

## Limits

- Python does not execute Rust. The two sides agree on the 107 fixture cases. Exhaustive agreement over the set comes only from the Rust coarseness test and the Python NFD test, each on its own side.
- CASE and FILE_AND_DIRECTORY are pairwise rules. The Python oracle checks the case key, but not `judge` itself; those two bits are covered by the Rust tests alone.
- The mutants disable or narrow refusal rules (soundness). No mutant widened a rule, so over-refusal of safe names is checked only by the accepted names in the fixture and the existing tests.
- The Python oracle was not itself mutation-tested.
- Not checked:
  - Windows 8.3 short names and path-length limits;
  - extraction on Windows or macOS;
  - bsdtar or Windows `tar` as readers;
  - a native or QEMU stage that runs the name check.
- The checks in this note are AI-executed host checks, not an independent human audit.
