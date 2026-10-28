# I02 — native qsim-kernelobserver

28 oktober 2026. Bouwt voort op I01-commit
`5ea1fe127252ee3c1894bb976e2768751bad4244`. Alleen
`kernel/src/m2/m3/qsim.rs` en deze bewijsmap zijn eigendom van deze taak.
Featureselectie, ELF-koppeling, M3-dispatch en native QEMU-harnas blijven
bij de hoofdagent; zij zijn niet door deze deelagent gewijzigd.

De observer beoordeelt twee gewone, afzonderlijke ring-3-processen met de
ongewijzigde bestaande qsim-engine. Hij voert zelf geen quantumberekening uit
en voegt geen syscall, capability, appbeleid, FPU-toestand of allocator toe.

## Acceptatiegrens

`manifest(&mut State)` vereist twee geldige eigen adresruimten met lege
rapporten, verschillende PID's/patronen/report-/heapframes en een eigen RW/NX
rapportpagina. `poll(&mut State)` kopieert per proces uitsluitend veertien
u64-velden (112 bytes) naar kernel-eigen snapshots terwijl IF=0 en de
permanente kerneladresruimte/servicestack actief zijn.

Een onverwachte exit/fault, verkeerde procestoestand, rapportfout,
patroonwijziging, afwijkende referentie-uitkomst of 2.000 gastticks leidt tot
`QSIM:FAIL:*` en debug-exit `0x11` (QEMU-rawstatus 35). Zolang een proces nog
bezig is of geen echte timer-IRQ is geobserveerd, kiest de observer een
volgend gebruikersproces; de gewone M3-proef kan deze run niet goedkeuren.

Beide terminale rapporten moeten het volledige masker 127, Bell4096
2082/2014, zes-qubit count63=4096, nuttige arena 7.216 bytes, werkafstand
448, volledige arena 65.536 bytes en echte core-uitputtingsfout tonen.
Checkpointvoortgang, geselecteerde processen en timer-IRQ's moeten bestaan.
Dit is bewijs voor de vastgebonden testbinary, geen algemene appautorisatie.

Vóór opruimen controleert de bestaande `State::audit` opnieuw eigendom,
verschillende frames en effectieve paginarechten. Daarna maakt de bestaande
cleanup de echte gebruikersframes, kernelstacks en paginatabellen vrij. De
getelde toename moet exact overeenkomen met alle owned frames, en de
vrijeframestand moet de bij het manifest vastgelegde verwachting bereiken.
Pas daarna volgen `QSIM:CLEANUP:PASS`, `QSIM:PASS`, `M1:PASS` en exit `0x10`.

## Uitgevoerde controle op dit checkpoint

`observer_host.rs` compileert de **werkelijke observermodule**, met uitsluitend
voor deze hosttest vervangende MMU-/geheugen-/seriële adapters. Zes tests
slagen: goede paarresultaten en cleanup; bezig zijnde peer/ontbrekende timer;
mutatie van elk rapportveld voor beide processen; fout-/exit-/deadlinegevallen;
ongeldig beginmanifest; en framebalans-/cleanuplekken. Deze hostshim is geen
native MMU-, proces- of boottest. Exacte commando's en bronhashes staan in
`checks.json`.

```bash
rustc --edition 2021 --test docs/verification/parallel-i02/observer_host.rs -o /tmp/nieuw-os-i02-observer-tests
/tmp/nieuw-os-i02-observer-tests
```

Native compilatie en QEMU-uitvoering zijn op dit checkpoint nog de
integratietaak. Geen roadmapcriterium wordt uitsluitend op deze hosttests
gesloten. De opt-in feature kan worden weggelaten om de bestaande M3-route
zonder quantumobserver te blijven gebruiken.
