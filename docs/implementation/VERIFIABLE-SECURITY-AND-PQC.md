# Verifiable security and post-quantum integration

**Decision date:** 27 October 2026. **Baseline:**
`ece94a5cfebecaec763eb2855aa9f84e55dcdcbc` (N01, PR #7). This
document incorporates the supplied microkernel, CHERI, and PQC blueprint after
checking the actual repository and primary sources. Execution and test status
are recorded in PROJECT-STATE.md; a design requirement is not passing evidence.

## 1. Product objective and evidence boundary

Baryon OS retains a small custom Rust kernel, explicit authority, and isolated
services. Formal verification is a separate development track. seL4 provides
examples of bounded proofs; it is neither an adopted kernel nor evidence for
Baryon OS code. CHERI is a separate future platform experiment. The current
x86_64/QEMU profile has no CHERI protection.

The objective remains a modern, connected consumer experience. A device is
paired and a permission is selected once; normal repetition within those
boundaries proceeds automatically. A new identity or permission requires
renewed consent. Formal rights checks do not require a prompt for every
synchronisation or network transition. Privacy contract revision 4 remains
authoritative: personal data is local or end-to-end encrypted on an explicitly
paired personal NAS, a friend's or family member's NAS, or optionally stored as
opaque client-side ciphertext by an explicitly selected external provider.
Content keys remain with the user. A NAS administrator manages space and
quotas and may block a guest and delete that guest's NAS copies without
consent, but receives no content key. An external provider receives no content
key or document plaintext and is not an automatic fallback.

A formal proof has the form: under assumptions A, implementation I satisfies
specification S for property P. A, I, S, and P must be recorded. An error in
the intended specification, an unverified system service, the update chain,
protocol integration, or rights configuration is also a software problem.
Residual risk is therefore not limited to physical chips and social
engineering. Testing, model checking, source-code verification, and binary
refinement are distinct evidence levels.

## 1.1 Security Floor when less privacy is chosen voluntarily

Privacy and system authority are orthogonal axes. A user may deliberately
permit internet access, external storage, microphone access, cloud
synchronisation, or data sharing. That decision expands only the explicitly
granted capability or data flow.

A privacy choice must not automatically disable or weaken:

- process/address-space isolation and W^X/NX;
- capability lookup, rights attenuation, generation, and revocation;
- separation between application, object, key, synchronisation, and network
  services;
- package/update authenticity and rollback policy;
- protection of unshared data;
- DMA/IOMMU confinement once it exists for the supported hardware profile.

An application with both plaintext authority and internet authority can
deliberately transmit authorised plaintext. This is a data-flow risk, not a
reason to grant that application kernel authority or broad filesystem
authority. The UI must make the combination visible, while the security model
continues to enforce least authority.

Privacy guidance is not an enforcement layer either. Educational privacy
guidance may be disabled; doing so does not change the capability set.

## 1.2 External storage is an untrusted ciphertext backend

Under privacy contract revision 4, an explicitly selected general-purpose
cloud-storage provider may store opaque encrypted objects. The provider is not
a trusted Baryon OS service and receives no content key.

The complete object, key, metadata, rollback, and multi-provider boundary is
defined in
[ENCRYPTED-OBJECT-STORAGE.md](ENCRYPTED-OBJECT-STORAGE.md). Its core rules are:

- a project-owned, versioned container format, without custom cryptographic
  primitives;
- authenticated encryption of content and sensitive manifest metadata;
- per-object keys and a separate Key Service as the target architecture;
- provider credentials separated from content keys;
- cryptographic agility and an explicit migration path;
- separate tests for rollback/replay, deletion, and availability;
- candid documentation of provider metadata and endpoint compromise as
  residual risks.

This is an architecture decision, not evidence that OneDrive, Google Drive, or
another provider already works on Baryon OS, and not a claim of absolute
resistance to compromise.

## 2. Corrections to the supplied blueprint

| Subject | Governing formulation |
|---|---|
| seL4 in 2026 | AArch64 security proofs were published in August. The September reduction of 250,000 lines/57% concerns the abstract-to-design refinement layer, not every proof. The configuration matrix distinguishes C-level functional correctness, access/information flow, and binary correctness by architecture. No universal machine-code guarantee is adopted. |
| Kernel size | A small kernel makes review and proof more manageable. Fewer lines do not establish a proven percentage reduction in attacks. The TCB is determined per guarantee; for NAS privacy it includes, among other components, the client, keys, updater, and permission handling. |
| Existing operating systems | XNU describes itself as hybrid; Windows also has user-mode drivers. An exploitable kernel-driver defect can have broad effects, but not every overflow automatically yields complete compromise. |
| Driver isolation | CPU page tables do not restrict arbitrary DMA by a device. Devices, mappings, IOMMU policy, and recovery require separate control. M5/M6 still keep parts of virtio transport, MMIO, and DMA in the kernel. |
| Capabilities | Unforgeable references and limited delegation are necessary. A misconfigured broker can still misuse its own authority. Caller binding, rights attenuation, generation/revocation, and initialisation are separate invariants. |
| Rust/CHERI | Safe Rust relies on correct unsafe/compiler boundaries. CHERI uses protected hardware tags and bounds; complete lifetime and subobject protection requires an appropriate compiler, ABI, allocator, and revocation mechanism. The ISA alone does not eliminate every use-after-free. |
| Cryptography | NIST algorithms rest on studied hardness assumptions; functional verification does not prove that future cryptanalysis is impossible. A library with verified components does not prove its integration into Baryon OS or every backend and machine-code path. |
| XTS and storage | AES-XTS provides confidentiality, not authenticity. A Merkle tree requires a trust-anchored root, length/position/version binding, and rollback policy. A standalone checksum is not an integrity guarantee against an attacker. |
| Quantum claims | Grover/preimage and collision attacks are different problems. A 256-bit hash does not simply have 128-bit quantum collision resistance. A small Grover simulation does not prove PQC security. |
| Entropy and clearing | Hashing predictable clock values does not create entropy. Zeroisation is useful memory hygiene, not a guarantee that every copy, register, or crash-memory location has been erased; abort does not reliably execute `Drop`. |

The supplied image may illustrate an attack scenario. Labels such as “complete
compromise” are scenario outcomes under stated assumptions; a green colour is
not executed isolation evidence. In Baryon OS, a screen may say “verified” only
when the version, property, and evidence are available.

## 3. Protocol choice: no custom hybrid tunnel

The supplied KDF processes `X || M || optional_psk? || context` without
framing between the optional PSK and the context. `Some(P), context=C`
and `None, context=P||C` therefore produce identical input. This is an
ambiguous format, not a discovered BLAKE3 collision. It does not by itself
demonstrate an exploitable attack on a system that does not yet exist, but the
fragment is not an acceptable session protocol. Transcript and role binding,
authentication, key confirmation, replay handling, and downgrade policy must
be coherent together.

The first NAS transport experiment uses the development host's maintained TLS
implementation with **TLS 1.3 and only X25519MLKEM768**, as specified by
**RFC 10024 (August 2026)**. The standard combines the secrets within the TLS
transcript and existing key schedule. No custom BLAKE3 KDF, additional PSK
combiner, or self-written record encryption is added. This is an experimental
profile; broad application/web compatibility and product migration require
separate policies later.

### N02 — executable host profile

- Server and client use only `127.0.0.1`; there is no real NAS, internet
  port, router configuration, or DNS discovery.
- TLS 1.3, exactly `X25519MLKEM768`, exactly
  `TLS_AES_256_GCM_SHA384`, and a fixed ALPN; no classical fallback,
  session reuse, or 0-RTT in this experiment.
- A temporary test CA and mutual certificate validation, with expected server
  and client certificate pins. Server-hostname validation remains mandatory.
- Permission is checked before the socket attempt. After actual
  authentication, an opaque transport fact is created. Rights are checked
  again for every application write. Closing the connection invalidates the
  fact.
- Only a bounded echo of N01 ciphertext with namespace validation. The NAS
  endpoint receives no document-decryption key. This is not a storage or
  synchronisation API and does not replace N01 revoke/evict.
- Handshake and application limits bound counts, bytes, and time. Errors close
  the connection without moving to a weaker profile.
- Evidence reports the actually negotiated group, protocol, and cipher, and
  the runtime used. A supported configuration string alone is not evidence of
  a successful PQC handshake.

**Explicit boundary:** the test certificates use classical Ed25519
authentication. Hybrid key exchange therefore does not provide post-quantum
protection for the entire authentication, boot, or update chain. Additional
solutions are required against a future quantum attacker targeting online
authentication. The LAN/internet labels in policy tests are harness assumptions
over actual loopback transport; they are not an observed WAN route or native
socket enforcement. Node/OpenSSL are development tools, not a selected
Baryon OS runtime.

## 4. Actual native integration boundaries

| Boundary at the baseline commit | Consequence for the next native task |
|---|---|
| Rust 1.90.0; `x86_64-unknown-uefi` and `x86_64-unknown-none` | No custom target JSON or libc. Verify every dependency against exactly this target and the compiler used. |
| `tools/build_users.py` builds probes with direct `rustc` and `#[path]` modules | A new Cargo crate is not automatically connected to ring 3. First create an explicit, reproducible service-build route. |
| No x87/SSE/AVX in the current profile | Select the portable backend explicitly and inspect generated code; do not enable AVX2 implicitly. |
| 64 KiB user stack with guards; fixed user heap without a general allocator | Measure stack peaks and heap/allocation requirements, including error paths. An arbitrary new 128 KiB minimum is not evidence. |
| M6 text at most 64 KiB; rodata at most 4 KiB | Cryptographic code and keys do not necessarily fit. Adjust the ELF layout and its regressions together if required. |
| Message320: at most 256 payload bytes; block protocol 192 bytes | Large keys, ciphertexts, and signatures require bounded transfer, explicit ownership, and lifetime; do not pass an arbitrary user pointer. |
| No Entropy capability/GetRandom | Do not generate native keys until source policy, readiness, failure handling, and the syscall/service boundary have been implemented and tested. |
| `services/block/` uses FNV in synthetic M5/M6 experiments | This is diagnostics, not cryptographic integrity. Do not present write recovery or the open M6a gate as verified encrypted storage. |

`libcrux-ml-kem` and `libcrux-ml-dsa` **0.0.11** are evaluated
candidates, not added dependencies. The reviewed tag states Rust 1.89 as the
minimum and Apache-2.0 as the licence. The `no_std` feature selection
and Rust build scripts, including the transitive tree, must be reviewed.
`default-features=false` is not independent evidence that AVX2 is absent.
Record the backend choice, exact source/lock hashes, build-script output, and
available proofs. ML-KEM verification is partial and described by the
maintainers as evolving.

The ML-KEM-768/ML-DSA-65 parameter choice remains a candidate for integration.
FIPS 203 uses implicit rejection: a manipulated ciphertext need not produce a
public decapsulation `Err`; subsequent authentication/key confirmation
must fail. The actual protocol behaviour must therefore be tested. Signatures
establish provenance under a key, not good intentions or entitlement to
document or network capabilities.

## 5. Bounded follow-up tasks

| ID | Exclusive outcome | Acceptance before the claim |
|---|---|---|
| V01 — capability proof | Formal model of rights attenuation, transfer, object generation, and revocation | Machine-checked invariants, model assumptions, and their relation to the actual Rust functions. Do not call model checking without refinement a complete code proof. |
| P01 — native cryptographic build | Isolated `crates/pqc-core` adapter using a reviewed standard library; no dependencies in `kernel/` | Pinned dependency tree and licences; host vectors; actual `no_std`/ring-3 build; portable instructions; measured stack/ELF budgets. No custom network protocol. |
| P02 — entropy | Specified entropy capability and source policy with readiness/error path | No key generation before ready; an empty or failed source remains blocked; a test seed never reaches a product path. Assess DMA/virtio-rng and VM snapshots separately. |
| P03 — system image | Signed manifests with algorithm version, target, length, digest, version, and rollback policy | Both required signatures over exactly the same canonical message; pinned trust anchors; invalid, missing, substituted, or old metadata rejected in an actual guest run; firmware boundary stated. |
| P04 — storage | Authenticated storage format with crash, nonce, and rollback design above the block service | First complete the M6a port/regression; then test bit flips, block swaps, replay, interrupted writes, recovery, and key rotation on disposable images. AES-XTS alone is insufficient. |
| P05 — opaque provider storage | Provider-independent ciphertext adapter above the authenticated object format | Provider credentials without a content key yield no plaintext; names/manifests remain protected; deletion, outage, rollback, and absence of silent fallback are tested. Initial adapters use synthetic/test accounts before real user data is permitted. |
| N03 — native NAS | Project network service, transport library, secure pairing/key lifecycle, and storage adapter | Peer-bound rights before connection and per operation; E2EE through the user client; administrator revoke/evict blocks old uploads; actual OS/network experiments. |
| Q02 — educational Grover | Separate six-qubit oracle/diffusion experiment in qsim | All 64 targets, norm preservation, success probability after six iterations, decline after excessive iterations, and cancellation. No claim of physical quantum acceleration or PQC proof. |

Qsim now supports H/X/Z/S/CX and at most 12 qubits. A general six-bit Grover
experiment therefore requires an actual simulator extension, not merely a test
of the existing gates. The 28–34-qubit estimate in the supplied text is not a
property or tested limit of this repository.

For P03, package authenticity and application authorisation remain separate: a
valid package receives only the rights granted by the OS/user. Application
store choice and privacy must not depend on one supplier. For N03, automatic
resume, usable key recovery, and multi-device operation remain product
requirements. A provisional test limit is not a permanent consumer
restriction.

## 6. Primary sources

Accessed 27 October 2026. The project choices and integration consequences are
based on project analysis, not certification by the listed organisations.

- Proofcraft. (2026, 21 August and 21 September). *News 2026*. https://proofcraft.systems/news-2026/
- seL4 Project. (n.d.). *Verified configurations*; *Frequently asked questions*. https://docs.sel4.systems/projects/sel4/verified-configurations.html ; https://sel4.systems/About/FAQ.html
- University of Cambridge. (n.d.). *CHERI frequently asked questions*. https://www.cl.cam.ac.uk/research/security/ctsrd/cheri/cheri-faq.html
- Wang, Y., et al. (2026). *CHERI-D: Secure and efficient inline object ID for CHERI temporal memory safety*. https://arxiv.org/abs/2606.19055
- Apple. (n.d.). *XNU README*. https://github.com/apple-oss-distributions/xnu
- Microsoft. (n.d.). *User mode and kernel mode*. https://learn.microsoft.com/en-us/windows-hardware/drivers/gettingstarted/user-mode-and-kernel-mode
- Kwiatkowski, K., Kampanakis, P., Westerbaan, B. E., & Stebila, D. (2026). *Post-Quantum Traditional (PQ/T) Hybrid Key Agreement Mechanisms for TLS 1.3* (RFC 10024). https://www.rfc-editor.org/rfc/rfc10024.html
- NIST. (2024). *FIPS 203; FIPS 204; FIPS 205*. https://csrc.nist.gov/pubs/fips/203/final ; https://csrc.nist.gov/pubs/fips/204/final ; https://csrc.nist.gov/pubs/fips/205/final . Check the published errata for every implementation pin.
- Gueron, S., Langley, A., & Lindell, Y. (2019). *AES-GCM-SIV: Nonce Misuse-Resistant Authenticated Encryption* (RFC 8452). https://www.rfc-editor.org/rfc/rfc8452
- Biryukov, A., Dinu, D., Khovratovich, D., & Josefsson, S. (2021). *Argon2 Memory-Hard Function for Password Hashing and Proof-of-Work Applications* (RFC 9106). https://www.rfc-editor.org/rfc/rfc9106
- NIST. (2026). *Considerations for Achieving Crypto Agility: Strategies and Practices*. https://csrc.nist.gov/pubs/cswp/39/upd1/final
- NIST. (2010). *SP 800-38E: XTS-AES*. https://csrc.nist.gov/pubs/sp/800/38/e/final
- NIST. (2018). *SP 800-90B: Entropy sources*. https://csrc.nist.gov/pubs/sp/800/90/b/final
- Brassard, G., Høyer, P., & Tapp, A. (1997). *Quantum algorithm for the collision problem*. https://arxiv.org/abs/quant-ph/9705002
- Libcrux contributors. (2026). *libcrux-ml-kem 0.0.11; verification status*. https://docs.rs/libcrux-ml-kem/0.0.11/libcrux_ml_kem/ ; https://github.com/celabshq/libcrux/blob/libcrux-ml-kem-v0.0.11/libcrux-ml-kem/proofs/verification_status.md
- RustCrypto. (2026). *zeroize*. https://docs.rs/zeroize/latest/zeroize/
- OpenSSL Project. (2026). *SSL_CTX_set1_curves, OpenSSL 3.5*. https://docs.openssl.org/3.5/man3/SSL_CTX_set1_curves/
- OpenJS Foundation. (2026). *TLS API* (current documentation 26.12.1; the runtime actually tested is recorded separately). https://nodejs.org/api/tls.html
