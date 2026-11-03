# NVMe PCI-probe — 3 november 2026

**Bewijsniveau:** broncode, testcode/typecheck en freestanding compilecheck.
Geen controller gestart, geen block-I/O, QEMU-run of fysieke hardwaretest.

## Bron en resultaat

- Branch: `copilot/universal-driver-framework`
- Basiscommit: `af7a4f7b1291bea30d91b59a32deb8921aac77cd`
- Geteste implementatiecommit: `6733559898e1c95536666472422c000e02f75599`
- Remote branch is bij de implementatiepush op exact die SHA gecontroleerd.
- Driver: standalone `no_std` crate `drivers/nvme`, niet opgenomen in de
  kernelworkspace of volumeconsumer.
- Functie: PCI class match `01:08:02`, weigeren van meerdere matches,
  devicegebonden BAR0-grantcontrole en read-only CAP/VS-decode.
- Geen busmaster-enable, DMA-allocatie, queue-, Identify-, namespace- of
  read/write/flushroute.

## Uitgevoerde controles

| Commando | Exit | Resultaat |
|---|---:|---|
| `$HOME/.cargo/bin/cargo +1.90.0 fmt --manifest-path drivers/nvme/Cargo.toml -- --check` | 0 | Formattering in orde |
| `CARGO_TARGET_DIR=build/nvme-freestanding $HOME/.cargo/bin/cargo +1.90.0 check --manifest-path drivers/nvme/Cargo.toml --target x86_64-unknown-none --offline --locked` | 0 | Freestanding drivercheck geslaagd; log `/tmp/nieuw-os-nvme-none-check-final.log`, SHA-256 `dde62274cb6ef73ab0e46a1c6cfa58e8923ea8a378169c7bdc8f7bff38e3e928` |
| `CARGO_TARGET_DIR=build/nvme-clippy $HOME/.cargo/bin/cargo +1.90.0 clippy --manifest-path drivers/nvme/Cargo.toml --all-targets --offline --locked -- -D warnings` | 0 | Strikte Clippy geslaagd; driver- en testcode zijn gecompileerd/typechecked; log `/tmp/nieuw-os-nvme-clippy-final.log`, SHA-256 `f2cc04f0d2d8797e2f2195d6bfb0bbe3f11d012e8abdd36a57290ff099ea286d` |
| `$HOME/.cargo/bin/cargo +1.90.0 fmt --manifest-path drivers/nvme/Cargo.toml -- --check && git diff --check` | 0 | Formatter- en patchwhitespacecheck geslaagd |

## Geblokkeerde hosttest

Eén tijdelijke userspace-only sysrootpoging gebruikte de reeds beschikbare
toolchain en `/tmp/nieuw-os-nvme-sysroot-2026-11-03`; er is geen sudo of
systeemwijziging gebruikt. Deze poging vond plaats vóór de laatste, op de
implementatiecommit aanwezige register-rangetest. Het hosttestproces bereikte
de linker maar beëindigde met exit 101: `rust-lld` probeerde het absolute pad
`/usr/lib/x86_64-linux-gnu/libc_nonshared.a` te openen. De hosttests zijn dus
niet uitgevoerd. Log: `/tmp/nieuw-os-nvme-host-test-final.log`, SHA-256
`2eed3d05b7b7616eacbe0771ef35763b5b5561c08d51a1cce17224856e7051eb`.

Die ene tijdelijke oplossing werkte niet; er is niet verder aan de host- of
systeemomgeving gesleuteld. De finale testcode typecheckt wel onder strict
Clippy, maar de hosttests zijn niet op de implementatiecommit uitgevoerd en
typecheck is geen hosttestresultaat.

QEMU was niet beschikbaar (`qemu-system-x86_64` ontbreekt) en fysieke hardware
is niet getest. De volgende echte NVMe-stap blijft geblokkeerd op een
kernel-owned devicegebonden DMA-grant en expliciete busmaster-/interrupt-
lifecycle; dit checkpoint voegt die autoriteit niet toe.
