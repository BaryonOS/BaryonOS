# H02-T3 native same-port USB keyboard reattach

The opt-in `native-usb-hid-reattach` profile passes in QEMU. It performs a real
same-port keyboard re-enumeration in the isolated xHCI/VT-d fixture and
rejects generation-1 work after slot reuse. The existing `reports` and
`readonly` cases still pass with unchanged markers. H02 remains **open**.
No score change is proposed.

## Source identity

- Base: `9c707c8` (main `035b44e` plus the local H05-T3 commit).
- Executed source-v3 fingerprint:
  `a5894c23b6006b615f0a75c1fffd72a71903226d2ff8f3dbd550403185e17b22`.
  Documentation and evidence files are outside this fingerprint. The H02-T3
  commit therefore has the same fingerprint as the executed source.
- Toolchain: Rust 1.100.0, QEMU 8.2.2 (Debian 1:8.2.2+ds-0ubuntu1.18),
  `pc-q35-8.2`, qemu64, single-thread TCG, one CPU, 256 MiB, OVMF 4M from
  `/usr/share/OVMF`.

### Execution checkout

The native gates ran from an exact content copy of the task worktree, located
outside the parent checkout. The two source-v3 fingerprints were compared and
equal before execution. The copy has no `.git`, so each report's
`base_commit_field` is `null`; the fingerprint is the binding.

The copy was necessary. The task worktree is nested inside the main checkout,
so Cargo merges both identical `.cargo/config.toml` files. The kernel
`rustflags`, including `-Tkernel/linker.ld`, are then passed twice. lld
re-applies `SECTIONS` and redefines `__text_end` and `__kernel_end` as
`0x2000000`. Every M2 profile then takes an instruction-fetch page fault
directly after the CR3 switch (QEMU log: `v=0e e=0010`, then a triple fault),
which QEMU reports as raw exit 0. This affected unmodified `035b44e` and
`9c707c8` identically. It is a host-layout artifact, not a source defect.
No tracked configuration was changed.

## Native results

Each row is a separate boot with raw QEMU exit **33**, no timeout, matching
executed artifacts and the fingerprint above. The [compact report](native-report.json)
contains commands, artifact and serial digests, ordered markers and QMP replies.

| Case | Kernel feature | Observed result |
|---|---|---|
| `reattach` | `native-usb-hid-reattach` | Full `reports` sequence, then detach, generation-2 keyboard enumeration, slot reuse, three stale rejections, C press/release, second unplug and a refused third attach; retired canaries and guard unchanged; 13 QMP replies |
| `reports` | `native-usb-hid-probe` | Unchanged marker sequence and 7 QMP replies |
| `readonly` | `test-usb-hid-readonly-fault` | Unchanged VT-d write denial, reason 5 |

The `reattach` case passed three consecutive runs (4.97 s, 4.53 s and
4.60 s). The report lists all three.

### Observed QEMU 8.2.2 behaviour

- `device_del` produced a port-status change with CCS=0. `device_add` produced
  one with CCS=1 and PED=0 before any reset, and the guest accepts it only in
  its Detached wait. While Stop Endpoint and Disable Slot commands are in
  flight, the existing command wait still acknowledges port-status events
  without inspecting them. The run therefore does not establish an exact
  count of port events per transition.
- Enable Slot returned the previously disabled slot, so generation 2
  reused generation 1's slot (`USB:REATTACH:SLOT_REUSED`). The replayed
  generation-1 event matched slot and endpoint and was rejected only by its
  TRB address.
- The fixture does not record whether a stopped Transfer Event preceded a
  Stop Endpoint completion. It accepts at most one correlated stopped event.
- In the first local run (earlier fingerprint `04052f92…`, without the quiet
  window), the guest emitted every marker with raw exit 33. It refused the
  third attach and exited before QEMU answered `device_add hid-kbd-g3`. The harness then failed on
  the missing reply. A bounded post-denial quiet window now precedes the
  exit; it also requires no further event and an unchanged CCS=1/PED=0 port.
  The harness still requires every reply.

## Host checks

| Command | Exit | Result |
|---|---:|---|
| `cargo +1.100.0 test --locked --manifest-path drivers/usb/Cargo.toml` | 0 | 49 tests, including the new ticket-lifetime test |
| `cargo +1.100.0 clippy --locked --manifest-path drivers/usb/Cargo.toml --all-targets -- -D warnings` | 0 | |
| `cargo +1.100.0 fmt --manifest-path drivers/usb/Cargo.toml -- --check` | 0 | |
| `cargo +1.100.0 fmt --all -- --check` | 0 | |
| `cargo +1.100.0 build --locked --release --manifest-path drivers/usb/Cargo.toml --target x86_64-unknown-none` | 0 | |
| `python3 -B -m unittest discover -s tools/tests -p 'test_usb_hid*.py' -v` | 0 | 15 tests |
| `python3 -m compileall -q tools` | 0 | |
| `cargo +1.100.0 check ... -p os-kernel --features native-usb-hid-reattach,test-usb-hid-readonly-fault` | 101 | Expected `compile_error` exclusivity |
| `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` | 1 | Not caused by this change; see below |

The full tools suite has one failure and six errors, all present on the
unmodified base `9c707c8`. `test_gs_discipline` failed there because H05-T3
added GS writes in `kernel/src/m2/sleep.rs` and `arch.rs`; the integration
branch fixed it in `c54b9e2` (shared `arch::load_tables`) and it passes on
the merged tree. The six `test_agent_isolation`/`test_agent_pipeline` errors
come from this host's git controller environment (older git without
`--no-lazy-fetch`), not from this change.

## Re-verification on the integrated tree

The builder ran its native gates from an exact copy outside the checkout,
because a worktree nested inside the main checkout makes Cargo merge both
`.cargo/config.toml` files and pass the kernel linker script twice. After
the lead merged the branch as `af9b0bc` on `ccr-38238c58-g8y1l6` (on top of
the H05 S3 and H06 work), all three cases were rerun from the main checkout
on source fingerprint v3
`1ae4ed7f77917d6c1b4d8c560662923c1f659c146d884e7d71945f4936711658`
(reports' `base_commit` `af9b0bc6956a`): `reports` raw 33 in 2.60 s,
`readonly` raw 33 in 2.71 s, `reattach` raw 33 in 4.53 s and again on a
second run; USB crate tests, root `fmt --all`, `test_usb_hid*.py` and
`test_gs_discipline.py` exit 0. Those local reports stay in ignored
`build/h02-*/`; the committed `native-report.json` remains the builder's
fingerprint-bound record.

## Security review and follow-up

A read-only AI security review of the merged tree (`af9b0bc`) found no
blocking issue. It checked DMA scope and direction, slot reuse and ring
continuation, untrusted lengths, fail-closed teardown, profiles without the
feature, and every new `unsafe` block. It reported one low finding and four
informational observations:

- **O1 (low, fixed).** The native `ports.complete(old_ticket) == Stale` check
  ran after generation 2 was `Ready`, so it would have refused any ticket. It
  showed phase-based, not generation-based, rejection. The follow-up commit
  adds a second check inside the generation-2 attach while that port is still
  `Enumerating`: the generation-1 ticket must return `Stale`, the port must
  stay `Enumerating`, and the generation-2 ticket is then accepted in that
  phase. A failure reports diagnostic code 51.
- **O2–O5 (informational, not changed).** The new slot is not compared with
  the live mouse slot (a conforming controller cannot return it with
  `MaxSlotsEn=2`). The shared DESCRIPTOR and INPUT_CONTEXT pages are not
  canaried, and the IOMMU cannot separate generations of one requester. The
  mouse transfer stays armed during reattach, and any mouse report fails the
  fixture closed. The report-page clear became an 8-byte clear in every
  profile.

Verification of the follow-up, from the main checkout on base `1f64575`,
source fingerprint v3
`6ed7b9a2b1e42b3c204100da579be5a81e9cf6f36697ca829f6173cac5757b62`:

- `reattach`, `reports` and `readonly` each passed with raw exit 33 and
  artifact binding (4.47 s, 2.66 s, 2.68 s). Root `fmt --all -- --check`
  exited 0.
- **Mutation control.** The lifetime and attempt comparison was temporarily
  removed from `Ports::complete`, then restored. With that mutant, `reattach`
  failed at the new check: `USB:REATTACH:DETACHED`, then
  `USB-DIAG:REJECT:0000000000000033`, then `M1:FAIL:KERNEL_PANIC`, and the
  harness rejected the run. So the native check discriminates by generation.

The local reports stay in ignored `build/xhci-o1/`. An AI review is not an
independent human audit.

## Limits

- QEMU `device_add`/`device_del` are emulated hotplug. Physical hotplug,
  physical reset, IRQ delivery, hubs, other ports, mouse reattach and
  enumeration retries are not covered.
- The retired-page canary detects stray writes during this run; it does not
  prevent them. Retired pages stay mapped until the VM exits.
- The replayed completion is a captured real event presented again by
  software, not a hardware-generated stale event.
- No input-policy authority, long-lived owner or userspace handoff exists.
- `.github/workflows/native-xhci.yml` runs the `reattach` case since the
  lead's integration commit; hosted results are recorded on PR #409.
