# Rust 1.100 cloud compatibility checkpoint

Source: PR #354, `codex/phase0-rust-1-100`, commit
`92e44b0c23a34be1b31037dc0561a4b22b04ef5a`. The temporary base remains
`codex/phase0-pr340-preintegration` at `f5a976f17a3167a08ad513f0328cba8de3dd8a62`.
This is **preintegration evidence**, not acceptance of the eventual combined main.

The candidate completes compiler compatibility repairs, matches both native input
workflows to the selected toolchain, preserves the C memory-intrinsic ABI using
`c_void`, and accepts Cargo 1.100's exact crate/hash-bound dependency-artifact
layout. Existing authority, pointer-range, pixel-validation and gate rules remain
in force. The official stable manifest identifies Rust 1.100.0, released on
12 November 2026; its observed SHA-256 is recorded in [checks.json](checks.json).

| Verification | Source and result |
| --- | --- |
| Strict preflight, normal build, M1 smoke | Candidate source above; exits 0, 0, 0; real QEMU exit 33 and `M1:PASS` |
| DocumentSession normal and cold read-only recovery | Same source; exit 0, two real VMs, both raw exit 33; [complete report](document-session/result.json) |
| Complete native volume/archive gate | Same source; exit 0, 15/15 stages, 304.104 s, including ustar and three real crash injections; [report](volume/volume-gate.json) |
| Native display component Clippy | All five workflow host manifests pass with `-D warnings` |
| Native display scalar builds | All four workflow libraries cross-build with the original no-SIMD flags |
| Surface-protocol tests | 23 passed, including malformed input, expiry, failed callbacks and stale authority |

Both complete native gates bind source-v3 fingerprint
`62ee135a73f76641d57dcda5870b0d6b5f19c4f76337154bea6fccb217e62ed7`.
The existing capture tool verified all 15 volume stages and their retained image
hashes before publication. Disk images, compiled binaries and credentials are not
included. `.gitattributes` preserves the captured bytes without line-ending
normalisation. [SHA256SUMS](SHA256SUMS) covers the published receipt material;
the volume subdirectory also retains the capture tool's manifest.
The serial-file whitespace attribute recognises the firmware's captured CRLF
line endings without changing its bytes or disabling other whitespace checks.

[checks.json](checks.json) preserves commands, raw exits, source limits and log
hashes. Earlier failed or interrupted attempts remain explicit: a changing-source
DocumentSession run was rejected; an earlier volume attempt was stopped for a
later compiler diagnostic; the extended local SMP contention test was terminated
after 414.654 seconds and is not a passing SMP suite. The separate 19 scheduler
tests passed. Other earlier host checks retain their own source versions.
Raw command logs use deterministic gzip archives; decompression restores the
original bytes, including terminal line endings and trailing blank lines.

The [hosted snapshot](hosted-ci-at-source.json) records checks observed on this
candidate, including the passing mandatory QEMU job; pending checks are not
accepted. Reconciliation after the documentation, design and S09 integrations,
current-head checks, independent final review and actual integrated-main
verification remain required. The broad source-v3 fingerprint includes documents
under SDK, service, userspace and tooling directories, so unchanged executables
alone do not establish an unchanged fingerprint.

Compiler warnings remain visible, including now-unnecessary CPUID `unsafe`
wrappers, profile-dependent unused items and an upstream deprecated attribute in
the protected vendored JPEG crate. These warnings were not suppressed. Physical
hardware, KVM, production update custody, complete S09 destination/key decisions
and human audit remain unverified. No progress criterion is newly accepted.
