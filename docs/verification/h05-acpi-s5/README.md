# H05-stap — native ACPI S5-poweroff door de eigen kernel

**Branch:** `hw/h05-native-acpi-s5`, op main `c2c29c5f`. **Datum:** 5 november
2026. **Bewijsniveau:** native QEMU/TCG (`pc-q35-8.2`, QEMU 10.2.1, OVMF);
geen fysieke hardware. **Score:** 41 → 41/100. H05 blijft open: de gate eist
meting op ondersteunde apparatuur, slaap/wek en thermische/batterijfuncties.

## Gerealiseerd

Tot nu toe had de S5-parser `drivers/acpi-power` geen enkele aanroeper; de
eigen OS kon zichzelf niet uitzetten. Met het opt-in kernelprofiel
`native-acpi-s5` gebeurt dat nu volledig met eigen code:

1. `boot-protocol`-discovery legt het adres van de enige `FACP`-root-entry vast
   (`Discovery.fadt`). Bij een dubbele FADT wordt dat adres 0 en is er geen
   S5-bron; de MCFG-, DMAR- en MADT-inventaris blijven ongemoeid.
2. In dezelfde eenmalige discovery (BSP, IF=0) leest de kernel FADT (≤ 4 KiB)
   en DSDT (≤ 64 KiB) via de bestaande `FirmwareReader`. Die laat alleen
   UEFI ACPI-reclaim/NVS-geheugen toe.
3. `parse_s5` valideert signatures, lengtes, checksums, de FADT→DSDT-binding,
   de PM1-registers en de AML-`_S5_`-subset.
4. Pas na een geldig plan volgt de enige port-I/O: PM1a lezen, rapporteren,
   `SLP_TYP|SLP_EN` schrijven (PM1b eerst als die bestaat). Wat geen geldig
   plan oplevert, rapporteert `UNAVAILABLE`, `UNREADABLE` of
   `REJECTED:<fout>`, doet geen port-write en eindigt met de normale
   debug-exit.

Er is geen syscall en geen IPC-route; geen proces kan een poweroff aanvragen.

## Orakel

- QEMU eindigt alleen met exit 0 bij een guest-poweroff. De runner vervangt
  het standaard-`-no-reboot` door `-action reboot=reset` (`reboot_on_reset`
  in `tools/run.py`). Lokaal is gecontroleerd dat een `system_reset` met
  alleen `-no-reboot` QEMU met rc 0 beëindigt, en met de override de VM laat
  doorlopen. Een reset of triple fault levert dus een tweede `BOOT:EBS_OK`
  op en wordt afgewezen.
- De verwachte poort `0x0604` volgt uit de vaste q35-ICH9-PMIO-basis
  `0x600 + 4`, niet uit de parser. Dezelfde waarde staat in de eerdere
  QMP-firmwarecapture van [`drivers/acpi-power/verification`](../../../drivers/acpi-power/verification/README.md).
- De seriële staart moet exact overeenkomen; extra uitvoer na `POWER:S5:WRITE`
  of een write in het afwijzingsprofiel laat de run falen.

## Native gevallen (7/7 geslaagd, geen time-out)

| Profiel | Runs | Exit | Exacte staart na `M1:PASS` |
|---|---:|---:|---|
| `native-acpi-s5` | 5 | 0 (poweroff) | `POWER:S5:PM1A=0x0604:SLP_TYP=0:SCI_EN=1`, `POWER:S5:WRITE` |
| `test-acpi-s5-corrupt-dsdt` | 2 | 33 (debug-exit) | `POWER:S5:REJECTED:InvalidChecksum` |

In het negatieve profiel wordt één byte in de kernelkopie van de DSDT
omgedraaid (de laatste byte). Exit 33 in plaats van 0 bewijst dat er geen PM1-write is gedaan.
Duur, hashes, commando's en volledige seriële uitvoer staan in
[native-report.json](native-report.json); beide rapporten zijn gebonden aan
bronvingerafdruk v3
`6cf56d7512de876251d2f1fe0253d5f817e13fe4ecf57d91ccc3a092057613e6`.

## Commando's en regressie

```bash
python3 -B tools/build.py --kernel-feature native-acpi-s5
python3 -B tools/run_power.py --repeat 5
NOS_BUILD_DIR=build/corrupt python3 -B tools/build.py --kernel-feature test-acpi-s5-corrupt-dsdt
NOS_BUILD_DIR=build/corrupt python3 -B tools/run_power.py --repeat 2
```

CI draait dit in `.github/workflows/native-power.yml`. Hostresultaten:
`boot-protocol` 29 tests, waaronder de nieuwe FADT-/duplicaattest;
`acpi-power` 22 + 2. De root-hostsuite uit `ci.yml` met 413 tests en Clippy
`-D warnings` geeft exit 0. `tools/tests` 208/208 (`test_power` 3 nieuw),
fmt is schoon. De profielen `milestone-m2`, `milestone-m6c` en `native-smp`
bouwen. Kernel-Clippy meldt alleen bestaande punten in `paging.rs` en
`main.rs`, niets in de nieuwe code.

## Open

- SCI_EN stond al op 1 (OVMF). ACPI-mode via `SMI_CMD`/`ACPI_ENABLE`
  inschakelen is niet geïmplementeerd. Op hardware met SCI_EN=0 is de
  write ongetest.
- AML wordt alleen binnen de bestaande `_S5_`-subset gelezen; er is geen
  `_PTS`/`_TTS`-uitvoering. Dat is passend voor QEMU, niet algemeen.
- Er is geen gebruikers- of dienstgestuurd afsluiten: dat vereist een
  niet-delegeerbare poweroff-capability met flush van diensten (S08/volume)
  vóór de write.
- Slaap/wek (S3), thermische en batterijfuncties en fysieke apparatuur
  ontbreken; H05 en H06 blijven open.
