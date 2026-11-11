# Security, privacy and post-quantum design

Status: security design, updated 27 October 2026. M1–M5 have their own QEMU test evidence, M6a still has an open final gate and N01 has host/mock evidence. There is no formally verified own kernel. The requirements below are not automatically implemented or proven; the current execution is in PROJECT-STATE.md.

The new microkernel/CHERI/PQC blueprint has been reviewed and translated into [VERIFIABLE-SECURITY-AND-PQC.md](implementation/VERIFIABLE-SECURITY-AND-PQC.md), with source corrections, the actual native build boundaries, an N02 trial profile for TLS 1.3/X25519MLKEM768 (RFC 10024) and separate evidence/integration tasks. No own hybrid KDF/tunnel, silent seL4 kernel switch or unproven CHERI guarantee. Privacy and a smooth connected product experience remain joint requirements.

## 1. Starting point

The goal is demonstrable limitation of damage: a malicious app may use only the resources explicitly granted to it. An independently designed kernel can pursue that, but does not automatically become more secure than existing systems because it is new, small, open source or written in Rust. An unknown kernel in fact starts with much less field experience, audit work and hardware support.

Do not use claims such as “uncrackable”, “AI-proof”, “100% quantum-safe” or “as secure as macOS” without a precisely described measurement method. Publish per security property: implementation status, version, tests, assumptions and remaining risks.

## 2. What is correct about the macOS example?

Apple uses, among other things, app sandboxing: the app gets restricted access to files, network and hardware. That reduces the damage on compromise. App Sandbox is mandatory for distribution via the Mac App Store; the documentation also describes how existing apps can adopt it. This does not mean that all software on a Mac runs in fully separated, impenetrable compartments. System Integrity Protection is a different layer and restricts, among other things, modification of critical system files, including by processes with administrator privileges (Apple, n.d.; Apple, 2021).

Design lesson: combine small privileges, process isolation, integrity checking, secure updating and recovery. A sandbox is not proof that escape is impossible. Nor is a microkernel synonymous with a secure sandbox.

## 3. Trust boundaries and trusted computing base

A trust boundary is a boundary where input, code or permissions must be checked again. The trusted computing base (TCB) is all code and hardware on which a particular security guarantee relies. “Small kernel” and “small TCB of the whole product” are different things.

Proposed design:

| Component | Trust and authority | Design requirement |
|---|---|---|
| Firmware, boot path and CPU | Underlying trust basis | Record exact platform assumptions; do not promise protection against malicious firmware |
| Kernel | Memory, scheduling, interrupts, IPC and capability checks | As small as possible; no browser, network protocol parser or complex file parser in the kernel |
| Initial manager | Distributes resources and authority at startup | Temporary broad permissions; split and restrict after initialisation; version the policy |
| Drivers | Only the assigned device and buffers | Separate processes where feasible; DMA restriction before real DMA devices are trusted |
| System services | Own isolated address space | Only specific IPC endpoints and explicit transfer of resources |
| Apps | No general access by default | Own storage; consent via system portals for additional files/devices/network |
| Updater and key management | Highly sensitive roles | Separate responsibilities and keys; no general app permissions |
| Any AI agent | Untrusted app with delegation | The same permission model as every app, plus short task-bound authorisations |

In this proposal, a capability is a kernel-managed reference to an object with rights. A process must not obtain new rights by guessing a number. seL4 is a useful primary learning source for this concept, but no seL4 code or verification guarantee is adopted here (seL4 Project, n.d.-a).

For the own capability API, the following must be specified before implementation: creation, transfer, restriction of rights, revocation, object lifetime, generation checking on reuse of handles, cleanup on process failure, and protection against the confused-deputy error. The latter error arises when a privileged service unintentionally uses its own rights on behalf of an unauthorised requester. Make clear which identity/rights are decisive for each IPC operation. Also limit numbers of objects, queues and CPU time: access control alone does not prevent exhaustion.

## 4. Threat model and testable evidence

Protected assets: user documents, private keys, account data, camera/microphone, device control, update integrity, availability and recoverability. Adversaries: malicious app, network party, compromised dependency, stolen account or key, malicious contributor, physical attacker and automated or AI-assisted attacker.

| Risk | Proposed control | Minimum evidence trial |
|---|---|---|
| App reads memory of another app/kernel | Hardware address-space isolation and controlled mappings | Process A attempts read/write/execute in B and kernel; A is stopped, B keeps functioning |
| App forges or reuses capability | Kernel management, type/right/generation checks | Random, revoked, wrong and reused handles give no access |
| Privileged service abused by unauthorised app | Explicit delegation and caller checks | Requester cannot obtain a broader right via an intermediary than permitted |
| App causes disruption | Quotas, fault isolation and bounded IPC | Crash, large messages and resource exhaustion leave other test services usable |
| Driver or device writes arbitrary memory | Minimal mappings and, on a suitable platform, a correctly configured IOMMU | DMA attempt outside assigned buffers is blocked; absence of protection is visibly reported |
| Corrupted document or network message | Small parsers outside the kernel, bounds checks and fuzzing | Corpus of malformed input causes no escape or uncontrolled resource growth |
| Bad or manipulated update | Signed metadata, version policy, key rotation, atomic activation | Wrong signature, rollback, mix-and-match, expired metadata and aborted installation are handled correctly |
| Device theft | Encrypted user storage and controlled key release | Offline copy of storage contains no plaintext user data; recovery path is tested |
| Release key theft | Offline trust anchors, separated roles and multiple signers where appropriate | Rotation and recovery after compromise work on an existing installation |
| AI agent is steered by malicious document | Document content grants no rights; system check on actions | A document saying “upload all files” cannot cause files outside the task authorisation to be read |
| Privacy loss through diagnostics | No telemetry by default; minimal local logs with retention period | Network observation of default installation and check for secrets in logs |
| Supply-chain attack | Pinned dependencies, source provenance, controlled builds and review | Rebuild with pinned inputs; analysis of deviating outputs; dependency change visible in review |

These trials are necessary but not sufficient for a general security claim. A test proves the tested scenario, not the absence of all errors.

## 5. Development rules for kernel and services

1. Use predominantly safe Rust. Keep assembly, `unsafe`, FFI and hardware access small and separately reviewable. Every `unsafe` construct gets an explanation of the invariants, their owner and the relevant test or manual check. Rust permits, among other things, deadlocks, logical race conditions, memory leaks and incorrect authorisation; `unsafe` and incorrect abstractions can moreover break memory safety (Rust Project, n.d.).
2. Define a W^X policy: memory pages are in principle not writable and executable at the same time. Record exceptions, mappings, executable loaders and future JIT needs separately.
3. Validate all user pointers, lengths, integer computations and object types at the boundary. Determine what happens when a mapping changes during an operation; a one-time check does not by itself prevent TOCTOU errors.
4. Apply ASLR, stack protection and additional hardware measures where the chosen platform supports them correctly. Name missing measures explicitly; settings are not a substitute for tests.
5. Introduce network, USB, GPU, SMP, DMA and persistent storage step by step. Every addition enlarges the boundary to be examined. Start with QEMU and synthetic data.
6. Use model checking or formal specifications for bounded invariants, for example capability transfer and object lifetime. Passing an abstract model does not automatically prove that the implementation does the same.
7. Make errors in security-sensitive subsystems visible and fail closed. At the same time design a recovery path so that one unusable update does not permanently make all data inaccessible.

seL4 shows how strong formal verification can be, but also documents assumptions around boot code, assembly, hardware and DMA. Timing channels are not covered by the stated confidentiality coverage. The own kernel needs its own specifications and proofs; naming seL4 or using the same architecture type transfers no proof (seL4 Project, n.d.-b).

## 6. Protection against AI-assisted attacks

Treat AI as a possible accelerator of existing attack classes: more attempts, faster searching for errors, credible deception and attacks on development tools. The OS does not need to recognise an “AI attack” to refuse an unauthorised system call.

For an optional assistant, only local inference on the device applies, with network access denied and without cloud fallback. Model downloads go through a separate service without document or context access. See LOCAL-PRIVACY-CONTRACT.md. Furthermore:

- No permanent admin rights, general shell authorisation or unrestricted access to the home directory.
- Authorisation has a concrete scope: objects, action, destination, duration and possibly maximum amount/quantity. The kernel/service enforces this; the promise in the model prompt is insufficient.
- Separate reading from executing and reading from sending. Access to a document is not consent to upload it.
- Provide trusted system dialogs for a new sensitive permission. An app must not be able to forge consent text as a system notification.
- Log execution decisions locally without needlessly retaining document content or secrets. Make consent revocable.

For Codex during development: no production keys, real user data or release privileges in the workspace. Tool output, issues, comments and dependency documents may contain untrusted instructions. Have an independent reviewer assess the changed code and the evidence; having the same agent say again that it is secure is not an independent audit.

## 7. Post-quantum cryptography: what is and is not protected

Post-quantum cryptography (PQC) runs on ordinary computers and aims to provide cryptographic protection against attackers with a large quantum computer. It does not render harmless a bug in the kernel, misconfigured permissions, a stolen key or a leaked password.

| Standard | Algorithm | Role in a possible OS |
|---|---|---|
| FIPS 203 | ML-KEM | Establishing a shared secret; not a standalone file encryptor and not a signature |
| FIPS 204 | ML-DSA | Digital signatures, for example in a suitably designed update protocol |
| FIPS 205 | SLH-DSA | Hash-based digital signatures; different trade-offs for size and speed |

These standards were published on 13 August 2024. NIST phrases the resistance as based on current knowledge, not an unconditional guarantee. For actual implementation, check the applicable specification, errata, parameter choice, library version and protocol support; the consulted FIPS 203 and 204 pages refer to errata (NIST, 2024a, 2024b, 2024c).

Recommended design policy:

1. **No own cryptographic algorithms or self-devised hybrid combinations.** Reuse maintained, independently reviewed implementations that support the chosen standard version. An own OS does not need to use its own mathematics.
2. **Create a crypto inventory.** Register algorithms, keys, certificates, protocols, trust anchors, retention periods and dependencies on firmware/TPM/cloud. Start before the first encrypted network protocol.
3. **Design for crypto agility.** Give keys and metadata explicit algorithm and format versions, with a restricted list of permitted options. Never negotiate down insecurely to a weaker algorithm. Test revocation and migration.
4. **Use PQC in a suitable, specified protocol.** “ML-KEM added” is not proof that authentication, channel binding, downgrade protection and key derivation are correct. Hybrid deployment can be useful, but requires an established protocol and independent review; do not glue algorithms together yourself.
5. **Protect long-term confidential data early.** Recorded network traffic can be attacked later. Disk encryption therefore does not replace a future-proof transport protocol, and vice versa.
6. **Keep using symmetric encryption where appropriate.** AES-256 is a candidate for security margin in a suitable storage or message format. Mode, nonces, authentication, key derivation and key management must each be correct separately. ML-KEM does not replace those components. NIST treats the quantum threat to symmetric crypto differently from that to classical public-key systems (NIST, 2026).
7. **Test the whole chain.** Known test vectors, interoperability, malformed keys/signatures, entropy failures, key lifetime and side channels. An algorithm name on a checklist is insufficient.

A PQ-signed system image does not guarantee a PQ-secure secure-boot chain if firmware only enforces a classical vulnerable signature. State explicitly which part of boot, recovery, firmware and updates is protected. No end-to-end claim as long as one trust anchor is not covered by the claim.

Secure entropy is a separate design problem. Do not derive keys from time, fixed seeds or only a predictable emulator source. In the absence of suitable entropy, do not perform cryptographic key generation. A test seed belongs exclusively in clearly marked tests.

## 8. Updates, rollback and recovery

The Update Framework is a reference for separated signing roles and protection against, among other things, old updates, frozen update information and inconsistent metadata. Use the specification and a suitable implementation; do not claim TUF compatibility based only on a similar design (The Update Framework, n.d.-a, n.d.-b).

The OS proposal combines atomic activation with a tested previous installation. Distinguish between an authorised recovery action and an attacker offering a vulnerable old version. Define the lowest permitted security version, authorisation for exceptions, key rotation and what happens when the clock is unreliable. Test power loss before, during and after activation and verify that user data is preserved. Never simply copy half a system update over the active installation.

Reproducible builds help to compare source and output, but do not prove that the source code is free of vulnerabilities. Signing proves provenance under the key used, not that the software is good.

## 9. Privacy and European autonomy

Revised user requirement (28 October 2026): OS data stays local, goes end-to-end encrypted to an explicitly paired own NAS/NAS of friends or family, or, after an explicit choice, is stored as opaque client-side encrypted objects with an external storage provider. Remote reach via the internet requires separate permissions. No mandatory vendor cloud, silent relay/fallback, provider key escrow or OS telemetry. The NAS administrator may withdraw capacity and wipe their ciphertext copies without the guest's consent; external providers likewise hold no Nieuw OS content keys. Providers cannot guarantee availability and see necessary account/traffic metadata. Local tasks keep working without NAS/provider. Own recovery copies, rollback detection and key recovery are separate requirements. See LOCAL-PRIVACY-CONTRACT.md revision 4, implementation/NAS-SHARING-AND-INTERNET.md and implementation/ENCRYPTED-OBJECT-STORAGE.md.

European hosting alone is not evidence of privacy; nor is open source alone. Also keep dependencies on stores, signing services, build infrastructure, firmware, documentation and external authentication visible. Geography is not a quality mark for cryptography: assess public standards on content, implementation and independent analysis.

For future use by institutions, a separate assessment of concrete data processing, legislation and responsibilities is needed. This document makes no claim of GDPR or other legal conformity.

## 10. A quantum computer on which the OS runs

The practical extension route is a classical computer running this OS that drives a QPU as a specialised compute resource. IBM describes systems with classical runtime servers, qubit control and combined classical/quantum workflows (IBM, n.d.).

“Running the kernel directly on qubits as on an x86 CPU” is therefore not a useful first portability goal. Any later quantum job service may, under this privacy contract, drive only an own locally managed QPU; external quantum cloud processing falls outside the OS integration. This is an explicit research track from the start of design, with Q0–Q4 in QUANTUM-RESEARCH.md. The own kernel as the classical control layer of a hybrid quantum computer is a separate goal; hardware integration follows after specific interface and controller trials. Access permissions and protection of inputs/results remain required. Compatibility with a concrete QPU must be demonstrated per platform. For this OS, which is yet to be built, that compatibility cannot be verified.

## 11. Claims per development phase

| Phase | What may be said after evidence | What not yet |
|---|---|---|
| Boot demo | Starts in the named emulator configuration | Safe for sensitive data |
| Process isolation | Named negative isolation tests pass on this commit | All sandboxes are inescapable |
| Capability prototype | Specified transfer/revocation works in tests | Formally proven authorisation |
| Crypto integration | Named implementation, vectors and interoperability work | The entire OS is quantum-safe or FIPS-validated |
| External review | Named components were reviewed on date X; findings are transparent | All future changes are automatically secure |

Use the prototype only with disposable data until the relevant protection has been demonstrably implemented, repeatably tested and reviewed. Plan maintenance, security advisories and updates as permanent project components.

## Sources (APA 7)

Apple. (n.d.). *Protecting user data with App Sandbox*. Apple Developer Documentation. Accessed 26 October 2026, from https://developer.apple.com/documentation/security/protecting-user-data-with-app-sandbox

Apple. (2021, 18 February). *System Integrity Protection*. Apple Platform Security. https://support.apple.com/en-gw/guide/security/secb7ea06b49/web

IBM. (n.d.). *IBM Quantum hardware and roadmap*. Accessed 26 October 2026, from https://www.ibm.com/quantum/hardware

National Institute of Standards and Technology. (2024a). *Module-lattice-based key-encapsulation mechanism standard* (FIPS 203). https://doi.org/10.6028/NIST.FIPS.203 — Publication page and errata: https://csrc.nist.gov/pubs/fips/203/final

National Institute of Standards and Technology. (2024b). *Module-lattice-based digital signature standard* (FIPS 204). https://doi.org/10.6028/NIST.FIPS.204 — Publication page and errata: https://csrc.nist.gov/pubs/fips/204/final

National Institute of Standards and Technology. (2024c). *Stateless hash-based digital signature standard* (FIPS 205). https://doi.org/10.6028/NIST.FIPS.205

National Institute of Standards and Technology. (2026, 5 August). *Post-quantum cryptography: FAQs*. https://csrc.nist.gov/projects/post-quantum-cryptography/faqs

Rust Project. (n.d.). *What unsafe Rust can do*. The Rustonomicon. Accessed 26 October 2026, from https://doc.rust-lang.org/nomicon/what-unsafe-does.html

seL4 Project. (n.d.-a). *Capabilities*. seL4 Documentation. Accessed 26 October 2026, from https://docs.sel4.systems/Tutorials/capabilities.html

seL4 Project. (n.d.-b). *What the proofs assume*. Accessed 26 October 2026, from https://sel4.systems/Verification/assumptions.html

The Update Framework. (n.d.-a). *Roles and metadata*. Accessed 26 October 2026, from https://theupdateframework.io/docs/metadata/

The Update Framework. (n.d.-b). *Security*. Accessed 26 October 2026, from https://theupdateframework.io/docs/security/
