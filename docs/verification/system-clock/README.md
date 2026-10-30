# NATIVE-CLOCK-1 — ring-3 UTC-clockproef

Uitgevoerd op 30 oktober 2026 op broncommit
`baeb09aece05a6fb65bfa7178ae1c146d5506b75` (implementatiecommit
`652cd5d4c9c68afb0b143c0147197aa7fc6f8220`). Het opt-in profiel
`native-system-clock` bootte de eigen x86_64/UEFI-kernel en bestaande
systeemdiensten in een disposable QEMU-VM. De echte ring-3-systeemapp riep
syscall `0x73` tweemaal aan en testte een niet-nulargument.

## Resultaat

| Controle | Uitkomst |
|---|---|
| Runner | exit 0; `passed=true`; geen timeout; ruwe QEMU-exitcode 33 |
| Guestprofiel | QEMU 10.2.1, `pc-q35-8.2`, TCG single-thread, `qemu64`, 1 CPU, 256 MiB |
| Expliciete virtuele RTC-basis | `2026-10-30T12:00:00Z` |
| Ring-3 UTC Unixseconden | `1793361601`, `1793361601` (beide basis +1 seconde) |
| Boot-monotone PIT-ticks rond de reads | `1`, `1` (niet-regressief; geen kalenderconversie) |
| Niet-nulargument | app accepteerde uitsluitend de verwachte `-22`; marker `SYSTEM:CLOCK:INVALID_ARGS:PASS` |
| SDK-kwaliteit | `UnauthenticatedRtc`; de waarde is geen TLS-/certificaat- of vertrouwde tijd |

De native runner verifieerde de verwachte markers, exacte aantallen, beide
UTC-waarden binnen 60 gastseconden van de vaste basis, hun volgorde en
monotone tickvolgorde. De volledige seriële uitvoer bevat de markers in het
hashgebonden log hieronder. De disposable volume-image is niet in Git bewaard.

## Exacte acceptatieopdracht

```sh
. .tools/nieuw-os/rtc-env.sh
PATH="$PWD/.tools/nieuw-os:$PATH" python3 -B -u tools/run_system_clock.py
```

De runner bouwde en inspecteerde kernel, loader en drie systeem-ELF's en
startte de bestaande echte systeemketen. Gerichte verificatie:

```sh
python3 -B -m unittest tools.tests.test_system_clock
PATH="$PWD/.tools/nieuw-os:$PATH" python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime
```

Beide hostcommando's eindigden eerder op deze bronstap met exit 0:
respectievelijk 3 verifier-tests en 20 SDK-runtime-tests. Deze tests zijn
geen vervanging voor de native QEMU-run.

## Hashbinding

`evidence.json` bevat de bronfingerprint v3, firmware- en ELF-hashes en
SHA-256-hashes van de originele gegenereerde runnerreport, manifest en
seriële uitvoer. Die ruwe bestanden blijven buiten Git onder
`build/rtc-native/`; `SHA256SUMS` bindt de compacte, gecommitteerde rapporten.
De runnerreport zelf bevat absolute lokale buildpaden en is daarom niet
gekopieerd.

## Bewijsgrens

Dit toont uitsluitend deze QEMU-firmware-/machineconfiguratie en opt-in
single-CPU IF=0-profiel. Het authenticeert de RTC niet en bewijst geen
persistente antirollback, NTP, suspendcorrectie, certificaatvalidatie,
SMP-serialisatie, fysieke hardwareondersteuning of volledige FR-S03/FR-N03-
acceptatie. De vaste productroadmap blijft **34%**; deze deelproef sluit geen
volledig criterium.
