# DRV-RNG-USED-BOUNDARIES

Basis `d10b5cdaf56774a39e3ef10463f674c6188a463a`, branch
`copilot/driver-rng-used-boundaries`, 3 november 2026.

Alleen twee tests van de bestaande `validate_used` en dit bewijsbestand.
Geen productiecode, PCI-I/O, MMIO, kernel-/SDK-adapter, dependency of
DMA-/resetsemantiek gewijzigd.

De tests controleren alle toegestane aanvraaggroottes 1..256 tegen
antwoordlengtes 0..257 en `u32::MAX`, op gewone, ringslot- en indexwrapgrenzen.
Een kort geldig antwoord behoudt exact zijn lengte, nul is een I/O-fout,
een overrun of verkeerde descriptor-ID een protocolfout. Alle 65536
indexdelta's worden vanaf drie grensindices gecontroleerd: uitsluitend
delta 1 is een completion, delta 0 blijft Pending en alle andere delta's
worden geweigerd. De tests controleren ook dat Pending geen stale
descriptorinhoud interpreteert.

```sh
export PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export CC=/tmp/nieuw-os-c-toolchain/bin/cc
export CARGO_TARGET_DIR="$PWD/build/driver-rng-used"
cargo +1.90.0 test -p virtio-rng-driver --locked --offline
cargo +1.90.0 fmt -p virtio-rng-driver --check
cargo +1.90.0 clippy -p virtio-rng-driver --all-targets --locked --offline -- -D warnings
cargo +1.90.0 build -p virtio-rng-driver --lib --release --target x86_64-unknown-none --locked --offline
git diff --check
```

Definitief alle exit 0: 5 tests geslaagd, 0 doctests. Eerste fmt-check gaf
exit 1 op de nieuwe arrayopmaak; hersteld met `cargo +1.90.0 fmt -p
virtio-rng-driver`, daarna bovenstaande controles opnieuw uitgevoerd.
Geen tools geinstalleerd.

Logs onder `/tmp/nieuw-os-rng-used-`, SHA-256:

| Suffix | SHA-256 |
|---|---|
| `test.log` | `d7d1e5e65fa4ecb6c7aa718e06725141a23edd53f11aa515e7599cd60bf2dea6` |
| `fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `clippy.log` | `92523215d2943e7114a247bf24147dfe2b2637a6c30c3d9c56e27f061788a90d` |
| `none.log` | `05c30831661a026d0211b53a2266e84a7671805165d1e41f162fcf7b58878bf3` |

Bewijsgrens: pure hostvalidatie van device-antwoorden, niet de runtimekopie,
entropykwaliteit, cryptografische geschiktheid of hardware-isolatie.
Freestanding librarybuild is geen uitvoering. Nieuwe QEMU-, fysieke en
menselijke controles niet uitgevoerd. Geen nieuwe acceptatiepunten; centraal
assessment bij integrator. Revert van tests/bewijs raakt productiegedrag niet.
