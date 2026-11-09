# DRV-USB-FRAMING-BOUNDARIES

Basis `ab85d6339ce5bdddb10a2de171d0c1b04e4b62ff`, branch
`copilot/driver-usb-framing-boundaries`, 3 november 2026.
Alleen nieuwe `tests/framing.rs` en dit bewijsbestand; geen productieparser,
controller, enumeratie of kernel-/inputinterface gewijzigd.

Twee integratietests gebruiken uitsluitend de bestaande publieke
DescriptorIter/ParseError-API. Iedere bLength-waarde 0..255 wordt op alle
transferafkappingen getoetst, na nul, een of zeven geldige prefixdescriptors.
Fouttype, offset, remaining en lengte moeten exact kloppen. Geldige ranges
blijven aantoonbaar geleende slices met dezelfde pointer, type en payload.
Na eind/fout blijft de iterator gestopt. Een apart geldig-ogend suffix mag
na een foute lengte nooit alsnog worden blootgelegd.

```sh
export PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export CC=/tmp/nieuw-os-c-toolchain/bin/cc
export CARGO_TARGET_DIR="$PWD/build/usb-framing"
cargo +1.90.0 test --offline --locked --manifest-path drivers/usb/Cargo.toml
cargo +1.90.0 fmt --manifest-path drivers/usb/Cargo.toml -- --check
cargo +1.90.0 clippy --offline --locked --manifest-path drivers/usb/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 build --offline --locked --manifest-path drivers/usb/Cargo.toml --release --target x86_64-unknown-none --lib
git diff --check
```

Definitief alle exit 0: 7 unit- en 2 integratietests, 0 doctests.
Eerste strict Clippy exit 101 wegens een overbodige vec in de nieuwe fixture;
vervangen door een array, daarna alle bovenstaande gates opnieuw uitgevoerd.
Geen installatie, dependencywijziging of rootworkspacewijziging.

Logs onder `/tmp/nieuw-os-usb-framing-`, SHA-256:

| Suffix | SHA-256 |
|---|---|
| `test.log` | `4eaaa39c9c85a0639f48582ccdcd83c3c05da751e4819f67f91edfdce897ecdc` |
| `fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `clippy.log` | `1a3485d77aa140526df1d3907b34a4dfe315338ccd4c77eaae520c64749d8cef` |
| `none.log` | `c803d2904754d978837e455d2a7eaa651fca0947d683c37b4e43d067c32bb949` |

Bewijsgrens: pure transferbytes op de host. Geen USB-device, xHCI-rings,
enumeratie, class-policy of HID-input bewezen; geen nieuwe foundation.
Geen nieuwe VM-, fysieke of menselijke tests; librarybuild is geen uitvoering.
Geen nieuw criterium/scorepunten; centraal assessment bij integrator.
Revert van tests/bewijs raakt productiegedrag niet.
