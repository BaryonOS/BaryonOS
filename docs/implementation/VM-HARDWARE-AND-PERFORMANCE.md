# QEMU, PC hardware and display latency

Research and bounded launcher change, 2 November 2026. Baseline:
`e08fb0e48c1bb0781e98357f588c2f73b6ae8500`. This document does not describe
a measurement on the Windows PC concerned or physical-hardware acceptance.

## Contract and actual cause

Goal: let the interactive Windows VM use CPU virtualisation when available
and able to initialise, with visible diagnostics. No host-setting changes,
disk/device passthrough or disabling of security measures. Original native
acceptance profiles remain TCG; Windows/WHPX acceptance is a separate check
that is still open.

The original launcher forced `tcg,thread=single`, `qemu64`, one vCPU,
256 MiB and `pc-q35-8.2`. TCG translates CPU instructions in software.
`run_materiaal_preview.py` also uses this explicit profile; its default
`timeout=45` is an abort bound, not a measured display time. The reported
delay of approximately 45 seconds per screen must be reproduced with
separate phase timings.

The code also shows an expensive rendering path:

- `ui/surface-protocol/src/lib.rs`: tiles of 32×8 pixels, at most 1024 bytes;
  `CHUNK=208`.
- `ui/surface-protocol/src/client.rs::Session::blit`: each complete tile
  sends BEGIN + five CHUNK + COMMIT and waits for one acknowledgement.
- `sdk/examples/compositor/src/surface.rs::Surface::step`: processes at most
  one request per call. The compositor yields its CPU turn after each main
  loop (`yield_now`).
- `sdk/examples/materiaal/src/lib.rs::paint_step`: one tile per app iteration;
  a changed view generation restarts at tile zero. There is no protocol yet
  for changed rectangles only.
- `kernel/src/framebuffer.rs::write_pixels`: validated scalar volatile stores
  to a supervisor-only uncached framebuffer. This is not GPU acceleration.

A complete **1280×720 surface** contains `40×90 = 3600` tiles: **25,200
requests and 3600 replies**, excluding display syscalls, input, polling and
compositor borders. With the existing 16/48/16-pixel borders, the assigned
app surface on a 1280×720 screen is 1248×656: 3198 complete tiles, hence
22,386 requests and 3198 replies. These are exact protocol counts from the
code, not a time measurement. Additional virtual cores do not solve this
serial path; the kernel is also still single-core.

## Implemented in the Windows launcher

`Start-NieuwOS.ps1 -Acceleration Auto` (default):

1. Check bundle hashes, including the new `Acceleration.ps1` helper.
2. Query `-accel help`. A listed WHPX backend proves only that QEMU was built
   with it.
3. Test WHPX with the same q35/CPU/memory profile, paused through `-S`. This
   trial has no disks, network, USB, host directories or display. A QMP reply
   to `query-status` with the expected request ID and status
   `paused`/`prelaunch`, capability/quit acknowledgements and exit 0 must
   confirm successful initialisation. Commands travel to QMP as exact UTF-8
   bytes with LF only, without Windows text-newline translation. The launcher
   first waits for the actual QMP greeting and then for each command
   acknowledgement before sending the next command; handshake and process
   exit share one ten-second deadline. After at most ten seconds, the trial
   is stopped. No guest code runs.
4. Then start the OS VM with WHPX. If initialisation is absent/fails, Auto
   reports the cause and selects TCG. `-Acceleration Whpx` refuses fallback;
   `-Acceleration Tcg` selects the original software profile.
5. Write the selected backend, reason, profile and start time to
   `state/launch.json`. An error during actual VM startup does not cause
   automatic restart or repetition of disk operations.

`-CheckOnly` checks hashes and lists, creates no mutable state and starts
no paused trial machine either. It therefore explicitly reports that WHPX
has not yet been initialised. A successful WHPX trial does not prove a
working Baryon OS session. The launcher disables **no** WHPX security-domain
mitigation (`ssd=off` is not used).

The package accepts exactly the two seven-process profiles
`native-system-materiaal` and `native-system-session-preview`. It retains
the actual profile in its manifest; the session preview is not presented
as historical Materiaal evidence. Both retain the same ELF, source, hash
and ESP checks.

## Using hardware while preserving boundaries

| Component | Next appropriate route | Evidence boundary |
|---|---|---|
| CPU on Windows x86-64 | WHPX through Windows Hypervisor Platform and firmware virtualisation | 6/6 actual helper trials and 6/6 native Windows runs plus Auto/WHPX/SDL startup; see source-bound final report |
| CPU on Linux x86-64 | KVM as QEMU backend with existing access to `/dev/kvm` | Researched; current native runners remain explicitly TCG |
| CPU on Intel Mac | HVF, if QEMU/host provide it | Researched, not implemented or tested |
| Apple Silicon or Windows ARM | An x86-64 guest requires architecture emulation; ARM virtualisation does not directly accelerate this x86-64 kernel | Another guest architecture is separate porting work |
| Memory | Bounded virtual RAM; one vCPU until SMP invariants are proven | More RAM does not reduce IPC transactions |
| Display | First rendering batches/dirty regions, then an isolated virtio-gpu driver | WHPX accelerates CPU execution, not a new GPU driver |
| Keyboard/mouse | Current emulated input channels | No passthrough of a complete host USB controller |
| YubiKey/FIDO2 | First simulated protocol errors and a bounded USB/CTAP2 service; later one explicitly selected test token | Current guest has no working FIDO2 login; no passthrough of a personal security key |
| Fingerprint reader | Later a supported driver + isolated authentication service and enrolment | Host Windows Hello does not provide a generic fingerprint API to this guest |
| TPM | Software TPM with dedicated VM state for early trials | No host TPM passthrough; the host uses its own keys/PCRs |
| Disk/network | Synthetic image files, read-only bundle ESP, network off by default | No physical host disk, bridge, clipboard, host home or GPU/PCI passthrough |

The VM runs as an ordinary user with current QEMU/firmware. CPU virtualisation
means guest instructions run on the physical CPU under hypervisor control;
it does not give the guest unrestricted access to host disks or USB. Device
passthrough is a separate trust boundary. A later PCI/GPU trial requires
IOMMU support, suitable device groups, driver/reset behaviour and a specific
authorised test device. Those steps were not performed here. TCG is not a
hard sandbox either: the current QEMU security policy gives the emulation
use case no isolation guarantees.

## Measurement and next rendering step

Measure three repetitions per backend with the same source hash, resolution,
firmware, clean VM state and host load. Record cold boot → compositor ready,
first completely acknowledged frame, input → complete new frame and idle CPU.
Use monotonic time for intervals; `launch.json` contains only the startup
configuration, not fictional rendering measurements. A timeout counts as an
error. Do not treat TCG/WHPX results as interchangeable with existing native
gates.

The next rendering optimisation can proceed without broader app permissions:
process a small bounded number of already available SUR1 messages per
compositor turn and check clock, terminal state and input/control between
batches. Retain sender/capability/epoch/length checks and COMMIT acknowledgement;
prevent unbounded draining/starvation. Test stalled clients, revocation
mid-batch and checking of every acknowledged frame. This changes timing and
requires native acceptance; here it is only a proposal. Then versionable
dirty-region transport; longer term, a capability-bound transfer buffer with
explicit ownership, bounded length and revocation. Do not give apps raw
framebuffer or DMA permission for speed.

## Executed checks

- PowerShell 7.5.4 on the Linux cloud executor: 33 synthetic acceleration-policy/
  QMP-reply tests passed (success, absent backend, initialisation error,
  timeout, explicit choice, no fallback, no probe for CheckOnly,
  invalid/missing/duplicate QMP acknowledgement and sequential handshake with
  bounded event processing).
- The six launcher checks with a synthetic bundle and fake QEMU binary ran
  on that same Linux PowerShell; a changed helper hash is rejected before
  execution. Those tests started no actual VM.
- The actual Linux QEMU binary rejects the WHPX initialisation trial with
  `invalid accelerator
  whpx`; the helper reports `Ready=false` and exit 1. This tests process/error
  handling, not a Windows hypervisor.
- Package host tests cover hash binding of the helper and retention of the
  session-preview profile. The integrator retains the final command/result.
- Three synthetic subprocess trials check exact LF-only QMP request bytes
  and accept only confirmed paused initialisation; `running` and a missing
  quit acknowledgement are rejected. This is Linux process evidence without
  hypervisor initialisation, not Windows evidence.
- Windows PowerShell 5.1, an actual WHPX session, KVM/HVF, physical PC/
  peripherals and a speed comparison: **not executed**. These cloud results
  do not describe the Windows PC concerned.

Rollback: revert the launcher/helper/package change as a separate commit;
the explicit `-Acceleration Tcg` meanwhile retains the old software profile.
No host feature or firmware was modified.

## Primary sources

Accessed 2 November 2026; QEMU web documentation lists version 11.1.50.
This is a development manual, not evidence that all options work identically
in the minimum permitted QEMU 10.2.1 build.

- QEMU Project. *Windows Hypervisor Platform*. Requirements, WHPX acceleration,
  legacy VGA/MMIO/SIMD limitations and security-domain mitigation.
  https://www.qemu.org/docs/master/system/whpx.html
- QEMU Project. *Introduction*. Hardware-acceleration backends and hosts.
  https://www.qemu.org/docs/master/system/introduction.html
- QEMU Project. *Invocation*. Backend selection and initialisation/fallback.
  https://www.qemu.org/docs/master/system/invocation.html
- QEMU Project. *Security*. Virtualisation, TCG boundaries, device passthrough
  and ordinary user privileges.
  https://www.qemu.org/docs/master/system/security.html
- QEMU Project. *QEMU TPM Device*. TPM passthrough does not freely share the
  physical TPM with host/guests; software TPM is a separate route.
  https://www.qemu.org/docs/master/specs/tpm.html

## Local Windows findings of 2 November 2026

The authorised local Codex session reports a Ryzen 9 9950X3D with 64 GiB
and Windows 11 25H2 for the PC concerned. WHPX initialises without changing
host settings. On the old bundle, one cold trial took 10.453 s with WHPX
versus 19.952 s with TCG to exactly the same complete frame. On source
`7648a8f`, the five-step route passed with WHPX in 43.824 s; screen transitions
took 6.856–9.107 s. That is execution evidence for that specific source,
not a product-latency claim. Raw local artifacts remain with that session;
the reported source/firmware/measurement boundary is recorded in
[local-pc-reported.json](../verification/session-desktop-20261102/local-pc-reported.json).

The first HMP helper incorrectly timed out on Windows: input could arrive
before monitor readiness; LF alone did not resolve it. The current helper
waits for the QMP greeting and each correlated reply. Version `51d883e` also
fixed the platform-sorting error in source-v3; byte-identical Windows and
Linux checkouts have since directly produced the same source hash. No input
source or integrity check was removed.

Test directories belong outside synchronisation: OneDrive created additional
hardlinks during the local trial, which the harness correctly rejected.
The follow-up trial uses LocalAppData; hardlink checks remain active. The
final Windows runtime on `344dc0a` is recorded separately in the final
evidence; earlier results are not presented as execution on the new source.

The definitive local series on `344dc0a` is complete: 3 WHPX and 3 TCG runs,
all 8 phases and 48 app images exact, exit codes 0. An actual Windows start
with Auto/WHPX/SDL and launcher exit 0 followed. WHPX first account:
mean 6.625 s (6.481–6.823); guest → desktop: 4.070 s (3.932–4.231).
TCG: 13.356 s and 10.023 s respectively. Complete route: 31.481 s versus
71.982 s. Three starts per backend on the same PC, not controlled for
background load. Further source/package hashes and the separate idle-CPU
observation are in the local final report above.

## Known ABI 1.11 and existing idle-wait routes

2 November 2026, fix baseline `2bb84c2`. The window kernel provides IPC wait
under ABI 1.11, but the existing keyring and volume services enabled idle wait
only on 1.10. Those two services therefore kept polling empty queues in this
profile. The shared syscall-free SDK helper `supports_ipc_wait` now recognises
only known 1.10 and 1.11 profiles; both existing selectors use it. General
ABI compatibility, for example RTC 1.9, does not imply a wait facility.
Unknown major/minor versions remain excluded.

Keyring's RANDOM/administration exclusions, existing queue conditions,
absolute deadlines, capability roles, error handling and processing of
ongoing broker work are unchanged. RANDOM or administration ingress retains
its existing polling path. There is no change to GUI, SUR/INP protocols or
source hardware polling.

Targeted SDK host tests, including negative ABI selection without a host
syscall, and strict SDK Clippy pass. Both actual scalar x86_64-unknown-none
services build and pass ELF inspection. Logs and service hashes are under
`build/abi-wait/logs/`. Independent diff review found no blocker; no VM was
started for this candidate. Final integrated native gates must still execute
the 1.11 wait path.

This fixes two concrete compatibility errors and proves neither complete VM
idling nor latency improvement. The compositor, both GUI apps and window-init
still contain their own polling loops; their further idle configuration and
actual idle-CPU measurement remain open. CPU usage of 100% during active
drawing is not an idle measurement, and no U02 points are awarded.
