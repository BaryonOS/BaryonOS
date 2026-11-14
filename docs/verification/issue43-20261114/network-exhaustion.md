# N01 full per-owner port exhaustion — 2026-11-14

## Verified scope and source

The full 12,768-port allocation/exhaustion trial passed with all four existing
native network-service cases on source-v3
`d1bfdb99d058d4a20896b4b0e4e7fb7d7904cb6b1826b89b3f33f88723f0ee30`.
No executable source changed after that pass; this documentation followed.
The worktree was uncommitted on base
`b8a7e83658bdbcd4970c26f753dda1b342fce0d4`. The fingerprint identifies the
executed source; integration requires a separate final-source gate.

QEMU 10.0.13 (Debian 1:10.0.13+ds-0+deb13u1), machine `pc-q35-10.0`, single
CPU/TCG and disposable OVMF state were used. The synthetic local peer sent
no replies. No physical hardware, external network, TLS, automatic recovery
or performance guarantee is claimed.

## Actual operations and cleanup

Owner A completed 12,768 real NET1 open/close pairs, then received the
actual `Exhausted` result at tick 5,365. The kernel released owner B at that
tick. B opened, observed `Connecting` and closed at tick 5,402. A remained
registered through B's close and exited at tick 5,415. These were native
RPCs through the unchanged service allocator and original endpoint rights.

The service recorded exactly 25,544 decoded requests, 12,769 opens and
12,769 closes, four rejections and retired mask 3. All three processes exited
successfully and the generic capability, object, process and frame cleanup
gates passed. NIC evidence recorded 55 published/completed TX, zero RX,
two denied app NIC calls, zero TX timeouts and one reset/acknowledgement.

The exhaustion profile alone composes the existing optional `ipc-wait`
feature. Its SDK transport performs a real syscall first, waits on the same
authorized queue only after `Again`, and performs a real retry after the
advisory wakeup. Successful sends are never resent. Both independent source
reviews found no remaining blocking finding; one independently re-evaluated
the successful native report and original byte stream.

The original 6,000-tick kernel watchdog and 90-second host limit were retained.
The final A exit left 585 ticks of watchdog margin in this environment.
Per-open deadlines remain 300 ticks; production limits and the full quota
were not changed. The watchdog is an inherited fixture bound, not an issue-39
throughput requirement. No proposed larger workload budget was used.

## Same-source commands and results

All native cases ran `python3 -B -u tools/run_network_service.py --case CASE`
with `CARGO_BUILD_JOBS=2` and isolated build/target directories. Paths below
are relative to the development worktree; each retains its report and
`run/network-service-CASE-SUFFIX/{result.json,serial.txt,stderr.txt,network-peer.json}`.

| Case | Result / raw exit | QEMU seconds | Build directory / run suffix |
|---|---|---:|---|
|exhaustion|pass / 33|56.129|`build/issue39-exhaustion` / `f9005b733a51450d98da668b8f209933`|
|normal|pass / 33|6.807|`build/issue39-exhaustion-regression-normal` / `c167b5006bb9479c8bdc68fbd2739405`|
|client-exit|pass / 33|6.647|`build/issue39-exhaustion-regression-client-exit` / `65a0829413b14f43846e804b76d0534a`|
|revoke|pass / 33|6.880|`build/issue39-exhaustion-regression-revoke` / `a0002124997b43e8b8cba3c753928582`|
|device-reset|pass / 33|2.381|`build/issue39-exhaustion-regression-device-reset` / `f7e57b7264184bbb806910baf862df4d`|

The successful exhaustion `result.json` SHA-256 is
`4282ae84ad176e62bfd7e013b027423583c11661ae87f61677419b4475c18883`;
the original serial SHA-256 is
`8069c0c9f0cbdf61feb303c7fa77f8b9906fdbde3a87cd7406a4750a68380c1d`.

Host checks passed: 23 peer/oracle tests, including raw stream corruption,
truncation, cross-owner scope, quota/causality tampering and a subprocess-bound
FIFO refusal; the existing production test
`one_owner_port_exhaustion_preserves_the_other_owners_budget` (one test,
Rust 1.91.1, offline/locked); formatting and Python syntax checks. Logs remain
in `build/issue39-exhaustion-checks/`. Native builds retained the two existing
warnings in unchanged kernel lock/serial code.

## Durable raw capture

[network-exhaustion-discard.hex](network-exhaustion-discard.hex) is the exact
3,520-byte framed stream, encoded as hexadecimal with one frame per line.
It contains 55 actual ARP requests and no server frames. The SHA-256 of the
decoded bytes is
`7d5e22e621d28ce27dfa3f2ab12d851e07a0632681ede12343d5bc5676f035f2`.
`bytes.fromhex()` ignores the line separators and reconstructs the original
`network-discard.bin` without transformation. The raw file and its report
remain in the successful run directory. Frame parsing and digest checks are
independent of the claimed native allocation counters.

## Preserved failed attempts

All three failed runs remain in `build/issue39-exhaustion/run/`. None was
replaced by a successful result. The first and third ran into the inherited
watchdog; the second exposed an unsupported optional syscall in the original
profile. These are fixture execution failures, not successful quota evidence.

| Attempt / run suffix | Raw exit / QEMU seconds | Observed reason |
|---|---|---|
|1 / `7912d26c4bdc4f3c9709e825933fa649`|35 / 61.956|B failed 134 at the 6,000-tick wait bound; A's pair count was not captured.|
|2 / `2b0ec8c64b55432fade3277793726623`|35 / 1.812|B failed 141 at tick 2; the SDK wait call lacked the optional `ipc-wait` implementation in that profile.|
|3 / `e9ec825d9a844d55af9816ed95fd275c`|35 / 62.154|B failed 134 at tick 6,000; A had 9,424 opens/9,423 observed closes, while the service had completed 9,424 closes and 18,850 requests.|

Attempt 1 source:
`89cc76faf204ba15dea624e8ac0c76e65061301b94961a78451806e25fa327be`;
result SHA-256:
`6f932f40dd020cdcb9164aaca673705a0e1bbe655ebb75c1165649685c63e735`;
serial SHA-256:
`9d8742acf210d905f175f28503dc4edf5caca58107b3664b755cd62eb636471e`.

Attempt 2 source:
`d0517c4e07a44d8a199a994c89e405c0b280a3631bbb0ccc0272ff741d01168e`;
result SHA-256:
`b0a8631cdd268da0c58764db630e8b01f8d8cb6b9bdbbabcd23d88acf90e0a1f`;
serial SHA-256:
`b4c9e2fbfc325250516e0daa49b2eef073d0dd4b9acc4b22da6e91fa1b8889af`.

Attempt 3 source:
`36fed5bd0cfd9d90751b22a45b8d495af636cb69066149170175ba03ea32fee1`;
result SHA-256:
`d8f04b7bfdd0bb7b41cd6264f1c66fd64ffebcbf8611313a2f2bff3e5ed4a781`;
serial SHA-256:
`21ca219d5621f008074ccbeb2a509be372a93f62c32111e9b5bd6789cda1b6a0`.
