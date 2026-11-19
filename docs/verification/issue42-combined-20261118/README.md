# H02 verification of the combined Rust 1.100 source

All six existing native H02 cases pass at source
`2a08f4a33eff63b354821f6892e1be28f7602c9b`, source fingerprint v3
`25dd6d69621f1d9d4c8ed353633b3db62818562fa6ce50efa4445207505adb44`.
The source fingerprint remained unchanged throughout execution. Twelve separate
build/run commands returned exit 0; the outer Token Guard receipt records exit
0 and 55.209 seconds. All six QEMU invocations returned raw exit 33 without a
timeout and passed artifact binding.

| Case | Scope |
| --- | --- |
| xHCI No-Op | Actual requester-bound command fetch and completion |
| Unmapped write | Actual VT-d denial; independent guard remains unchanged |
| Missing DMAR | Fail-closed refusal before unsupported DMA execution |
| USB reports and unplug | Actual keyboard/mouse reports, held-key unplug and stale-token rejection |
| Read-only mapping | Actual DMA write-permission fault and unchanged guard |
| HID IPC | Actual protected IPC with explicitly synthetic HID completions |

The runtime is QEMU 10.0.13, `pc-q35-8.2`, qemu64, TCG, one CPU and 256 MiB,
with the matched local OVMF pair. It is separate from hosted CI. The USB
transport and synthetic HID IPC cases remain distinct proofs; this receipt
does not establish an operational USB-to-UI broker, physical compatibility,
H02 acceptance, a score increase or issue #42 closure.

`checks.json` retains every command, environment, raw exit and log digest.
`capture-index.json` records the captured files and six guest outcomes; 25
actually loaded runtime artifacts were independently rehashed against their
guest receipts. ELF, firmware and disk binaries remain local and are excluded
from this archive. Build manifests, reports, exact serial/stderr bytes and
losslessly compressed build/run logs are preserved. Known current credential
values and common token/private-key patterns were absent from captured text.
`SHA256SUMS` verifies the archived bytes. No original evidence was rewritten.

For reproduction, check out the exact source above, use the repository's
documented Rust 1.100/QEMU/OVMF setup, and execute the individual build/run
commands in `checks.json` with build/target directories inside the checkout.
`reproduce.py` preserves the exact local orchestration used for this run; its
local checkout and VGA-ROM paths must match the execution environment. The
existing runners perform their own bounded runtime and oracle validation.

## Remaining implementation boundary

The current native xHCI profile explicitly excludes `milestone-m3`, SMP,
framebuffer and other bootstrap consumers (`kernel/src/main.rs`, lines 18–28).
Its PCI adapter assumes exclusive bootstrap execution with interrupts disabled
(`kernel/src/m2/xhci.rs`, lines 45–64). Its VT-d domain leaves other requester
contexts absent, and its DMA/translation pages remain retained until VM exit.
The userspace `hid::Source` expects an authenticated transport owner and does
not grant device, DMA or focus authority.

An integrated long-lived path therefore requires an accepted kernel/service
completion handoff: process/requester ownership, bounded queue and overflow
semantics, completion provenance, cancellation and detach acknowledgement,
process-failure recovery, DMA teardown and coexistence with other devices.
Removing the existing compile guards or assigning authority through an
untrusted integer would violate the current contract. No shared API, privilege
or lifecycle policy was changed by this verification task. Real IRQ handling,
general hotplug/reset, malformed-device coverage and the authorised physical
reference matrix remain open.

The recalled low-speed polling-interval advisory is already fixed in current
`boot_interface.rs`: low-speed endpoints require at least 10 ms, while
full-speed endpoints require at least 1 ms. No duplicate fix was introduced.
