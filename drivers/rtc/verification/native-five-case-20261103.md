# DRV-RTC-NATIVE-FIVE-CASE

Werkelijke VM-test, 3 november 2026, bron
`d10b5cdaf56774a39e3ef10463f674c6188a463a`, branch
`copilot/driver-rtc-native-gate`. Alleen dit bewijsbestand toegevoegd.
Bronfingerprint v3:
`623faf4ac6baef8dd8eb5300c4b69c5ce63d56a7d34623a4d3a13ac604993794`.
Manifest SHA-256:
`9cc05743b425879d629ebfd0ab7da2b8c1e3bc42c6a31345f846514a9060d7a1`.

Vanuit `/tmp/nieuw-os-driver-rtc-native`, bestaande tools, geen installatie:

```sh
export PATH=/home/simon/.cargo/bin:/tmp/sm-oom-qemu-root/usr/bin:/tmp/nieuw-os-c-toolchain/bin:$PATH
export LD_LIBRARY_PATH=/tmp/sm-oom-qemu-root/usr/lib/x86_64-linux-gnu
export CC=/tmp/nieuw-os-c-toolchain/bin/cc
export OS_OVMF_CODE=/tmp/sm-oom-qemu-root/usr/share/OVMF/OVMF_CODE_4M.fd
export OS_OVMF_VARS=/tmp/sm-oom-qemu-root/usr/share/OVMF/OVMF_VARS_4M.fd
export NOS_QEMU_MACHINE=pc-q35-8.2
export NOS_BUILD_DIR="$PWD/build/rtc-native"
export NOS_TARGET_DIR="$PWD/target/rtc-native"
export CARGO_TARGET_DIR="$PWD/target/rtc-host"
export CARGO_NET_OFFLINE=true
python3 -B tools/build.py --kernel-feature native-rtc
python3 -B -u tools/run_rtc.py --case all
git diff --check
```

Alle exit 0. De bestaande gate controleert vijf echte guestuitvoeringen,
bron-/artifactbinding, de onafhankelijke kalenderberekening, negatieve
drivergevallen, indexherstel en expliciet `UnauthenticatedRtc`.

| Case | Waargenomen UTC | Unixseconden | Tijd | Raw exit |
|---|---|---:|---:|---:|
| millennium | 2000-01-01 00:00:01 | 946684801 | 1.9643 s | 33 |
| leap | 2000-02-29 12:34:57 | 951827697 | 1.9405 s | 33 |
| current | 2026-10-30 12:00:01 | 1793361601 | 1.8544 s | 33 |
| y2038 | 2038-01-19 03:14:09 | 2147483649 | 1.9565 s | 33 |
| future | 2096-02-29 00:00:01 | 3981312001 | 1.8408 s | 33 |

Alle cases passed/artifact_binding_passed true en timed_out false.
QEMU meldt niet-fataal ontbrekend `kvmvapic.bin` op stderr; dit is niet
weggelaten of als afwezige waarschuwing voorgesteld. De bestaande gate
accepteert deze uitvoering op de echte guestmarkers en exitcode.

Bewijs onder `build/rtc-native/rtc-report.json`, inclusief alle vijf
guestresultaten/serial/runpaden:
SHA-256 `434d5e15f87fcc512211314a008a73ab2727cbc4ba2143387e2f14db7e2d7895`.
Volledige buildlog `/tmp/nieuw-os-driver-rtc-native-build.log`:
`73ecdd9de537a3be68fad48c4fa51e90a43176ed280230ed03b0f519399aee23`.
Volledige gatelog `/tmp/nieuw-os-driver-rtc-native-gate.log`:
`71ede6f05412cf41c43e1ab58d0f7fe88ed733671ea78130b1c96edf805490e0`.

Bewijsgrens: native CMOS-polling in QEMU, niet fysieke RTC/NMI-concurrency,
geauthenticeerde tijd of gebruikersinterface. `epoch`/`century` zijn bestaande
firmwarediagnostiek, geen onderdelen van `--case all`, hier niet uitgevoerd.
De extra hostfouttests op `5de45bc9` zaten niet in deze bron en krijgen geen
VM-bewijs van deze run. Fysieke/menselijke tests niet uitgevoerd. Geen
productcriterium gesloten; centrale beoordeling bij integrator.
Revert verwijdert alleen dit bestand; binaries/caches blijven buiten Git.
