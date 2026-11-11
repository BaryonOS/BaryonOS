# Native RANDOM trial alongside the integrated system chain

TASK-RANDOM-NATIVE-PROBE, baseline 985ed4d with the two reviewed KEYRANDOM
commits. This step provides the probe/observer/service adapter and component
builds; **no executed native RANDOM gate yet**. The integrator supplies the
actual bootstrap, feature, images, cold chain and runtime evidence. All
requirements from [FULL-TEST-RELEASE](../FULL-TEST-RELEASE.md) remain mandatory.

## Boundary and process structure

`native-system-random` builds on `native-system,monotonic-clock`. Init,
volume and the existing v7 document/cryptography trial retain their behaviour;
Key receives the separate RND ingress and the app only the opt-in STATUS
barrier described below. A fifth initial process runs the RANDOM trial.
Its actual PID is stored; slot 4 may be reused by a worker after exit.
The observer reads probe data before cleanup and retains a snapshot.

The probe receives exactly two handles: RANDOM SEND and its own reply RECV.
No rawEntropySource, key ingress, recovery/snapshot, storage, network,
process management, DERIVE, TRANSFER or REVOKE. The existing Key Service owns
two key ingresses; RANDOM becomes its third of at most four endpoints. The
probe owns one reply endpoint. No endpoint quotas are increased.

Bootstrap retains a RANDOM root at Key and a SEND|TRANSFER|REVOKE ancestor
derived from it. Only a SEND leaf goes to the probe. Key RECV is a separate
branch. Revoking the SEND ancestor does not affect that RECV; the ancestor
itself is revoked. The rebind phase uses the still-valid ancestor before
revocation. The reply root is closed after deriving the unique probe RECV
and Key SEND. There is no hidden RECV alias.

The fixture grants only explicit test rebind/revoke within this scope. It
is not a product API or implementation of account/NAS onboarding. Kernel and
capability engine receive no cryptography, RNG or protocol parser.

## Four cases

| Case | Actual operations and acceptance evidence |
|---|---|
| 0 RAW | Native RND1 IPC for 1/32/64/128/224 bytes; all 9 old opcodes and 9 invalid request fields rejected; 64 calls within an actual 100-tick window and a quota reply; waiting for actual ticks; both call and 4096-byte reseed; native close→new SEND generation without principal reset; old ID rejected; closed unique reply RECV followed by an actual request; Key continues working, pending is cleared/ingress retired; then SEND-subtree revocation and an actual Revoked error. |
| 1 SDK/cancel | The actual SDK/NativeTransport requests 1/32/64/128 bytes; an actually queued 32-byte request is then cancelled. Both handles close; later begin/poll publish nothing. The probe remains alive until the service has actually processed the request. |
| 2 SDK/uncertain | The test transport adapter executes actual native.send and deliberately returns InvalidReturn only after success. This is explicit caller fault injection. The SDK must report Unknown, close both handles, publish no bytes and not resend. No replacement of an OS service with a host result. |
| 3 Wrong PID | The RND descriptor expects the existing app PID; the probe actually sends as its own PID. Seen=1/badCaller=1 and calls/bytes/seeds=0 prove actual consumption+rejection. The caller then confirms at least 10 actual ticks without a reply; silence alone is insufficient. |

For cancel/uncertain, the reply may already have been queued before
cancellation or may fail only at Closed. Revocation cannot undo bytes already
legitimately produced. The SDK does not release the unanswered output to the
caller. Case 0 deterministically causes a closed-reply error without that
race. The existing system app must in every case continue its
2 seals/2 unseals/9 authentication rejections and the 4096-byte plaintext
disk fixture. This disk fixture does not prove encrypted storage.
After the terminal RND observation, the existing app additionally executes
a **new actual legacy STATUS RPC** using its existing sole live KeySession.
The received state must be Ready. No additional KEY permissions are granted
to the RANDOM probe and STATUS must consume no entropy.

Individual zero bytes and repeated short values are valid random output.
Only for samples of at least 32 bytes does the trial require an additional
not-all-zero/difference check. That is not an entropy estimate, statistical
security test or physical RNG quality claim. The implementation must use
the actual virtio-rng and maintained ChaCha20Rng; the probe has no host/CPU
RNG fallback.

## Startup, hooks and counters

The probe uses the existing private 1024-byte startup page:
32=version1, 40=result, 48=RND-SEND, 56=reply-RECV, 112=case0..3,
160=KeyPID, 176=ownPID. Result1 plus native exit0 means the probe passed;
240 identifies the step on failure. Measurements are in 400..584.
Phase 640=request, 648=ack, 656=newSEND; these cells are solely the opt-in
test fixture. Their definitions are in
[`contract.rs`](../../sdk/examples/random-probe/src/contract.rs).

Key retains the previously established descriptor:
120=`NOSRNG01`, 128=RND-RECV, 136=reply-SEND, 144=callerPID, 152=domain0.
Old profiles without the descriptor continue to work without RND Sys3 calls.
Additional Key observation words:

| Word | Meaning |
|---|---|
| 272/280/288 | actual core calls/bytes/seeds |
| 296/304/312 | native ingress messages seen / wrong caller / wrong opcode |
| 320/328/336 | RND ingress retired / reply pending / malformed request |
| 344/352/360 | actually queued replies / Closed errors / pending deadline/budget errors |
| 368/376 | successful legacy receive/poll returns (Ok/EAGAIN) / unexpected legacy poll errors |

Request counters increase only after actual native.receive. Pending is
published before its associated output counters. Observer phases wait for
a complete terminal combination; individual userland words are not atomic
with respect to timer pre-emption.
After the terminal RND observation, the fixture also waits for at least two
new actual legacy poll returns and zero errors. The native trace thereby
demonstrates progress after Closed/cancel/uncertain/wrong caller; a heartbeat
in an RND retry loop does not count. A new successful legacy RPC round trip
after that error is separately performed by the existing app; its earlier
2/2/9 or this polling counter alone is no substitute for that evidence.

Only this profile populates the existing app's 624=`NOSRNGT1`, 632=release,
640=STATUS-ACK, 648=start ticks and 656=end ticks. Magic0 keeps all old cases
without an additional RPC/Sys3 path. The fixture sets release only after
actually consumed/processed RND input or an actual wrong-caller rejection.
The app waits boundedly with Sys3, uses begin/poll on its existing session
for STATUS with an actual tick deadline, and closes that session only
afterwards. The phase ACK to the probe follows only after the app's actual
STATUS ACK; the old app may already have exited by then, after which the
observer uses its snapshot. Document storage, key revoke, 2/2/9 and disk
request IDs remain the same. The observer retains this evidence before app
cleanup too and requires
`terminalTick <= STATUSbegin <= STATUSeinde <= appExitTick` and ACK1
(STATUSeinde denotes the STATUS end tick).

Bootstrap calls `random_fixture::bind(s,e,Binding{case,key_pid,probe_pid,
random_root,send_ancestor,send_leaf,reply_receive})` after registration/grants/
startup. Then required: `poll(s)`, `is_probe(s,slot)`,
`observe_exit(s,slot)->bool` before cleanup on **all** exits: app status is
stored with return false, probe status with return true. Use `finished()`,
which requires both snapshots, as an additional finish condition, and
`check_and_print(s)` before Key cleanup. Root retains module/ELF selection,
five initial processes, feature conflicts, timeout diagnostics, build
manifests, shared locks and run/capture validation in one integration patch.

RAW expects app 89 successful replies/4737 bytes, 110 enqueues/109 validated
replies; variables5, legacy9, invalid9, quota1, rebind/stale/replay/revoke
1 each, capability rejections6. The Key vector is exactly
`[90,4769,3,110,0,9,1,0,9,109,1,0]` according to the twelve service words.
SDK cancel expects 5 core calls/257 bytes/1 seed, 4 positive outputs/225 bytes,
5 native enqueues, 2 closes and one cancellation without retry. SDK uncertain
expects 1 call/64 bytes/1 seed, 1 enqueue, 0 output and 2 closes without retry.
WrongPID expects exactly `[0,0,0,1,1,0,0,0,0,0,0,0]`.
`RANDOM_PROBE_PID`, `RANDOM_KEY_PID`, `RANDOM_APP_PID` and
`RANDOM_EXPECTED_CALLER_PID` bind the measurements to stored actual process
identities; only WrongPID expects the app as caller. All cases compare actual
broker bytes with 1024+256×seeds and Key264; the original cryptography,
storage, lifecycle and cleanup gates remain required.

## Bounded RANDOM replies

The old shared send helper could perform 200,000 EAGAIN yields before legacy
received another turn. Only the RANDOM reply route has therefore changed:
one non-blocking send attempt per main loop, at most one managed 256-byte
pending reply, 4096 attempts and a verified 100-tick deadline after EAGAIN.
Each main loop first handles the existing key ingresses. Closed, other fatal
transport errors and time/budget errors terminate only RND ingress and erase
the pending/output envelope; master, Ready and legacy RPCs are not reset.
A retired ingress is not silently granted again.

The RAW Closed trial and SDK cancel cases must execute this natively. An
actually full reply queue with EAGAIN and its deadline path additionally
remain to be checked natively; source/host checks do not satisfy that runtime
requirement. Missing/regressing time and source failure also remain separate
negative runtime requirements; this profile does not deliberately break
actual monotonic PIT time to simulate a result for them.

## Component build and evidence boundary

`LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=<eigenmap>
python3 -B tools/dev.py python3 -B sdk/examples/random-probe/build.py`
builds `random-probe` with Rust 1.90 for x86_64-unknown-none (`<eigenmap>`
denotes a dedicated directory). The kernel receives
`NOS_RANDOM_PROBE_ELF=<gecontroleerd ELF>` (a checked ELF). Argument `clippy`
runs the same native selection with `-D warnings`. The probe has its own
lockfile; no root/SDK lockfile was changed for this component build.

Host checks compare all 1..224 RAW codec lengths with the actual Key codec,
reject changed reply fields/capabilities and use the real capability engine
for Closed/rebind/subtree revocation. They do not prove native RNG execution.
The component report is in
[`verification/component.json`](../../sdk/examples/random-probe/verification/component.json):
4 new host trials plus 61 existing SDK and 15 Key-client trials pass; probe,
Key service and system app build and satisfy Clippy for the scalar native
target. Disassemblies contain no matches for FP/SIMD registers or CPU RNG
instructions. This is a bounded code-generation check, not execution evidence.
The observer is also type-checked in a private copy of the actual 985 kernel,
with only a temporary module declaration and checked existing 985 ELFs;
bootstrap/hooks are not executed by that check.

For native acceptance, root integration, all four actual cases, the cold
normal/recovery chain, exact image/source hashes and regressions remain
mandatory. This partial step makes no full test-release, TLS, identity,
certificate-time, PQC, entropy-quality or complete memory-erasure claim.
