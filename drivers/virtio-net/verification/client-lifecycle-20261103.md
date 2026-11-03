# DRV-NET-CLIENT-LIFECYCLE

Werkelijke VM-uitvoering 3 november 2026 op
`d10b5cdaf56774a39e3ef10463f674c6188a463a`, branch
`copilot/driver-net-client-lifecycle`. Alleen dit bewijsbestand toegevoegd;
geen driver-, kernel-, SDK-, service- of testcode gewijzigd.
Bronfingerprint v3:
`623faf4ac6baef8dd8eb5300c4b69c5ce63d56a7d34623a4d3a13ac604993794`.

Vanuit `/tmp/nieuw-os-driver-net-client`, bestaande omgeving, geen installatie:

```sh
export PATH=/home/simon/.cargo/bin:/tmp/sm-oom-qemu-root/usr/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export LD_LIBRARY_PATH=/tmp/sm-oom-qemu-root/usr/lib/x86_64-linux-gnu
export CC=/tmp/nieuw-os-c-toolchain/bin/cc
export OS_OVMF_CODE=/tmp/sm-oom-qemu-root/usr/share/OVMF/OVMF_CODE_4M.fd
export OS_OVMF_VARS=/tmp/sm-oom-qemu-root/usr/share/OVMF/OVMF_VARS_4M.fd
export NOS_QEMU_MACHINE=pc-q35-8.2
export NOS_TARGET_DIR="$PWD/target/client-native"
export CARGO_TARGET_DIR="$PWD/target/client-host"
export CARGO_NET_OFFLINE=true
NOS_BUILD_DIR="$PWD/build/client-normal" python3 -B -u tools/run_network_service.py --case normal
NOS_BUILD_DIR="$PWD/build/client-exit" python3 -B -u tools/run_network_service.py --case client-exit
git diff --check
```

Alle exit 0. De bestaande runner bouwt de brongebonden profielen en controleert
echte serial, ELF-/firmwarehashes en het onafhankelijke peertranscript.

| Case | Tijd | Raw exit | Gate | TX submitted/completed | RX completed/delivered | Reset/ACK |
|---|---:|---:|---|---:|---:|---:|
| normal | 6.9323 s | 33 | true | 45/45 | 35/35 | 1/1 |
| client-exit | 6.7167 s | 33 | true | 25/25 | 18/18 | 1/1 |

Beide cases: 2 denials en 0 TX-timeouts. In client-exit verlaat APP_A het
systeem op tick 31, APP_B pas op tick 463 na zijn eigen geteste netwerkwerk.
De peer observeert respectievelijk 45/25 clientframes en 35/18 serverframes.
Geen van die peerantwoorden vervangt de eigen NIC-, TCP-/UDP- of
proceslevenscyclusimplementatie.

Onder `build/client-normal/`: `network-service-normal-report.json`,
SHA-256 `f69833bc02ef0e5394e50e5abe04e1aed201a76b889748fde8eb4cf5299267a4`.
Onder `build/client-exit/`: `network-service-client-exit-report.json`,
SHA-256 `cbe4173a0039cc21f402925fb523e9bc2e7fce8e29cbbb8346da68f60f3c4b3b`.
Identieke `result.json`-kopieen staan in de respectievelijke runmappen:
`network-service-normal-4bbcb627d0d1411d9c47ab0e81c864d7` en
`network-service-client-exit-6ce5293209e54cd1ac67139688601348`.

| Runbestand | Normal SHA-256 | Client-exit SHA-256 |
|---|---|---|
| `serial.txt` | `b8cc1ceb164642556648ab5b81b5881bc01de0aa7944610620f54f7444347728` | `69ea5596b559b77c687440a01a5cd7384a1802e15a8503a475e0764897fc6d2d` |
| `network-peer.json` | `6b4525c5eeb44e564823e878fbcd954c25bd973b740f52f8e1608f81e02310d8` | `2b0abc294fe76e088cef14b5ea75cdb6fcf50c3b62c2299172e7cc8717129345` |

Volledige logs: `/tmp/nieuw-os-driver-net-client-normal.log`,
SHA-256 `bdf710b3ad2757b47794449ff3454515befd7399eaa8746b27699c0e009d12a9`;
`/tmp/nieuw-os-driver-net-client-exit.log`,
SHA-256 `1c8838e540e5b03b6ff073ab73420a6af518a59f48d18964d239b8e1142bcd4c`.

Bewijsgrens: bestaande virtuele NIC met localhost-tegenpartij en bestaande
ring-3-clientlifetimes, geen fysieke NIC, algemene internetpolicy, TLS,
IOMMU-isolatie of nieuwe reset-failure-injectie. Geen VM-bewijs voor losse
hosttestheads die niet in deze bron zaten. Fysieke/menselijke tests niet
uitgevoerd; geen nieuwe acceptatiepunten. Centrale beoordeling bij integrator.
Revert verwijdert alleen dit bewijsbestand; binaries/caches blijven buiten Git.
