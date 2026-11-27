# H05-T5d-T — S3 suspend requires a LOCKED, erased Key Service instance (27 November 2026)

**Decision:** ADR-KEYRING-LOCK-20261126 (KL2), H05 S3 track. **Profiles:** the
opt-in TEST kernel features `test-system-s3*` and the new labelled
`test-system-s3-lock-busy` (not a published ABI). **Status:** lead-reviewed; an independent AI verifier (not a human audit) found no blocking findings;
verified locally in QEMU on base `20d9089f` plus the uncommitted working
tree; not accepted, H05 stays open. Hosted CI has not run.

## What changed

- **KL2 in SUSPEND (0x91).** After the argument, authority and sleep-ready
  checks, SUSPEND calls `keylock::latch()` and refuses with -25 unless the
  Key Service instance is LOCKED and erased. The check runs before the
  storage gate. A LOCK (0x92) in the same boot reaches SUSPEND only in a
  later kernel entry, after `m3_restore` has cleared the scrub request
  before the CPL3 `iretq`, so no retry on -25 exists; any -25 after
  LOCK=0 is a failure.
- **Every S3 profile carries LOCK.** `test-system-s3` now enables
  `native-system-lock`; a `compile_error!` refuses `test-system-s3`
  without it. The system init gains feature `lock-caller` (only through
  `test-system-s3`; a `compile_error!` refuses it otherwise).
  `sdk/examples/system/build.py --test-system-s3` selects it, and
  `tools/build.py` passes that flag only for S3 kernel profiles.
- **Init sequence.** Check 70 now expects -25 (no LOCK yet). After check
  73, init issues LOCK with its POWER handle (bounded retries on -8),
  then the granted SUSPEND. The kernel's exact SUSPEND log is now
  `[-25, -6, -4, -1, 0, -14]`, or `[-25, -6, -4, -1, -8, 0, -14]` with a
  held block completion.
- **Re-init target without the Key Service (C1).** After LOCK the
  instance is gone, so the re-init profiles prove the validated
  virtio-rng re-init through a kernel-owned binding
  (`pid = u64::MAX`, handle slot 0, which ring 3 cannot produce). It exists
  only under `all(test-system-s3-reinit, native-system-lock)`. The
  delivery buffer is zeroed after use. The 0x61 completed path now shares
  `redeem()` with this kernel delivery.
- **A7 `power-s3-lock-busy` (labelled).** Before each init LOCK attempt a
  wrapper submits one kernel entropy request while the broker is quiet, so
  LOCK returns -8 eight times. SUSPEND then refuses with -25 and init
  falls back to S5 once the kernel has delivered its requests (init word
  640). The case prints `SYSTEM:S3:LOCK_BUSY:LOCK=-8:SUSPEND=-25:S5` and the
  O5 label. It does not print the "instance erased" label.
- **Host oracle.** `tools/run_system.py` requires `SYSTEM:LOCK:LOCKED`
  exactly once (between the manifest and the S3 entry), the eight `LOCK_*`
  counters with exact values, entropy objects falling, and the exact
  `SYSTEM:S3:` label set for each S3 case. `power-lock` keeps its existing
  expectations.

## Executed evidence

| Command | Result | Log |
|---|---|---|
| `python3 -B -u tools/test_power_system.py` | exit 0, `passed=True`, 26 stages (13 cases plus 13 cold recoveries) | `power-gate-2.log`, `power-system-gate-2.json` |
| `rerun_s3.py 2` (each S3 case twice more, fresh disk) | 18/18 passed, source unchanged | `s3-reruns.log`, `s3-reruns.json` |
| `python3 -B -m unittest discover -s tests -p 'test_system_*.py'` (as `ci.yml#host-quality`; includes the renamed `tests/test_system_s3_lock_guard.py`) | 112 tests OK, exit 0 | `unittest-lead.log` |
| `cargo +1.100.0 fmt --all -- --check` (root, sdk) | exit 0 | `fmt-check-root-2.log`, `fmt-check-sdk-3.log` |
| `cargo +1.100.0 clippy --manifest-path sdk/Cargo.toml --locked --all-targets -- -D warnings` | exit 0 | `clippy-sdk-2.log` |

Source fingerprint (v3) of the gate and the reruns:
`1f2f6b89dba0485fd8335fd8ac43132eecfffae81b5f930eaca50c955931142d`.
Log names refer to the builder's local log directory; raw serial logs are
not committed. Per-case results are in `summary.json`.

The first gate attempt (`power-gate-1.log`) failed at `power-s3`, init step
82. The cause was init seeding `rdx` with 7: `rdx` is also argument 2 of
the int80 ABI, so LOCK refused with -1. After that fix the kernel's own
SUSPEND log pin in `finish` still expected the old -8 first entry; it now
expects -25, and a structural test pins all three log variants.

## Mutants

| Mutant | Result |
|---|---|
| m1: remove the KL2 branch | killed: `power-s3` and `power-s3-lock-busy` fail at init step 70 (raw exit 35); structural test fails |
| m2: drop the cfg gate on the `KERNEL_BINDING` check in `binding_is_live` | killed: E0425 in `test-system-s3` and `native-system-power`; structural test fails. Equivalent in re-init profiles, where the gate condition holds |
| m3: skip `out.fill(0)` in `kernel_deliver` | survives natively (`power-s3-reinit`, `power-s3-lock-busy` pass); only the structural source test catches it |

## Not run

- **`power-s3-noscrub`:** B's widening of `test-system-lock-noscrub` to the
  S3 profiles is not on `main`, so no S3 red control for the stack scrub
  exists in this change.
- Hosted CI (`native-power.yml`, `ci.yml`), hardware, and an independent
  human audit.

## Limits

- QEMU `pc-q35-8.2` with OVMF only, `-accel tcg,thread=single`, `-cpu qemu64`,
  `-smp 1`, 256 MiB (from the run `result.json`).
- The erased label refers to the TEST unwrap secret, which is a public
  constant. No key-state durability across S3 is claimed (O5 open).
- The zeroing of the kernel delivery buffer (m3) has no runtime oracle.
- KL2 precedes the storage gate, so no S3 case now exercises SUSPEND's
  storage-gate -8 refusal natively (check 70 previously did).
- `LOCK_REFUSALS=0` in the S3 cases is an exact oracle pin that held in
  27/27 runs; it is empirical, not structural.
- `tests/test_system_s3_lock_guard.py` matches the existing
  `ci.yml#host-quality` discovery pattern `test_system_*.py`; it needs `cargo`
  on the runner and skips with a stated reason otherwise.
- With `native-system-lock` in every S3 profile, `system_lock.rs` emits an
  unused `STACK_BYTES` warning in S3 builds without `test-system-lock`.
  The kernel build does not use `-D warnings`; the file belongs to another
  owner and is unchanged.
