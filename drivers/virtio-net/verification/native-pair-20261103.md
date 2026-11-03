# DRV-NET-NATIVE-RESET-ACCEPTANCE

Werkelijk uitgevoerde VM-proef, 3 november 2026, broncommit
`d10b5cdaf56774a39e3ef10463f674c6188a463a`, branch
`copilot/driver-net-native-acceptance`. Geen productie- of testcode gewijzigd.
De afzonderlijke nieuwe hosttests op `ddf52070` en `89aa6709` zijn niet
in deze bron opgenomen en krijgen geen native bewijs van deze uitvoering.

Bronfingerprint versie 3:
`623faf4ac6baef8dd8eb5300c4b69c5ce63d56a7d34623a4d3a13ac604993794`.
Definitieve normale kernel:
`2df623742571bbcd9d798c6e3bd73b11cad29de4885811d7a48354d8cedb582f`.

## Uitvoering

Vanuit `/tmp/nieuw-os-driver-net-native`, uitsluitend bekende bestaande
tools/firmware, geen installatie:

```sh
export PATH=/home/simon/.cargo/bin:/tmp/sm-oom-qemu-root/usr/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export LD_LIBRARY_PATH=/tmp/sm-oom-qemu-root/usr/lib/x86_64-linux-gnu
export CC=/tmp/nieuw-os-c-toolchain/bin/cc
export OS_OVMF_CODE=/tmp/sm-oom-qemu-root/usr/share/OVMF/OVMF_CODE_4M.fd
export OS_OVMF_VARS=/tmp/sm-oom-qemu-root/usr/share/OVMF/OVMF_VARS_4M.fd
export NOS_BUILD_DIR="$PWD/build/driver-net-native"
export NOS_TARGET_DIR="$PWD/target/driver-net-native"
export CARGO_TARGET_DIR="$PWD/target/driver-net-host"
export CARGO_NET_OFFLINE=true
python3 -B -u tools/test_net_system.py
```

Exit 0. De bestaande gate draait hostpeer-, capability-core- en SDK-tests,
bouwt beide profielen, draait de echte QEMU-pair en vergelijkt de opnieuw
gebouwde normale kernel. Alle drie hostcommando's exit 0.
QEMU versie: 10.2.1 (Debian 1:10.2.1+ds-1ubuntu3.3).
Een eerdere losse `python3 -B -u tools/run_net_system.py --case normal`
gaf exit 1 doordat het buildmanifest nog ontbrak; daarbij geen guest gestart.
De bovenstaande volledige gate heeft die prerequisite zelf correct gebouwd.

| VM-stage | Tijd | Raw exit | Systeemgate | Netwerkgate |
|---|---:|---:|---|---|
| normal | 45.8715 s | 33 | true | true |
| recover | 44.1365 s | 33 | true | true |

Beide stages: TX submitted/completed 8, RX completed/delivered 7,
reset/ACK 1, TX-timeouts 0, denials 7, ARP-/DNS-afwijzingen ieder 2.
Onafhankelijke lokale peer: werkelijk 8 aanvragen ontvangen en 7 antwoorden
verstuurd; ARP 3, UDP-echo 1, DNS 4 waarvan 1 bewust onbeantwoord.
Recovery: 0 diskwrites en 0 flushes.

Normale image voor uitvoering:
`49c40c316234e2e4e4c8dd681a057b1b36611648896dcacba1794263b711c266`.
Na normal en voor/na recovery:
`7730c041609c20e646c197decde43bcd2255dba2fa6754f64fd098932f518a23`.

## Bewijsbestanden

Volledige uitvoer: `/tmp/nieuw-os-driver-net-native-pair.log`,
SHA-256 `0b23c63045df934fab686552bcc64e3dc9258f647798716d025fe89fda6101ad`.
Onder `build/driver-net-native/`:

| Bestand | SHA-256 |
|---|---|
| `net-system-gate.json` | `429415ffa1054f8beb0cb751b8a2fb525f19951060005bcd153094b0bdae62f7` |
| `manifest.json` | `330d6347b29d7df57c2984054055e21e661a02772ad267b28e3ce3ab9d525493` |
| `run/system-normal-c40bdea82f1c49d581e250980462243e/result.json` | `7e418c8524eebbbde0c1d10802da11bdb2797370fa4bcc81999267270d9f3682` |
| `run/system-normal-c40bdea82f1c49d581e250980462243e/serial.txt` | `c12461b3f4bf913cb09a0bfb1e529d906461078cfa5506e76779f3127725e6ba` |
| `run/system-recover-06807dad48f54e1e9a116afaac1b69d9/result.json` | `02a447f0f119ee2ba647ea6010464c69a826cac7c08b89e519c15f926852cd62` |
| `run/system-recover-06807dad48f54e1e9a116afaac1b69d9/serial.txt` | `6ba68c143fe2bcace431fdb0a4a960d443ce12fa0b7c3887227715e07fa12101` |

Beide `network-peer.json`:
`1199837668d75958576a52b990d188474f349d2dbc776474767cc815a2ed6fa4`.
Beide `stderr.txt`:
`b5319852fac78e3f4765c32bd54b3fcc78efbab607f40404c38870a9c269dddf`.
Gegenereerde images/binaries blijven buiten Git.

## Grens

Dit is het bestaande normale netwerk-/koudeherstelprofiel met echte virtuele
NIC-I/O, geen extra native injectie van late completions of geweigerde reset.
De proef bewijst geen fysieke NIC, IOMMU-isolatie, menselijke bruikbaarheid,
NVMe-/xHCI-execution of volledige release. Alleen localhost-testpeer en
wegwerpimage; geen externe dienst of hostdisk. Geen nieuw scorecriterium
gesloten; centraal assessment blijft bij integrator. Revert van dit
bewijsbestand wijzigt geen productiegedrag.
