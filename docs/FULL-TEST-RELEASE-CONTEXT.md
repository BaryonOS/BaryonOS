# Supplementary execution and product context — 30 October 2026

This normative supplement incorporates the twenty sections of the supplementary
project brief. It supplements [FULL-TEST-RELEASE](FULL-TEST-RELEASE.md);
the original full brief, `/goal`, architecture, 62 stable
acceptance IDs, work performed and valid source-bound evidence
are retained. It is not a separate roadmap or a reduced release scope.

## Comparison with the current execution

Already established: own Rust kernel/microkernel direction, capabilities/IPC/isolation,
Materiaal, full apps/NAS/AI/update/recovery scope, optional opaque external
storage, distinction between evidence levels, and VM-FR versus narrow regressions.
VM-FR is already pc-q35-8.2/Nehalem-v1/1vCPU/TCG/8GiB/32GiB sparse disk, with
fixed GOP/PS2/virtio devices and separate TPM2-CRB/swtpm state.

Actually executed: native services/documents/record crypto, own NIC with
ARP/UDP/DNS, FPU/clock and now TCP with a protocol peer; recent source versions
are in PROJECT-STATE. These are bounded partial profiles with plaintext
test disks/RAM keys, not secure persistent product storage or a release image.
Native desktop, complete user tasks and the full test suite remain open.

Concretely added: the task/compatibility register below, minimum
format coverage, explicit Chromium/Gecko comparison, accessibility details,
TCB per property and the fixed hundred cold starts/24-hour product session.
Ongoing memory/allocator/TLS/entropy/RTC tasks incorporate the relevant runtime
and security conditions. UI/app work remains a mandatory integration route.

## Fixed product and consumer contract

Nieuw OS must support high-quality daily use for communication,
internet/services, administration/work/study/education, gaming/music/video,
photography/design/creation and diverse digital skills/accessibility.
The European alternative and the own kernel are strategic choices, not
evidence of consumer preference, technical superiority or security.
Rust remains the primary language where prescribed. No existing host kernel as a replacement.
Responsible engine/library reuse and host tooling are permitted;
host functionality does not count as execution by Nieuw OS.

Consumer literature provides hypotheses, not already measured Nieuw OS usage:
being able to perform necessary tasks, noticeable benefit, understandable/recoverable
trying out and switching, met daily expectations, understandable
privacy/security, and care together with reliability. Research separately
installation on existing hardware, use alongside an existing environment and
purchase of a supported computer: migration, costs, support and recovery
differ. Take satisfied non-switchers seriously. Interest, wanting to
try, actually being able to switch and voluntarily continuing to use are
separate outcomes; downloads/positive reactions/AI review do not prove
retention, market share, preference or willingness to pay. No fabricated interviews.

## Complete user tasks and hard blockers

All rows below are open. VM-FR/HW-FR are targets, not a current
support claim. Each execution fills in concrete app/site/account/file/device
versions, initial situation, limits, configuration and evidence. Product tests use
own fixtures/test peers; real accounts, content, devices or drastic
actions require the existing targeted authorisation. A necessary
missing app/game/device/accessibility function remains a visible hard
switching blocker, even if other averages are favourable.

| ID and goal/initial situation | Requirements and normal outcome | Errors/recovery | Permissions | Configuration/evidence and FR mapping |
|---|---|---|---|---|
| UC01 receive document→edit→send to school/work | Concrete communication source, editor, UTF8/agreed exchange format; content/metadata preserved, edit/save/export/send | Corrupt/large file, conflict, offline, unknown storage outcome; recovery without false ACK | Only chosen source/object/save/export/destination | VM-FR/HW-FR; native entire chain, independent bytes/cold reopen; A01,A02,A13,O01–O07,N01–N04 |
| UC02 banking/government service | Named supported site/browser, test account and applicable authentication; complete representative task | TLS/identity/cert time/connection/cancel; no unintended transaction or silent retry | Named destination, explicit credential/device grants; no real financial action as an automated test | Native browser/transport/task evidence, external compatibility separately; A05,N01–N04,B04 |
| UC03 video calling | Named app/site/version, camera/mic/audio and controlled call partner | Permission denial/revocation, device loss, mute/stop/offline/recovery | Separate camera/mic/audio/network grants | VM-FR where available plus HW-FR; audible/visible execution, latency and revocation; H01–H03,A05,A06,N04 |
| UC04 import photos→select→edit→share→find again | Concrete source/device, PNG/JPEG, Materiaal and editor; quality/metadata/ordering preserved | Large/corrupt photo, full volume, wrong revision, offline sharing, recovery | Chosen import source, objects, export/share destination | Native entire chain, independent reference images and cold reopen; A01,A03,A13,O01–O07 |
| UC05 deliver creative project correctly | Named application/version/project, fonts/plugins/linked files, colleagues and export/print target | Missing dependency, fidelity/colour/export error and recovery | Selected project, graphics/audio/devices and controlled exchange | Named combination/task fidelity/performance; no general suite claim; A13–A15,H02,O07 |
| UC06 specific game with friends | Title/version/launcher/account/install/purchases/saves/multiplayer/chat/controller/headset | Install/save/connection/anti-cheat/DRM problem; no lowering of host security | Bounded own storage/network/window/input/audio/devices | Exact combination and conditions, FPS/frame pacing/input latency/stability; compatibility register, H01–H03,A11–A12,N04 |
| UC07 transfer files→cold reopen | Named source/files/formats, import/export/volume; find exact content again | Partial write, full, corruption, offline, conflict/uncertain ACK; confirmed bytes preserved | Only selected source/target/objects | Cold new VM, independent bytes/oracle; O01–O07,R01 |
| UC08 recover from error without unclear loss | Named error, history/backup/recovery/update and known data | Power interruption, key/anchor/slot loss, failed update | Scoped recovery authority, explicit downgrade policy | Disposable images/disk+separate TPM state, real recovery and user data check; B01–B03,O04–O06,R01 |
| UC09 work independently with accessibility | Keyboard-only or real screen reader, native semantics/status, 200% and suitable input | Focus trap, lost status, colour dependency, scale/long text, recovery | Bounded accessibility/focus/surface grants | Actually rendered and operated, human assessment where required; U01–U05,R01 |

## Materiaal, formats and compatibility

Materiaal keeps objects recognisable; opening apps directly remains possible. Names,
ordering, open/edit/save/share, location/history/permissions, progress and
actual cancellation are consistent. Saved/in progress/failed remain distinct;
no silent loss. Notifications/interruptions are controllable; core functions work
without AI, NAS or internet. Innovation does not make ordinary tasks unnecessarily hard.
One maintainable source for native tokens/components including accessibility
semantics, typography, icons, spacing, focus/selection, errors and motion.
Icons remain recognisable. Keyboard focus/screen reader status, contrast/legibility,
long texts, 200%, reduced motion and information without colour are release conditions.
Accessibility permissions and the security boundary are designed together.

All existing base apps plus spreadsheet/presentation material remain mandatory.
Minimum format coverage: UTF8, PNG/JPEG, PDF, WAV/FLAC/MP3, MP4(H264/AAC) and
WebM(VP9/Opus). Per format, read/write/edit operations/profiles,
content/metadata, corruption/size/unsupported variants and reference results are
concretely recorded before sign-off. An extension or a window that starts does not prove
full support. Parsers/codecs/engines get maintenance,
licence, dependency/build script, target, resource and isolation review.

| Compatibility register | Mandatory record before a claim | State |
|---|---|---|
| COMP01 named game combination | UC06 plus native port/API-ABI layer/runtime/guest choice, title/version, conditions and all functional/performance blockers | Desired concrete target still to be named; no universal game store/anti-cheat promise |
| COMP02 named professional creative combination | UC05 plus project fidelity/fonts/plugins/links/colour management/GPU/peripherals/colleague export/print | Desired concrete target still to be named; no Adobe/Windows suite compatibility claim |
| COMP03 compatibility environment | Actually enforced boundary, maintenance/runtime, mutual apps in shared guest, selected portals/network/surfaces/input/audio/devices/print | Optional design decision required; guest kernel does not replace host kernel |

Gaming/professional creation remain explicitly visible; the priority is the
usable OS. No automatic mandate to rebuild full creative suites
or to sign off unlimited universal compatibility. A layer is
not evidence of isolation; a shared guest can be a joint trust domain.
No general host permissions or lowering of host security for compatibility.
Data transfer goes through bounded open/save/clipboard/drag/print portals.

## Security, trust and data flows

Define the TCB per property: boot/initialisation/kernel, policy broker,
key management, compositor and relevant hardware may be co-trusted.
Small kernel/LOC/Rust do not prove security; check unsafe/FFI/safe
abstractions and logic/races/deadlocks/authorisation. Formal properties/assumptions,
tests and review remain distinguished; no inherited seL4 proofs or absolute claims.
An adopted component remains within explicit initial, object, domain,
operation, delegation/revocation/lifetime and resource permissions. IPC/shared memory
checks bounds/overflow/pointers/alignment/ownership/TOCTOU/reply binding/
cancel/deadline/cleanup. Zero-copy only with proven suitable lifetimes.
W^X/memory/process/FPU/SSE isolation and bounded queues/quotas remain on.
Read plus network can enable permitted transmission: explain combined
permissions. Revocation blocks future access; it does not retrieve old copies.

Drivers are isolated according to the architecture; provisional kernel transports and
remaining trust are explicit. Userspace driver does not mean DMA isolation:
IOMMU/groups/topology/IRQ/buffer lifetime, crash/reset/no-ACK/revoke/reuse
are concrete gates. Missing isolation means a visible limitation/refusal,
not a guarantee. Software composition remains the baseline; GPU acceleration is a separate
technical task, not a mandatory two-GPU consumer configuration. Measure on the right
hardware; secure focus/input/surfaces/capture/clipboard/transfer. Trusted
prompts are recognisable without relying on colour. Isolation of untrusted media/documents
and startup/cleanup costs are measurable; no unproven immediate residue destruction.
Timing/cache/SMT/hardware side channels are separate questions, not general guarantees.

## Storage, network, crypto, AI and recovery

Local/own NAS/shared NAS remain possible; external provider only explicitly
chosen client-side opaque ciphertext, without content keys/plaintext/sensitive
manifests. Also encrypt names/sensitive metadata before the provider adapter; credentials
separated. No mandatory vendor account/cloud, escrow/silent fallback/
OS telemetry/external system inference. An own provider account grants no OS permissions.
Client/NAS roles: the administrator manages quota/blocks/deletes their own guest copies
without guest decryption key/consent; no reading of content or wiping on other
devices. Durable keys/unlock/rotation/recovery, context-authenticated objects/
manifests, transactions/ACKs, offline/resume/conflicts, partial uploads/
revoke, import/export/independent backup remain mandatory. Encryption does not guarantee
availability/anonymity/recovery; size/timing/connection metadata may be
visible, sync is not backup. LAN/name/private IP is not authentication.
Retry automatically within granted permissions; new trust/broader permissions/
drastic action again requires the relevant authorisation.

Network comprises driver/config/routes/IP/TCP/UDP/DNS/time/authenticated transport/
process-destination permissions. The host is exclusively a peer. Suitable entropy and
maintained crypto, no predictable product seed/own primitive. Separate content
encryption/KEM/signature/key management/boot verification/measurement. ML-KEM is secret agreement,
ML-DSA is a signature, not access policy. Hybrid/PQC/boot contract remains on;
where dual signatures are required, one is insufficient. Crypto remains, where appropriate,
outside the kernel, boot in an explicit boot TCB. Updates: authenticity/integrity,
key/algorithm rotation, slots/atomic activation, every interruption, explicit
downgrade and offline recovery with data protection. Test anti-rollback and recovery
together: TPM2-CRB with its own swtpm state, disk/anchor transactions, anchor loss/
recovery. Host rollback of both disk and TPM falls outside this VM guarantee.
No secrets in sources/ordinary logs/error messages/test archives.

AI is optional, genuinely local, without direct LAN/internet; explicit context,
CPU/memory/output quotas, cancellation and restricted action brokers. Document/model output
grants no authority. A separate downloader fetches only chosen models, no
personal documents/index/prompt/NAS keys. Assess provenance/licence/integrity/parser/
resources/target of model and engine. Missing model/capacity:
understandable limitation/queue/smaller supported local model/AI off; no external
inference fallback. Core functions remain usable. Explicit encrypted AI backup
is allowed under the storage contract; NAS/provider do not thereby become an inference service.

## Execution, evidence and final package

Retain the Chromium/Gecko comparison before the engine decision. Concrete browser and
inference versions must demonstrate runtime/toolchain/memory/threading/FS/net/graphics/CPU/
sandbox/resources/maintenance. Target trials show a real local page or
model execution; host trial separately. The early chain boot→native UI→input→app→secure
save→shutdown→cold reopen is an interim milestone, not a reduced final scope. Native
quantum function retained; physical QPU/CHERI/extra ISA are separate existing research.

Maximum useful real agents with exact base, exclusive files/worktree/outputs,
interface/dependencies/acceptance/delivery. Root owns shared ABI/workspaces/
build/status/Git and integration. Reuse caches, preserve work, no general
re-audit/reinstall without concrete reason. A worker pass is not an integrated pass.
Git according to existing authorisation/branch rules; verify the actual remote.

Evidence levels: design, host prototype, host test, target build, own-kernel QEMU,
integrated user action, physical test, human research/review and
formal proof. No mutual substitution. Retain the regression profile; change VM-FR
only with a reasoned decision, single CPU does not prove SMP, TCG does not prove physical
game/AI performance. Test wrong/grants/context/generation/identity, corruption/
overflow/size/full/OOM, process/driver/service crash, I/O/reset no-ACK, network/cert,
transaction/power cut/stale request, key/update loss, conflicts/Unknown-ACK.
A timeout is green only if the intended negative behaviour has actually been established.
Actually render and operate, scaling/accessibility/error/recovery. Fix budgets before
sign-off; no favourable measurements or limits shifted after the fact.
**Hundred cold starts and a 24-hour combined product session** as soon as a stable
integrated image is available; the obligation remains open now, never shorten
to quick partial trials. Bind exact commit/config/tools/deps/model/assets/command/exit/
loader-kernel-image hash/log/measurement, repeat after the last relevant change.
Freeze during tests; CI counts only for its own commit, skipped review is not review,
AI review does not replace a human gate.

One progress chain, unchanged rubric/final scope/weights; defects visible,
extra tests do not automatically earn points. Continue autonomously with the next
independent step; at most one necessary product clarification when
actually blocking. Respect physical/destructive/production data/credentials/paid/
publication boundaries, prepare a reviewable package and continue working
independently. On a real session/context/tool/quota limit, preserve exact commit/worktrees/dirt/
artifacts/evidence/tests/active/open tasks/resume command; do not promise later
execution. Only a completely passed final package is called a full test release:
source, actually tested image, reproducible build, start/recovery/update, Dutch-language
manual, configurations, matrix/evidence/limitations and error/log route.

Primary decision anchors remain seL4 proofs/assumptions/capabilities, Rust Nomicon
races, Linux VFIO/IOMMU, Qubes FAQ, LionsOS verification status and NIST FIPS203/204
as recorded in the project brief. Consult applicable versions in a targeted way; these
sources yield no implementation, kernel replacement or inherited proof.


- https://sel4.systems/Verification/proofs
- https://sel4.systems/Verification/assumptions.html
- https://docs.sel4.systems/Tutorials/capabilities.html
- https://doc.rust-lang.org/nomicon/races.html
- https://kernel.org/doc/html/latest/driver-api/vfio.html
- https://doc.qubes-os.org/en/latest/introduction/faq.html
- https://lionsos.org/
- https://csrc.nist.gov/pubs/fips/203/final
- https://csrc.nist.gov/pubs/fips/204/final
