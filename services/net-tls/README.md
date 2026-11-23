# Native TLS port: preconditions, not yet a TLS service

Task `TL-NET-TLS-PORT-1`, base `94c8908` plus the unchanged CORE time port
`9afabbd` (cherry-pick `00608c4`). Write area exclusively
`services/net-tls/**`; the CORE files are an existing agreed
dependency, not a second implementation.

Goal: compile the maintained rustls core without `std` and connect its existing
random/time interfaces to the existing SDK boundaries. No self-written
crypto, provider, record protocol, TLS handshake, native ELF, manifest grant,
certificate root, signer or NAS principal in this step.

## Invariants and interfaces

- `KeyServiceRandom<T, C>` implements rustls `SecureRandom` with an exclusively
  owned existing `keyring-random-client::Session<T>`. Native `T` is the existing
  `NativeTransport`; `NativeClock` uses the real monotonic SDK clock. The
  process/account/generation binding must already have been established by trusted bootstrap.
  This adapter grants no rights.
- Rustls requires `fill(&self)` and `Send + Sync`; a maintained `spin::Mutex`
  protects the mutable SDK session. Only `try_lock`, no spin loop or new
  `unsafe`. Busy/reentrancy fails immediately without touching the ongoing operation.
- A request delivers at most the existing 128 bytes, completely or nothing.
  No chunking, cache, getrandom/RDRAND, republication or automatic regrant.
  Requesting zero bytes publishes nothing. Invalid length/busy wipes the output
  without cancelling another ongoing operation.
- An absolute monotonic deadline applies to the entire request, also after receipt
  of a complete answer. Rollback, overflow, clock error, SDK poll budget,
  protocol error and service refusal are errors. The adapter then cancels the
  lease; an `Unknown` publication is not repeated either. The output is
  zeroized in advance, and copied only after all checks. The SDK wipes its current
  received answer buffer; this is no claim about historical compiler copies.
- `try_fill` retains the SDK error, delivery state and cleanup error. Rustls'
  prescribed `fill` translates this to `GetRandomFailed`, not a success fallback.
  `Debug` contains no bytes, handles or key content.
- `CertificateClock` and `checked_time` use exclusively the CORE port.
  `checked_now` keeps source errors separate from `TimeUnavailable`.
  Rustls `TimeProvider` gets `None` on unavailability, never RTC, monotonic
  ticks, host time or build time as a substitute. A future TLS consumer must
  already check `checked_now` for certificate validation. Every current
  clock reading fails; there is no fictitious trusted provider.

**Important:** rustls `SecureRandom` does not automatically cover the RNG of the
crypto backend, KEM or signer. A future provider must also bind those routes to the
real authorized Key-Service. This adapter alone does not prove that.

## Dependency check before addition

Directly pinned: rustls `=0.23.46`, spin `=0.12.3`, existing zeroize `=1.9.1`;
transitive versions/checksums are in the standalone `Cargo.lock`.

| Library | Origin / licence | Selected / build script |
|---|---|---|
| rustls 0.23.46 | rustls/rustls; Apache-2.0 OR ISC OR MIT, maintained release October 2026 | Only `custom-provider`; no `std`, TLS 1.2, logging, AWS-LC or ring. `build.rs` has an empty main without `read_buf`; source read in advance. |
| spin 0.12.3 | codeberg.org/zesterer/spin; MIT, release August 2026 | Only `mutex,spin_mutex`; `no_std`, no dependencies or build script in this selection. |
| rustls-webpki 0.103.16 | rustls/webpki; ISC | `alloc`, no crypto backend/build script. Manifest of the actually resolved patch checked, not only that of 0.103.14. |
| rustls-pki-types 1.15.1 | rustls/pki-types; MIT OR Apache-2.0 | `alloc`, no build script. |
| once_cell 1.21.4 | matklad/once_cell; MIT OR Apache-2.0 | `alloc,race`, no `std` or build script. |
| subtle 2.6.1 | dalek-cryptography/subtle; BSD-3-Clause | Without default features, no build script. |
| untrusted 0.9.0 | briansmith/untrusted; ISC | No `std` or build script. |
| zeroize 1.9.1 | RustCrypto/utils; Apache-2.0 OR MIT | Existing SDK pin, `alloc` via rustls; no build script. |

Primary sources: crates.io version metadata and the published Cargo manifests
at `https://docs.rs/crate/<naam>/<versie>/source/Cargo.toml`.
Rustls interfaces: `https://docs.rs/rustls/0.23.46/rustls/crypto/trait.SecureRandom.html`
and `https://docs.rs/rustls/0.23.46/rustls/time_provider/trait.TimeProvider.html`.
The registry archives were fetched via Cargo, not by running external scripts.
Cargo can include/fetch unselected optional/other-platform packages in the
lockfile; only the explicit feature tree and compiler output
prove what is actually built in this selection.

Rustls' unbuffered API supports `no_std` but still uses `alloc`.
A real native ELF requires a bounded private allocator and OOM probe;
a library build proves no heap, stack or runtime behaviour.

Not chosen as a covert product fallback: AWS-LC (no demonstrated bare-metal
RNG/platform port), rustls-rustcrypto 0.0.2-alpha (old experimental provider),
rustls-graviola 0.4.0 (forces `std`/TLS 1.2; graviola requires among other things
AVX/AVX2, not the current VM profile). No CPU profile change made.
The agreed probe profile remains TLS 1.3, X25519MLKEM768,
TLS_AES_256_GCM_SHA384 and fixed ALPN, without classical fallback/session resumption/
0-RTT mutations. No suitable native provider in this delivery.

## N03-T2 host loopback oracle (host evidence only)

Project-owner decision, 23 November 2026: "Host oracle first — allow aws-lc-rs
as a host-only test dependency for the N03-T2 loopback handshake; the native
provider choice stays open until the decision memo is reviewed."

`tests/loopback.rs` runs a rustls client and server over an in-memory record
pump. The profile side restricts the aws-lc-rs provider to TLS 1.3,
X25519MLKEM768 and TLS13_AES_256_GCM_SHA384, uses the fixed test ALPN
`baryon-n03-host-loopback/1`, disables resumption, tickets, 0-RTT and half-RTT
data, and pins the peer's exact end-entity DER and SPKI with no intermediates.
TLS 1.3 handshake signatures are still verified through the provider; TLS 1.2
signatures are refused. Application data is written only after an acceptance
gate re-checks version, group, suite, ALPN and peer pin on both sides.

Project code exercised: `KeyServiceRandom` (over a synthetic Key-Service
transport) is the rustls `SecureRandom` of every profile side, and
`CertificateClock` is the rustls `TimeProvider` in the unavailable-time negative.
The profile and pin verifier are test code because `src/` provides neither yet.

Negative cases, each with an asserted specific error: X25519-only server and
client (no fallback), other TLS 1.3 suite, TLS 1.2-only server and client, wrong
server pin, wrong client pin, missing client certificate, wrong client ALPN,
Key-Service RNG denial (`FailedToGetRandomBytes`, exactly one request, no retry)
and unavailable certificate time (`FailedToGetCurrentTime`). In every handshake
negative the client queues application data before the handshake; the server
gate refuses and neither side releases any plaintext. A positive control shows
the same queued bytes do arrive after a completed handshake, so the empty reads
are not vacuous.

Observed design findings for the native port: (1) in TLS 1.3 the client
completes, and its own gate passes, before the server rejects a wrong or missing
client certificate; only the later fatal alert ends it, so a consumer must treat
a received alert as revoking the connection. (2) rustls completes a handshake
with a server that selects no ALPN; only the gate refuses it, and data queued
before the gate already reached that server. A profile consumer must therefore
run the gate before its first write or read.

Harness assumptions, not product properties: TEST-ONLY Ed25519 keys and
self-signed certificates are generated at runtime by rcgen and never stored; no
trust root is created. Certificate validity periods are not checked, and positive
runs use a fixed test `TimeProvider`, because no trusted certificate-time
authority exists. aws-lc generates the X25519MLKEM768 and signing randomness
itself; that route does **not** pass through `KeyServiceRandom`.

Host-only dev-dependencies (resolver 2 keeps them out of the library and
`x86_64-unknown-none` graphs; checked with `cargo tree -e normal,build` and the
freestanding build, whose crate list contains no aws-lc/rcgen):

| Crate | Version | Licence (manifest) | Build script / note |
|---|---|---|---|
| rustls (dev features `aws_lc_rs,std,tls12`) | 0.23.46 | Apache-2.0 OR ISC OR MIT | Existing pin; normal graph stays `custom-provider` only |
| rustls-webpki (dev feature `alloc`) | 0.103.16 | ISC | Existing transitive pin; used for SPKI extraction |
| rcgen (`aws_lc_rs`, no default features) | 0.14.10 | MIT OR Apache-2.0 | `build = false`; no `ring` selected |
| aws-lc-rs | 1.19.0 | ISC AND (Apache-2.0 OR ISC) | `build.rs`; github.com/aws/aws-lc-rs |
| aws-lc-sys | 0.47.0 | ISC AND (Apache-2.0 OR ISC) AND Apache-2.0 AND MIT AND BSD-3-Clause AND (Apache-2.0 OR ISC OR MIT) AND (Apache-2.0 OR ISC OR MIT-0) | `builder/main.rs` compiles bundled AWS-LC C/assembly; local build script reported `Building with: CC` (cmake unused); needs a host C compiler |
| cc, cmake, jobserver, shlex, find-msvc-tools, dunce, fs_extra, pkg-config, libc | lockfile | MIT OR Apache-2.0 family (dunce: CC0-1.0 OR MIT-0 OR Apache-2.0; fs_extra: MIT) | aws-lc-sys build helpers |
| time, time-core, deranged, num-conv, powerfmt, yasna | lockfile | MIT OR Apache-2.0 | rcgen certificate encoding |

The lockfile also lists optional packages (for example x509-parser, ring
helpers) that this feature selection does not build. Exact versions and
checksums are in `Cargo.lock`. Not reviewed: a complete source audit of AWS-LC,
advisories at release time, or a distribution notice set; this oracle is never
linked into a Baryon OS image.

## Acceptance and evidence boundary

For this step: targeted host tests, Clippy, formatting, `x86_64-unknown-none`
library build and inspection of the actually selected dependency features.
Executed commands (exact results and source binding below):

```sh
python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path services/net-tls/Cargo.toml
python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked --manifest-path services/net-tls/Cargo.toml --all-targets -- -D warnings
python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path services/net-tls/Cargo.toml -- --check
python3 -B tools/dev.py cargo +1.90.0 build --offline --locked --manifest-path services/net-tls/Cargo.toml --lib --release --target x86_64-unknown-none
python3 -B tools/dev.py cargo +1.90.0 tree --offline --locked --manifest-path services/net-tls/Cargo.toml --target x86_64-unknown-none -e features
```

Tests use synthetic SDK messages and model clocks, no real entropy
or TLS counterparty. Native TLS/Key-Service request from a TLS process,
certificate validation, handshake, NAS peer binding and native normal/cold: **not
executed and not present**. No signing service or trusted calendar source
authorized by SEC; REL confirms that no TLS process/bootstrap grant
exists. Those boundaries remain open; FR-N03/N03 gets no points.
Proposal to central manager: network **0/10 -> 0/10**, total **34 -> 34**.

Rollback: do not integrate/remove this standalone leaf; the other
network services have no dependency on it. Central status/score files
remain manager-owned.

## Execution 30 October 2026

Pushed source `7af60df90468e47e2c551022b799bed37ea55b96`;
native source fingerprint v3 of that run
`e659fbc8117587fccbccfc641e6e3730cf0980a2f665607fea1480a5a1711347`.
This later documentation/evidence commit changes the total source fingerprint:
the service README also counts towards it. Old native results are not relabelled
to that new head; production/test code and lockfile remain equal to
`7af60df`. The targeted host/format/Clippy checks were repeated afterwards.
Changed files: exclusively this new leaf (manifest/lockfile,
random/time adapters, tests, documentation and evidence); the earlier CORE cherry-pick
remains separately recognizable. No kernel/manifest/build tool change.

| Check | Result |
|---|---|
| First host test on checkpoint `c291e6e` | Exit 101: two fixture closures used an untyped `1 << 32`. Kept as `tests-c291-red.txt`; no red result concealed. |
| Correction `7af60df` | Only the two fixture closures explicitly `1u64 << 32`; no production behaviour changed. |
| Targeted host tests on corrected source | Exit 0, 13/13: 1 contention test, 9 RANDOM/deadline/protocol tests, 3 time refusals. No native entropy or handshake evidence. |
| Clippy `--all-targets -- -D warnings`, formatting | Exit 0. |
| Release library build `x86_64-unknown-none` on `7af60df` | Exit 0; real rustls core and native SDK callbacks compile. No ELF/link/allocator/runtime evidence. |
| Dependency feature tree `--target x86_64-unknown-none` | Exit 0, no std/crypto backend/host RNG in the selected tree. |
| `python3 -B -u tools/test_tcp_system.py` | **Exit 1**, 14.81 s; all five host commands exit 0. Normal raw 0 after KS-01/KS-02, before KS-03/ring 3; cold not reached. No current TCP acceptance. |
| Diagnostic normal repeat | **Exit 1**, 2.66 s total; same kernel and source, extra QEMU ROM search path and exception trace. Still KS-02 followed by triple fault; not an acceptance run. |

The TCP run uses the official gate unchanged, on this newer base
instead of the historical `0ade8014`. Exact kernel hash:
`10619b27ced3a57096817b52af3f11a0283029eebdee2446eda58fbc81659999`.
Loader hash `0bfa2cbcccd921146b6de36cd4773e998dc85b0b1f718b5ca7bea5b4014ced59`.
QEMU 10.2.1, pc-q35-10.0, qemu64, TCG, 1 CPU/256 MiB;
firmware code and vars hashes are in the raw reports.
This small regression profile is not the full VM FR profile.
The disk remains byte-identical before/after the failed normal run.

Diagnosis, not a causal repair: QEMU records a not-present
instruction-fetch page fault `v=0e/e=0010`, `RIP=CR2=0x200130c`, directly after
`mov %rdi,%cr3` at `0x2001309` in `m2::arch::activate`; CR3 is `0x1000`.
Then double/triple fault, with the IDT fetch at `0x22f6118`.
Objdump is of exactly the kernel named above. Why the new page tables
do not translate the fetch is **not established**. No mapping right removed,
NX disabled, page fault ignored or kernel code changed.

The diagnostic wrapper adds exclusively these options to the same
QEMU binary: `-L /tmp/nieuw-os-qemu-102/root/usr/share/qemu`,
`-d int,cpu_reset,guest_errors`, `-D /tmp/nieuw-os-tcp-main94-qemu-int.log`.
With that the `kvmvapic.bin` warning disappears but the boot error remains.
This does not justify a general causality claim about that ROM.
The wrapper, raw trace and both separate run reports have been kept.

Evidence is in `verification/`, with SHA-256 over the actually stored
bytes. Serial firmware CRLF remains unchanged through local Git attributes.
Full original build/test logs are also under
`/tmp/nieuw-os-tls-port-*.log` and `/tmp/nieuw-os-tcp-main94-*.log`.
The gate image files remain exclusively local under
`build/tcp-main94-regression/`, not in Git.

Integration boundary: SEC reports an additional `Received::Drop` change in #72;
SEC exclusively owns the coupled SDK RANDOM client/probe consumer fix.
These results apply to the base named here **without that
future Drop integration**; no green result for the new combination.
REL/manager must resolve the kernel boot and TLS principal/signer/time boundary
separately. No FR-N01--N04 sign-off; network 0/10 and total 34/100 remain unchanged.
