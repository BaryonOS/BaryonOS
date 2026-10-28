# I03 — strikte native qsim-acceptatierunner

28 oktober 2026. Deze taak bezit uitsluitend `tools/run_qsim.py`,
`tests/test_qsim_harness.py` en deze bewijsmap. De rootintegrator beheert de
gedeelde bouw-, QEMU- en kernelhooks.

## Nieuw gedrag

`python3 tools/run_qsim.py` start de al gebouwde `native-qsim`-kernel via de
bestaande `run_guest` in een eigen map binnen `NOS_BUILD_DIR`. Het harnas
vereist een exclusieve quantumfeature, normale loader, fingerprintversie 3,
actuele bronhash en een exact overeenkomend scalar qsim-bouwrapport. Het
controleert de ELF-/disassemblyhashes, herhaalt de scalarinstructiegate en
controleert dat de geselecteerde kernel deze exacte qsim-ELF bevat.

De echte VM krijgt uitsluitend het vastgelegde q35/qemu64/TCG/1CPU/256MiB-
profiel, netwerk uit, private firmwarekopieën en een alleen-lezen ESP.
Andere flags, extra disks, een ander profiel of een artifact buiten de
private bouwmap worden niet geaccepteerd. De timeout is 60 seconden.

De uitkomst vereist rawstatus 33 en een exacte reeks van 28 regels na
`BOOT:EBS_OK`: boot/M2, manifest, de vier numerieke velden voor beide
processen, zes cases, timer/syscall/frameaantallen en de drie eindmarkers.
Numerieke velden zijn precies twintig ASCII-decimalen binnen u64.
PID's zijn 1 en 2, checkpointaantallen positief, nuttige arena 7.216 bytes,
werkafstand 448 en IRQ/syscall/werkelijk vrijgemaakte frames positief.
Dubbele, onbekende, extra, verschoven of misvormde regels worden geweigerd.

Het resultaat wordt alleen geaccepteerd als de bron vóór én na de run gelijk
blijft, de buildartifacts opnieuw kloppen, QEMU's entrybestand ongewijzigd
blijft en de bewaarde ELF-/bouwrapportsnapshots exact aan de build gebonden
zijn. Native bron, gebruikers-ELF en bron van de checker hebben eigen hashes.

## Werkelijk uitgevoerd

**15 hosttests slagen.** Zij controleren de parser en bron-/artifactbinding,
inclusief false-greengevallen voor markers, numeriekformaat, nulvoortgang,
verkeerd profiel, foutstatus/time-out, verdwenen hashes, gewijzigd kernel-/
gebruikersbestand, verkeerde embedded ELF, afwijkende seriële bytes, bron-
wijziging tijdens een gemockte run en dubbele JSON-sleutels. Pythoncompilatie
en diff-whitespacecontrole slagen. Opdrachten en bronhashes: `checks.json`.

`observed-qsim-serial.txt` is de uitvoer van de eerdere **werkelijke native
rootrun** `native-qsim-directed-1b55f345b4`, uitsluitend hergebruikt als
parserfixture. De herkomst en seriële hash staan in `fixture-origin.json`.
Dit hergebruik is geen nieuwe VM-run en geen I03-nativeacceptatie. De
hosttests maken geen goedgekeurd gastbewijs aan.

## Native reproduceren op de gecombineerde bron

Gebruik een eigen build-/targetmap binnen de checkout; het bestaande
`run_guest` verwacht die relatie. Selecteer QEMU expliciet via
`NOS_QEMU_MACHINE`; er is geen automatische fallback.

```bash
export NOS_BUILD_DIR="$PWD/build/qsim-acceptance"
export NOS_TARGET_DIR="$PWD/target/qsim-acceptance"
python3 tools/build.py --kernel-feature native-qsim
python3 tools/run_qsim.py
```

De run produceert `native-qsim-result.json`, een eigen `run/native-qsim-*`
met echte seriële uitvoer en firmware-/ESP-hashes, plus een eigen
`qsim-inputs-*` met de bewaarde gebruikers-ELF, scalarrapport en disassembly.
Het hoofdagent moet deze runner op de gecombineerde bron uitvoeren en dat
bewijs afzonderlijk opslaan. De ontwikkeltoolchain en scalar-port staan in
I01; de minimale kernelobserver en diens negatieve hostcontroles in I02.
