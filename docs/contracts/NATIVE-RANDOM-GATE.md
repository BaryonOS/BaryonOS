# Native RANDOM integration gate

This gate checks the custom ring3 RND1 client, isolated Key Service,
actual virtio-rng and maintained ChaCha20Rng together with the existing
init/document/volume/cryptographic chain. It is a subcriterion of
[FULL-TEST-RELEASE](../FULL-TEST-RELEASE.md), not a declaration of completion.
The native contract is in
[NATIVE-RANDOM-PROBE](../implementation/NATIVE-RANDOM-PROBE.md).

## Profiles and operation

| `--case` | Kernel profile | Expected native case |
|---|---|---|
| `raw` | `native-system-random` | 0: variable output, validation, quota/reseed, rebind, Closed, revoke |
| `sdk` | `test-system-random-sdk` | 1: actual SDK output and cancellation after actual enqueue |
| `unknown` | `test-system-random-unknown` | 2: explicit uncertain caller return after actual enqueue |
| `wrong-caller` | `test-system-random-wrong-caller` | 3: actual ingress consumption and PID rejection without RNG output |

Build each profile on the same frozen source with the normal builder. For example:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B tools/dev.py python3 -B tools/build.py --kernel-feature native-system-random
LOCALAPPDATA=/workspace/nieuw-os/.tools python3 -B tools/dev.py python3 -B tools/run_random.py --case raw
```

Use dedicated `NOS_BUILD_DIR` and `NOS_TARGET_DIR` paths within the checkout,
the explicitly selected QEMU machine profile and its specified OVMF files.
The gate accepts only TCG, one qemu64 CPU, 256MiB, no network, no host shares
or snapshots, one owned disposable disk and virtio-rng. The QEMU entropy
backend is the isolated virtual counterpart; the probe cannot call a host
RNG or CPU RNG and receives no EntropySource capability.

Every case starts a new normal write test on an empty 1MiB image under
`BUILD/run`. There is no `--volume` for selecting existing or personal content.
The maximum timeout is 180 seconds. No fifth readonly RNG profile is assumed:
the existing FR-I01 normal→cold readonly recovery chain remains separately
required regression evidence. This gate does not prove persistent keys or
random state after a cold restart.

## Sign-off

`run_random` reuses `run_system` and therefore actual native execution,
raw QEMU exit33, the original 2 seals/2 unseals/9 authentication rejections,
record tests, process lifecycle/cleanup and independent 4096-byte disk oracle.
All RANDOM data is then reread from the actually saved serial output.
The explicit order, fixed 20-digit unsigned u64 counters and exactly one
RANDOM-PASS are mandatory. Unknown, duplicate, missing, incorrectly ordered
or overflowing RANDOM fields fail.

Individual test figures are in the implementation specification. The gate
also requires actual, distinct probe/Key/app PIDs and the corresponding
expected caller. Wrong-caller requires ingressSeen1, badCaller1, zero core
calls/bytes/seeds/replies and at least 10 ticks without a reply after the
new STATUS round trip. Silence alone, a timeout or a success flag is not
rejection. Cancel and uncertain require their respective correct delivery,
closure, output and no-retry figures; replies already enqueued are not
represented as undone output on revocation.

After the observed terminal RNG operation, the ordinary app must complete
a **new actual STATUS RPC** using its existing sole KEY session and receive
Ready. The phase ACK to the probe follows. The gate checks:

`probeStart <= RNGterminal <= STATUSbegin <= STATUSeinde <= probeEinde`

An old Ready status, heartbeat or only the original cryptographic test does
not replace this evidence. At least two actual legacy-poll returns after
RNG handling are additionally required. The actual broker must supply exactly
`1024 + 256 × seeds` entropy bytes; STATUS requests no extra entropy.

## Source, image and file binding

Before `random_gate=True` is possible, the gate checks:

- Source-fingerprint version 3 and the currently selected manifest feature;
  generic normal, net or TCP manifests are invalid.
- Exactly six inspected system ELFs in `manifest.system`, including
  `random_probe`, with the same additional `manifest.random_probe` entry.
  Hashes and fresh ELF inspection must match the actually built files;
  all complete ELF bytes must be present in the loaded kernel.
- The exact kernel/loader in both the build ESP and actual QEMU ESP,
  without unknown files, symlinks, hard-linked publications or special files.
  Only read-only consumption of actual Cargo output permits Cargo's internal
  hard links.
- The exact QEMU command profile, selected executable hash, OVMF code hash
  and preboot VARS template hash. Writable UEFI variables are read as current
  managed bytes, not incorrectly equated with their old content.
- An actually empty new image before the test and exactly the same resulting
  bytes in the live disposable image and separate post-run snapshot. The
  independent on-disk parser must confirm the original document with 4096
  exact fixture bytes, revision1, name `systeemproef` and actual SAVE requestID3.
- Equality of the saved oracle/counters with reread bytes, and a final check
  that source, firmware and the QEMU executable remain unchanged.

The result retains `case=normal`, receives `random_case` and an initially
false `random_gate`. Every additional validation error also sets `passed=False`.
The current `random-report.json` is atomically published as `status=starting`,
`passed=False`, `random_gate=False` before manifest checking and delegation.
A preflight or launch exception records `status=failed` and false flags and
is propagated to the caller. A previously passing current index report
therefore does not remain after a new failed attempt; historical per-run
results are retained. An unsafely linked report target is rejected before
publication to protect external content. The source ESP is also checked
for exactly the two hashed files and all components/links before every
shared copy/VM delegation; checking the copied ESP after execution remains
additionally mandatory. Results are saved atomically under the dedicated
build/run paths as `random-report.json`, `result.json` and
`last-system-stage.json`. The latter cannot authorise normal FR-I01 cold
recovery because its feature differs. The JSON object is an evidence record,
not signed attestation against a malicious host that falsifies source,
binaries and all evidence files.

The builder hook is root integration: build the probe with
`CARGO_TARGET_DIR=TARGET/random-probe-app`, supply `NOS_RANDOM_PROBE_ELF` and
add the inspected `random_probe` to the existing system manifest.
`run_system` accepts these four features only as normal; all old profiles
and their own thresholds are retained.

## Host verification and outstanding evidence

```sh
python3 -B -m unittest discover -s tools/tests -p test_random.py
```

The host tests use explicitly synthetic logs/ELFs/disposable images to
demonstrate counterexamples: timeout versus rejection, mixed PIDs, old STATUS,
incorrect profile/source claims, missing/duplicate figures, excess output
or retry after cancellation, changed ELF/ESP/serial/disk, snapshot/network
commands and external symlink/hardlink targets. These model files are not
a VM log or evidence that random output worked. No PASS artifact is filled
in for an unexecuted VM.

Four actual VM cases on the final frozen integration source and existing
cold system regressions remain necessary. A full reply queue with
EAGAIN/deadline, a defective clock/entropy source, actual account/broker
authorisation, TLS identity/certificate time/PQC and physical entropy quality
have their own open acceptance criteria. This gate does not sign them off
and claims neither complete memory erasure nor a complete test release.
