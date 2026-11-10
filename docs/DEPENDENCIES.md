# Dependencies and provenance

M6b adds only the project's own local crate `object-core 0.1.0`: `no_std`,
no allocator or external dependencies, and `forbid(unsafe_code)` in the core.
The native adapter uses existing fixed syscall buffers and a justified
unsafe boundary. Root Cargo.lock retains `r-efi 7.1.0` with the same checksum.

The SDK runtime now directly uses `zeroize 1.9.1` (exactly pinned,
`default-features = false`) to erase IPC envelopes. This is the same
RustCrypto dependency already used in `services/keyring`; the pinned crate
is `no_std`, has no transitive dependency or build script in this
configuration, and is Apache-2.0 OR MIT. Provenance: crates.io lock checksum
and <https://github.com/RustCrypto/utils/tree/master/zeroize>.
This erases only explicitly managed buffers. Since 2 November 2026, the
native session UI directly uses the same exact pin for owned password/ticket
buffers; no new external crate, feature or build script. This does not prove
cleanup of registers, compiler copies, crash memory or copies outside the
SDK-owned envelopes.

### Network services: smoltcp, rustls and spin (30 October 2026)

These external crates are confined to the independent workspaces of ring-3
network services, each with its own `Cargo.lock`; they are not in the kernel,
loader or root workspace. Provenance and build-script checks are recorded
with the component.

| Crate (exact pin) | Used by | Provenance/licence | no_std/transitive | Detailed source |
|---|---|---|---|---|
| `smoltcp =0.14.0` | `services/net-stack`, `services/net-broker` | smoltcp-rs/smoltcp; 0BSD | `default-features = false`, Ethernet/IPv4/TCP only; transitive `managed`, `heapless`, `hash32`, `byteorder`, `bitflags`, `cfg-if`, `stable_deref_trait` under the lockfile | ADR-046 in `DECISIONS.md`, `services/net-stack/THIRD-PARTY.md` |
| `rustls =0.23.46` | `services/net-tls` | rustls/rustls; Apache-2.0 OR ISC OR MIT | `custom-provider` only, no `std`/ring/AWS-LC; transitive `rustls-webpki`, `rustls-pki-types`, `once_cell`, `subtle`, `untrusted`, `zeroize` | `services/net-tls/README.md` (dependency review) |
| `spin =0.12.3` | `services/net-tls` | zesterer/spin; MIT | `mutex,spin_mutex` only; no dependencies or build script | `services/net-tls/README.md` |

Unselected optional or platform packages (such as `ring`, `windows-*`) may
appear in `services/net-tls/Cargo.lock`; only the feature tree and compiler
output establish what is built. **Update policy:** versions remain exactly
pinned and `--locked`; an update requires its own PR with a lockfile diff,
repeated licence/build-script checks and the component gates.
**Removal plan:** each crate is confined to one service workspace; removal
means replacing that service or adapter without kernel, ABI or root-lock changes.

The new Linux development variant uses Rust 1.90.0, QEMU 10.0.2
(`1:10.0.2+ds-2+deb13u1~bpo12+1`) and matching OVMF 2025.02-8+deb13u1.
Rust came from static.rust-lang.org; Debian packages were verified through
signed InRelease, Packages SHA256 and package SHA256 and extracted locally
only. Metadata, URLs, binary/firmware hashes and host libraries are in
`verification/native-linux-toolchain/`. These are host tools, not new runtime
libraries of the independent kernel or object service.

N02 (27 October 2026) additionally uses only built-in `node:tls`, `node:net`
and the existing `node:crypto` API on Node 24.22.1/OpenSSL 3.5.9. No npm or
Cargo dependency was added. The separate system OpenSSL CLI **3.0.13**
generates temporary Ed25519 test certificates; that CLI does not perform
the hybrid TLS handshake. The handshake occurs in Node's OpenSSL **3.5.9**.
Runtime values, source hashes and test output are under
`verification/pqc-transport/`. No certificate/private key or production CA
in Git. TLS group X25519MLKEM768 is standardised in RFC 10024 (August 2026);
native protocol/library porting remains a separate dependency selection.

Investigated, **not installed/added**: `libcrux-ml-kem` and `libcrux-ml-dsa`
0.0.11, official repository `celabshq/libcrux`, Apache-2.0 and manifest Rust
minimum 1.89.0. Exact no_std features, Rust build scripts/SIMD selection,
transitive licences and the actual target build still need to be recorded
before inclusion. Project Rust version 1.90.0 alone is not compatibility
evidence. Partial formal-verification coverage is described in the library's
own documentation; no blanket constant-time or machine-code claim.
Sources and native budgets: `implementation/VERIFIABLE-SECURITY-AND-PQC.md`.

N01 (27 October 2026) uses the available Node.js 24.22.1 host runtime with
OpenSSL 3.5.9 and only built-in modules (`node:crypto`, filesystem and
test/process APIs). No npm package, Cargo dependency, external crypto
implementation or build script was added. Node/OpenSSL are development/
experimental tools, not a selected native Baryon OS runtime. AES-GCM,
randomness and authenticated-decryption APIs were checked against primary
Node v24 documentation; concrete versions/test evidence are in
verification/nas/report.json. Native crypto/TLS library selection, no_std
suitability, licence/supply-chain checks and key lifecycle remain separate
gates before that integration.

Execution status on 27 October 2026: Rust 1.90.0, r-efi 7.1.0, serde 1.0.229
and serde_json 1.0.154 were actually fetched and built. Root and quantum
lockfiles were generated. The emulator used was the permitted Windows
development variant QEMU 11.1.0 with its matching packaged CODE/VARS pair;
exact provenance/hashes and evidence limits are in DEVELOPMENT.md,
third-party/README.md and ../experiments/quantum/THIRD-PARTY.md. The original
selection table below is historical context; PROJECT-STATE.md describes
current verification.

M2–M4 add only local project crates: kernel-core 0.1.0 (frames/ELF/copy policy)
and capability-core 0.1.0 (fixed permission/IPC transactions). Both are
dependency-free/no_std and forbid unsafe in the library. The kernel
architecture/copy boundary contains justified unsafe/assembly; the project's
own crates have no external build script. Root Cargo.lock retains r-efi
exactly 7.1.0 with the same checksum. No new external runtime library was
added for M2–M4. Core-source/host/target checks and current test-case limits
are in verification/m2-m4/.
M5 adds only local project crates `virtio-core 0.1.0`, `virtio-blk-driver 0.1.0`
and `block-protocol 0.1.0`. Both cores are dependency-free/no_std and forbid
unsafe; the driver depends only on virtio-core. The driver contains justified
PCI/MMIO/DMA unsafe, with no external build script. No external package was
added or changed in Cargo.lock. Protocol basis: OASIS VIRTIO 1.2; the code
is project-specific and contains no copied existing OS kernel. MPL-2.0 applies to
project-owned software within the scope defined in `../LICENSING.md` and
`../MPL-2.0-FILES.txt`; external components retain their own terms. M5 target/review evidence is in verification/m5/; the full gate passed.

Historical selection, revision 5 of 26 October 2026: the following table
describes the original preparation. Current builds and pins are above and
in PROJECT-STATE.md. Do not add fictitious lockfiles or firmware hashes.

| Selection | Role | Status |
|---|---|---|
| Rust 1.90.0 with bundled LLVM linker | Host compiler; UEFI and no_std kernel target | Recorded baseline, no claim to be the latest version; pin rust-toolchain.toml during B01 |
| r-efi =7.1.0, default-features=false | Small no_std UEFI bindings, not a loader/kernel | Official manifest checked; fetch, compatibility and transitive tree still to be tested |
| QEMU 10.0.14, pc-q35-10.0, qemu64, TCG | Reference emulator on host | Version investigated; practical compatible package route permitted and logged under boot specification |
| OVMF from edk2-stable202508, X64 RELEASE/GCC5 | Reference UEFI firmware | CODE/VARS must come from one matching build; record hashes of actual files |
| Python 3 | Inventory/build/test harness | tools/preflight.py present and executed; no external Python packages needed |
| HTML/CSS/JS | Local P2 product prototype | No framework, CDN, external assets or runtime network |
| qsim-core | Independent safe simulator core | Design: no_std+alloc, zero external dependencies |
| serde =1.0.229; serde_json =1.0.154 | JSON adapter for host quantum experiment only | Documentation pins; registry resolution still to be performed. Concrete recovery policy in QUANTUM-SPEC.md |
| Crypto library | Later ML-KEM-768/ML-DSA-65 integration | Parameters selected; choose library and concrete protocol binding only during the integration task |

For every dependency actually incorporated, record name, version/commit/hash,
primary source, licence, direct/transitive dependencies, build scripts,
maintenance/security reporting, host/guest/firmware role, no_std support,
rationale, update policy and removal plan. Verify actual resolver output
and record Cargo.lock; a documentation pin is not a successful build.

Do not adopt existing kernel code without an explicit scope change. Small
adopted algorithms/examples remain external source code: preserve provenance
and licensing. An AI rewrite does not remove those obligations.

Toolchain, firmware and build environment belong to the trust chain.
Reproducible builds demonstrate agreement between builds; they do not prove
that source code, compiler or hardware is free of backdoors.
