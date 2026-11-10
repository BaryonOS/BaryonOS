# Result-oriented GitHub Milestones M6–M11

## Proposal and execution boundary

This proposal for [issue #276](https://github.com/simonvanieperen/BaryonOS/issues/276)
provides copy-ready GitHub Milestone titles and descriptions for the active
phases in the [product roadmap](ROADMAP.md#technical-milestones). Merging this
file does not create or update GitHub Milestone objects, assign issues, or
complete issue #276. Those repository-metadata changes require a separately
authorised execution and a live verification afterwards.

The four fields under each milestone title form its complete proposed GitHub
description. The roadmap defines the phases, while
[PROGRESS.md](PROGRESS.md) remains the sole criteria and evidence register.
Pull-request updates and test logs remain in issues, `PROJECT-STATE.md`,
`CHANGELOG.md`, or `docs/verification/`. These descriptions introduce no
deadline, status claim, or new product acceptance.

M6–M11 is the sole active GitHub Milestone taxonomy. Existing rubric-named
objects are migration sources, not a second active milestone family. Rubric
identifiers such as `S09`, `U04`, and `N03` remain criterion and issue
identifiers.

## M6 — Storage and Recovery

**Required outcome:** Persistent user storage with scalable documents,
catalog/history, fault handling, backup/export, and controlled update/recovery
design

**Acceptance evidence:** Fault injection, cold recovery, full-storage cases,
corruption handling, rollback/downgrade tests, independent disk checks

**Decision gate:** Daily-use claims remain blocked without recovery and
independent review

**Canonical sources:** [Roadmap](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/ROADMAP.md#technical-milestones)
and [criteria and evidence](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROGRESS.md).

## M7 — Native Product Integration

**Required outcome:** Desktop/session, input, Material workflows, settings,
accessibility, and at least one useful native application integrated on the
independent OS

**Acceptance evidence:** Native task tests, accessibility checks,
visible/revocable application authority, recovery behavior, measured
interaction budgets

**Decision gate:** Product direction follows user/task evidence; do not scale
the app ecosystem before the core experience is usable

**Canonical sources:** [Roadmap](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/ROADMAP.md#technical-milestones)
and [criteria and evidence](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROGRESS.md).

## M8 — Networking and Security Hardening

**Required outcome:** Bounded network stack/services, maintained cryptography,
key management, privacy contract, update trust chain, and targeted security
review

**Acceptance evidence:** Protocol/fuzz tests, negative handshakes, traffic
capture, no unintended listeners, key/update chain evidence, independent
findings resolved

**Decision gate:** No sensitive-data safety claim without scoped audit and
published residual risks

**Canonical sources:** [Roadmap](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/ROADMAP.md#technical-milestones)
and [criteria and evidence](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROGRESS.md).

## M9 — First Reference PC

**Required outcome:** One documented physical reference system works end to
end

**Acceptance evidence:** Boot, drivers, input, graphics/audio as applicable,
DMA boundaries, sleep/wake, power, network, storage, and recovery on that exact
hardware

**Decision gate:** Support matrix must state exactly what is and is not
supported

**Canonical sources:** [Roadmap](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/ROADMAP.md#technical-milestones)
and [criteria and evidence](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROGRESS.md).

## M10 — Second Architecture

**Required outcome:** The core process/capability/IPC model runs on a second
ISA, initially for example AArch64 under QEMU

**Acceptance evidence:** The same core contract tests pass with
architecture-specific differences documented

**Decision gate:** Generalize abstractions only after real architecture
differences are observed

**Canonical sources:** [Roadmap](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/ROADMAP.md#technical-milestones)
and [criteria and evidence](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROGRESS.md).

## M11 — Experimental Community Release

**Required outcome:** Reproducible experimental release, SDK, contributor path,
support policy, security reporting, licensing/SBOM, and known-issues
documentation

**Acceptance evidence:** Independent build reproduction, release artifacts,
documented supported scope, responsible reporting channel, maintenance
process

**Decision gate:** Production classification follows evidence, not project
ambition

**Canonical sources:** [Roadmap](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/ROADMAP.md#technical-milestones)
and [criteria and evidence](https://github.com/simonvanieperen/BaryonOS/blob/main/docs/PROGRESS.md).

## Migration of the existing rubric milestones

The ten existing rubric-named objects are migrated through one sequence:

1. Create exactly one open GitHub Milestone object for each canonical M6–M11
   title above.
2. Re-query every issue assigned to a legacy object immediately before the
   metadata change; do not rely on the assignment count observed while this
   proposal was prepared.
3. Reassign each issue by its current remaining outcome using the crosswalk
   below. An issue has at most one primary GitHub Milestone even when other
   phases remain dependencies.
4. After all assigned issues have moved, close rather than delete all ten
   legacy objects so their descriptions and closed history remain available.
5. Verify that exactly six canonical objects remain open, all ten legacy
   objects are closed, and no parallel active taxonomy remains.

| Existing rubric milestone | Canonical destination | Migration rationale |
|---|---|---|
| `K06 — Multicore and process isolation` | M7 | Current-architecture kernel work enables native product integration; it is not second-ISA evidence. |
| `B02 / S09 — Encrypted storage and backups` | M6 | Storage, recovery, backup, and export belong to the storage phase. |
| `U02–U06 — A native desktop for everyday use` | M7 | Desktop, session, accessibility, and native tasks are product integration. |
| `N02–N04 — Internet, TLS and network permissions` | M8 | Bounded networking, authenticated transport, and permissions are security hardening. |
| `A02–A04 — Installing apps and using them daily` | M7 | Native applications and their daily workflows are product integration. |
| `P01–P04 — NAS, synchronisation and multiple devices` | M8 | Networked privacy, authenticated transport, revocation, and multi-device security are primary; M6 remains a dependency. |
| `L01 — Local system AI` | M7 | Local AI is a native product function; its no-network constraints remain an M8 dependency. |
| `H02–H06 — Hardware, drivers and power` | M9 | The work establishes the physical reference-system support matrix. |
| `B04–B06 — Chain of trust and security review` | M8 | Trust-chain work and independent review are security hardening. |
| `R01–R03 — Full test release and maintenance` | M11 | Reproduction, release acceptance, and maintenance belong to the community release. |

M10 intentionally receives no legacy object merely to populate the phase.
Second-architecture work receives M10 only when its issue has that actual
remaining outcome.

## Proposed current issue assignments

The table records current remaining scope, not completion status. Each issue
must be re-queried before assignment.

| Issue | Primary milestone | Remaining scope before assignment |
|---|---|---|
| [#39](https://github.com/simonvanieperen/BaryonOS/issues/39) — residual native network hardening | M8, after explicit issue retitling or rescoping | N01 is accepted. The remaining work is the combined native retransmission, backpressure, capability-revocation, unexpected-NIC-reset, exhaustion, and cleanup matrix. |
| [#41](https://github.com/simonvanieperen/BaryonOS/issues/41) — production update trust chain | M8 | B03 is accepted. The remaining work is production release anchors, rotation, expiry/revocation, release policy, and evidence for the real trust chain. |
| [#42](https://github.com/simonvanieperen/BaryonOS/issues/42) — input and USB/hotplug | M9 | H02 remains open for the selected physical reference profile. |
| [#274](https://github.com/simonvanieperen/BaryonOS/issues/274) — English documentation | M11 | Complete the remaining active technical and community documentation. |
| [#277](https://github.com/simonvanieperen/BaryonOS/issues/277) — GitHub Wiki | M11 | Publish the human-readable Wiki layer; a source proposal alone does not complete it. |
| [#278](https://github.com/simonvanieperen/BaryonOS/issues/278) — private vulnerability reporting | M11 | Enable the actual private reporting channel; this does not replace M8 security review. |

Issue [#43](https://github.com/simonvanieperen/BaryonOS/issues/43) remains
unassigned until its multiple product outcomes are split. Issue #276 is the
governance operation itself and has no product milestone. No unrelated issue
is assigned to M6, M7, or M10 merely to populate a phase.

## Verification after separately authorised execution

Live GitHub metadata must show exactly one open object for every canonical
M6–M11 title, all ten rubric-named objects closed, and at most one primary
milestone per issue. Every canonical description must contain both source
links above. Issue content and closed milestone history must remain available.
Only this live verification can satisfy issue #276; merging this proposal and
its document-contract test cannot.
