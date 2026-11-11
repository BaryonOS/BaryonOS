# Test strategy and evidence status
This is the project's own test plan, not a report of OS tests already performed.

## N02 — distinguish actual TLS, policy and native evidence

The completed N02 integration under `verification/pqc-transport/` contains 124 tests: 85 service tests, including 30 actual loopback TLS trials, plus 39 UI status tests. Handshake evidence records the negotiated group/protocol/cipher, mutual identity, negative downgrade/server cases and the absence of session reuse. Policy tests are not TLS evidence; neither is a configurable group name without a handshake. LAN/internet harness assumptions remain distinct from actually measured loopback transport. Do not test production keys or external devices. App/permission generations prevent old authenticated connections from reviving after revocation and renewed permission. These results support no native/QEMU, formal, human-audit or PQ-authentication claim.

## From M1 onwards
A successful build, successful emulator boot and functional isolation are separate results. Give each test a name, requirement, exact version/configuration, expected observation, timeout and failure criterion. Archive relevant serial output outside the source tree; record how to reproduce it in the repository.

The boot smoke test must pass on a marker written only by the project's own kernel after handover. A loader banner is insufficient. The harness must fail on a missing marker, unexpected exit and timeout. Also test that the harness actually detects an error. Detect later silent hangs through progress/exit rather than an endless QEMU run.

## Test layers
| Layer | Examples | Evidence limit |
|---|---|---|
| Host unit/property tests | Bounds, allocation model, capability transitions, parsers | No evidence for bare-metal timing/MMIO |
| Cross-build/lint | Target, ABI, linker layout, warnings, unsafe inventory | No boot evidence |
| Emulator integration | Boot, exceptions, processes, syscall/IPC, crash recovery | Virtual hardware is not physical hardware |
| Fuzzing/fault injection | IPC, untrusted parsers, updates, full disk, failures | Limited to examined inputs and execution time |
| Formal model | Rights cannot grow spontaneously; revocation/lifetimes | The model must match the implementation and assumptions |
| Physical reference tests | DMA, firmware, sleep, power and device recovery | Only the exact hardware/firmware combination |
| External review | Invariants, design and implementation | Review scope and residual risks explicit |

Use tools such as Miri/sanitizers only for components/targets they support; do not present a host test as a full kernel check. Keep test/debug facilities out of production profiles.

## Integration requirements for each milestone
- No related regression; known failing tests remain visible.
- A negative test for every new authority boundary.
- The test run and output belong to the integrated version.
- A labelled mock/stub may explore an interface but cannot satisfy a requirement.
- Do not change a test merely to make a bug pass without a requirements decision.
- Bounded, reproducible inputs; timeouts; no real personal data.

## Binding privacy checks
After introducing networking, NAS or AI, all relevant LP tests from LOCAL-PRIVACY-CONTRACT.md apply. Test peer identity, single-use invitations, explicit remote access, encryption of content and names, administrator deletion without a user key, stale uploads after revocation, quotas/restarts, network transitions, no vendor fallback, networkless inference, a separate model downloader, combinations of app permissions and independent recovery of NAS backups. These are future acceptance criteria, not completed results.

## Performance
Use a fixed machine configuration, versions, workload and measurement boundaries. Report sample count, median and spread/p95 where meaningful. Compare the same work with the same security guarantees. Do not compare TCG results with native Windows/macOS figures. Set realistic budgets only after measuring a baseline.

## Supply chain and releases
Later work: record dependencies and checksums, an SBOM, repeatable builds in clean environments, source review, signed artefacts and a recovery test. No CI secrets for unreviewed external pull requests. Keep release keys outside ordinary agents/CI build jobs; use a separate restricted signing step. Self-hosted runners for untrusted PR code must be disposable and isolated.

A CI badge proves only that the configured checks passed for the specified version. Do not describe an OS release as publishable before checking criteria, licensing, reporting channels, recovery procedures and limitations.
