# TPM-CRB-TRANSPORT-1 — bounded driver core

Status: separate safe/no_std transport and codec logic in
`drivers/tpm-crb`; opt-in MMIO bootstrap fixture now in implementation per
ADR-052, both native cases now green on `4b09869`. No product kernel grant or cold
rollback anchoring. Host models are exclusively protocol/register tests.
The still mandatory B02/boot/update requirements remain open.

`native-tpm-crb` holds exclusively one fixed supervisor RW/NX/UC mapping
at VA `0x3f180000`, with typed register offsets, checked buffer bounds,
at most 128 B copies, volatile accesses and MFENCE/compiler barriers.
The single claim and quarantine are preserved for the VM lifetime.
No app gets a TPM right; the existing two real CPL3 waiting apps keep
using their original rights and full oracles. The scheduler
polls the existing transport core with real PIT ticks; one read-only
family property query and fully confirmed release are required.
`test-tpm-crb-absent` requires Unsupported/NotStarted/quarantine and zero
writes. Seven adapter boundary rejections must not cause any MMIO write.
Runner: `tools/run_tpm.py --case normal|absent`; swtpm uses exclusively
a managed own Unix socket/state, no real TPM or network connection.
The code below remains the transport contract; old open-status sentences
are not new native acceptance. Current execution: PROJECT-STATE.
Source/artifact/log binding and the preserved first failure are in
`verification/native-test-version/tpm-evidence.json`. Both VMs raw33;
absent makes zero writes, positive gives family `0x322e3000` and confirmed
Idle/relinquish. The existing full CPL3 waiting trials remain green.

## Supported interface contract

The only device model examined is QEMU 10.0.13 `tpm-crb`, locality 0,
MMIO aperture 0xFED40000..0xFED40FFF. The machine compatibility used remains
`pc-q35-8.2`; that is not QEMU version 8.2. The local managed binary reports
10.0.13 Debian 1:10.0.13+ds-0+deb13u1 and advertises `tpm-crb`. No
TPM instance with a backend has been started yet. The configuration below is an expected
model contract based on upstream source, not native discovery evidence.

CRB registers are little-endian 32-bit. ControlRequest/Status/Cancel/Start are
at 0x40/44/48/4C; commandSize/address at 0x58/5C/60, responseSize/address at
0x64/68/6C. Both hardware buffers must point to exactly the same 3968 B region starting at
0xFED40080. Other addresses, external RAM/DMA buffers, other sizes,
FIFO interface, other locality, idle bypass or a non-selected CRB
are rejected. There is no automatic FIFO or device fallback.
InterfaceType 1/version 1, locality 0 only, CRB support and selector 1 are required;
vendor/revision and transfer width flags are not treated as authority.

`Io` owns the single adapter. It provides typed read32/write32(Register),
read_buffer/write_buffer with relative offsets within the fixed hardware buffer,
and quarantine(). Every buffer action is at most 128 B. A future adapter
must itself enforce the bounds, exactly completed copies, UC/device mapping,
compiler/CPU memory ordering and volatile register/buffer accesses.
Buffer writes must be visible before START; response reads follow only after
the confirmed completion. These conditions are not yet an examined unsafe
MMIO implementation. No arbitrary physical pointers or frames
from device data are mapped.

In this QEMU model the buffer is device MMIO, not allocator/user RAM or a
virtio DMA pool. Quarantine prevents reuse or transfer of the device
authority and new writes. It does not prove IOMMU isolation on physical
hardware. A future broker remains responsible for mappings,
capability holder, revocation, deadlines and process death.

## Transport and lifetime

`Transport` is not Clone/Copy and does not release its Io during a
hardware transaction. API: acquire(now,deadline)→Ticket, poll_ready,
execute(&codec::Command,now,deadline,cancel_ticks)→Ticket, poll,
cancel, release(now,deadline)→Ticket, poll_released, detach.
Tickets are non-0, checked monotonic and must not be swapped between
operations. Wrong tickets do not change the active transaction and perform
no I/O. A completed response is returned once as its own typed
value; a broker must retain it itself when usercopy has not yet
been confirmed. No request is automatically republished.

The caller supplies real monotonic Sys3/PIT ticks, never user-chosen time or a
poll-count clock. The core has no clock of its own or hidden wait loop: every
poll is bounded, and the future broker must keep polling until the deadline at
the latest and handle loss of ownership. Deadline is absolute, greater
than now and at most 60000 ticks later; cancel_ticks lies in 1..60000. For the
current 100 Hz profile this is at most 600 s, not a standard product timeout.
EAGAIN-like polling does not renew the original deadline.
Time going backwards or overflowing blocks active polls, new commands and release. Already verified Ready/Released/Cancelling ACKs can be returned idempotently without a new clock check or I/O; they confirm exclusively that earlier state and grant no new mutation.

Acquire requests exclusively locality 0, never seize/resetEstablishment. After
confirmed locality, COMMAND_READY follows; only after request has been cleared and the
device is Ready/START0 can the command buffer be written. Buffers,
locality and status are re-checked before use. Already active
work or a foreign/seized locality is not taken over.

Before START the entire shared buffer is actually cleared via Io,
with no I/O outside the fixed aperture in between. Then the exactly
encoded command is written. This prevents an old success header from being used as a
new result. The new command may be ready immediately;
a previously observed Busy1 is therefore not required. A vanished
START bit alone is insufficient: locality, non-idle status, fixed
buffer descriptors, bounded response header, stable header and the specific
response format must also be correct. Publication follows only after full validation
and actual clearing of the hardware buffer that is now available to the CPU again.

`Failure {fault,effect}` distinguishes NotStarted, Unknown and Completed.
NotStarted means exclusively that no START has been published for this command
operation; it promises no absence of locality/Ready state changes.
A returned Ready after cancellation does not resolve the earlier Unknown
transaction. Nor does is_nv_mutation=false promise absence of other TPM or
authorization state effects. Effect becomes Unknown **before** the START write: even a failed write may have
reached the device. Every uncertain error after that remains Unknown.
Completed on a TPM error code means only that a canonical TPM response
was received; it does not mean success, a storage flush or a general
no-side-effects claim. TPM_RC_INITIALIZE, for example, is explicitly
returned and not silently marked as a successful Startup.

On timeout/cancellation, CANCEL1 is written once. Only real START0,
a CANCEL0 write and observed CANCEL0 permit buffer clearing/Ready. The caller
then still receives Timeout/Cancelled with Unknown effect, never a positive
NV mutation confirmation. If the bounded completion/clear ACK is missing,
permanent quarantine follows. No buffer writes, detach, regrant, replay or fabricated
reset confirmation. There is deliberately no reset() API. The same applies on
unexpected device reset, changed descriptors, corrupt response or
partial START I/O. Drop calls the quarantine hook once unless never
used or fully released; it performs no hidden device reset.
The adapter must preserve that block in the broker even after its own Drop.

Release requires Ready/START0, clears the buffer, requests GO_IDLE and waits for the
cleared request plus Idle1. Only then localityRelinquish and the confirmation
that Assigned and Granted are off. Only after that can detach return the
adapter. Without ACK there is no implicit release.

## TPM2 codecs, authorization and data

The separate codec encodes only Startup(Clear/State), SelfTest(full),
GetCapability(TPM_PROPERTIES, at most 16), NV_ReadPublic, NV_DefineSpace for
one 8 B counter, NV_Read of exactly those 8 B and NV_Increment. No Clear,
UndefineSpace, hierarchy change, general raw command interface, RNG
alternative, new crypto primitive or silent provisioning password.

Commands are at most 128 B, responses at most 512 B. Tag, big-endian length,
command code, count, order, fixed nvIndex, name algorithm, counter type,
authPolicy length and datasize are checked. Every command encoder
checks input/output space before output mutation. The command value borrows
only the written caller buffer. The caller protects/clears its own RAM;
the transport core keeps no second RAM copy of the password command.

NV authorization is an explicitly caller-supplied TPM_RS_PW borrow
(max 32 B, no Default/Debug). An empty value is only possible through an
explicitly empty caller value, never a hidden fallback. The host tests
use visible synthetic values and perform no NV write on a
TPM. The current codec profile is not a worked-out production auth policy.
PW sessions offer no encryption of the command buffer or cryptographic
response HMAC. The TPM/MMIO chain remains trusted; the ordinary app must not get any
right to it. For counters, AUTHWRITE/AUTHREAD, counter type and
SHA256 name algorithm apply; dictionary attack protection is not disabled.
Only the dynamic WRITTEN bit is additionally accepted.

A password response requires empty nonce/HMAC and continueSession 1, not 0.
Properties are bounded and strictly ascending; foreign capabilities or
non-canonical more/count fields fail. NV_ReadPublic returns an **opaque**
structurally fitting name: the codec does not compute or authenticate the
SHA256 digest and thereby proves no lineage. That belongs later to the
reviewed key/provisioning service outside the kernel. Error responses have
exactly the fitting 10 B header; no partial data is accepted.

## Rollback, boot, updates and privacy

`services/keyring/src/persistent.rs` requires an independently trusted
minimum_generation. The same number next to a recovery blob or on the same
rollbackable disk is not an anchor. This driver does not call restore_persistent
and does not yet make that guarantee true. A TPM counter alone also does not yet bind
a key lineage, authenticated snapshot, two-copy journal, flush ordering,
crash outcome or user confirmation. Uncertain NV_Increment/Define outcomes
must not be re-executed; first a genuinely examined
reconciliation/recovery contract is needed.

The still open cold trial requires separate disposable swtpm state and
OS disk, with a new QEMU/swtpm process per cold start. Rolling back only the disk
while the independent TPM anchor stays current must be
rejected. If a host administrator also rolls back TPM state/snapshots, that
independence assumption is gone; no false protection is claimed against
that. No personal disk, real NAS, physical TPM or firmware is
used here. Missing swtpm is no reason to present a test password, RAM value
or stored snapshot as a hardware anchor.

TPM presence does not create an authenticated boot/update chain or PQC guarantee.
The existing boot/signature/PQC contracts and the privacy Security Floor continue
to apply in full. No vendor account, cloud key, telemetry, external
storage or extra network right is introduced. Per property the
TCB remains named: the transport boundary relies on Io, device, clock and exclusive broker;
cold anti-rollback additionally on provisioning/auth, key codec, crash/disk commit,
boot/update trust and the genuinely independent persistent TPM state.

## Source check and still open normative check

The normative TCG PC Client PTP resource at
https://trustedcomputinggroup.org/resource/pc-client-platform-tpm-profile-ptp-specification/
was not reachable: host proxy 403. There is **no** read normative PTP PDF
or claim of TCG compliance. FIFO/CRB are not equated. The still
necessary primary document check comprises PTP Chapter 5 CRB and the relevant
TPM2 Library Parts 2/3 commands/session/NV counter rules.

The implementation sources actually read so far are:

- QEMU 10.0.13, commit 98f88efe8f07067667b8d9b1a5ce9551d34036c1:
  hw/tpm/tpm_crb.c, include/hw/acpi/tpm.h, tests/qtest/tpm-crb-test.c.
  Register definitions/layout, Ready/Start/completion/Idle/locality and the
  immediate-versus-deferred completion have been checked.
- Tianocore edk2-stable202502, commit fbe0805b2091393406952e84724188f8c1941837:
  MdePkg/Include/IndustryStandard/TpmPtp.h and Tpm20.h, plus
  SecurityPkg/Library/Tpm2CommandLib/Tpm2NVStorage.c. The Tpm20 header refers
  historically to Library rev 0.96; that is not a current normative spec check.
- Microsoft/TCG TPM2 reference, commit ee21db0a941decd3cac67925ea3310873af60ab3:
  TPMCmd/tpm/src/main/SessionProcess.c, BuildResponseSessions: PW response sets
  continueSession and empty nonce/HMAC. This implementation likewise does not replace the
  missing normative PTP/Library documents.

Precise URLs, local content hashes, query results and the managed QEMU binary hash
are under drivers/tpm-crb/verification. Upstream source has been read, not adopted as
C code or built along: QEMU GPL2+, edk2 BSD-2-Clause-Patent,
Microsoft/TCG reference BSD licence with explicit patent disclaimer. The
reference itself states “This code is informative”. The notices and terms
remain with their sources.
There are no new Cargo dependencies. No general
project licence is chosen here and no legal audit is claimed.

Still open: normative TCG check, independent source review, real swtpm
backend, unsafe/MMIO adapter, capability/broker grants, native transport and
cold failure/rollback trials, full boot/update/PQC/B02 integration and
product acceptance. Host tests and target build earn no points for this.
