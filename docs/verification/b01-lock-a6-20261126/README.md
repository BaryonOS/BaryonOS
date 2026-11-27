# B01-LOCK: F1 after erasure, app KEY1 call after LOCK (26 November 2026)

Issue #564; contract `docs/implementation/B01-KEYRING-LOCK.md` and
ADR-KEYRING-LOCK-20261126 (PR #563). This directory records the project-owner
decisions of 26 November 2026 on PR #573:

- Q-C option 1: an actual KEY1 call from an application after LOCK must be
  refused, without partial output or returned key material. The A6 text is
  unchanged.
- F1 counts as proven only once the required stack and buffer erasure has
  actually completed.

TEST profiles on QEMU/TCG only; no real device was used. The earlier evidence in
`docs/verification/b01-lock-l1-20261126/` stays bound to `82c5ad03`.

- Source commit: `174e78c1` on `claude/b01-lock-l1-20261126`. Every run below
  used that source (`source_sha256` `9e9628c245f9…` in each `result.json`). The
  commit that adds this directory changes documentation only.
- Each subdirectory holds the unchanged `serial.txt` and `result.json` of one
  run. All runs are local; hosted CI does not run the power, S3 or LOCK cases.

## Commands and raw exit status

| Command | Exit | Result |
|---|---|---|
| `python3 -B -m unittest discover -s tests -p test_system_harness.py` | 0 | host; the LOCK rows pin `LOCK_APP_SEND` 10, `LOCK_APP_RECEIVE` 8 and zero LOCKED markers for noscrub |
| `cargo +1.100.0 test --locked -p kernel-core` | 0 | 60 tests, including `keylock::tests::erased_needs_every_condition` |
| `cargo +1.100.0 clippy --locked -p kernel-core --all-targets -- -D warnings` | 0 | host |
| `python3 -B -u tools/test_power_system.py` | 0 | 24 stages: every power case on a fresh disk, each followed by `recover`; `power-lock` raw 0 (S5) |
| `python3 -B -u tools/test_system.py` | 0 | FR-I01: `normal` and `recover` raw 33, plus the relay cases |
| `build.py --kernel-feature test-system-lock-noscrub` + `run_system.py --case power-lock-noscrub` | 0 / 0 | expected failure `LOCK_CANARY_RESIDUE`, raw 35, no `SYSTEM:LOCK:LOCKED` |
| `build.py --kernel-feature test-system-lock-exit` + `run_system.py --case power-lock-exit` | 0 / 0 | expected failure `UNEXPECTED_EXIT`, raw 35 (A5) |
| `python3 -B tools/build.py --kernel-feature native-system-lock` | 0 | LOCK without the fixture |
| `python3 -B tools/build.py --kernel-feature native-smp-m3` | 0 | the disabled `m3_restore` block assembles |
| `python3 -B tools/build.py` + `python3 -B tools/run.py --smoke` | 0 / 0 | QEMU raw 33, `M1:PASS` (`smoke-default.log`) |

Subdirectories: `power-lock`, `power-lock-noscrub`, `power-lock-exit`, `power`,
`power-s3` (SUSPEND unchanged) and `normal`.

## F1: LOCKED only after the erasure completed

The LOCK handler now records only that the instance is gone. `latch()` runs at
every later kernel entry in every `native-system-lock` build. It sets LOCKED,
and prints `SYSTEM:LOCK:LOCKED`, only when `m3_restore` has zeroed the whole
32 KiB service stack and read it back as zero (`scrubs 1, residue 0, pending 0`).
A LOCK without that scrub halts the system. `keylock::erased()` (the pure
predicate `kernel_core::keylock::erased`) adds the remaining conditions: no
scrub pending, `scrubs >= locks`, no residue, and an entropy broker without
pending or answered bytes.

| Check in `power-lock` | Observed |
|---|---|
| right after the LOCK call, same event | instance gone, LOCKED false, `erased()` false (fixture stage 3) |
| one event later | canary copies 3 → 0, one verified scrub, residue 0; LOCKED latched, `erased()` true (stage 4) |
| at `finish` | `erased()` required after any LOCK |
| without the scrub (`power-lock-noscrub`) | `LOCK_CANARY_RESIDUE` at the latch, zero LOCKED markers |

How each §2.1 item of the contract is erased before F1 holds:

| §2.1 item | Mechanism | Shown by |
|---|---|---|
| Key Service memory: `master`, DRBG key and pool, per-call keys | `cleanup` zeroes every freed frame; its owner assert halts on a frame that still names the instance | A4: 65 recorded frames read back zero |
| replies already queued from the instance | LOCK refuses with -8 while any message from the instance is queued | A3 |
| kernel service-stack arrays (`snapshot`, `bytes`, `out`, device `buffer`) | whole-stack scrub in `m3_restore`, read back before LOCKED latches | A10 canary, scrub counters |
| kernel broker `r.bytes` | wiped after delivery; LOCK refuses with -8 while a request is pending or answered; `erased()` requires a quiet broker | A2 |
| `RECOVERY_SECRET`, the volume blob | F2, not F1; TEST label per O1 | not applicable |

## Q-C: an app KEY1 call after LOCK

In the lock profile the system app keeps its KEY1 session open, sends DONE and
waits for the kernel's go word. The kernel writes that word only after LOCKED
latched. The app then sends a raw `UNSEAL_RECORD` for the record it sealed
with key index 1 before LOCK. That request returns the plaintext while the
instance lives. The kernel reads the app's codes at its exit and pins them:

| Value | Observed | Meaning |
|---|---|---|
| `LOCK_APP_SEND` | 10 (`-10`, Revoked) | teardown revoked the app's SEND handle; nothing was delivered |
| `LOCK_APP_RECEIVE` | 8 (`-8`, Again) | no reply is queued for the app; it received no bytes |

Init's admin-ingress check from L1 still runs: 6 requests seen, 4 denied.

## Other `power-lock` values

Refusal log `[-1, -6, -4, -8, -8, -14]`, LOCK count 1, entropy objects 1 → 0,
grant holds 695.

## Limits

- No ring-3 program issues `0x92`; the fixture calls the real handler on
  init's behalf from `poll()`.
- KL5 is shown for one domain request (UNSEAL_RECORD) and init's admin
  ingress; RANDOM after LOCK is not tried.
- A10 scans the service stack only. By code reading, not a native test, the
  Engine resets a received queue slot and a retired endpoint.
- SUSPEND is unchanged; H's T5d gate will call `keylock::erased()`.
- Hosted CI does not run these cases; no physical S3, no hardware, no human
  audit.
