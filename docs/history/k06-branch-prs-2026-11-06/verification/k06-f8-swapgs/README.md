# K06 auditbevinding F8: `swapgs`-discipline voor het GS-anker (ADR-076)

**Datum:** 6 november 2026. **Branch:** `fix/k06-f8-swapgs`, gestapeld op
`fix/k06-f1-f2-lifo-oncpu` (`5dccf290`). **Bewijsniveau:** native QEMU 10.2.1/OVMF/TCG op de
WSL-host. Geen CI, geen fysieke hardware, review door AI. **Score:** 41 → 41/100. K06 blijft open.

## Rood eerst (commit `c2f1bed0`, fixture op de oude stubs)
`red/`: bronfingerprint `1b1c6766…` (v3), QEMU raw exit 35, de oracle weigert:
```
SMP:GS:CR4=104
SMP:GS:HIJACK
M2:FAIL:KERNEL_EXCEPTION
```
Ring 3 laadt GS (0x1b/null). De 0xf0-stub volgt `gs:[0]` naar VA 0 en krijgt een
supervisor-#PF (cr2 < 4096). Commit 1 bevatte nog geen fix; zie `red/SHA256SUMS`. De regel
`commit=` in `red/RUN.txt` noemt de pre-amend-ID `d9efdf25`. De bron is identiek: dezelfde
fingerprint als `c2f1bed0`.

## Groen (commit `91e84b75`, de fix)
`green/native-report.json`, bronfingerprint `fe473124…`, schone werkboom:
- **21/21 SMP-gevallen over 16 profielen**: `native-smp` 6/6, alle bestaande negatieve en
  controleprofielen (waaronder `ap-nmi-ist`, `ap-double-fault-ist`, `double-run`), plus
  N1 `SMP:GS:PASS:CPUS=4`, N2 `SMP:GS:IRET_FIXUP:PASS` en N3 `SMP:FAULT:AP_NMI_IST:PASS`
  (seriallogs in `green/`).
- **M3/K05-regressie** (`m2_exception_common` en `m3_restore` zijn veranderd):
  `tools/test_milestones.py --through m2` geslaagd, `tools/test_supervisor.py` 6/6, `native-fpu`
  en `test-fpu-wait` geslaagd.
- **Mutant:** zonder de GS-7-labelvergelijking faalt N2 (`M2:FAIL:KERNEL_EXCEPTION`). N2 meet dus
  de fixup.
- **Statisch (enige bewijs voor GS-5):** `tools/tests/test_gs_discipline.py` 7/7. Op de bron
  van commit 1 faalt hij (7 fouten en 2 errors).

De commit daarna wijzigt alleen een ADR-nummer in commentaar en documentatie.

## Grenzen
- N2 gebruikt SS=0x13. QEMU-TCG controleert een niet-canonieke RIP bij `iretq` niet (gemeten:
  #GP vanuit CPL3, RIP 0x8000…).
- VA 0 is niet als user-pagina gemapt. Rood is daarom de #PF in de handler, niet een schrijfactie
  in een schildwachtpagina.
- TCG modelleert geen speculatie. Het AMD-null-selectorgedrag is niet getest.
- Kernel-clippy `-D warnings` faalt al vóór deze wijziging op `paging.rs`, `main.rs` en
  `serial.rs`. Er is geen bevinding in `arch.rs`/`smp.rs`.

## Commando's
```
export NOS_TARGET_DIR="$PWD/target/smp-native" NOS_BUILD_DIR="$PWD/build/k06-<profiel>"
python3 -B tools/build.py --kernel-feature <profiel>
python3 -B -u tools/run_smp.py --profile <profiel>
python3 -B -m unittest discover -s tools/tests -p test_gs_discipline.py -v
```
