# DRV-USB-HID-INTERFACE-BINDING

## Mandatory HID descriptor validation — 14 November 2026

Base: `2009fcf79a6f2f6d542a341e43e6d568cd8932c9`, issue #42.

The selected alternate-zero interface now requires one HID descriptor before
its endpoints. The descriptor's declared subordinate count must match its
complete byte length, and its first entry must advertise a nonempty Report
descriptor. Missing, misplaced, duplicate or malformed HID descriptors return
`HidDescriptor`; a neighboring interface or alternate cannot supply the entry.
The returned `report_descriptor_length` is untrusted metadata, not allocation,
transfer, device-lifetime or input authority. HID version, optional descriptor
semantics and the actual Report descriptor remain separate validation duties.

Primary source read: USB Implementers Forum. (2001). *Device class definition
for human interface devices (HID)* (Version 1.11, sections 6.2.1 and 7.1).
<https://www.usb.org/sites/default/files/hid1_11.pdf>.

The missing-HID regression first failed against the unchanged base selector
(exit 101). The corrected crate passed **31 tests**, formatting, strict Clippy,
and the freestanding build. Tests exercise all HID byte lengths, declared
counts, first descriptor-type values and report-length words; optional-entry
framing reaches the 255-byte descriptor boundary. Existing endpoint/interval
tests remain intact. No tests are ignored.

Commands, with Rust 1.90.0 and an isolated
`CARGO_TARGET_DIR=build/issue42-next-target`:

```sh
cargo +1.90.0 test --offline --locked --manifest-path drivers/usb/Cargo.toml
cargo +1.90.0 fmt --manifest-path drivers/usb/Cargo.toml -- --check
cargo +1.90.0 clippy --offline --locked --manifest-path drivers/usb/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 build --offline --locked --manifest-path drivers/usb/Cargo.toml --release --target x86_64-unknown-none --lib
git diff --check
```

Tested source SHA-256 hashes:

| File | SHA-256 |
|---|---|
| `src/boot_interface.rs` | `343b06ac7f5b2bc75212f57fa4d4c29eaaa32c29727b39802f851e26aa000b29` |
| `tests/boot_interface.rs` | `de19b7a088caa26ec1ac5a0126d4da965559236284b7d7ee718620aa92708d86` |

Ignored logs under `build/issue42-next-checks/` in the component worktree:

| Log | Exit | SHA-256 |
|---|---|---|
| `baseline-missing-hid.log` | 101 (expected) | `1c7920011e94ef4f4d07af66f8b32c91199a6ddd097fd151d3e8e11f83d6b54d` |
| `usb-tests.log` | 0 | `aea31f833087ee82c534dc456db39816208573ade45d48c59bdcdc7b4aafca1d` |
| `usb-fmt.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `usb-clippy.log` | 0 | `841f30a51ade72a3b1f167fde478fb53ffd35e95e4abcdb8e921dbbf162d37ca` |
| `usb-freestanding.log` | 0 | `1917677c44bb3913ac990dec0776cc102f839ce06dddaba61b9e44cfc251e291` |

This is host/component and cross-compilation evidence. Full-system preflight,
build/smoke, native USB execution, DMA isolation, unplug/reset/hotplug and
physical reference-hardware checks were not run in this worktree. H02 remains
open with no additional acceptance points. Reverting this scoped change restores
the earlier selector contract; the low-speed interval repair is preserved.

## Low-speed interval correction — 14 November 2026

Base: `b8a7e83658bdbcd4970c26f753dda1b342fce0d4`, issue #42.

The selector rejects interrupt endpoint descriptors advertising `bInterval`
below 10 ms at low speed. Full-speed intervals remain 1..255 ms. This check
applies to every endpoint of the selected interface, including interrupt OUT.
The existing exhaustive interval test checks all 256 values at both speeds.
Packet-size limits, interface isolation and the HID boot mouse range are
unchanged. This metadata validation adds no device or input authority.

This corrects the historical 1..255 ms claim below. USB 2.0 section 5.7.4
(printed page 51) restricts a low-speed endpoint's requested period to
10..255 ms. Table 9-13's general full-/low-speed field range must be read with
that bus-access rule. The host's permission to schedule shorter actual
periods does not expand the endpoint's advertised range.

Source read: USB Implementers Forum. (2000, April 27). *Universal Serial Bus
specification* (Revision 2.0, section 5.7.4, p. 51).
[Official distribution](https://www.usb.org/document-library/usb-20-specification);
[readable specification mirror](https://www.alsa-project.org/files/pub/manuals/usb/usb_20.pdf).

### Executed verification

After the initial missing-Cargo attempt, Rust 1.90.0 became available. The
corrected exhaustive test was first run against a disposable crate copy
containing the original production source from the base above. It failed as
expected (exit 101): low-speed interval 1 returned `Ok(1)` instead of
`Err(Endpoint)`. The fixed source then passed all 29 USB tests, formatting,
Clippy with warnings denied, and the freestanding release build (exit 0
each). No test was ignored.

Commands, from the component worktree, with `CARGO_BUILD_JOBS=2` and separate
`CARGO_TARGET_DIR` values `build/issue42-baseline-target` and
`build/issue42-fixed-target` for the baseline and fixed source respectively:

```sh
cargo +1.90.0 test --offline --locked --manifest-path build/issue42-baseline/Cargo.toml --test boot_interface all_packet_words_and_intervals_obey_speed_without_confusing_packets_with_reports
cargo +1.90.0 test --offline --locked --manifest-path drivers/usb/Cargo.toml
cargo +1.90.0 fmt --manifest-path drivers/usb/Cargo.toml -- --check
cargo +1.90.0 clippy --offline --locked --manifest-path drivers/usb/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 build --offline --locked --manifest-path drivers/usb/Cargo.toml --release --target x86_64-unknown-none --lib
git diff --check
```

Tested SHA-256 source hashes:

| File | SHA-256 |
|---|---|
| `src/boot_interface.rs` | `4a0e4dcad831ac6cba51cba4700162846995fdf5ec681e5ef882f66718fc692b` |
| `tests/boot_interface.rs` | `e0b9245c777b1b565901bfcd63b931432ec68717133b741a722e1615402a9e18` |

Ignored local logs under `build/issue42-verification/`:

| Log | Exit | SHA-256 |
|---|---|---|
| `baseline-regression.log` | 101 (expected) | `dfa30559b90e2516d9f40f96e4c686c97b77ad506a7d0e4a082fd239db26876a` |
| `usb-tests.log` | 0 | `0bdad30806b0794293511f1a430d3469d04003d92fa5e6f2ee1e194f922d0e54` |
| `usb-format.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `usb-clippy.log` | 0 | `973380aa963af0351a0611851b870405f55ad99ac7c1f65894c612f2da7a60ee` |
| `usb-freestanding.log` | 0 | `120e6c9092f0732a2c38af99fde4c0e3a38db01e6eea630c85d47258874998f6` |
| `diff-check.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

This is component host/cross-compilation evidence. Full-system preflight,
build and QEMU smoke were not run in this component worktree. Native USB
execution, hotplug and physical hardware acceptance remain open; H02
receives no acceptance points from this parser correction.

## Historical implementation receipt — 3 November 2026

Basis `6d523964ab85638d4dd981bcdb19580d18646fab`, branch
`copilot/driver-usb-hid-interface`, 3 november 2026.

## Bereik

De nieuwe bounded selector bindt een expliciet gekozen interface, alternate
setting nul, HID-class 3/subclass 1/protocol 1 of 2 aan precies een
interrupt-IN-endpoint. Configuratiewaarde is niet nul en wTotalLength moet
exact de aangeboden transfer beslaan. Alleen low/full-speed is representabel,
geen high/superspeed of geclaimde controllerondersteuning.

Per gekozen interface worden bNumEndpoints, zevenbyte endpointvorm,
endpointnummer/richting/reservedbits, interrupttype, packetwoord, interval,
dubbele adressen en meerdere IN-endpoints gecontroleerd. OUT mag dezelfde
endpointnummers gebruiken als IN, maar wordt nooit als input geselecteerd.
Een volgende interface of alternatieve setting sluit het huidige bereik.
Een ontbrekend endpoint kan niet uit de volgende interface worden geleend.
Een tweede match wordt geweigerd. Malformed transferstaarten geven een fout,
ook na een eerder volledig gevonden match.

Packetgrenzen: 1..8 low-speed, 1..64 full-speed, bInterval 1..255 als
geadverteerde millisecondenwaarde. Dit is geen daadwerkelijke hostplanning.
Een HID-report kan meerdere USB-transacties beslaan: maxPacketSize hoeft
dus niet minimaal de reportlengte te zijn. De eerdere WIP-aanname dat een
keyboardpacket minimaal acht bytes moest bevatten is op basis van de
primaire HID-specificatie verwijderd, niet door controles te omzeilen.
De uitvoerende transportconsument moet volledige reports samenstellen.

Dit is **driverbron + hosttest + freestandingcompilatie**, geen USB-runtime.
Geen discovery, configuratiecommando, SET_PROTOCOL, DMA, IRQ, rechten,
kernel/SDK-write of OS-inputpublicatie. Uitvoer is kopieerbare metadata,
geen token of autoriteit. De caller moet device-identiteit, configuratiekeuze,
snelheid, HID/reportdescriptors, alle overige interfaces/endpoints en actieve
lifetime onafhankelijk valideren. Alleen gekozen-interface-endpoints worden
semantisch gecontroleerd; dit is geen volledige USB-configuratievalidator.
Framingfoutoffsets tellen vanaf de descriptorlijst na de negenbyte header.

## Bronnen

USB Implementers Forum, *Device Class Definition for Human Interface Devices
(HID)*, firmware specification versie 1.11, 27 mei 2001:
https://www.usb.org/sites/default/files/hid1_11.pdf

Daadwerkelijk gelezen: sectie 4.3 protocolcodes, 5.6 en 8.4
transaction/reportonderscheid, Appendix E.3/E.5/E.7/E.9 voor interface- en
endpointvelden. PDF-SHA256:
`206272dacaf01ff34beda038b8ed59abbb02f028394896345c393bc026795d36`.
De oude HID-bron gebruikt "high-speed" voor het toenmalige 64-byteprofiel;
dit is hier uitdrukkelijk full-speed en geen USB 2.0 high-speedclaim.

USB 2.0, USB-IF, 27 april 2000, tabel 9-13 is de norm voor de
speedafhankelijke endpointvelden. Actuele officiele distributie:
https://www.usb.org/document-library/usb-20-specification
https://www.usb.org/sites/default/files/usb_20_20250603.zip
De base-PDF is opgehaald (SHA256
`d39698a33486c399124af92bd02e4f978fd9a836b5cf4e52e6e4633eb1d89f61`)
maar de lokale PDF-reader kon de versleutelde streams niet ontsluiten.
Daarom wordt die download **niet** als gelezen primaire passage geclaimd.
Packetgrenzen sluiten aan op de gelezen HID-bron; het intervalveld is
gecorroboreerd met Arm/Keil *USB Component: Endpoint Descriptor*:
https://www.keil.com/pack/doc/mw/USB/html/_u_s_b__endpoint__descriptor.html
Dit vervangt geen native implementatie of onafhankelijke standaardreview.
Geen upstreamcode/specificatietekst overgenomen.

## Acceptatie

Vanuit `/tmp/nieuw-os-driver-usb-interface`:

```sh
export PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export CC=/tmp/nieuw-os-c-toolchain/bin/cc
export CARGO_TARGET_DIR="$PWD/build/hid-interface-host"
cargo +1.90.0 test --offline --locked --manifest-path drivers/usb/Cargo.toml
cargo +1.90.0 fmt --manifest-path drivers/usb/Cargo.toml -- --check
cargo +1.90.0 clippy --offline --locked --manifest-path drivers/usb/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 build --offline --locked --manifest-path drivers/usb/Cargo.toml --release --target x86_64-unknown-none --lib
git diff --check
```

Alle vijf commando's exit 0. Zeven bestaande framing-unittests en negen
nieuwe bindingtests geslaagd, nul doctests, niets genegeerd.
De matrix toetst alle 65.536 packetwoorden voor beide snelheden/protocollen,
alle 65.536 total-lengthwoorden, alle endpointadressen, attributes en
intervalbytes, alle endpointcounts, alternatieve settings en
class/subclass/protocolwaarden, verkeerde descriptorvormen, duplicaten,
ambiguiteit, interfacegrenzen en malformed tails.

Logs onder `/tmp/nieuw-os-hid-interface-`, SHA-256:

| Suffix | SHA-256 |
|---|---|
| `test.log` | `acf39b75cebd85c02ba199a60b0541409c3ea4bb00ba8d67ca9827004319effe` |
| `fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `clippy.log` | `a151e3e576bf7414f7953dfe0c355cf2c872f2a4b5bf07c2efe5173707d283c5` |
| `none.log` | `bb0e39c4090f3aaccc28d8cf4e6de6448521d1c8ccfc0eb9b094eae2270d6ea2` |

Queued USB-framing/bootreportbranches zijn niet gewijzigd of overgenomen.
Geen afhankelijkheden toegevoegd, geen installatie, geen PR zonder lease.
Geen centrale writes of scorepunten. Native USB/hardware/menselijke
acceptatie niet uitgevoerd; kernel-execution/xHCI-enumeration zijn de
runtimeafhankelijkheden. Rollback: verwijder uitsluitend de module/export,
de eigen tests en dit bewijsbestand.
