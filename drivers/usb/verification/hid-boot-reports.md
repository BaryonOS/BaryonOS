# DRV-USB-HID-BOOT-REPORTS

Basis `6d523964ab85638d4dd981bcdb19580d18646fab`, branch
`copilot/driver-usb-hid-boot`, 3 november 2026.

## Contract en bewijsgrens

Nieuwe driver-eigen, no_std HID-bootcodec: keyboardreport van acht bytes,
maximaal zes usage-ID's, acht modifierbits, toestandvergelijking als sets,
en muisreports van drie tot acht bytes met drie knoppen en signed X/Y.
Geen tekst-/scancodeconversie, layout, timing, repeat of muiswiel raden.
Bootprotocol is niet het algemene HID-reportprotocol.

Een caller moet eerst op een geverifieerde HID-bootinterface bootprotocol
selecteren en exact de werkelijk ontvangen transferbytes aanbieden. Deze
code verricht geen selectiecommand, enumeration, discovery, I/O, DMA,
IRQ-configuratie, inputpublicatie of autorisatie. Device/lifetimebinding,
kernel-executioninterface, xHCI-enumeration en de OS-inputconsument ontbreken
nog. Parseruitvoer is geen inputcapability. Dit is **broncode + hostbewijs +
freestandingcompilatie**, geen USB-invoer in QEMU of fysieke hardware.

OEM-byte 1 van het keyboard wordt genegeerd, niet foutief tot nul verplicht.
Alle keyboard-usagebytes 4..255 blijven wirewaarden, niet alleen oude
BIOS-tekens. Nul is een lege slot; dubbele niet-nul usages worden geweigerd.
Een foutwaarde 1/2/3 in een slot markeert de gehele keyarray expliciet als
Rollover/PostFail/Undefined; daaruit worden nooit keyevents gesynthetiseerd.
Modifiers blijven bij deze keyboardstatus wel doorlopen, de laatst bekende
keyset blijft behouden tot een volgende geldige report of `clear`.
Bij inconsistente meerdere foutcodes wint de eerste; de keyarray blijft
volledig verworpen. `clear` geeft eenmaal alle laatst bekende releases.
Ongeldige lengte/duplicaten veranderen de staat niet.
Geen automatische releases op een parsefout: de caller moet deviceverlies,
reset en het einde van een lifetime expliciet afhandelen.

Muisbits boven bit 2 en extra bytes zijn OEM-data, geen extra knoppen/wiel.
X/Y blijven relatieve wirewaarden; geen GUI-coordinaten of acceleratie.
Waarde -128 wordt geweigerd wegens de bootdescriptorgrens -127..127.
Lengte- en inhoudscontrole gebeurt zonder allocatie, unsafe of externe deps.

## Bronnen

USB Implementers Forum, *Device Class Definition for Human Interface Devices
(HID)*, firmware specification versie 1.11, 27 mei 2001,
Appendix B pp. 59-61 en Appendix C pp. 62-63:
https://www.usb.org/sites/default/files/hid1_11.pdf

Primaire PDF daadwerkelijk gelezen; lokaal onderzoeksbestand SHA-256
`206272dacaf01ff34beda038b8ed59abbb02f028394896345c393bc026795d36`.
B.1 definieert de acht bytes, OEM-reserved en usage range 0..255; B.2
de mouseprefix, OEM-extensies en displacementgrens; Appendix C zegt dat
arrayvolgorde geen betekenis heeft en phantomreporting modifiers behoudt.
Geen tekst/code uit de specificatie overgenomen.

Routekaart: OSDev *USB Human Interface Devices*, W004063,
revisie 28137, 10 juli 2023. De primaire bootdescriptor is leidend.
Geen parser voor algemene HID-descriptors of stilzwijgende SET_PROTOCOL-aanname.

## Acceptatie

Vanuit de eigen worktree `/tmp/nieuw-os-driver-usb-hid`:

```sh
export PATH=/home/simon/.cargo/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export CC=/tmp/nieuw-os-c-toolchain/bin/cc
export CARGO_TARGET_DIR="$PWD/build/hid-boot-host"
cargo +1.90.0 test --offline --locked --manifest-path drivers/usb/Cargo.toml
cargo +1.90.0 fmt --manifest-path drivers/usb/Cargo.toml -- --check
cargo +1.90.0 clippy --offline --locked --manifest-path drivers/usb/Cargo.toml --all-targets -- -D warnings
cargo +1.90.0 build --offline --locked --manifest-path drivers/usb/Cargo.toml --release --target x86_64-unknown-none --lib
git diff --check
```

Alle vijf commando's exit 0. Zeven bestaande framing-unittests en zeven
nieuwe bootreporttests geslaagd, nul doctests. Geen tests genegeerd.
De matrix omvat 65.536 modifier/OEM-combinaties, iedere usage in ieder slot,
alle dubbele usage/slotparen, ontbrekende/extra transferbytes, phantomherstel,
alle muisknopbytes en displacementbytes per as, reorder/repeat en clear.

Logs onder `/tmp/nieuw-os-hid-boot-`, SHA-256:

| Suffix | SHA-256 |
|---|---|
| `test.log` | `1dbc331742caff8f96c88b24bc2a2fa29f9dadcf42f8512db68b9f93f568db1d` |
| `fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `clippy.log` | `c6d5d42f739fdf0a1fc24e56b43c70cdafbe44705a30d6c0cee9f39e1cfeeec2` |
| `none.log` | `c39bd7bf7f6f7eb8a13bcff3451ed38c39dc42696ebcca550e25dc5b2b6a5189` |

Native runtime/USB-deviceinput, fuzzing en menselijke acceptatie:
**niet uitgevoerd**. Geen nieuwe afhankelijkheden of tools geinstalleerd.
Geen centrale status/scorewrites; dit is geen nieuw geaccepteerd nativecriterium.
Queued USB-framingtests en hun bewijs zijn niet gewijzigd/overgenomen.
Rollback: verwijder uitsluitend de nieuwe bootmodule, export, eigen tests en
dit bewijsbestand; bestaande descriptorframing blijft ongewijzigd.
