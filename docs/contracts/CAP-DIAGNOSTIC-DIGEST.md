# Capability diagnostics — bit-exact variants and native comparison

## Current outcome: host gain did not predict the TCG result

The first zero-run variant is in db04c385ee8a19c45594b267f5dfa654a9b8919e.
The host ratio of approximately 2.48 below was actually measured, but the
GUI test integrated with it became slower and failed. This is not a native
performance improvement or signed-off GUI acceptance. Source commit
1fd879dfdde88890a68f0a23901c69acd464b582, source fingerprint v3
6f9410a38d6cc1323344489c19c57f4e59c5a858e41f8e0710d7792065081fd3:

- The integrator reported 170.0707855s to the first Ready capture with 1608
  tiles, compared with approximately 106s for the earlier variant. The 170.07s
  derives from the difference between capture and input-manifest mtimes and
  includes more than digest work alone.
- The actual result is `passed:false`, raw QEMU exit35, guest_timeout true and
  duration 203.591759599s. At its deadline, the guest reached 1932 tiles and
  one complete frame; no Saved/STOP/cleanup acceptance.
- The result was directly inspected at
  `/workspace/nieuw-os-worktrees/root-materiaal/build/materiaal-digest/run/materiaal-normal-230683c431/result.json`,
  SHA256 68c5cf5dd67d9b63c8b1c03a29bce9ab9a488d33fe1380eac52c9fa9c4504b28.
  The 200s timeout is a guest boundary; the recorded host watchdog is 240s.
  That report contains the exact kernel, ELF, firmware, QEMU and disk hashes.

PERF-NATIVE-BENCH-1 therefore adds only an explicit measurement profile.
The ordinary production `state_digest`, `stats`, rights and all before/after
checks for every syscall remain byte-identical to db04. This step selects no
new production variant and does not increase the GUI timeout.

## Opt-in native measurement contract

Crate feature `capability-core/diagnostic-bench` builds two additional
read-only methods: `Engine::diagnostic_reference_digest()` and
`Engine::diagnostic_direct_digest()`. The first uses the literally preserved
old body from 3f20; the second differs only by hashing the 40 Message wire
words directly instead of through encode(). Every word unconditionally
receives the same XOR/multiply; no zero detection, zero run or live/len
selection. All existing differential corpora now also test this direct
variant. Without the feature or cfg(test), neither additional path is
built. No default feature or dependency.

The new module `kernel/src/m2/m3/cap_bench.rs` provides
`pub(super) fn run(engine: &capability_core::Engine) -> !`.
Root owns the module declaration, feature alias, hook, builder and runner:

```text
native-cap-digest-bench = [native-system, capability-core/diagnostic-bench]
State::setup();
cap_bench::run(m4::engine()); // vóór timer_init, IF=0, één CPU
```

This uses the actually constructed four-process manifest graph and existing
kernel helpers `clock()->u64`, `print_number(&[u8],u64)` and
`serial::{write,exit}`. No second Engine is created and no process executes.
The manifest assertion requires 4 processes, 25 handles, 35 nodes,
11 open endpoints, 0 queued messages, 1 block, 1 launch and 1 entropy object,
0 process/network/display/input/POWER objects and valid structural invariants.
The 10 closed parent nodes remain present because of the derived nodes. The
POWER object table participates in the digest but has no live object in this
benchmark profile. The current manifest digest is `0x77cd1386b23a996b`,
decimal 8632577529975839083; the program calculates the observed value.

The three fixed variants are REF, DIRECT and ZERO. For each variant:
one pre-hash check, 4 warm-up calls, 144 measured calls and one post-hash check.
Measurement consists of 9 rounds × 16 calls; the order rotates ABC/BCA/CAB
and so on. Each variant therefore occupies each measurement position equally
often. The guest records the actual variant order per round as
`CAPBENCH:ROUND_VARIANT`; the runner checks that order alongside measured
times and derived totals. Each call uses `black_box` for the function pointer,
Engine reference and result. Each outcome contributes to a difference
accumulator and is compared with the expected hash after the batch. No Stats
or serial operation occurs inside the measured interval. All Stats fields
before and after must be exactly equal; complete structural invariants are
checked again. No actor/device operation, allocation, Engine clone or unsafe
in the measurement module.

The existing clock helper uses **unserialised RDTSC**. Results are coarse
QEMU/TCG timestamp differences per batch, including loop, indirect-call,
black_box and parity-calculation work. They are not physical CPU cycles,
exact per-call latencies, host wall-clock or real-time guarantees. The runner
records QEMU version/profile and actual total duration. The target for this
small test is less than 5s of guest measurement work; that is not yet a measured
fact. The count is not adjusted afterwards to obtain a preferred winner.

The exact output order is:

1. `CAPBENCH:MANIFEST:PASS`.
2. Numeric lines `CAPBENCH:{NAAM}:` with 20 decimal digits:
   DIGEST,PROCESSES,HANDLES,NODES,ENDPOINTS,WARMUP,ROUNDS,BATCH.
3. REF_COUNT,REF_TSC_SUM,REF_TSC_MIN,REF_TSC_MAX; the same four with DIRECT_
   and then ZERO_. COUNT is 144 measured calls, SUM is the sum of 9 batch
   intervals, MIN/MAX are the extreme **16-call batch intervals**.
4. For each round, three ROUND_VARIANT lines with the actually executed
   indices (0=REF, 1=DIRECT, 2=ZERO), followed by ROUND_REF_TSC,
   ROUND_DIRECT_TSC and ROUND_ZERO_TSC in fixed variant order, for rounds
   0..8. The host checks rotation, sum, minimum and maximum and retains the order.
5. `CAPBENCH:PARITY:PASS`, `CAPBENCH:STATE:PASS`, `CAPBENCH:PASS`, `M1:PASS`;
   exit0x10 gives raw QEMU33. Any deviation produces CAPBENCH:FAIL and
   exit0x11 (raw35), including hash, state, manifest, time-progression or
   sum-overflow errors.

The profile receives an actual empty disposable disk and virtio-blk/rng as
required by system-manifest construction. No user apps, disk/entropy syscalls
or other actors run during measurement. A passing CAPBENCH is therefore
explicitly **not** a functional app, GUI, Saved, entropy, disk, cleanup,
security or release test. The integrator selects a subsequent production
variant only after actual measurements and then repeats the complete GUI
test with unchanged syscall checks.

### Component checks of the measurement path

Without the feature: 52 tests passed, 1 benchmark ignored. With diagnostic-bench:
53 tests passed, 1 ignored; the additional test builds the same manifest
graph, checks all three hashes and compares the complete Engine and Stats
before and after. Both sets retain all 47 original core tests. The old
reference body remained byte-identical to 3f20, SHA256
29ca84c1172b4c8dffd6f4aa36270ce23fa349075963d287fd0c9b08628cb7ed.

Dedicated checks under `build/cap-native-checks/`: fmt, both host test sets,
Clippy/all-targets with the feature, scalar-none release build/Clippy with
and without the feature: all exit0. A separate no_std compilation harness
type-checks the actual cap_bench module for scalar-none with external,
never-executed clock/serial symbols; build and Clippy exit0. That compilation
harness is not native measurement execution. The ordinary production rlib
contains no reference/direct symbols. The dedicated component source
fingerprint v3 is
8f59dbbc1169ef8b78af3c57f961c0bab518e7b129e918eeb2dddc78d823d49b.
Native source/image hashes and measurement outcomes derive only from root's
runner; this component check does not supply them.

## Historical initial optimisation and host investigation

This changes only the calculation of the existing diagnostic
`Engine::state_digest` in `crates/capability-core/src/lib.rs`. Stats fields,
authority, error order, PID/slot/node generations, queues, ABI and limits
remain unchanged. There is no digest cache, dirty flag or live/len selection.
The kernel retains its complete before/after digest check for every syscall,
including EAGAIN. Native GUI duration requires separate measurement; host
time is not evidence for it. No release criterion or score is signed off.

## The same complete word stream

The base is commit 3f20f087a7f0ddf4757af8d3ea110dd0c044a72c. The existing
mixer starts at 0xcbf29ce484222325 and performs, for every typed word:
`h = (h XOR woord) * 0x100000001b3 modulo2^64`.
Word order and count remain exactly identical for every valid Rust field
value, including invalid logical states. Compiler padding was not part of
the old typed digest and is still not interpreted. These diagnostics are
not a cryptographic hash, authentication or proof of collision freedom.
Engine equality and actual capability checks retain their own roles.

Every Message yields the same 40 little-endian wire words as encode():
senderPID, tag, `u64(len) | (u64(cap_count)<<32)`, reserved, both complete
cap-handle/rights pairs and all 32 payload words. Nothing is sliced by len
or cap_count, and no unused byte is treated as implicitly 0. Queued cap IDs
are then hashed separately as before. This removes the 1024 temporary
320B encodes per digest without skipping any field or byte. QUEUE_CAPACITY
is 16, hence 64×16=1024 messages and 327680B of temporary encode output per
old digest; a doubly checked EAGAIN incurred 655360B for this alone.

A 40-word message may be added as a zero run only after every header/cap
field and every payload word has actually been found to be 0. On any
deviation, all 40 values are processed normally, including an inactive
endpoint, len0, cap_count0, noncanonical reserved/unused capabilities or
nonzero trailing data.

## Exact zero-run composition

For a zero word, `(h XOR0)*P = h*P`. A run of k actually checked zero words
is therefore exactly `h*P^k modulo2^64`. The private DiagnosticHasher counts
such words. Before every nonzero word, the complete factor is applied,
followed only then by XOR and multiplication for that nonzero word. Finish
processes the final run. No zero words are therefore omitted from the result.

A compile-time table of 65 u64 factors (520B, P^0..P^64) handles short runs.
Longer runs receive the same wrapping factor through binary exponentiation.
That complete arbitrary-state fallback is separately cold/noinline, so it
is not copied into every hot-path word operation. Invalid fully zeroed
tables are still examined by the complete fallback. The fixed maximum
stream contains 74575 words: 1 lastPID, 16×(2+256×4) processes/slots,
1024×12 nodes, 64×(8+16×44) endpoints/queues, 4×9 block,
8×7 launch, 16×9 process and 11×6 console/entropy/network/POWER words.
The local usize zero counter cannot overflow under these fixed bounds.

No heap, unsafe, raw cast, wire/struct-layout change, expansion of rights or
new dependency. Only the local hash calculation changes; the engine is
observed read-only. Existing serialisation of engine operations remains
required. No new SMP, constant-time or formal-verification guarantee is claimed.

## Reference and differential checking

The old state_digest body was copied literally from 3f20 as
`state_digest_reference`; only the function name differs. The body is
compared bytewise with the source version, with the stored SHA below.
Regular production builds do not contain this reference function. The explicit
diagnostic-bench profile above enables it, in addition to cfg(test).

Alongside all 47 existing core tests, five active test groups were added:

- All 320 wire bytes × 8 individual bits × 5 initial hash values, plus 256
  fully mixed messages; forbidden sender/reserved values, cap fields beyond
  count, large len/count values and every trailing payload byte still participate.
- Exact zero-run factors around 0/1/40/64/65 and up to the 74575 words of a
  fully zeroed engine, plus 10000 mixed zero/nonzero words.
- Actual registrations, full/wrapped queues, EAGAIN equality, cap transfer/revoke,
  all object classes, reported process death, close and exit. After the
  relevant steps, old/new digests and the Stats digest are compared.
- Three deterministic dense corruption corpora covering every field of every
  table, including inactive queues, invalid indices/counts/scopes/generations
  and all message bytes. The digest does not dereference such IDs.
- A completely logically invalid zero state plus separate changes in the
  first/middle/last inactive queues and private fields, with comparison and
  an observed change in the diagnostic value.

The sixth new test is ignored by default and runs only explicitly as a
host microbenchmark. It compares the unchanged reference and production
variant on the same 7-process/21-endpoint engine using std::Instant/black_box.
That is host cost/ratio research, not a native OS performance test.

## Measurements and reproducible checks

The first separate benchmark used 500 calls × 5 samples per state and took
the median. Canonical host measurements use the normal x86_64 host target,
Rust1.90.0 release, codegen-units1, LTO disabled. An earlier exploratory host
run with SSE disabled produced a Rust host-ABI warning; it is not a canonical
comparison. All listed benchmarks are integer code; the freestanding scalar
build is listed separately below. Shared host load causes visible variation.

| Variant | GUI-like digest µs | GUI-like Stats µs | Full 1024-queue digest µs |
| --- | ---: | ---: | ---: |
|Original|87.323|90.946|89.560|
|Typed wire hash only|89.492|104.281|86.323|
|Combine only a 40-zero-word message|45.282|49.890|85.577|
|General zero runs, cold long path|40.439|39.416|76.290|

The selected variant also benefits the dense corpus; nothing is hidden to
favour only the empty case. Exact digest values remained equal:
empty55dcd2dffcdca9c3,7-process5ff47a6aeb481b3d,
full queues0e68e4b80f5acd13. Removing temporary encoding provided no demonstrated
native gain; the actual failed GUI test is at the top. Root must rerun the
GUI200s gate, actual input/frames and relevant native regressions on the
exact integrated commit/image.

Focused commands, dedicated CARGO_TARGET_DIR and managed LOCALAPPDATA/PATH:

```text
cargo +1.90.0 fmt -p capability-core -- --check
cargo +1.90.0 test --offline --locked -p capability-core
cargo +1.90.0 clippy --offline --locked -p capability-core --all-targets -- -D warnings
cargo +1.90.0 test --offline --locked --release -p capability-core digest_differential::host_microbenchmark_reference_and_optimized -- --ignored --nocapture
cargo +1.90.0 build --offline --locked --release -p capability-core --target x86_64-unknown-none
cargo +1.90.0 clippy --offline --locked --release -p capability-core --target x86_64-unknown-none -- -D warnings
```

The none build uses the same scalar flags as the kernel: code-model=kernel,
relocation-model=static,no-redzone and SSE/SSE2/SSE3/SSSE3/SSE4.1/SSE4.2/AVX/AVX2
disabled, with explicit root-path remapping. The produced digest function is
inspected for FP/SIMD, Message.encode calls and the presence of the test
reference. Logs and provisional benchmark harnesses reside only under the
dedicated build/target paths; the compact source/test binding below remains in Git.

### Recorded component checks

On the frozen code, source fingerprint v3 remained identical before and after
the checks:
`cc6dc31a7365704210828457e97e2169ec0d4c838fc6f29f85ab0d0ff5ae7410`.
The modified lib.rs has SHA256
`4c48675b247e4a9f6076c228fa610ade884f235de373b4a91132286c144f24ea`.
The reference body retained byte-identically from 3f20 has SHA256
`29ca84c1172b4c8dffd6f4aa36270ce23fa349075963d287fd0c9b08628cb7ed`.
The scalar-none rlib has SHA256
`8c60a3727f7cc960d3822404d402288d6303df94b6610482eb9d4cde7056555f`.

All focused commands above: exit0. The normal test loop reports
52passed,0failed,1ignored: all 47 existing tests plus five new groups.
The explicitly started ignored benchmark reports 1passed. Its final
7-sample comparison on the same 7-process/21-endpoint engine:

```text
reference=true median_ns=88906 digest=5ff47a6aeb481b3d samples=[87197,87438,88384,88906,89193,89213,94514]
reference=false median_ns=35894 digest=5ff47a6aeb481b3d samples=[34111,35271,35858,35894,35981,38187,38539]
```

This is an observed host ratio of approximately 2.48, not a native GUI/product
budget. The actually generated none digest function contains no FP/SIMD or
Message.encode call. The reference function is absent from the production
archive. The scalar state_digest symbol size is 11929B; the separate cold
long-path factor is 64B. This is a focused calculation/code-size trade-off,
not a claim of a smaller complete kernel or a formally proven implementation.

Still to be performed by the integrator: independent patch review, a new
integrated kernel/image with its own hashes, the complete GUI200s test and
relevant native regressions. If improvement is insufficient, the actual
bottleneck will be investigated further; checks or mandatory functions are
not disabled to obtain a passing result. The complete test release remains
open, and this component receives no independent progress points.
