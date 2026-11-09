# DRV-USB-HID-INTERFACE-BINDING

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
