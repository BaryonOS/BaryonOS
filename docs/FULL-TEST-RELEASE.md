<a id="nieuw-os--volledige-testrelease"></a>
# Nieuw OS — full test release

**Status: in progress; full product acceptance remains open.** This is the
binding release assignment and acceptance register. A documentation checkpoint,
an individual milestone or a passing partial VM test does not constitute delivery.

<a id="doel-bron-en-voortgang"></a>
## Objective, source and progress

The recorded `/goal`, as stated by the project owner, is:

> Realiseer de volledige native testrelease van Nieuw OS volgens docs/FULL-TEST-RELEASE.md. Blijf implementeren, integreren, testen en repareren tot alle verplichte functionele, visuele, beveiligings- en herstelcriteria aantoonbaar slagen op het geleverde releaseimage. Behoud de volledige productomvang en architectuur; vervang ontbrekende functies niet door mocks, verborgen bediening of uitstel. Gebruik beschikbare agents voor onafhankelijk werk. Bewaar bij een harde blokkade of limiet de exacte werkstand en hervatstap. Meld voltooiing uitsluitend met het complete testpakket en herleidbaar acceptatiebewijs.

**English translation of the project owner's recorded `/goal`:**

> Deliver the full native test release of Nieuw OS under docs/FULL-TEST-RELEASE.md. Continue implementation, integration, testing and repair until all mandatory functional, visual, security and recovery criteria demonstrably pass on the delivered release image. Preserve the full product scope and architecture; do not replace missing functions with mocks, hidden controls or deferral. Use available agents for independent work. At a hard blocker or limit, preserve the exact work state and resume step. Report completion only with the complete test package and traceable acceptance evidence.

The original assignment is preserved **in full** at the end of this document,
followed by a clearly labelled English translation. The supplementary instruction
of 30 October 2026 is incorporated normatively in the
[execution and product context](FULL-TEST-RELEASE-CONTEXT.md), including concrete
user tasks and compatibility limits, format coverage, TCB and endurance acceptance.
It preserves this full assignment, stable IDs, `/goal` and progress rubric.
The supplementary requirement states: **“externe opslag van uitsluitend
versleutelde objecten moet kunnen”**. **English translation:** “external storage
of encrypted objects only must be possible”. An explicitly selected external
provider therefore remains permitted as an untrusted storage layer solely for
client-side encrypted opaque objects under
[privacy contract revision 4](LOCAL-PRIVACY-CONTRACT.md) and the
[encrypted-object contract](implementation/ENCRYPTED-OBJECT-STORAGE.md).
No content keys, plaintext, mandatory vendor cloud, silent fallback,
provider key escrow, OS telemetry or cloud inference. This clarification
leaves the rest of the assignment unchanged.

There is one progress chain; the files have distinct responsibilities:

| Source | Responsibility |
|---|---|
| This document | Binding full assignment, stable product acceptance and sign-off rules |
| [progress.json](progress.json) and [PROGRESS.md](PROGRESS.md) | Existing fixed weighted development rubric; no replacement release score |
| [PROJECT-STATE.md](PROJECT-STATE.md) | Current source, artifact, executed tests, blockers and next resume step |
| [WORK-QUEUE.md](WORK-QUEUE.md) | Executable tasks, current owners, exclusive write sets, contracts and acceptance IDs |
| [ROADMAP.md](ROADMAP.md) | Existing technical/product route and separately recorded research |

The lead agent maintains these links and the shared progress files.
Tasks use the IDs below and the existing rubric IDs; they do not introduce
a competing roadmap. The recorded starting score is **32/100, rubric v1**.
That score remains unchanged without new criterion evidence and does not mean
that 32% of this release has already been accepted on one image. Historical
native results remain bound to their own source version. All product rows below
start open; existing work is reused and reassessed on the final combination.

<a id="gecontroleerde-ontwikkelbasis"></a>
### Checked development baseline

| Check | Finding and evidence limit |
|---|---|
| Starting source | Branch `work`, commit `73ebc1e169b893eb5aba4b4f8ac06d7798136de3`; clean worktree at the repository check of 29 October 2026 |
| Rubric check | `python3 tools/progress.py --check`: exit 0, 32/100, rubric v1 consistent; checks calculation/references/displays, no OS execution |
| Historical K05 baseline | Exact tested source `56b576b16da1e4503bc5f9167a476474a58c33cf`, five native cases; [K05 evidence](verification/k05-startup-slots/README.md) |
| Current repeat | The integrator reports five repeated K05 VM runs and 238 root Rust host tests on source `73ebc1e`, exit 0. Archive: `docs/verification/full-release-baseline/supervisor-gate.json` and `git-build-audit.json`; complete raw host logs were not archived again in full, and some output exists only in the tool transcript. This closes no product row; no combined desktop/storage/network image has been demonstrated. The integrator records the exact current command logs and artifact binding. |
| Key open items | Combined native profile, B02 persistent authenticated storage, full native UI/apps/network/NAS/AI/updates and integrated release validation; physical/human-audit gates remain open at their required evidence level |

<a id="profielen-grenzen-en-bewijs"></a>
## Profiles, boundaries and evidence

`VM-B0` is solely the existing narrow regression baseline: x86_64,
`pc-q35-8.2`, `qemu64`, TCG, one CPU, 256 MiB, UEFI/OVMF 4M, the project's own
loader/kernel, and modern virtio-blk and virtio-rng where the test requires them.
The integrator records the actual QEMU/OVMF versions and hashes;
the historical baseline used QEMU 10.0.13 and OVMF 2025.02-8+deb13u1.

`VM-FR` is the planned integrated functional release profile:
`pc-q35-8.2`, `Nehalem-v1`, one vCPU/TCG, 8 GiB RAM, a dedicated sparse
32 GiB disk, GOP 1280×720 plus layout checks at 1920×1080 and 200% scale,
PS/2 input and virtio-blk/rng/net/snd. TPM 2.0 CRB uses separate
swTPM state. QEMU 10.0.13 and suitable OVMF 4M are available in the current
development environment; this does not claim that all planned devices
or components already work natively. `VM-AI` uses this profile or a suitable
extension specified in advance with an explicit model/resource budget; the
actual model and required resources remain a visible gate.
The exact configuration and hashes are frozen before acceptance runs.
A profile name alone is not a support claim. Existing small
probe limits are not product limits. The concrete supported
file-format/capacity register and performance limits are fixed before the
corresponding implementation gate; missing values block sign-off and
must not conceal a reduced product scope.

`HW-FR` and `HW-AI` are separate physical profiles. Hardware identity,
components, supported functions, verified image, instructions and
recovery route must be concrete. Disk writes, firmware changes or changes
to boot configuration on the physical device have not yet been authorized.
This does not block independent VM/source work. An existing
mandatory physical or human gate remains open; it does not become `N/A`
because access, equipment or external review is unavailable.

All VM disks are purpose-built disposable images under the designated build
and run paths. No personal disk, real NAS content, account or physical
device may be attached without specific authorization. Network/NAS peers are
bounded protocol-conforming test servers with synthetic data. Host tools
may build, observe and emulate; they do not perform missing OS client,
cryptographic, storage, UI or inference functions on behalf of Nieuw OS.

<a id="aftekenmodel"></a>
### Sign-off model

Every signed-off row receives a compact report under `docs/verification/`
with its matrix ID, exact source commit and source fingerprint, configuration,
tool versions, exact commands, exit status, log/artifact paths and SHA-256 of
the delivered loader/kernel/images. Evidence captured before a relevant change
is not relabelled as evidence for the new source. A failed run remains
archived as failed; a timeout counts as a pass only when that exact negative
scenario is demonstrably expected and checked.

Evidence codes in the matrix are cumulative:

| Code | Required evidence |
|---|---|
| N | Actual native execution through the project's own kernel/services on the integrated image; observable action plus normal and negative assertions, with the source/hash binding above |
| D | Independent inspection of actual stored bytes, a cold new VM after a complete stop and, where relevant, a crash/interruption test; a memory snapshot does not demonstrate durability |
| V | Actually rendered interface and an interaction test; screenshots/video support the assertions and do not replace interaction |
| T | Traffic/ciphertext at the agreed observation point, identity/permissions and negative protocol cases; an external test server serves only as a peer |
| P | Measurements using a workload specified in advance, repetitions, p50/p95 or the applicable distribution, memory/CPU/latency/energy limits and failure/endurance criteria |
| H | Actual execution on the exact physical profile, with a component list and recovery route; VM results remain separate |
| A | Bounded human/user/external-expert checks where existing acceptance requires them; AI review remains separate and is no substitute |
| X | Complete verifiable delivery artifact, reproducible procedure and verification from another clean environment where required |

States: `open`, `in uitvoering` (in progress), `geblokkeerd` (blocked),
`afgetekend` (signed off). `afgetekend` requires all specified evidence.
Denied permission or an unavailable supported device is a normal runtime
condition only if the corresponding implementation actually exists.
Missing implementation remains an open release requirement. Each owner named
below denotes a responsible workstream, not an already active agent. The
current person/agent and exclusive files appear only in the work queue.

<a id="actuele-deelbewijzen-behouden-de-volledige-acceptatie"></a>
## Current partial evidence preserves full acceptance requirements

All 62 mandatory rows remain open. FR-B01/FR-O05/FR-N03 have partial native RANDOM evidence: four real cases on 2b5 and 9aa, all four rerun in CI on `844ec40`. Firmware, ELFs, disk bytes and canaries were checked separately. This does not establish persistent keys, full TLS or VM-FR sign-off.

FR-U01/FR-A02 show real Materiaal and editor frames in GUI source `5e126b7`; the fixed 200-second chain still fails before Saved/Reopen/afsluiten (shutdown). A host optimization on `1fd879d` proves slower natively; variant selection therefore follows an actual measurement on the project's own kernel. No security check or acceptance boundary is removed.

FR-A03 has a passing native PNG/JPEG decoder test on `844ec40` after a targeted repair of the actual JPEG stack-guard failure. Stack, heap and pixel oracles are preserved; independent artifact verification passes. This does not sign off an image viewer, full format coverage or native OOM/recovery. All nine CI workflows on that exact source pass, including five mandatory native foundation steps. On the later combined source `9596abc`, all nine workflows with the five real foundation steps pass again; the separate GUI chain still fails after 20.000 PIT ticks (twenty thousand) with two actually presented frames. [Combined CI](verification/full-release-ci-9596abc/README.md) and [failing GUI output](verification/full-release-gui-9596abc/README.md). [Decoder evidence](verification/full-release-images-844ec40/README.md) and [CI evidence](verification/full-release-ci-844ec40/README.md).

The full assignment, stable IDs, required profiles and evidence levels remain unchanged. Exact current execution is recorded at the top of PROJECT-STATE and in WORK-QUEUE; a restricted fixture profile does not replace VM-FR.

<a id="acceptatiematrix"></a>
## Acceptance matrix

Work packages in the current queue use the `TASK-` prefix and explicitly
reference the acceptance IDs below. Older development contracts and
test logs sometimes call these work packages `FR-I01`, `FR-A03` or `FR-B02`;
these are not sign-offs for matrix rows of the same name. `FR-A03` therefore
remains the image viewer and `FR-B02` boot/image authenticity. A document adapter
or record codec cannot close these criteria.

IDs `FR-<domein><nummer>` remain stable during replanning. Splitting adds new
IDs and preserves the original requirement; no row may be removed, hidden
or marked optional to obtain a pass. Every reference in the
last column points to the existing register in [progress.json](progress.json).
`nieuw` (new) for an interface means that its native contract specification
remains open, not that a service already exists.

<a id="kern-drivers-sessie-en-dagelijkse-bediening"></a>
### Kernel, drivers, session and daily interaction

| ID / user action | Native components / interfaces | Normal + failure scenarios | Profile | Evidence | Dependencies / blockers | Owner | Status | Rubric |
|---|---|---|---|---|---|---|---|---|
| FR-K01 — start the project's own OS | Project UEFI loader, BootInfo, Rust no_std kernel | Normal handover; missing/corrupt kernel, panic and bounded boot timeout | VM-FR; HW-FR | N,H | Integrated boot chain and final image binding | kernel | open | K01,B04 |
| FR-K02 — use multiple apps without mutual damage | Paging/W^X, scheduler, ring 3, generic process/service management | App start/exit/restart; app fault, parent exit, orphan/cleanup, resource exhaustion; other apps remain operational | VM-FR; HW-FR | N,P,H | Combine profiles; SMP for required multicore support | kernel | open | K02,K03,K05,K06,K07 |
| FR-K03 — grant, deny and revoke task permissions | Capability engine, Message320 IPC, usercopy, generations/epochs | Valid restricted transfer; stale handles, incorrect types/permissions/lengths/pointers and future actions after revocation denied | VM-FR; HW-FR | N,H | Connect all product services; no permission expansion | kernel/security | open | K04,A03,B06 |
| FR-H01 — use keyboard, mouse and supported USB devices | Input drivers, device broker, native events to session | Keys/pointer/hotplug; unplug, malformed device response, failure/restart and bounded buffers | VM-FR; HW-FR | N,V,P,H | Native input chain, named physical components | drivers | open | H02,U02 |
| FR-H02 — see graphics and hear audio | Framebuffer/GPU/audio driver, isolated compositor/audio broker | Rendering/audio under load; device loss, invalid buffers, clipping/underrun, driver/compositor recovery | VM-FR; HW-FR | N,V,P,H | Native graphics/audio, latency budgets and physical measurement | drivers/session | open | H03,U01 |
| FR-H03 — use agreed connections and devices | Network driver, Ethernet/wifi and agreed Bluetooth | Connect/disconnect/resume; hotplug, driver error and missing component with the actual reason | VM-FR; HW-FR | N,T,P,H | Concrete hardware matrix and driver support | drivers/network | open | H04,H06 |
| FR-S01 — complete first use and configure a local profile | Native onboarding, profile/session service, keyring, settings storage | Choose language/interaction/profile/storage and start locally; back/cancel, error and interrupted setup without a partial profile | VM-FR; HW-FR | N,D,V,H | Native UI, key/profile persistence; no mandatory vendor account | session | open | U01,U05,B01 |
| FR-S02 — lock and unlock the session | Session broker, authentication, keyring/surface boundaries | Repeated lock/unlock; incorrect secret, aborted attempt, input race and access from locked apps | VM-FR; HW-FR | N,V,H | Actual authentication/key boundary, no decorative lock screen | session/security | open | U01,U05,B02 |
| FR-S03 — change settings and recover them after restart | Settings service, user scope, volume/keyring | Change/reset; invalid value, full storage, I/O error and cold restart without false confirmation | VM-FR; HW-FR | N,D,V,H | FR-O01–O05, profile/device settings | session | open | U05,S08 |
| FR-S04 — restart or shut down with work open | Session broker, app lifecycle, storage flush, ACPI/power | Preserve saved work; handle save/discard/cancel for unsaved work, an unresponsive app and flush failure understandably | VM-FR; HW-FR | N,D,V,H | Native shutdown/restart path and app save contract | session/drivers | open | H05,U05,R01 |
| FR-S05 — resume and use power management according to the profile | ACPI/power policy, device/session recovery | Supported sleep/wake, battery/thermal behavior; interruption, timeout and recovery without loss or revived permissions | HW-FR; VM-FR where applicable | N,D,P,H | Exact physical power profile and authorization; no extrapolation from VM to hardware | drivers | open | H05,R02 |
| FR-U01 — navigate Materiaal, Bewerken, Gereedschap, Toegang, Apparaten and Assistent | Native compositor/session, surfaces and bounded app/service endpoints | All agreed routes back/forward; app switching, focus recovery, compositor crash and missing content | VM-FR; HW-FR | N,V,H | Port all screens natively; browser prototype is only a design source | user environment | open | U01,U02,U03,U05 |
| FR-U02 — select, copy/paste, use context menus and drag | Focus/selection model, clipboard/drag broker, task grants | Keyboard/mouse, multiple selection where agreed; cancelled drag, invalid destination and denied cross-app data | VM-FR; HW-FR | N,V,H | Native input, secure data transfer and product specification | user environment | open | U02,A03,B06 |
| FR-U03 — understand progress, cancellation and errors | Shared native component/status model, asynchronous operations | Actual loading/empty/success states; cancel, timeout, stale response, retry and recovery without fake status | VM-FR; HW-FR | N,V,H | Working service operations behind every control | user environment | open | U02,U05,U06 |
| FR-U04 — use a polished, consistent interface | Maintainable design tokens, typography/icons/spacing/motion and app components | All states; hover/focus/selection/disabled, long text, window size, scaling; no unfinished screen or placeholder for requirements | VM-FR; HW-FR | N,V,A,H | Native rendering chain; visually assess and operate all apps | design/UI | open | U06,A04 |
| FR-U05 — interact accessibly | Native accessibility tree/events, text/focus, scale/zoom, contrast/reduced-motion | Keyboard-only, screen reader, readability, 200%/agreed scales; prevent focus traps/truncated labels/lost status, reduced motion and information without color alone | VM-FR; HW-FR | N,V,A,H | Native assistive tools and real user tasks; web WCAG test is not native evidence | design/UI | open | U04,U06 |

<a id="basisprogrammas-en-appbeheer"></a>
### Basic applications and app management

| ID / user action | Native components / interfaces | Normal + failure scenarios | Profile | Evidence | Dependencies / blockers | Owner | Status | Rubric |
|---|---|---|---|---|---|---|---|---|
| FR-A01 — create, find and organize material/files | Native Materiaal app, catalog/volume client, search/metadata service | Create/open/search/organize/rename/move; no results, name conflict, revoked permission, stale revision and full storage | VM-FR; HW-FR | N,D,V,H | FR-O01–O07, native search index and usable capacity | apps/storage | open | U03,U05,A04,S06 |
| FR-A02 — write and edit text | Native text editor, document-session, storage/clipboard portal | New/open/edit/select/undo/save/reopen; large UTF-8 input, externally changed revision, crash, closing unsaved work and I/O error | VM-FR; HW-FR | N,D,V,H | Editor and concrete format/size matrix; no prefilled screen | apps | open | A04,U03,S05 |
| FR-A03 — view images | Native image app, bounded decoder, object/render broker | Open supported real images, zoom/pan/scale; damaged, oversized, unsupported format and denied object | VM-FR; HW-FR | N,V,P,H | Fixed formats/decoders, dependency/license review | apps | open | A04,H03 |
| FR-A04 — read PDF | Native PDF app/maintained engine, object/render broker | Actual multipage documents, navigation/zoom/search where agreed; corrupt/protected/oversized document and parser/resource limit | VM-FR; HW-FR | N,V,P,H | PDF engine port, feature/format matrix and sandbox | apps | open | A04,U03 |
| FR-A05 — browse and download within granted permissions | Native browser/maintained engine, network/TLS/DNS, download portal | Navigate, link/form, own test site and download/open; DNS/timeout/TLS error, denied destination/file action and revocation | VM-FR; HW-FR | N,D,V,T,H | Chromium/Gecko comparison before engine decision, engine port, FR-N01–N04, app/object permissions | apps/network | open | A04,N01,N02,N03,N04,A03 |
| FR-A06 — play audio | Native player, codec, audio broker and object client | Play/pause/seek/volume for real agreed common files; corrupt/unsupported, end, device loss and resource limit | VM-FR; HW-FR | N,V,P,H | Audio/codec port and concrete formats; measured audible output | apps/drivers | open | A04,H03 |
| FR-A07 — play video | Native player, demux/codec, render/audio broker | Real agreed video with picture/sound synchronization, pause/seek; corrupt/unsupported, dropped frames and CPU/memory pressure | VM-FR; HW-FR | N,V,P,H | Video/audio/render chain and concrete codec profiles | apps/drivers | open | A04,H03,R02 |
| FR-A08 — calculate with the calculator | Native app, bounded parser/calculation core | Actual input, editing and results; division by zero, large values, invalid expression and input recovery | VM-FR; HW-FR | N,V,H | Native app and documented calculation/precision limits | apps | open | A04 |
| FR-A09 — create and save a screenshot | Capture broker, compositor, object/save portal | Capture screen/agreed selection, save and reopen; cancel, denied capture, full storage and lock-screen protection | VM-FR; HW-FR | N,D,V,H | Screenshot is the selected permitted minimum capture function; native capture boundary | apps/UI | open | A04,A03,U01 |
| FR-A10 — view system information, diagnostics and Dutch help | Native diagnostic/help app, scoped status/log API | Actual configuration/version/resource data, error explanation/log export; missing device, redactions, denied privileged data and offline help | VM-FR; HW-FR | N,V,X,H | Actual measurement sources, no hardcoded system status or secrets in logs | apps/release | open | A04,U05,R03 |
| FR-A11 — install an app from agreed sources | Package service, manifest/signature, app image loader, capability policy | Install a valid package and actually execute it; invalid signature/hash/manifest, full disk, interruption and denied installation source | VM-FR; HW-FR | N,D,V,T,H | Package format, multiple sources, no unrestricted install scripts or automatic grants | app management/security | open | A01,A02,B04,B06 |
| FR-A12 — update and remove apps | Package transaction/version management, process/object permissions | Update/remove/reopen where applicable; running app, interruption, invalid package, preservation/deletion behavior and revoked old authority | VM-FR; HW-FR | N,D,V,H | FR-A11, storage transactions and explicit user-data contract | app management | open | A02,A03 |
| FR-A13 — choose default tools and use coherent open/save behavior | MIME/format register, open/save portal, SDK, task broker | File to suitable app, preserve default and limited grants; unknown format, removed default, deny/revoke and malicious helper | VM-FR; HW-FR | N,D,V,H | Concrete formats and portals; signing establishes provenance, not permission | app management/UI | open | A01,A03,A04,B06 |
| FR-A14 — create and edit tabular material | Native table tool under P3/P4, document/object model and shared UI/save broker | Create/open/edit/save/reopen a realistic table; invalid input, large table, conflict and I/O error; test agreed import/export per feature | VM-FR; HW-FR | N,D,V,P,H | Existing PRODUCT-SPEC productivity model; concrete table/format features remain visibly open until specified/demonstrated | apps | open | A04,U03,S05,U06 |
| FR-A15 — create and edit presentation material | Native presentation tool under P3/P4, object/render/save broker | Create/open/edit/save/display a realistic presentation; damaged content, scaling/long text, conflict and I/O error; test agreed import/export per feature | VM-FR; HW-FR | N,D,V,P,H | Existing PRODUCT-SPEC productivity model; no unsupported PPTX round-trip claim | apps | open | A04,U03,S05,U06 |

<a id="opslag-netwerk-nas-en-lokale-ai"></a>
### Storage, networking, NAS and local AI

| ID / user action | Native components / interfaces | Normal + failure scenarios | Profile | Evidence | Dependencies / blockers | Owner | Status | Rubric |
|---|---|---|---|---|---|---|---|---|
| FR-O01 — save and recover data after a cold restart | Virtio block broker, volume/catalog, READ/WRITE/FLUSH, document-session | Read confirmed writes in a new VM; kill before/during/after commit and no loss of confirmed data within the contract | VM-FR; HW-FR | N,D,H | Combined native profile, independent disk oracle and real power path | storage | open | S01,S02,S03,S07 |
| FR-O02 — work with usable volumes and documents | Volume allocator/compaction, streams, quotas and bounded buffers | Normal prolonged use/reuse/large input; Full/Quota/partial write/overflow and interrupted compaction | VM-FR; HW-FR | N,D,P,H | Responsibly expand probe caps; fix exact product scope/limits | storage | open | S04,S05,K07 |
| FR-O03 — organize metadata and use history/recovery | NOSVOL02/IPC v6 or versioned successor, collections/revisions | Change names/collections/metadata/revisions and restore as a new revision; conflict, delete/recover and interrupted transaction | VM-FR; HW-FR | N,D,V,H | Native app integration, recovery policy and index consistency | storage/UI | open | S03,S06,U03 |
| FR-O04 — recover from storage errors | Supervisor, block/volume service, epochs/reset/quarantine | Service restart/corruption/timeout/disk failure; Unknown still receives no ACK, no write replay, no silent reinitialization and other apps remain responsive | VM-FR; HW-FR | N,D,V,P,H | FR-I01, requester-crash/reset-without-ACK evidence, general recovery path | storage/kernel | open | S07,S08,K05,K07 |
| FR-O05 — protect content, metadata and keys | Keyring/entropy, authenticated persistent volume/object layer | Persistent seal/unseal/recovery; tampering/tag/header/replay/rollback under the threat model, key loss/revocation and cold reopen | VM-FR; HW-FR | N,D,H,A | B02 open; no invented cryptography, explicit cold anti-rollback | security/storage | open | B01,B02,B06 |
| FR-O06 — import/export and create recovery copies | Format adapters, export/backup portal, independent copy | Import/export/restore real supported files; corrupt/incompatible, name conflict, full destination, interrupted transfer and source loss | VM-FR; HW-FR | N,D,V,X,H | Locate/reuse S09 source first; concrete formats/compatibility limits | storage/apps | open | S09,A04,R01 |
| FR-O07 — search local content and metadata | Scoped local index/search service, object/revision identity | Find new/updated/deleted content; index crash/rebuild, deny/revoke, no results from other domains and no index leak | VM-FR; HW-FR | N,D,V,P,H | Index persistence, actual queries and access boundary | storage/UI | open | U05,S06,B06 |
| FR-O08 — initialize new storage safely | Volume detection/initialization, format/version marker, explicit recovery choice | Use a newly designated empty test volume; reject or migrate unknown/nonempty/corrupt/older volumes in a controlled way; interruption preserves previously confirmed data | VM-FR; HW-FR | N,D,V,H | Never automatically overwrite an unrecognized existing volume; physical target only after specific authorization | storage/release | open | S02,S04,S08,R01 |
| FR-N01 — configure networking and connect | Native network driver/service, IP/TCP/UDP, address/route management | Real packets/configuration/reconnect; malformed packets, backpressure, link loss, timeout and bounded resources | VM-FR; HW-FR | N,T,P,H | Native stack and named device profiles | network | open | N01,N02,H04 |
| FR-N02 — use name resolution | Native DNS/resolver/network service, process destination permissions | Valid name/response/cache; NXDOMAIN, timeout, malformed/truncated, stale response and denial | VM-FR; HW-FR | N,T,H | FR-N01, actual protocol-conforming test resolver | network | open | N02,N04 |
| FR-N03 — use authenticated secure transport | Maintained TLS/cryptography, keyring/entropy, hybrid PQC contract | Authentic peer/hostname/ALPN; expired/incorrect certificate, mismatch, malformed handshake, no silent classical/insecure fallback | VM-FR; HW-FR | N,T,P,H,A | Native cryptography/TLS port; host-N02 is preparatory evidence | network/security | open | N03,B01,B04 |
| FR-N04 — allow and revoke networking per app/destination | Kernel capabilities, network broker, grants/generations | Browser may access the intended destination; deny/revoke on active/new connections, helper route and unexpected listening port | VM-FR; HW-FR | N,T,H | Socket/destination enforcement; LP-08–10/23/30/31 | network/security | open | N04,A03,B06 |
| FR-P01 — pair a personal or shared NAS | Native client/NAS role, identity verification, transport and invitation | Controlled pairing/receiving/offering; incorrect peer, old/reused/expired invite and separate LAN/internet permission | VM-FR with test NAS; HW-FR | N,V,T,H | FR-N01–N04, native key lifecycle; LP-02/03/19/24 | NAS | open | P01,P04,B06 |
| FR-P02 — store encrypted data on NAS and work offline | Client-side encrypted objects, keyring, local volume/sync layer | Names/content encrypted before transmission; NAS state+admin credentials yield no plaintext/keys, outage/timeout preserves local usability | VM-FR with test NAS; HW-FR | N,D,V,T,H,A | FR-O05, no host client or NAS inference; LP-01/14/15/20/22 | NAS/security | open | P01,B02,B06 |
| FR-P03 — synchronize, resume and resolve conflicts | Native sync engine, revisions, tickets and peer-bound authority | Offline edits, reconnect/LAN-internet transition; concurrent writers, partial upload, conflict, crash and no silent data loss | VM-FR, multiple test clients; HW-FR | N,D,V,T,P,H | Persistent sync/recovery keys and bounded queues | NAS/storage | open | P02,P03,S09 |
| FR-P04 — invite, share, manage quotas and reclaim space | Separate NAS admin capability, quotas/reservations, tombstones | Offer/receive/share/revoke/evict space; full quota, abort/retry, offline/refusing guest, old session/upload after restart cannot revive access | VM-FR with test NAS; HW-FR | N,D,V,T,H | Administrator can erase their own copy without the user's key/consent; LP-16–18/21/25 | NAS | open | P04,P01,B06 |
| FR-P05 — add devices and rotate/recover user keys | Device identity, keyring, authenticated multi-device recovery | Newly authorized device and rotation; revocation, lost key, concurrent writers and no provider key escrow | VM-FR, multiple test clients; HW-FR | N,D,V,T,H,A | Threat model/rollback limits and independent recovery medium | NAS/security | open | P03,B01,B02 |
| FR-P06 — choose optional external ciphertext storage | Provider-independent native encrypted-object adapter, provider namespace/credentials and network broker | Selected ciphertext namespace only; credentials without content key, tampering/rollback, outage/delete and revoke without silent fallback | VM-FR with test provider; HW-FR | N,D,V,T,H,A | Explicit choice, FR-O05/N04; LP-26–28; provider is not an OS trust anchor | NAS/provider | open | P01,B02,B06,S09 |
| FR-L01 — enable/remove local AI and choose a model | Native model management, isolated downloader, pinned model/license/format | Opt-in/download/install/remove; changed model bytes, unsupported parser, no model and insufficient storage/RAM; OS remains usable | VM-AI; HW-AI | N,D,V,T,P,H | Model/engine port and profile; downloader receives no personal context/keys | AI | open | L01,A02,B06 |
| FR-L02 — use AI with explicitly selected context | Actual local inference on the profile's CPU/GPU/NPU, context broker | Agreed tasks on new input, actual processing/response; missing model/capacity and unreadable/unselected context | VM-AI; HW-AI | N,V,T,P,H | No fixed answers; networkless inference; context/output boundaries | AI | open | L01,B06 |
| FR-L03 — cancel AI and control permitted actions | Bounded job/action broker, quotas and lifecycle | Cancel during processing and scoped action; deny/revoke, memory/CPU pressure, model crash and helper/socket/upload denial | VM-AI; HW-AI | N,V,T,P,H | FR-K03, no general shell broker/cloud fallback; LP-04–07 | AI/security | open | L01,K07,A03,B06 |
| FR-L04 — retain native local quantum functionality | Existing local simulator/job interface with quotas/cancellation | Reference circuits and cancellation after integration; invalid circuit/resource limit and no external quantum cloud | VM-FR; named HW profile | N,P,H where required | Preserve existing L02; physical QPU remains separate research | quantum/integration | open | L02 |

<a id="beveiliging-updates-integrale-validatie-en-oplevering"></a>
### Security, updates, integrated validation and delivery

| ID / user action | Native components / interfaces | Normal + failure scenarios | Profile | Evidence | Dependencies / blockers | Owner | Status | Rubric |
|---|---|---|---|---|---|---|---|---|
| FR-B01 — rely on entropy and key protection | Virtio-rng/suitable physical source, broker, keyring with maintained cryptography | Healthy source/key lifecycle/recovery; stuck/no-device/time-out, monotonic revocation, requester crash and reset without ACK/quarantine | VM-FR; HW-FR | N,D,H,A | Reuse historical B01 evidence within its scope; do not invent physical RNG/TPM support | security/drivers | open | B01,B02 |
| FR-B02 — boot/verify only permitted images | Boot/signature/PQC contract, key rotation and distribution chain | Valid test release with recognizable test keys; modified kernel/image/signature, unknown key, rotation and trust failure | VM-FR; HW-FR | N,D,H,A,X | Native boot authenticity, bounded test keys; no production claim | security/boot | open | B04,B03 |
| FR-B03 — update the system and recover from failure | Native update planning/installation, slots/transactions, authenticity/integrity | Valid update and permitted previous version; corrupt/incorrectly signed, interruption at every critical phase, downgrade attack and preservation of user data | VM-FR; HW-FR | N,D,V,H,A | Update/rollback/migration contract and actual recovery boot | updates/storage | open | B03,R01 |
| FR-B04 — preserve privacy and the Security Floor | Enforcement at kernel/service boundaries, local logs, download/app grants | All applicable LP-01–31; clean installation/periodic services/privacy popup disabled without capability delta; no OS telemetry or AI fallback | VM-FR/VM-AI; HW-FR/HW-AI | N,D,V,T,H,A | All relevant native components and negative helper routes | security | open | B06,A03,N04 |
| FR-B05 — obtain independent checks of security boundaries | Kernel/service/DMA/parser/capability contracts, selected formal models | Bounded expert review and machine-checked selected invariants; fix blockers and retest, publish limitations | Exact release source/profiles | A,N,H where required,X | Human external-expert checks not yet executed; AI review is no substitute | security/release | open | B05,B04,R03 |
| FR-R01 — exercise the daily product as a whole | All product components on one final image | First start, session/apps, material CRUD/restore, cold save, browser/download/media, app management/settings, NAS/AI/permissions/update and recovery; failures from each involved row | VM-FR/VM-AI; HW-FR/HW-AI | N,D,V,T,P,H,A | All functional rows; separate feature images are not overall evidence | integrator/verifier | open | U01–U06,A02–A04,P01–P04,L01,R01 |
| FR-R02 — boot repeatedly and run extended combined sessions | Product workload, resource/performance observer with local logs | One hundred cold starts and 24 hours of combined product use on a stable final image; memory growth, CPU/IPC pressure, full storage and network failures; data integrity preserved | VM-FR/VM-AI; HW-FR/HW-AI | N,D,P,H | Fix endurance/workload/p50/p95/energy/memory/latency limits in advance; do not relax them afterwards | release/verifier | open | R02,K07,H05 |
| FR-R03 — install/recover on supported physical configurations | Verified image, installation/recovery procedure and hardware drivers | Practical test with preservation/erasure options; failed start/update and independent recovery on named components | HW-FR/HW-AI | N,D,V,H,A,X | Concrete device identity and specific permission for physical operations are missing; multiple configurations where H06 requires them | hardware/release | open | H06,R01,H02–H05 |
| FR-R04 — reproduce and assess the complete test package | Source commit, build/start/recovery tools, manual, matrix and evidence bundle | Clean build/hash verification, Dutch task manual/test plan, log/error-reporting route; missing dependency/license/SBOM and untraceable artifact block acceptance | Clean build environment + all release profiles | X,N,H,A where required | Complete package from §14, maintenance/patch policy and reproducibility | integrator/release | open | R03,A01,B04 |

The rubric link refers to existing criteria; a combined reference such as
`U01–U06` covers all existing intermediate IDs. The provider adapter from the
privacy roadmap remains explicitly visible even though the current
fixed rubric has no separate P05 point. This changes neither weights nor
acceptance obligations. Full formal verification, CHERI, additional ISAs and
physical QPU control retain their separately recorded research route;
they do not justify removing existing release requirements from this matrix.

<a id="uitvoering-en-integratie"></a>
## Execution and integration

The cycle remains research → implement → build → execute → assess
→ repair → integrate. Reuse existing kernel, service, SDK and
tool implementations; fix specific defects without restarting the overall architecture.
Libraries/engines are permitted after review of provenance, version, maintenance,
license, no_std/ABI/build scripts and sandboxing. The kernel remains
small: product policy, document formats, rendering, cryptographic policy and inference
belong in isolated services/apps. A necessary boundary change receives
rationale, a contract and negative tests before integration.

<a id="eerste-concrete-verticale-integratie-fr-i01"></a>
### First concrete vertical integration: FR-I01

**Objective:** run the generic supervisor/process path, actual block I/O and
the existing entropy/keyring source together on one profile using the project's
own kernel. This step is a dependency of FR-K02, FR-O04 and FR-B01; it completes
neither a desktop nor B02 nor the full release.

- The integrator owns the shared launch/startup-capability/syscall contracts,
  feature selection and final profile configuration. Preserve `proc_spawn` positions,
  restricted Launch/Process permissions, Message320 and existing block/entropy APIs.
- Ordinary app processes receive task endpoints, not block/entropy/MMIO/DMA
  permissions or unrestricted launch rights. The supervisor may manage only
  the intended services; do not add a general root capability to accelerate integration.
- Confirm actual block bytes/WRITE/FLUSH and real entropy completion through
  their existing brokers on the same image. Keep the client and independent
  progress processes running during service crash, timeout, deny/revoke and
  resource exhaustion. Old epochs/handles are rejected; unknown I/O receives
  no success ACK or replay. DMA pools remain pinned until reset ACK
  or enter quarantine.
- Acceptance: targeted host tests, freestanding build, actual normal and negative
  VM cases, independent disk oracle, source/artifact hashes and relevant K05,
  volume/storage-resilience and keyring regressions on the combined source.
  Explicitly label fixture injections; a test feature proves only the property
  under examination. The executable command and actual results belong in the
  current work queue/status, not as invented execution here.

<a id="parallel-werk-met-één-integratie-eigenaar"></a>
### Parallel work with one integration owner

Use all genuinely available independent agent slots productively, with
separate worktrees/build directories where needed. Only a successful agent start
counts as active capacity. Each task brief contains an ID, objective, exclusive
write files, contract version, dependencies, invariants, failure cases, acceptance
commands and delivery format. One writer per shared file and one owner per
kernel/wire/SDK interface. A reviewer begins read-only. A verifier
checks the actually integrated source and artifacts independently of
the builder; AI verification remains labelled as such.

Independent workstreams are kernel/IPC, drivers, storage/recovery, UI/design,
apps/app management, network/NAS/provider, local AI and security/update/release.
Available slots are allocated according to concrete dependencies; not every
workstream must be active simultaneously. The lead agent integrates conflicts
while preserving content, checks current Git/PR status and regressions, maintains
score/status/queue and delivers the single working product. Completing an
agent task does not constitute product sign-off.

<a id="integrale-test--en-aftekenvolgorde"></a>
### Integrated test and sign-off sequence

1. Freeze profile, file formats, capacity and measurable performance/endurance
   and recovery criteria before the relevant gates. Preserve baseline and failure cases.
2. Integrate each vertical function with normal/negative native tests;
   rerun relevant existing regressions after changes to shared boundaries.
3. Build the final image reproducibly and exercise FR-R01/FR-R02 as
   a coherent product, with cold storage verification and network observation.
4. Actually execute required human/accessibility/expert tests and separate
   authorized physical-profile tests. If authorization is missing, prepare
   the concrete package; continue independent work.
5. Fix blockers and retest on the final source. Complete every
   matrix row with source/hash-bound evidence and deliver the full package.

**The declaration “Volwaardige testrelease gereed” (full test release ready) is
permitted only when all mandatory matrix rows pass on the exact delivered
integrated image(s), all existing requirements have been demonstrated at their
prescribed evidence level, no blocking defects remain and the complete test
package is reproducible.** Existing mandatory physical/human-audit criteria remain
open until actual evidence exists; no VM-only completion or unauthorized execution
of external gates. A fully passing VM functional gate with open external gates
is an interim integrated VM validation, not a completed assignment.

<a id="oplevering-blokkades-en-hervatten"></a>
## Delivery, blockers and resumption

The test package contains integrated source code/commit; the actually tested
release image with hashes; a reproducible build; start/recovery procedures;
an accessible Dutch user manual; supported configurations;
a product-wide assessment plan; completed matrix/evidence; only
nonblocking known limitations; and a clear error-reporting/log route.
Development test keys are recognizable as test keys. Compatibility,
hardware, security and performance claims remain limited to actually
investigated properties and assumptions.

At a blocker, the relevant row remains open/blocked with its reason and
dependency. Authorization boundaries are not bypassed. Physical write tests,
firmware/boot changes, publication, paid infrastructure and external
assignments follow their specific authority boundaries. The integrator first
makes the concrete image, instructions and recovery route reviewable.

At each interruption, the current `NEXT AGENT HANDOFF` in
PROJECT-STATE preserves: exact branch/commit and local changes; last usable
artifact with hashes/configuration; open matrix/rubric IDs; executed
commands/exit statuses/logs and unexecuted tests; concrete blockers; and
one next executable resume step. On resumption, reread this document,
current status/queue/AGENTS and actual source for changes.
The assignment below and the explicit ciphertext clarification remain
substantively intact through summaries, compaction and handoff.

<a id="oorspronkelijke-opdracht--onverkort"></a>
## Original assignment — unabridged

The following quotation from the project owner contains 14 numbered sections.
The SHA-256 of the exact UTF-8 text between the markers, excluding the
surrounding marker newlines, is
`e47f7a5885fd1f45d29a1edec7883a3dbaa8fdfba97f396fd1ebda9549d1eea2`.
This makes unintended later changes to the preserved assignment detectable;
it is not a claim of external authentication or release acceptance.

<!-- FULL-RELEASE-REQUEST:BEGIN -->
ROL EN HOOFDOPDRACHT

Je bent de hoofdontwikkelaar en integratieverantwoordelijke van Nieuw OS.

Mijn opdracht is om door te ontwikkelen tot een volwaardige, samenhangende, professioneel afgewerkte testrelease van het besturingssysteem. Ik wil het complete product kunnen beoordelen en gebruiken op het vastgelegde testplatform.

Het einddoel omvat werkende systeemfuncties, een verzorgde gebruikersomgeving, bruikbare basisprogramma’s, instellingen, opslag, netwerk, lokale AI, NAS-functionaliteit, beveiliging, updates en herstel.

Een bootdemo, notitieproef, technische preview, browserprototype, verzameling losse componenten of interface met gesimuleerde functies voldoet niet.

Leg deze opdracht vast in docs/FULL-TEST-RELEASE.md. Verbind dit document met de bestaande projectstatus en werkqueue, zodat er één consistente bron voor voortgang ontstaat. Behoud de inhoud van deze opdracht bij samenvatten en hervatten.

Maak vervolgens een uitvoerbaar plan en begin direct met de uitvoering. Een plan opleveren is geen afronding van de opdracht.


1. CONTROLEER DE WERKELIJKE PROJECTSTAND

Lees de actuele AGENTS.md-bestanden, README, projectstatus, architectuurbesluiten, productspecificaties, privacycontracten, beveiligingscontracten, buildinstructies en testprocedures.

Controleer de actieve branch, commits, relevante ontwikkelbranches, open pull requests en bestaande lokale wijzigingen. Gebruik de actuele implementatie en herleidbaar testbewijs als uitgangspunt.

Oude chatclaims, voortgangspercentages, documentatie en geslaagde tests van andere commits gelden niet automatisch als bewijs voor de huidige release.

Bouw voort op bruikbare bestaande implementaties. Hergebruik en integreer goed werk. Los conflicten, dubbele implementaties en tegenstrijdige interfaces op zonder bestaand gebruikerswerk te verliezen.

Voer geen algemene architectuurherstart uit wanneer een gerichte verbetering volstaat.


2. LEG DE VOLLEDIGE ACCEPTATIE VAST

Maak een acceptatiematrix voor de volledige testrelease.

Vermeld per verplichte functie:
- Welke concrete gebruikershandeling moet werken.
- Welke native componenten en interfaces daarvoor nodig zijn.
- Welke normale en foutscenario’s worden getest.
- Op welk hardware- of VM-profiel de functie wordt ondersteund.
- Welk bewijs nodig is om de functie af te tekenen.
- Welke afhankelijkheden of blokkades nog openstaan.

Gebruik de bestaande productafspraken én de onderstaande eisen als basis. Deze opdracht vervangt het eerdere beperkte einddoel van alleen opstarten en een korte notitie opslaan.

Verlaag de productomvang niet om sneller een groen resultaat te krijgen.

Verwijder, verberg of deactiveer geen verplichte functie om de acceptatie te laten slagen. Verplaats moeilijke releasevereisten niet zelfstandig naar “later”. Als een eis nog niet uitvoerbaar is, blijft deze zichtbaar open.

Maak onderscheid tussen bestaande releasevereisten en onderzoek dat in de vastgelegde roadmap al een afzonderlijk traject heeft. Gebruik dat onderscheid niet om nieuwe uitzonderingen te creëren.


3. BEHOUD DE EIGEN ARCHITECTUUR

Nieuw OS blijft een zelfstandig besturingssysteem met een eigen kernel en eigen productervaring.

Behoud de afgesproken microkernelrichting, capabilitybeveiliging, procesisolatie, IPC-contracten en scheiding tussen kernel, drivers en systeemdiensten.

Vergroot de vertrouwde kern niet uitsluitend om integratie gemakkelijker te maken. Motiveer noodzakelijke wijzigingen en controleer de gevolgen.

Gebruik toegestane standaarden, bibliotheken, engines en ontwikkelgereedschappen wanneer die het project helpen. Controleer daarbij compatibiliteit, onderhoud, licenties en beveiligingsgrenzen. Een browserengine of andere bibliotheek hoeft niet opnieuw te worden uitgevonden als verantwoord hergebruik mogelijk is.

Vervang de eigen kernel niet door Linux, BSD, XNU, Zircon of een andere bestaande kernel.

Native functionaliteit moet via de eigen kernel en systeemdiensten werken. Hostgereedschappen mogen bouwen, testen en emuleren, maar mogen ontbrekende OS-functionaliteit niet uitvoeren alsof die al in Nieuw OS bestaat.

Testgegevens en protocolconforme testservers zijn toegestaan. Gesimuleerde succesmeldingen en hostprototypes gelden niet als implementatiebewijs.


4. LEVER EEN COMPLETE GEBRUIKERSOMGEVING

Werk Materiaal uit tot een samenhangende, native gebruikersomgeving.

Zorg voor:
- Een verzorgde opstart- en eerstegebruikservaring.
- Gebruikersprofiel, sessie, vergrendeling en ontgrendeling.
- De afgesproken schermen voor materiaal, bewerken, gereedschappen, toegang, apparaten en assistent.
- Betrouwbare navigatie, focus en bediening met toetsenbord en muis.
- Werkende selectie, contextmenu’s, klembord en slepen waar de productspecificatie dit vraagt.
- Begrijpelijke meldingen, voortgang, annulering en foutafhandeling.
- Consistente instellingen die na herstart behouden blijven.
- Correct herstarten en afsluiten, met voorafgaande afhandeling van opgeslagen en onopgeslagen werk.
- Energie- en hervatgedrag volgens de vastgelegde eisen van het ondersteunde profiel.

Iedere zichtbare bediening moet de bedoelde handeling werkelijk uitvoeren.

Voorwaardelijk onbeschikbare functies, bijvoorbeeld door ontbrekende hardware of geweigerde toestemming, moeten hun werkelijke toestand en reden tonen. Een ontbrekende implementatie mag niet als zo’n normale toestand worden vermomd.


5. MAAK VISUELE KWALITEIT EEN RELEASEVOORWAARDE

Het OS moet coherent, rustig, herkenbaar en professioneel aanvoelen.

Gebruik één samenhangend ontwerpsysteem voor typografie, kleuren, iconen, afstanden, componenten, animaties en interactietoestanden. Bewaar de gekozen eigen identiteit en de Materiaal-richting.

Ontwerp en controleer ook:
- Laden, leegte, fouten, bevestiging en herstel.
- Focus, selectie, hover en uitgeschakelde bediening.
- Lange teksten, verschillende venstergroottes en beeldschalen.
- Toetsenbordnavigatie, leesbaarheid, contrast en toegankelijkheid.
- Samenhang tussen systeemschermen en meegeleverde programma’s.

Geen lorem ipsum, onafgewerkte standaardvensters, kapotte lay-outs, onverklaarde technische meldingen of “coming soon” voor verplichte functies.

Houd vormgeving en assets onderhoudbaar, zodat later aangeleverde ontwerpen consistent kunnen worden geïntegreerd.

Controleer de daadwerkelijke gerenderde interface. Screenshots ondersteunen die controle, maar vervangen geen bedieningsproef.


6. LEVER BRUIKBARE BASISPROGRAMMA’S EN APPBEHEER

Implementeer de basisprogramma’s uit de productspecificatie. Neem voor deze testrelease minimaal op:
- Materiaal-/bestandsbeheer met zoeken en ordenen.
- Een bruikbare teksteditor.
- Afbeeldingsweergave.
- PDF-weergave.
- Een webbrowser.
- Audio- en videoweergave voor vastgelegde gangbare formaten.
- Een rekenmachine.
- Schermopname of screenshotfunctionaliteit.
- Systeeminformatie, diagnose en toegankelijke hulp.
- Appinstallatie, bijwerken en verwijderen.

Leg ondersteunde bestandsformaten en gedrag concreet vast. Maak openen, opslaan, standaardgereedschappen en toegangsrechten samenhangend.

Een programma dat alleen opent of een vooraf gevuld scherm toont, voldoet niet.

Appinstallatie moet via een werkende pakket- en rechtenstructuur verlopen. Ondersteun de afgesproken installatiebronnen. Behandel een geldige handtekening als bewijs van herkomst, niet als algemene toestemming.

Claim compatibiliteit met programma’s van andere besturingssystemen alleen wanneer die daadwerkelijk is geïmplementeerd en getest.


7. MAAK OPSLAG GESCHIKT VOOR WERKELIJK GEBRUIK

Werk de opslaglaag uit tot een bruikbare basis voor de volledige testrelease.

Ondersteun de afgesproken functies voor namen, metadata, organiseren, zoeken, wijzigen, verwijderen, geschiedenis en herstel.

Verwijder experimentele capaciteitsbeperkingen die normaal gebruik verhinderen door de onderliggende implementatie verantwoord uit te breiden.

Zorg voor:
- Duurzame opslag na volledig afsluiten en opnieuw starten.
- Correcte grenzen bij grote invoer, volle opslag en mislukte I/O.
- Behoud van eerder bevestigde gegevens bij relevante fouten.
- Veilige initialisatie, ruimtebeheer en herstel.
- De afgesproken versleuteling en bescherming van inhoud en metadata.
- Betrouwbare import en export in de ondersteunde formaten.

Een herstelde geheugensnapshot is geen bewijs van duurzame opslag.

Voer herstelproeven uit op wegwerpimages en controleer de werkelijk opgeslagen gegevens onafhankelijk van alleen succesmeldingen in de interface.


8. INTEGREER NETWERK, NAS EN LOKALE AI

Deze onderdelen behoren tot deze testrelease en mogen niet worden teruggebracht tot losse demonstraties.

NETWERK
Lever werkende netwerkconfiguratie, verbinding, naamresolutie en beveiligd transport voor het gekozen profiel. Controleer netwerkuitval, herstel, time-outs, certificaatfouten en geweigerde rechten.

NAS
Voer het actuele privacycontract uit voor expliciet gekoppelde eigen of gedeelde NAS-opslag:
- Werkelijke koppeling en identiteitscontrole.
- Versleuteling voordat persoonlijke inhoud wordt verzonden.
- Sleutels bij de gebruiker volgens het afgesproken model.
- Werkende opslag, synchronisatie en foutafhandeling.
- Quota, uitnodigen, delen en intrekken.
- Het afgesproken onderscheid tussen gebruiker en NAS-beheerder.
- Lokale bruikbaarheid wanneer de NAS onbereikbaar is.
- Afzonderlijk toegestane verbindingen volgens de actuele afspraken.

Een protocolconforme test-NAS mag als externe tegenpartij dienen. De OS-client, rechtencontrole en lokale verwerking moeten werkelijk in Nieuw OS functioneren.

LOKALE AI
Lever een optioneel te activeren, daadwerkelijk werkende lokale AI-functie op een passend ondersteund profiel:
- Werkelijke modeluitvoering op het eigen apparaat.
- Expliciete selectie van context.
- Begrensde toegang en acties via de afgesproken broker.
- Annulering en zichtbare verwerkingstoestand.
- Beheersing van geheugen- en rekengebruik.
- Begrijpelijk gedrag bij ontbrekend model of onvoldoende capaciteit.

Gebruik geen vooraf geschreven antwoorden als bewijs dat AI werkt.

Behoud de afgesproken uitsluiting van leverancierscloud voor persoonlijke OS-opslag, verplichte leverancieraccounts, externe OS-telemetrie en cloudfallback voor systeem-AI.


9. LEVER WERKENDE BEVEILIGING, UPDATES EN HERSTEL

Beveiligingsbeleid moet daadwerkelijk worden afgedwongen door de betreffende systeemgrenzen.

Controleer onder meer:
- Proces- en dienstisolatie.
- Rechtenverlening, weigering en intrekking.
- Gevalideerde communicatie tussen processen.
- Bescherming van opslag en sleutels.
- Veilige entropy en correcte cryptografische integratie.
- De vastgelegde boot-, ondertekenings- en PQC-contracten.
- Foutisolatie en begrensde resourceconsumptie.

Schakel controles niet uit om een demonstratie te laten slagen.

Lever een werkend update- en herstelpad:
- Controle van authenticiteit en integriteit.
- Correcte installatie.
- Herstel bij een mislukte of onderbroken update.
- Een gecontroleerde terugkeer naar een toegestane werkende versie.
- Behoud van gebruikersgegevens volgens het herstelcontract.

Gebruik duidelijk afgebakende testsleutels waar dat voor ontwikkeling nodig is. Presenteer die niet als productie-infrastructuur.

Claim geen onkraakbaarheid, universele hardwareondersteuning of volledige formele verificatie. Koppel iedere beveiligingsclaim aan de werkelijk onderzochte eigenschap en aannames.


10. WERK MET MAXIMALE NUTTIGE PARALLELLISATIE

Gebruik werkelijk beschikbare agents om onafhankelijke onderdelen parallel uit te voeren.

Verdeel taken bijvoorbeeld over:
- Kernel, processen en IPC.
- Drivers en hardwareondersteuning.
- Opslag en herstel.
- Gebruikersomgeving en ontwerp.
- Basisprogramma’s en appbeheer.
- Netwerk en NAS.
- Lokale AI.
- Beveiliging, updates en releasevalidatie.

Stem de verdeling af op beschikbare capaciteit en concrete afhankelijkheden.

Leg eigenaarschap, interfacecontracten en acceptatie per taak vast. Gebruik afzonderlijke worktrees waar nodig. Voorkom gelijktijdige ongecoördineerde wijzigingen aan dezelfde bestanden.

De hoofdagent blijft verantwoordelijk voor samenvoeging, conflictoplossing, regressies en de werking van het gehele OS.

Het afronden van een subtaak betekent niet dat het product klaar is.


11. TEST HET SAMENHANGENDE PRODUCT

Voer echte gebruikersscenario’s uit op het uiteindelijke releaseimage.

Controleer minimaal:
- Eerste start en inrichting.
- Dagelijks starten, vergrendelen, ontgrendelen en afsluiten.
- Meerdere programma’s gebruiken en ertussen wisselen.
- Materiaal maken, openen, bewerken, zoeken, verwijderen en herstellen.
- Opslaan en terugvinden na koude herstart.
- Browsergebruik en downloaden binnen verleende rechten.
- Media openen en afspelen.
- Apps installeren, bijwerken en verwijderen.
- Instellingen wijzigen en na herstart terugvinden.
- NAS koppelen, gebruiken, onderbreken en opnieuw verbinden.
- Lokale AI met geselecteerde context gebruiken en annuleren.
- Toegang weigeren en intrekken.
- Update, mislukte update en herstel.
- Relevante resource-, opslag- en netwerkfouten.

Test daarnaast herhaald opstarten en langere gecombineerde gebruikssessies met vooraf vastgelegde criteria.

Leg voor prestaties meetbare grenzen vast op het referentieprofiel. Meet in plaats van alleen te schrijven dat iets “snel” of “soepel” voelt.

Koppel bewijs aan de exacte commit, configuratie en hashes van de geleverde images. Een geslaagde hosttest of compilatie vervangt geen native uitvoering.

Verklaar de release pas klaar wanneer alle verplichte acceptatiecriteria slagen en bekende blokkerende defecten zijn opgelost.


12. BESCHERM DE TESTCOMPUTER

Bouw en valideer eerst in een gecontroleerde virtuele omgeving met eigen testimages.

Gebruik voor netwerk- en NAS-proeven afgebakende testnetwerken, testgegevens en toegestane testservers.

Koppel geen persoonlijke schijven, echte NAS-inhoud, accounts of fysieke apparaten zonder gerichte autorisatie. Gebruik geen fysieke schijfwrites, partitionering, firmwareflash of wijziging van de bestaande bootconfiguratie als automatische vervolgstap.

Beperk bouw-, start- en resetscripts tot hun aangewezen bestanden en paden.

Lever voor fysieke hardware een apart profiel met ondersteunde onderdelen, gecontroleerde image, concrete instructies en herstelroute. Een geslaagde VM-test mag niet als geslaagde test op mijn laptop worden gepresenteerd.

Ontbrekende informatie over mijn fysieke apparaat mag onafhankelijk ontwikkelwerk en VM-validatie niet stilleggen.


13. BLIJF UITVOEREN TOT HET EINDDOEL IS BEREIKT

Werk voortdurend in de cyclus:
onderzoeken → implementeren → bouwen → uitvoeren → beoordelen → repareren → integreren.

Neem normale technische beslissingen zelfstandig binnen de opdracht en bestaande bevoegdheden.

Stop niet na:
- Een plan.
- Een documentatie-update.
- Een deelcommit of pull request.
- Een geslaagde compilatie.
- Een afzonderlijke werkende functie.
- Een tussentijdse mijlpaal.

Geef korte voortgangsberichten met afgeronde acceptatiecriteria, werkelijk bewijs, eerstvolgende werk en concrete blokkades. Ga daarna verder.

Behoud de bestaande voortgangsrubric. Verhoog percentages alleen wanneer de vereiste bewijzen bestaan.

Stop bij volledige acceptatie, mijn pauze/stop, een harde systeem- of budgetlimiet, of een externe blokkade waarvoor geen toegestane vervolgstap bestaat.

Werk bij een gedeeltelijke blokkade de onafhankelijke onderdelen af. Omzeil geen toestemmingen, limieten of beveiligingsregels.

Bewaar bij onderbreking:
- De exacte werkstand.
- Het laatste bruikbare artifact.
- Open acceptatiecriteria.
- Uitgevoerde tests en hun uitkomsten.
- Concrete blokkades.
- De eerstvolgende uitvoerbare hervatstap.

Noem een onderbroken of geblokkeerde opdracht niet voltooid.


14. LEVER EEN COMPLEET TESTPAKKET OP

De oplevering bevat:
- De geïntegreerde broncode en bijbehorende commit.
- Het werkelijk geteste releaseimage.
- Een reproduceerbare bouwprocedure.
- Start- en herstelprocedures.
- Een toegankelijke Nederlandse gebruikershandleiding.
- De ondersteunde configuraties.
- Een testplan waarmee ik het gehele product kan beoordelen.
- Het ingevulde acceptatieoverzicht met bewijs.
- Bekende beperkingen en resterende niet-blokkerende bevindingen.
- Een duidelijke route voor foutmeldingen en logs.

De eindverklaring “volwaardige testrelease gereed” is alleen toegestaan als het geleverde pakket de volledige verplichte acceptatie doorstaat.

Begin nu met de actuele repositorycontrole, leg het volledige werk vast en ga aansluitend door met implementatie.
<!-- FULL-RELEASE-REQUEST:END -->

## English translation of the original assignment

The following is a faithful English translation of the project owner's quotation
above. The original wording and its hash remain authoritative as the preserved
source record; this translation creates no new requirement or authority.

> ROLE AND MAIN ASSIGNMENT
>
> You are the lead developer and integration lead for Nieuw OS.
>
> My assignment is to continue development until a complete, coherent, professionally finished test release of the operating system is available. I want to be able to assess and use the complete product on the specified test platform.
>
> The final objective includes working system functions, a polished user environment, usable basic applications, settings, storage, networking, local AI, NAS functionality, security, updates and recovery.
>
> A boot demo, note-taking test, technical preview, browser prototype, collection of separate components or interface with simulated functions does not satisfy the assignment.
>
> Record this assignment in docs/FULL-TEST-RELEASE.md. Link this document to the existing project status and work queue so that there is one consistent source for progress. Preserve the content of this assignment during summarization and resumption.
>
> Then create an executable plan and begin execution immediately. Delivering a plan does not complete the assignment.
>
> 1. CHECK THE ACTUAL PROJECT STATE
>
> Read the current AGENTS.md files, README, project status, architecture decisions, product specifications, privacy contracts, security contracts, build instructions and test procedures.
>
> Check the active branch, commits, relevant development branches, open pull requests and existing local changes. Use the current implementation and traceable test evidence as the starting point.
>
> Old chat claims, progress percentages, documentation and passing tests from other commits do not automatically count as evidence for the current release.
>
> Build on usable existing implementations. Reuse and integrate good work. Resolve conflicts, duplicate implementations and contradictory interfaces without losing existing user work.
>
> Do not restart the overall architecture when a targeted improvement is sufficient.
>
> 2. RECORD FULL ACCEPTANCE
>
> Create an acceptance matrix for the full test release.
>
> For each mandatory function, state:
> - Which concrete user action must work.
> - Which native components and interfaces it requires.
> - Which normal and failure scenarios are tested.
> - On which hardware or VM profile the function is supported.
> - What evidence is required to sign off the function.
> - Which dependencies or blockers remain open.
>
> Use the existing product agreements and the requirements below as the basis. This assignment replaces the earlier limited objective of only booting and saving a short note.
>
> Do not reduce the product scope to obtain a passing result sooner.
>
> Do not remove, hide or disable a mandatory function to pass acceptance. Do not independently move difficult release requirements to “later”. If a requirement is not yet executable, it remains visibly open.
>
> Distinguish existing release requirements from research that already has a separate track in the recorded roadmap. Do not use that distinction to create new exceptions.
>
> 3. PRESERVE THE PROJECT'S OWN ARCHITECTURE
>
> Nieuw OS remains an independent operating system with its own kernel and product experience.
>
> Preserve the agreed microkernel direction, capability security, process isolation, IPC contracts and separation between kernel, drivers and system services.
>
> Do not expand the trusted core solely to simplify integration. Justify necessary changes and check their consequences.
>
> Use permitted standards, libraries, engines and development tools when they help the project. Check compatibility, maintenance, licenses and security boundaries. A browser engine or another library need not be reinvented if responsible reuse is possible.
>
> Do not replace the project's own kernel with Linux, BSD, XNU, Zircon or another existing kernel.
>
> Native functionality must work through the project's own kernel and system services. Host tools may build, test and emulate, but must not perform missing OS functionality as though it already exists in Nieuw OS.
>
> Test data and protocol-conforming test servers are permitted. Simulated success messages and host prototypes do not count as implementation evidence.
>
> 4. DELIVER A COMPLETE USER ENVIRONMENT
>
> Develop Materiaal into a coherent native user environment.
>
> Provide:
> - A polished boot and first-use experience.
> - User profile, session, locking and unlocking.
> - The agreed screens for material, editing, tools, access, devices and assistant.
> - Reliable navigation, focus and interaction with keyboard and mouse.
> - Working selection, context menus, clipboard and dragging where the product specification requires them.
> - Understandable notifications, progress, cancellation and error handling.
> - Consistent settings that persist after restart.
> - Correct restart and shutdown, with prior handling of saved and unsaved work.
> - Power and resume behavior according to the recorded requirements of the supported profile.
>
> Every visible control must actually perform the intended action.
>
> Conditionally unavailable functions, for example because hardware is missing or permission is denied, must show their actual state and reason. Missing implementation must not be disguised as such a normal condition.
>
> 5. MAKE VISUAL QUALITY A RELEASE CONDITION
>
> The OS must feel coherent, calm, recognizable and professional.
>
> Use one coherent design system for typography, colors, icons, spacing, components, animations and interaction states. Preserve the selected distinct identity and the Materiaal direction.
>
> Also design and check:
> - Loading, empty states, errors, confirmation and recovery.
> - Focus, selection, hover and disabled controls.
> - Long text, different window sizes and display scales.
> - Keyboard navigation, readability, contrast and accessibility.
> - Consistency between system screens and included applications.
>
> No lorem ipsum, unfinished default windows, broken layouts, unexplained technical messages or “coming soon” for mandatory functions.
>
> Keep design and assets maintainable so that designs supplied later can be integrated consistently.
>
> Check the actually rendered interface. Screenshots support that check but do not replace an interaction test.
>
> 6. DELIVER USABLE BASIC APPLICATIONS AND APP MANAGEMENT
>
> Implement the basic applications from the product specification. Include at least the following in this test release:
> - Material/file management with search and organization.
> - A usable text editor.
> - Image viewing.
> - PDF viewing.
> - A web browser.
> - Audio and video playback for specified common formats.
> - A calculator.
> - Screen recording or screenshot functionality.
> - System information, diagnostics and accessible help.
> - App installation, updating and removal.
>
> Specify supported file formats and behavior concretely. Make opening, saving, default tools and access permissions coherent.
>
> An application that only opens or shows a prefilled screen does not satisfy the assignment.
>
> App installation must use a working package and permission structure. Support the agreed installation sources. Treat a valid signature as evidence of provenance, not as general permission.
>
> Claim compatibility with applications from other operating systems only when it has actually been implemented and tested.
>
> 7. MAKE STORAGE SUITABLE FOR ACTUAL USE
>
> Develop the storage layer into a usable foundation for the full test release.
>
> Support the agreed functions for names, metadata, organization, search, modification, deletion, history and recovery.
>
> Remove experimental capacity limits that prevent normal use by responsibly extending the underlying implementation.
>
> Provide:
> - Durable storage after a complete shutdown and restart.
> - Correct boundaries for large input, full storage and failed I/O.
> - Preservation of previously confirmed data during relevant failures.
> - Safe initialization, space management and recovery.
> - The agreed encryption and protection of content and metadata.
> - Reliable import and export in supported formats.
>
> A restored memory snapshot is not evidence of durable storage.
>
> Run recovery tests on disposable images and inspect the actual stored data independently of success messages in the interface alone.
>
> 8. INTEGRATE NETWORKING, NAS AND LOCAL AI
>
> These components belong to this test release and must not be reduced to separate demonstrations.
>
> NETWORKING
>
> Deliver working network configuration, connectivity, name resolution and secure transport for the selected profile. Check network loss, recovery, timeouts, certificate errors and denied permissions.
>
> NAS
>
> Implement the current privacy contract for explicitly paired personal or shared NAS storage:
> - Actual pairing and identity verification.
> - Encryption before personal content is transmitted.
> - Keys held by the user according to the agreed model.
> - Working storage, synchronization and error handling.
> - Quotas, invitations, sharing and revocation.
> - The agreed distinction between user and NAS administrator.
> - Local usability when the NAS is unreachable.
> - Separately permitted connections according to the current agreements.
>
> A protocol-conforming test NAS may serve as an external peer. The OS client, permission checks and local processing must actually function in Nieuw OS.
>
> LOCAL AI
>
> Deliver an optionally enabled, actually working local AI function on a suitable supported profile:
> - Actual model execution on the user's own device.
> - Explicit context selection.
> - Bounded access and actions through the agreed broker.
> - Cancellation and visible processing state.
> - Control of memory and compute use.
> - Understandable behavior when a model is missing or capacity is insufficient.
>
> Do not use prewritten answers as evidence that AI works.
>
> Preserve the agreed exclusion of vendor cloud for personal OS storage, mandatory vendor accounts, external OS telemetry and cloud fallback for system AI.
>
> 9. DELIVER WORKING SECURITY, UPDATES AND RECOVERY
>
> Security policy must actually be enforced by the relevant system boundaries.
>
> Check, among other things:
> - Process and service isolation.
> - Granting, denial and revocation of permissions.
> - Validated communication between processes.
> - Storage and key protection.
> - Secure entropy and correct cryptographic integration.
> - The recorded boot, signing and PQC contracts.
> - Fault isolation and bounded resource consumption.
>
> Do not disable checks to make a demonstration pass.
>
> Deliver a working update and recovery path:
> - Authenticity and integrity checks.
> - Correct installation.
> - Recovery from a failed or interrupted update.
> - A controlled return to a permitted working version.
> - Preservation of user data according to the recovery contract.
>
> Use clearly bounded test keys where needed for development. Do not present them as production infrastructure.
>
> Do not claim unbreakability, universal hardware support or full formal verification. Bind every security claim to the actually investigated property and assumptions.
>
> 10. USE MAXIMUM USEFUL PARALLELIZATION
>
> Use actually available agents to execute independent components in parallel.
>
> Distribute tasks, for example, across:
> - Kernel, processes and IPC.
> - Drivers and hardware support.
> - Storage and recovery.
> - User environment and design.
> - Basic applications and app management.
> - Networking and NAS.
> - Local AI.
> - Security, updates and release validation.
>
> Align the distribution with available capacity and concrete dependencies.
>
> Record ownership, interface contracts and acceptance per task. Use separate worktrees where needed. Prevent simultaneous uncoordinated changes to the same files.
>
> The lead agent remains responsible for integration, conflict resolution, regressions and operation of the entire OS.
>
> Completing a subtask does not mean the product is ready.
>
> 11. TEST THE COHERENT PRODUCT
>
> Execute real user scenarios on the final release image.
>
> Check at least:
> - First start and setup.
> - Daily startup, locking, unlocking and shutdown.
> - Using multiple applications and switching between them.
> - Creating, opening, editing, searching, deleting and restoring material.
> - Saving and recovering data after a cold restart.
> - Browser use and downloading within granted permissions.
> - Opening and playing media.
> - Installing, updating and removing apps.
> - Changing settings and recovering them after restart.
> - Pairing, using, interrupting and reconnecting NAS.
> - Using and cancelling local AI with selected context.
> - Denying and revoking access.
> - Update, failed update and recovery.
> - Relevant resource, storage and network failures.
>
> Also test repeated booting and longer combined usage sessions with criteria specified in advance.
>
> Set measurable performance limits on the reference profile. Measure instead of merely stating that something feels “fast” or “smooth”.
>
> Bind evidence to the exact commit, configuration and hashes of the delivered images. A passing host test or compilation does not replace native execution.
>
> Declare the release ready only when all mandatory acceptance criteria pass and known blocking defects have been resolved.
>
> 12. PROTECT THE TEST COMPUTER
>
> Build and validate first in a controlled virtual environment with dedicated test images.
>
> Use bounded test networks, test data and permitted test servers for network and NAS tests.
>
> Do not attach personal disks, real NAS content, accounts or physical devices without specific authorization. Do not automatically proceed to physical disk writes, partitioning, firmware flashing or changes to the existing boot configuration.
>
> Restrict build, startup and reset scripts to their designated files and paths.
>
> For physical hardware, provide a separate profile with supported components, a verified image, concrete instructions and a recovery route. A passing VM test must not be presented as a passing test on my laptop.
>
> Missing information about my physical device must not halt independent development work and VM validation.
>
> 13. CONTINUE EXECUTION UNTIL THE FINAL OBJECTIVE IS REACHED
>
> Work continuously in the cycle:
> research → implement → build → execute → assess → repair → integrate.
>
> Make normal technical decisions independently within the assignment and existing authority.
>
> Do not stop after:
> - A plan.
> - A documentation update.
> - A partial commit or pull request.
> - A successful compilation.
> - An individual working function.
> - An interim milestone.
>
> Give brief progress reports with completed acceptance criteria, actual evidence, next work and concrete blockers. Then continue.
>
> Preserve the existing progress rubric. Increase percentages only when the required evidence exists.
>
> Stop at full acceptance, my pause/stop instruction, a hard system or budget limit, or an external blocker for which no permitted next step exists.
>
> At a partial blocker, complete the independent components. Do not bypass permissions, limits or security rules.
>
> At an interruption, preserve:
> - The exact work state.
> - The last usable artifact.
> - Open acceptance criteria.
> - Executed tests and their results.
> - Concrete blockers.
> - The next executable resume step.
>
> Do not call an interrupted or blocked assignment complete.
>
> 14. DELIVER A COMPLETE TEST PACKAGE
>
> Delivery includes:
> - Integrated source code and its commit.
> - The actually tested release image.
> - A reproducible build procedure.
> - Startup and recovery procedures.
> - An accessible Dutch user manual.
> - Supported configurations.
> - A test plan with which I can assess the entire product.
> - The completed acceptance overview with evidence.
> - Known limitations and remaining nonblocking findings.
> - A clear route for error reports and logs.
>
> The final declaration “volwaardige testrelease gereed” (full test release ready) is permitted only if the delivered package passes all mandatory acceptance.
>
> Begin now with the current repository check, record the full work and then continue with implementation.
