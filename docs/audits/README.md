# English reading guide to the multidisciplinary source assessment

The [coordinating report](OS-MULTI-AGENT-AUDIT.md) and sixteen specialist records describe the explicitly selected source `beafdedfc188154fe7285848fbd411e651013f54`, dated 3 November 2026. Every specialist record identifies that source in its opening metadata. The coordinating report states that later remote changes were not audited, integrated or reverted. The records therefore provide source-bound assessments and proposals; they are not the current normative product contracts.

This English entrypoint summarises each record's reported conclusion and evidence boundary, using the coordinating report's specialist summary, matrix and explicit reconciliation of disagreements. Detailed Dutch observations, citations, commands and original wording remain unchanged in the linked records. The guide is not a full line-by-line translation of those detailed dossiers, a new security scan or a determination of their findings' current repair status. Current [release requirements](../FULL-TEST-RELEASE.md), [privacy contract](../LOCAL-PRIVACY-CONTRACT.md), [accepted decisions](../DECISIONS.md), [checkpoint](../PROJECT-STATE.md) and [criterion register](../progress.json) remain authoritative.

<a id="coordinating-report"></a>

## Coordinating report and method

Original: [OS-MULTI-AGENT-AUDIT.md](OS-MULTI-AGENT-AUDIT.md).

The report recognised the project's own kernel, native services and meaningful bounded VM evidence. It did not establish a complete personal, connected, accessible, recoverable and maintainable release. It separated one concrete high-confidence security source finding, product-integration gaps and specific evidence-archive defects; these categories were not added together as a vulnerability count.

The recorded source branch was `copilot/add-os-specialist-agents`, with parent `69809c2cb53dfb3bd77abe5352892810fba6e82d`; report commit `6929b986440fe659fa1d45167f476313abb6c6d3` recorded the original integration. Ordinary project chats read specialist profiles because the host did not expose custom-agent selectors. At most three analyses ran concurrently. Their assigned write areas were a coordination agreement, not an enforced sandbox. Child worktrees selected the same commit while detached, rather than literally sharing the requested branch name. This was coordinated AI review, with no independent human audit, formal security proof, legal sign-off or release validation.

The shared E0–E8 legend distinguished missing ambition/design/host work/native source/native positive/native negative/repeated gate/physical hardware/independent human evidence. Each label applied only to its named source, property and profile. Historical execution retained its own source; missing evidence was not silently treated as approval or an executed runtime failure. The recorded 41/100 and twenty-one accepted rubric criteria were separate from the sixty-two open full-release rows. Translation changes neither register.

<a id="audit-01"></a>

## AUDIT-01 — Architecture, trust boundaries and interfaces

Original: [01-architecture.md](subagents/01-architecture.md).

The specialist identified the independent kernel, capability IPC and separate services. Account authority, trusted cold key freshness, native TLS and an integrated signed desktop profile remained open in the examined source. Single-core and DMA assumptions required explicit limits. The report treated architectural intent, native source and executed profile evidence separately; it supplied no product-wide containment or physical-hardware acceptance.

<a id="audit-02"></a>

## AUDIT-02 — Buildability and implementation status

Original: [02-build-status.md](subagents/02-build-status.md).

The reported inventory contained eighty-four non-vendor manifests, twenty root-workspace members and fifty-three lockfiles. Historical ELF/UEFI builds and native component gates were recognised, while the exact audit head was not newly built. Tools missing from that specialist's PATH described that local execution surface, not a universal inability to build the project. Host models, source presence, target builds and executed native components remained distinct.

<a id="audit-03"></a>

## AUDIT-03 — Reproducibility and evidence binding

Original: [03-verification.md](subagents/03-verification.md).

The manifest check reported 102 manifests and 1,814 entries: 1,788 matches, twenty-five missing files and one mismatch. Fifteen host-harness tests passed. The archive verification failed, and current integrated/native release acceptance was not established. That archive result did not turn every historical native run into a runtime failure, nor did intact subsets validate the missing bytes. Missing original logs were to remain explicitly missing rather than be reconstructed or replaced with newly generated runs presented as old evidence.

<a id="audit-04"></a>

## AUDIT-04 — Defensive security review

Original: [04-security.md](subagents/04-security.md).

The report recorded a legacy recovery-export authorisation problem on an ordinary domain ingress in the examined acceptance profile. Its reported classification was HIGH, confidence 9/10, E3 source evidence. No exploit was executed. Production provisioning/reachability outside the observed profile was unknown. The original recommended a bounded authorisation repair and native denial/regression evidence; the audit itself repaired no code. A general security helper review was disclosed alongside the profile-based work. This entrypoint supplies no new exploit procedure and does not decide the finding's current disposition.

<a id="audit-05"></a>

## AUDIT-05 — Product maturity and functional problem solving

Original: [05-maturity.md](subagents/05-maturity.md).

Real bounded native results were recognised, while all sixty-two full release requirements remained open on the examined source. Task-oriented comparison covered eight reference operating-system families without asserting newly verified external feature parity. A targeted source correction recognised existing TEST-key rotation/revocation evidence; production custody, firmware trust and the complete user update/recovery route remained separate. Product scoring, release acceptance and security confidence were not interchangeable percentages.

<a id="audit-06"></a>

## AUDIT-06 — Data flows and user control

Original: [06-privacy.md](subagents/06-privacy.md).

The specialist examined fourteen data flows. A complete private native client/NAS/provider/key chain was not demonstrated, and local encryption, retention, rights visibility and AI boundaries lacked product-wide acceptance. The coordinating report resolved older document drift in favour of the leading privacy revision permitting explicitly selected opaque provider storage. It did not modify the contract or infer a complete encrypted/synchronised product from a narrower opaque-storage or cold/replay probe.

<a id="audit-07"></a>

## AUDIT-07 — Technical European compliance gaps

Original: [07-eu-compliance.md](subagents/07-eu-compliance.md).

Thirteen technical gaps were recorded with conditional applicability. Product/market role, support and security response, SBOM/licensing and accessibility acceptance required further work. Current legal sources and applicability were not independently verified. These were technical questions for later qualified review, with no compliance certificate, legal conclusion or inference that a CI result established conformity.

<a id="audit-08"></a>

## AUDIT-08 — Release and software supply chain

Original: [08-release-supply-chain.md](subagents/08-release-supply-chain.md).

Historical eighteen-stage TEST-policy gates and fifty-six hashes were recognised. Production trust anchors, the firmware/loader trust chain, a hermetic independent double build, complete SBOM/licensing and operational support were not established for distribution. Public TEST anchors remained deliberately public test material. An integrated signed graphical image, trial-boot/recovery behaviour and controlled production signing/custody required their own evidence; component signature tests did not complete release authority.

<a id="audit-09"></a>

## AUDIT-09 — Hardware, drivers and power

Original: [09-hardware-drivers.md](subagents/09-hardware-drivers.md).

The report bounded evidence to fixed QEMU/q35/device-address/single-core profiles. Pinning or quarantine did not establish IOMMU containment. Host ACPI parsing did not establish native poweroff, and no named physical support profile/E7 result was supplied. A review correction distinguished a packet-driver probe from later integrated native network evidence and retained the latter's narrower acceptance. Physical NIC, DMA/IOMMU and broad power/device support remained separate.

<a id="audit-10"></a>

## AUDIT-10 — User experience and accessibility

Original: [10-ux-accessibility.md](subagents/10-ux-accessibility.md).

Twelve task paths were examined. Native U01/compositor/clipboard evidence was retained, while personal installation/login/lock/recovery and a complete assistive-technology chain were not established. Concrete source-level observations concerned F6/Alt+Tab, status semantics and input limits. App-local semantics, screenshots and browser checks did not establish a complete native screen-reader or human usability result. The report's original UI literals and fixture-specific observations remain in the source.

<a id="audit-11"></a>

## AUDIT-11 — Reliability, data preservation and recovery

Original: [11-reliability-recovery.md](subagents/11-reliability-recovery.md).

The specialist raised the early boot-floor/trial-health sequencing risk and noted that service failures beyond the demonstrated compositor route were terminal in the examined profile. Independent native backup/restore, durable key write-ahead and a persistent synchronisation queue remained open. Thirty-five synthetic harness tests and eighty-five historical hashes passed; no VM recovery was repeated. The coordinating report merged duplicate descriptions of the boot-floor concern into one risk and retained its static, unreproduced evidence boundary.

<a id="audit-12"></a>

## AUDIT-12 — App platform, ABI and SDK

Original: [12-app-platform-sdk.md](subagents/12-app-platform-sdk.md).

Package verification, SDK interfaces and capabilities existed, while durable issuer trust/store/catalogue and a general verified-image-to-authorised-process lifecycle remained incomplete. ABI, feature/bootstrap and documentation drift were recorded. Successful verification did not itself grant launch authority, and a manifest did not itself confer capabilities. Installation, update, removal, defaults, grants and compatibility required an integrated native lifecycle rather than only host models or fixed example apps.

<a id="audit-13"></a>

## AUDIT-13 — Defensive threat model

Original: [13-red-team.md](subagents/13-red-team.md).

Eight defensive AND/OR threat trees described hypotheses and safe test proposals. The specialist executed no exploit and confirmed no additional exploitable vulnerability. The coordinating report kept those hypotheses separate from the concrete security source finding and from ordinary missing product functions. Detailed trees remain in the original; this English entrypoint communicates their status and review purpose without treating repeated hypotheses as independent evidence.

<a id="audit-14"></a>

## AUDIT-14 — Performance and energy

Original: [14-performance-energy.md](subagents/14-performance-energy.md).

The historical CPU-median reduction was independently recalculated as `88.2600281961856578%` from six WHPX runs, three per arm. The small discrepancy with a rounded published value was rounding, not a new measurement. A complete latency workload, p95/frame pacing, RAM/I/O/duration evidence and physical energy/thermal acceptance remained absent. Virtual CPU use and a diagnostic microbenchmark did not establish physical watt/joule savings or full product responsiveness.

<a id="audit-15"></a>

## AUDIT-15 — Networking and distributed systems

Original: [15-network-distributed.md](subagents/15-network-distributed.md).

Historical native ARP/UDP/DNS/TCP and NAS cold/replay evidence was recognised. Native TLS/trusted time, peer continuity, durable queues, dynamic policy, complete resolution and provider integration remained open. Host TLS research and native adapter source represented different evidence levels. The coordinating report clarified that `NET_CACHE_INTACT` referred to the ARP neighbour cache; a native DNS query/TTL check did not establish a complete resolver/cache-expiry product.

<a id="audit-16"></a>

## AUDIT-16 — Documentation and evidence claims

Original: [16-claims.md](subagents/16-claims.md).

The claims review confirmed the known twenty-five missing archive files and traced the RTC README mismatch to ADR renumbering. Some older documentation underreported later native progress. It preserved the distinction between twenty-one accepted rubric criteria/forty-one points and sixty-two open full-release requirements. Source citations, manifest integrity, historical execution and current product acceptance each retained their own bounds. The original author-hash table applied to the earlier report integration, not to later corrected report bytes.

## Combined limits and use of the records

The coordinator reconciled overlapping findings instead of counting their frequency as extra evidence. Current report bytes retain later citation/scope corrections, so the original integration hash table is not presented as their present checksum. Original local logs and private integration paths were not republished; some were session artifacts rather than a promised permanent CI archive.

The proposed next work linked authorisation, identity/key continuity, boot health/recovery, durable encrypted storage/queues, native peer authority, app lifecycle and assistive technology before final source/image/workload and physical/human qualification. Those were recorded recommendations, with no new implementation authority or competing roadmap created by this guide. No product gate, score, issue, finding or historical test result is accepted, closed or re-executed by the translation.
