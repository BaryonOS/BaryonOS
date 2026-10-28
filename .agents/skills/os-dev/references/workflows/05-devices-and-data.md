# Drivers, persistent data and network interfaces

Use [drivers](../themas/13-drivers.md), [platform](../themas/14-platform.md),
[storage](../themas/15-opslag.md), [filesystems](../themas/16-bestanden.md),
[networking](../themas/17-netwerk.md), [USB/input](../themas/18-usb-invoer.md),
[graphics](../themas/19-grafisch.md) and [audio](../themas/20-audio.md).

## Common driver contract

Identify the actual device/bus/specification version and discovery method. Define
register access widths, byte order, memory attributes, DMA ownership, descriptor
alignment, queue limits and interrupt policy. Inspect current repository abstractions
rather than hard-coding an emulator address into a portable driver.

Specify bounded polling, timeouts, unsupported-feature behavior and partial-init cleanup.
Treat device-provided lengths/indices/status as input requiring validation. Review
resource reuse and teardown while DMA or interrupt work is outstanding. Do not treat
an IOMMU, a driver boundary or a feature bit as available until detection and setup
are evidenced by implementation and tests.

## Storage and filesystems

Default to a disposable image with known content. Verify block size, capacity, integer
arithmetic and offsets before reads/writes. Keep raw block operations separate from
partition parsing and filesystem interpretation. Begin read-only when that suits the
project's development stage; do not imply read support proves write consistency.

For writes, state ordering, durability/flush assumptions, atomicity, crash recovery,
metadata validation and rollback. Test truncated/corrupt images, out-of-range blocks,
full storage, I/O failures and interrupted operations. Define the expected recovered
state before fault injection. Never use a real host disk as an unapproved test target.

## Network and external input

Use isolated, reproducible test inputs or the project's explicitly authorized emulator
network. Parse incrementally with checked lengths, bounded queues and an explicit
state machine. Address checksum/format checks, timeouts, fragmentation/reassembly if
supported, resource exhaustion and malformed packets. Distinguish protocol support
from security hardening or broad internet interoperability.

Do not expose debug services, bridge onto a real LAN or introduce remote telemetry as
a convenience without task-specific authorization. No source snippet should leak keys,
logs or private project material to an external endpoint.

## Input, rendering and audio

Keep controller/protocol parsing separate from user-facing mapping and policy. Test
unexpected scan-code/event sequences, clipping/buffer bounds, format/stride assumptions,
unsupported modes, buffer underruns and recovery as applicable. A framebuffer demo is
not a window manager; a tone is not a complete audio stack. State the actual milestone.

## Acceptance

Document the tested emulator/device configuration, success criteria and negative cases.
Check missing devices and repeated initialization/teardown. Record what requires real
hardware validation separately. Inspect source-specific licensing before copying code.
