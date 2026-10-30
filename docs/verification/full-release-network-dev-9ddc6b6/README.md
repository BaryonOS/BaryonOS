# Native ARP/UDP/DNS-herhaling op 9ddc6b6

Datum: 30 oktober 2026. Branch `simonvanieperen-musical-happiness`.
Broncommit `9ddc6b6df82fc555ea6d28247a971ea1b09a8b99`.
Bronfingerprint v3:
`f5c4968216a35c4fab9f85d3c9c34729c5e667cf5a0066561f067ba5c452789c`.

De bestaande netwerkproef is uitgevoerd door de eigen kernel en een echt
ring-3-proces. De hostpeer was alleen Ethernet-tegenpartij en foutinjector.
Er is geen DHCP-client, TLS, algemene internetroute of appbestemmingsbroker
toegevoegd of bewezen. N01–N04 blijven open; netwerk 0/10, totaal 34/100.

## Uitvoering

| Commando | Exit | Uitkomst |
|---|---:|---|
| `python3 -B tools/preflight.py --strict` | 0 | Matching OVMF en QEMU gevonden |
| `python3 -B -u tools/test_net_system.py` | 0 | Hostgates en twee echte VM-runs groen; 77,51 s totaal |
| `python3 -B tools/capture_system.py --network --destination docs/verification/full-release-network-dev-9ddc6b6` | 0 | Beide runs opnieuw gecontroleerd op raw artifacts, counters, peertranscript en image |

Normal: raw exit 33, `network_gate=true`, 32,859 s.
Koude recovery: raw exit 33, `network_gate=true`, 32,498 s.
De schijf na normal en voor/na recovery heeft SHA-256
`6f6268c11f1ceba867e7c00f71faeabd2b3100b00a4e044c6e7a24efe275aab2`.
De finale normale kernel is opnieuw gebouwd en hashgelijk aan de geteste
normale kernel: `502ff0a1b8802ec941511a651d22e8a95572f39690aae1f53c37a7ab553c97e5`.
De capturebestanden in deze map bevatten de feitelijke tegenpartij- en
kernelwaarnemingen; binaries en schijfimages blijven buiten Git.

## Lokale ontwikkelomgeving

Officiële Ubuntu Resolute-pakketten zijn via `apt-get download` uitsluitend
naar `/tmp/nieuw-os-qemu-102/` gedownload en met `dpkg-deb -x` uitgepakt:
QEMU `1:10.2.1+ds-1ubuntu3.3`, OVMF Generic `2025.11-3ubuntu7.3`,
SeaBIOS `1.17.0-1ubuntu1` en benodigde runtimebibliotheken. Geen globale
installatie. QEMU toont werkelijk `pc-q35-10.0`; TCG, één CPU, 256 MiB en
alleen de tijdelijke loopback-Ethernetpeer zijn gebruikt.

`OS_OVMF_CODE` en `OS_OVMF_VARS` wijzen naar dezelfde uitgepakte OVMF-build:

- CODE: `874f15a7c9110e78a89db159f0f68fc97e9076b74777520de1c235426f758aa7`
- VARS-template: `5d2ac383371b408398accee7ec27c8c09ea5b74a0de0ceea6513388b15be5d1e`

`PATH` bevat `/tmp/nieuw-os-qemu-102/root/usr/bin`;
`LD_LIBRARY_PATH` bevat diens `usr/lib/x86_64-linux-gnu` en de eerdere
GCC-sysroot. `NOS_BUILD_DIR=build/network-dev`,
`NOS_TARGET_DIR=target/network-dev`, `CARGO_TARGET_DIR=target/network-dev`
(absoluut ingesteld). De hostlinkerwrapper in `/tmp/nieuw-os-qemu-102/host-cc`
roept GCC 13 aan met `--sysroot=/tmp/nieuw-os-dhcp-hostcc/root`.
`GCC_EXEC_PREFIX`, `COMPILER_PATH` en `LIBRARY_PATH` wijzen naar die sysroot.
Dit betreft uitsluitend hostbuilds; de native kernel gebruikt geen host-libc.

Eerste poging faalde vóór de VM op ontbrekende gepinde `aead`-cache;
`cargo +1.90.0 fetch --locked --manifest-path sdk/Cargo.toml` en
`cargo +1.90.0 fetch --locked` herstelden de bestaande cache. Tweede poging
faalde op ontbrekende host-`libc_nonshared.a`; de expliciete sysrootwrapper
herstelde de buildscriptlinker. Geen mislukte poging is als geslaagd geteld.
Een eerste capture met een niet-toegestane mapnaam werd terecht geweigerd;
de bovenstaande capture met `full-release-`-prefix slaagde.

Volledige logs: `/tmp/nieuw-os-qemu-102/native-network-hostcc.log`,
`capture-network.log` en `preflight.json`. Raw artifacts en de machineleesbare
gate staan onder `build/network-dev/`.
