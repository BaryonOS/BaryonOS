# U01 presentation retirement — host and cross-build evidence

Baseline: `2bb84c204677c790d871639cbd2484a782c799be`, plus known-ABI wait fix
`e28671322596098d3ca89b54a5a23d370d6c34f9`. This does not close U01.

The preceding native gate failed after a real compositor UD fault: init exited
with code15 while awaiting pause acknowledgements. An app-owned reply receiver
stays open after its peer's SEND leaf disappears. The typed INP1/SUR1 session
therefore receives EAGAIN and cannot reach its control handler until its bounded
RPC loop finishes. Increasing the supervisor deadline or reducing a poll budget
does not repair that lifecycle dependency.

The unreleased window-only ABI1.11 syscall0x53 now has two explicit phases.
Init first requests `a2=1`, using the exact existing typed Launch/dead Process
grants. That phase closes only eight recorded app presentation handles. It does
not allocate or replace processes, keys, storage, control, epochs or the old
Process capability. Init then sends Pause and waits for both real ACKs before
requesting the existing `a2=0` construction phase. Construction requires the
prior retirement and retains its complete resource preflight.

Explicit `Session::retire()` normalizes only an individual handle's BadHandle
and attempts both closes. Other cleanup errors remain errors. Ordinary close,
RPC budgets, model contents and document ownership remain unchanged.

Executed on the isolated recovery worktree, Rust1.90.0, offline/locked:

| Command / check | Actual result |
|---|---|
| `cargo test -p capability-core` | 66 unit and 7 wait tests passed; one existing ignored test; the targeted actual-kernel retirement suite passed all 3 tests after the lifecycle follow-up |
| `cargo test --manifest-path sdk/input-client/Cargo.toml` | 15 tests passed |
| `cargo test --manifest-path ui/surface-protocol/Cargo.toml --features client` | 20 tests passed |
| `cargo test --manifest-path sdk/examples/materiaal/Cargo.toml --test chain` | 29 tests passed |
| `cargo test --manifest-path sdk/examples/calculator/Cargo.toml --test chain` | 9 tests passed |
| Strict Clippy for these five graphs | Passed, `-D warnings` |
| Formatting and `git diff --check` | Passed |
| `tools/build.py --kernel-feature native-window-session-probe` | All eight userspace ELFs, kernel and UEFI loader built/inspected; scalar flags and existing ELF budgets preserved |

Typed-client counterexamples use the actual Engine and native-shaped app RX /
peer SEND leaves. INP1 PEEK and ACK plus SUR1 COMMIT waits observe a real EAGAIN
after peer death, then fail closed within one yield of a 100000-poll budget when
the recorded leaves retire. No replay or positive ACK is fabricated. Tests also
retain other cleanup errors when one handle is already absent. App-chain tests
cover both still-open owners and an input owner already locally closed, while
preserving the draft/caret and the calculation/result/counters.

The lifecycle follow-up preflights both Engine app registrations independently
of scheduler state, explicitly rejects Dying app slots, and treats an impossible
commit-time Engine close error as a kernel invariant failure. A real Engine
test exits app1 while app0 stays live: both authorization and direct retirement
reject without changing any app0 handle or other engine byte.

Logs: `/workspace/scratch/window-recovery-{capability,calculator,materiaal}-{tests,clippy}.log`,
`/workspace/scratch/window-retirement-{input,surface}-{tests,clippy}.log`,
`/workspace/scratch/window-recovery-native-build.log`.

No VM was started for this change. Actual integrated crash recovery, preserved
pixels, storage behavior and final cleanup remain for the root native gate.

## Follow-up: explicit init recovery phases

Integrated source `b1b292aadf244dacf029b7307aebdc6103756e0d` reached real
presentation retirement in both CI and the root VM but then exited Material at
step18 without a `RESTART_GENERATION` event. The init loop retained the old
mask31 and could send Rebind8 in the very iteration that began Pause, before its
two acknowledgements or construction syscall. This is a separate policy-order
defect; it does not invalidate the demonstrated retirement wakeup mechanism.

Init now uses the actual pure `recovery::Recovery` stage machine: AwaitPause,
AwaitFreshReady and AwaitRebindAck. It resets mask3 both at recovery start and
after installation. Only after successful syscall0x53 and the existing fresh
process/capability identity guards can installation advance the stage. Both
pause ACKs, all three current service READYs and both rebind ACKs are gated by
their respective phases. The current sender/generation/epoch READY validator
is shared with the host regressions. No identity guard or timeout was weakened.

Six policy tests cover old-generation/mask31/pause0, partial pause ACKs, invalid
installation, old generation/epochs/unbound sender READY, duplicate transitions
and a second recovery. These targeted tests, strict Clippy and formatting
passed; `/workspace/scratch/window-recovery-policy-{tests,clippy}.log` records
the executions. The complete window host suite passed all 16 tests, and the
full eight-ELF/kernel/loader native cross-build passed; corresponding logs are
`/workspace/scratch/window-recovery-phase-{tests,clippy,native-build}.log`.
No VM was started for this follow-up; root owns integration and
the next native acceptance run.
