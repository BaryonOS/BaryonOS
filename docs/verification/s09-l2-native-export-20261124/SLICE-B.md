# S09 phase 3 slice B: `l2-portal` build measurement

Slice B of `docs/implementation/S09-L2-NATIVE-EXPORT.md` (section 9) adds the
`l2-portal` feature of `volume-app`. Its acceptance is a measurement: every
portal links within every linker bound, and the deepest flow fits the stack.
This record replaces the `SLICE-B.md` that the slice-B branch cited but never
committed.

**Evidence level:** host build and static analysis. No guest ran any `l2-portal`
code; nothing selects kernel cases 25–28 yet (slice C). This is not native
acceptance and not a human audit.

## Source and tools

- Source: `origin/main` `1fc9b8ca` plus the slice-B commits on
  `claude/s09-l2-slice-b-main`. The exact tested head is recorded in the pull
  request, not in this file.
- Toolchain: `rustc 1.100.0` (repository pin), `x86_64-unknown-none`,
  release profile at `CARGO_PROFILE_RELEASE_OPT_LEVEL=s` (the
  `sdk/examples/volume-docs/build.py` default).
- Commands, each with its own `CARGO_TARGET_DIR`:
  `python3 -B sdk/examples/volume-docs/build.py [--archive-portal | --l2-portal]`,
  all exit 0. Sizes from `size -A`; stack from `stackbound.py` (this
  directory) on `objdump -d` output.

## Section sizes (bytes)

Bounds from `sdk/examples/volume-docs/linker.ld`: `.text` ≤ 65 536;
`.data` + `.bss` ≤ one page (4 096); `.rodata` ≤ 32 768. No binary has a
`.bss` section.

| Binary | Variant | `.text` | `.data` | `.rodata` |
|---|---|---:|---:|---:|
| `volume-app` | default | 35 061 | 1 088 | 4 376 |
| `volume-app` | `archive-portal` | 50 123 | 1 328 | 8 696 |
| `volume-app` | `l2-portal` | 47 477 | 1 088 | 4 232 |
| `volume-service` | (all variants) | 50 101 | 1 096 | 21 192 |

Every bound holds. The linker asserts them, so a violation is a link error.

## Static stack bound

`stackbound.py` adds, per function, 8 bytes of return address, 8 bytes per
`push` and every `sub rsp` immediate. It takes the maximum over direct-call
paths from `_start`. It follows direct tail jumps (`jmp` to another
function) conservatively, as if they were calls. It does **not** follow
indirect calls or jumps. An unknown root or a recursive cycle makes it exit
1 without a bound; none of the measured binaries has one. It is an estimate
of the direct-call bound, not a proof of the worst case. Counting tail jumps
(review finding on #482) left every bound below unchanged.

| Binary | Variant | Bound from `_start` (exit 0) | Largest frames |
|---|---|---:|---|
| `volume-app` | default | 6 888 | `run` 1 520 |
| `volume-app` | `archive-portal` | 9 952 | `run` 3 824 |
| `volume-app` | `l2-portal` | 21 688 | `l2_used` 11 952, `l2_restore` 9 712, `l2_export` 8 528 |
| `volume-service` | (all variants) | 55 024 | `_start` 20 592, `recover` 20 384 |

The app stack is 64 KiB. The `l2-portal` bound leaves about 43 KiB of
headroom. `l2_export`, `l2_restore`, `l2_used` and `l2_portal` are
`#[inline(never)]`, so their `ArchiveV2` copies are not in the frame of `run`.

The `volume-service` binary is not changed by slice B. Its direct-call bound
of 55 024 bytes is close to the 64 KiB stack; that is a pre-existing margin
to watch, not a slice-B result.

## Other checks

- `cargo clippy --locked -p nieuw-os-volume-docs-example --features <f>
  --bins --target x86_64-unknown-none -- -D warnings` for `native`,
  `native,archive-portal` and `native,l2-portal`: exit 0.
- `cargo fmt --manifest-path sdk/Cargo.toml --all -- --check`: exit 0.
- The native 26-stage volume gate is regression evidence for the shared
  `app.rs` and `build.py`; its result is recorded in the pull request.

## Review

An independent AI review of the rebased diff (read-only) found no blocker.
It reported four items that this change fixes: stale status text in the
design document, the missing measurement record, the meaning of words 536
and 568 after #477, and step ids 912–915 in `l2_source_step` that collided
with `meta`'s internal `step + 1` (now 916–919). Remaining notes are open
items in section 10 of the design document. An AI review is not a human
audit.
