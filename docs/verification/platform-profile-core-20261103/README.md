# M1 platformprofielkern — hostbewijs

**Status:** hosttests en statische controles geslaagd; geen kernel-/dienstintegratie
of native hardwareprofilering.

- **Bron:** `c4b3e57e` (`feat(platform): add evidence-aware hardware profile core`)
- **Gewijzigd:** `Cargo.toml`, `Cargo.lock`,
  `crates/platform-profile-core/Cargo.toml`,
  `crates/platform-profile-core/src/lib.rs`
- **Architectuur:** Rust 1.90.0, `no_std`, `forbid(unsafe_code)`;
  afhankelijkheid op het bestaande gevalideerde `boot-protocol`.
- **Gemeten invoer:** UEFI `ConventionalMemory`-paginatelling en gevalideerde
  GOP-resolutie/stride/pixelindeling. De geheugensom gebruikt checked
  vermenigvuldiging en optelling.
- **Niet gemeten:** CPU/topologie, SIMD, GPU, opslag, refresh/HDR, media,
  AI-accelerator en energie. Die eigenschappen blijven `Unknown`; onbekend
  betekent niet dat hardware ontbreekt.
- **Niet aangetoond:** ophalen via native kernel/service-API, automatische
  profielselectie, Legacy/Modern/Advanced-acceptatie, VM-/hardwaregedrag of
  performancewinst.

## Controles

| Commando | Resultaat |
|---|---|
| `cargo +1.90.0 check --offline --locked -p platform-profile-core --tests` | exit 0 |
| `cargo +1.90.0 test --offline --locked -p platform-profile-core` | exit 0; 4 tests |
| `cargo +1.90.0 fmt --package platform-profile-core -- --check` | exit 0 |
| `cargo +1.90.0 clippy --offline --locked -p platform-profile-core --all-targets -- -D warnings` | exit 0 |
| `git diff --check` | exit 0 |

De eerste testpoging kon niet linken: de host had geen `cc`. De uiteindelijke
test gebruikte GCC 15.2 en `libc6-dev` uit Ubuntu-pakketten, uitsluitend
uitgepakt onder `/tmp/nieuwos-profiler-toolchain`; er is niets
systeemwijd geïnstalleerd. De geslaagde test is uitgevoerd met:

```sh
PATH=/tmp/nieuwos-profiler-toolchain/root/usr/bin:/home/simon/.cargo/bin:$PATH \
RUSTFLAGS='-C link-arg=-L/tmp/nieuwos-profiler-toolchain/linker -C link-arg=-L/tmp/nieuwos-profiler-toolchain/root/usr/lib/x86_64-linux-gnu -C link-arg=-B/tmp/nieuwos-profiler-toolchain/root/usr/lib/gcc/x86_64-linux-gnu/15/' \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/tmp/nieuwos-profiler-toolchain/root/usr/bin/x86_64-linux-gnu-gcc-15 \
cargo +1.90.0 test --offline --locked -p platform-profile-core
```

De log van die exacte test:
`/tmp/nieuwos-profiler-toolchain/profile-test.log`,
SHA-256 `83eef3e1d068da4f623963c097d888c9089da7afedd3e232e450feac68f60a21`.

Deze hosttests toetsen aggregatie en onbekende waarden; ze zijn geen
UEFI/QEMU- of fysieke hardwareproef.
